//! 网络页的纯逻辑：十六进制地址/端口解析、TCP 状态码、`/proc/net` 表解析、进程树。
//!
//! 任务书要求「解析代码自己写成纯函数（不引入额外 crate）」：
//! 这里只用到标准库（`std::net::Ipv6Addr` 只用来做地址格式化），
//! 逻辑简单，便于用 `tests/fixtures/` 里的 `/proc/net` 样例直接测试。

/// `/proc/net/tcp`、`tcp6`、`udp`、`udp6` 四张表。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Tcp6,
    Udp,
    Udp6,
}

impl Protocol {
    pub fn label(self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Tcp6 => "TCP6",
            Protocol::Udp => "UDP",
            Protocol::Udp6 => "UDP6",
        }
    }

    fn is_ipv6(self) -> bool {
        matches!(self, Protocol::Tcp6 | Protocol::Udp6)
    }

    fn is_udp(self) -> bool {
        matches!(self, Protocol::Udp | Protocol::Udp6)
    }
}

/// `/proc/net/*` 里的一行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetEntry {
    pub protocol: Protocol,
    /// 本地地址:端口
    pub local: String,
    /// 远端地址:端口（监听时是 0.0.0.0:0）
    pub remote: String,
    pub state: &'static str,
    pub inode: u64,
}

impl NetEntry {
    pub fn is_listening(&self) -> bool {
        self.state == "LISTEN"
    }
}

