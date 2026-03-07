use serde_json::Value;
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    let hook_input = read_hook_input();
    let project_root = project_root();

    match parse_file_path(&hook_input) {
        Some(file_path) if file_path.ends_with(".rs") => {
            rustfmt_file(&project_root, &file_path);
        }
        // Non-Rust file or no file path — silent no-op.
        Some(_) | None => {}
    }
}

fn read_hook_input() -> String {
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);
    input
}

fn parse_file_path(input: &str) -> Option<String> {
    serde_json::from_str::<Value>(input).ok().and_then(|value| {
        value
            .pointer("/tool_input/file_path")
            .and_then(Value::as_str)
            .map(ToString::to_string)
    })
}

fn project_root() -> PathBuf {
    env::var("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .or_else(|_| detect_git_root())
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn detect_git_root() -> Result<PathBuf, std::io::Error> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other("failed to discover git root"));
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        return Err(std::io::Error::other("git root was empty"));
    }
    Ok(PathBuf::from(root))
}

fn rustfmt_file(project_root: &Path, file_path: &str) {
    let path = Path::new(file_path);
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    };

    let mut command = Command::new("rustfmt");
    command.arg(absolute).current_dir(project_root);
    run_silent(command);
}

fn run_silent(mut command: Command) {
    let _ = command.stdout(Stdio::null()).stderr(Stdio::null()).status();
}
