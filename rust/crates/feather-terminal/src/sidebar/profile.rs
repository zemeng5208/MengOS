//! 侧栏第 5 页：个人资料。T16 实现。

use std::path::Path;

use super::{build_placeholder_page, set_page_cwd, PageView};

pub const ID: &str = "profile";
pub const TITLE: &str = "个人资料";
/// 图标名由 CI 在 `/usr/share/icons` 下核实存在（见 `rust.yml` 的图标检查步骤）。
pub const ICON: &str = "avatar-default-symbolic";

/// 建这一页（T12 只有占位内容）。
pub fn build() -> PageView {
    build_placeholder_page(TITLE, "尚未实现（T16）")
}

/// 当前目录变化时调用（个人资料页暂时用不到，保持与其他页面一致的接口）。
pub fn set_current_dir(view: &PageView, dir: Option<&Path>) {
    set_page_cwd(view, dir);
}
