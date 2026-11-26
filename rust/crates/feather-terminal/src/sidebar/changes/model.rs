//! 文件更改页的纯逻辑：解析 `git status --porcelain=v1 -z`、给 diff 每一行分类、截断。
//!
//! 这里不执行任何命令、不碰界面，全部是可以用 `tests/fixtures/` 样例直接测的函数。

/// 一个文件在 Git 里的状态（用于列表上的标记）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Untracked,
    Conflicted,
    Unknown,
}

impl ChangeKind {
    /// 中文说明，给提示行用。
    pub fn label(self) -> &'static str {
        match self {
            ChangeKind::Added => "新增",
            ChangeKind::Modified => "修改",
            ChangeKind::Deleted => "删除",
            ChangeKind::Renamed => "重命名",
            ChangeKind::Copied => "复制",
            ChangeKind::TypeChanged => "类型变化",
            ChangeKind::Untracked => "未跟踪",
            ChangeKind::Conflicted => "冲突",
            ChangeKind::Unknown => "未知",
        }
    }
}

/// `git status --porcelain=v1 -z` 里的一行（一个文件）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeEntry {
    /// 暂存区状态位（`X`），空格表示没有暂存改动
    pub index_status: char,
    /// 工作区状态位（`Y`）
    pub worktree_status: char,
    pub kind: ChangeKind,
    /// 显示用的路径；重命名显示成「新 ← 旧」
    pub display_path: String,
    /// 传给 `git diff` 的路径（重命名取新路径）
    pub diff_path: String,
    pub staged: bool,
    pub unstaged: bool,
}

impl ChangeEntry {
    /// 列表上的状态标记，例如 `M `、` M`、`??`、`R `。
    pub fn marker(&self) -> String {
        format!("{}{}", self.index_status, self.worktree_status)
    }

    /// 是否未跟踪（未跟踪文件没有 diff，直接显示内容）。
    pub fn is_untracked(&self) -> bool {
        self.kind == ChangeKind::Untracked
    }
}

/// diff 里一行的类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Added,
    Removed,
    Hunk,
    Meta,
    Context,
}

/// 最多显示多少行 diff。
pub const MAX_DIFF_LINES: usize = 5000;
/// 未跟踪文件最多预览多少字节。
pub const MAX_PREVIEW_BYTES: usize = 200 * 1024;

/// diff 里属于「文件头 / 元信息」的前缀。
///
/// 注意 `+++ b/x` 与 `--- a/x` 必须以 `Meta` 判定，否则会被当成新增/删除行。
const META_PREFIXES: [&str; 14] = [
    "diff --git ",
    "index ",
    "+++ ",
    "--- ",
    "new file mode ",
    "deleted file mode ",
    "old mode ",
    "new mode ",
    "similarity index ",
    "rename from ",
    "rename to ",
    "copy from ",
    "copy to ",
    "Binary files ",
];

/// 解析 `git status --porcelain=v1 -z` 的输出（字段之间是 `\0`）。
///
/// 重命名/复制的记录后面会跟一个「原路径」字段。
pub fn parse_status(raw: &str) -> Vec<ChangeEntry> {
    let mut entries = Vec::new();
    let mut fields = raw.split('\0');

    while let Some(record) = fields.next() {
        if record.is_empty() {
            continue;
        }
        // 正常记录至少是 "XY p" 四字节；不足就跳过，避免 panic
        if record.len() < 4 {
            continue;
        }
        let mut chars = record.chars();
        let index_status = chars.next().unwrap_or(' ');
        let worktree_status = chars.next().unwrap_or(' ');
        // 前三个字节是 X、Y、空格（都是 ASCII），第 4 字节起才是路径
        let path = record.get(3..).unwrap_or("").to_string();

        let kind = kind_of(index_status, worktree_status);
        let mut display_path = path.clone();
        let mut diff_path = path;

        if matches!(kind, ChangeKind::Renamed | ChangeKind::Copied) {
            if let Some(original) = fields.next() {
                if !original.is_empty() {
                    display_path = format!("{display_path} ← {original}");
                }
            }
        }

        entries.push(ChangeEntry {
            index_status,
            worktree_status,
            kind,
            display_path,
            diff_path: std::mem::take(&mut diff_path),
            staged: index_status != ' ' && index_status != '?',
            unstaged: worktree_status != ' ' && worktree_status != '?',
        });
    }

    entries
}

