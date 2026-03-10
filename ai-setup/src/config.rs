use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::{Map, Value};
use toml_edit::{DocumentMut, value};

#[derive(Debug, Clone, Default)]
pub struct GitConfig {
    pub user_name: String,
    pub user_email: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, Default)]
pub struct ClaudeConfig {
    pub base_url: String,
    pub auth_token: String,
}

#[derive(Debug, Clone, Default)]
pub struct CodexConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalConfig {
    pub editor: String,
    pub proxy: String,
}

pub fn load_git_config() -> GitConfig {
    GitConfig {
        user_name: git_config_get("user.name").unwrap_or_default(),
        user_email: git_config_get("user.email").unwrap_or_default(),
        default_branch: git_config_get("init.defaultBranch").unwrap_or_else(|| "main".to_string()),
    }
}

pub fn save_git_config(config: &GitConfig) -> Result<()> {
    git_config_set("init.defaultBranch", &config.default_branch)?;
    if config.user_name.trim().is_empty() {
        git_config_unset("user.name")?;
    } else {
        git_config_set("user.name", &config.user_name)?;
    }
    if config.user_email.trim().is_empty() {
        git_config_unset("user.email")?;
    } else {
        git_config_set("user.email", &config.user_email)?;
    }
    Ok(())
}

pub fn load_claude_config() -> ClaudeConfig {
    let mut cfg = ClaudeConfig::default();
    let settings_path = claude_settings_path();
    if let Ok(content) = fs::read_to_string(&settings_path)
        && let Ok(json) = serde_json::from_str::<Value>(&content)
        && let Some(env) = json.get("env")
    {
        cfg.base_url = env
            .get("ANTHROPIC_BASE_URL")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        cfg.auth_token = env
            .get("ANTHROPIC_AUTH_TOKEN")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }

    if cfg.base_url.is_empty() {
        cfg.base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
    }
    if cfg.auth_token.is_empty() {
        cfg.auth_token = std::env::var("ANTHROPIC_AUTH_TOKEN").unwrap_or_default();
    }
    cfg
}

pub fn save_claude_config(config: &ClaudeConfig) -> Result<()> {
    let path = claude_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut root = if path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(&path).unwrap_or_default())
            .unwrap_or_else(|_| Value::Object(Map::new()))
    } else {
        Value::Object(Map::new())
    };

    if !root.is_object() {
        root = Value::Object(Map::new());
    }
    let obj = root
        .as_object_mut()
        .context("expected JSON object in Claude settings")?;
    if !obj.contains_key("env") || !obj.get("env").is_some_and(Value::is_object) {
        obj.insert("env".into(), Value::Object(Map::new()));
    }
    let env = obj
        .get_mut("env")
        .and_then(Value::as_object_mut)
        .context("expected env object in Claude settings")?;
    env.insert(
        "ANTHROPIC_BASE_URL".into(),
        Value::String(config.base_url.clone()),
    );
    env.insert(
        "ANTHROPIC_AUTH_TOKEN".into(),
        Value::String(config.auth_token.clone()),
    );

    fs::write(&path, serde_json::to_string_pretty(&root)?)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load_codex_config(default_model: &str) -> CodexConfig {
    let mut cfg = CodexConfig {
        base_url: "https://api.openai.com".to_string(),
        model: default_model.to_string(),
        ..Default::default()
    };

    let config_path = codex_config_path();
    if let Ok(content) = fs::read_to_string(&config_path)
        && let Ok(doc) = content.parse::<DocumentMut>()
    {
        if let Some(model) = doc.get("model").and_then(|item| item.as_str()) {
            cfg.model = model.to_string();
        }
        if let Some(section) = doc
            .get("model_providers")
            .and_then(|item| item.get("OpenAI"))
            && let Some(base_url) = section.get("base_url").and_then(|item| item.as_str())
        {
            cfg.base_url = base_url.to_string();
        }
    }

    let auth_path = codex_auth_path();
    if let Ok(content) = fs::read_to_string(&auth_path)
        && let Ok(json) = serde_json::from_str::<Value>(&content)
    {
        cfg.api_key = json
            .get("OPENAI_API_KEY")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }

    if cfg.api_key.is_empty() {
        cfg.api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    }
    if cfg.base_url == "https://api.openai.com" {
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL")
            && !base_url.trim().is_empty()
        {
            cfg.base_url = base_url;
        }
    }
    cfg
}

