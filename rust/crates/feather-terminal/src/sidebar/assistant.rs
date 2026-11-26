//! 侧栏第 4 页：终端助手（第三批的 FeatherBot）。本批不实现。

use std::path::Path;

use super::{build_placeholder_page, set_page_cwd, PageView};

pub const ID: &str = "assistant";
pub const TITLE: &str = "终端助手";
/// 图标名由 CI 在 `/usr/share/icons` 下核实存在（见 `rust.yml` 的图标检查步骤）。
pub const ICON: &str = "dialog-information-symbolic";

/// 建这一页（T12 只有占位内容）。
pub fn build() -> PageView {
    build_placeholder_page(TITLE, "尚未实现（第三批：FeatherBot）")
}

/// 当前目录变化时调用（助手页暂时用不到，保持与其他页面一致的接口）。
pub fn set_current_dir(view: &PageView, dir: Option<&Path>) {
    set_page_cwd(view, dir);
}
