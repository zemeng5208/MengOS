//! 侧栏第 3 页：文件树与预览（T13）。
//!
//! - 上方：文件树（`gtk::ListView` + `gtk::TreeListModel`，展开文件夹时才后台读它的内容）
//! - 下方：预览区（文本前 200KB / 图片缩略 / 二进制提示 / 具体错误）
//! - 双击文件夹只改文件树的显示，不改终端目录；顶部会出现「正在浏览：…」和「回到终端目录」
//!
//! 读目录、读文件、查 MIME 类型都在后台线程做（`std::thread` + `async-channel`），
//! 主线程只负责把结果填进控件。

mod menu;
mod model;
mod preview;
mod tree;

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use super::{set_page_cwd, DirCallback, PageView};

pub const ID: &str = "files";
pub const TITLE: &str = "文件树";
/// 图标名由 CI 在 `/usr/share/icons` 下核实存在（见 `rust.yml` 的图标检查步骤）。
pub const ICON: &str = "folder-symbolic";

/// 文件树与预览共用的状态。
struct FilesState {
    /// 终端当前目录（来自 T11 的 `/proc/<pid>/cwd` 跟踪）
    terminal_dir: RefCell<Option<PathBuf>>,
    /// 文件树当前显示的目录
    browse_dir: RefCell<Option<PathBuf>>,
    /// 是否正在浏览别的目录（双击文件夹之后为真）
    browsing_elsewhere: Cell<bool>,
    show_hidden: Rc<Cell<bool>>,
    tree: tree::FileTree,
    preview: preview::PreviewPane,
    menu: Rc<menu::FileMenu>,
    banner: gtk::Box,
    banner_label: gtk::Label,
    back_button: gtk::Button,
    truncated_label: gtk::Label,
}

