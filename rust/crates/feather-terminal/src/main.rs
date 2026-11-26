//! FeatherTerminal：MengOS 的终端窗口（GTK4 + VTE4）。
//!
//! 窗口里是标签页：每个标签一个独立的 VTE 终端、一个独立的 shell 进程；上方有搜索栏，
//! 右键有菜单。与界面无关的纯逻辑（标签标题、正则转义）放在 `model` 模块里，便于单元测试。

mod model;
mod settings;
mod sidebar;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};

use gtk::gio;
use gtk::glib;
use gtk::glib::translate::IntoGlib;
use gtk::prelude::*;
use vte4::prelude::*;

use model::{escape_regex, title_from_path};
use sidebar::Sidebar;

/// 编译时由 CI 注入的版本号（例如 `0.0.1+31`）；本地构建没有注入时显示 `dev`。
const BUILD_VERSION: &str = match option_env!("MENGOS_BUILD") {
    Some(version) => version,
    None => "dev",
};

/// VTE 的搜索接口要传 PCRE2 的 flags（`vte4::Regex::for_search` 的第二个参数）。
/// 这里只开 UTF / UCP（按字符而不是按字节匹配）和多行模式。
const PCRE2_UTF: u32 = 0x0008_0000;
const PCRE2_UCP: u32 = 0x0002_0000;
const PCRE2_MULTILINE: u32 = 0x0000_0400;
const SEARCH_FLAGS: u32 = PCRE2_UTF | PCRE2_UCP | PCRE2_MULTILINE;

/// 依次尝试的 shell：`/usr/bin/zsh` → 环境变量 `$SHELL` → `/bin/bash`。
fn shell_candidates() -> Vec<String> {
    let mut candidates: Vec<String> = Vec::new();

    if PathBuf::from("/usr/bin/zsh").exists() {
        candidates.push(String::from("/usr/bin/zsh"));
    }

    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.is_empty() && PathBuf::from(&shell).exists() && !candidates.contains(&shell) {
            candidates.push(shell);
        }
    }

    if !candidates.contains(&String::from("/bin/bash")) {
        candidates.push(String::from("/bin/bash"));
    }

    candidates
}

/// 一个标签页：一个独立的 VTE 终端 + 一个独立的 shell 进程。
struct Tab {
    /// 标签编号。信号回调里只带这个编号，不捕获终端对象，避免引用环让 shell 无法被回收。
    id: u64,
    terminal: vte4::Terminal,
    title_label: gtk::Label,
    /// shell 进程的 PID，来自 `spawn_async` 的回调。
    pid: Cell<Option<i32>>,
    /// 最近一次读到的当前目录。
    cwd: RefCell<Option<PathBuf>>,
}

impl Tab {
    /// 当前目录。数据来源是 `/proc/<pid>/cwd` 符号链接的目标（由每秒一次的定时器刷新）。
    fn current_dir(&self) -> Option<PathBuf> {
        self.cwd.borrow().clone()
    }
}

/// 窗口上的所有动作。快捷键通过 application accelerator 绑到这些动作上。
struct Actions {
    new_tab: gio::SimpleAction,
    close_tab: gio::SimpleAction,
    next_tab: gio::SimpleAction,
    prev_tab: gio::SimpleAction,
    toggle_sidebar: gio::SimpleAction,
    copy: gio::SimpleAction,
    paste: gio::SimpleAction,
    select_all: gio::SimpleAction,
    reset: gio::SimpleAction,
    toggle_search: gio::SimpleAction,
}

impl Actions {
    fn new() -> Actions {
        Actions {
            new_tab: gio::SimpleAction::new("new-tab", None),
            close_tab: gio::SimpleAction::new("close-tab", None),
            next_tab: gio::SimpleAction::new("next-tab", None),
            prev_tab: gio::SimpleAction::new("prev-tab", None),
            toggle_sidebar: gio::SimpleAction::new("toggle-sidebar", None),
            copy: gio::SimpleAction::new("copy", None),
            paste: gio::SimpleAction::new("paste", None),
            select_all: gio::SimpleAction::new("select-all", None),
            reset: gio::SimpleAction::new("reset", None),
            toggle_search: gio::SimpleAction::new("toggle-search", None),
        }
    }

