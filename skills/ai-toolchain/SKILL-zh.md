---
name: ai-toolchain
---

# AI Toolchain（中文）

本文件是 `SKILL.md` 的中文对照版。

这是 `ai-setup.exe` 提供的 AI 工作站工具使用说明。

## 适用场景

适用于以下情况：

- 在 Windows 上使用 Git Bash
- 组合仓库、搜索、结构化数据处理命令
- 在 `git`、`gh`、`rg`、`fd`、`jq`、`yq`、`bat`、`delta`、`sg`、`python`、`which`、`make`、`7z`、`just` 之间做工具选择
- 编写或运行 Python 脚本和辅助程序

## 工具分工

- `git`：仓库状态、分支、历史、diff、暂存
- `gh`：GitHub issue、PR、release、仓库元数据
- `rg`：全文搜索
- `fd`：文件搜索
- `sg`：语法树级搜索
- `jq` / `yq`：JSON / YAML 处理
- `bat`：可读文件预览
- `delta`：可读 diff
- `python`：Python 脚本、pip 包管理、虚拟环境
- `which`：查找 PATH 中的可执行文件位置
- `make`：构建自动化（GNU Make）
- `7z`：压缩包解压与创建（7-Zip）
- `just`：项目级命令运行器

## 推荐模式

- 仓库和文本工作流优先使用 Git Bash。
- Windows 注册表类环境变量修改优先使用 PowerShell。
- 内容搜索优先 `rg`。
- 文件发现优先 `fd`。
- 虚拟环境使用 `python -m venv`，包管理使用 `pip`。

## 常用示例

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

## 说明

- 默认假设 Git Bash 由 `ai-setup` 管理的 Git 安装提供。
- 如果缺少这些工具，应通过 `ai-setup.exe` CLI 安装，而不是走 scoop。
- 示例命令：
  - `.\ai-setup.exe --action install --tools git,gh,claude,codex --non-interactive`
  - `.\ai-setup.exe --action install --tools rg,fd,jq,bat,delta,sg,yq,python,which,make,7zip,just --non-interactive`
