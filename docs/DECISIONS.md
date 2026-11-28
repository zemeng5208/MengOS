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
| 2026-11-17 | Rust 工具链用 `rust-toolchain.toml` 锁 1.100.0，CI 容器 `rust:1.100.0-trixie` | 与 Debian trixie 的 glibc 对齐；本地无法编译，版本必须由仓库固定，避免 CI 漂移 |
| 2026-11-17 | `Cargo.lock` 提交进仓库（由 CI 生成后回传） | Windows 本地没有 Rust 工具链，lock 只能由 CI 产出；提交后构建可复现 |
| 2026-11-18 | FeatherTerminal 锁 `vte4 0.10.0` + `gtk4 0.11.5`，并给 vte4 打开 `gtk_v4_18` 特性 | 实测：gtk4 0.11.5 的 `auto/mod.rs` 把 `Accessible` 放在 `#[cfg(feature = "v4_10")]` 下，而 vte4 的 wrapper 无条件 `@implements gtk::Accessible`，不开特性就报 `E0425`；系统 GTK 4.18.6 满足 `v4_18` 的版本要求 |
| 2026-11-18 | 代码里用 `vte4::Format::Text`（不是 `TEXT`） | gtk-rs 0.11 起枚举变体改为 UpperCamelCase，CI 编译错误与编译器建议都证实这点 |
| 2026-11-18 | `spawn_async` 的 `envv` 显式传 `std::env::vars()` 拼出的环境 | vte4 0.10 中该参数类型是 `&[&str]`（不是 `Option`），传空切片可能给子进程一个空环境，导致 shell 没有 PATH/HOME |
| 2026-11-18 | `rust.yml` 增加「安装 GTK4/VTE4 开发库 + release 编译 + 上传二进制 artifact」 | 任务书 T8 要求；`cargo build` 需要 `libgtk-4-dev`/`libvte-2.91-gtk4-dev`/`pkg-config` |
| 2026-11-18 | 用 `cargo-deb`（`cargo install cargo-deb --locked`，CI 实测 3.8.0）打包，`maintainer = "zemeng5208"`（= 用户 GitHub 用户名）、`depends = "$auto, zsh"` | 任务书 T9 第 1 步；`$auto` 由 cargo-deb 用 ldd 推导库依赖，实测产出 `libc6/libglib2.0-0t64/libgtk-4-1/libvte-2.91-gtk4-0` 再并上 `zsh` |
| 2026-11-18 | deb 的 asset 源路径统一写 `target/release/feather-terminal` | cargo-deb 文档规定：即使是 workspace 也一律写 `target/release/` 前缀；实测 `cargo deb -p feather-terminal` 确实从该路径取到二进制 |
| 2026-11-18 | `build-iso.yml` 拆成 `build-deb`（rust 容器）与 `build-iso`（debian 容器，`needs: build-deb`），deb 通过 `download-artifact` 落到 `iso/config/packages.chroot/` | live-build 会自动安装该目录下所有 deb；两个 job 各用合适镜像，避免在 debian 容器里额外装 Rust 工具链 |
| 2026-11-18 | 桌面图标：hook `0300-mengos-terminal-icon` 用 `install -m 0755` 把 `.desktop` 放进 `/etc/skel/Desktop/`（若 `/home/user`、`/home/live` 已存在也各放一份） | 任务书 T9 第 4 步；实测 hook 运行时 `/home` 为空 → Live 用户在其后创建并继承 `/etc/skel`，Live 桌面会显示图标；可执行位用于尽量避免 Xfce 的「不受信任的启动器」提示 |
| 2026-11-18 | `.gitignore` 忽略 `/iso/config/packages.chroot/*.deb` | deb 只在 CI 里由 cargo-deb 生成，不提交进 Git |
| 2026-11-19 | 壁纸改为「按 `xrandr` 报告的输出名建键 + 补 `monitor0` + 兜底改所有已存在的键 + `xfdesktop --reload`」，并在 `/etc/skel` 放一份用户级 `xfce4-desktop.xml` | 用户实测 `dev.14` 显示的是 Debian 默认壁纸。构建时诊断证明 ISO 里壁纸文件、系统级/用户级 XML、`librsvg2-common 2.60.0`、`libpixbufloader_svg.so` 全都在，所以问题在运行时的键名：VMware 下输出名是 `Virtual-1`，系统级 XML 只写了 `monitor0`，而旧脚本只改「已存在」的键，首次登录时匹配为空 |
| 2026-11-19 | 壁纸自动启动项改为 `sh -c "sleep 4; …; sleep 3; …"`，登录后跑两遍 | 避开与 xfdesktop 启动、首次写配置的竞争 |
| 2026-11-19 | 终端右键菜单用 `gtk::PopoverMenu` + `GestureClick`（`PropagationPhase::Capture`），动作挂在窗口上（`win.copy`/`win.paste`/`win.select-all`/`win.reset`），弹出前用 `read_text_future()` 检查剪贴板再决定「粘贴」是否可点 | VTE4 不提供默认右键菜单，必须由应用自建；捕获阶段避免被 VTE 自己的点击处理吃掉 |
| 2026-11-19 | `vte4::Terminal::reset()` 必须传两个参数 `reset(true, true)` | docs.rs 上 vte4 0.10.0 的 `TerminalExt::reset` 签名是 `reset(clear_tabstops: bool, clear_history: bool)`，无参调用编译不过 |
| 2026-11-19 | Calamares 里删掉 `passwordRequirements.libpwquality`，并把 `minLength` 从 `-1` 改成 `1` | 用户实测密码过不了字典校验、无法建用户。Debian 的 `users.conf` 写的是 `libpwquality: [minlen=0, minclass=0]`，但 Calamares 3.3.14 `src/modules/users/CheckPWQuality.cpp` 里 `pwquality_check()` 仍会跑 cracklib 字典检查并要求评分 ≥ 40（`arbitrary_minimum_strength`），`Config::passwordStatus()` 在 `requireStrongPasswords` 为真时把失败判为 `Invalid` → 「下一步」永久不可用 |
| 2026-11-19 | 临时诊断 hook `0310-mengos-diag` 只用于这一轮核对 ISO 内容，验证完即删除（`b9f4c14`） | 要先确认 ISO 里到底有没有壁纸文件、xfconf、SVG 加载器，而不是靠猜 |
| 2026-11-21 | 密码策略保持「任意非空密码」（`minLength: 1`，无 `libpwquality` 字典/强度检查） | 派发方在第二批任务书第零部分明确决定：这是个人虚拟机系统，不要求密码强度；记录下来，后续任务不再改动。原因见上一轮：Calamares 3.3.14 的 cracklib 检查会把普通密码判为 Invalid |
| 2026-11-21 | FeatherTerminal 的 .deb 通过 GitHub 预发布 `dev-latest` 分发，文件名固定 `feather-terminal_latest_amd64.deb`，版本号 `0.0.1+<CI 运行号>` | 用户网络慢：只改 `rust/**` 的任务，用户下载约 200KB 的 deb 就能验收，不必每次下 1.8G 的 ISO；版本号带运行号便于确认装的是不是最新 |
| 2026-11-21 | CI 用 `Xvfb + openbox + imagemagick(import) + xdotool` 做 GUI 截图（`gui-smoke` job） | 第二批任务书第一部分第 2 条要求；截图只证明程序能启动、画面大致正确，**不代替用户验收** |
| 2026-11-21 | 安装器图标只改 `.desktop` 的 `Name`/`Name[zh_CN]`/`Comment`，图标沿用系统自带的 `/usr/share/pixmaps/install-debian.png` | T10 第 3 步明确「图标保持系统自带的安装图标即可，不要自己画」 |
| 2026-11-21 | `rust.yml` 的 `push` 触发由 `branches: ["**"]` 收窄为 `[main]` | 一次 PR 推送原本会同时触发 `push` 与 `pull_request` 两份运行（build-iso 自 T3 起就只限 main，没有这个问题；rust 有）。收窄后 PR 推送只跑一份，`pull_request` 仍然覆盖 PR 分支 |
| 2026-11-26 | 多标签用 `gtk::Notebook`，不引入 libadwaita | 任务书要求不更换技术栈（GTK4），`Notebook` 自带标签栏、关闭按钮和 `switch-page` 信号，够用；libadwaita 会带来新的依赖与主题差异 |
| 2026-11-26 | 快捷键用 `application.set_accels_for_action("win.*", …)` 绑到窗口动作上，不再用窗口上的 `EventControllerKey` | 实测在 Xvfb + openbox 下自建的窗口级 key controller 收不到按键（加了 `MENGOS_DEBUG_KEYS` 调试打印，`app.log` 里一条都没有）；accelerator 是 GTK 的标准路径，且能和右键菜单共用同一批 `win.*` 动作 |
| 2026-11-26 | 搜索用 VTE 原生接口（`Regex::for_search` + `search_set_regex` + `search_find_next/previous`），并把用户输入先经 `escape_regex` 转义 | 任务书要求「查找」；转义后按字面量匹配，避免用户输入 `.` `*` 时被当正则通配符。PCRE2 的 flags 由 vte4 以裸 u32 传入（`PCRE2_UTF|PCRE2_UCP|PCRE2_MULTILINE`），vte4 没有导出对应常量 |
| 2026-11-26 | 当前目录用 `/proc/<pid>/cwd` 的符号链接目标，1 秒定时器刷新（`glib::timeout_add_seconds_local`），并把它显示在标签标题和窗口标题上 | 这是内核给出的真实工作目录，一次 `readlink` 系统调用、不解析文本文件，开销可忽略；窗口标题带目录也让「目录跟踪是否生效」在 CI 截图里一眼可见 |
| 2026-11-26 | `State` 用 `thread_local!` 里的强引用保活，所有信号回调只持 `Weak<State>` | 直接原因是 `build_ui` 返回时 `Rc<State>` 被回收，导致定时器立刻 Break、所有动作处理器变成空操作（标签建不出来、目录不更新）。用 `thread_local` 而不是让回调持强引用，是为了不形成「State → 控件 → 回调 → State」的引用环 |
| 2026-11-26 | 标签标题 `Label` 必须同时设 `set_width_chars(12)` 与 `set_ellipsize(End)` | GTK4 里设了 `ellipsize` 的 Label 自然宽度按省略号算，只给 `max_width_chars` 会被压成「…」；`width_chars` 提供最小字符宽度 |
| 2026-11-26 | `spawn_async` 回调里的 PID 用 `pid.into_glib()` 转 i32 再拼 `/proc/<pid>/cwd` | vte4 0.10 的回调类型是 `Result<glib::Pid, glib::Error>`（见 docs.rs 的 `vte4/src/terminal.rs` 源码），`glib::Pid` 不是 i32，直接赋值报 `E0308` |
| 2026-11-26 | 侧栏用 `gtk::Paned`（水平），左侧终端、右侧侧栏；最小宽度靠 `sidebar.widget().set_size_request(260, -1)` + `set_shrink_end_child(false)` 实现 | 任务书要求默认 360、最小 260；`gtk::Paned` 不会把子控件压到它的最小宽度以下，比在 `notify::position` 里手工夹取简单可靠 |
| 2026-11-26 | 侧栏设置存 `~/.config/mengos/feather-terminal.toml`，用 `serde` + `toml 1.x`，字段加 `#[serde(default)]`，宽度夹取 `[260, 1200]`，坏文件退回默认值 + stderr 警告 | 任务书 T12 明确要求用 serde + toml、目录不存在要创建、文件损坏不能崩；`#[serde(default)]` 让旧版本文件少字段也能读 |
| 2026-11-26 | 侧栏 6 个页面拆成 `src/sidebar/{changes,network,files,assistant,profile,vault}.rs`，每个模块导出 `ID`/`TITLE`/`ICON` 与 `build()`/`set_current_dir()`；`sidebar/mod.rs` 用 `PageModule` 表拼装 | 任务书要求「每个页面一个模块，对外提供创建页面和目录变化两个函数，后续任务只改对应模块」，这样 T13–T16 互不干扰 |
| 2026-11-26 | 侧栏图标按钮用 `gtk::ToggleButton` + `set_group`（同一时间只有一个按下），图标经 CI 核实存在（Adwaita + elementary-xfce） | 任务书要求确认图标名在主题里存在；`set_group` 是 GTK 保证单选的标准做法，不用自己维护状态 |
| 2026-11-26 | 「点侧栏按钮后焦点回到终端」用 `GestureClick`（`PropagationPhase::Capture`）的 `released`，不用 `clicked` 信号 | GTK4 的 `GtkToggleButton` 只有 `toggled` 信号；点已经按下的按钮不会触发 `toggled`，但那时同样需要把焦点还给终端 |
| 2026-11-26 | 侧栏宽度只在「切换显示状态」和「关窗」时写盘 | 拖动分隔线时每像素写一次文件既浪费又可能卡顿；用户正常使用不会在拖动后立即断电 |
| 2026-11-26 | 文件树用 `gtk::ListView` + `gtk::TreeListModel` + `glib::BoxedAnyObject`（条目塞进 `gio::ListStore`），不用自定义 GObject 子类 | 任务书要求 `gtk::ListView` + `gtk::TreeListModel`；`BoxedAnyObject` 能直接把普通 Rust struct 放进 GListModel，省掉一个 `glib::wrapper!` 子类（少一层 unsafe 样板）。`TreeListModel` 的 `create_func` 只在展开时才调用，天然满足「懒加载」 |
| 2026-11-26 | 读目录 / 查 MIME / 读文件内容一律 `std::thread::spawn` + `async-channel::bounded(1)` + `glib::spawn_future_local` 回主线程 | 任务书要求「后台读取，不卡界面」。用 `std::thread` 而不是 `gio::spawn_blocking` 是为了不依赖 glib 的线程池行为，且 `async-channel` 的 `send_blocking` / `recv().await` 正好把「线程 → 主线程」这一步写成顺序代码，主线程不碰文件系统 |
| 2026-11-26 | 页面状态 `FilesState` 由 `PageView.on_dir_changed` 这个闭包**强引用**持有，控件自己的信号回调只持 `Weak<FilesState>` | 第一次实现时所有回调都持弱引用，`build()` 一返回 `Rc<FilesState>` 就被丢弃，文件树永远是空的（CI 截图空白、`app.log` 里一条读取记录都没有）。改成「有一个强引用保活 + 其余弱引用」既不会让状态被回收，也不会形成「状态 → 控件 → 回调 → 状态」的引用环 |
| 2026-11-26 | 文件操作菜单锚在预览区右上角的「···」按钮上（`gtk::PopoverMenu::from_model` + `insert_action_group("file", …)`），不按鼠标坐标手动摆放 | 右键时手动算坐标在不同缩放下容易跑偏；菜单由 `gio::Menu` 描述、动作走 `gio::SimpleAction`，和 T11 的 `win.*` 动作风格一致。右键（`GestureClick` 的 `BUTTON_SECONDARY`）只负责「选中该文件 + 弹出菜单」 |
| 2026-11-26 | 「用默认程序打开」统一走 `gio::AppInfo::launch_default_for_uri`，`.html`/`.htm` 只是把菜单文案换成「在默认浏览器中打开」 | 任务书要求「不能自己写浏览器，用系统默认程序打开」；`launch_default_for_uri` 就是 xdg-open 的 GIO 路径。文案区分是任务书对 `.html` 的额外要求，行为不变 |
| 2026-11-26 | 「在文件管理器中显示」先走 D-Bus `org.freedesktop.FileManager1.ShowItems`，失败再 `thunar <目录>` | 前者是桌面标准接口（Xfce 由 Thunar 实现，能高亮文件）；但服务不一定在（容器 / 精简系统），所以保留进程回退，保证按钮永远有反应 |
| 2026-11-26 | 图片预览用 `gdk::Texture::from_bytes` + `gtk::Picture`，超过 32MB 的图片不解码，只提示大小 | 解码巨图会吃掉内存并卡住主线程；32MB 足以覆盖常见照片/截图。任务书只要求「图片文件能显示缩略图」 |
| 2026-11-26 | 文件更改页的文件列表用 `gtk::ListBox`（每行 `gtk::ListBoxRow` + 标记 `Label` + 路径 `Label`），不用 `gtk::ListView` | 文件数量通常只有几十个，`ListBox` 不需要模型/工厂那套样板，选中（`row_selected`）也更直接；列表每 3 秒重建一次，重建时用 `restoring` 标志避免「恢复选中」触发一次多余的 diff 读取 |
| 2026-11-26 | 所有 git 命令都用 `std::process::Command` 在 `std::thread` 里跑，结果经 `async-channel` 回主线程；页面用「代号（generation）」丢弃过期结果 | 任务书要求后台执行、不准阻塞界面；3 秒一次的刷新可能比一次 `git status` 还快，用代号保证只有最新一次刷新的结果会写进界面 |
| 2026-11-26 | 自动刷新只重绘文件列表，不重绘已选文件的 diff（点文件或按「刷新」才重读 diff） | 否则每 3 秒把 diff 重画一遍，用户正在读的长 diff 会被反复拉回顶部；任务书只要求「刷新文件列表」 |
| 2026-11-26 | diff 行分类把 `+++ `、`--- `、`diff --git`、`index `、`new file mode`、`\ No newline` 等一律判为「元信息」，在 `+`/`-` 判断之前 | `+++ b/x` 以 `+` 开头、`--- a/x` 以 `-` 开头，如果先判增删会把 diff 头染成绿色/红色；单元测试专门断言样例里 5 行元信息 |
| 2026-11-26 | 未跟踪文件读文件开头（最多 200KB+4 字节）后全部按新增行显示 | 任务书要求「未跟踪文件显示新文件（未跟踪）并预览前 200KB，全部按新增行显示」；多读 4 字节用来判断是否被截断 |
| 2026-11-26 | `truncate_lines` 写成泛型 `Vec<T>`，diff 显示先摊平「段标题 + 每行」再统一截断 | 这样「只显示前 5000 行」是按最终显示行数算的，段标题也算一行；同时避免这个函数只在测试里被用到（clippy `-D warnings` 会报 dead_code） |
| 2026-11-26 | 页面「是否可见」用 `root.is_mapped()` 判断，不额外给 `PageModule` 加可见性回调 | `gtk::Stack` 会把非当前页的子控件 unmap，`is_mapped()` 就是现成的判据，T12 的 `PageModule` 接口不用改 |
| 2026-11-28 | 网络连接页的 `/proc/net` 解析全部自己写成纯函数，只借标准库 `std::net::Ipv6Addr` 做 IPv6 格式化，不引入 `netlink` 之类的 crate | 任务书原话要求「解析代码自己写成纯函数（不引入额外 crate）」；逻辑简单，样例数据放 `tests/fixtures/` 就能测 |
| 2026-11-28 | 进程树用「把所有 `/proc/<pid>/stat` 拼成一张表 → 纯函数解析 → BFS」的方式，而不是每个进程单独解析 | 同一个纯函数既能在 CI 里用样例测试，又能在真机上跑；`/proc/<pid>/stat` 的进程名可能带空格和括号，所以父进程号取「最后一个 `)` 之后的第二个字段」 |
| 2026-11-28 | 网络页的「当前标签 shell PID」通过新增的 `PageView.on_pid_changed` / `Sidebar::set_current_pid` 传入，每秒随目录一起刷新 | 网络页必须知道「哪个标签」才谈得上按标签过滤；复用已有的每秒定时器，不额外加定时器 |
| 2026-11-28 | UDP 的 `07` 状态码显示成 `UDP`，而不是 `CLOSE` | `/proc/net/udp` 没有 `LISTEN` 概念，`07` 表示未连接；直接显示 `CLOSE` 会让人误以为是 TCP 状态 |
| 2026-11-28 | 侧栏窄（约 350px）时，四列靠「进程 13 字符 + 地址 15 字符 + 状态不设宽度 + `caption` 小字号」挤下，地址用中间省略、状态用尾部省略 | 任务书要求每行四个字段都要有；`set_width_chars` 会同时定死自然宽度，给小了 PID 就被吃掉 |
