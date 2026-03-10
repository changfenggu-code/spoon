use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Action {
    Install,
    Update,
    Uninstall,
    Configure,
    Skills,
    Status,
}

#[derive(Debug, Parser)]
#[command(name = "ai-setup")]
#[command(about = "AI workstation setup CLI for Windows")]
pub struct Cli {
    #[arg(long)]
    pub install_path: Option<PathBuf>,

    #[arg(long)]
    pub no_log: bool,

    #[arg(long, value_enum)]
    pub action: Option<Action>,

    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,

    #[arg(long)]
    pub cleanup: bool,

    #[arg(long)]
    pub non_interactive: bool,
}