pub fn save_codex_config(config: &CodexConfig) -> Result<()> {
    let config_path = codex_config_path();
    let auth_path = codex_auth_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut doc = DocumentMut::new();
    doc["model_provider"] = value("OpenAI");
    doc["model"] = value(config.model.clone());
    doc["review_model"] = value(config.model.clone());
    doc["model_reasoning_effort"] = value("medium");
    doc["disable_response_storage"] = value(true);
    doc["network_access"] = value("enabled");
    doc["model_context_window"] = value(400000);
    doc["model_auto_compact_token_limit"] = value(300000);
    doc["model_providers"]["OpenAI"]["name"] = value("OpenAI");
    doc["model_providers"]["OpenAI"]["base_url"] = value(config.base_url.clone());
    doc["model_providers"]["OpenAI"]["wire_api"] = value("responses");

    fs::write(&config_path, doc.to_string())
        .with_context(|| format!("failed to write {}", config_path.display()))?;
    let auth_json = serde_json::json!({ "OPENAI_API_KEY": config.api_key });
    fs::write(&auth_path, serde_json::to_string_pretty(&auth_json)?)
        .with_context(|| format!("failed to write {}", auth_path.display()))?;
    Ok(())
}

pub fn load_global_config() -> GlobalConfig {
    let mut cfg = GlobalConfig::default();
    let path = global_config_path();
    if let Ok(content) = fs::read_to_string(&path)
        && let Ok(doc) = content.parse::<DocumentMut>()
    {
        if let Some(editor) = doc.get("editor").and_then(|item| item.as_str()) {
            cfg.editor = editor.to_string();
        }
        if let Some(proxy) = doc.get("proxy").and_then(|item| item.as_str()) {
            cfg.proxy = proxy.to_string();
        }
    }
    cfg
}

pub fn save_global_config(config: &GlobalConfig) -> Result<()> {
    let path = global_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut doc = DocumentMut::new();
    doc["editor"] = value(config.editor.clone());
    doc["proxy"] = value(config.proxy.clone());
    fs::write(&path, doc.to_string())
        .with_context(|| format!("failed to write {}", path.display()))?;

    apply_global_proxy_env(config.proxy.trim())?;
    Ok(())
}

pub fn apply_global_proxy_env(proxy: &str) -> Result<()> {
    let proxy_value = if proxy.is_empty() { "" } else { proxy };
    for key in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"] {
        set_user_env_var(key, proxy_value)?;
    }
    Ok(())
}

fn set_user_env_var(name: &str, value: &str) -> Result<()> {
    let script = r#"$name = $args[0];
$value = $args[1];
if ([string]::IsNullOrWhiteSpace($value)) {
    [Environment]::SetEnvironmentVariable($name, $null, "User")
} else {
    [Environment]::SetEnvironmentVariable($name, $value, "User")
}"#;
    Command::new("powershell")
        .args(["-NoProfile", "-Command", script, name, value])
        .status()
        .with_context(|| format!("failed to set user environment variable {name}"))?;
    Ok(())
}

fn git_config_get(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn git_config_set(key: &str, value: &str) -> Result<()> {
    Command::new("git")
        .args(["config", "--global", key, value])
        .status()
        .with_context(|| format!("failed to set git config {key}"))?;
    Ok(())
}

fn git_config_unset(key: &str) -> Result<()> {
    let _ = Command::new("git")
        .args(["config", "--global", "--unset", key])
        .status();
    Ok(())
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn git_config_path() -> PathBuf {
    home_dir().join(".gitconfig")
}

pub fn claude_settings_path() -> PathBuf {
    home_dir().join(".claude").join("settings.json")
}

pub fn codex_config_path() -> PathBuf {
    home_dir().join(".codex").join("config.toml")
}

pub fn codex_auth_path() -> PathBuf {
    home_dir().join(".codex").join("auth.json")
}

pub fn global_config_path() -> PathBuf {
    home_dir().join(".ai-setup").join("config.toml")
}
