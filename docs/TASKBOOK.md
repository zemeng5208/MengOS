> **本文件说明**：本文是任务书 v3 的全文转存。系统名已按“OS 名字就是项目文件夹名字”的约定改为 **MengOS**（`FeatherOS` → `MengOS`、`featheros` → `mengos`、`FEATHEROS` → `MENGOS`），`Feather` / `feather` 前缀的组件名保持不变（FeatherTerminal、feather-core、feather-terminal 等）；除此之外规则内容未作改动。

# MengOS 开发任务书 v3（给 OpenCode 代理，从零开始）

> **用户填写（发送前改掉尖括号里的内容）**
> - GitHub 用户名：`zemeng5208`
> - 新仓库名：`MengOS`
> - 仓库可见性：`public`（公开仓库的 Actions 存储不限额，ISO 才能上传；改成 private 很可能放不下 ISO）
> - 本地工作目录：`D:\MengOS`
> - 当前要执行的任务：见 `docs/STATUS.md` 的“当前任务”一节

---

## 第一部分：你必须先读懂的规则（每次都要遵守）

### 1.1 你是谁、在做什么

你是一名 Linux 发行版开发助手。你要帮用户（一名学生）做一个叫 **MengOS** 的操作系统：
- 基于 **Debian 13（代号 trixie）**，64 位（amd64）。
- 桌面是 **Xfce**，登录界面是 **LightDM**，安装器是 **Calamares**。
- 用 **live-build** 生成可启动的 ISO 文件，在 **VMware Workstation** 中安装使用。
- MengOS 自己写的程序用 **Rust + GTK4** 编写。

**你不是从零写操作系统**。绝大部分功能直接安装 Debian 现成的软件包。你的工作是：写配置文件、写构建脚本、写少量 Rust 程序、让 GitHub 自动构建 ISO。

### 1.2 工作环境（非常重要）

- 用户的电脑是 **Windows**，终端是 **PowerShell**。你在本地执行的命令必须是 PowerShell 命令，不能用 bash 语法（不能用 `&&` 串联以外的 bash 特性、不能用 `export`、不能用 `sudo`）。
- **Windows 上无法构建 ISO，也无法编译 GTK4 程序**。所有构建都在 **GitHub Actions** 上进行（在 `debian:trixie` 容器里跑 bash）。
- 所以你的工作循环是：
  1. 在本地写文件；
  2. `git add` → `git commit` → `git push`；
  3. 用 `gh run watch` 等待 GitHub Actions 结果；
  4. 失败就用 `gh run view <运行ID> --log-failed` 看错误，修复后重新推送；
  5. 成功后把结果汇报给用户。

### 1.3 一次只做一个任务

- 本文件第三部分有任务卡 T0、T1、T2……，**每次只做用户指定的那一个任务**。
- 做完一个任务后：**停下来**，按第 1.6 节的格式汇报，然后**等待用户验收**。
- 用户回复“Tx 通过”之后，才可以做下一个任务。用户没说通过，不准开始下一个任务。
- **不准提前做后面任务的内容**，即使你觉得顺手。

### 1.4 绝对禁止（违反任何一条，任务视为失败）

1. **禁止编造结果**。没运行过的命令不准说“已通过”；没看到的输出不准写进汇报；CI 没变绿不准说“构建成功”。
2. **禁止假数据**。程序里不准写死假的 CPU 数字、假的网络连接、假的 AI 回复来“演示”。做不到就显示“尚未实现”。
3. **禁止声称测过 VMware**。你没有 VMware，只有用户能在 VMware 里测试。你只能写“等待用户验收”。
4. **禁止 `git push --force`**，禁止删除用户的文件，禁止 `git reset --hard`。
5. **禁止把密码、Token、API Key 写进任何文件或汇报**。不准让用户把 Token 粘贴到聊天里。
6. **禁止把 ISO 文件提交进 Git**。ISO 只通过 GitHub Actions 的 artifact 下载。
7. **禁止更换技术方案**。不准把 Xfce 换成别的桌面，不准把 live-build 换成别的工具，不准把 Rust 换成别的语言。遇到困难就停下来汇报。
8. **禁止 `curl ... | sh` 这种直接执行网络脚本的写法**。
9. **禁止修改本任务书没让你修改的文件**。

### 1.5 遇到问题怎么办

- 一个错误，你可以尝试修复。**同一个错误修了 3 次还是失败，就停下来汇报**，写清楚：错误原文、你试过的 3 种方法、你认为的原因。
- 不知道某个 Debian 软件包名对不对时，在 CI 里用 `apt-cache policy <包名>` 查证，不要猜。
- 不知道某个 Rust 库的函数怎么用时，去 docs.rs 查**你实际使用的那个版本**的文档，不要凭记忆写。
- 需要用户做的事（例如登录 GitHub、安装软件），写清楚完整的命令让用户复制，然后停下来等。

### 1.6 汇报格式（每个任务结束时必须严格使用）

```markdown
# Tx 汇报：<任务名>

## 1. 结论（一句话）
例：T3 已完成，ISO 已由 GitHub Actions 构建成功，等待你在 VMware 中验收。
或：T3 未完成，卡在 xxx，需要你 xxx。

## 2. 我做了什么
| 文件路径 | 新建/修改 | 作用（一句话） |
|---|---|---|
| iso/auto/config | 新建 | live-build 的配置脚本 |

## 3. 我运行了哪些命令，真实结果是什么
（每条命令附上关键输出的原文摘录，不要改写）
- `git push` → 输出：……
- GitHub Actions 运行链接：https://github.com/zemeng5208/MengOS/actions/runs/<ID> → 结果：成功/失败

## 4. 产物
- 提交（commit）：<完整 SHA>
- 分支 / PR 链接：……
- ISO 下载位置（如有）：Actions 运行页面 → Artifacts → <名字>
- ISO 大小与 SHA256（如有）：……

## 5. 没做完的、我不确定的
- 逐条写。没有就写“无”。

## 6. 需要你验收的清单
| 编号 | 你要做的操作（一步一步） | 应该看到的结果 | 如果不对，请发给我什么 |
|---|---|---|---|
| A1 | …… | …… | 截图 / 某条命令的输出 |

## 7. 下一步
- 如果验收通过：请回复“Tx 通过”，我将开始 T(x+1)：<任务名>。
- 如果不通过：请把第 6 节中不对的编号和截图发给我。
```