    fn all(&self) -> [&gio::SimpleAction; 10] {
        [
            &self.new_tab,
            &self.close_tab,
            &self.next_tab,
            &self.prev_tab,
            &self.toggle_sidebar,
            &self.copy,
            &self.paste,
            &self.select_all,
            &self.reset,
            &self.toggle_search,
        ]
    }
}

/// 整个窗口共享的状态。
struct State {
    window: gtk::ApplicationWindow,
    /// 左右分栏：左边是标签页终端，右边是侧栏。
    paned: gtk::Paned,
    notebook: gtk::Notebook,
    sidebar: Rc<Sidebar>,
    /// 侧栏宽度与显示状态（读写 `~/.config/mengos/feather-terminal.toml`）。
    settings: RefCell<settings::SidebarSettings>,
    search_bar: gtk::SearchBar,
    search_entry: gtk::SearchEntry,
    search_status: gtk::Label,
    paste_action: gio::SimpleAction,
    tabs: RefCell<Vec<Rc<Tab>>>,
    next_tab_id: Cell<u64>,
}

thread_local! {
    /// 保住 `State`。所有信号回调都只持 `Weak<State>`（避免引用环），
    /// 如果这里不存一个强引用，`build_ui` 一返回 `State` 就被回收，
    /// 定时器和动作处理器会全部变成空操作。
    static STATE: RefCell<Option<Rc<State>>> = const { RefCell::new(None) };
}

