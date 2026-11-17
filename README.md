# MengOS

基于 Debian 13（trixie）的 Xfce 桌面操作系统，面向 VMware Workstation。

## 当前状态

开发中，尚无可用 ISO。

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

## 如何获取 ISO

待 T3 完成后补充。

## 目录结构

| 路径 | 作用 |
|---|---|
| `iso/` | live-build 配置（`auto/`、`config/package-lists/`、`config/includes.chroot/`、`config/hooks/`） |
| `scripts/` | 构建与检查脚本 |
| `rust/` | Rust 工作区（T7 之后建立） |
| `docs/` | 任务书、进度、技术决策 |
| `dist/` | ISO 等构建产物（不入 Git） |

## 文档

- 任务书：[docs/TASKBOOK.md](docs/TASKBOOK.md)
- 开发进度：[docs/STATUS.md](docs/STATUS.md)
- 技术决策：[docs/DECISIONS.md](docs/DECISIONS.md)
