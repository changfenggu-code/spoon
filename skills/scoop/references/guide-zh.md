# Scoop 使用说明（中文）

本文件是 spoon 中 `scoop` skill 的中文说明，强调当前边界：

- Scoop skill 负责 scoop 本身、bucket 管理，以及普通 scoop 软件包
- `ai-setup/` Rust 可执行程序负责 AI 基础设施保留包

## 适用范围

适用于以下请求：

- 安装、卸载、更新 scoop
- 添加、移除、查看 bucket
- 使用 scoop 安装普通开发工具
- 清理缓存、检查健康状态、查看已安装软件

## AI 保留包

以下工具不应再通过 scoop skill 安装、更新或卸载，而应统一交给仓库根目录的 `ai-setup.exe`：

- `git`
- `gh`
- `claude-code`
- `codex`
- `ripgrep`
- `fd`
- `jq`
- `bat`
- `delta`
- `ast-grep`
- `yq`
- `python`
- `which`
- `make`
- `7zip`
- `just`

如果用户请求这些工具，应明确转交给 `.\ai-setup.exe`。建议使用：
- `.\ai-setup.exe --action install --tools git,gh,claude,codex --non-interactive`
- `.\ai-setup.exe --action update --tools rg,fd,jq,bat,delta,sg,yq,python,which,make,7zip,just --non-interactive`
- `.\ai-setup.exe --action uninstall --tools codex,claude --non-interactive`

## 安装 Scoop

1. 检查是否已安装：`scoop --version`
2. 让用户确认安装路径，例如 `D:\Scoop` 或默认 `~/scoop`
3. 执行官方安装脚本
4. 设置用户环境变量 `SCOOP`
5. 提示用户重启 VS Code 或终端，使新环境变量生效

推荐通过仓库脚本刷新环境后再运行 scoop：

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop --version
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop bucket add extras
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop install nodejs
```

## Bucket 管理

常见 bucket：

- `extras`
- `versions`
- `java`
- `nerd-fonts`

添加 bucket：

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop bucket add <name>
```

如果 `scoop bucket add` 因远端 refs 太多而失败，可退回到手动 clone：

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 git clone --depth 1 https://github.com/ScoopInstaller/<BucketName>.git <SCOOP>/buckets/<bucket_name>
```

注意：这里依赖系统中已经存在 `git`，通常由 `ai-setup.exe` 提供。

## 常用操作

```bash
powershell -Command "scoop search <query>"
powershell -Command "scoop install <app>"
powershell -Command "scoop uninstall <app>"
powershell -Command "scoop update"
powershell -Command "scoop update *"
powershell -Command "scoop list"
powershell -Command "scoop status"
powershell -Command "scoop info <app>"
powershell -Command "scoop bucket list"
powershell -Command "scoop bucket add <name>"
powershell -Command "scoop bucket rm <name>"
powershell -Command "scoop cleanup *"
powershell -Command "scoop cache rm *"
powershell -Command "scoop checkup"
```

## Recipe 规则

安装 scoop 软件包后，如果 `references/recipes/` 下存在对应 recipe，则可继续执行安装后配置。

可用 recipe：
- `android-clt` — Android SDK 命令行工具
- `flutter` — Flutter SDK（含 Dart）
- `nodejs` — Node.js / npm
- `pixi` — 项目级包管理器
- `pkl-cli` — Apple 配置语言
- `rustup` — Rust 工具链管理器

### 别名解析

用户可能用别名代替实际包名，安装前需先解析：
- "安装 npm" → 安装 `nodejs`（或 `nodejs-lts`），应用 `recipes/nodejs.md`
- "安装 pip" → `python` 是 AI 保留包，转交 `.\ai-setup.exe`
- "安装 cargo" / "安装 rust" → 安装 `rustup`，应用 `recipes/rustup.md`
- "安装 dart" → 安装 `flutter`，应用 `recipes/flutter.md`
- "安装 android sdk" / "安装 android" → 安装 `android-clt`，应用 `recipes/android-clt.md`

AI 保留包不走 recipe：

- `git`、`gh`、`claude-code`、`codex`

这些统一由 `ai-setup.exe` 管理。`ripgrep`、`fd`、`jq`、`bat`、`delta`、`ast-grep`、`yq`、`python`、`which`、`make`、`7zip`、`just` 这些 AI helper CLI 也不再走 scoop recipe。

## 卸载 Scoop

卸载 scoop 前应再次确认，因为这是破坏性操作。

核心步骤：

1. `scoop uninstall scoop`
2. 清理 `SCOOP` / `SCOOP_GLOBAL`
3. 清理 PATH 中的 scoop 项
4. 删除 scoop 安装目录

## 代理与镜像

代理和镜像配置交给 `proxy` skill 处理。

当 scoop 因网络或 SSL 问题失败时，应转交给 `proxy` skill，而不是在本文件里重复维护另一套逻辑。
