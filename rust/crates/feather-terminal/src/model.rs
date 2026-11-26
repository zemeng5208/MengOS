//! 与界面无关的纯逻辑：标签标题、搜索用的正则转义。
//!
//! 这些函数不依赖 GTK，所以能在 `cargo test` 里直接测（见文末 `tests` 模块）。
//! 任务书要求「跟界面无关、可以单独测试的逻辑」放在单独模块里，这里就是那个模块。

use std::path::Path;

/// 把路径转成标签标题：取最后一级的名字。
///
/// - `/home/user` → `user`
/// - `/tmp` → `tmp`
/// - `/` → `/`（根目录没有最后一级，保留一个斜杠）
/// - 空路径 → `终端`
pub fn title_from_path(path: &Path) -> String {
    if let Some(name) = path.file_name() {
        return name.to_string_lossy().into_owned();
    }
    if path.has_root() {
        return String::from("/");
    }
    String::from("终端")
}

/// 把用户输入的搜索文字转义成「只匹配字面文本」的 PCRE2 正则。
///
/// 用户输入 `a.b` 时不该把 `.` 当通配符，所以逐个字符检查要不要加反斜杠。
pub fn escape_regex(input: &str) -> String {
    const SPECIAL: &str = r"\.^$|()[]{}*+?";
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        if SPECIAL.contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{escape_regex, title_from_path};
    use std::path::Path;

    #[test]
    fn title_uses_last_component() {
        assert_eq!(title_from_path(Path::new("/home/user")), "user");
        assert_eq!(title_from_path(Path::new("/tmp")), "tmp");
        assert_eq!(title_from_path(Path::new("relative/dir")), "dir");
        assert_eq!(
            title_from_path(Path::new("/usr/share/applications")),
            "applications"
        );
    }

    #[test]
    fn title_keeps_root_slash() {
        assert_eq!(title_from_path(Path::new("/")), "/");
    }

    #[test]
    fn title_falls_back_for_empty_path() {
        assert_eq!(title_from_path(Path::new("")), "终端");
    }

    #[test]
    fn escape_leaves_plain_text_alone() {
        assert_eq!(escape_regex("python"), "python");
        assert_eq!(escape_regex("hello world"), "hello world");
        assert_eq!(escape_regex("中文"), "中文");
    }

    #[test]
    fn escape_quotes_metacharacters() {
        assert_eq!(escape_regex("a.b"), r"a\.b");
        assert_eq!(escape_regex("[x]"), r"\[x\]");
        assert_eq!(escape_regex("a+b*c?"), r"a\+b\*c\?");
        assert_eq!(escape_regex(r"\d"), r"\\d");
        assert_eq!(escape_regex("^$|(){}"), r"\^\$\|\(\)\{\}");
    }

    #[test]
    fn escape_keeps_non_ascii_with_punctuation() {
        assert_eq!(escape_regex("中文 (1)"), r"中文 \(1\)");
    }
}
