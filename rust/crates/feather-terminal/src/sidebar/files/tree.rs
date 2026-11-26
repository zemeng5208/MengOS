//! 文件树：`gtk::ListView` + `gtk::TreeListModel`。
//!
//! 根目录的条目由调用方填进 `root_store`；某个文件夹被展开时，`TreeListModel`
//! 才会调用 `load_children` 去读它的内容（懒加载，而且是后台线程读）。

use std::path::Path;
use std::rc::Rc;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use super::model::DirEntryInfo;

pub type EntryCallback = Rc<dyn Fn(&DirEntryInfo)>;
pub type LoadChildren = Rc<dyn Fn(&Path) -> gio::ListStore>;

/// 文件树需要的回调。
pub struct Callbacks {
    pub on_select: EntryCallback,
    pub on_activate: EntryCallback,
    pub on_menu: EntryCallback,
    pub load_children: LoadChildren,
}

/// 文件树控件。
pub struct FileTree {
    pub root: gtk::ScrolledWindow,
    root_store: gio::ListStore,
    selection: gtk::SingleSelection,
}

/// 从 `glib::Object` 里取出我们存进去的 `DirEntryInfo`。
fn entry_of(object: &glib::Object) -> Option<DirEntryInfo> {
    let boxed = object.downcast_ref::<glib::BoxedAnyObject>()?;
    Some(boxed.borrow::<DirEntryInfo>().clone())
}

/// 从 `TreeListRow` 里取出它代表的目录项。
fn entry_of_row(row: &gtk::TreeListRow) -> Option<DirEntryInfo> {
    let item = row.item()?;
    entry_of(&item)
}

pub fn build(callbacks: Callbacks) -> FileTree {
    let root_store = gio::ListStore::new::<glib::BoxedAnyObject>();

    // 懒加载：只有文件夹会被问「你的子项是什么」，这时才去后台读目录。
    let load_children = Rc::clone(&callbacks.load_children);
    let tree_model = gtk::TreeListModel::new(root_store.clone(), false, false, move |object| {
        let entry = entry_of(object)?;
        if !entry.is_dir {
            return None;
        }
        Some(load_children(&entry.path).upcast::<gio::ListModel>())
    });

    let selection = gtk::SingleSelection::new(Some(tree_model.clone()));
    selection.set_autoselect(false);
    selection.set_can_unselect(true);

    let factory = gtk::SignalListItemFactory::new();
    let on_menu = Rc::clone(&callbacks.on_menu);
    factory.connect_bind(move |_, list_item| {
        let Some(row) = list_item
            .item()
            .and_then(|item| item.downcast::<gtk::TreeListRow>().ok())
        else {
            return;
        };
        let Some(entry) = entry_of_row(&row) else {
            return;
        };

        let expander = gtk::TreeExpander::new();
        expander.set_list_row(Some(&row));

        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let icon = gtk::Image::from_icon_name(if entry.is_dir {
            "folder-symbolic"
        } else {
            "text-x-generic-symbolic"
        });
        let label = gtk::Label::new(Some(&entry.name));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        // 只设 ellipsize 会被压成「…」，必须同时给最小字符宽度
        label.set_width_chars(10);
        label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        row_box.append(&icon);
        row_box.append(&label);
        expander.set_child(Some(&row_box));

        // 右键：菜单针对这个文件弹出
        let gesture = gtk::GestureClick::new();
        gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
        let on_menu = Rc::clone(&on_menu);
        let entry_for_menu = entry.clone();
        gesture.connect_pressed(move |_, _, _, _| on_menu(&entry_for_menu));
        expander.add_controller(gesture);

        list_item.set_child(Some(&expander));
    });

    let list_view = gtk::ListView::new(Some(selection.clone()), Some(factory));
    list_view.set_show_separators(false);

    // 单击（选中）→ 预览
    let on_select = Rc::clone(&callbacks.on_select);
    let tree_model_for_select = tree_model.clone();
    selection.connect_selection_changed(move |selection, _, _| {
        let position = selection.selected();
        if position == gtk::INVALID_LIST_POSITION {
            return;
        }
        let Some(row) = tree_model_for_select.row(position) else {
            return;
        };
        let Some(entry) = entry_of_row(&row) else {
            return;
        };
        on_select(&entry);
    });

    // 双击 → 进入文件夹（只改文件树，不改终端目录）
    let on_activate = Rc::clone(&callbacks.on_activate);
    let tree_model_for_activate = tree_model.clone();
    list_view.connect_activate(move |_, position| {
        let Some(row) = tree_model_for_activate.row(position) else {
            return;
        };
        let Some(entry) = entry_of_row(&row) else {
            return;
        };
        on_activate(&entry);
    });

    let root = gtk::ScrolledWindow::new();
    root.set_child(Some(&list_view));
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.set_min_content_height(160);
    root.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    FileTree {
        root,
        root_store,
        selection,
    }
}

impl FileTree {
    /// 用新读出来的条目替换根目录内容。
    pub fn fill(&self, entries: &[DirEntryInfo]) {
        self.root_store.remove_all();
        for entry in entries {
            self.root_store
                .append(&glib::BoxedAnyObject::new(entry.clone()));
        }
    }

    /// 清空文件树（还不知道目录、或者正在读新目录时）。
    pub fn clear(&self) {
        self.selection.set_selected(gtk::INVALID_LIST_POSITION);
        self.root_store.remove_all();
    }
}
