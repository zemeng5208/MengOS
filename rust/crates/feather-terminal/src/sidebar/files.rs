//! 侧栏第 3 页：文件树。T13 实现。

use std::path::Path;

use super::{build_placeholder_page, set_page_cwd, PageView};

pub const ID: &str = "files";
pub const TITLE: &str = "文件树";
/// 图标名由 CI 在 `/usr/share/icons` 下核实存在（见 `rust.yml` 的图标检查步骤）。
pub const ICON: &str = "folder-symbolic";

/// 建这一页（T12 只有占位内容）。
pub fn build() -> PageView {
    build_placeholder_page(TITLE, "尚未实现（T13）")
}

/// 当前目录变化时调用。T13 会把这个目录作为文件树的根。
pub fn set_current_dir(view: &PageView, dir: Option<&Path>) {
    set_page_cwd(view, dir);
}
