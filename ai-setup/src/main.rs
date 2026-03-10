mod cli;
mod config;
mod operations;
mod skills;
mod status;
mod tool;
mod tui;
mod winget;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use cli::{Action, Cli};
use operations::ToolAction;

fn print_result_lines(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}

fn execute_cli_tool_action(
    action: Action,
    selected_tools: &[&'static tool::Tool],
    install_root: Option<&std::path::Path>,
) -> Result<()> {
    if selected_tools.is_empty() {
        println!("No valid tools selected.");
        return Ok(());
    }

    let tool_action = match action {
        Action::Install => ToolAction::Install,
        Action::Update => ToolAction::Update,
        Action::Uninstall => ToolAction::Uninstall,
        _ => unreachable!("action is filtered by caller"),
    };

    let results = operations::execute_tool_action(tool_action, selected_tools, install_root)?;
    let lines = operations::flatten_command_results(results);
    print_result_lines(&lines);
    Ok(())
}

fn detect_repo_root() -> PathBuf {
    // 1. exe 所在目录有 skills/ → 已部署到仓库根
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("skills").is_dir() {
                return dir.to_path_buf();
            }
        }
    }
    // 2. 当前工作目录有 skills/ → 从仓库根运行
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("skills").is_dir() {
            return cwd;
        }
    }
    // 3. 开发环境 fallback（cargo run 时有效）
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    status::refresh_process_env_from_registry()?;

    let install_root = cli.install_path.or_else(tool::default_install_root);
    let repo_root = detect_repo_root();

    if cli.action.is_none() && !cli.non_interactive {
        return tui::run_tui(install_root, repo_root);
    }

    let action = cli.action.unwrap_or(Action::Status);
    let selected_tools = tool::resolve_requested_tools(&cli.tools);

    match action {
        Action::Status => status::print_status(install_root.as_deref()),
        Action::Install | Action::Update | Action::Uninstall => {
            execute_cli_tool_action(action, &selected_tools, install_root.as_deref())?;
        }
        Action::Configure => {
            println!(
                "configure action is available in TUI mode. Run `ai-setup` without --non-interactive."
            );
        }
        Action::Skills => {
            let available = skills::available_skills(&repo_root);
            for line in skills::install_skills(&available)? {
                println!("{line}");
            }
        }
    }

    Ok(())
}
