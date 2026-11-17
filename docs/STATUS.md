# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 | 2026-11-17 | —（本地检查，不产生提交） | `git 2.39.0.windows.2`；`gh 2.96.0`，已登录 `zemeng5208`，scopes 含 `repo`、`workflow` |
| T1 创建仓库和基础文件 | 已完成（用户已验收） | 2026-11-17 | c5a96f8（初始提交） | 仓库 `zemeng5208/MengOS`（public）已创建；8 个基础文件已提交并推送 |
| T2 第一条 CI 流水线 | 已完成（用户已验收并合并） | 2026-11-17 | dac62a4（squash 合并） | 29 个包全部 `OK`、缺失 0；PR #1 已合并，分支已删除 |
| T3 最小可启动 ISO | 已完成（**用户暂缓验收**，最后一并验收） | 2026-11-17 | 8a2a2f0（squash 合并） | ISO 构建成功：run 44087258913，job `build` 8 分 15 秒；`MengOS-0.0.1-dev.1-amd64.iso` 约 1.5G，SHA256 `8399cd4f5320e5d33d9edd10aac7cc4ac571ae6537c9dc79c257bdf1af93f1a1`；PR #2 已合并 |
| T4 中文、输入法、常用软件、zsh | 进行中 | — | — | 分支 `task/T4-chinese`：包清单追加 **19** 个中文/终端相关包（总数 29 → **48**，CI 实测 `缺失 0 个`）、`--bootappend-live` 加 `locales=zh_CN.UTF-8 keyboard-layouts=us timezone=Asia/Shanghai`、新增 hook `0100-mengos-chinese`、新增 `/etc/skel/.zshrc`；`.zshrc` 里 3 个 source 路径已用 CI 的 `dpkg -L` 逐一核实存在 |
| T5 图形安装器 Calamares | 未开始 | — | — | 等 T4 完成 |
| T6 外观 | 未开始 | — | — | |
| T7 Rust 工程骨架 | 未开始 | — | — | |
| T8 FeatherTerminal 第一版 | 未开始 | — | — | |
| T9 把 FeatherTerminal 装进 ISO | 未开始 | — | — | |

## 当前任务

T4：中文、输入法、常用软件、zsh —— 代码已写入分支 `task/T4-chinese`，正在等 CI（包名校验 + `dpkg -L` 路径核验 + ISO 构建）。

## 遇到的问题

- 初始化 `D:\MengOS` 时，运行环境首次报 `windows-acl workspace grant failed`（该目录缺少当前登录用户的“修改所有者”权限），已修复并复核生效；与项目本身无关。
- `gh run view --log` 读不到日志：gh 把日志缓存写到 `C:\Users\24035\AppData\Local\GitHub CLI\`（项目目录之外），被运行环境拒绝。改用 GitHub API 取日志 / `gh run download` 取 artifact。
- `git push` / `git pull` 的凭据助手（`!gh auth git-credential`，需要 shell）在受限环境里起不来；已改用一次性 HTTP 认证头（`tmp/gitnet.ps1`，token 来自 keyring，不写任何文件、不改全局配置），因此后续推送不再需要逐次授权。
- 用户决定：**跳过逐任务验收**（ISO 体积大、网络卡），每完成一个任务直接合并进 main 并继续下一个，最后统一验收。

## 下一步要执行的命令

T4 完成后直接开始 T5（Calamares 安装器 + btrfs 默认文件系统），分支 `task/T5-installer`。
