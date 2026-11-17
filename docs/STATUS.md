# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 | 2026-11-17 | —（本地检查，不产生提交） | `git 2.39.0.windows.2`；`gh 2.96.0`，已登录 `zemeng5208`，scopes 含 `repo`、`workflow`；user.name / user.email 已配置；`D:\MengOS` 原为空目录 |
| T1 创建仓库和基础文件 | 已完成（用户已验收） | 2026-11-17 | c5a96f8（初始提交） | 仓库 `zemeng5208/MengOS`（public）已创建；8 个基础文件已提交并推送 |
| T2 第一条 CI 流水线 | 等待验收 | 2026-11-17 | cab3c86 | CI 变绿（run 44084704016，`conclusion=success`）；artifact 显示 29 个包全部 `OK`、缺失 0 个；PR #1 等待验收 |
| T3 最小可启动 ISO | 未开始 | — | — | 等 T2 验收通过 |
| T4 中文、输入法、常用软件、zsh | 未开始 | — | — | |
| T5 图形安装器 Calamares | 未开始 | — | — | |
| T6 外观 | 未开始 | — | — | |
| T7 Rust 工程骨架 | 未开始 | — | — | |
| T8 FeatherTerminal 第一版 | 未开始 | — | — | |
| T9 把 FeatherTerminal 装进 ISO | 未开始 | — | — | |

## 当前任务

T2：第一条 CI 流水线 —— 分支 `task/T2-ci` 已推送，PR 已创建，**等待用户验收**。

- PR：https://github.com/zemeng5208/MengOS/pull/1
- CI 运行：https://github.com/zemeng5208/MengOS/actions/runs/44084704016 （`conclusion=success`，job `check` 29 秒）
- artifact `package-check` → `package-check.txt`：29 行 `OK 包名 版本`，最后一行 `总计 29 个包，缺失 0 个`

## 遇到的问题

- 初始化 `D:\MengOS` 时，运行环境首次报 `windows-acl workspace grant failed`（该目录缺少当前登录用户的“修改所有者”权限），已修复并复核生效；与项目本身无关。
- `gh run view --log` 读不到日志：gh 把日志缓存写到 `C:\Users\24035\AppData\Local\GitHub CLI\`（项目目录之外），被运行环境拒绝。改用 `gh run download` 下载 `package-check` artifact 核对真实输出，不影响验收。
- `git push` 需要在不限制文件/进程的条件下执行一次：git 的凭据助手是 `!'...\gh.exe' auth git-credential`，要用 shell 启动，受限环境下 `sh.exe` 起不来（`couldn't create signal pipe, Win32 error 5`）。这是每次推送的已知情况，不是仓库配置问题。

## 下一步要执行的命令

用户回复“T2 通过”后：

```powershell
Set-Location D:\MengOS
gh pr merge 1 --squash --delete-branch
git switch main
git pull
```

然后开始 T3（最小可启动 ISO）：创建 `iso/auto/config`、`iso/auto/build`、`iso/auto/clean`、`scripts/build-iso.sh`、`.github/workflows/build-iso.yml`。
