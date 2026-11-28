//! 侧栏第 2 页：网络连接。
//!
//! 显示**当前标签里运行的程序**（shell 及其所有子孙进程）的真实 TCP/UDP 连接：
//! - 进程：从当前标签的 shell PID 出发，读 `/proc/*/stat` 的父进程号得到整棵进程树
//! - 连接：读每个进程 `/proc/<pid>/fd/` 里形如 `socket:[inode]` 的链接，
//!   再到 `/proc/net/{tcp,tcp6,udp,udp6}` 按 inode 匹配
//! - 页面可见时每 2 秒刷新一次；读不到的进程直接跳过（不需要 root，也不用 sudo）

mod collector;
mod model;

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use super::{set_page_cwd, PageView, PidCallback};

pub const ID: &str = "network";
pub const TITLE: &str = "网络连接";
/// 图标名由 CI 在 `/usr/share/icons` 下核实存在（见 `rust.yml` 的图标检查步骤）。
pub const ICON: &str = "network-wired-symbolic";

/// 页面可见时的自动刷新间隔。
const REFRESH_SECONDS: u32 = 2;

const FOOTER: &str =
    "数据来源：本机连接表（/proc/net）。仅表示本机存在该连接，不代表对方已收到数据。";
const EMPTY_MESSAGE: &str = "暂无网络连接";

struct NetworkState {
    terminal_dir: RefCell<Option<PathBuf>>,
    shell_pid: RefCell<Option<i32>>,
    generation: Cell<u64>,
    root: gtk::Box,
    status_label: gtk::Label,
    listen_header: gtk::Label,
    listen_list: gtk::ListBox,
    conn_header: gtk::Label,
    conn_list: gtk::ListBox,
    empty_label: gtk::Label,
}

pub fn build() -> PageView {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    // 侧栏窄：左右只留 4px，把宽度让给列表里的四列
    root.set_margin_start(4);
    root.set_margin_end(4);

    let title_label = gtk::Label::new(None);
    title_label.set_markup("<b>网络连接</b>");
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

    // 顶部极简路径：当前程序 → 本机网络 → 远端地址
    let path_label = gtk::Label::new(Some("当前程序 → 本机网络 → 远端地址"));
    path_label.set_halign(gtk::Align::Start);
    path_label.set_xalign(0.0);
    path_label.set_wrap(true);
    path_label.add_css_class("dim-label");
    root.append(&path_label);

    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let refresh_button = gtk::Button::with_label("刷新");
    refresh_button.set_tooltip_text(Some("重新读一遍 /proc"));
    let status_label = gtk::Label::new(Some("正在读取…"));
    status_label.set_halign(gtk::Align::Start);
    status_label.set_xalign(0.0);
    status_label.set_hexpand(true);
    status_label.set_width_chars(10);
    status_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    status_label.add_css_class("dim-label");
    toolbar.append(&refresh_button);
    toolbar.append(&status_label);
    root.append(&toolbar);

    let listen_header = section_header("监听中的端口");
    let listen_list = gtk::ListBox::new();
    listen_list.set_selection_mode(gtk::SelectionMode::None);
    let conn_header = section_header("连接");
    let conn_list = gtk::ListBox::new();
    conn_list.set_selection_mode(gtk::SelectionMode::None);

    let empty_label = gtk::Label::new(Some(EMPTY_MESSAGE));
    empty_label.set_halign(gtk::Align::Center);
    empty_label.set_margin_top(12);
    empty_label.add_css_class("dim-label");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.append(&listen_header);
    content.append(&listen_list);
    content.append(&conn_header);
    content.append(&conn_list);
    content.append(&empty_label);

    let scroller = gtk::ScrolledWindow::new();
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.set_child(Some(&content));
    scroller.set_vexpand(true);
    root.append(&scroller);

    let footer = gtk::Label::new(Some(FOOTER));
    footer.set_halign(gtk::Align::Start);
    footer.set_xalign(0.0);
    footer.set_wrap(true);
    footer.add_css_class("dim-label");
    root.append(&footer);

    let state = Rc::new(NetworkState {
        terminal_dir: RefCell::new(None),
        shell_pid: RefCell::new(None),
        generation: Cell::new(0),
        root,
        status_label,
        listen_header,
        listen_list,
        conn_header,
        conn_list,
        empty_label,
    });

    // 刷新按钮
    let state_for_refresh = Rc::clone(&state);
    refresh_button.connect_clicked(move |_| refresh(&state_for_refresh));

    // 页面可见时每 2 秒刷新
    let state_for_timer = Rc::clone(&state);
    let _source = glib::timeout_add_seconds_local(REFRESH_SECONDS, move || {
        if state_for_timer.root.is_mapped() {
            refresh(&state_for_timer);
        }
        glib::ControlFlow::Continue
    });

    // 终端目录变化只影响「当前目录」那一行，不重新读连接
    let state_for_dir = Rc::clone(&state);
    let on_dir_changed: super::DirCallback = Rc::new(move |dir: Option<&Path>| {
        state_for_dir
            .terminal_dir
            .replace(dir.map(Path::to_path_buf));
    });

    // 切标签 / 新开标签时 shell PID 变了，要立刻刷新
    let state_for_pid = Rc::clone(&state);
    let on_pid_changed: PidCallback = Rc::new(move |pid: Option<i32>| {
        let changed = *state_for_pid.shell_pid.borrow() != pid;
        state_for_pid.shell_pid.replace(pid);
        if changed {
            refresh(&state_for_pid);
        }
    });

    PageView {
        root: state.root.clone(),
        cwd_label,
        on_dir_changed,
        on_pid_changed,
    }
}

