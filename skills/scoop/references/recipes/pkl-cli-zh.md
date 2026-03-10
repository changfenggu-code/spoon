# pkl-cli - 安装后配置（中文）

## 何时安装

适用于以下场景：

- 用户说“安装 pkl”“我要 pkl”“配置 pkl”
- 需要处理 `.pkl` 配置文件
- 需要从结构化配置生成 JSON、YAML、plist
- 提到 Apple 的 Pkl 配置语言
- 需要在命令行验证或执行 Pkl 模块

## 安装

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop install pkl-cli
```

## 安装后配置

pkl-cli 本身几乎开箱即用。可选项主要是 VS Code 扩展。

### 编辑器支持

先检查是否已经安装 Pkl VS Code 扩展：

```bash
code --list-extensions | grep -i pkl
```

如果未安装，可直接通过 GitHub Release API 下载最新 VSIX 并安装，这样就不再依赖额外的 CLI。

```bash
powershell -Command '$release = Invoke-RestMethod -Uri "https://api.github.com/repos/apple/pkl-vscode/releases/latest"; $asset = $release.assets | Where-Object { $_.name -like "*.vsix" } | Select-Object -First 1; if (-not $asset) { throw "No VSIX asset found" }; $target = Join-Path $env:TEMP $asset.name; Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $target; Write-Output $target'
code --install-extension "$TEMP\pkl-vscode-*.vsix"
```

如果 GitHub 无法访问，应先配置代理，或者手动下载 VSIX。

该扩展需要 Java 22+ 来运行 Pkl Language Server。如未安装 Java，可继续通过 scoop 安装：

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop install temurin22-jdk
```

### 环境变量（可选）

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `PKL_HOME` | Pkl 缓存和包的自定义目录 | `~/.pkl` |

## 验证

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 pkl-cli --version
```

## 卸载

```bash
powershell -File <plugin_root>/scripts/run-cmd.ps1 scoop uninstall pkl-cli
```


