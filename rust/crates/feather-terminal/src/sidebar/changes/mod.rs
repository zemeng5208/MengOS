//! 侧栏第 1 页：文件更改（Git Diff）。
//!
//! - 后台跑 `git rev-parse --show-toplevel` / `git status --porcelain=v1 -z`
//! - 页面可见时每 3 秒刷新一次列表；页面不可见（被 `gtk::Stack` 切走）时不刷新
//! - 单击文件 → 后台跑 `git diff` 与 `git diff --cached`，两部分都显示并标明「已暂存 / 未暂存」
//! - 未跟踪文件直接读内容（最多 200KB），全部按新增行显示
//!
//! 所有数据都来自真实的 git 命令输出，不缓存、不自己比较文件内容。

mod diff_view;
mod git;
mod model;

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use super::{set_page_cwd, DirCallback, PageView};

pub const ID: &str = "changes";
pub const TITLE: &str = "文件更改";
/// 图标名由 CI 在 `/usr/share/icons` 下核实存在（见 `rust.yml` 的图标检查步骤）。
pub const ICON: &str = "document-edit-symbolic";

/// 页面可见时的自动刷新间隔。
const REFRESH_SECONDS: u32 = 3;

const NOT_REPO_MESSAGE: &str =
    "当前目录不是 Git 仓库，无法显示文件更改。（非 Git 文件需要基线快照才能对比，本功能暂不支持）";
const PICK_FILE_MESSAGE: &str = "点左边的文件，这里显示它的 diff。";
const CLEAN_MESSAGE: &str = "工作区是干净的：没有未提交的更改。";

struct ChangesState {
    terminal_dir: RefCell<Option<PathBuf>>,
    repo_root: RefCell<Option<PathBuf>>,
    entries: RefCell<Vec<model::ChangeEntry>>,
    /// 当前选中文件的 `diff_path`
    selected: RefCell<Option<String>>,
    /// 刷新列表时恢复选中，不希望它重新跑一次 diff
    restoring: Cell<bool>,
    /// 每次刷新递增，用来丢弃过期的后台结果
    generation: Cell<u64>,
    root: gtk::Box,
    status_label: gtk::Label,
    list_box: gtk::ListBox,
    truncated_label: gtk::Label,
    diff_view: diff_view::DiffView,
}

pub fn build() -> PageView {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);

    let title_label = gtk::Label::new(None);
    title_label.set_markup("<b>文件更改</b>");
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

    // 工具条：刷新按钮 + 状态说明
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let refresh_button = gtk::Button::with_label("刷新");
    refresh_button.set_tooltip_text(Some("重新执行 git status"));
    let status_label = gtk::Label::new(Some("正在读取…"));
    status_label.set_halign(gtk::Align::Start);
    status_label.set_xalign(0.0);
    status_label.set_hexpand(true);
    status_label.set_width_chars(12);
    status_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    status_label.add_css_class("dim-label");
    toolbar.append(&refresh_button);
    toolbar.append(&status_label);
    root.append(&toolbar);

    let truncated_label = gtk::Label::new(None);
    truncated_label.set_xalign(0.0);
    truncated_label.set_wrap(true);
    truncated_label.add_css_class("error");
    truncated_label.set_visible(false);
    root.append(&truncated_label);

    // 文件列表
    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.set_activate_on_single_click(true);
    let list_scroller = gtk::ScrolledWindow::new();
    list_scroller.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    list_scroller.set_child(Some(&list_box));
    list_scroller.set_min_content_height(110);

    let diff_view = diff_view::DiffView::build();
    diff_view.show_message(PICK_FILE_MESSAGE);

    let paned = gtk::Paned::new(gtk::Orientation::Vertical);
    paned.set_start_child(Some(&list_scroller));
    paned.set_end_child(Some(&diff_view.root));
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_vexpand(true);
    paned.set_position(240);
    root.append(&paned);

    let state = Rc::new(ChangesState {
        terminal_dir: RefCell::new(None),
        repo_root: RefCell::new(None),
        entries: RefCell::new(Vec::new()),
        selected: RefCell::new(None),
        restoring: Cell::new(false),
        generation: Cell::new(0),
        root,
        status_label,
        list_box,
        truncated_label,
        diff_view,
    });

    // 刷新按钮：清掉选中，重新读一遍
    let state_for_refresh = Rc::clone(&state);
    refresh_button.connect_clicked(move |_| refresh(&state_for_refresh, true));

    // 选中文件 → 显示 diff
    let state_for_select = Rc::clone(&state);
    state.list_box.connect_row_selected(move |_, row| {
        let Some(row) = row else {
            return;
        };
        let index = row.index();
        if index < 0 {
            return;
        }
        let entry = state_for_select
            .entries
            .borrow()
            .get(index as usize)
            .cloned();
        let Some(entry) = entry else {
            return;
        };
        state_for_select
            .selected
            .replace(Some(entry.diff_path.clone()));
        if state_for_select.restoring.get() {
            // 自动刷新时恢复选中，不要重新跑 diff（否则每 3 秒闪一次）
            return;
        }
        show_diff(&state_for_select, entry);
    });

    // 页面可见时每 3 秒刷新文件列表
    let state_for_timer = Rc::clone(&state);
    let _source = glib::timeout_add_seconds_local(REFRESH_SECONDS, move || {
        if state_for_timer.root.is_mapped() {
            refresh(&state_for_timer, false);
        }
        glib::ControlFlow::Continue
    });

    // 终端目录变化时重新判断是不是 Git 仓库
    let state_for_dir = Rc::clone(&state);
    let on_dir_changed: DirCallback = Rc::new(move |dir: Option<&Path>| {
        let dir = dir.map(Path::to_path_buf);
        let changed = *state_for_dir.terminal_dir.borrow() != dir;
        state_for_dir.terminal_dir.replace(dir);
        if changed {
            refresh(&state_for_dir, true);
        }
    });

    PageView {
        root: state.root.clone(),
        cwd_label,
        on_dir_changed,
        // 文件更改页只关心目录，PID 变化不用管
        on_pid_changed: Rc::new(|_| {}),
    }
}

