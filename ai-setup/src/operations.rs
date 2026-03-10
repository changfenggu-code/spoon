use std::path::Path;

use anyhow::Result;

use crate::tool::Tool;
use crate::winget::{self, CommandResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAction {
    Install,
    Update,
    Uninstall,
}

pub fn execute_tool_action(
    action: ToolAction,
    tools: &[&'static Tool],
    install_root: Option<&Path>,
) -> Result<Vec<CommandResult>> {
    match action {
        ToolAction::Install => winget::install_tools(tools, install_root),
        ToolAction::Update => winget::update_tools(tools),
        ToolAction::Uninstall => winget::uninstall_tools(tools),
    }
}

pub fn flatten_command_results(results: Vec<CommandResult>) -> Vec<String> {
    let mut lines = Vec::new();
    for result in results {
        lines.push(format!("== {} ==", result.title));
        if result.output.is_empty() {
            lines.push("(no output)".to_string());
        } else {
            lines.extend(result.output);
        }
        lines.push(String::new());
    }
    lines
}