fn kind_of(index_status: char, worktree_status: char) -> ChangeKind {
    if index_status == '?' && worktree_status == '?' {
        return ChangeKind::Untracked;
    }
    if index_status == 'U'
        || worktree_status == 'U'
        || (index_status == 'A' && worktree_status == 'A')
        || (index_status == 'D' && worktree_status == 'D')
    {
        return ChangeKind::Conflicted;
    }
    for status in [index_status, worktree_status] {
        match status {
            'R' => return ChangeKind::Renamed,
            'C' => return ChangeKind::Copied,
            'T' => return ChangeKind::TypeChanged,
            _ => {}
        }
    }
    for status in [index_status, worktree_status] {
        match status {
            'A' => return ChangeKind::Added,
            'D' => return ChangeKind::Deleted,
            'M' => return ChangeKind::Modified,
            _ => {}
        }
    }
    ChangeKind::Unknown
}

/// 给 diff 的一行分类。
pub fn classify_line(line: &str) -> DiffLineKind {
    if line.starts_with('\\') || META_PREFIXES.iter().any(|prefix| line.starts_with(prefix)) {
        return DiffLineKind::Meta;
    }
    if line.starts_with("@@") {
        return DiffLineKind::Hunk;
    }
    match line.chars().next() {
        Some('+') => DiffLineKind::Added,
        Some('-') => DiffLineKind::Removed,
        _ => DiffLineKind::Context,
    }
}

/// 把文本按行拆开（保留空行，去掉结尾的换行）。
pub fn split_lines(text: &str) -> Vec<String> {
    let trimmed = text.strip_suffix('\n').unwrap_or(text);
    if trimmed.is_empty() {
        return Vec::new();
    }
    trimmed.split('\n').map(|line| line.to_string()).collect()
}

/// 未跟踪文件：内容全部当成「新增行」，超过 `max_bytes` 截断。
///
/// 返回（行、是否因为太大而截断）。
pub fn untracked_lines(content: &str, max_bytes: usize) -> (Vec<String>, bool) {
    let (slice, truncated) = if content.len() > max_bytes {
        // 按字节截断可能切在多字节字符中间，退到最近的字符边界
        let mut end = max_bytes;
        while end > 0 && !content.is_char_boundary(end) {
            end -= 1;
        }
        (&content[..end], true)
    } else {
        (content, false)
    };

    let lines = split_lines(slice)
        .into_iter()
        .map(|line| format!("+{line}"))
        .collect();
    (lines, truncated)
}

