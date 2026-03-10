use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGroup {
    Core,
    Helper,
}

#[derive(Debug, Clone, Copy)]
pub struct Tool {
    pub key: &'static str,
    pub display_name: &'static str,
    pub command: &'static str,
    pub winget_id: &'static str,
    pub dir_name: &'static str,
    pub group: ToolGroup,
}

pub const TOOLS: &[Tool] = &[
    Tool {
        key: "git",
        display_name: "Git",
        command: "git",
        winget_id: "Git.Git",
        dir_name: "git",
        group: ToolGroup::Core,
    },
    Tool {
        key: "claude",
        display_name: "Claude Code",
        command: "claude",
        winget_id: "Anthropic.ClaudeCode",
        dir_name: "claude-code",
        group: ToolGroup::Core,
    },
    Tool {
        key: "codex",
        display_name: "Codex",
        command: "codex",
        winget_id: "OpenAI.Codex",
        dir_name: "codex",
        group: ToolGroup::Core,
    },
    Tool {
        key: "gh",
        display_name: "GitHub CLI",
        command: "gh",
        winget_id: "GitHub.cli",
        dir_name: "gh",
        group: ToolGroup::Core,
    },
    Tool {
        key: "rg",
        display_name: "ripgrep",
        command: "rg",
        winget_id: "BurntSushi.ripgrep.MSVC",
        dir_name: "ripgrep",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "fd",
        display_name: "fd",
        command: "fd",
        winget_id: "sharkdp.fd",
        dir_name: "fd",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "jq",
        display_name: "jq",
        command: "jq",
        winget_id: "jqlang.jq",
        dir_name: "jq",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "bat",
        display_name: "bat",
        command: "bat",
        winget_id: "sharkdp.bat",
        dir_name: "bat",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "delta",
        display_name: "delta",
        command: "delta",
        winget_id: "dandavison.delta",
        dir_name: "delta",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "sg",
        display_name: "ast-grep",
        command: "sg",
        winget_id: "ast-grep.ast-grep",
        dir_name: "ast-grep",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "yq",
        display_name: "yq",
        command: "yq",
        winget_id: "MikeFarah.yq",
        dir_name: "yq",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "python",
        display_name: "Python",
        command: "python",
        // NOTE: pinned to minor version; update when new Python 3.x releases
        winget_id: "Python.Python.3.13",
        dir_name: "python",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "which",
        display_name: "which",
        command: "which",
        // FIXME: may not exist in winget repo yet (GitHub issue #111751)
        winget_id: "GnuWin32.Which",
        dir_name: "which",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "make",
        display_name: "GNU Make",
        command: "make",
        // NOTE: GnuWin32 ships Make 3.81 (2006); no newer winget package available
        winget_id: "GnuWin32.Make",
        dir_name: "make",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "7zip",
        display_name: "7-Zip",
        command: "7z",
        winget_id: "7zip.7zip",
        dir_name: "7zip",
        group: ToolGroup::Helper,
    },
    Tool {
        key: "just",
        display_name: "Just",
        command: "just",
        winget_id: "Casey.Just",
        dir_name: "just",
        group: ToolGroup::Helper,
    },
];

pub fn find_tool(key: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|tool| tool.key.eq_ignore_ascii_case(key))
}

pub fn all_tools() -> Vec<&'static Tool> {
    TOOLS.iter().collect()
}

pub fn resolve_requested_tools(requested: &[String]) -> Vec<&'static Tool> {
    if requested.is_empty() {
        return Vec::new();
    }

    let mut selected = Vec::new();
    for item in requested {
        match item.trim().to_ascii_lowercase().as_str() {
            "all" => selected.extend(all_tools()),
            "core" => selected.extend(TOOLS.iter().filter(|tool| tool.group == ToolGroup::Core)),
            "helpers" => {
                selected.extend(TOOLS.iter().filter(|tool| tool.group == ToolGroup::Helper))
            }
            other => {
                if let Some(tool) = find_tool(other) {
                    selected.push(tool);
                }
            }
        }
    }

    selected.sort_by_key(|tool| tool.key);
    selected.dedup_by_key(|tool| tool.key);
    selected
}

pub fn expected_tool_dir(root: Option<&Path>, tool: &Tool) -> Option<PathBuf> {
    root.map(|base| base.join(tool.dir_name))
}

pub fn default_install_root() -> Option<PathBuf> {
    env::var_os("AI_SETUP_INSTALL_ROOT").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_tool_returns_correct_tool() {
        let tool = find_tool("git").unwrap();
        assert_eq!(tool.command, "git");
        assert_eq!(tool.display_name, "Git");
    }

    #[test]
    fn find_tool_case_insensitive() {
        assert!(find_tool("GIT").is_some());
        assert!(find_tool("Git").is_some());
    }

    #[test]
    fn find_tool_returns_none_for_unknown() {
        assert!(find_tool("nonexistent").is_none());
    }

    #[test]
    fn resolve_all_returns_all_tools() {
        let tools = resolve_requested_tools(&["all".to_string()]);
        assert_eq!(tools.len(), TOOLS.len());
    }

    #[test]
    fn resolve_core_returns_core_tools() {
        let tools = resolve_requested_tools(&["core".to_string()]);
        assert!(tools.iter().all(|t| t.group == ToolGroup::Core));
        assert_eq!(tools.len(), 4);
    }

    #[test]
    fn resolve_helpers_returns_helper_tools() {
        let tools = resolve_requested_tools(&["helpers".to_string()]);
        assert!(tools.iter().all(|t| t.group == ToolGroup::Helper));
        assert_eq!(tools.len(), TOOLS.len() - 4);
    }

    #[test]
    fn resolve_deduplicates() {
        let tools = resolve_requested_tools(&["git".to_string(), "git".to_string()]);
        assert_eq!(tools.len(), 1);
    }

    #[test]
    fn resolve_empty_returns_empty() {
        let tools = resolve_requested_tools(&[]);
        assert!(tools.is_empty());
    }
}