### 1.7 进度文件

仓库里有一个 `docs/STATUS.md`，**每个任务结束时你都必须更新它**，格式：

```markdown
# MengOS 开发进度

| 任务 | 状态 | 完成日期 | 提交 SHA | 说明 |
|---|---|---|---|---|
| T0 环境检查 | 已完成 / 等待验收 / 进行中 / 未开始 / 受阻 | 2026-11-17 | abc123… | …… |

## 当前任务
Tx：……

## 遇到的问题
- ……

## 下一步要执行的命令
- ……
```

如果用户开启新会话让你“继续”，你**先读 `AGENTS.md` 和 `docs/STATUS.md`**，从“当前任务”接着做。

---

## 第二部分：项目最终目标（了解即可，现在不要做）

最终 MengOS 会有这些东西。**它们属于很后面的阶段，本任务书的 T0–T9 都不涉及，不要提前做**：

- FeatherTerminal：Rust + GTK4 + VTE 写的终端，左边是真实 zsh，右边是辅助侧栏（文件改动、网络连接、文件树、AI 助手等）。
- FeatherBot / FeatherModels：多个 AI 助手，私聊和群聊，接入模型网关。
- FeatherNet：网络与代理管理。
- FeatherPulse：任务管理器。
- FeatherTrace / FeatherApps：操作记录、软件安装记录、快照回滚。
- FeatherVault：基于 KeePassXC 的密码库。

**原则**：能用现成软件就用现成软件；只有 MengOS 独有的功能才自己写，自己写的部分用 Rust。

---

## 第三部分：任务卡

### 任务总览

| 任务 | 名称 | 产出 | 用户需要做什么 |
|---|---|---|---|
| T0 | 环境检查 | 一份环境报告 | 可能需要安装 git / gh 并登录 |
| T1 | 创建仓库和基础文件 | GitHub 上的新仓库 | 打开网页确认 |
| T2 | 第一条 CI 流水线 | GitHub Actions 变绿，软件包名全部验证 | 打开网页确认 |
| T3 | 最小可启动 ISO | 能启动到 Xfce 桌面的 Live ISO | 在 VMware 里启动 |
| T4 | 中文、输入法、常用软件 | 中文界面、拼音输入、浏览器等 | 在 VMware 里逐项检查 |
| T5 | 图形安装器 | 能安装到硬盘的 ISO | 在 VMware 里完整安装一次 |
| T6 | 外观 | 壁纸、主题、顶部面板、桌面图标 | 看效果 |
| T7 | Rust 工程骨架 | Rust 代码在 CI 中编译、测试通过 | 打开网页确认 |
| T8 | FeatherTerminal 第一版 | 能运行真实 zsh 的 GTK4 窗口 | 暂不需要 |
| T9 | 把 FeatherTerminal 装进 ISO | 桌面上双击打开终端 | 在 VMware 里测试 |

---

### T0：环境检查

**目标**：确认用户电脑上有 git 和 GitHub CLI（gh），并且 gh 已经登录。

**步骤**：

1. 在 PowerShell 中逐条运行，记录输出：
   ```powershell
   git --version
   gh --version
   gh auth status
   git config --global user.name
   git config --global user.email
   ```
2. 判断：
   - 如果 `git` 找不到：**停下**，请用户运行 `winget install --id Git.Git -e`，然后**关闭并重新打开终端**。
   - 如果 `gh` 找不到：**停下**，请用户运行 `winget install --id GitHub.cli -e`，然后重新打开终端。
   - 如果 `gh auth status` 显示未登录：**停下**，请用户自己运行 `gh auth login`（选 GitHub.com → HTTPS → 用浏览器登录）。**不要让用户把 Token 发给你。**
   - 如果 `user.name` / `user.email` 为空：请用户自己运行：
     ```powershell
     git config --global user.name "你的名字"
     git config --global user.email "你的GitHub邮箱"
     ```
3. 检查 `gh auth status` 的输出里，权限（scopes）是否包含 `repo` 和 `workflow`。如果没有 `workflow`，请用户运行：
   ```powershell
   gh auth refresh -s workflow
   ```
4. 检查本地目录 `D:\MengOS` 是否已存在。如果存在且不为空，**停下**，问用户怎么处理，不准删除。

**完成标准**：以上命令都有正常输出；gh 已登录，且有 `repo` 和 `workflow` 权限。

**本任务的验收清单**（写进汇报第 6 节）：
| 编号 | 操作 | 应该看到 |
|---|---|---|
| A1 | 看汇报第 3 节中 `gh auth status` 的输出 | 显示 `Logged in to github.com account <你的用户名>` |

---

### T1：创建仓库和基础文件

**目标**：在 GitHub 上创建新仓库，提交基础文件。

**步骤**：

1. 先检查仓库是否已存在：
   ```powershell
   gh repo view zemeng5208/MengOS
   ```
   如果已存在，**停下**问用户，不准覆盖。

2. 创建本地目录并初始化：
   ```powershell
   New-Item -ItemType Directory -Path D:\MengOS
   Set-Location D:\MengOS
   git init -b main
   ```

3. 创建以下文件（内容见下方）：
   - `README.md`
   - `LICENSE`（MIT 许可证，版权人写用户的 GitHub 用户名，年份 2026）
   - `.gitignore`
   - `.gitattributes`
   - `AGENTS.md`
   - `docs/STATUS.md`
   - `docs/DECISIONS.md`
   - `docs/TASKBOOK.md`（把**本任务书的全文**原样保存进去，以后会话可以重新读取）

4. **`.gitattributes` 内容**（非常重要：Windows 会把换行改成 CRLF，Linux 脚本就会运行失败）：
   ```gitattributes
   * text=auto eol=lf
   *.png binary
   *.jpg binary
   *.svg text eol=lf
   ```