/// 只保留前 `max` 行，返回是否发生了截断。
pub fn truncate_lines<T>(lines: &mut Vec<T>, max: usize) -> bool {
    if lines.len() <= max {
        return false;
    }
    lines.truncate(max);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS_BASIC: &str = include_str!("../../../tests/fixtures/status-basic.txt");
    const STATUS_RENAME: &str = include_str!("../../../tests/fixtures/status-rename.txt");
    const STATUS_CHINESE: &str = include_str!("../../../tests/fixtures/status-chinese.txt");
    const STATUS_MIXED: &str = include_str!("../../../tests/fixtures/status-mixed.txt");
    const DIFF_SAMPLE: &str = include_str!("../../../tests/fixtures/diff-sample.txt");

    #[test]
    fn basic_status_entries_are_parsed() {
        let entries = parse_status(STATUS_BASIC);
        assert_eq!(entries.len(), 4, "样例里有 4 个文件");

        assert_eq!(entries[0].display_path, "a.txt");
        assert_eq!(entries[0].kind, ChangeKind::Modified);
        assert!(!entries[0].staged);
        assert!(entries[0].unstaged);

        assert_eq!(entries[1].display_path, "b.txt");
        assert!(entries[1].is_untracked());
        assert_eq!(entries[1].marker(), "??");

        assert_eq!(entries[2].kind, ChangeKind::Added);
        assert!(entries[2].staged);

        assert_eq!(entries[3].kind, ChangeKind::Deleted);
        assert!(entries[3].unstaged);
    }

    #[test]
    fn rename_keeps_both_paths() {
        let entries = parse_status(STATUS_RENAME);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, ChangeKind::Renamed);
        assert_eq!(entries[0].display_path, "new name.txt ← old name.txt");
        // 传给 git diff 的必须是新路径
        assert_eq!(entries[0].diff_path, "new name.txt");
    }

    #[test]
    fn chinese_file_names_survive() {
        let entries = parse_status(STATUS_CHINESE);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].display_path, "中文 文件.txt");
        assert_eq!(entries[0].kind, ChangeKind::Modified);
        assert_eq!(entries[1].display_path, "新文件.txt");
        assert!(entries[1].is_untracked());
    }

    #[test]
    fn mixed_status_marks_both_sides() {
        let entries = parse_status(STATUS_MIXED);
        assert_eq!(entries.len(), 2);

        // MM：暂存区和工作区都有改动
        assert_eq!(entries[0].display_path, "both.txt");
        assert!(entries[0].staged && entries[0].unstaged);
        assert_eq!(entries[0].marker(), "MM");

        // UU：冲突
        assert_eq!(entries[1].kind, ChangeKind::Conflicted);
        assert!(entries[1].staged && entries[1].unstaged);
    }

    #[test]
    fn empty_or_short_input_is_ignored() {
        assert!(parse_status("").is_empty());
        assert!(parse_status("\0\0").is_empty());
        assert!(parse_status("M\0").is_empty());
    }

    #[test]
    fn diff_headers_are_meta_not_changes() {
        assert_eq!(classify_line("diff --git a/x b/x"), DiffLineKind::Meta);
        assert_eq!(
            classify_line("index 1234567..89abcde 100644"),
            DiffLineKind::Meta
        );
        assert_eq!(classify_line("--- a/x"), DiffLineKind::Meta);
        assert_eq!(classify_line("+++ b/x"), DiffLineKind::Meta);
        assert_eq!(
            classify_line("\\ No newline at end of file"),
            DiffLineKind::Meta
        );
        assert_eq!(classify_line("new file mode 100644"), DiffLineKind::Meta);
    }

    #[test]
    fn diff_body_lines_are_classified() {
        assert_eq!(classify_line("@@ -1,2 +1,3 @@"), DiffLineKind::Hunk);
        assert_eq!(classify_line("+第二行"), DiffLineKind::Added);
        assert_eq!(classify_line("-第一行"), DiffLineKind::Removed);
        assert_eq!(classify_line(" 上下文"), DiffLineKind::Context);
        assert_eq!(classify_line(""), DiffLineKind::Context);
    }

    #[test]
    fn sample_diff_is_classified_line_by_line() {
        let lines = split_lines(DIFF_SAMPLE);
        let kinds: Vec<DiffLineKind> = lines.iter().map(|line| classify_line(line)).collect();
        assert_eq!(kinds[0], DiffLineKind::Meta, "第一行是 diff --git");
        assert_eq!(*kinds.last().unwrap(), DiffLineKind::Added);
        assert!(kinds.contains(&DiffLineKind::Hunk));
        assert!(kinds.contains(&DiffLineKind::Removed));
        assert!(kinds.contains(&DiffLineKind::Context));
        // 三个 `+++`/`---` 头不能被算成新增/删除
        let meta_count = kinds
            .iter()
            .filter(|kind| **kind == DiffLineKind::Meta)
            .count();
        assert_eq!(
            meta_count, 5,
            "diff/index/---/+++/\\ No newline 共 5 行元信息"
        );
    }

    #[test]
    fn truncation_only_happens_past_the_limit() {
        let mut lines: Vec<String> = (0..6000).map(|i| format!("+{i}")).collect();
        assert!(truncate_lines(&mut lines, MAX_DIFF_LINES));
        assert_eq!(lines.len(), MAX_DIFF_LINES);

        let mut small: Vec<String> = vec!["+a".into(), "+b".into()];
        assert!(!truncate_lines(&mut small, MAX_DIFF_LINES));
        assert_eq!(small.len(), 2);
    }

    #[test]
    fn untracked_preview_marks_every_line_added() {
        let (lines, truncated) = untracked_lines("一\n二\n", MAX_PREVIEW_BYTES);
        assert!(!truncated);
        assert_eq!(lines, vec!["+一".to_string(), "+二".to_string()]);

        let long = "a".repeat(MAX_PREVIEW_BYTES + 10);
        let (lines, truncated) = untracked_lines(&long, MAX_PREVIEW_BYTES);
        assert!(truncated);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].len(), MAX_PREVIEW_BYTES + 1, "1 个 + 号 + 200KB");
    }

    #[test]
    fn untracked_preview_does_not_split_multibyte_chars() {
        // 每个汉字 3 字节，限制 4 字节时只能放下一个汉字
        let (lines, truncated) = untracked_lines("汉字", 4);
        assert!(truncated);
        assert_eq!(lines, vec!["+汉".to_string()]);
    }
}
