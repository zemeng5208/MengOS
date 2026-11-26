//! 文件树与预览里能脱离 GTK 单独测试的逻辑。
//!
//! 这里只有纯函数和「读文件系统」的函数，不碰任何 GTK 控件，
//! 所以能在没有显示器的 CI 里直接跑单元测试。

use std::io::Read;
use std::path::{Path, PathBuf};

use gtk::gio;

/// 一个目录最多显示多少项，超过就截断并提示。
pub const MAX_ENTRIES: usize = 2000;
/// 文本预览最多显示多少字节。
pub const MAX_PREVIEW_BYTES: usize = 200 * 1024;
/// 图片超过这个大小就不预览，避免解码卡住界面。
pub const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;

/// 文件树里的一项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntryInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// 预览该怎么显示。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewKind {
    Text,
    Image,
    Binary,
}

/// 后台读出来的预览数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewData {
    /// 文本内容（已经按 200KB 截断）与是否被截断
    Text { content: String, truncated: bool },
    /// 图片原始字节与文件大小
    Image { bytes: Vec<u8>, size: u64 },
    /// 二进制文件：只告诉大小和类型
    Binary { size: u64, content_type: String },
    /// 图片太大，不预览
    ImageTooLarge { size: u64 },
}

/// 以点开头的文件算隐藏文件。
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// 文件夹排在前面；同一类里按名称排序（先不区分大小写，再按原名，保证顺序稳定）。
pub fn sort_entries(entries: &mut [DirEntryInfo]) {
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
}

/// 超过 `max` 项时只留前 `max` 项，返回是否发生了截断。
pub fn truncate_entries(entries: &mut Vec<DirEntryInfo>, max: usize) -> bool {
    if entries.len() > max {
        entries.truncate(max);
        true
    } else {
        false
    }
}

/// 把字节数写成 `1.5 MB` 这样的短文本。
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// 按 MIME 类型决定预览方式。
pub fn preview_kind(content_type: &str) -> PreviewKind {
    let content_type = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();

    if content_type.starts_with("image/") {
        return PreviewKind::Image;
    }
    if content_type.starts_with("text/") {
        return PreviewKind::Text;
    }

    const TEXT_LIKE: [&str; 14] = [
        "application/json",
        "application/xml",
        "application/javascript",
        "application/x-shellscript",
        "application/x-sh",
        "application/x-python",
        "application/toml",
        "application/x-toml",
        "application/yaml",
        "application/x-yaml",
        "application/x-desktop",
        "application/x-httpd-php",
        "application/x-perl",
        "inode/x-empty",
    ];
    if TEXT_LIKE.contains(&content_type) {
        PreviewKind::Text
    } else {
        PreviewKind::Binary
    }
}

/// `.html` / `.htm` 文件用浏览器打开。
pub fn is_html(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
        })
        .unwrap_or(false)
}

/// 读一个目录：过滤隐藏文件、排序、截断。这个函数会在后台线程里调用。
pub fn read_dir_sorted(
    path: &Path,
    show_hidden: bool,
    max: usize,
) -> Result<(Vec<DirEntryInfo>, bool), String> {
    let read_dir =
        std::fs::read_dir(path).map_err(|error| format!("{}：{error}", path.display()))?;

    let mut entries = Vec::new();
    for entry in read_dir {
        let Ok(entry) = entry else { continue };
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && is_hidden(&name) {
            continue;
        }
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        entries.push(DirEntryInfo {
            name,
            path: entry.path(),
            is_dir,
        });
    }

    sort_entries(&mut entries);
    let truncated = truncate_entries(&mut entries, max);
    Ok((entries, truncated))
}

