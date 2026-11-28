//! 从本机读个人资料的真实数据。所有函数都在后台线程里调用。
//!
//! 数据来源（一个都不编造，读不到就显示「未知」）：
//! - 用户名 / UID / 组：`/proc/self/status` + `/etc/passwd` + `/etc/group`
//! - 主机名：`/proc/sys/kernel/hostname`
//! - 系统版本：`/etc/os-release` 的 `PRETTY_NAME`
//! - 内核版本：`/proc/sys/kernel/osrelease`（就是 `uname -r`）
//! - 当前 shell：环境变量 `SHELL`
//! - 开机时长：`/proc/uptime`
//! - 常用命令：`~/.zsh_history`（或 `$HISTFILE`）

use std::path::PathBuf;

use super::model;

/// 历史文件的读取情况。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryState {
    /// 读到了，可以显示 Top 10
    Loaded,
    /// 文件不存在（还没用过 zsh 历史）
    Missing,
    /// 文件存在但读不出来
    Unreadable,
}

/// 一次读到的全部个人资料。
#[derive(Debug, Clone)]
pub struct Profile {
    pub username: String,
    pub uid: String,
    pub groups: String,
    pub hostname: String,
    pub os_version: String,
    pub kernel: String,
    pub shell: String,
    pub version: String,
    pub uptime: String,
    pub top_commands: Vec<(String, usize)>,
    pub history_state: HistoryState,
}

const UNKNOWN: &str = "未知";

/// 读一遍本机资料。`version` 由调用方传入（编译期注入的版本号）。
pub fn collect(version: &str) -> Profile {
    let (uid, gid, group_ids) = read_self_status();
    let passwd = read_file("/etc/passwd");
    let group = read_file("/etc/group");

    let username = match (passwd.as_deref(), uid) {
        (Some(content), Some(uid)) => {
            model::username_for_uid(content, uid).unwrap_or_else(|| UNKNOWN.to_string())
        }
        _ => UNKNOWN.to_string(),
    };

    let mut wanted: Vec<u32> = Vec::new();
    if let Some(gid) = gid {
        wanted.push(gid);
    }
    for id in &group_ids {
        if !wanted.contains(id) {
            wanted.push(*id);
        }
    }
    let groups = match group.as_deref() {
        Some(content) if !wanted.is_empty() => {
            let names = model::group_names(content, &wanted);
            if names.is_empty() {
                UNKNOWN.to_string()
            } else {
                names.join("、")
            }
        }
        _ => UNKNOWN.to_string(),
    };

    let hostname = read_trimmed("/proc/sys/kernel/hostname").unwrap_or_else(|| UNKNOWN.to_string());
    let os_version = read_file("/etc/os-release")
        .and_then(|content| model::parse_os_release(&content))
        .unwrap_or_else(|| UNKNOWN.to_string());
    let kernel = read_trimmed("/proc/sys/kernel/osrelease").unwrap_or_else(|| UNKNOWN.to_string());
    let shell = std::env::var("SHELL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| UNKNOWN.to_string());

    let uptime = read_file("/proc/uptime")
        .and_then(|content| model::parse_uptime(&content))
        .map(model::format_uptime)
        .unwrap_or_else(|| UNKNOWN.to_string());

    let (top_commands, history_state) = read_history();

    Profile {
        username,
        uid: uid
            .map(|value| value.to_string())
            .unwrap_or_else(|| UNKNOWN.to_string()),
        groups,
        hostname,
        os_version,
        kernel,
        shell,
        version: version.to_string(),
        uptime,
        top_commands,
        history_state,
    }
}

/// 历史文件位置：优先 `$HISTFILE`，否则 `~/.zsh_history`。
fn history_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("HISTFILE") {
        if !path.trim().is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".zsh_history"))
}

fn read_history() -> (Vec<(String, usize)>, HistoryState) {
    let Some(path) = history_path() else {
        return (Vec::new(), HistoryState::Missing);
    };
    if !path.exists() {
        return (Vec::new(), HistoryState::Missing);
    }
    // zsh 历史可能是 latin1 存的中文；读不出来时按「不可读」处理，不猜内容
    let Ok(bytes) = std::fs::read(&path) else {
        return (Vec::new(), HistoryState::Unreadable);
    };
    let Ok(content) = String::from_utf8(bytes) else {
        return (Vec::new(), HistoryState::Unreadable);
    };
    (model::top_commands(&content, 10), HistoryState::Loaded)
}

fn read_self_status() -> (Option<u32>, Option<u32>, Vec<u32>) {
    let Some(content) = read_file("/proc/self/status") else {
        return (None, None, Vec::new());
    };
    match model::parse_self_status(&content) {
        Some((uid, gid, groups)) => (Some(uid), Some(gid), groups),
        None => (None, None, Vec::new()),
    }
}

fn read_file(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn read_trimmed(path: &str) -> Option<String> {
    let content = read_file(path)?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_path_prefers_histfile() {
        // 只测「有 HOME 时能拼出 ~/.zsh_history」，不改环境变量以免影响别的测试
        let path = history_path();
        if let Ok(home) = std::env::var("HOME") {
            if std::env::var("HISTFILE").is_err() {
                assert_eq!(path, Some(PathBuf::from(home).join(".zsh_history")));
            }
        }
    }
}