pub fn set_current_dir(view: &PageView, dir: Option<&Path>) {
    set_page_cwd(view, dir);
    view.dir_changed(dir);
}

/// 重新读一遍连接表（后台线程）。
fn refresh(state: &Rc<NetworkState>) {
    let Some(pid) = *state.shell_pid.borrow() else {
        state.status_label.set_text("还不知道当前标签的 shell 进程");
        clear(state);
        return;
    };

    let generation = state.generation.get() + 1;
    state.generation.set(generation);

    let state_for_done = Rc::downgrade(state);
    let (sender, receiver) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = sender.send_blocking(collector::snapshot(pid));
    });
    glib::spawn_future_local(async move {
        let Ok(snapshot) = receiver.recv().await else {
            return;
        };
        let Some(state) = state_for_done.upgrade() else {
            return;
        };
        if state.generation.get() != generation {
            return;
        }
        apply(&state, snapshot);
    });
}

fn apply(state: &Rc<NetworkState>, snapshot: collector::Snapshot) {
    let listening = snapshot
        .connections
        .iter()
        .filter(|connection| connection.is_listening())
        .count();
    let active = snapshot.connections.len() - listening;

    state.status_label.set_text(&format!(
        "{active} 个连接 · {listening} 个监听端口 · {} 个进程",
        snapshot.process_count
    ));

    clear(state);

    for connection in &snapshot.connections {
        let row = connection_row(connection);
        if connection.is_listening() {
            state.listen_list.append(&row);
        } else {
            state.conn_list.append(&row);
        }
    }

    state
        .listen_header
        .set_visible(state.listen_list.first_child().is_some());
    state
        .conn_header
        .set_visible(state.conn_list.first_child().is_some());
    state
        .empty_label
        .set_visible(snapshot.connections.is_empty());
}

fn clear(state: &Rc<NetworkState>) {
    while let Some(child) = state.listen_list.first_child() {
        state.listen_list.remove(&child);
    }
    while let Some(child) = state.conn_list.first_child() {
        state.conn_list.remove(&child);
    }
    state.listen_header.set_visible(false);
    state.conn_header.set_visible(false);
    state.empty_label.set_visible(true);
}

/// 一行：`进程名(PID)  协议  远端地址:端口  状态`。
///
/// 监听中的端口没有「对方」，地址列显示本地监听地址。
fn connection_row(connection: &collector::Connection) -> gtk::ListBoxRow {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 2);

    let process = gtk::Label::new(Some(&format!("{}({})", connection.process, connection.pid)));
    // 侧栏只有 ~350px：`width_chars` 同时决定最小宽度和自然宽度，
    // 给 13 刚好放得下 `python3(12345)`，否则 PID 会被省略号吃掉。
    process.set_width_chars(13);
    process.set_ellipsize(gtk::pango::EllipsizeMode::End);
    process.set_xalign(0.0);
    process.set_tooltip_text(Some(&format!("{}({})", connection.process, connection.pid)));
    process.add_css_class("monospace");

    let protocol = gtk::Label::new(Some(connection.protocol));
    protocol.set_width_chars(4);
    protocol.set_xalign(0.0);
    protocol.add_css_class("monospace");
    protocol.add_css_class("dim-label");
    protocol.add_css_class("caption");

    let address_text = if connection.is_listening() {
        format!("监听 {}", connection.local)
    } else {
        connection.remote.clone()
    };
    let address = gtk::Label::new(Some(&address_text));
    address.set_width_chars(15);
    address.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    address.set_xalign(0.0);
    address.set_hexpand(true);
    address.set_tooltip_text(Some(&format!(
        "本地 {} → 远端 {}",
        connection.local, connection.remote
    )));
    address.add_css_class("monospace");

    let state_label = gtk::Label::new(Some(connection.state));
    state_label.set_xalign(0.0);
    // 不设 width_chars：状态名（ESTABLISHED / LISTEN）要完整显示，不能省略
    state_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    state_label.add_css_class("monospace");
    state_label.add_css_class("dim-label");
    state_label.add_css_class("caption");

    line.append(&process);
    line.append(&protocol);
    line.append(&address);
    line.append(&state_label);

    let row = gtk::ListBoxRow::new();
    row.set_child(Some(&line));
    row.set_tooltip_text(Some(&format!(
        "{}  {}  {} → {}  {}",
        connection.process,
        connection.protocol,
        connection.local,
        connection.remote,
        connection.state
    )));
    row
}

fn section_header(text: &str) -> gtk::Label {
    let label = gtk::Label::new(None);
    label.set_markup(&format!("<b>{text}</b>"));
    label.set_halign(gtk::Align::Start);
    label.set_margin_top(4);
    label.set_visible(false);
    label
}
