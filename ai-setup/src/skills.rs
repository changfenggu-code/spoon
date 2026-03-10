use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub source: PathBuf,
}

pub fn available_skills(repo_root: &Path) -> Vec<Skill> {
    let mut skills = Vec::new();
    let local = repo_root.join("skills").join("ai-toolchain");
    if local.is_dir() {
        skills.push(Skill {
            name: "ai-toolchain".to_string(),
            source: local,
        });
    }

    let candidates = [
        (
            "claude-automation-recommender",
            dirs::home_dir()
                .unwrap_or_default()
                .join(".claude/plugins/marketplaces/claude-plugins-official/plugins/claude-code-setup/skills/claude-automation-recommender"),
        ),
        (
            "writing-hookify-rules",
            dirs::home_dir()
                .unwrap_or_default()
                .join(".claude/plugins/marketplaces/claude-plugins-official/plugins/hookify/skills/writing-rules"),
        ),
    ];

    for (name, path) in candidates {
        if path.is_dir() {
            skills.push(Skill {
                name: name.to_string(),
                source: path,
            });
        }
    }

    skills
}

pub fn install_skills(skills: &[Skill]) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    let target_root = dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("skills");
    fs::create_dir_all(&target_root)
        .with_context(|| format!("failed to create {}", target_root.display()))?;

    for skill in skills {
        let target = target_root.join(&skill.name);
        if target.exists() {
            fs::remove_dir_all(&target)
                .with_context(|| format!("failed to replace {}", target.display()))?;
        }
        copy_dir_recursive(&skill.source, &target)?;
        lines.push(format!(
            "Installed skill: {} -> {}",
            skill.name,
            target.display()
        ));
    }

    if lines.is_empty() {
        lines.push("No installable skills were found.".to_string());
    }

    Ok(lines)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), &destination)
                .with_context(|| format!("failed to copy to {}", destination.display()))?;
        }
    }
    Ok(())
}
