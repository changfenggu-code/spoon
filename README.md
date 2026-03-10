# Spoon

A Claude Code plugin for managing Windows development environments.

Spoon focuses on two areas:

- Scoop package manager operations for general Windows development tools
- Proxy and mirror management across common development tooling

AI workstation bootstrap is handled separately by the Rust project in `ai-setup/`. That executable owns Git, Claude Code, Codex, proxy bootstrap, and AI helper CLI installation.

## Skills

### scoop

Manages the [Scoop](https://scoop.sh/) package manager and software installed through it.

- Install/uninstall/update scoop and scoop packages
- Bucket management (add, remove, list)
- Health checks and cache cleanup
- Post-install recipes for scoop-managed tools that need extra setup (android-clt, flutter, nodejs, pixi, pkl-cli, rustup)

### ai-toolchain

Usage guide for the workstation tools provisioned by `ai-setup.exe` (git, gh, rg, fd, jq, yq, bat, delta, sg, python, which, make, 7z, just).

### proxy

Manages proxy and mirror configuration across development tools.

- HTTP/SOCKS5 proxy for git, scoop, npm, pip, cargo, flutter, etc.
- China mirror sources (TUNA, USTC, SJTUG) for package registries
- Unified enable/disable across all tools

## Project Structure

```text
spoon/
├── ai-setup/                  # Rust CLI/TUI project for AI workstation setup
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json
├── skills/
│   ├── scoop/
│   │   ├── SKILL.md
│   │   └── references/
│   │       ├── commands.md / commands-zh.md
│   │       ├── guide-zh.md
│   │       └── recipes/            # Post-install recipes (en + zh pairs)
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

The `ai-setup/` directory contains a Rust CLI/TUI for AI workstation bootstrap. Build the binary with:

```text
cd ai-setup && cargo xtask deploy
```

This compiles a release build and copies `ai-setup.exe` to the repository root (gitignored). Examples:

```text
.\ai-setup.exe
.\ai-setup.exe --action status --non-interactive
.\ai-setup.exe --action install --tools git,claude,codex,rg
```
## Installation

In Claude Code, run:

```text
/plugin marketplace add VIDLG/spoon
```

Then install the spoon plugin from the marketplace. The plugin will be available across all your projects.

## Requirements

- Windows 10/11
- [Claude Code](https://claude.ai/code) CLI

## License

MIT


