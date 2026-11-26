//! 侧栏设置的读写：`~/.config/mengos/feather-terminal.toml`。
//!
//! 只存两个值：侧栏宽度、侧栏是否显示。文件不存在、内容损坏、没有权限等情况
//! 都不能让程序崩溃——一律退回默认值，并在 stderr 打印一行警告。

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 侧栏最小宽度（任务书要求 260 像素）。
pub const SIDEBAR_MIN_WIDTH: i32 = 260;
/// 侧栏默认宽度（任务书要求 360 像素）。
pub const SIDEBAR_DEFAULT_WIDTH: i32 = 360;
/// 侧栏最大宽度，防止窗口很宽时侧栏把终端挤没。
pub const SIDEBAR_MAX_WIDTH: i32 = 1200;

/// 侧栏设置。字段都带默认值，所以旧版本写下的文件少字段也能读。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SidebarSettings {
    pub width: i32,
    pub visible: bool,
}

impl Default for SidebarSettings {
    fn default() -> Self {
        SidebarSettings {
            width: SIDEBAR_DEFAULT_WIDTH,
            visible: true,
        }
    }
}

impl SidebarSettings {
    /// 把宽度收进 `[260, 1200]`：手改配置文件写成 0 或负数时也不会把侧栏挤没。
    pub fn clamped(mut self) -> Self {
        self.width = self.width.clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        self
    }

    /// 解析配置文件内容；格式错误返回错误信息（调用方负责退回默认值）。
    pub fn from_toml(text: &str) -> Result<Self, String> {
        toml::from_str::<SidebarSettings>(text)
            .map(SidebarSettings::clamped)
            .map_err(|error| error.to_string())
    }

    /// 序列化成配置文件内容。
    pub fn to_toml(&self) -> String {
        let mut text = String::from("# MengOS FeatherTerminal 的窗口设置，由程序自动写入\n");
        match toml::to_string(&self.clamped()) {
            Ok(body) => text.push_str(&body),
            Err(error) => eprintln!("警告：侧栏设置序列化失败（{error}）"),
        }
        text
    }
}

/// 配置文件路径：`$XDG_CONFIG_HOME/mengos/feather-terminal.toml`，
/// 没有 `XDG_CONFIG_HOME` 时用 `~/.config/mengos/feather-terminal.toml`。
pub fn config_path() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return Some(
                PathBuf::from(dir)
                    .join("mengos")
                    .join("feather-terminal.toml"),
            );
        }
    }
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("mengos")
            .join("feather-terminal.toml")
    })
}

/// 读设置。文件不存在是正常情况，直接用默认值、不打印警告；
/// 读不了或解析不了才打印警告，然后退回默认值。
pub fn load() -> SidebarSettings {
    let Some(path) = config_path() else {
        eprintln!("警告：找不到 HOME 环境变量，侧栏设置使用默认值");
        return SidebarSettings::default();
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "警告：读不了 {}（{error}），侧栏设置使用默认值",
                    path.display()
                );
            }
            return SidebarSettings::default();
        }
    };
    match SidebarSettings::from_toml(&text) {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!(
                "警告：{} 内容无法解析（{error}），侧栏设置使用默认值",
                path.display()
            );
            SidebarSettings::default()
        }
    }
}

/// 写设置。目录不存在就创建；写失败只打印警告，不影响程序继续运行。
pub fn save(settings: &SidebarSettings) {
    let Some(path) = config_path() else {
        eprintln!("警告：找不到 HOME 环境变量，侧栏设置没有保存");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!(
                "警告：创建目录 {} 失败（{error}），侧栏设置没有保存",
                parent.display()
            );
            return;
        }
    }
    if let Err(error) = fs::write(&path, settings.to_toml()) {
        eprintln!("警告：写 {} 失败（{error}）", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::{SidebarSettings, SIDEBAR_DEFAULT_WIDTH, SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};

    #[test]
    fn default_values_match_spec() {
        let settings = SidebarSettings::default();
        assert_eq!(settings.width, 360);
        assert_eq!(settings.width, SIDEBAR_DEFAULT_WIDTH);
        assert!(settings.visible);
    }

    #[test]
    fn parses_saved_file() {
        let settings = SidebarSettings::from_toml("width = 420\nvisible = false\n")
            .expect("正常的配置文件应当解析成功");
        assert_eq!(settings.width, 420);
        assert!(!settings.visible);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let settings =
            SidebarSettings::from_toml("visible = false\n").expect("缺字段时应当用默认值补齐");
        assert_eq!(settings.width, SIDEBAR_DEFAULT_WIDTH);
        assert!(!settings.visible);
    }

    #[test]
    fn width_is_clamped_into_range() {
        let tiny = SidebarSettings::from_toml("width = 10\nvisible = true\n")
            .expect("宽度过小也应当能解析");
        assert_eq!(tiny.width, SIDEBAR_MIN_WIDTH);

        let huge = SidebarSettings::from_toml("width = 99999\nvisible = true\n")
            .expect("宽度过大也应当能解析");
        assert_eq!(huge.width, SIDEBAR_MAX_WIDTH);
    }

    #[test]
    fn broken_file_is_an_error() {
        assert!(SidebarSettings::from_toml("width = ???\n").is_err());
    }

    #[test]
    fn round_trip() {
        let settings = SidebarSettings {
            width: 512,
            visible: false,
        };
        let text = settings.to_toml();
        let parsed = SidebarSettings::from_toml(&text).expect("自己写出的文件应当能读回来");
        assert_eq!(parsed, settings);
    }
}
