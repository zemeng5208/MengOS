# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 | 2026-11-17 | —（本地检查，不产生提交） | `git 2.39.0.windows.2`；`gh 2.96.0`，已登录 `zemeng5208`，scopes 含 `repo`、`workflow`；user.name / user.email 已配置；`D:\MengOS` 原为空目录 |
| T1 创建仓库和基础文件 | 等待验收 | 2026-11-17 | 待回填 | 仓库 `zemeng5208/MengOS`（public）已创建；8 个基础文件已提交并推送 |
| T2 第一条 CI 流水线 | 未开始 | — | — | 等 T1 验收通过 |
| T3 最小可启动 ISO | 未开始 | — | — | 等 T2 验收通过 |
| T4 中文、输入法、常用软件、zsh | 未开始 | — | — | |
| T5 图形安装器 Calamares | 未开始 | — | — | |
| T6 外观 | 未开始 | — | — | |
| T7 Rust 工程骨架 | 未开始 | — | — | |
| T8 FeatherTerminal 第一版 | 未开始 | — | — | |
| T9 把 FeatherTerminal 装进 ISO | 未开始 | — | — | |

## 当前任务

T1：创建仓库和基础文件 —— 实施已完成，**等待用户验收**（验收清单见汇报第 6 节）。

## 遇到的问题

- 初始化 `D:\MengOS` 时，运行环境首次报 `windows-acl workspace grant failed`（该目录缺少当前登录用户的“修改所有者”权限），已修复并复核生效；与项目本身无关。

## 下一步要执行的命令

用户回复“T1 通过”后：

```powershell
Set-Location D:\MengOS
git switch main
git pull
git switch -c task/T2-ci
```

然后按任务书 T2 创建 `iso/config/package-lists/mengos.list.chroot`、`scripts/check-packages.sh` 与 `.github/workflows/check-packages.yml`。