pub fn set_current_dir(view: &PageView, dir: Option<&Path>) {
    set_page_cwd(view, dir);
    view.dir_changed(dir);
}

/// 重新读一遍仓库状态。`reset_selection` 为真时清掉当前选中的文件（「刷新」按钮）。
fn refresh(state: &Rc<ChangesState>, reset_selection: bool) {
    let Some(dir) = state.terminal_dir.borrow().clone() else {
        state.repo_root.replace(None);
        state.status_label.set_text("还不知道终端的当前目录");
        clear_list(state);
        state.diff_view.show_message(PICK_FILE_MESSAGE);
        return;
    };

    let generation = state.generation.get() + 1;
    state.generation.set(generation);
    if reset_selection {
        state.selected.replace(None);
        state.list_box.unselect_all();
        state.truncated_label.set_visible(false);
    }

    let state_for_done = Rc::downgrade(state);
    run_in_background(
        move || git::load_repo_state(&dir),
        move |repo_state| {
            let Some(state) = state_for_done.upgrade() else {
                return;
            };
            if state.generation.get() != generation {
                return;
            }
            apply_repo_state(&state, repo_state, reset_selection);
        },
    );
}

fn apply_repo_state(state: &Rc<ChangesState>, repo_state: git::RepoState, reset_selection: bool) {
    match repo_state {
        git::RepoState::NotInstalled => {
            state.repo_root.replace(None);
            state.status_label.set_text("未安装 git");
            clear_list(state);
            state
                .diff_view
                .show_message("未安装 git，无法显示文件更改。");
        }
        git::RepoState::NotRepo => {
            state.repo_root.replace(None);
            state.status_label.set_text("不是 Git 仓库");
            clear_list(state);
            state.diff_view.show_message(NOT_REPO_MESSAGE);
        }
        git::RepoState::Error(error) => {
            state.repo_root.replace(None);
            state.status_label.set_text("读取失败");
            clear_list(state);
            state.diff_view.show_message(&error);
        }
        git::RepoState::Repo { root, entries } => {
            state.repo_root.replace(Some(root));
            let count = entries.len();
            state.status_label.set_text(&if count == 0 {
                "没有更改".to_string()
            } else {
                format!("{count} 个文件有更改")
            });
            fill_list(state, entries, reset_selection);
        }
    }
}

