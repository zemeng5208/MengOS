# 技术决策记录

| 日期 | 决策 | 原因 |
|---|---|---|
| 2026-11-17 | 基础系统 Debian 13 trixie amd64 | 任务书规定 |
| 2026-11-17 | 桌面 Xfce + LightDM，安装器 Calamares，构建工具 live-build | 任务书规定 |
| 2026-11-17 | 所有构建在 GitHub Actions 的 debian:trixie 容器中进行 | 用户电脑为 Windows，无法本地构建 |
| 2026-11-17 | 系统名定为 **MengOS**，仓库名 `zemeng5208/MengOS` | 用户要求：OS 名字就是项目文件夹的名字（`D:\MengOS`） |
| 2026-11-17 | 任务书中 `Feather`/`feather` 前缀的组件名保持不变（FeatherTerminal、feather-core、feather-terminal） | 用户选择：只改系统名，组件命名沿用任务书 |
| 2026-11-17 | 系统级标识使用小写 `mengos`（hostname、`mengos.list.chroot`、`MENGOS_VERSION`、artifact 名、ISO 文件名、壁纸目录、hook 前缀） | 由“系统名 = MengOS”推导，保证同类标识一致 |
| 2026-11-17 | 所有产物与中间产物限定在项目目录 `D:\MengOS` 内（构建输出 `dist/`，临时文件 `tmp/`） | 用户明确要求 |
| 2026-11-17 | GitHub 仓库设为 public | 公开仓库 Actions 存储不限额，ISO 产物才能长期保留为 artifact（任务书要求） |
| 2026-11-17 | 输入法选 `fcitx5` + `fcitx5-chinese-addons`（自带简体拼音，开箱即用） | 任务书 T4 指定；避免额外下载词库 |
| 2026-11-17 | Live 启动参数增加 `locales=zh_CN.UTF-8 keyboard-layouts=us timezone=Asia/Shanghai` | 任务书 T4 规定，让 live 环境默认中文 |
| 2026-11-17 | 推送改用一次性 HTTP 认证头（token 来自 `gh auth token`），不改全局 git 配置 | 受限环境里 git 的 shell 凭据助手无法启动，同时避免每次推送都打断用户授权 |
| 2026-11-17 | 跳过逐任务人工验收，每任务合并后继续下一个，最后统一验收 | 用户决定：ISO 体积大、网络卡，逐个验收不现实 |
| 2026-11-17 | Calamares 默认文件系统改 btrfs：修改真实存在的 `partition.conf` 里的 `defaultFileSystemType`（CI 实测该文件在 `calamares` 包的 `/usr/share/calamares/modules/` 下，`calamares-settings-debian` 只提供 bootloader/fstab/mount/unpackfs 等覆盖项） | 任务书 T5 要求「先调查，再修改」，避免按记忆猜配置项名称与路径 |
| 2026-11-17 | 在 T5 的 hook 里把 Calamares `users.conf` 的 `userShell` 设为 `/usr/bin/zsh` | T4 第 5 步把「安装后新建用户默认 zsh」留给 T5 验收（A10），而安装器建用户只读 `users.conf`，不读 `/etc/adduser.conf` |
