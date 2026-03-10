# AGENTS.md

## ai-setup Direction

- `ai-setup` is now a Rust project under `ai-setup/`.
- The primary runnable artifact is the repository-root `ai-setup.exe`.
- Do not re-introduce PowerShell or shell scripts as the primary ai-setup entrypoint.
- CLI mode must remain available for automation.
- TUI mode is the default interactive experience.

## UI Decisions

- `install`, `update`, and `uninstall` live in one unified tools page.
- Do not split those operations back into separate pages unless there is a very strong reason.
- The tools page should preselect actionable items by mode:
  - `install` skips already-installed tools
  - `update` skips missing tools
  - `uninstall` skips missing tools
- `?` is the detailed help trigger for the current page.
- Avoid cluttering pages with duplicate help panels if `?` already covers it.
- Prefer compact summaries over large decorative stat cards when space can be used for real status content.

## Tool Scope

- Core tools owned by `ai-setup`:
  - `git`
  - `gh`
  - `claude`
  - `codex`
- Helper tools owned by `ai-setup`:
  - `rg`
  - `fd`
  - `jq`
  - `bat`
  - `delta`
  - `sg`
  - `yq`
  - `python`
  - `which`
  - `make`
  - `7zip`
  - `just`
- Do not add `fzf` by default.
- Spoon scoop recipes should not take ownership of the AI-reserved tools above.

## Configuration Scope

- `ai-setup` owns configuration for:
  - Git identity / default branch
  - Claude Code settings
  - Codex config and auth
  - Claude skill installation into `~/.claude/skills`

## Path / Environment Rules

- Do not hardcode install paths such as `D:\devtools`.
- If a default install root is needed, make it configurable, not embedded in code.
- Refreshing environment variables must distinguish between:
  - current-process changes
  - persisted user/machine environment changes

## Artifact Update Rules

- When refreshing the root `ai-setup.exe`, prefer replacing it in place.
- Do not create extra renamed variants unless blocked and no cleaner option exists.
- If the exe is locked, first try to stop the running `ai-setup` process, then replace the file.

## Repository Hygiene

- The old PowerShell / shell ai-setup flow is deprecated.
- Do not move AI bootstrap logic back into spoon skills or scoop recipes.
- Keep docs aligned with the Rust executable workflow.

## Encoding Rules

- Use UTF-8 (without BOM) for all text files (`.md`, `.rs`, `.toml`, `.json`, `.yml`, `.yaml`, `.ps1`).
- Do not commit files encoded as GBK/ANSI/UTF-16.
- If touching a legacy file with encoding issues, convert it to UTF-8 in the same change.
