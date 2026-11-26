//! 从 `/proc` 读真实的进程树和连接表。所有函数都在后台线程里调用。

use std::collections::HashMap;

use super::model::{self, Protocol};

/// 一个连接，连同它属于哪个进程。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub process: String,
    pub pid: i32,
    /// `TCP` / `TCP6` / `UDP` / `UDP6`
    pub protocol: &'static str,
    /// 本地地址:端口
    pub local: String,
    /// 远端地址:端口
    pub remote: String,
    pub state: &'static str,
    /// 这一条是不是「监听中的端口」（由 /proc/net 的状态码判定）
    pub listening: bool,
}

impl Connection {
    pub fn is_listening(&self) -> bool {
        self.listening
    }
}

/// 一次刷新读到的全部内容。
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub connections: Vec<Connection>,
    /// 这次一共看了多少个进程（含 shell 自己）
    pub process_count: usize,
}

/// 读一次「当前标签的 shell 及其所有子孙进程」的真实连接。
pub fn snapshot(root_pid: i32) -> Snapshot {
    let pids = process_tree(root_pid);
    let owners = socket_owners(&pids);

    let mut connections = Vec::new();
    for (protocol, path) in [
        (Protocol::Tcp, "/proc/net/tcp"),
        (Protocol::Tcp6, "/proc/net/tcp6"),
        (Protocol::Udp, "/proc/net/udp"),
        (Protocol::Udp6, "/proc/net/udp6"),
    ] {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for entry in model::parse_net_table(&content, protocol) {
            let Some((process, pid)) = owners.get(&entry.inode) else {
                continue;
            };
            connections.push(Connection {
                process: process.clone(),
                pid: *pid,
                protocol: protocol.label(),
                local: entry.local,
                remote: entry.remote,
                state: entry.state,
                listening: entry.is_listening(),
            });
        }
    }

    // 监听端口排前面，其余按进程名、协议、远端地址排
    connections.sort_by(|left, right| {
        right
            .is_listening()
            .cmp(&left.is_listening())
            .then_with(|| left.process.cmp(&right.process))
            .then_with(|| left.protocol.cmp(right.protocol))
            .then_with(|| left.remote.cmp(&right.remote))
    });

    Snapshot {
        connections,
        process_count: pids.len(),
    }
}

/// 从 `root` 出发，用 `/proc/*/stat` 的父进程号找出所有子孙进程。
fn process_tree(root: i32) -> Vec<i32> {
    // 把所有 /proc/<pid>/stat 拼成一张表再解析：内容第一个字段就是 PID
    let mut table = String::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let Ok(content) = std::fs::read_to_string(entry.path().join("stat")) else {
                continue;
            };
            if content.is_empty() {
                continue;
            }
            table.push_str(&content);
            if !content.ends_with('\n') {
                table.push('\n');
            }
        }
    }

    let parents = model::parse_proc_stat_table(&table);
    model::descendant_pids(&parents, root)
}

/// socket inode → （进程名, PID）。
fn socket_owners(pids: &[i32]) -> HashMap<u64, (String, i32)> {
    let mut owners = HashMap::new();

    for pid in pids {
        let Some(process) = process_name(*pid) else {
            continue;
        };
        let Ok(entries) = std::fs::read_dir(format!("/proc/{pid}/fd")) else {
            // 读不到（权限不足 / 进程已退出）就跳过
            continue;
        };
        for entry in entries.flatten() {
            let Ok(target) = std::fs::read_link(entry.path()) else {
                continue;
            };
            let Some(inode) = socket_inode(&target.to_string_lossy()) else {
                continue;
            };
            owners.insert(inode, (process.clone(), *pid));
        }
    }

    owners
}

/// `socket:[12345]` → `12345`。
fn socket_inode(target: &str) -> Option<u64> {
    let rest = target.strip_prefix("socket:[")?;
    rest.strip_suffix(']')?.parse().ok()
}

fn process_name(pid: i32) -> Option<String> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    let name = text.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_links_are_parsed() {
        assert_eq!(socket_inode("socket:[20001]"), Some(20001));
        assert_eq!(socket_inode("socket:[0]"), Some(0));
        assert_eq!(socket_inode("/dev/pts/0"), None);
        assert_eq!(socket_inode("socket:[abc]"), None);
        assert_eq!(socket_inode("socket:[]"), None);
    }
}
