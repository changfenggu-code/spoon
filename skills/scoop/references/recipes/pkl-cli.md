# pkl-cli - Post-Install Recipe

## When to install

Install pkl-cli when the user:

- Says "install pkl", "I need pkl", or "set up pkl"
- Works with `.pkl` configuration files
- Needs to generate JSON, YAML, or plist output from structured configuration
- Mentions Pkl, Apple's configuration language, or configuration-as-code workflows
- Wants to validate or evaluate Pkl modules from the command line

## Install

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop install pkl-cli
```

## Post-Install Configuration

pkl-cli works out of the box. Optional setup:

### Editor support

Ask whether the user wants the Pkl VS Code extension.

First check whether it is already installed:

```bash
code --list-extensions | grep -i pkl
```

If not installed, download the latest VSIX from the GitHub release API and install it. This keeps the recipe self-contained and avoids any extra CLI dependency.

```bash
powershell -Command '$release = Invoke-RestMethod -Uri "https://api.github.com/repos/apple/pkl-vscode/releases/latest"; $asset = $release.assets | Where-Object { $_.name -like "*.vsix" } | Select-Object -First 1; if (-not $asset) { throw "No VSIX asset found" }; $target = Join-Path $env:TEMP $asset.name; Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $target; Write-Output $target'
code --install-extension "$TEMP\pkl-vscode-*.vsix"
```

If GitHub access is blocked, tell the user to configure proxy first or download the VSIX manually.

The extension requires Java 22+ to run the Pkl Language Server. If Java is unavailable, ask whether to install it via scoop (`scoop install temurin22-jdk`).

### Environment variable (optional)

| Variable | Description | Default |
|----------|-------------|---------|
| `PKL_HOME` | Custom directory for Pkl caches and packages | `~/.pkl` |

## Verify

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 pkl-cli --version
```

## Uninstall

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop uninstall pkl-cli
```


