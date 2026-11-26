//! 文件操作菜单：用默认程序打开 / 在文件管理器中显示 / 复制路径。

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::gio;
use gtk::glib::variant::ToVariant;
use gtk::prelude::*;

use super::model;

/// 挂在预览区「···」按钮上的菜单。
pub struct FileMenu {
    popover: gtk::PopoverMenu,
    anchor: gtk::Button,
    menu: gio::Menu,
    open: gio::SimpleAction,
    reveal: gio::SimpleAction,
    copy_path: gio::SimpleAction,
    target: Rc<RefCell<Option<PathBuf>>>,
}

impl FileMenu {
    pub fn build(anchor: &gtk::Button) -> Rc<FileMenu> {
        let menu = gio::Menu::new();
        let open = gio::SimpleAction::new("open", None);
        let reveal = gio::SimpleAction::new("reveal", None);
        let copy_path = gio::SimpleAction::new("copy-path", None);
        let target: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

        let group = gio::SimpleActionGroup::new();
        group.add_action(&open);
        group.add_action(&reveal);
        group.add_action(&copy_path);
        anchor.insert_action_group("file", Some(&group));

        let popover = gtk::PopoverMenu::from_model(Some(&menu));
        popover.set_has_arrow(false);

        let menu = Rc::new(FileMenu {
            popover,
            anchor: anchor.clone(),
            menu,
            open,
            reveal,
            copy_path,
            target,
        });
        menu.set_target(None);

        // 用默认程序打开（.html 由浏览器打开，和默认程序是同一个接口）
        let menu_for_open = Rc::clone(&menu);
        menu.open.connect_activate(move |_, _| {
            let path = menu_for_open.target.borrow().clone();
            if let Some(path) = path {
                open_with_default_app(path);
            }
        });

        // 在文件管理器中显示
        let menu_for_reveal = Rc::clone(&menu);
        menu.reveal.connect_activate(move |_, _| {
            let path = menu_for_reveal.target.borrow().clone();
            if let Some(path) = path {
                show_in_file_manager(path);
            }
        });

        // 复制路径
        let menu_for_copy = Rc::clone(&menu);
        menu.copy_path.connect_activate(move |_, _| {
            let path = menu_for_copy.target.borrow().clone();
            if let Some(path) = path {
                menu_for_copy
                    .anchor
                    .clipboard()
                    .set_text(&path.to_string_lossy());
            }
        });

        menu
    }

    /// 指定菜单当前针对的文件；`None` 表示没有选中文件（菜单项禁用）。
    pub fn set_target(&self, path: Option<&Path>) {
        self.target.replace(path.map(Path::to_path_buf));
        let enabled = path.is_some();
        self.open.set_enabled(enabled);
        self.reveal.set_enabled(enabled);
        self.copy_path.set_enabled(enabled);

        self.menu.remove_all();
        let Some(path) = path else { return };
        let open_label = if model::is_html(path) {
            "在默认浏览器中打开"
        } else {
            "用默认程序打开"
        };
        self.menu.append(open_label, "file.open");
        self.menu.append("在文件管理器中显示", "file.reveal");
        self.menu.append("复制路径", "file.copy-path");
    }

    /// 弹出菜单（锚在「···」按钮下方）。
    pub fn popup(&self) {
        if self.popover.parent().is_none() {
            self.popover.set_parent(&self.anchor);
        }
        self.popover.popup();
    }
}

/// 交给系统的默认程序打开（`.html` 就是默认浏览器）。
fn open_with_default_app(path: PathBuf) {
    std::thread::spawn(move || {
        let uri = gio::File::for_path(&path).uri().to_string();
        if let Err(error) =
            gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>)
        {
            eprintln!("打开 {uri} 失败：{error}");
        }
    });
}

/// 让文件管理器选中这个文件；D-Bus 接口不可用时退回用 Thunar 打开所在目录。
fn show_in_file_manager(path: PathBuf) {
    std::thread::spawn(move || {
        let uri = gio::File::for_path(&path).uri().to_string();
        let parameters = (vec![uri], String::new()).to_variant();
        let result = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).and_then(
            |connection| {
                connection.call_sync(
                    Some("org.freedesktop.FileManager1"),
                    "/org/freedesktop/FileManager1",
                    "org.freedesktop.FileManager1",
                    "ShowItems",
                    Some(&parameters),
                    None,
                    gio::DBusCallFlags::NONE,
                    3000,
                    gio::Cancellable::NONE,
                )
            },
        );

        if let Err(error) = result {
            eprintln!("FileManager1.ShowItems 失败（{error}），改用 Thunar 打开所在目录");
            let dir = path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("/"));
            if let Err(error) = std::process::Command::new("thunar").arg(dir).spawn() {
                eprintln!("启动 thunar 失败：{error}");
            }
        }
    });
}
