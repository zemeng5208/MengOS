# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 | 2026-11-17 | —（本地检查，不产生提交） | `git 2.39.0.windows.2`；`gh 2.96.0`，已登录 `zemeng5208`，scopes 含 `repo`、`workflow`；user.name / user.email 已配置；`D:\MengOS` 原为空目录 |
| T1 创建仓库和基础文件 | 已完成（用户已验收） | 2026-11-17 | c5a96f8（初始提交） | 仓库 `zemeng5208/MengOS`（public）已创建；8 个基础文件已提交并推送 |
| T2 第一条 CI 流水线 | 已完成（用户已验收并合并） | 2026-11-17 | dac62a4（squash 合并） | 29 个包全部 `OK`、缺失 0；PR #1 已 squash 合并进 main，分支 `task/T2-ci` 已删除 |
| T3 最小可启动 ISO | 进行中 | — | 待回填 | 分支 `task/T3-iso`；`iso/auto/{config,build,clean}` + `scripts/build-iso.sh` + `.github/workflows/build-iso.yml` 已写入，等待 GitHub Actions 构建（预计 20–40 分钟） |
| T4 中文、输入法、常用软件、zsh | 未开始 | — | — | 等 T3 验收通过 |
| T5 图形安装器 Calamares | 未开始 | — | — | |
| T6 外观 | 未开始 | — | — | |
| T7 Rust 工程骨架 | 未开始 | — | — | |
| T8 FeatherTerminal 第一版 | 未开始 | — | — | |
| T9 把 FeatherTerminal 装进 ISO | 未开始 | — | — | |

## 当前任务

T3：最小可启动 ISO —— 构建脚本与 live-build 配置已提交到分支 `task/T3-iso`，**正在等 GitHub Actions 的 ISO 构建结果**；构建完成后补 PR 与验收链接。

## 遇到的问题

- 初始化 `D:\MengOS` 时，运行环境首次报 `windows-acl workspace grant failed`（该目录缺少当前登录用户的“修改所有者”权限），已修复并复核生效；与项目本身无关。
- `gh run view --log` 读不到日志：gh 把日志缓存写到 `C:\Users\24035\AppData\Local\GitHub CLI\`（项目目录之外），被运行环境拒绝。改用 `gh run download` 下载 artifact 核对真实输出，不影响验收。
- `git push` / `git pull` 需要在不限制文件/进程的条件下执行：git 的凭据助手是 `!'...\gh.exe' auth git-credential`，要用 shell 启动，受限环境下 `sh.exe` 起不来（`couldn't create signal pipe, Win32 error 5`）。这是每次联网 git 操作的已知情况，不是仓库配置问题。
- 每次 push 会同时命中 `push` 和 `pull_request` 两个触发条件，`check-packages` 因此产生 2 次运行；ISO 构建（`build-iso`）只在 PR 与 main 上触发，避免额外构建。

## 下一步要执行的命令

用户回复“T3 通过”后（PR 编号以汇报中给出的为准）：

```powershell
Set-Location D:\MengOS
gh pr merge <PR编号> --squash --delete-branch
git switch main
git pull
```

然后开始 T4（中文、输入法、常用软件、zsh）：向 `iso/config/package-lists/mengos.list.chroot` 追加中文与终端相关包、修改 `iso/auto/config` 的 `--bootappend-live`、新增 `iso/config/hooks/normal/0100-mengos-chinese.hook.chroot` 与 `iso/config/includes.chroot/etc/skel/.zshrc`。
