# MengOS

基于 Debian 13（trixie）的 Xfce 桌面操作系统，面向 VMware Workstation。

## 当前状态

第二批任务（T10–T16）已完成：FeatherTerminal 有多标签、搜索、当前目录跟踪，右侧侧栏 6 个页面里
**4 个有真实内容**（文件更改、网络连接、文件树、个人资料），另外 2 个（终端助手、密码库）
显示「尚未实现」，等后续批次做。

- 每次推送到 `main` 都会自动构建 ISO，产物在 GitHub Actions 的 Artifacts 里。
- 终端程序另外打了一个约 400KB 的 `.deb`，可以不下 ISO 就更新（见下）。

## 如何下载 ISO

1. 打开 Actions 页面：<https://github.com/zemeng5208/MengOS/actions/workflows/build-iso.yml>
2. 点最新一次 **成功**（绿色 ✓）的 `build-iso` 运行
3. 页面底部 **Artifacts** → 下载 `mengos-iso`
4. 解压得到 `MengOS-<版本>-amd64.iso`，用 VMware Workstation 新建虚拟机挂载启动

校验：在解压目录里跑 `sha256sum MengOS-*.iso`，和 `build-iso` 日志里 `SHA256` 那行对比。

## 如何用 `mengos-update-terminal` 更新终端

已经装好 MengOS 的机器上，不用重下 ISO：

```bash
mengos-update-terminal
```

它会从 `dev-latest` 预发布下载最新的 `feather-terminal` deb 并安装，最后打印安装后的版本号。
更新完请关闭所有 FeatherTerminal 窗口再重新打开。

想手动下载 deb：<https://github.com/zemeng5208/MengOS/releases/tag/dev-latest>

## FeatherTerminal 快捷键

| 快捷键 | 作用 |
|---|---|
| `Ctrl+Shift+T` | 新建标签页 |
| `Ctrl+Shift+W` | 关闭当前标签页 |
| `Ctrl+Shift+F` | 打开/关闭搜索栏 |
| `Ctrl+Shift+B` | 显示/隐藏右侧侧栏 |
| `Ctrl+Shift+C` | 复制选中的文字 |
| `Ctrl+Shift+V` | 粘贴 |
| `Ctrl+PageUp` | 切换到上一个标签页 |
| `Ctrl+PageDown` | 切换到下一个标签页 |

## 侧栏 6 个页面

| 页面 | 状态 | 数据来源 |
|---|---|---|
| 文件更改 | 已实现 | `git status` / `git diff` |
| 网络连接 | 已实现 | `/proc/<pid>/fd` + `/proc/net` |
| 文件树 | 已实现 | 文件系统（`read_dir`） |
| 终端助手 | 尚未实现 | — |
| 个人资料 | 已实现 | `/proc`、`/etc/passwd`、`/etc/group`、`~/.zsh_history` |
| 密码库 | 尚未实现 | — |

每个页面怎么用，见 [docs/USER_GUIDE_ZH.md](docs/USER_GUIDE_ZH.md)。

## 技术栈

| 项目 | 选择 |
|---|---|
| 基础系统 | Debian 13（trixie），amd64 |
| 桌面环境 | Xfce |
| 登录界面 | LightDM |
| 安装器 | Calamares |
| ISO 构建工具 | live-build |
| 自研程序 | Rust + GTK4 |
| 构建方式 | GitHub Actions（`debian:trixie` 容器） |

## 目录结构

| 路径 | 作用 |
|---|---|
| `iso/` | live-build 配置（`auto/`、`config/package-lists/`、`config/includes.chroot/`、`config/hooks/`） |
| `scripts/` | 构建与检查脚本 |
| `rust/` | Rust 工作区（`feather-core`、`feather-terminal`） |
| `docs/` | 任务书、进度、技术决策、用户手册 |
| `dist/` | ISO 等构建产物（不入 Git） |

## 文档

- 任务书：[docs/TASKBOOK.md](docs/TASKBOOK.md)
- 开发进度：[docs/STATUS.md](docs/STATUS.md)
- 技术决策：[docs/DECISIONS.md](docs/DECISIONS.md)
- 用户手册（中文）：[docs/USER_GUIDE_ZH.md](docs/USER_GUIDE_ZH.md)
