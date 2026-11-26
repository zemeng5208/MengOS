//! 右侧侧栏：一个 `gtk::Stack`（6 个页面）+ 左下角一排 6 个图标按钮。
//!
//! 每个页面一个子模块（`changes`、`network`、`files`、`assistant`、`profile`、`vault`），
//! 子模块各自提供「创建页面」和「当前目录变化时调用」两个函数；本模块只负责
//! 拼装、切换页面，以及把当前目录分发给每个页面。

pub mod assistant;
pub mod changes;
pub mod files;
pub mod network;
pub mod profile;
pub mod vault;

use std::path::Path;
use std::rc::Rc;

use gtk::prelude::*;

/// 一页侧栏：标题 + 「当前目录」那一行 + 占位内容。
pub struct PageView {
    pub root: gtk::Box,
    pub cwd_label: gtk::Label,
}

/// 一个页面模块的元信息，以及它的两个入口函数。
struct PageModule {
    id: &'static str,
    title: &'static str,
    icon: &'static str,
    build: fn() -> PageView,
    set_current_dir: fn(&PageView, Option<&Path>),
}

/// 页面顺序固定：文件更改、网络连接、文件树、终端助手、个人资料、密码库。
fn page_modules() -> [PageModule; 6] {
    [
        PageModule {
            id: changes::ID,
            title: changes::TITLE,
            icon: changes::ICON,
            build: changes::build,
            set_current_dir: changes::set_current_dir,
        },
        PageModule {
            id: network::ID,
            title: network::TITLE,
            icon: network::ICON,
            build: network::build,
            set_current_dir: network::set_current_dir,
        },
        PageModule {
            id: files::ID,
            title: files::TITLE,
            icon: files::ICON,
            build: files::build,
            set_current_dir: files::set_current_dir,
        },
        PageModule {
            id: assistant::ID,
            title: assistant::TITLE,
            icon: assistant::ICON,
            build: assistant::build,
            set_current_dir: assistant::set_current_dir,
        },
        PageModule {
            id: profile::ID,
            title: profile::TITLE,
            icon: profile::ICON,
            build: profile::build,
            set_current_dir: profile::set_current_dir,
        },
        PageModule {
            id: vault::ID,
            title: vault::TITLE,
            icon: vault::ICON,
            build: vault::build,
            set_current_dir: vault::set_current_dir,
        },
    ]
}

/// 侧栏整体。
pub struct Sidebar {
    root: gtk::Box,
    buttons: Vec<gtk::ToggleButton>,
    pages: Vec<PageView>,
    modules: [PageModule; 6],
}

impl Sidebar {
    /// 建出侧栏。返回 `Rc` 是因为按钮的 `toggled` 回调要和侧栏共用同一个 `gtk::Stack`。
    pub fn build() -> Rc<Sidebar> {
        let modules = page_modules();

        let stack = gtk::Stack::new();
        stack.set_vexpand(true);
        stack.set_hexpand(true);
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        let mut pages = Vec::with_capacity(modules.len());
        for module in &modules {
            let page = (module.build)();
            stack.add_named(&page.root, Some(module.id));
            pages.push(page);
        }

        // 左下角一排图标按钮：同一时间只有一个按下（用 `set_group` 保证）。
        let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        button_row.set_halign(gtk::Align::Start);
        button_row.set_margin_top(6);
        button_row.set_margin_bottom(6);
        button_row.set_margin_start(6);
        button_row.set_margin_end(6);

        let mut buttons: Vec<gtk::ToggleButton> = Vec::with_capacity(modules.len());
        for (index, module) in modules.iter().enumerate() {
            let button = gtk::ToggleButton::new();
            button.set_child(Some(&gtk::Image::from_icon_name(module.icon)));
            button.set_tooltip_text(Some(module.title));
            button.set_active(index == 0);
            button_row.append(&button);

            // 同一时间只有一个按下：后面的按钮加入第一个按钮的组。
            if let Some(first) = buttons.first() {
                button.set_group(Some(first));
            }

            let stack_for_toggle = stack.clone();
            let id = module.id;
            button.connect_toggled(move |button| {
                if button.is_active() {
                    stack_for_toggle.set_visible_child_name(id);
                }
            });

            buttons.push(button);
        }

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.append(&stack);
        root.append(&button_row);

        let sidebar = Rc::new(Sidebar {
            root,
            buttons,
            pages,
            modules,
        });
        sidebar.set_current_dir(None);
        sidebar
    }

    /// 侧栏根控件（放进 `gtk::Paned` 的右侧）。
    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    /// 切换标签或 `cd` 之后调用：把当前目录写进每一页的「当前目录」那一行。
    pub fn set_current_dir(&self, dir: Option<&Path>) {
        for (module, page) in self.modules.iter().zip(self.pages.iter()) {
            (module.set_current_dir)(page, dir);
        }
    }

    /// 点击任一页面按钮后回调（用来把键盘焦点还给终端）。
    ///
    /// 用 `GestureClick` 而不是 `clicked` 信号：GTK4 的 `ToggleButton` 只有 `toggled`，
    /// 而点击「已经按下的那个按钮」不会触发 `toggled`，这时也要把焦点还给终端。
    pub fn connect_page_activated<F: Fn() + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        for button in &self.buttons {
            let callback = Rc::clone(&callback);
            let gesture = gtk::GestureClick::new();
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            gesture.connect_released(move |_, _, _, _| callback());
            button.add_controller(gesture);
        }
    }
}

/// 建一页占位内容。T13–T16 会把对应模块里的这个函数换成真正的界面。
fn build_placeholder_page(title: &str, placeholder: &str) -> PageView {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);

    let title_label = gtk::Label::new(None);
    title_label.set_markup(&format!("<b>{title}</b>"));
    title_label.set_halign(gtk::Align::Start);
    root.append(&title_label);

    let cwd_label = gtk::Label::new(None);
    cwd_label.set_halign(gtk::Align::Start);
    cwd_label.set_xalign(0.0);
    // 和标签页标题同理：只设 ellipsize 会被压成「…」，必须给最小字符宽度。
    cwd_label.set_width_chars(18);
    cwd_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    cwd_label.add_css_class("dim-label");
    cwd_label.set_tooltip_text(Some("当前标签的当前目录（数据来自 /proc/<pid>/cwd）"));
    root.append(&cwd_label);

    let placeholder_label = gtk::Label::new(Some(placeholder));
    placeholder_label.set_vexpand(true);
    placeholder_label.set_halign(gtk::Align::Center);
    placeholder_label.set_valign(gtk::Align::Center);
    placeholder_label.set_wrap(true);
    placeholder_label.add_css_class("dim-label");
    root.append(&placeholder_label);

    PageView { root, cwd_label }
}

/// 把当前目录写进某一页的「当前目录」标签（6 个页面模块共用）。
fn set_page_cwd(view: &PageView, dir: Option<&Path>) {
    let text = match dir {
        Some(path) => format!("当前目录：{}", path.display()),
        None => String::from("当前目录：未知"),
    };
    view.cwd_label.set_text(&text);
    view.cwd_label.set_tooltip_text(Some(&text));
}
