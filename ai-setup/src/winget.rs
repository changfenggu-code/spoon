use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::status;
use crate::tool::Tool;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub title: String,
    pub success: bool,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum RunMode {
    Install,
    Update,
    Uninstall,
}

fn strip_ansi_and_controls(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek().copied() {
                // CSI: ESC [ ... final-byte
                Some('[') => {
                    let _ = chars.next();
                    for c in chars.by_ref() {
                        if ('@'..='~').contains(&c) {
                            break;
                        }
                    }
                }
                // OSC: ESC ] ... BEL or ESC \
                Some(']') => {
                    let _ = chars.next();
                    let mut prev = '\0';
                    for c in chars.by_ref() {
                        if c == '\x07' || (prev == '\x1b' && c == '\\') {
                            break;
                        }
                        prev = c;
                    }
                }
                _ => {}
            }
            continue;
        }

        if ch == '\x08' {
            let _ = out.pop();
            continue;
        }

        if ch.is_control() && ch != '\t' {
            continue;
        }

        out.push(ch);
    }

    out
}

fn normalize_output_stream(raw: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(raw)
        .replace('\r', "\n")
        .lines()
        .map(strip_ansi_and_controls)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn run_winget(action: &str, tool: &Tool, install_root: Option<&Path>) -> Result<CommandResult> {
    let mut args = vec![
        action.to_string(),
        "--id".into(),
        tool.winget_id.into(),
        "-e".into(),
        "--disable-interactivity".into(),
    ];
    if action == "install" {
        args.push("--accept-package-agreements".into());
        args.push("--accept-source-agreements".into());
        if let Some(root) = install_root {
            args.push("--location".into());
            args.push(root.join(tool.dir_name).display().to_string());
        }
    }

    let output = Command::new("winget")
        .args(&args)
        .output()
        .with_context(|| format!("failed to run winget {} for {}", action, tool.display_name))?;

    let mut lines = Vec::new();
    lines.push(format!(
        "> winget {} --id {} -e --disable-interactivity",
        action, tool.winget_id
    ));
    lines.extend(normalize_output_stream(&output.stdout));
    lines.extend(normalize_output_stream(&output.stderr));

    Ok(CommandResult {
        title: format!("{} {}", action, tool.display_name),
        success: output.status.success(),
        output: lines,
    })
}

fn run_claude_cli_update(tool: &Tool) -> Result<CommandResult> {
    let mut output = Command::new("claude")
        .args(["update", "--yes"])
        .output()
        .with_context(|| "failed to run claude update --yes")?;

    let mut lines = Vec::new();
    lines.push("> claude update --yes".to_string());
    lines.extend(normalize_output_stream(&output.stdout));
    lines.extend(normalize_output_stream(&output.stderr));

    let fallback_needed = !output.status.success()
        && lines.iter().any(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("unknown option")
                || lower.contains("unknown argument")
                || lower.contains("unrecognized option")
        });

    if fallback_needed {
        output = Command::new("claude")
            .args(["update"])
            .output()
            .with_context(|| "failed to run claude update")?;
        lines.push("> claude update".to_string());
        lines.extend(normalize_output_stream(&output.stdout));
        lines.extend(normalize_output_stream(&output.stderr));
    }

    Ok(CommandResult {
        title: format!("update {}", tool.display_name),
        success: output.status.success(),
        output: lines,
    })
}

fn skipped(tool: &Tool, reason: &str, mode: RunMode) -> CommandResult {
    let action = match mode {
        RunMode::Install => "install",
        RunMode::Update => "update",
        RunMode::Uninstall => "uninstall",
    };

    CommandResult {
        title: format!("{} {}", action, tool.display_name),
        success: true,
        output: vec![format!("Skipped {}: {}", tool.display_name, reason)],
    }
}