5. **`.gitignore` 内容**：
   ```gitignore
   # 构建产物
   /dist/
   *.iso
   /iso/.build/
   /iso/chroot/
   /iso/binary/
   /iso/cache/
   /iso/local/
   /iso/*.log
   /iso/live-image-*
   /iso/config/binary
   /iso/config/bootstrap
   /iso/config/chroot
   /iso/config/common
   /iso/config/source
   # Rust
   /rust/target/
   # 编辑器
   .vscode/
   .idea/
   *.swp
   ```
   注意：`iso/config/` 下面的 `package-lists/`、`includes.chroot/`、`hooks/` 是我们自己写的，**必须提交**；只忽略 `lb config` 自动生成的那几个文件。

6. **`README.md` 内容**（中文）：
   - 第一行：`# MengOS`
   - 一句话介绍：基于 Debian 13 的 Xfce 桌面操作系统，面向 VMware Workstation。
   - “当前状态”一节：写“开发中，尚无可用 ISO”。
   - “如何获取 ISO”一节：写“待 T3 完成后补充”。

7. **`AGENTS.md` 内容**：把本任务书**第一部分（规则）全文**复制进去。OpenCode 每次启动都会自动读取这个文件。

8. **`docs/DECISIONS.md` 内容**：
   ```markdown
   # 技术决策记录

   | 日期 | 决策 | 原因 |
   |---|---|---|
   | 2026-11-17 | 基础系统 Debian 13 trixie amd64 | 任务书规定 |
   | 2026-11-17 | 桌面 Xfce + LightDM，安装器 Calamares，构建工具 live-build | 任务书规定 |
   | 2026-11-17 | 所有构建在 GitHub Actions 的 debian:trixie 容器中进行 | 用户电脑为 Windows，无法本地构建 |
   ```

9. 提交并创建远程仓库：
   ```powershell
   git add -A
   git commit -m "chore: initialize MengOS repository"
   gh repo create zemeng5208/MengOS --public --source . --remote origin --push --description "MengOS: Debian 13 based Xfce desktop OS for VMware"
   ```

10. 确认：
    ```powershell
    git log --oneline -1
    git remote -v
    gh repo view --web
    ```

**完成标准**：GitHub 上能看到仓库，包含上述 8 个文件。

**本任务的验收清单**：
| 编号 | 操作 | 应该看到 |
|---|---|---|
| A1 | 打开 `https://github.com/zemeng5208/MengOS` | 能看到 README 显示中文介绍 |
| A2 | 在网页上点开 `AGENTS.md` | 内容是规则全文 |
| A3 | 点开 `docs/STATUS.md` | T0、T1 状态已更新 |

---

### T2：第一条 CI 流水线（验证软件包）

**目标**：建立 GitHub Actions，在 Debian 13 容器中验证我们要用的所有软件包都真实存在。

**工作分支**：从本任务开始，每个任务都新建分支，完成后开 PR，用户验收通过后再合并。
```powershell
Set-Location D:\MengOS
git switch main
git pull
git switch -c task/T2-ci
```

**步骤**：

1. 创建软件包清单 `iso/config/package-lists/mengos.list.chroot`，内容如下（每行一个包，`#` 开头是注释）：
   ```text
   # ===== 桌面 =====
   xfce4
   xfce4-goodies
   xfce4-whiskermenu-plugin
   lightdm
   lightdm-gtk-greeter
   # ===== 文件管理 =====
   thunar
   thunar-volman
   gvfs
   gvfs-backends
   file-roller
   # ===== 常用软件 =====
   firefox-esr
   mousepad
   ristretto
   # ===== 声音 =====
   pipewire
   pipewire-pulse
   wireplumber
   pavucontrol
   # ===== 网络 =====
   network-manager
   network-manager-gnome
   # ===== VMware =====
   open-vm-tools
   open-vm-tools-desktop
   # ===== 系统 =====
   sudo
   dbus-user-session
   polkitd
   pkexec
   xdg-user-dirs
   # ===== Live 系统 =====
   live-boot
   live-config
   live-config-systemd
   ```
   （T4、T5 会往里加包，现在只放这些。）

2. 创建检查脚本 `scripts/check-packages.sh`：
   ```bash
   #!/usr/bin/env bash
   # 作用：检查软件包清单里的每个包在 Debian trixie 中是否存在
   set -euo pipefail
   LIST_DIR="iso/config/package-lists"
   missing=0
   total=0
   for f in "$LIST_DIR"/*.list.chroot; do
     while IFS= read -r line; do
       pkg="$(echo "$line" | sed 's/#.*//' | xargs)"
       [ -z "$pkg" ] && continue
       total=$((total+1))
       cand="$(apt-cache policy "$pkg" | awk '/Candidate:/ {print $2}')"
       if [ -z "$cand" ] || [ "$cand" = "(none)" ]; then
         echo "MISSING  $pkg"
         missing=$((missing+1))
       else
         echo "OK       $pkg  $cand"
       fi
     done < "$f"
   done
   echo "总计 $total 个包，缺失 $missing 个"
   [ "$missing" -eq 0 ]
   ```
   然后在 PowerShell 里给它加可执行权限（Windows 必须这样做）：
   ```powershell
   git add scripts/check-packages.sh
   git update-index --chmod=+x scripts/check-packages.sh
   ```

3. 创建 `.github/workflows/check-packages.yml`：
   ```yaml
   name: check-packages
   on:
     push:
       branches: ["**"]
     pull_request:
     workflow_dispatch:
   jobs:
     check:
       runs-on: ubuntu-latest
       container:
         image: debian:trixie
       steps:
         - name: 安装 git
           run: apt-get update && apt-get install -y git ca-certificates
         - uses: actions/checkout@v4
         - name: 显示系统版本
           run: cat /etc/os-release
         - name: 启用 contrib/non-free 源并更新
           run: |
             sed -i 's/^Components: main$/Components: main contrib non-free non-free-firmware/' /etc/apt/sources.list.d/debian.sources
             apt-get update
         - name: 检查软件包
           run: bash scripts/check-packages.sh | tee package-check.txt
         - name: 上传检查结果
           if: always()
           uses: actions/upload-artifact@v4
           with:
             name: package-check
             path: package-check.txt
   ```

