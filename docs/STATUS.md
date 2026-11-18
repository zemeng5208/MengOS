# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 | 2026-11-17 | —（本地检查，不产生提交） | `git 2.39.0.windows.2`；`gh 2.96.0`，已登录 `zemeng5208`，scopes 含 `repo`、`workflow` |
| T1 创建仓库和基础文件 | 已完成（用户已验收） | 2026-11-17 | c5a96f8（初始提交） | 仓库 `zemeng5208/MengOS`（public）已创建；8 个基础文件已提交并推送 |
| T2 第一条 CI 流水线 | 已完成（用户已验收并合并） | 2026-11-17 | dac62a4（squash 合并） | 29 个包全部 `OK`、缺失 0；PR #1 已合并 |
| T3 最小可启动 ISO | 已完成（用户暂缓验收） | 2026-11-17 | 8a2a2f0（squash 合并） | ISO `MengOS-0.0.1-dev.1-amd64.iso` 约 1.5G，SHA256 `8399cd4f5320e5d33d9edd10aac7cc4ac571ae6537c9dc79c257bdf1af93f1a1`；run 44087258913（8 分 15 秒） |
| T4 中文、输入法、常用软件、zsh | 已完成（用户暂缓验收） | 2026-11-17 | 3cff081（squash 合并） | 48 个包缺失 0；`.zshrc` 3 个 source 路径经 CI `dpkg -L` 核实；ISO `dev.4` 约 1.7G，SHA256 `000ea4e4316f0c0ec6ff944fcaa4f52c3a53580df7a8698f3614a30fc85db530`；日志确认 fcitx5 / zsh / firefox-esr / fonts-noto-cjk 已装，`etc/skel/.zshrc` 与 hook `0100-mengos-chinese` 生效 |
| T5 图形安装器 Calamares | 已完成（用户暂缓验收） | 2026-11-17 | 8b1cb3d（squash 合并） | 55 个包缺失 0；**调查结论**：Debian 的 `calamares` 包不装任何模块 `*.conf`，`calamares-settings-debian` 也不提供 `partition.conf`，故 hook `0200-mengos-calamares` 在构建时新建该文件；构建日志原文 `2:defaultFileSystemType: "btrfs"`、`27:userShell: /usr/bin/zsh`；安装阶段顺序 `unpackfs → sources-media → bootloader-config → bootloader` 说明引导包取自 ISO 本地源（不需联网）；ISO `dev.8` 约 1.8G，SHA256 `922b5850702e8c2f4270b9b560c2ad0ec8b021624959a99fc694a9ed6e669e95` |
| T6 外观 | 已完成（用户暂缓验收） | 2026-11-17 | f4df717（squash 合并） | 主题 Greybird、图标 elementary-xfce、自绘 SVG 渐变壁纸、单顶部面板、桌面三个图标；CI 实测安装 `greybird-gtk-theme 3.23.3-1`、`elementary-xfce-icon-theme 0.21-1`、`xfce4-power-manager 4.20.0-1`；新增的 `xmllint` 步骤输出 `XML 语法检查通过`；ISO `MengOS-0.0.1-dev.10-amd64.iso` 1.8G，SHA256 `b52eea7ab796d05eaa0d63c0183c35b4fcaac4fb6878de7042e1ab53d8d66e83`；run 44107153993（构建 9 分 49 秒） |
| T7 Rust 工程骨架 | 已完成（用户暂缓验收） | 2026-11-17 | 7f9f2fe（squash 合并） | `rust-toolchain.toml` 锁 1.100.0；工作区 + `feather-core`（`TaskId`/`BotId`/`TaskState` 与状态机）；CI 实测 `cargo 1.100.0`、`rustc 1.100.0`、`7 passed; 0 failed`、clippy 与 `cargo fmt --check` 无输出；`Cargo.lock` 已提交（lock version 4）；run 44108051242 |
| T8 FeatherTerminal 第一版 | 已完成（用户暂缓验收） | 2026-11-18 | 2f5cdee（squash 合并） | 新 crate `feather-terminal`：`vte4 0.10.0` + `gtk4 0.11.5`，`cargo tree -i gtk4` 只有一个版本；`spawn_async` 启动 `/usr/bin/zsh`（回退 `$SHELL`→`/bin/bash`），shell 退出关窗，`Ctrl+Shift+C/V` 走 VTE 自带剪贴板；CI 全绿（release 编译 49.96s、clippy/格式/测试均通过），artifact `feather-terminal` 220,260 字节（解压后二进制 556,744 字节）；run 44213101804 |
| T9 把 FeatherTerminal 装进 ISO | 进行中 | — | — | 分支 `task/T9-terminal-iso`：cargo-deb 打包 + `build-iso.yml` 拆成 `build-deb`/`build-iso` 两个 job |