/// 读出一个文件该怎么预览、以及预览要用的数据。这个函数会在后台线程里调用。
pub fn load_preview(path: &Path) -> Result<PreviewData, String> {
    let file = gio::File::for_path(path);
    let info = file
        .query_info(
            "standard::content-type,standard::size,standard::type",
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        )
        .map_err(|error| format!("无法读取文件信息：{error}"))?;

    if info.file_type() == gio::FileType::Directory {
        return Err(String::from("这是文件夹"));
    }

    let size = info.size().max(0) as u64;
    let content_type = info
        .content_type()
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("application/octet-stream"));

    match preview_kind(&content_type) {
        PreviewKind::Text => {
            let file = std::fs::File::open(path).map_err(|error| format!("打开失败：{error}"))?;
            let mut data = Vec::new();
            file.take((MAX_PREVIEW_BYTES + 1) as u64)
                .read_to_end(&mut data)
                .map_err(|error| format!("读取失败：{error}"))?;

            let truncated = data.len() > MAX_PREVIEW_BYTES;
            if truncated {
                data.truncate(MAX_PREVIEW_BYTES);
            }
            Ok(PreviewData::Text {
                content: String::from_utf8_lossy(&data).to_string(),
                truncated,
            })
        }
        PreviewKind::Image => {
            if size > MAX_IMAGE_BYTES {
                return Ok(PreviewData::ImageTooLarge { size });
            }
            let bytes = std::fs::read(path).map_err(|error| format!("读取失败：{error}"))?;
            Ok(PreviewData::Image { bytes, size })
        }
        PreviewKind::Binary => Ok(PreviewData::Binary { size, content_type }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        format_size, is_hidden, is_html, preview_kind, sort_entries, truncate_entries,
        DirEntryInfo, PreviewKind, MAX_ENTRIES,
    };
    use std::path::PathBuf;

    fn entry(name: &str, is_dir: bool) -> DirEntryInfo {
        DirEntryInfo {
            name: String::from(name),
            path: PathBuf::from(format!("/tmp/{name}")),
            is_dir,
        }
    }

    fn names(entries: &[DirEntryInfo]) -> Vec<String> {
        entries.iter().map(|entry| entry.name.clone()).collect()
    }

    #[test]
    fn folders_come_before_files() {
        let mut entries = vec![
            entry("z.txt", false),
            entry("a", true),
            entry("b.txt", false),
            entry("b", true),
        ];
        sort_entries(&mut entries);
        assert_eq!(names(&entries), vec!["a", "b", "b.txt", "z.txt"]);
    }

    #[test]
    fn sorting_ignores_case_but_keeps_stable_order() {
        let mut entries = vec![
            entry("beta", false),
            entry("Alpha", false),
            entry("alpha", false),
        ];
        sort_entries(&mut entries);
        assert_eq!(names(&entries), vec!["Alpha", "alpha", "beta"]);
    }

    #[test]
    fn truncation_only_happens_when_too_many_entries() {
        let mut entries = vec![entry("a", false), entry("b", false), entry("c", false)];
        assert!(!truncate_entries(&mut entries, 3));
        assert_eq!(entries.len(), 3);

        assert!(truncate_entries(&mut entries, 2));
        assert_eq!(names(&entries), vec!["a", "b"]);
    }

    #[test]
    fn truncation_limit_matches_the_documented_2000() {
        let mut entries: Vec<DirEntryInfo> = (0..(MAX_ENTRIES + 5))
            .map(|index| entry(&format!("file-{index:04}"), false))
            .collect();
        assert!(truncate_entries(&mut entries, MAX_ENTRIES));
        assert_eq!(entries.len(), MAX_ENTRIES);
    }

    #[test]
    fn dot_files_are_hidden() {
        assert!(is_hidden(".bashrc"));
        assert!(is_hidden(".git"));
        assert!(!is_hidden("bashrc"));
        assert!(!is_hidden("a.b"));
    }

    #[test]
    fn sizes_are_formatted_for_humans() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(999), "999 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn content_types_are_classified() {
        assert_eq!(preview_kind("text/plain"), PreviewKind::Text);
        assert_eq!(preview_kind("text/plain; charset=utf-8"), PreviewKind::Text);
        assert_eq!(preview_kind("application/json"), PreviewKind::Text);
        assert_eq!(preview_kind("application/x-shellscript"), PreviewKind::Text);
        assert_eq!(preview_kind("image/png"), PreviewKind::Image);
        assert_eq!(
            preview_kind("application/x-executable"),
            PreviewKind::Binary
        );
        assert_eq!(
            preview_kind("application/octet-stream"),
            PreviewKind::Binary
        );
    }

    #[test]
    fn html_files_are_recognised() {
        assert!(is_html(&PathBuf::from("/tmp/test.html")));
        assert!(is_html(&PathBuf::from("/tmp/test.HTM")));
        assert!(!is_html(&PathBuf::from("/tmp/test.txt")));
        assert!(!is_html(&PathBuf::from("/tmp/test")));
    }
}