4. 提交、推送、等待结果：
   ```powershell
   git add -A
   git commit -m "ci: verify package list against Debian trixie"
   git push -u origin task/T2-ci
   gh run list --limit 3
   gh run watch
   ```
   如果失败：
   ```powershell
   gh run view --log-failed
   ```
   如果是某个包 `MISSING`：在 Debian 官网 https://packages.debian.org/trixie/ 搜索正确的名字，**只替换为同一个软件的正确包名**，并在 `docs/DECISIONS.md` 记录。

5. 成功后创建 PR：
   ```powershell
   gh pr create --base main --head task/T2-ci --title "T2: CI package verification" --body "见 docs/STATUS.md"
   ```

**完成标准**：Actions 显示绿色对勾；`package-check.txt` 最后一行是“缺失 0 个”。

**本任务的验收清单**：
| 编号 | 操作 | 应该看到 |
|---|---|---|
| A1 | 打开汇报中的 Actions 链接 | 绿色对勾 |
| A2 | 在该页面底部 Artifacts 下载 `package-check`，打开 txt | 每行是 `OK 包名 版本`，最后一行“缺失 0 个” |
| A3 | 打开 PR 链接 | PR 存在，检查全部通过 |

验收通过后，用户会回复“T2 通过”。此时你执行合并：
```powershell
gh pr merge --squash --delete-branch
git switch main
git pull
```

---

### T3：最小可启动 ISO

**目标**：用 live-build 构建一个能启动到 Xfce 桌面的 Live ISO（这一步**还不能安装到硬盘**，只是能启动）。

**分支**：`task/T3-iso`（从最新 main 创建，方法同 T2）。

**步骤**：

1. 创建 `iso/auto/config`（live-build 配置脚本）：
   ```sh
   #!/bin/sh
   set -e
   lb config noauto \
     --mode debian \
     --distribution trixie \
     --architectures amd64 \
     --archive-areas "main contrib non-free non-free-firmware" \
     --binary-images iso-hybrid \
     --debian-installer none \
     --iso-application "MengOS" \
     --iso-volume "MengOS" \
     --iso-publisher "MengOS Project" \
     --bootappend-live "boot=live components username=user hostname=mengos" \
     --memtest none \
     "${@}"
   ```

2. 创建 `iso/auto/build`：
   ```sh
   #!/bin/sh
   set -e
   lb build noauto "${@}" 2>&1 | tee build.log
   ```

3. 创建 `iso/auto/clean`：
   ```sh
   #!/bin/sh
   set -e
   lb clean noauto "${@}"
   rm -f config/binary config/bootstrap config/chroot config/common config/source
   rm -f build.log
   ```

4. 给三个文件加可执行权限：
   ```powershell
   git add iso/auto
   git update-index --chmod=+x iso/auto/config iso/auto/build iso/auto/clean
   ```

5. 创建构建脚本 `scripts/build-iso.sh`：
   ```bash
   #!/usr/bin/env bash
   # 作用：在 Debian trixie 中构建 MengOS ISO，输出到 dist/
   set -euo pipefail
   VERSION="${MENGOS_VERSION:-0.0.1-dev}"
   ROOT="$(cd "$(dirname "$0")/.." && pwd)"
   cd "$ROOT/iso"
   lb clean --purge || true
   lb config
   lb build
   mkdir -p "$ROOT/dist"
   ISO_SRC="$(ls -1 live-image-amd64*.iso | head -n1)"
   ISO_DST="$ROOT/dist/MengOS-${VERSION}-amd64.iso"
   mv "$ISO_SRC" "$ISO_DST"
   cp live-image-amd64.packages "$ROOT/dist/MengOS-${VERSION}-amd64.packages.txt" || true
   cd "$ROOT/dist"
   sha256sum "$(basename "$ISO_DST")" > "$(basename "$ISO_DST").sha256"
   ls -lh
   cat ./*.sha256
   ```
   并加可执行权限（`git update-index --chmod=+x scripts/build-iso.sh`）。

6. 创建 `.github/workflows/build-iso.yml`：
   ```yaml
   name: build-iso
   on:
     workflow_dispatch:
     pull_request:
       paths: ["iso/**", "scripts/build-iso.sh", ".github/workflows/build-iso.yml"]
     push:
       branches: [main]
       paths: ["iso/**", "scripts/build-iso.sh", ".github/workflows/build-iso.yml"]
   jobs:
     build:
       runs-on: ubuntu-latest
       timeout-minutes: 120
       container:
         image: debian:trixie
         options: --privileged
       steps:
         - name: 安装构建工具
           run: |
             apt-get update
             apt-get install -y git ca-certificates live-build debootstrap squashfs-tools xorriso \
               grub-pc-bin grub-efi-amd64-bin mtools dosfstools syslinux syslinux-common isolinux
         - uses: actions/checkout@v4
         - name: 构建 ISO
           env:
             MENGOS_VERSION: 0.0.1-dev.${{ github.run_number }}
           run: bash scripts/build-iso.sh
         - name: 上传构建日志
           if: always()
           uses: actions/upload-artifact@v4
           with:
             name: build-log
             path: iso/build.log
         - name: 上传 ISO
           uses: actions/upload-artifact@v4
           with:
             name: mengos-iso
             path: |
               dist/*.iso
               dist/*.sha256
               dist/*.packages.txt
             retention-days: 14
             compression-level: 0
   ```

7. 推送并等待（构建大约需要 20–40 分钟）：
   ```powershell
   git add -A
   git commit -m "feat(iso): minimal bootable live ISO with Xfce"
   git push -u origin task/T3-iso
   gh pr create --base main --head task/T3-iso --title "T3: minimal bootable ISO" --body "见 docs/STATUS.md"
   gh run watch
   ```

8. 构建失败时：
   - 运行 `gh run view --log-failed`，找到**第一条**真正的错误（通常是 `E:` 开头或 `error` 字样），不要只看最后一行。
   - 常见原因：脚本是 CRLF 换行（检查 `.gitattributes`）、脚本没有可执行权限、包名错误。