/// 重建文件列表，并尽量保留原来的选中项。
fn fill_list(state: &Rc<ChangesState>, entries: Vec<model::ChangeEntry>, reset_selection: bool) {
    state.restoring.set(true);
    state.list_box.unselect_all();
    while let Some(child) = state.list_box.first_child() {
        state.list_box.remove(&child);
    }

    for entry in &entries {
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let marker = gtk::Label::new(Some(&entry.marker()));
        marker.add_css_class("monospace");
        marker.add_css_class("dim-label");
        marker.set_valign(gtk::Align::Center);

        let path_label = gtk::Label::new(Some(&entry.display_path));
        path_label.set_halign(gtk::Align::Start);
        path_label.set_xalign(0.0);
        path_label.set_width_chars(14);
        path_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        path_label.add_css_class("monospace");

        row_box.append(&marker);
        row_box.append(&path_label);

        let row = gtk::ListBoxRow::new();
        row.set_child(Some(&row_box));
        row.set_tooltip_text(Some(&format!(
            "{}（{}）",
            entry.display_path,
            entry.kind.label()
        )));
        state.list_box.append(&row);
    }

    let wanted = if reset_selection {
        None
    } else {
        state.selected.borrow().clone()
    };
    let index = wanted
        .as_ref()
        .and_then(|path| entries.iter().position(|entry| &entry.diff_path == path));

    match index {
        Some(index) => {
            if let Some(row) = state.list_box.row_at_index(index as i32) {
                state.list_box.select_row(Some(&row));
            }
        }
        None => {
            state.selected.replace(None);
            state.diff_view.show_message(if entries.is_empty() {
                CLEAN_MESSAGE
            } else {
                PICK_FILE_MESSAGE
            });
            state.truncated_label.set_visible(false);
        }
    }

    state.restoring.set(false);
    state.entries.replace(entries);
}

fn clear_list(state: &Rc<ChangesState>) {
    state.restoring.set(true);
    state.list_box.unselect_all();
    while let Some(child) = state.list_box.first_child() {
        state.list_box.remove(&child);
    }
    state.entries.replace(Vec::new());
    state.selected.replace(None);
    state.truncated_label.set_visible(false);
    state.restoring.set(false);
}

/// 读一个文件的 diff 并显示。
fn show_diff(state: &Rc<ChangesState>, entry: model::ChangeEntry) {
    let Some(root) = state.repo_root.borrow().clone() else {
        state.diff_view.show_message(NOT_REPO_MESSAGE);
        return;
    };

    let generation = state.generation.get();
    let entry_for_work = entry.clone();
    state.diff_view.show_message("正在读取 diff…");

    let state_for_done = Rc::downgrade(state);
    run_in_background(
        move || git::load_file_diff(&root, &entry_for_work),
        move |result| {
            let Some(state) = state_for_done.upgrade() else {
                return;
            };
            // 结果过期，或者用户已经点了别的文件
            if state.generation.get() != generation
                || state.selected.borrow().as_deref() != Some(entry.diff_path.as_str())
            {
                return;
            }
            match result {
                Ok(diff) => {
                    let sections = build_sections(&diff);
                    let truncated = state
                        .diff_view
                        .show_sections(&sections, model::MAX_DIFF_LINES);
                    if truncated {
                        state
                            .truncated_label
                            .set_text(&format!("已截断：只显示前 {} 行", model::MAX_DIFF_LINES));
                    }
                    state.truncated_label.set_visible(truncated);
                }
                Err(error) => {
                    state.truncated_label.set_visible(false);
                    state
                        .diff_view
                        .show_message(&format!("无法显示 diff：{error}"));
                }
            }
        },
    );
}

/// 把 `FileDiff` 拆成「已暂存 / 未暂存 / 新文件」三段。
fn build_sections(diff: &git::FileDiff) -> Vec<diff_view::Section> {
    let mut sections = Vec::new();

    if !diff.staged.trim().is_empty() {
        sections.push(diff_view::Section {
            title: "已暂存（git diff --cached）".to_string(),
            lines: model::split_lines(&diff.staged),
        });
    }
    if !diff.unstaged.trim().is_empty() {
        sections.push(diff_view::Section {
            title: "未暂存（git diff）".to_string(),
            lines: model::split_lines(&diff.unstaged),
        });
    }
    if let Some(content) = &diff.untracked {
        let (lines, truncated) = model::untracked_lines(content, model::MAX_PREVIEW_BYTES);
        let mut title = String::from("新文件（未跟踪）");
        if truncated {
            title.push_str("，只显示前 200KB");
        }
        sections.push(diff_view::Section { title, lines });
    }

    if sections.is_empty() {
        sections.push(diff_view::Section {
            title: "没有可显示的 diff".to_string(),
            lines: Vec::new(),
        });
    }

    sections
}

/// 在后台线程跑 `work`，把结果送回主线程交给 `done`。
///
/// 主线程不执行任何外部命令，也不做阻塞 IO。
fn run_in_background<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    done: impl FnOnce(T) + 'static,
) {
    let (sender, receiver) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = sender.send_blocking(work());
    });
    glib::spawn_future_local(async move {
        if let Ok(value) = receiver.recv().await {
            done(value);
        }
    });
}
