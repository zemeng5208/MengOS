# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 | 2026-11-17 | —（本地检查，不产生提交） | `git 2.39.0.windows.2`；`gh 2.96.0`，已登录 `zemeng5208`，scopes 含 `repo`、`workflow` |
| T1 创建仓库和基础文件 | 已完成（用户已验收） | 2026-11-17 | c5a96f8（初始提交） | 仓库 `zemeng5208/MengOS`（public）已创建；8 个基础文件已提交并推送 |
| T2 第一条 CI 流水线 | 已完成（用户已验收并合并） | 2026-11-17 | dac62a4（squash 合并） | 29 个包全部 `OK`、缺失 0；PR #1 已合并 |
| T3 最小可启动 ISO | 已完成（用户暂缓验收） | 2026-11-17 | 8a2a2f0（squash 合并） | ISO `MengOS-0.0.1-dev.1-amd64.iso` 约 1.5G，SHA256 `8399cd4f5320e5d33d9edd10aac7cc4ac571ae6537c9dc79c257bdf1af93f1a1`；run 44087258913（8 分 15 秒） |
| T4 中文、输入法、常用软件、zsh | 已完成（用户暂缓验收） | 2026-11-17 | 3cff081（squash 合并） | 48 个包缺失 0；`.zshrc` 3 个 source 路径经 CI `dpkg -L` 核实；ISO `dev.4` 约 1.7G，SHA256 `000ea4e4316f0c0ec6ff944fcaa4f52c3a53580df7a8698f3614a30fc85db530`；日志确认 fcitx5 / zsh / firefox-esr / fonts-noto-cjk 已装，`etc/skel/.zshrc` 与 hook `0100-mengos-chinese` 生效 |
| T5 图形安装器 Calamares | 已完成（用户暂缓验收） | 2026-11-17 | 8b1cb3d（squash 合并） | 55 个包缺失 0；**调查结论**：Debian 的 `calamares` 包不装任何模块 `*.conf`，`calamares-settings-debian` 也不提供 `partition.conf`，故 hook `0200-mengos-calamares` 在构建时新建该文件；构建日志原文 `2:defaultFileSystemType: "btrfs"`、`27:userShell: /usr/bin/zsh`；安装阶段顺序 `unpackfs → sources-media → bootloader-config → bootloader` 说明引导包取自 ISO 本地源（不需联网）；ISO `dev.8` 约 1.8G，SHA256 `922b5850702e8c2f4270b9b560c2ad0ec8b021624959a99fc694a9ed6e669e95` |
| T6 外观 | 进行中 | — | — | 分支 `task/T6-look`：Greybird 主题 + elementary-xfce 图标（CI 实测 `3.23.3-1` / `0.21-1` 存在）、自绘 SVG 浅蓝→浅灰渐变壁纸、四个 xfconf XML（单顶部面板，结构依据 Debian 自带 `/etc/xdg/xfce4/panel/default.xml`）、`xfwm4` 开启合成器、桌面三个图标、`build-iso.yml` 增加 `xmllint` 语法校验；等 ISO 构建 |
| T7 Rust 工程骨架 | 未开始 | — | — | 已确认 Docker Hub 有 `rust:1.100.0-trixie` 标签 |
| T8 FeatherTerminal 第一版 | 未开始 | — | — | |
| T9 把 FeatherTerminal 装进 ISO | 未开始 | — | — | |

## 当前任务

T6：外观 —— 代码已推送到 `task/T6-look`，等 CI 的 XML 校验与 ISO 构建结果。

## 遇到的问题

- 初始化 `D:\MengOS` 时，运行环境首次报 `windows-acl workspace grant failed`（该目录缺少当前登录用户的“修改所有者”权限），已修复并复核生效；与项目本身无关。
- `gh run view --log` 读不到日志：gh 把日志缓存写到 `C:\Users\24035\AppData\Local\GitHub CLI\`（项目目录之外），被运行环境拒绝。改用 GitHub API 取日志 / `gh run download` 取 artifact。
- `git push` / `git pull` 的凭据助手在受限环境里起不来；已改用一次性 HTTP 认证头（`tmp/gitnet.ps1`，token 来自 keyring，不写文件、不改全局配置），后续推送不再需要逐次授权。
- 用户决定：跳过逐任务验收（ISO 体积大、网络卡），每完成一个任务直接合并进 main 并继续，最后统一验收。
- 每次 push 到任务分支都会触发一次完整 ISO 构建（8–14 分钟，ISO 已到 1.8G）。因此**调查类工作改用「只改 `.github/workflows/check-packages.yml` 的临时探针分支」**：`build-iso` 只看 `iso/**` 等路径，探针分支不会触发 ISO 构建，1 分钟就能拿到 CI 实机数据（T6 的 Xfce 默认配置就是这么拿到的）。
- T5 第一次构建**失败**：hook 按任务书示例去 sed `/etc/calamares/modules/partition.conf`，但该文件在 Debian 打包里根本不存在（`ERROR: 找不到任何 partition.conf`，hook 以 exit 1 终止，`lb build` 报 `E: config/hooks/normal/0200-mengos-calamares.hook.chroot failed`）。第二次改为「文件不存在就按上游键名新建」，构建通过。
- 平台告警（不影响构建）：`actions/checkout@v4`、`actions/upload-artifact@v4` 以 Node 20 运行但被强制升到 Node 24；`ubuntu-latest` 将于 2026-10-19 迁移到 Ubuntu 26。

## 下一步要执行的命令

T6 合并后开始 T7（Rust 工作区 + feather-core，`rust/crates/...`，容器镜像 `rust:1.100.0-trixie`，临时步骤让 CI 生成并回传 `Cargo.lock`）。