9. 成功后，在汇报中写出：ISO 文件名、大小、SHA256（从 Actions 日志的 `cat ./*.sha256` 一行复制）。

**完成标准**：Actions 绿色；artifact `mengos-iso` 中有 `.iso` 和 `.sha256` 文件。

**本任务的验收清单**（这一节请完整写进汇报，用户要照着做）：

| 编号 | 你要做的操作 | 应该看到的结果 | 如果不对，请发给我 |
|---|---|---|---|
| A1 | 打开汇报中的 Actions 链接，页面底部 Artifacts 里下载 `mengos-iso`，解压 | 得到一个 `.iso` 文件和一个 `.sha256` 文件 | 截图 |
| A2 | 在 PowerShell 中运行 `Get-FileHash .\MengOS-*.iso -Algorithm SHA256` | 输出的哈希值与 `.sha256` 文件内容一致（大小写不同没关系） | 两个值的截图 |
| A3 | 打开 VMware → 创建新的虚拟机 → 典型 → 安装程序光盘映像文件选这个 ISO → 客户机操作系统选“Linux / Debian 12.x 64 位” → 名称 MengOS-T3 → 磁盘 60GB → 自定义硬件：内存 6GB、处理器 4 核 → 完成 | 虚拟机创建成功 | 报错截图 |
| A4 | 关机状态下：虚拟机设置 → 选项 → 高级 → 固件类型选 **UEFI**，**不勾选**安全引导 | 设置保存成功 | — |
| A5 | 启动虚拟机，在启动菜单选第一项 | 1–3 分钟内进入 Xfce 桌面，**不需要输入密码** | 卡住画面的截图 |
| A6 | 在桌面上移动鼠标、打开左上角菜单 | 鼠标正常，菜单能打开 | 截图 |
| A7 | 拖动 VMware 窗口边缘改变大小 | 桌面分辨率跟着变化 | 不变的话说明一下 |
| A8 | 关机，把固件类型改成 **BIOS**，重新启动一次 | 同样能进入桌面 | 截图 |

---

### T4：中文、输入法、常用软件、zsh

**目标**：系统默认中文界面，能用拼音输入中文，有浏览器、zsh 终端。

**分支**：`task/T4-chinese`

**步骤**：

1. 在 `iso/config/package-lists/mengos.list.chroot` **末尾追加**：
   ```text
   # ===== 中文 =====
   locales
   fonts-noto-cjk
   fonts-noto-color-emoji
   fcitx5
   fcitx5-chinese-addons
   fcitx5-frontend-gtk3
   fcitx5-frontend-gtk4
   fcitx5-frontend-qt5
   fcitx5-config-qt
   im-config
   # ===== 终端与 Shell =====
   xfce4-terminal
   zsh
   zsh-autosuggestions
   zsh-syntax-highlighting
   fzf
   git
   python3
   btop
   # ===== 工具 =====
   xfce4-screenshooter
   ```
   说明：选择 `fcitx5-chinese-addons`（自带简体拼音，开箱即用）。在 `docs/DECISIONS.md` 记录这一条。

2. 修改 `iso/auto/config` 的 `--bootappend-live` 一行，改为：
   ```text
   --bootappend-live "boot=live components username=user hostname=mengos locales=zh_CN.UTF-8 keyboard-layouts=us timezone=Asia/Shanghai" \
   ```

3. 创建 hook 脚本 `iso/config/hooks/normal/0100-mengos-chinese.hook.chroot`（会在构建时于系统内部执行）：
   ```bash
   #!/bin/sh
   set -e
   # 生成中文和英文 locale
   sed -i 's/^# *zh_CN.UTF-8 UTF-8/zh_CN.UTF-8 UTF-8/' /etc/locale.gen
   sed -i 's/^# *en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen
   locale-gen
   update-locale LANG=zh_CN.UTF-8
   # 默认输入法框架设为 fcitx5
   im-config -n fcitx5 || true
   # 新用户默认 shell 为 zsh
   sed -i 's|^DSHELL=.*|DSHELL=/usr/bin/zsh|' /etc/adduser.conf || true
   sed -i 's|^SHELL=.*|SHELL=/usr/bin/zsh|' /etc/default/useradd || true
   ```
   加可执行权限。

4. 创建新用户的默认 zsh 配置 `iso/config/includes.chroot/etc/skel/.zshrc`：
   ```zsh
   # MengOS 默认 zsh 配置
   autoload -Uz compinit && compinit
   HISTFILE=~/.zsh_history
   HISTSIZE=10000
   SAVEHIST=10000
   setopt HIST_IGNORE_DUPS SHARE_HISTORY
   PROMPT='%F{cyan}%n@%m%f %F{yellow}%~%f %# '
   [ -f /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh ] && source /usr/share/zsh-autosuggestions/zsh-autosuggestions.zsh
   [ -f /usr/share/doc/fzf/examples/key-bindings.zsh ] && source /usr/share/doc/fzf/examples/key-bindings.zsh
   [ -f /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ] && source /usr/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
   ```
   注意：路径要在 CI 里用 `dpkg -L zsh-autosuggestions`、`dpkg -L fzf`、`dpkg -L zsh-syntax-highlighting` 核实。核实方法：在 `check-packages.yml` 中临时加一步安装这些包并打印 `dpkg -L` 结果，确认后删掉这一步。

5. 关于 Live 用户的 shell：Live 用户 `user` 是开机时由 live-config 创建的，**不一定**会使用上面设置的 zsh。本任务**不要**为此另写复杂脚本，只保证“安装后新建的用户”默认用 zsh（T5 验收）。在汇报第 5 节写明：“Live 用户的默认 shell 未确认，请看验收 A6 的结果。”

6. 推送、等待构建、开 PR（同 T3 第 7 步，提交信息 `feat(iso): Chinese locale, fcitx5 pinyin, zsh`）。

7. 构建成功后，从 artifact 的 `.packages.txt` 中确认 `fcitx5-chinese-addons`、`zsh`、`firefox-esr`、`fonts-noto-cjk` 确实在里面，把这几行原文贴进汇报。

**完成标准**：构建成功；`.packages.txt` 包含上述包。