/// 解析 8 位十六进制小端 IPv4，例如 `0100007F` → `127.0.0.1`。
pub fn parse_ipv4(hex: &str) -> Option<String> {
    if hex.len() != 8 {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    let bytes = value.to_le_bytes();
    Some(format!(
        "{}.{}.{}.{}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    ))
}

/// 解析 32 位十六进制小端 IPv6（每 8 位是一个 32 位小端字）。
pub fn parse_ipv6(hex: &str) -> Option<String> {
    if hex.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for index in 0..4 {
        let chunk = hex.get(index * 8..index * 8 + 8)?;
        let word = u32::from_str_radix(chunk, 16).ok()?;
        bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    Some(std::net::Ipv6Addr::from(bytes).to_string())
}

/// 解析端口（4 位十六进制，大端），例如 `1F90` → 8000。
pub fn parse_port(hex: &str) -> Option<u16> {
    u16::from_str_radix(hex, 16).ok()
}

/// 解析 `地址:端口` 字段，例如 `0100007F:1F90` → (`127.0.0.1`, 8000)。
pub fn parse_address(field: &str, ipv6: bool) -> Option<(String, u16)> {
    let (address, port) = field.split_once(':')?;
    let ip = if ipv6 {
        parse_ipv6(address)?
    } else {
        parse_ipv4(address)?
    };
    Some((ip, parse_port(port)?))
}

/// TCP 状态码（两位十六进制）转文字。
pub fn tcp_state(code: &str) -> &'static str {
    match code.to_ascii_uppercase().as_str() {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        "0C" => "NEW_SYN_RECV",
        _ => "UNKNOWN",
    }
}

/// 解析一张 `/proc/net/*` 表（跳过表头）。
pub fn parse_net_table(content: &str, protocol: Protocol) -> Vec<NetEntry> {
    let mut entries = Vec::new();

    for line in content.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        // 至少要有到 inode 那一列
        if fields.len() < 10 {
            continue;
        }
        let Some((local_ip, local_port)) = parse_address(fields[1], protocol.is_ipv6()) else {
            continue;
        };
        let Some((remote_ip, remote_port)) = parse_address(fields[2], protocol.is_ipv6()) else {
            continue;
        };
        let Ok(inode) = fields[9].parse::<u64>() else {
            continue;
        };

        let state = if protocol.is_udp() {
            // UDP 没有 LISTEN，状态码 07（CLOSE）就是「未连接」
            match tcp_state(fields[3]) {
                "CLOSE" => "UDP",
                other => other,
            }
        } else {
            tcp_state(fields[3])
        };

        entries.push(NetEntry {
            protocol,
            local: format!("{local_ip}:{local_port}"),
            remote: format!("{remote_ip}:{remote_port}"),
            state,
            inode,
        });
    }

    entries
}

/// 从 `/proc/<pid>/stat` 里取父进程号。
///
/// 进程名（第 2 个字段）可能包含空格和括号，所以从最后一个 `)` 之后开始取字段。
pub fn parse_stat_ppid(content: &str) -> Option<i32> {
    let after_comm = content.rsplit_once(')')?.1;
    let mut fields = after_comm.split_whitespace();
    fields.next()?; // 状态字母，例如 S
    fields.next()?.parse().ok()
}

/// 把若干行 `/proc/<pid>/stat` 内容解析成（pid, 父 pid）列表。
pub fn parse_proc_stat_table(content: &str) -> Vec<(i32, i32)> {
    let mut pairs = Vec::new();
    for line in content.lines() {
        let Some((pid_text, rest)) = line.split_once(' ') else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<i32>() else {
            continue;
        };
        if let Some(ppid) = parse_stat_ppid(rest) {
            pairs.push((pid, ppid));
        }
    }
    pairs
}

/// 从 `root` 出发，算出它自己加所有子孙进程的 PID。
pub fn descendant_pids(parents: &[(i32, i32)], root: i32) -> Vec<i32> {
    let mut result = vec![root];
    let mut index = 0;
    while index < result.len() {
        let parent = result[index];
        for (pid, ppid) in parents {
            if *ppid == parent && !result.contains(pid) {
                result.push(*pid);
            }
        }
        index += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROC_NET_TCP: &str = include_str!("../../../tests/fixtures/proc-net-tcp.txt");
    const PROC_NET_TCP6: &str = include_str!("../../../tests/fixtures/proc-net-tcp6.txt");
    const PROC_NET_UDP: &str = include_str!("../../../tests/fixtures/proc-net-udp.txt");
    const PROC_STAT: &str = include_str!("../../../tests/fixtures/proc-stat.txt");

    #[test]
    fn ipv4_hex_is_little_endian() {
        assert_eq!(parse_ipv4("0100007F").as_deref(), Some("127.0.0.1"));
        assert_eq!(parse_ipv4("00000000").as_deref(), Some("0.0.0.0"));
        assert_eq!(parse_ipv4("08080808").as_deref(), Some("8.8.8.8"));
        assert_eq!(parse_ipv4("0100"), None);
    }

    #[test]
    fn ipv6_loopback_and_mapped() {
        assert_eq!(
            parse_ipv6("00000000000000000000000001000000").as_deref(),
            Some("::1")
        );
        assert_eq!(
            parse_ipv6("0000000000000000FFFF00000100007F").as_deref(),
            Some("::ffff:127.0.0.1")
        );
    }

    #[test]
    fn port_hex_is_big_endian() {
        assert_eq!(parse_port("1F90"), Some(8000));
        assert_eq!(parse_port("01BB"), Some(443));
        assert_eq!(parse_port("0050"), Some(80));
        assert_eq!(parse_port("ZZZZ"), None);
    }

    #[test]
    fn address_field_splits_ip_and_port() {
        assert_eq!(
            parse_address("0100007F:1F90", false),
            Some(("127.0.0.1".to_string(), 8000))
        );
        assert_eq!(
            parse_address("00000000000000000000000001000000:0050", true),
            Some(("::1".to_string(), 80))
        );
        assert_eq!(parse_address("broken", false), None);
    }

    #[test]
    fn tcp_states_are_translated() {
        assert_eq!(tcp_state("01"), "ESTABLISHED");
        assert_eq!(tcp_state("0A"), "LISTEN");
        assert_eq!(tcp_state("0a"), "LISTEN");
        assert_eq!(tcp_state("08"), "CLOSE_WAIT");
        assert_eq!(tcp_state("FF"), "UNKNOWN");
    }

    #[test]
    fn tcp_table_from_fixture() {
        let entries = parse_net_table(PROC_NET_TCP, Protocol::Tcp);
        assert_eq!(entries.len(), 2, "样例里有一行监听 + 一行已建立");

        let listen = &entries[0];
        assert_eq!(listen.local, "127.0.0.1:8000");
        assert_eq!(listen.remote, "0.0.0.0:0");
        assert_eq!(listen.state, "LISTEN");
        assert_eq!(listen.inode, 20001);
        assert!(listen.is_listening());

        let established = &entries[1];
        assert_eq!(established.local, "127.0.0.1:50000");
        assert_eq!(established.remote, "127.0.0.1:8000");
        assert_eq!(established.state, "ESTABLISHED");
        assert_eq!(established.inode, 20002);
    }

    #[test]
    fn tcp6_table_from_fixture() {
        let entries = parse_net_table(PROC_NET_TCP6, Protocol::Tcp6);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].local, "::1:80");
        assert_eq!(entries[0].remote, "::ffff:127.0.0.1:443");
        assert_eq!(entries[0].state, "ESTABLISHED");
        assert_eq!(entries[0].protocol.label(), "TCP6");
    }

    #[test]
    fn udp_table_from_fixture() {
        let entries = parse_net_table(PROC_NET_UDP, Protocol::Udp);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].local, "0.0.0.0:53");
        assert_eq!(entries[0].state, "UDP", "UDP 的 07 状态显示成 UDP");
        assert_eq!(entries[0].inode, 20004);
    }

    #[test]
    fn malformed_table_lines_are_skipped() {
        let content = "header\n\n 0: garbage\n 1: 0100007F:1F90 00000000:0000 0A 0:0 00:0 0 1000 0 notanumber 1\n";
        assert!(parse_net_table(content, Protocol::Tcp).is_empty());
    }

    #[test]
    fn stat_ppid_handles_spaces_in_process_name() {
        let line = "1001 (python3 -m http.server) S 1000 1001 1001 34816 1001 4194304 0 0";
        assert_eq!(parse_stat_ppid(line), Some(1000));
        // 进程名里带括号也不能切错
        let weird = "1002 (a) b) S 1001 1002 1002";
        assert_eq!(parse_stat_ppid(weird), Some(1001));
        assert_eq!(parse_stat_ppid("no parenthesis here"), None);
    }

    #[test]
    fn process_tree_from_fixture() {
        let parents = parse_proc_stat_table(PROC_STAT);
        assert_eq!(parents.len(), 4);
        assert_eq!(
            descendant_pids(&parents, 1000),
            vec![1000, 1001, 1002],
            "1000 的子孙是 1001 和 1002，2000 不属于这棵树"
        );
        assert_eq!(descendant_pids(&parents, 2000), vec![2000]);
    }

    #[test]
    fn process_tree_ignores_cycles_and_unknown_root() {
        let parents = vec![(1, 2), (2, 1)];
        let pids = descendant_pids(&parents, 1);
        assert_eq!(pids, vec![1, 2], "互相为父子的记录不能无限循环");
        assert_eq!(descendant_pids(&parents, 99), vec![99]);
    }
}
