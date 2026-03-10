# Spoon

用于管理 Windows 开发环境的 Claude Code 插件。

Spoon 现在只聚焦两类能力：

- Scoop 包管理器及其软件生态
- 常见开发工具的代理与镜像配置

AI 工作站初始化已经独立到仓库中的 Rust 项目 `ai-setup/`。Git、Claude Code、Codex、代理引导以及 AI 辅助 CLI 安装都由这个可执行程序负责。

## Skills

### scoop

负责 [Scoop](https://scoop.sh/) 包管理器及其安装的软件。

- 安装、卸载、更新 scoop 和 scoop 软件包
- Bucket 管理（添加、移除、查看）
- 健康检查与缓存清理
- 需要额外配置的 scoop 软件 recipe（android-clt、flutter、nodejs、pixi、pkl-cli、rustup）

### ai-toolchain

`ai-setup.exe` 提供的工作站工具使用指南（git、gh、rg、fd、jq、yq、bat、delta、sg、python、which、make、7z、just）。

### proxy

负责统一管理代理和镜像配置。

- 为 git、scoop、npm、pip、cargo、flutter 等工具设置 HTTP/SOCKS5 代理
- 管理国内镜像源（TUNA、USTC、SJTUG）
- 统一启用 / 禁用代理配置

## 项目结构

```text
spoon/
├── ai-setup/                  # AI 工作站初始化 Rust CLI/TUI 项目
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json
├── skills/
│   ├── scoop/
│   │   ├── SKILL.md
│   │   └── references/
│   │       ├── commands.md / commands-zh.md
│   │       ├── guide-zh.md
│   │       └── recipes/            # 安装后配置 recipe（英文 + 中文）
│   │           └── android-clt, flutter, nodejs, pixi, pkl-cli, rustup
│   ├── proxy/
│   │   ├── SKILL.md
│   │   └── references/
│   │       └── guide-zh.md
│   └── ai-toolchain/
│       ├── SKILL.md
│       └── SKILL-zh.md
├── scripts/
│   ├── run-cmd.ps1
│   └── add-path.ps1
├── CLAUDE.md
├── README.md
└── README-zh.md
```

## AI Setup

`ai-setup/` 目录包含 AI 工作站初始化的 Rust CLI/TUI 项目。构建方式：

```text
cd ai-setup && cargo xtask deploy
```

这会编译 release 版本并将 `ai-setup.exe` 拷贝到仓库根目录（已 gitignore）。示例：

```text
.\ai-setup.exe
.\ai-setup.exe --action status --non-interactive
.\ai-setup.exe --action install --tools git,claude,codex,rg
```
## 安装

在 Claude Code 中运行：

```text
/plugin marketplace add VIDLG/spoon
```

然后从 marketplace 安装 spoon 插件。

## 系统要求

- Windows 10/11
- [Claude Code](https://claude.ai/code) CLI

## 许可证

MIT


