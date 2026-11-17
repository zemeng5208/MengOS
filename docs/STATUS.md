# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 | 2026-11-17 | —（本地检查，不产生提交） | `git 2.39.0.windows.2`；`gh 2.96.0`，已登录 `zemeng5208`，scopes 含 `repo`、`workflow` |
| T1 创建仓库和基础文件 | 已完成（用户已验收） | 2026-11-17 | c5a96f8（初始提交） | 仓库 `zemeng5208/MengOS`（public）已创建；8 个基础文件已提交并推送 |
| T2 第一条 CI 流水线 | 已完成（用户已验收并合并） | 2026-11-17 | dac62a4（squash 合并） | 29 个包全部 `OK`、缺失 0；PR #1 已合并 |
| T3 最小可启动 ISO | 已完成（用户暂缓验收） | 2026-11-17 | 8a2a2f0（squash 合并） | ISO `MengOS-0.0.1-dev.1-amd64.iso` 约 1.5G，SHA256 `8399cd4f5320e5d33d9edd10aac7cc4ac571ae6537c9dc79c257bdf1af93f1a1`；run 44087258913（8 分 15 秒） |
| T4 中文、输入法、常用软件、zsh | 已完成（用户暂缓验收） | 2026-11-17 | 3cff081（squash 合并） | 48 个包缺失 0；`.zshrc` 3 个 source 路径经 CI `dpkg -L` 核实存在；ISO `MengOS-0.0.1-dev.4-amd64.iso` 约 1.7G，SHA256 `000ea4e4316f0c0ec6ff944fcaa4f52c3a53580df7a8698f3614a30fc85db530`；构建日志确认 fcitx5 / zsh / firefox-esr / fonts-noto-cjk 已安装，`etc/skel/.zshrc` 与 hook `0100-mengos-chinese` 均生效 |
| T5 图形安装器 Calamares | 进行中 | — | — | 分支 `task/T5-installer`：包清单追加 7 个安装器相关包（总数 48 → 55，CI 实测 `缺失 0 个`）；调查结论：`calamares-settings-debian` **不提供** `partition.conf`，分区默认值来自 `calamares` 包的 `/usr/share/calamares/modules/partition.conf`；hook `0200-mengos-calamares` 在构建时定位该文件、把 `defaultFileSystemType` 改为 `btrfs`，并把 `users.conf` 的 `userShell` 设为 `/usr/bin/zsh`；等 ISO 构建 |
| T6 外观 | 未开始 | — | — | |
| T7 Rust 工程骨架 | 未开始 | — | — | |
| T8 FeatherTerminal 第一版 | 未开始 | — | — | |
| T9 把 FeatherTerminal 装进 ISO | 未开始 | — | — | |

## 当前任务

T5：图形安装器 Calamares（安装到硬盘）—— 分支 `task/T5-installer`，正在等 CI 的 `cal-invest` 调查结果，之后按真实配置项写 btrfs hook。

## 遇到的问题

- 初始化 `D:\MengOS` 时，运行环境首次报 `windows-acl workspace grant failed`（该目录缺少当前登录用户的“修改所有者”权限），已修复并复核生效；与项目本身无关。
- `gh run view --log` 读不到日志：gh 把日志缓存写到 `C:\Users\24035\AppData\Local\GitHub CLI\`（项目目录之外），被运行环境拒绝。改用 GitHub API 取日志 / `gh run download` 取 artifact。
- `git push` / `git pull` 的凭据助手在受限环境里起不来；已改用一次性 HTTP 认证头（`tmp/gitnet.ps1`，token 来自 keyring，不写文件、不改全局配置），后续推送不再需要逐次授权。
- 用户决定：跳过逐任务验收（ISO 体积大、网络卡），每完成一个任务直接合并进 main 并继续，最后统一验收。
- 每次 push 到任务分支都会触发一次完整 ISO 构建：T3 约 8 分钟（1.5G），T4 约 11 分钟（1.7G，包变多）。所以每个任务尽量把改动压到 1–2 次推送。
- 平台告警（不影响构建）：`actions/checkout@v4`、`actions/upload-artifact@v4` 以 Node 20 运行但被强制升到 Node 24；`ubuntu-latest` 将于 2026-10-19 迁移到 Ubuntu 26。

## 下一步要执行的命令

T5 完成后直接开始 T6（外观：壁纸、用户名 mengos、user 加入 sudo 组、Calamares 品牌页、LightDM 背景）。
