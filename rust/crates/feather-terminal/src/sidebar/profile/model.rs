//! 个人资料页的纯逻辑：解析 zsh 历史、`/proc/uptime`、`/etc/os-release`、
//! `/proc/self/status`、`/etc/passwd`、`/etc/group`。
//!
//! 全部只依赖标准库，样例放 `tests/fixtures/`，便于直接单元测试。

/// zsh 扩展历史格式是 `: 1700000000:0;命令`，这里去掉前缀只留命令。
///
/// 不是扩展格式的行（普通 `~/.zsh_history`）原样返回。
pub fn strip_history_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix(':') else {
        return trimmed;
    };
    // 期望形式：` <时间戳>:<耗时>;<命令>`
    let Some((stamps, command)) = rest.split_once(';') else {
        return trimmed;
    };
    let stamps = stamps.trim();
    let mut parts = stamps.split(':');
    let (Some(timestamp), Some(duration), None) = (parts.next(), parts.next(), parts.next()) else {
        return trimmed;
    };
    if timestamp.chars().all(|c| c.is_ascii_digit())
        && duration.chars().all(|c| c.is_ascii_digit())
        && !timestamp.is_empty()
    {
        command.trim_start()
    } else {
        trimmed
    }
}

/// 一行历史记录里的命令名（只取第一个单词，不保留参数）。
///
/// 空行、注释行返回 `None`。
pub fn command_name(line: &str) -> Option<String> {
    let command = strip_history_prefix(line).trim();
    if command.is_empty() || command.starts_with('#') {
        return None;
    }
    let name = command.split_whitespace().next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// 统计常用命令：按次数从多到少，次数相同按命令名字典序。
pub fn top_commands(history: &str, limit: usize) -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();

    for line in history.lines() {
        let Some(name) = command_name(line) else {
            continue;
        };
        match counts.iter_mut().find(|(known, _)| *known == name) {
            Some((_, count)) => *count += 1,
            None => counts.push((name, 1)),
        }
    }

    counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    counts.truncate(limit);
    counts
}

/// `/proc/uptime` 的第一个数字是开机秒数。
pub fn parse_uptime(content: &str) -> Option<u64> {
    let first = content.split_whitespace().next()?;
    let seconds: f64 = first.parse().ok()?;
    if seconds.is_finite() && seconds >= 0.0 {
        Some(seconds as u64)
    } else {
        None
    }
}

/// 开机时长写成人话。
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{days} 天 {hours} 小时")
    } else if hours > 0 {
        format!("{hours} 小时 {minutes} 分")
    } else if minutes > 0 {
        format!("{minutes} 分 {secs} 秒")
    } else {
        format!("{secs} 秒")
    }
}

/// 从 `/etc/os-release` 里取 `PRETTY_NAME`（去掉两侧引号）。
pub fn parse_os_release(content: &str) -> Option<String> {
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "PRETTY_NAME" {
            continue;
        }
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|text| text.strip_suffix('"'))
            .unwrap_or(value);
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

/// 从 `/proc/self/status` 里取（真实 UID、真实 GID、附加组列表）。
pub fn parse_self_status(content: &str) -> Option<(u32, u32, Vec<u32>)> {
    let mut uid = None;
    let mut gid = None;
    let mut groups = Vec::new();

    for line in content.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let numbers: Vec<u32> = value
            .split_whitespace()
            .filter_map(|text| text.parse().ok())
            .collect();
        match key.trim() {
            "Uid" => uid = numbers.first().copied(),
            "Gid" => gid = numbers.first().copied(),
            "Groups" => groups = numbers,
            _ => {}
        }
    }

    Some((uid?, gid?, groups))
}

/// 在 `/etc/passwd` 里按 UID 找用户名。
pub fn username_for_uid(passwd: &str, uid: u32) -> Option<String> {
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 4 {
            continue;
        }
        if fields[2].trim().parse::<u32>().ok() == Some(uid) {
            return Some(fields[0].to_string());
        }
    }
    None
}