**本任务的验收清单**：
| 编号 | 你要做的操作 | 应该看到的结果 | 如果不对，请发给我 |
|---|---|---|---|
| A1 | 下载新 ISO，核对 SHA256（同 T3 的 A2） | 一致 | — |
| A2 | 在 T3 建好的虚拟机设置里，把光驱换成新 ISO，启动 | 进入桌面 | 截图 |
| A3 | 看菜单和窗口文字 | 是中文 | 截图 |
| A4 | 打开 Mousepad（文本编辑器），按 `Ctrl+空格` 切换输入法，输入 `nihao` | 出现候选框，能选出“你好” | 截图 |
| A5 | 打开 Firefox，访问 https://www.debian.org | 页面正常显示，中文不乱码 | 截图 |
| A6 | 打开终端，输入 `echo $SHELL` 回车 | 记录输出（`/usr/bin/zsh` 或 `/bin/bash`） | 输出原文 |
| A7 | 在终端输入 `gi` 然后按 Tab | 能补全出 `git` 等命令 | 截图 |
| A8 | 在宿主机复制一段文字，到虚拟机的 Mousepad 粘贴 | 能粘贴 | 说明结果 |
| A9 | 打开 YouTube 或任意有声音的网页 / 系统音量调节 | 有声音，音量图标可调 | 说明结果 |

---

### T5：图形安装器 Calamares（安装到硬盘）

**目标**：Live 桌面上有“安装 MengOS”图标，能用 Calamares 安装到虚拟硬盘，默认使用 Btrfs 文件系统，重启后从硬盘进入系统，用密码登录。

**分支**：`task/T5-installer`

**步骤**：

1. 软件包清单末尾追加：
   ```text
   # ===== 安装器 =====
   calamares
   calamares-settings-debian
   btrfs-progs
   snapper
   grub-efi-amd64-bin
   grub-pc-bin
   efibootmgr
   ```

2. **先调查，再修改**。在 CI 中临时加一个步骤（可放在 `check-packages.yml`），安装 `calamares-settings-debian`，并打印以下内容，把结果贴进汇报：
   ```bash
   apt-get install -y calamares-settings-debian
   dpkg -L calamares-settings-debian
   cat /etc/calamares/settings.conf
   cat /etc/calamares/modules/partition.conf
   cat /usr/sbin/bootloader-config 2>/dev/null || cat /usr/lib/calamares/*bootloader* 2>/dev/null || true
   ```
   你要从输出中弄清楚：
   - `partition.conf` 中默认文件系统的配置项叫什么名字（通常是 `defaultFileSystemType`）；
   - 安装引导程序时需要哪些包，是从网络下载还是从 ISO 里取。
   调查完后删除这个临时步骤。

3. 根据调查结果，创建 hook `iso/config/hooks/normal/0200-mengos-calamares.hook.chroot`，把默认文件系统改成 btrfs。示例（**配置项名称以第 2 步实际看到的为准**）：
   ```bash
   #!/bin/sh
   set -e
   CONF=/etc/calamares/modules/partition.conf
   if [ -f "$CONF" ]; then
     sed -i 's/^defaultFileSystemType:.*/defaultFileSystemType: "btrfs"/' "$CONF"
     grep -n "defaultFileSystemType" "$CONF"
   else
     echo "找不到 $CONF" >&2
     exit 1
   fi
   ```
   加可执行权限。

4. 若第 2 步发现引导程序安装需要联网下载，在汇报“不确定”一节写明，并在验收清单中要求用户安装时保持 NAT 网络连接。**不要自己发明复杂的离线方案**，先让用户测一次再说。

5. 修改安装器显示的名称：如果 `settings.conf` 或 `branding` 目录中有 “Debian” 字样的产品名，**本任务不改**（T6 再处理），避免改坏。

6. 推送、构建、开 PR（提交信息 `feat(iso): Calamares installer with btrfs default`）。

**完成标准**：构建成功；构建日志中 hook 的 `grep` 输出显示 `defaultFileSystemType: "btrfs"`（把这行原文贴进汇报）。

**本任务的验收清单**（这是最重要的一次验收，请写得非常完整）：
| 编号 | 你要做的操作 | 应该看到的结果 | 如果不对，请发给我 |
|---|---|---|---|
| A1 | 新建一台虚拟机 MengOS-T5（配置同 T3 的 A3、A4，UEFI），网络用 NAT，用新 ISO 启动 | 进入 Live 桌面 | 截图 |
| A2 | 双击桌面上的安装图标（或菜单中“Install Debian / 安装系统”） | Calamares 安装器打开 | 截图 |
| A3 | 语言选“简体中文”，时区选上海，键盘默认 | 能进入下一步 | — |
| A4 | 分区页选“抹除磁盘”，看文件系统下拉框 | 默认是 **btrfs** | 截图 |
| A5 | 用户页：填写你的名字、用户名、**设置一个密码**，**不要**勾选自动登录 | 能进入下一步 | — |
| A6 | 确认并开始安装 | 进度条走完，提示安装完成，无红色报错 | 报错截图；并在终端运行 `cat ~/.cache/calamares/session.log` 或 `sudo cat /root/.cache/calamares/session.log` 拍照/复制 |
| A7 | 关机 → 虚拟机设置 → CD/DVD 取消勾选“启动时连接” → 开机 | 从硬盘启动，出现 LightDM 登录界面 | 截图 |
| A8 | 输入你设置的密码登录 | 进入中文 Xfce 桌面 | 截图 |
| A9 | 打开终端运行 `findmnt /` | `FSTYPE` 一列显示 `btrfs` | 输出原文 |
| A10 | 运行 `echo $SHELL` | 显示 `/usr/bin/zsh` | 输出原文 |
| A11 | 运行 `systemctl --failed` | 显示 `0 loaded units listed`（或列出失败的服务） | 输出原文 |
| A12 | 运行 `sudo apt update` | 能正常更新软件源 | 输出最后几行 |

---

### T6：外观

**目标**：简洁的浅色主题、MengOS 壁纸、顶部单面板、桌面图标。**不准使用 Apple 的图标、字体、壁纸或商标。**

**分支**：`task/T6-look`

**要求**：

