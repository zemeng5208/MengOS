//! diff 显示控件：`gtk::TextView` + `TextTag`（任务书要求用 TextTag 上色）。

use gtk::glib;
use gtk::prelude::*;

use super::model::{self, DiffLineKind};

/// 一段带标题的内容，例如「已暂存」「未暂存」「新文件（未跟踪）」。
pub struct Section {
    pub title: String,
    pub lines: Vec<String>,
}

/// 只读的等宽 diff 视图。
pub struct DiffView {
    pub root: gtk::ScrolledWindow,
    buffer: gtk::TextBuffer,
    view: gtk::TextView,
}

impl DiffView {
    pub fn build() -> Self {
        let buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);
        let table = buffer.tag_table();

        add_tag(&table, "plain", &[]);
        add_tag(
            &table,
            "added",
            &[("background", glib::Value::from("#c8f0c8"))],
        );
        add_tag(
            &table,
            "removed",
            &[("background", glib::Value::from("#f6c9c9"))],
        );
        add_tag(
            &table,
            "hunk",
            &[
                ("background", glib::Value::from("#ccd6e8")),
                ("weight", glib::Value::from(700i32)),
            ],
        );
        add_tag(
            &table,
            "meta",
            &[("foreground", glib::Value::from("#808080"))],
        );
        add_tag(
            &table,
            "section",
            &[
                ("weight", glib::Value::from(700i32)),
                ("background", glib::Value::from("#e4e4e4")),
            ],
        );

        let view = gtk::TextView::with_buffer(&buffer);
        view.set_editable(false);
        view.set_cursor_visible(false);
        view.set_monospace(true);
        view.set_wrap_mode(gtk::WrapMode::None);
        view.set_top_margin(4);
        view.set_bottom_margin(4);
        view.set_left_margin(6);
        view.set_right_margin(6);

        let root = gtk::ScrolledWindow::new();
        root.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        root.set_child(Some(&view));
        root.set_vexpand(true);
        root.set_min_content_height(120);

        Self { root, buffer, view }
    }

    /// 显示若干段内容；返回是否因为超过 `max_lines` 而被截断。
    pub fn show_sections(&self, sections: &[Section], max_lines: usize) -> bool {
        self.buffer.set_text("");
        let mut shown = 0usize;
        let mut truncated = false;

        for section in sections {
            if section.lines.is_empty() {
                continue;
            }
            if shown >= max_lines {
                truncated = true;
                break;
            }
            self.insert_line(&format!("== {} ==", section.title), "section");
            shown += 1;

            for line in &section.lines {
                if shown >= max_lines {
                    truncated = true;
                    break;
                }
                let tag = match model::classify_line(line) {
                    DiffLineKind::Added => "added",
                    DiffLineKind::Removed => "removed",
                    DiffLineKind::Hunk => "hunk",
                    DiffLineKind::Meta => "meta",
                    DiffLineKind::Context => "plain",
                };
                self.insert_line(line, tag);
                shown += 1;
            }
        }

        self.scroll_to_top();
        truncated
    }

    /// 显示一句提示（不是 diff）。
    pub fn show_message(&self, message: &str) {
        self.buffer.set_text("");
        for line in message.split('\n') {
            self.insert_line(line, "meta");
        }
        self.scroll_to_top();
    }

    fn insert_line(&self, text: &str, tag: &str) {
        let mut end = self.buffer.end_iter();
        self.buffer
            .insert_with_tags_by_name(&mut end, &format!("{text}\n"), &[tag]);
    }

    fn scroll_to_top(&self) {
        self.buffer.place_cursor(&self.buffer.start_iter());
        let insert = self.buffer.get_insert();
        self.view.scroll_to_mark(&insert, 0.0, false, 0.0, 0.0);
    }
}

fn add_tag(table: &gtk::TextTagTable, name: &str, properties: &[(&str, glib::Value)]) {
    let tag = gtk::TextTag::new(Some(name));
    for (key, value) in properties {
        tag.set_property(*key, value.clone());
    }
    table.add(&tag);
}