/// 在 `/etc/group` 里把 GID 换成组名；查不到的保留数字。
pub fn group_names(group: &str, gids: &[u32]) -> Vec<String> {
    let mut names = Vec::with_capacity(gids.len());

    for gid in gids {
        let mut found = None;
        for line in group.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() < 3 {
                continue;
            }
            if fields[2].trim().parse::<u32>().ok() == Some(*gid) {
                found = Some(fields[0].to_string());
                break;
            }
        }
        names.push(found.unwrap_or_else(|| gid.to_string()));
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZSH_HISTORY: &str = include_str!("../../../tests/fixtures/zsh-history.txt");
    const PROC_UPTIME: &str = include_str!("../../../tests/fixtures/proc-uptime.txt");
    const OS_RELEASE: &str = include_str!("../../../tests/fixtures/os-release.txt");
    const SELF_STATUS: &str = include_str!("../../../tests/fixtures/proc-self-status.txt");
    const PASSWD: &str = include_str!("../../../tests/fixtures/passwd.txt");
    const GROUP: &str = include_str!("../../../tests/fixtures/group.txt");

    #[test]
    fn extended_history_prefix_is_stripped() {
        assert_eq!(
            strip_history_prefix(": 1700000000:0;git status"),
            "git status"
        );
        assert_eq!(strip_history_prefix(": 1700000000:12;ls -la"), "ls -la");
        // 不是扩展格式就原样返回
        assert_eq!(strip_history_prefix("ls -la"), "ls -la");
        // 前缀不像时间戳（不是数字）时不剥
        assert_eq!(strip_history_prefix(": abc:0;ls"), ": abc:0;ls");
    }

    #[test]
    fn only_the_command_name_is_kept() {
        assert_eq!(
            command_name("git commit -m 密码123").as_deref(),
            Some("git")
        );
        assert_eq!(
            command_name(": 1700000000:0;echo 秘密参数").as_deref(),
            Some("echo")
        );
        assert_eq!(command_name("  ls  ").as_deref(), Some("ls"));
        assert_eq!(command_name(""), None);
        assert_eq!(command_name("   "), None);
        assert_eq!(command_name("# 注释不算命令"), None);
    }

    #[test]
    fn history_fixture_is_counted() {
        let top = top_commands(ZSH_HISTORY, 10);
        let names: Vec<&str> = top.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names.first().copied(), Some("git"));
        assert_eq!(top[0].1, 3, "样例里 git 出现 3 次");
        assert!(names.contains(&"ls"));
        assert!(names.contains(&"echo"));
        assert!(!names.contains(&"#"), "注释行不能被统计");
        assert!(
            !top.iter().any(|(name, _)| name.contains("密码")),
            "参数不能进统计结果"
        );
    }

    #[test]
    fn top_commands_respects_the_limit_and_ties() {
        let history = "ls\nls\ngit\ngit\npwd\n";
        let top = top_commands(history, 2);
        assert_eq!(top.len(), 2);
        // 次数相同时按命令名字典序：git 在 ls 前面
        assert_eq!(top[0], ("git".to_string(), 2));
        assert_eq!(top[1], ("ls".to_string(), 2));
        assert_eq!(top_commands("", 10).len(), 0);
    }

    #[test]
    fn uptime_is_parsed_and_formatted() {
        assert_eq!(parse_uptime(PROC_UPTIME), Some(93_784));
        assert_eq!(parse_uptime("0.00 0.00"), Some(0));
        assert_eq!(parse_uptime(""), None);
        assert_eq!(format_uptime(93_784), "1 天 2 小时");
        assert_eq!(format_uptime(3_723), "1 小时 2 分");
        assert_eq!(format_uptime(125), "2 分 5 秒");
        assert_eq!(format_uptime(9), "9 秒");
    }

    #[test]
    fn os_release_uses_pretty_name() {
        assert_eq!(
            parse_os_release(OS_RELEASE).as_deref(),
            Some("Debian GNU/Linux 13 (trixie)")
        );
        assert_eq!(parse_os_release("NAME=Debian\n"), None);
    }

    #[test]
    fn self_status_gives_uid_gid_and_groups() {
        let (uid, gid, groups) = parse_self_status(SELF_STATUS).expect("样例能解析");
        assert_eq!(uid, 1000);
        assert_eq!(gid, 1000);
        assert_eq!(groups, vec![1000, 27, 999]);
        assert_eq!(parse_self_status("Name:\tbash\n"), None);
    }

    #[test]
    fn passwd_and_group_lookups() {
        assert_eq!(username_for_uid(PASSWD, 1000).as_deref(), Some("student"));
        assert_eq!(username_for_uid(PASSWD, 4242), None);

        let names = group_names(GROUP, &[1000, 27, 7777]);
        assert_eq!(names, vec!["student", "sudo", "7777"]);
    }
}
