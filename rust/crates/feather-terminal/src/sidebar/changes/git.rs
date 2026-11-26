//! 真正执行 git 命令的地方。所有函数都在后台线程里调用，主线程不碰。

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::model::{self, ChangeEntry};

/// 调用 git 时可能出的错。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    /// 系统里没有 git
    NotInstalled,
    Other(String),
}

impl GitError {
    pub fn describe(&self) -> String {
        match self {
            GitError::NotInstalled => "未安装 git".to_string(),
            GitError::Other(error) => error.clone(),
        }
    }
}

/// 一次 git 调用的结果。
#[derive(Debug, Clone)]
pub struct GitOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// 执行 `git -C <dir> <args…>`。
pub fn run_git(dir: &Path, args: &[&str]) -> Result<GitOutput, GitError> {
    let output = Command::new("git").arg("-C").arg(dir).args(args).output();

    match output {
        Ok(output) => Ok(GitOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(GitError::NotInstalled),
        Err(error) => Err(GitError::Other(format!("执行 git 失败：{error}"))),
    }
}

/// 一个目录对应的仓库状态，页面据此显示不同内容。
#[derive(Debug, Clone)]
pub enum RepoState {
    NotInstalled,
    NotRepo,
    Repo {
        root: PathBuf,
        entries: Vec<ChangeEntry>,
    },
    Error(String),
}

/// 在后台算一次「这个目录在哪个仓库里、有哪些改动」。
pub fn load_repo_state(dir: &Path) -> RepoState {
    let toplevel = match run_git(dir, &["rev-parse", "--show-toplevel"]) {
        Ok(output) => output,
        Err(error) => {
            return match error {
                GitError::NotInstalled => RepoState::NotInstalled,
                other => RepoState::Error(other.describe()),
            };
        }
    };

    if toplevel.status != 0 {
        return RepoState::NotRepo;
    }

    let root = PathBuf::from(toplevel.stdout.trim());
    if root.as_os_str().is_empty() {
        return RepoState::NotRepo;
    }

    match run_git(&root, &["status", "--porcelain=v1", "-z"]) {
        Ok(output) if output.status == 0 => RepoState::Repo {
            root,
            entries: model::parse_status(&output.stdout),
        },
        Ok(output) => RepoState::Error(format!("git status 失败：{}", output.stderr.trim())),
        Err(error) => RepoState::Error(error.describe()),
    }
}

/// 选中一个文件后要显示的内容。
#[derive(Debug, Clone, Default)]
pub struct FileDiff {
    /// 已暂存的改动（`git diff --cached`）
    pub staged: String,
    /// 未暂存的改动（`git diff`）
    pub unstaged: String,
    /// 未跟踪文件的内容（最多读 `MAX_PREVIEW_BYTES + 4` 字节）
    pub untracked: Option<String>,
}

/// 取某个文件的 diff；未跟踪文件改为读它的内容。
pub fn load_file_diff(root: &Path, entry: &ChangeEntry) -> Result<FileDiff, String> {
    if entry.is_untracked() {
        let path = root.join(&entry.diff_path);
        return Ok(FileDiff {
            untracked: Some(read_head(&path, model::MAX_PREVIEW_BYTES)?),
            ..FileDiff::default()
        });
    }

    let unstaged = run_git(
        root,
        &[
            "diff",
            "--no-color",
            "--no-ext-diff",
            "--",
            &entry.diff_path,
        ],
    )
    .map_err(|error| error.describe())?;
    let staged = run_git(
        root,
        &[
            "diff",
            "--no-color",
            "--no-ext-diff",
            "--cached",
            "--",
            &entry.diff_path,
        ],
    )
    .map_err(|error| error.describe())?;

    if unstaged.status != 0 {
        return Err(format!("git diff 失败：{}", unstaged.stderr.trim()));
    }
    if staged.status != 0 {
        return Err(format!("git diff --cached 失败：{}", staged.stderr.trim()));
    }

    Ok(FileDiff {
        staged: staged.stdout,
        unstaged: unstaged.stdout,
        untracked: None,
    })
}

/// 读文件开头最多 `max + 4` 字节（多读几个字节用来判断有没有截断）。
fn read_head(path: &Path, max: usize) -> Result<String, String> {
    let file =
        std::fs::File::open(path).map_err(|error| format!("读不了 {}：{error}", path.display()))?;
    let mut buffer = Vec::new();
    file.take(max as u64 + 4)
        .read_to_end(&mut buffer)
        .map_err(|error| format!("读不了 {}：{error}", path.display()))?;
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}
