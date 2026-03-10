use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::tool::{self, Tool};

#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub tool: &'static Tool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub expected_dir: Option<PathBuf>,
}

pub fn refresh_process_env_from_registry() -> Result<()> {
    let script = r#"
$machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($userPath -and $machinePath) {
  Write-Output "$userPath;$machinePath"
} elseif ($userPath) {
  Write-Output $userPath
} elseif ($machinePath) {
  Write-Output $machinePath
}
"#;

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", script])
        .output()
        .context("failed to refresh PATH from registry")?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            unsafe {
                env::set_var("PATH", path);
            }
        }
    }

    Ok(())
}

pub fn command_path(command: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let pathext = env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".EXE;.CMD;.BAT;.COM"));
    let exts: Vec<String> = pathext
        .to_string_lossy()
        .split(';')
        .map(|item| item.trim().to_ascii_lowercase())
        .collect();

    let has_ext = Path::new(command).extension().is_some();
    for dir in env::split_paths(&path_var) {
        if has_ext {
            let candidate = dir.join(command);
            if candidate.exists() {
                return Some(candidate);
            }
        } else {
            for ext in &exts {
                let candidate = dir.join(format!("{command}{ext}"));
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

pub fn command_version(command: &str) -> Option<String> {
    let path = command_path(command)?;
    let output = Command::new(path).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    if line.is_empty() { None } else { Some(line) }
}

pub fn collect_statuses(install_root: Option<&Path>) -> Vec<ToolStatus> {
    tool::all_tools()
        .into_iter()
        .map(|tool| ToolStatus {
            tool,
            path: command_path(tool.command),
            version: command_version(tool.command),
            latest_version: None,
            update_available: false,
            expected_dir: tool::expected_tool_dir(install_root, tool),
        })
        .collect()
}

fn normalize_lines(raw: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(raw)
        .replace('\r', "\n")
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn extract_version_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        let is_token_char = ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-');
        if is_token_char {
            current.push(ch);
        } else if !current.is_empty() {
            let has_digit = current.chars().any(|c| c.is_ascii_digit());
            if has_digit && current.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        let has_digit = current.chars().any(|c| c.is_ascii_digit());
        if has_digit && current.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            tokens.push(current);
        }
    }
    tokens
}

fn first_version_token(text: &str) -> Option<String> {
    extract_version_tokens(text).into_iter().next()
}

fn winget_upgrade_versions(winget_id: &str) -> Option<(String, String)> {
    let output = Command::new("winget")
        .args([
            "upgrade",
            "--id",
            winget_id,
            "-e",
            "--disable-interactivity",
            "--accept-source-agreements",
        ])
        .output()
        .ok()?;

    let mut lines = normalize_lines(&output.stdout);
    lines.extend(normalize_lines(&output.stderr));

    for line in lines {
        if !line.contains(winget_id) {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(idx) = parts.iter().position(|part| *part == winget_id) {
            if idx + 2 < parts.len() {
                let current = parts[idx + 1].to_string();
                let latest = parts[idx + 2].to_string();
                if !current.is_empty() && !latest.is_empty() && current != latest {
                    return Some((current, latest));
                }
            }
        }
    }

    None
}

fn claude_latest_version(current_version: Option<&str>) -> Option<String> {
    let output = Command::new("claude")
        .args(["update", "--check"])
        .output()
        .ok()?;

    let mut lines = normalize_lines(&output.stdout);
    lines.extend(normalize_lines(&output.stderr));
    let joined = lines.join("\n").to_ascii_lowercase();
    if joined.contains("up to date") || joined.contains("already latest") {
        return None;
    }

    let mut tokens = Vec::new();
    for line in &lines {
        tokens.extend(extract_version_tokens(line));
    }
    tokens.dedup();

    if let Some(current) = current_version
        && let Some(last) = tokens.last()
        && last != current
    {
        return Some(last.clone());
    }

    if tokens.len() >= 2 && tokens.first() != tokens.last() {
        return tokens.last().cloned();
    }

    None
}

pub fn populate_update_info(statuses: &mut [ToolStatus]) {
    for status in statuses.iter_mut() {
        status.latest_version = None;
        status.update_available = false;
        if status.path.is_none() {
            continue;
        }

        if status.tool.key == "claude" {
            let current = status.version.as_deref().and_then(first_version_token);
            if let Some(latest) = claude_latest_version(current.as_deref()) {
                status.latest_version = Some(latest);
                status.update_available = true;
            }
            continue;
        }

        if let Some((_current, latest)) = winget_upgrade_versions(status.tool.winget_id) {
            status.latest_version = Some(latest);
            status.update_available = true;
        }
    }
}

pub fn status_lines(install_root: Option<&Path>) -> Vec<String> {
    let mut lines = vec![
        "Current status:".to_string(),
        "  Managed tools: Git, GitHub CLI, Claude Code, Codex".to_string(),
        "  Optional helper bundle: rg, fd, jq, bat, delta, sg, yq, python, which, make, 7zip, just".to_string(),
        "  Dependency: Claude Code requires Git".to_string(),
    ];

    let mut statuses = collect_statuses(install_root);
    populate_update_info(&mut statuses);
    let outdated_count = statuses.iter().filter(|status| status.update_available).count();
    lines.push(format!("  Updates available: {}", outdated_count));

    if let Some(root) = install_root {
        lines.push(format!("  Target root: {}", root.display()));
        for status in &statuses {
            if let (Some(current), Some(expected)) = (&status.path, &status.expected_dir) {
                let current_dir = current
                    .parent()
                    .and_then(|p| p.parent())
                    .unwrap_or(current.as_path());
                if !current_dir.starts_with(expected) {
                    lines.push(format!(
                        "  Path mismatch: {} -> current={} expected={}",
                        status.tool.display_name,
                        current.display(),
                        expected.display()
                    ));
                    lines.push(format!(
                        "    Suggested fix: ai-setup --action uninstall --tools {} && ai-setup --action install --tools {} --install-path <desired_path>",
                        status.tool.key,
                        status.tool.key
                    ));
                }
            }
        }
    }

    for status in statuses {
        match (&status.path, &status.version, &status.latest_version) {
            (Some(path), Some(version), Some(latest)) if status.update_available => {
                lines.push(format!(
                    "  {}: Installed ({}) -> Latest ({}) [update available]",
                    status.tool.display_name, version, latest
                ));
                lines.push(format!("    path: {}", path.display()));
            }
            (Some(path), Some(version), _) => {
                lines.push(format!(
                    "  {}: Installed ({})",
                    status.tool.display_name, version
                ));
                lines.push(format!("    path: {}", path.display()));
            }
            (Some(path), None, Some(latest)) if status.update_available => {
                lines.push(format!(
                    "  {}: Installed -> Latest ({}) [update available]",
                    status.tool.display_name, latest
                ));
                lines.push(format!("    path: {}", path.display()));
            }
            (Some(path), None, _) => {
                lines.push(format!("  {}: Installed", status.tool.display_name));
                lines.push(format!("    path: {}", path.display()));
            }
            (None, _, _) => lines.push(format!("  {}: Not installed", status.tool.display_name)),
        }
    }

    lines
}

pub fn print_status(install_root: Option<&Path>) {
    for line in status_lines(install_root) {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_version_tokens_git() {
        assert_eq!(
            extract_version_tokens("git version 2.43.0"),
            vec!["2.43.0"]
        );
    }

    #[test]
    fn extract_version_tokens_python() {
        assert_eq!(
            extract_version_tokens("Python 3.13.1"),
            vec!["3.13.1"]
        );
    }

    #[test]
    fn extract_version_tokens_ripgrep() {
        assert_eq!(
            extract_version_tokens("ripgrep 14.1.1 (rev abcdef)"),
            vec!["14.1.1"]
        );
    }

    #[test]
    fn extract_version_tokens_no_version() {
        assert!(extract_version_tokens("no version here").is_empty());
    }

    #[test]
    fn extract_version_tokens_multiple() {
        let tokens = extract_version_tokens("tool 1.2.3 -> 1.3.0");
        assert_eq!(tokens, vec!["1.2.3", "1.3.0"]);
    }

    #[test]
    fn first_version_token_extracts_first() {
        assert_eq!(
            first_version_token("ripgrep 14.1.1"),
            Some("14.1.1".to_string())
        );
    }

    #[test]
    fn first_version_token_none_for_no_version() {
        assert_eq!(first_version_token("no version"), None);
    }

    #[test]
    fn normalize_lines_handles_crlf() {
        let input = b"line1\r\nline2\r\n";
        let result = normalize_lines(input);
        assert_eq!(result, vec!["line1", "line2"]);
    }

    #[test]
    fn normalize_lines_skips_empty() {
        let input = b"line1\n\n\nline2\n";
        let result = normalize_lines(input);
        assert_eq!(result, vec!["line1", "line2"]);
    }
}