fn main() {
    let application = gtk::Application::new(Some("com.mengos.FeatherTerminal"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

/// 快捷键：用 GTK 的 accelerator 绑到窗口动作上（比自己在窗口上接按键事件可靠）。
fn install_accels(application: &gtk::Application) {
    let bindings = [
        ("win.new-tab", "<Control><Shift>t"),
        ("win.close-tab", "<Control><Shift>w"),
        ("win.toggle-sidebar", "<Control><Shift>b"),
        ("win.toggle-search", "<Control><Shift>f"),
        ("win.copy", "<Control><Shift>c"),
        ("win.paste", "<Control><Shift>v"),
        ("win.next-tab", "<Control>Page_Down"),
        ("win.prev-tab", "<Control>Page_Up"),
    ];
    for (action, accel) in bindings {
        application.set_accels_for_action(action, &[accel]);
    }
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    let title = format!("FeatherTerminal {BUILD_VERSION}");
    window.set_title(Some(&title));
    window.set_default_size(1000, 650);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // 搜索栏：默认隐藏，Ctrl+Shift+F 显示。
    let search_bar = gtk::SearchBar::new();
    search_bar.set_show_close_button(true);
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some("搜索终端内容：回车下一个，Shift+回车上一个"));
    search_entry.set_hexpand(true);
    let search_status = gtk::Label::new(None);
    search_status.add_css_class("error");
    let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    search_box.append(&search_entry);
    search_box.append(&search_status);
    search_bar.set_child(Some(&search_box));
    search_bar.connect_entry(&search_entry);
    root.append(&search_bar);

    let notebook = gtk::Notebook::new();
    notebook.set_vexpand(true);
    notebook.set_scrollable(true);

    // 右侧侧栏：6 个页面 + 左下角一排图标按钮。
    let sidebar = Sidebar::build();
    // 最小宽度 260：gtk::Paned 不会把子控件压到它的最小宽度以下
    sidebar
        .widget()
        .set_size_request(settings::SIDEBAR_MIN_WIDTH, -1);
    let saved_settings = settings::load();

    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_start_child(Some(&notebook));
    paned.set_end_child(Some(sidebar.widget()));
    paned.set_resize_start_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);
    paned.set_vexpand(true);
    // 窗口默认宽 1000，先按「1000 - 侧栏宽度」摆好分隔线；之后记住用户拖动后的宽度
    paned.set_position((1000 - saved_settings.width).max(200));
    sidebar.widget().set_visible(saved_settings.visible);
    root.append(&paned);

    window.set_child(Some(&root));

    let actions = Actions::new();
    let state = Rc::new(State {
        window: window.clone(),
        paned: paned.clone(),
        notebook: notebook.clone(),
        sidebar: sidebar.clone(),
        settings: RefCell::new(saved_settings),
        search_bar: search_bar.clone(),
        search_entry: search_entry.clone(),
        search_status: search_status.clone(),
        paste_action: actions.paste.clone(),
        tabs: RefCell::new(Vec::new()),
        next_tab_id: Cell::new(1),
    });
    STATE.with(|slot| *slot.borrow_mut() = Some(Rc::clone(&state)));

    // 点侧栏按钮后把键盘焦点还给终端，用户可以直接继续打字
    let state_weak = Rc::downgrade(&state);
    sidebar.connect_page_activated(move || {
        if let Some(state) = state_weak.upgrade() {
            focus_current_terminal(&state);
        }
    });

    // 关窗前把侧栏宽度和显示状态写回配置文件
    let state_weak = Rc::downgrade(&state);
    window.connect_close_request(move |_| {
        if let Some(state) = state_weak.upgrade() {
            save_settings(&state);
        }
        glib::Propagation::Proceed
    });

    for action in actions.all() {
        window.add_action(action);
    }
    wire_actions(&state, &actions);
    install_accels(application);
    install_search(&state);

    // 切换标签时：焦点回到终端，窗口标题、侧栏目录和搜索词跟着换。
    let state_weak = Rc::downgrade(&state);
    notebook.connect_switch_page(move |_, _, _page| {
        if let Some(state) = state_weak.upgrade() {
            focus_current_terminal(&state);
            update_window_title(&state);
            update_sidebar_cwd(&state);
            apply_search(&state);
        }
    });

    start_cwd_timer(&state);
    new_tab(&state);
    window.present();
}

/// 把动作接到具体操作上。
fn wire_actions(state: &Rc<State>, actions: &Actions) {
    let state_weak = Rc::downgrade(state);
    actions.new_tab.connect_activate(move |_, _| {
        if let Some(state) = state_weak.upgrade() {
            new_tab(&state);
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.close_tab.connect_activate(move |_, _| {
        if let Some(state) = state_weak.upgrade() {
            close_current_tab(&state);
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.next_tab.connect_activate(move |_, _| {
        if let Some(state) = state_weak.upgrade() {
            switch_tab(&state, 1);
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.prev_tab.connect_activate(move |_, _| {
        if let Some(state) = state_weak.upgrade() {
            switch_tab(&state, -1);
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.toggle_sidebar.connect_activate(move |_, _| {
        if let Some(state) = state_weak.upgrade() {
            toggle_sidebar(&state);
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.toggle_search.connect_activate(move |_, _| {
        if let Some(state) = state_weak.upgrade() {
            toggle_search(&state);
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.copy.connect_activate(move |_, _| {
        if let Some(tab) = state_weak.upgrade().and_then(|state| current_tab(&state)) {
            tab.terminal.copy_clipboard_format(vte4::Format::Text);
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.paste.connect_activate(move |_, _| {
        if let Some(tab) = state_weak.upgrade().and_then(|state| current_tab(&state)) {
            tab.terminal.paste_clipboard();
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.select_all.connect_activate(move |_, _| {
        if let Some(tab) = state_weak.upgrade().and_then(|state| current_tab(&state)) {
            tab.terminal.select_all();
        }
    });

    let state_weak = Rc::downgrade(state);
    actions.reset.connect_activate(move |_, _| {
        if let Some(tab) = state_weak.upgrade().and_then(|state| current_tab(&state)) {
            tab.terminal.reset(true, true);
        }
    });
}

/// 搜索框的三种事件：输入变化 → 重新搜；回车 → 下一个；Shift+回车 → 上一个；
/// 搜索栏关闭 → 焦点还给终端。
fn install_search(state: &Rc<State>) {
    let state_weak = Rc::downgrade(state);
    state.search_entry.connect_search_changed(move |_| {
        if let Some(state) = state_weak.upgrade() {
            apply_search(&state);
        }
    });

    let state_weak = Rc::downgrade(state);
    state.search_entry.connect_activate(move |_| {
        if let Some(state) = state_weak.upgrade() {
            find_in_current_tab(&state, true);
        }
    });

    // SearchEntry 的 activate 信号不带修饰键信息，Shift+回车 要自己接按键事件。
    let key_controller = gtk::EventControllerKey::new();
    let state_weak = Rc::downgrade(state);
    key_controller.connect_key_pressed(move |_, keyval, _keycode, modifier| {
        let shift = modifier.contains(gtk::gdk::ModifierType::SHIFT_MASK);
        if shift && keyval == gtk::gdk::Key::Return {
            if let Some(state) = state_weak.upgrade() {
                find_in_current_tab(&state, false);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    state.search_entry.add_controller(key_controller);

    let state_weak = Rc::downgrade(state);
    state
        .search_bar
        .connect_search_mode_enabled_notify(move |bar| {
            if !bar.is_search_mode() {
                if let Some(state) = state_weak.upgrade() {
                    focus_current_terminal(&state);
                }
            }
        });
}

/// 把搜索框里的文字当成普通文本（先转义成正则）交给当前标签的 VTE 去搜。
fn apply_search(state: &Rc<State>) {
    let Some(tab) = current_tab(state) else {
        return;
    };
    let text = state.search_entry.text().to_string();
    if text.is_empty() {
        tab.terminal.search_set_regex(None, 0);
        mark_not_found(state, false);
        return;
    }

    let pattern = escape_regex(&text);
    match vte4::Regex::for_search(&pattern, SEARCH_FLAGS) {
        Ok(regex) => {
            tab.terminal.search_set_regex(Some(&regex), 0);
            let found = tab.terminal.search_find_next();
            mark_not_found(state, !found);
        }
        Err(error) => {
            eprintln!("搜索表达式无效：{error}");
            mark_not_found(state, true);
        }
    }
}

/// 在当前标签里找下一个/上一个匹配。
fn find_in_current_tab(state: &Rc<State>, forward: bool) {
    let Some(tab) = current_tab(state) else {
        return;
    };
    let found = if forward {
        tab.terminal.search_find_next()
    } else {
        tab.terminal.search_find_previous()
    };
    mark_not_found(state, !found);
}

/// 找不到时把搜索框标红并显示「未找到」。
fn mark_not_found(state: &Rc<State>, not_found: bool) {
    if not_found {
        state.search_entry.add_css_class("error");
        state.search_status.set_text("未找到");
    } else {
        state.search_entry.remove_css_class("error");
        state.search_status.set_text("");
    }
}

/// 显示/隐藏搜索栏。
fn toggle_search(state: &Rc<State>) {
    let show = !state.search_bar.is_search_mode();
    state.search_bar.set_search_mode(show);
    if show {
        state.search_entry.grab_focus();
        apply_search(state);
    } else {
        focus_current_terminal(state);
    }
}

/// 新建一个标签页：独立的 VTE 终端 + 独立的 shell。
fn new_tab(state: &Rc<State>) -> Rc<Tab> {
    let terminal = vte4::Terminal::new();
    terminal.set_vexpand(true);
    terminal.set_hexpand(true);

    let title_label = gtk::Label::new(Some("终端"));
    // 只设 ellipsize 时 GTK 会把自然宽度算成省略号的宽度，标签会被压成「…」，
    // 所以必须再给一个字符宽度（最少 12 个字符宽，超出的部分才省略）。
    title_label.set_width_chars(12);
    title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_label.set_max_width_chars(20);

    let close_button = gtk::Button::from_icon_name("window-close-symbolic");
    close_button.set_has_frame(false);
    close_button.set_focusable(false);
    close_button.set_tooltip_text(Some("关闭这个标签页（Ctrl+Shift+W）"));

    let tab_label = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    tab_label.append(&title_label);
    tab_label.append(&close_button);

    let id = state.next_tab_id.get();
    state.next_tab_id.set(id + 1);

    let tab = Rc::new(Tab {
        id,
        terminal: terminal.clone(),
        title_label: title_label.clone(),
        pid: Cell::new(None),
        cwd: RefCell::new(None),
    });
    state.tabs.borrow_mut().push(tab.clone());

    let page = state.notebook.append_page(&terminal, Some(&tab_label));
    state.notebook.set_current_page(Some(page));

    // 标签上的关闭按钮：只关这一个标签。
    let state_weak = Rc::downgrade(state);
    close_button.connect_clicked(move |_| {
        if let Some(state) = state_weak.upgrade() {
            close_tab(&state, id);
        }
    });

    // 这个标签里的 shell 退出（例如输入 exit）：也只关这一个标签。
    let state_weak = Rc::downgrade(state);
    terminal.connect_child_exited(move |_, _status| {
        if let Some(state) = state_weak.upgrade() {
            close_tab(&state, id);
        }
    });

    install_terminal_menu(state, &terminal);
    spawn_shell(&terminal, Rc::downgrade(&tab));

    terminal.grab_focus();
    tab
}

/// 关闭指定编号的标签页；关掉最后一个标签时关闭窗口。
fn close_tab(state: &Rc<State>, id: u64) {
    let Some(index) = state.tabs.borrow().iter().position(|tab| tab.id == id) else {
        return;
    };
    let tab = state.tabs.borrow_mut().remove(index);
    state.notebook.remove_page(Some(index as u32));
    // 丢掉最后一个 Rc 之后终端会被回收，VTE 关闭 pty，shell 收到 SIGHUP 退出。
    drop(tab);

    if state.tabs.borrow().is_empty() {
        state.window.close();
    } else {
        focus_current_terminal(state);
        update_window_title(state);
    }
}

/// 关闭当前标签。
fn close_current_tab(state: &Rc<State>) {
    if let Some(tab) = current_tab(state) {
        close_tab(state, tab.id);
    }
}

/// 循环切换标签（Ctrl+PageUp 是 -1，Ctrl+PageDown 是 +1）。
fn switch_tab(state: &Rc<State>, delta: i32) {
    let pages = state.notebook.n_pages();
    if pages == 0 {
        return;
    }
    let current = state.notebook.current_page().unwrap_or(0) as i32;
    let next = (current + delta).rem_euclid(pages as i32) as u32;
    state.notebook.set_current_page(Some(next));
    focus_current_terminal(state);
    update_window_title(state);
}

/// 当前标签（标签顺序与 `state.tabs` 的顺序一致）。
fn current_tab(state: &Rc<State>) -> Option<Rc<Tab>> {
    let page = state.notebook.current_page()?;
    state.tabs.borrow().get(page as usize).cloned()
}

fn focus_current_terminal(state: &Rc<State>) {
    if let Some(tab) = current_tab(state) {
        tab.terminal.grab_focus();
    }
}

/// 窗口标题带上当前标签的目录，方便看出「当前目录跟踪」是活的。
fn update_window_title(state: &Rc<State>) {
    let title = match current_tab(state).and_then(|tab| tab.current_dir()) {
        Some(dir) => format!("FeatherTerminal {BUILD_VERSION} — {}", dir.display()),
        None => format!("FeatherTerminal {BUILD_VERSION}"),
    };
    state.window.set_title(Some(&title));
}

/// 把当前标签的目录同步给侧栏的 6 个页面。
fn update_sidebar_cwd(state: &Rc<State>) {
    let dir = current_tab(state).and_then(|tab| tab.current_dir());
    state.sidebar.set_current_dir(dir.as_deref());
}

/// 显示/隐藏侧栏（Ctrl+Shift+B）。隐藏后终端占满窗口。
fn toggle_sidebar(state: &Rc<State>) {
    let visible = !state.sidebar.widget().is_visible();
    state.sidebar.widget().set_visible(visible);
    state.settings.borrow_mut().visible = visible;
    // 立刻存一次：即使用户之后异常退出，显示状态也不会丢
    save_settings(state);
    focus_current_terminal(state);
}

/// 把「侧栏宽度 + 是否显示」写进 `~/.config/mengos/feather-terminal.toml`。
fn save_settings(state: &Rc<State>) {
    let mut current = state.settings.borrow_mut();
    current.visible = state.sidebar.widget().is_visible();

    let window_width = state.window.width();
    if window_width > 0 {
        let sidebar_width = window_width - state.paned.position();
        current.width =
            sidebar_width.clamp(settings::SIDEBAR_MIN_WIDTH, settings::SIDEBAR_MAX_WIDTH);
    }

    settings::save(&current);
}

/// 每个终端一份右键菜单：新建标签页 / 复制 / 粘贴 / 全选 / 清屏。
fn install_terminal_menu(state: &Rc<State>, terminal: &vte4::Terminal) {
    let menu = gio::Menu::new();
    menu.append(Some("新建标签页"), Some("win.new-tab"));
    menu.append(Some("复制"), Some("win.copy"));
    menu.append(Some("粘贴"), Some("win.paste"));
    menu.append(Some("全选"), Some("win.select-all"));
    menu.append(Some("清屏"), Some("win.reset"));

    let popover = gtk::PopoverMenu::from_model(Some(&menu));
    popover.set_parent(terminal);
    popover.set_has_arrow(false);

    // 右键把菜单弹在鼠标位置。用捕获阶段，避免 VTE 自己先吃掉点击。
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
    gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
    let popover_for_click = popover.clone();
    gesture.connect_pressed(move |gesture, _n_press, x, y| {
        let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
        popover_for_click.set_pointing_to(Some(&rect));
        popover_for_click.popup();
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });
    terminal.add_controller(gesture);

    // 弹出前检查剪贴板：没有文本就把「粘贴」置灰。
    let state_weak = Rc::downgrade(state);
    popover.connect_show(move |_| {
        let Some(state) = state_weak.upgrade() else {
            return;
        };
        let action = state.paste_action.clone();
        let clipboard = state.window.clipboard();
        glib::spawn_future_local(async move {
            let has_text = matches!(clipboard.read_text_future().await, Ok(Some(_)));
            action.set_enabled(has_text);
        });
    });
}

/// 用 VTE 的 `spawn_async` 在伪终端里启动真实的 shell，并把 PID 记到这个标签上。
fn spawn_shell(terminal: &vte4::Terminal, tab: Weak<Tab>) {
    let shell = match shell_candidates().into_iter().next() {
        Some(shell) => shell,
        None => {
            eprintln!("找不到可用的 shell");
            return;
        }
    };
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/"));

    // VTE 的 envv 参数会替换整个环境，所以显式传当前环境，避免 shell 没有 PATH/HOME。
    let env: Vec<String> = std::env::vars().map(|(k, v)| format!("{k}={v}")).collect();
    let envv: Vec<&str> = env.iter().map(String::as_str).collect();
    let argv = [shell.as_str()];
    let shell_for_log = shell.clone();

    terminal.spawn_async(
        vte4::PtyFlags::DEFAULT,
        Some(home.as_str()),
        &argv,
        &envv,
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        None::<&gio::Cancellable>,
        move |result| match result {
            Ok(pid) => {
                if let Some(tab) = tab.upgrade() {
                    // `spawn_async` 给的是 `glib::Pid`，转成 i32 方便拼 `/proc/<pid>/cwd`。
                    tab.pid.set(Some(pid.into_glib()));
                }
            }
            Err(error) => eprintln!("启动 {shell_for_log} 失败：{error}"),
        },
    );
}

/// 每秒刷新一次每个标签的当前目录（数据来源：`/proc/<pid>/cwd`），并更新窗口标题。
fn start_cwd_timer(state: &Rc<State>) {
    let state_weak = Rc::downgrade(state);
    glib::timeout_add_seconds_local(1, move || {
        let Some(state) = state_weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        let tabs: Vec<Rc<Tab>> = state.tabs.borrow().clone();
        for tab in &tabs {
            refresh_cwd(tab);
        }
        update_window_title(&state);
        update_sidebar_cwd(&state);
        glib::ControlFlow::Continue
    });
}

/// 读取 `/proc/<pid>/cwd` 的符号链接目标，得到 shell 当前所在目录。
///
/// 这是一次 `readlink` 系统调用，不读取也不解析 `/proc` 下的文本文件，
/// 因此放在 1 秒定时器里不会阻塞界面。
fn read_proc_cwd(pid: i32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/cwd")).ok()
}

fn refresh_cwd(tab: &Rc<Tab>) {
    let Some(pid) = tab.pid.get() else {
        return;
    };
    let Some(path) = read_proc_cwd(pid) else {
        // 读不到（例如 shell 刚退出）就保持上一次的值。
        return;
    };
    // 只在目录真的变了才更新标题（current_dir 也是后续侧栏任务要用的读取接口）。
    if tab.current_dir().as_deref() == Some(path.as_path()) {
        return;
    }
    *tab.cwd.borrow_mut() = Some(path.clone());
    tab.title_label.set_text(&title_from_path(&path));
    tab.title_label
        .set_tooltip_text(Some(&path.display().to_string()));
}
