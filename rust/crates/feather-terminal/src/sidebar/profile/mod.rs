//! 侧栏第 5 页：个人资料。
//!
//! 显示的全部是本机真实数据（读不到就写「未知」，绝不编造）：
//! 用户名/UID/所属组、主机名、系统版本、内核版本、当前 shell、FeatherTerminal 版本、
//! 本次开机时长、`~/.zsh_history` 里的常用命令 Top 10（只统计命令名，不含参数）。

mod collector;
mod model;

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use super::{set_page_cwd, PageView};

pub const ID: &str = "profile";
pub const TITLE: &str = "个人资料";
/// 图标名由 CI 在 `/usr/share/icons` 下核实存在（见 `rust.yml` 的图标检查步骤）。
pub const ICON: &str = "avatar-default-symbolic";

const COMMAND_NOTE: &str = "只统计命令名称，不显示参数。数据只在本机读取，不上传。";
const NO_HISTORY: &str = "暂无历史记录";
const AI_STATE: &str = "未关联（FeatherBot 尚未实现）";

struct ProfileState {
    root: gtk::Box,
    generation: Cell<u64>,
    username: gtk::Label,
    uid: gtk::Label,
    groups: gtk::Label,
    hostname: gtk::Label,
    os_version: gtk::Label,
    kernel: gtk::Label,
    shell: gtk::Label,
    version: gtk::Label,
    uptime: gtk::Label,
    commands_box: gtk::Box,
    commands_note: gtk::Label,
}

pub fn build() -> PageView {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);

    let title_label = gtk::Label::new(None);
    title_label.set_markup("<b>个人资料</b>");
    title_label.set_halign(gtk::Align::Start);
    root.append(&title_label);

    let cwd_label = gtk::Label::new(None);
    cwd_label.set_halign(gtk::Align::Start);
    cwd_label.set_xalign(0.0);
    cwd_label.set_width_chars(18);
    cwd_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    cwd_label.add_css_class("dim-label");
    cwd_label.set_tooltip_text(Some("终端当前目录（数据来自 /proc/<pid>/cwd）"));
    root.append(&cwd_label);

    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let refresh_button = gtk::Button::with_label("刷新");
    refresh_button.set_tooltip_text(Some("重新读一遍本机资料"));
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

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);

    content.append(&section_header("账户"));
    let username = gtk::Label::new(Some("…"));
    content.append(&info_row("用户名", &username));
    let uid = gtk::Label::new(Some("…"));
    content.append(&info_row("UID", &uid));
    let groups = gtk::Label::new(Some("…"));
    content.append(&info_row("所属组", &groups));

    content.append(&section_header("系统"));
    let hostname = gtk::Label::new(Some("…"));
    content.append(&info_row("主机名", &hostname));
    let os_version = gtk::Label::new(Some("…"));
    content.append(&info_row("系统版本", &os_version));
    let kernel = gtk::Label::new(Some("…"));
    content.append(&info_row("内核版本", &kernel));
    let shell = gtk::Label::new(Some("…"));
    content.append(&info_row("当前 shell", &shell));
    let version = gtk::Label::new(Some("…"));
    content.append(&info_row("终端版本", &version));
    let uptime = gtk::Label::new(Some("…"));
    content.append(&info_row("本次开机", &uptime));

    content.append(&section_header("常用命令 Top 10"));
    let commands_note = gtk::Label::new(Some(COMMAND_NOTE));
    commands_note.set_xalign(0.0);
    commands_note.set_wrap(true);
    commands_note.add_css_class("dim-label");
    content.append(&commands_note);
    let commands_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    content.append(&commands_box);

    content.append(&section_header("AI 使用统计"));
    let ai_label = gtk::Label::new(Some(AI_STATE));
    ai_label.set_xalign(0.0);
    ai_label.set_wrap(true);
    ai_label.add_css_class("dim-label");
    content.append(&ai_label);

    let scroller = gtk::ScrolledWindow::new();
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.set_child(Some(&content));
    scroller.set_vexpand(true);
    root.append(&scroller);

    let state = Rc::new(ProfileState {
        root,
        generation: Cell::new(0),
        username,
        uid,
        groups,
        hostname,
        os_version,
        kernel,
        shell,
        version,
        uptime,
        commands_box,
        commands_note,
    });

    let state_for_button = Rc::clone(&state);
    let status_for_button = status_label.clone();
    refresh_button.connect_clicked(move |_| refresh(&state_for_button, &status_for_button));

    // 页面一建出来就读一次（后台线程，不阻塞界面）
    let state_for_first = Rc::clone(&state);
    let status_for_first = status_label.clone();
    glib::idle_add_local_once(move || {
        refresh(&state_for_first, &status_for_first);
    });

    let on_dir_changed: super::DirCallback = Rc::new(|_| {});

    PageView {
        root: state.root.clone(),
        cwd_label,
        on_dir_changed,
        // 个人资料和标签无关，PID 变化不用管
        on_pid_changed: Rc::new(|_| {}),
    }
}