pub fn build() -> PageView {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);

    let title_label = gtk::Label::new(None);
    title_label.set_markup("<b>文件树</b>");
    title_label.set_halign(gtk::Align::Start);
    root.append(&title_label);

    let cwd_label = gtk::Label::new(None);
    cwd_label.set_halign(gtk::Align::Start);
    cwd_label.set_xalign(0.0);
    // 只设 ellipsize 会被压成「…」，必须同时给最小字符宽度
    cwd_label.set_width_chars(18);
    cwd_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    cwd_label.add_css_class("dim-label");
    cwd_label.set_tooltip_text(Some("终端当前目录（数据来自 /proc/<pid>/cwd）"));
    root.append(&cwd_label);

    // 工具条：显示隐藏文件 / 刷新 / 回到终端目录
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let hidden_switch = gtk::Switch::new();
    hidden_switch.set_valign(gtk::Align::Center);
    hidden_switch.set_tooltip_text(Some("显示以点开头的隐藏文件"));
    let hidden_label = gtk::Label::new(Some("显示隐藏文件"));
    hidden_label.add_css_class("dim-label");
    let refresh_button = gtk::Button::with_label("刷新");
    refresh_button.set_tooltip_text(Some("重新读取当前目录"));
    let back_button = gtk::Button::with_label("回到终端目录");
    back_button.set_tooltip_text(Some("文件树跟随终端目录"));
    back_button.set_visible(false);
    toolbar.append(&hidden_switch);
    toolbar.append(&hidden_label);
    toolbar.append(&refresh_button);
    toolbar.append(&back_button);

    // 「正在浏览：…（与终端目录不同）」
    let banner = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    banner.set_visible(false);
    let banner_label = gtk::Label::new(None);
    banner_label.set_wrap(true);
    banner_label.set_xalign(0.0);
    banner_label.add_css_class("dim-label");
    banner.append(&banner_label);

    // 「已截断」/「读不了目录」提示
    let truncated_label = gtk::Label::new(None);
    truncated_label.set_wrap(true);
    truncated_label.set_xalign(0.0);
    truncated_label.add_css_class("error");
    truncated_label.set_visible(false);

    let preview = preview::PreviewPane::build();
    let menu = menu::FileMenu::build(preview.menu_button());
    let show_hidden = Rc::new(Cell::new(false));

    // 用 `new_cyclic`：文件树的回调需要「指向 state 自己的弱引用」，
    // 而 state 里又要放文件树控件。
    let state = Rc::new_cyclic(|weak| {
        let on_select: tree::EntryCallback = {
            let weak = weak.clone();
            Rc::new(move |entry: &model::DirEntryInfo| {
                if let Some(state) = weak.upgrade() {
                    preview_file(&state, entry);
                }
            })
        };

        let on_activate: tree::EntryCallback = {
            let weak = weak.clone();
            Rc::new(move |entry: &model::DirEntryInfo| {
                if !entry.is_dir {
                    return;
                }
                if let Some(state) = weak.upgrade() {
                    browse_into(&state, &entry.path);
                }
            })
        };

        let on_menu: tree::EntryCallback = {
            let weak = weak.clone();
            Rc::new(move |entry: &model::DirEntryInfo| {
                if let Some(state) = weak.upgrade() {
                    preview_file(&state, entry);
                    state.menu.popup();
                }
            })
        };

        let load_children: tree::LoadChildren = {
            let show_hidden = Rc::clone(&show_hidden);
            Rc::new(move |path: &Path| load_children_store(&show_hidden, path))
        };

        let tree = tree::build(tree::Callbacks {
            on_select,
            on_activate,
            on_menu,
            load_children,
        });

        FilesState {
            terminal_dir: RefCell::new(None),
            browse_dir: RefCell::new(None),
            browsing_elsewhere: Cell::new(false),
            show_hidden,
            tree,
            preview,
            menu,
            banner,
            banner_label,
            back_button,
            truncated_label,
        }
    });

    // 刷新按钮
    let state_weak = Rc::downgrade(&state);
    refresh_button.connect_clicked(move |_| {
        if let Some(state) = state_weak.upgrade() {
            reload_current(&state);
        }
    });

    // 显示隐藏文件开关
    let show_hidden_for_switch = Rc::clone(&state.show_hidden);
    let state_weak = Rc::downgrade(&state);
    hidden_switch.connect_active_notify(move |switch| {
        show_hidden_for_switch.set(switch.is_active());
        if let Some(state) = state_weak.upgrade() {
            reload_current(&state);
        }
    });

    // 回到终端目录
    let state_weak = Rc::downgrade(&state);
    state.back_button.connect_clicked(move |_| {
        if let Some(state) = state_weak.upgrade() {
            back_to_terminal(&state);
        }
    });

    // 预览区右上角的「···」按钮
    let menu_for_button = Rc::clone(&state.menu);
    state
        .preview
        .menu_button()
        .connect_clicked(move |_| menu_for_button.popup());

    // 上半部分：工具条 + 提示 + 文件树
    let tree_side = gtk::Box::new(gtk::Orientation::Vertical, 4);
    tree_side.set_vexpand(true);
    tree_side.append(&toolbar);
    tree_side.append(&state.banner);
    tree_side.append(&state.truncated_label);
    tree_side.append(&state.tree.root);

    let paned = gtk::Paned::new(gtk::Orientation::Vertical);
    paned.set_start_child(Some(&tree_side));
    paned.set_end_child(Some(&state.preview.root));
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_vexpand(true);
    paned.set_position(300);
    root.append(&paned);

    // 终端目录变化时（切换标签或 cd）跟着走；同一条目录不重复读。
    //
    // 这里必须持有强引用：`FilesState` 里装的是控件，控件自己的回调都只拿弱引用
    // （否则就是引用环）。如果这里也用弱引用，`build()` 一返回 state 就被丢掉，
    // 文件树永远是空的。
    let state_for_dir = Rc::clone(&state);
    let on_dir_changed: DirCallback = Rc::new(move |dir: Option<&Path>| {
        let state = &state_for_dir;
        let dir = dir.map(Path::to_path_buf);
        let changed = *state.terminal_dir.borrow() != dir;
        state.terminal_dir.replace(dir.clone());
        if !changed || state.browsing_elsewhere.get() {
            return;
        }
        match dir {
            Some(dir) => load_root(state, &dir),
            None => {
                state.tree.clear();
                state.preview.show_message("还不知道终端的当前目录");
            }
        }
    });

    PageView {
        root,
        cwd_label,
        on_dir_changed,
    }
}

pub fn set_current_dir(view: &PageView, dir: Option<&Path>) {
    set_page_cwd(view, dir);
    view.dir_changed(dir);
}

/// 读取并显示某个目录（后台读，读完后填进文件树）。
fn load_root(state: &Rc<FilesState>, dir: &Path) {
    state.browse_dir.replace(Some(dir.to_path_buf()));
    state.truncated_label.set_visible(false);
    state.tree.clear();

    let path = dir.to_path_buf();
    let show_hidden = state.show_hidden.get();
    let (sender, receiver) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = sender.send_blocking(model::read_dir_sorted(
            &path,
            show_hidden,
            model::MAX_ENTRIES,
        ));
    });

    let dir_text = dir.display().to_string();
    let state_weak = Rc::downgrade(state);
    glib::spawn_future_local(async move {
        let Ok(result) = receiver.recv().await else {
            return;
        };
        let Some(state) = state_weak.upgrade() else {
            return;
        };
        match result {
            Ok((entries, truncated)) => {
                eprintln!("文件树：{dir_text} 读到 {} 项", entries.len());
                state.tree.fill(&entries);
                if truncated {
                    state.truncated_label.set_text(&format!(
                        "已截断：只显示前 {} 项（该目录条目更多）",
                        model::MAX_ENTRIES
                    ));
                    state.truncated_label.set_visible(true);
                }
            }
            Err(error) => {
                let text = format!("无法读取目录：{error}");
                state.truncated_label.set_text(&text);
                state.truncated_label.set_visible(true);
                state.preview.show_message(&text);
            }
        }
    });
}