fn process_image_names(tool: &Tool) -> Vec<String> {
    let mut names = Vec::new();
    let command = tool.command.trim();
    if command.is_empty() {
        return names;
    }

    let image = if command.to_ascii_lowercase().ends_with(".exe") {
        command.to_string()
    } else {
        format!("{command}.exe")
    };
    names.push(image);

    let stem = command.trim_end_matches(".exe");
    if !stem.is_empty() {
        names.push(format!("{stem}*.exe"));
    }

    if let Some(first_word) = tool.display_name.split_whitespace().next() {
        let word = first_word.trim();
        if !word.is_empty() {
            names.push(format!("{word}.exe"));
            names.push(format!("{word}*.exe"));
        }
    }

    names.sort();
    names.dedup();
    names
}

fn stop_related_processes(tool: &Tool) {
    for image in process_image_names(tool) {
        if image.contains('*') || image.contains('?') {
            let filter = format!("IMAGENAME eq {image}");
            let _ = Command::new("taskkill")
                .args(["/FI", &filter, "/F", "/T"])
                .output();
        } else {
            let _ = Command::new("taskkill")
                .args(["/IM", &image, "/F", "/T"])
                .output();
        }
    }
}

fn has_access_denied(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.contains("access is denied")
            || line.contains("拒绝访问")
            || line.contains("Access is denied")
    })
}

pub fn install_tools(
    tools: &[&'static Tool],
    install_root: Option<&Path>,
) -> Result<Vec<CommandResult>> {
    run_many(RunMode::Install, tools, install_root)
}

pub fn update_tools(tools: &[&'static Tool]) -> Result<Vec<CommandResult>> {
    run_many(RunMode::Update, tools, None)
}

pub fn uninstall_tools(tools: &[&'static Tool]) -> Result<Vec<CommandResult>> {
    run_many(RunMode::Uninstall, tools, None)
}

fn run_many(
    mode: RunMode,
    tools: &[&'static Tool],
    install_root: Option<&Path>,
) -> Result<Vec<CommandResult>> {
    let mut results = Vec::new();
    let update_statuses = if matches!(mode, RunMode::Update) {
        let mut statuses = status::collect_statuses(install_root);
        status::populate_update_info(&mut statuses);
        Some(statuses)
    } else {
        None
    };

    for tool in tools {
        let status_info = update_statuses
            .as_ref()
            .and_then(|all| all.iter().find(|item| item.tool.key == tool.key));
        let installed = status_info
            .map(|item| item.path.is_some())
            .unwrap_or_else(|| status::command_path(tool.command).is_some());
        let update_needed = status_info
            .map(|item| item.update_available)
            .unwrap_or(installed);

        let maybe_result = match mode {
            RunMode::Install if installed => Some(skipped(tool, "already installed", mode)),
            RunMode::Update if installed && !update_needed => {
                Some(skipped(tool, "already up to date", mode))
            }
            RunMode::Update if !installed => Some(skipped(tool, "not installed", mode)),
            RunMode::Uninstall if !installed => Some(skipped(tool, "not installed", mode)),
            _ => None,
        };

        if let Some(result) = maybe_result {
            results.push(result);
            continue;
        }

        let action = match mode {
            RunMode::Install => "install",
            RunMode::Update => "upgrade",
            RunMode::Uninstall => "uninstall",
        };

        if matches!(mode, RunMode::Uninstall) {
            stop_related_processes(tool);
        }

        let mut result = if matches!(mode, RunMode::Update) && tool.key == "claude" {
            run_claude_cli_update(tool)?
        } else {
            run_winget(action, tool, install_root)?
        };
        if matches!(mode, RunMode::Uninstall)
            && !result.success
            && has_access_denied(&result.output)
        {
            result
                .output
                .push("Retrying uninstall after stopping related processes...".to_string());
            stop_related_processes(tool);
            thread::sleep(Duration::from_millis(300));
            let retry = run_winget(action, tool, install_root)?;
            result.success = retry.success;
            result.output.extend(retry.output);
            if !result.success && has_access_denied(&result.output) {
                result
                    .output
                    .push("Uninstall is still blocked by file lock or permissions.".to_string());
                result.output.push(
                    "Try closing all Claude windows/processes and rerun ai-setup as Administrator."
                        .to_string(),
                );
            }
        }
        results.push(result);
    }

    Ok(results)
}