pub fn set_current_dir(view: &PageView, dir: Option<&Path>) {
    set_page_cwd(view, dir);
    view.dir_changed(dir);
}

/// 重新读一遍资料（后台线程）。
fn refresh(state: &Rc<ProfileState>, status: &gtk::Label) {
    let generation = state.generation.get() + 1;
    state.generation.set(generation);
    status.set_text("正在读取…");

    let version = crate::BUILD_VERSION;
    let state_for_done = Rc::downgrade(state);
    let status_for_done = status.clone();
    let (sender, receiver) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = sender.send_blocking(collector::collect(version));
    });
    glib::spawn_future_local(async move {
        let Ok(profile) = receiver.recv().await else {
            return;
        };
        let Some(state) = state_for_done.upgrade() else {
            return;
        };
        if state.generation.get() != generation {
            return;
        }
        apply(&state, profile);
        status_for_done.set_text("已刷新");
    });
}

fn apply(state: &Rc<ProfileState>, profile: collector::Profile) {
    state.username.set_text(&profile.username);
    state.uid.set_text(&profile.uid);
    state.groups.set_text(&profile.groups);
    state.hostname.set_text(&profile.hostname);
    state.os_version.set_text(&profile.os_version);
    state.kernel.set_text(&profile.kernel);
    state.shell.set_text(&profile.shell);
    state.version.set_text(&profile.version);
    state.uptime.set_text(&profile.uptime);

    while let Some(child) = state.commands_box.first_child() {
        state.commands_box.remove(&child);
    }

    match profile.history_state {
        collector::HistoryState::Loaded => {
            state.commands_note.set_text(COMMAND_NOTE);
            for (index, (name, count)) in profile.top_commands.iter().enumerate() {
                let row = gtk::Label::new(Some(&format!("{}. {name}  {count} 次", index + 1)));
                row.set_xalign(0.0);
                row.add_css_class("monospace");
                state.commands_box.append(&row);
            }
        }
        collector::HistoryState::Missing => {
            state.commands_note.set_text(COMMAND_NOTE);
            let empty = gtk::Label::new(Some(NO_HISTORY));
            empty.set_xalign(0.0);
            empty.add_css_class("dim-label");
            state.commands_box.append(&empty);
        }
        collector::HistoryState::Unreadable => {
            state.commands_note.set_text(COMMAND_NOTE);
            let broken = gtk::Label::new(Some("历史文件读不出来（可能不是 UTF-8 编码）"));
            broken.set_xalign(0.0);
            broken.set_wrap(true);
            broken.add_css_class("dim-label");
            state.commands_box.append(&broken);
        }
    }
}

/// `标签：值` 的一行（标签固定宽度，值自动换行）。
fn info_row(name: &str, value: &gtk::Label) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);

    let key = gtk::Label::new(Some(name));
    key.set_width_chars(10);
    key.set_xalign(0.0);
    key.set_valign(gtk::Align::Start);
    key.add_css_class("dim-label");

    value.set_xalign(0.0);
    value.set_wrap(true);
    value.set_hexpand(true);
    value.set_selectable(true);

    row.append(&key);
    row.append(value);
    row
}

fn section_header(text: &str) -> gtk::Label {
    let label = gtk::Label::new(None);
    label.set_markup(&format!("<b>{text}</b>"));
    label.set_halign(gtk::Align::Start);
    label.set_margin_top(6);
    label
}
