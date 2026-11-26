//! 预览区：文本（等宽、只读）、图片、提示信息三种状态。

use gtk::prelude::*;

/// 预览区：上方一行「文件名 + 提示 + ··· 按钮」，下方是三种状态之一。
pub struct PreviewPane {
    pub root: gtk::Box,
    title_label: gtk::Label,
    note_label: gtk::Label,
    stack: gtk::Stack,
    text_view: gtk::TextView,
    picture: gtk::Picture,
    message_label: gtk::Label,
    menu_button: gtk::Button,
}

impl PreviewPane {
    pub fn build() -> PreviewPane {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_size_request(-1, 150);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        header.set_margin_top(4);
        header.set_margin_bottom(4);
        header.set_margin_start(4);
        header.set_margin_end(4);

        let title_label = gtk::Label::new(None);
        title_label.set_halign(gtk::Align::Start);
        title_label.set_xalign(0.0);
        title_label.set_hexpand(true);
        // 只设 ellipsize 会被压成「…」，必须同时给最小字符宽度
        title_label.set_width_chars(10);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);

        let note_label = gtk::Label::new(None);
        note_label.add_css_class("dim-label");
        note_label.set_visible(false);

        let menu_button = gtk::Button::with_label("···");
        menu_button.set_tooltip_text(Some("文件操作：打开 / 在文件管理器中显示 / 复制路径"));
        menu_button.set_valign(gtk::Align::Center);

        header.append(&title_label);
        header.append(&note_label);
        header.append(&menu_button);
        root.append(&header);

        let stack = gtk::Stack::new();
        stack.set_vexpand(true);
        stack.set_hexpand(true);

        let text_view = gtk::TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_monospace(true);
        text_view.set_wrap_mode(gtk::WrapMode::None);
        text_view.set_top_margin(4);
        text_view.set_bottom_margin(4);
        text_view.set_left_margin(4);
        text_view.set_right_margin(4);
        let text_scroll = gtk::ScrolledWindow::new();
        text_scroll.set_child(Some(&text_view));
        text_scroll.set_vexpand(true);
        text_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        stack.add_named(&text_scroll, Some("text"));

        let picture = gtk::Picture::new();
        picture.set_can_shrink(true);
        picture.set_content_fit(gtk::ContentFit::Contain);
        let picture_scroll = gtk::ScrolledWindow::new();
        picture_scroll.set_child(Some(&picture));
        picture_scroll.set_vexpand(true);
        picture_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        stack.add_named(&picture_scroll, Some("image"));

        let message_label = gtk::Label::new(None);
        message_label.set_wrap(true);
        message_label.set_justify(gtk::Justification::Center);
        message_label.set_margin_top(12);
        message_label.set_margin_bottom(12);
        message_label.set_margin_start(12);
        message_label.set_margin_end(12);
        stack.add_named(&message_label, Some("message"));

        root.append(&stack);

        let pane = PreviewPane {
            root,
            title_label,
            note_label,
            stack,
            text_view,
            picture,
            message_label,
            menu_button,
        };
        pane.show_message("单击文件树里的文件，这里会显示内容");
        pane
    }

    /// 预览区右上角的「···」按钮（菜单挂在它上面）。
    pub fn menu_button(&self) -> &gtk::Button {
        &self.menu_button
    }

    pub fn set_title(&self, text: &str) {
        self.title_label.set_text(text);
        self.title_label.set_tooltip_text(Some(text));
    }

    /// 标题后面的灰色小字（例如「文件过大，仅显示前 200KB」）。
    pub fn set_note(&self, note: Option<&str>) {
        match note {
            Some(note) => {
                self.note_label.set_text(note);
                self.note_label.set_visible(true);
            }
            None => self.note_label.set_visible(false),
        }
    }

    pub fn show_text(&self, text: &str) {
        self.text_view.buffer().set_text(text);
        self.stack.set_visible_child_name("text");
    }

    pub fn show_image(&self, texture: &gtk::gdk::Texture) {
        self.picture.set_paintable(Some(texture));
        self.stack.set_visible_child_name("image");
    }

    pub fn show_message(&self, text: &str) {
        self.message_label.set_text(text);
        self.stack.set_visible_child_name("message");
    }
}
