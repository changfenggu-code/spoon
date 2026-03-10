use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("xtask error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_string());

    match command.as_str() {
        "deploy" => deploy(),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown xtask command: {other}"),
        )),
    }
}

fn deploy() -> io::Result<()> {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = project_dir.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to locate repository root from CARGO_MANIFEST_DIR",
        )
    })?;

    run_checked(
        "cargo",
        &["build", "--release", "--bin", "ai-setup"],
        &project_dir,
    )?;

    let source = project_dir
        .join("target")
        .join("release")
        .join("ai-setup.exe");
    let dest = repo_root.join("ai-setup.exe");

    replace_in_place(&source, &dest)?;
    print_metadata(&dest)?;
    Ok(())
}

fn replace_in_place(source: &Path, dest: &Path) -> io::Result<()> {
    match fs::copy(source, dest) {
        Ok(_) => Ok(()),
        Err(err) if is_lock_error(&err) => {
            // Artifact update rule: if locked, stop running ai-setup process first.
            stop_ai_setup_processes()?;
            fs::copy(source, dest).map(|_| ())
        }
        Err(err) => Err(err),
    }
}

fn stop_ai_setup_processes() -> io::Result<()> {
    // taskkill returns non-zero when no process matches; treat as best-effort.
    let status = Command::new("taskkill")
        .args(["/IM", "ai-setup.exe", "/F"])
        .status()?;
    if status.success() {
        return Ok(());
    }
    Ok(())
}

fn is_lock_error(err: &io::Error) -> bool {
    matches!(err.raw_os_error(), Some(5) | Some(32))
}

fn run_checked(program: &str, args: &[&str], cwd: &Path) -> io::Result<()> {
    let status = Command::new(program).args(args).current_dir(cwd).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{program} {:?} failed with status {status}",
            args
        )))
    }
}

fn print_metadata(path: &Path) -> io::Result<()> {
    let meta = fs::metadata(path)?;
    let modified = meta.modified().ok();
    println!("deployed: {}", path.display());
    println!("size: {}", meta.len());
    if let Some(modified) = modified {
        println!("modified: {:?}", modified);
    }
    Ok(())
}

fn print_help() {
    println!("xtask commands:");
    println!("  deploy    Build release ai-setup and replace repository-root ai-setup.exe");
}
