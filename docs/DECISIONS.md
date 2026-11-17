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
| 2026-11-17 | 用「新建 `/etc/calamares/modules/partition.conf`」的方式设置 `defaultFileSystemType: "btrfs"`（而不是任务书示例里的 sed 已有文件） | 构建实测：Debian 的 `calamares` 包不装任何模块 `*.conf`，`calamares-settings-debian` 也不提供 `partition.conf`，文件根本不存在，sed 无从下手；键名已用上游 v3.3.14 源码核实 |
| 2026-11-17 | 在 `partition.conf` 里同时写 `availableFileSystemTypes: ["btrfs", "ext4"]` | 上游注释说明：不写这一项就没有文件系统下拉框（只有固定默认值），而 T5 验收 A4 要求「看文件系统下拉框，默认是 btrfs」 |
| 2026-11-17 | 外观：GTK/窗口主题 `Greybird`、图标 `elementary-xfce`、自绘 SVG 渐变壁纸（浅蓝→浅灰）、单顶部面板 | T6 要求；禁止使用 Apple 图标/字体/壁纸，壁纸全部由自己写的 SVG 代码生成 |
| 2026-11-17 | 面板 XML 结构照抄 CI 里取到的 Debian 自带 `/etc/xdg/xfce4/panel/default.xml`（只保留一个顶部面板，插件改为 whiskermenu/tasklist/systray/pulseaudio/power-manager/clock） | 任务书明确要求「不要凭空编写」，面板插件名错一个就会弹「插件错误」 |
| 2026-11-17 | 额外安装 `xfce4-power-manager` | 面板要用 `power-manager-plugin`，而 `xfce4` 元包只把它列为 `Suggests`（音量的 `xfce4-pulseaudio-plugin` 则是 `Depends`，不用额外加） |
| 2026-11-17 | 额外增加 `/usr/local/bin/mengos-set-wallpaper` 与 `/etc/xdg/autostart/mengos-wallpaper.desktop` | xfdesktop 的壁纸按 XRandR 输出名存键（VMware 下可能是 `Virtual-1` 等），无法预知；登录后用 `xfconf-query` 按实际键名补设，保证验收 A1 能看到渐变壁纸 |
| 2026-11-17 | `build-iso.yml` 增加 `xmllint --noout` 校验 XML 的步骤 | 任务书 T6 第 4 步要求；XML 写错会让整个桌面配置静默失效 |