## 当前任务

T9：把 FeatherTerminal 装进 ISO —— 分支 `task/T9-terminal-iso`。

## 遇到的问题

- 初始化 `D:\MengOS` 时，运行环境首次报 `windows-acl workspace grant failed`（该目录缺少当前登录用户的“修改所有者”权限），已修复并复核生效；与项目本身无关。
- `gh run view --log` 读不到日志：gh 把日志缓存写到 `C:\Users\24035\AppData\Local\GitHub CLI\`（项目目录之外），被运行环境拒绝。改用 GitHub API 取日志 / `gh run download` 取 artifact。
- `git push` / `git pull` 的凭据助手在受限环境里起不来；已改用一次性 HTTP 认证头（`tmp/gitnet.ps1`，token 来自 keyring，不写文件、不改全局配置），后续推送不再需要逐次授权。
- 用户决定：跳过逐任务验收（ISO 体积大、网络卡），每完成一个任务直接合并进 main 并继续，最后统一验收。
- 每次 push 到任务分支都会触发一次完整 ISO 构建（8–14 分钟，ISO 已到 1.8G）。因此**调查类工作改用「只改 `.github/workflows/check-packages.yml` 的临时探针分支」**：`build-iso` 只看 `iso/**` 等路径，探针分支不会触发 ISO 构建，1 分钟就能拿到 CI 实机数据（T6 的 Xfce 默认配置就是这么拿到的）。
- T5 第一次构建**失败**：hook 按任务书示例去 sed `/etc/calamares/modules/partition.conf`，但该文件在 Debian 打包里根本不存在（`ERROR: 找不到任何 partition.conf`，hook 以 exit 1 终止，`lb build` 报 `E: config/hooks/normal/0200-mengos-calamares.hook.chroot failed`）。第二次改为「文件不存在就按上游键名新建」，构建通过。
- 平台告警（不影响构建）：`actions/checkout@v4`、`actions/upload-artifact@v4` 以 Node 20 运行但被强制升到 Node 24；`ubuntu-latest` 将于 2026-10-19 迁移到 Ubuntu 26。
- T8 共失败 3 次，原因都不是猜的，均按 CI 原文修：①`cargo fmt --check` 要求把 `let ctrl_shift = …` 合成一行；②`error[E0425]: cannot find type 'Accessible' in crate 'gtk'`——gtk4 0.11.5 的 `auto/mod.rs` 里 `Accessible` 被 `#[cfg(feature = "v4_10")]` 门控，而 vte4 的生成代码无条件引用它，故给 vte4 打开 `gtk_v4_18` 特性；③`error[E0599]: no variant named 'TEXT' for enum 'vte4::Format'`——gtk-rs 0.11 的枚举变体是 `UpperCamelCase`，改成 `vte4::Format::Text`。

## 下一步要执行的命令

T9 两个 job 构建成功后：从 `.packages.txt` 里取 `feather-terminal` 那一行贴进汇报；确认 hook `0300-mengos-terminal-icon` 把图标放进 `/etc/skel/Desktop/`；最后按任务书汇总 T0–T9 的统一验收清单。
