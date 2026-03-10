---
name: ai-toolchain
description: Use this skill when operating within the ai-setup toolchain on Windows: Git, Git Bash, gh, python, which, make, 7zip, just, rg, fd, jq, bat, delta, ast-grep, and yq.
---

# AI Toolchain

This skill is the usage guide for the workstation tools provisioned by `ai-setup.exe`.

## Scope

Use this skill when:

- working in Git Bash on Windows
- composing repository and text-search pipelines
- choosing between `git`, `gh`, `rg`, `fd`, `jq`, `yq`, `bat`, `delta`, `sg`, `python`, `which`, `make`, `7z`, and `just`
- writing Python scripts or helpers

## Tool routing

- `git`: repository state, branches, history, diffs, staging
- `gh`: GitHub issues, PRs, releases, repo metadata
- `rg`: fast content search
- `fd`: fast file discovery
- `sg`: syntax-aware code search
- `jq` / `yq`: structured JSON and YAML processing
- `bat`: readable file preview
- `delta`: readable diff output
- `python`: Python scripts, pip package management, virtual environments
- `which`: locate executables on PATH
- `make`: build automation (GNU Make)
- `7z`: archive compression and extraction (7-Zip)
- `just`: project-level command runner

## Preferred patterns

- Prefer Git Bash for repository and text-processing workflows.
- Prefer PowerShell only for Windows registry-backed environment changes.
- Prefer `rg` over slower recursive grep alternatives.
- Prefer `fd` before heavier recursive file listings.
- Use `python -m venv` for virtual environments and `pip` for package management.

## Quick recipes

```bash
git status
rg FIXME src
fd package.json
bat package.json
jq '.name' package.json
sg --pattern 'console.log($X)' .
python -m pip install requests
which python
make -j4
7z x archive.zip
just build
```

## Notes

- Assume Git Bash is provided by the Git installation managed through `ai-setup`.
- If one of these tools is missing, install it through `ai-setup.exe` CLI rather than through scoop.
- Example commands:
  - `.\ai-setup.exe --action install --tools git,gh,claude,codex --non-interactive`
  - `.\ai-setup.exe --action install --tools rg,fd,jq,bat,delta,sg,yq,python,which,make,7zip,just --non-interactive`
