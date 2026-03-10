# Spoon

Windows development environment management toolkit, running as a Claude Code plugin.

## Skills

- `scoop` — Manage the Scoop package manager and scoop-installed software
- `proxy` — Manage proxy settings and mirror sources for development tools

`ai-setup/` is the separate Rust bootstrap surface for Git, Claude Code, Codex, proxy bootstrap, and AI helper CLI tooling. Do not add those workflows back into scoop recipes.

## Project Conventions

- All skill documents use English as the primary version, with a `-zh.md` Chinese counterpart.
- Chinese versions must include the same executable command blocks as the English version.
- Recipes go under the corresponding skill's `references/recipes/` directory.
- Shared scripts go in `scripts/` and are referenced from skills as needed.
- AI bootstrap flows belong in the `ai-setup/` Rust project, not in the spoon skills.

## Development Guidelines

### Adding a Recipe

Use recipes only for scoop-managed packages that need extra post-install configuration.

Template:
1. When to install
2. Install command
3. Post-install configuration
4. Verify
5. Uninstall and cleanup options

Config check priority: config files first, environment variables second.
Sensitive values must be masked when displayed.

### Adding a Skill

Use a skill when the domain requires ongoing operations and management, not just one-time installation.

### Script Invocation

- Run commands: `powershell -File <plugin_root>/scripts/run-cmd.ps1 <cmd> <args>`
- Manage PATH: `powershell -File <plugin_root>/scripts/add-path.ps1 <app> <subdirs>`
- Non-scoop PowerShell commands: `powershell -Command '...'`

## Git & Release

### settings.json Strategy

- `.claude/settings.json` — Hooks only, committed to git
- `.claude/settings.local.json` — Permissions, gitignored
- Claude Code auto-inserts permission entries into settings.json — clean them before release

### Release Steps

1. Clean auto-inserted permissions from `.claude/settings.json`
2. Update `version` in `.claude-plugin/plugin.json`
3. Update `README.md` and `README-zh.md` if behavior changed
4. Commit and push
5. Create a release tag and publish the plugin release

### Versioning

- `patch`: bug fixes, recipe improvements, documentation updates
- `minor`: new skill or new supported workflow
- `major`: breaking changes