/// 重新读一遍当前显示的目录（刷新按钮、切换隐藏文件开关时用）。
fn reload_current(state: &Rc<FilesState>) {
    let dir = state.browse_dir.borrow().clone();
    if let Some(dir) = dir {
        load_root(state, &dir);
    }
}

/// 双击文件夹：只改文件树显示，不动终端目录。
fn browse_into(state: &Rc<FilesState>, path: &Path) {
    state.browsing_elsewhere.set(true);
    load_root(state, path);
    update_banner(state);
}

/// 「回到终端目录」：文件树重新跟随终端目录。
fn back_to_terminal(state: &Rc<FilesState>) {
    state.browsing_elsewhere.set(false);
    update_banner(state);
    let dir = state.terminal_dir.borrow().clone();
    match dir {
        Some(dir) => load_root(state, &dir),
        None => {
            state.tree.clear();
            state.preview.show_message("还不知道终端的当前目录");
        }
    }
}

fn update_banner(state: &Rc<FilesState>) {
    if state.browsing_elsewhere.get() {
        if let Some(dir) = state.browse_dir.borrow().clone() {
            state
                .banner_label
                .set_text(&format!("正在浏览：{}（与终端目录不同）", dir.display()));
        }
        state.banner.set_visible(true);
        state.back_button.set_visible(true);
    } else {
        state.banner.set_visible(false);
        state.back_button.set_visible(false);
    }
}

/// 建一个空 store，随后在后台把目录内容填进去（供 `TreeListModel` 懒加载用）。
fn load_children_store(show_hidden: &Rc<Cell<bool>>, path: &Path) -> gio::ListStore {
    let store = gio::ListStore::new::<glib::BoxedAnyObject>();

    let dir = path.to_path_buf();
    let show_hidden = show_hidden.get();
    let (sender, receiver) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = sender.send_blocking(model::read_dir_sorted(
            &dir,
            show_hidden,
            model::MAX_ENTRIES,
        ));
    });

    let store_for_async = store.clone();
    glib::spawn_future_local(async move {
        if let Ok(Ok((entries, _))) = receiver.recv().await {
            for entry in entries {
                store_for_async.append(&glib::BoxedAnyObject::new(entry));
            }
        }
    });

    store
}

/// 在预览区显示一个文件（或文件夹）的内容。
fn preview_file(state: &Rc<FilesState>, entry: &model::DirEntryInfo) {
    state.preview.set_title(&entry.name);
    state.preview.set_note(None);
    state.menu.set_target(Some(&entry.path));

    if entry.is_dir {
        state
            .preview
            .show_message("这是文件夹。展开它，再选中里面的文件就能预览内容。");
        return;
    }

    let path = entry.path.clone();
    let (sender, receiver) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = sender.send_blocking(model::load_preview(&path));
    });

    let state_weak = Rc::downgrade(state);
    glib::spawn_future_local(async move {
        let Ok(result) = receiver.recv().await else {
            return;
        };
        let Some(state) = state_weak.upgrade() else {
            return;
        };
        match result {
            Ok(model::PreviewData::Text { content, truncated }) => {
                state
                    .preview
                    .set_note(truncated.then_some("文件过大，仅显示前 200KB"));
                state.preview.show_text(&content);
            }
            Ok(model::PreviewData::Image { bytes, size }) => {
                match gtk::gdk::Texture::from_bytes(&glib::Bytes::from_owned(bytes)) {
                    Ok(texture) => {
                        state
                            .preview
                            .set_note(Some(&format!("图片，{}", model::format_size(size))));
                        state.preview.show_image(&texture);
                    }
                    Err(error) => {
                        state
                            .preview
                            .show_message(&format!("图片解码失败：{error}"));
                    }
                }
            }
            Ok(model::PreviewData::Binary { size, content_type }) => {
                state.preview.show_message(&format!(
                    "二进制文件，无法预览\n类型：{content_type}\n大小：{}",
                    model::format_size(size)
                ));
            }
            Ok(model::PreviewData::ImageTooLarge { size }) => {
                state.preview.show_message(&format!(
                    "图片太大（{}），无法预览",
                    model::format_size(size)
                ));
            }
            Err(error) => state.preview.show_message(&format!("无法预览：{error}")),
        }
    });
}