1. 软件包清单追加：`greybird-gtk-theme`、`elementary-xfce-icon-theme`（先用 `check-packages.sh` 验证存在）。
2. 壁纸：用 SVG 自己画一张简单的渐变背景（浅蓝到浅灰），存为 `iso/config/includes.chroot/usr/share/backgrounds/mengos/default.svg`。**必须是你自己写的 SVG 代码，不准从网上下载图片。**
3. 默认桌面配置放在 `iso/config/includes.chroot/etc/xdg/xfce4/xfconf/xfce-perchannel-xml/` 下，至少包括：
   - `xsettings.xml`：GTK 主题 Greybird，图标主题 elementary-xfce；
   - `xfwm4.xml`：窗口主题 Greybird，启用合成器（`use_compositing` 为 true）；
   - `xfce4-desktop.xml`：壁纸路径指向上面的 SVG；桌面显示“主文件夹”“文件系统”“回收站”图标；
   - `xfce4-panel.xml`：**只保留一个顶部面板**，包含：Whisker 菜单、窗口按钮、通知区域、PulseAudio 音量插件、电源管理插件、时钟。**删除底部面板。**
4. 每个 XML 文件**先在 CI 中用 `xmllint --noout` 检查语法**（在 `build-iso.yml` 构建前加一步：`apt-get install -y libxml2-utils && find iso/config/includes.chroot -name '*.xml' -print0 | xargs -0 -r xmllint --noout`）。
5. 面板插件的 XML 写法：先在 CI 中安装 `xfce4-panel`，打印 `/etc/xdg/xfce4/panel/default.xml` 原文作参考，基于它修改，**不要凭空编写**。
6. 提交信息：`feat(look): MengOS theme, wallpaper and top panel`。

**本任务的验收清单**：
| 编号 | 你要做的操作 | 应该看到的结果 | 如果不对，请发给我 |
|---|---|---|---|
| A1 | 用新 ISO 启动 Live | 渐变壁纸、只有顶部一个面板 | 截图 |
| A2 | 看桌面 | 有主文件夹、文件系统、回收站图标 | 截图 |
| A3 | 打开几个窗口 | 窗口有阴影，主题一致 | 截图 |
| A4 | 启动后是否弹出“面板插件错误”之类的对话框 | 没有 | 截图 |
| A5 | 在 Thunar 中删除一个文件，打开回收站，恢复它 | 能删除、能恢复 | 说明结果 |
| A6 | 告诉我你对外观的意见 | — | 文字 |

---

### T7：Rust 工程骨架

**目标**：建立 Rust 工作区和 `feather-core` 库，在 CI 中编译和测试。

**分支**：`task/T7-rust`

**重要说明**：用户 Windows 上**不需要**安装 Rust。所有编译在 CI 中进行。

**步骤**：

1. 创建目录结构：
   ```text
   rust/
   ├── Cargo.toml            # 工作区
   ├── rust-toolchain.toml   # 固定 Rust 版本
   └── crates/
       └── feather-core/
           ├── Cargo.toml
           └── src/lib.rs
   ```

2. `rust/Cargo.toml`：
   ```toml
   [workspace]
   resolver = "2"
   members = ["crates/*"]

   [workspace.package]
   version = "0.0.1"
   edition = "2021"
   license = "MIT"
   ```

3. `rust/rust-toolchain.toml`：先查 Docker Hub 上 `rust` 官方镜像当前可用的 trixie 标签（例如 `rust:1.xx-trixie`），把版本号写在这里，并在 CI 中使用**同一版本**：
   ```toml
   [toolchain]
   channel = "1.xx.0"   # 替换为实际版本
   components = ["rustfmt", "clippy"]
   ```

4. `feather-core` 第一版只放最基础的东西（不要一次写太多）：
   - `TaskId`、`BotId`：包装 `String` 的结构体（newtype），实现 `Display`；
   - `TaskState` 枚举：`Queued`、`Running`、`Succeeded`、`Failed`、`Cancelled`；
   - 函数 `TaskState::can_transition_to(&self, next: &TaskState) -> bool`，规则：
     - `Queued` → `Running` 或 `Cancelled`
     - `Running` → `Succeeded`、`Failed` 或 `Cancelled`
     - 其他任何转换都返回 `false`
   - 至少 5 个单元测试，覆盖合法和非法转换。
   - 依赖只允许 `serde`（带 `derive` 特性），用 `cargo add` 添加，不要手写版本号。

5. 创建 `.github/workflows/rust.yml`：
   ```yaml
   name: rust
   on:
     push:
       branches: ["**"]
       paths: ["rust/**", ".github/workflows/rust.yml"]
     pull_request:
       paths: ["rust/**", ".github/workflows/rust.yml"]
   jobs:
     check:
       runs-on: ubuntu-latest
       container:
         image: rust:1.xx-trixie   # 与 rust-toolchain.toml 一致
       defaults:
         run:
           working-directory: rust
       steps:
         - uses: actions/checkout@v4
         - run: rustup component add rustfmt clippy
         - run: cargo fmt --all -- --check
         - run: cargo clippy --workspace --all-targets -- -D warnings
         - run: cargo test --workspace
   ```

6. `Cargo.lock` **必须提交**。由于本地没有 Rust，第一次推送后从 CI 中生成：在工作流里临时加一步 `cat Cargo.lock`，把内容复制回本地文件提交；或者让 CI 生成后上传 artifact，下载提交。确认 lock 已提交后删除临时步骤。

7. 提交信息：`feat(rust): workspace and feather-core basics`。

**完成标准**：`rust` 工作流绿色；汇报中贴出 `cargo test` 输出中的 `test result: ok. N passed` 一行原文。

**本任务的验收清单**：
| 编号 | 操作 | 应该看到 |
|---|---|---|
| A1 | 打开 rust 工作流的运行链接 | 绿色对勾 |
| A2 | 点开 `cargo test` 步骤 | 有 `test result: ok`，通过数 ≥ 5 |
| A3 | 在 GitHub 网页查看 `rust/Cargo.lock` | 文件存在 |

---

### T8：FeatherTerminal 第一版

