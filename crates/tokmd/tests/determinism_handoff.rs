use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    Command::cargo_bin("tokmd").expect("binary must build")
}

#[test]
fn handoff_deterministic_ordering_same_code_lines() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // We want > 10 files to ensure truncate hits the limit or at least enough
    // files to see determinism issues. Let's make 25.
    for i in 0..25 {
        let path = root.join(format!("file_{:02}.rs", i));
        fs::write(path, "fn main() { println!(\"hello\"); }\n").unwrap();
    }

    // Create a mock .git dir so it thinks it's a git repo if handoff requires it
    fs::create_dir_all(root.join(".git")).unwrap();

    // Add an initial commit so git history works
    let _ = std::process::Command::new("git")
        .arg("init")
        .current_dir(root)
        .output()
        .unwrap();

    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(root)
        .output()
        .unwrap();

    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(root)
        .output()
        .unwrap();

    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(root)
        .output()
        .unwrap();

    let output1 = tokmd_cmd()
        .current_dir(root)
        .args(["handoff", "--json"])
        .output()
        .expect("failed to execute process");

    let stdout1 = String::from_utf8(output1.stdout).unwrap();

    let output2 = tokmd_cmd()
        .current_dir(root)
        .args(["handoff", "--json"])
        .output()
        .expect("failed to execute process");

    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    assert_eq!(
        stdout1, stdout2,
        "handoff command output should be deterministic"
    );
}