**目标**：用 Rust + GTK4 + VTE 写一个窗口，里面运行**真实的 zsh**。本任务**只做终端本身**，不做侧栏。

**分支**：`task/T8-terminal`

**要求**：

1. 新建 crate `rust/crates/feather-terminal`（二进制程序）。
2. 依赖添加方式：
   ```text
   cargo add vte4
   cargo add gtk4 --rename gtk   （版本必须与 vte4 依赖的 gtk4 主版本一致）
   ```
   添加后在 CI 中运行 `cargo tree -i gtk4`，确认**只有一个** gtk4 版本，把输出贴进汇报。
3. CI 中编译需要系统库。在 `rust.yml` 的 steps 开头加：
   ```yaml
   - run: apt-get update && apt-get install -y libgtk-4-dev libvte-2.91-gtk4-dev pkg-config
   ```
4. 功能（逐项实现，每项都必须是真实功能）：
   - 窗口标题 “FeatherTerminal”，默认大小 1000×650；
   - 窗口内是一个 `vte4::Terminal`，用 `spawn_async` 启动 `/usr/bin/zsh`（如果不存在则用 `$SHELL`，再不存在用 `/bin/bash`），工作目录为用户主目录；
   - **函数签名以 docs.rs 上你实际锁定的 vte4 版本为准**，在汇报中写出你查阅的 docs.rs 链接；
   - zsh 退出时关闭窗口；
   - 快捷键：`Ctrl+Shift+C` 复制，`Ctrl+Shift+V` 粘贴（用 VTE 自带的复制粘贴方法）；
   - `Ctrl+C` **不要拦截**，让它传给终端里的程序。
5. **禁止**：用 `TextView` 或 `Label` 模拟终端；禁止自己写命令解析；禁止“输入框 + 发送按钮”。
6. CI 中把编译出的程序作为 artifact 上传（`cargo build --release -p feather-terminal`，上传 `rust/target/release/feather-terminal`）。
7. 提交信息：`feat(terminal): GTK4 + VTE window running real zsh`。

**完成标准**：rust 工作流绿色；release 二进制已上传；`cargo tree -i gtk4` 只有一个版本。

**本任务的验收清单**：
| 编号 | 操作 | 应该看到 |
|---|---|---|
| A1 | 打开 rust 工作流链接 | 绿色对勾，Artifacts 中有 feather-terminal |
| A2 | 阅读汇报第 3 节 | 有 `cargo tree -i gtk4` 输出和 docs.rs 链接 |
| A3 | （本任务不需要在 VMware 测试，T9 会装进 ISO 一起测） | — |

---

### T9：把 FeatherTerminal 装进 ISO

**目标**：把 T8 的程序打成 `.deb`，装进 ISO，桌面上有图标，双击打开。

**分支**：`task/T9-terminal-iso`

**要求**：

1. 用 `cargo-deb` 打包。在 `feather-terminal/Cargo.toml` 中加入 `[package.metadata.deb]`：
   - `maintainer`：用户的 GitHub 用户名；
   - `depends = "$auto, zsh"`；
   - `assets`：二进制装到 `usr/bin/feather-terminal`，`.desktop` 文件装到 `usr/share/applications/mengos-terminal.desktop`。
2. 创建 `rust/crates/feather-terminal/assets/mengos-terminal.desktop`：
   ```ini
   [Desktop Entry]
   Type=Application
   Name=FeatherTerminal
   Name[zh_CN]=FeatherTerminal 终端
   Comment=MengOS terminal
   Exec=feather-terminal
   Icon=utilities-terminal
   Terminal=false
   Categories=System;TerminalEmulator;
   ```
3. 修改 `build-iso.yml`，改为两个 job：
   - `build-deb`：在 `rust:1.xx-trixie` 容器中安装依赖，`cargo install cargo-deb --locked`，执行 `cargo deb -p feather-terminal`，上传 `.deb` 为 artifact；
   - `build-iso`：`needs: build-deb`，下载 `.deb` 到 `iso/config/packages.chroot/`（live-build 会自动安装这个目录里的 deb），然后构建 ISO。
   - 同时把 `build-iso.yml` 的触发路径加上 `rust/**`。
4. 让 Live 桌面上出现图标：创建 hook，把 `.desktop` 复制到 `/etc/skel/Desktop/`（新用户桌面目录）并加可执行权限；Xfce 可能提示“不受信任的启动器”，在汇报中如实写明用户会看到什么。
5. `.gitignore` 中加入 `/iso/config/packages.chroot/*.deb`（deb 只在 CI 中生成，不提交）。
6. 构建成功后，在 `.packages.txt` 中找到 `feather-terminal` 那一行贴进汇报。
7. 提交信息：`feat(iso): ship FeatherTerminal in the ISO`。

**本任务的验收清单**：
| 编号 | 你要做的操作 | 应该看到的结果 | 如果不对，请发给我 |
|---|---|---|---|
| A1 | 用新 ISO 启动 Live | 桌面上有 FeatherTerminal 图标 | 截图 |
| A2 | 双击图标（如提示“不受信任”，点“标记为可执行/信任并启动”） | 打开 FeatherTerminal 窗口 | 截图 |
| A3 | 输入 `echo hello` 回车 | 显示 `hello` | 截图 |
| A4 | 输入 `python3 -c 'print(42)'` 回车 | 显示 `42` | 截图 |
| A5 | 输入 `ping -c 3 debian.org` 回车，运行中按 `Ctrl+C` | ping 被中断 | 截图 |
| A6 | 用鼠标选中一段文字，按 `Ctrl+Shift+C`，再到 Mousepad 粘贴 | 能粘贴 | 说明 |
| A7 | 输入 `btop` 回车，然后按 `q` 退出 | btop 正常显示、能退出 | 截图 |
| A8 | 在终端里按 `Ctrl+空格`，输入 `nihao` | 出现中文候选框 | 截图 |
| A9 | 输入 `exit` | 窗口关闭 | 说明 |

**T9 完成后**：整个第一批任务结束。代理在汇报最后写：“第一批任务 T0–T9 已全部完成，请用户找任务书作者生成第二批任务卡（侧栏、FeatherBot 等）。”
