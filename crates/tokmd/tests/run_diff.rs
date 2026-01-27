use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use std::process::Command as ProcessCommand;

#[test]
fn test_run_generates_artifacts() {
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run1");

    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");

    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.current_dir(&fixtures) // Run on test data so it's small and predictable
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg(".") // path to scan
        .assert()
        .success();

    assert!(
        output_dir.join("receipt.json").exists(),
        "receipt.json missing"
    );
    assert!(output_dir.join("lang.json").exists(), "lang.json missing");
    assert!(
        output_dir.join("module.json").exists(),
        "module.json missing"
    );
    assert!(
        output_dir.join("export.jsonl").exists(),
        "export.jsonl missing"
    );

    // Check content of receipt.json
    let receipt_content = fs::read_to_string(output_dir.join("receipt.json")).unwrap();
    assert!(receipt_content.contains("lang.json"));
    assert!(receipt_content.contains("schema_version"));
}

#[test]
fn test_diff_identical_runs() {
    let dir = tempdir().unwrap();
    let run1_dir = dir.path().join("run1");
    let run2_dir = dir.path().join("run2");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");

    // Run 1
    let mut cmd1 = Command::cargo_bin("tokmd").unwrap();
    cmd1.current_dir(&fixtures)
        .arg("run")
        .arg("--output-dir")
        .arg(run1_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    // Run 2 (same data)
    let mut cmd2 = Command::cargo_bin("tokmd").unwrap();
    cmd2.current_dir(&fixtures)
        .arg("run")
        .arg("--output-dir")
        .arg(run2_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    // Diff
    let mut cmd_diff = Command::cargo_bin("tokmd").unwrap();
    cmd_diff
        .arg("diff")
        .arg("--from")
        .arg(run1_dir.join("receipt.json").to_str().unwrap())
        .arg("--to")
        .arg(run2_dir.join("receipt.json").to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Diffing Language Summaries"));
    // Should produce empty diff table (header only) because counts are identical
    // But headers are always printed.
}

fn git_available() -> bool {
    ProcessCommand::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn git_cmd(dir: &std::path::Path, args: &[&str]) {
    let status = ProcessCommand::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git command failed to run");
    assert!(status.success(), "git command failed");
}

#[test]
fn test_diff_git_refs() {
    if !git_available() {
        return;
    }

    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    git_cmd(&repo, &["init"]);
    git_cmd(&repo, &["config", "user.email", "test@example.com"]);
    git_cmd(&repo, &["config", "user.name", "Tokmd Test"]);

    let src_dir = repo.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "fn a() {}\n").unwrap();
    git_cmd(&repo, &["add", "."]);
    git_cmd(&repo, &["commit", "-m", "initial"]);

    fs::write(src_dir.join("lib.rs"), "fn a() {}\nfn b() {}\n").unwrap();
    git_cmd(&repo, &["add", "."]);
    git_cmd(&repo, &["commit", "-m", "add b"]);

    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.current_dir(&repo)
        .arg("diff")
        .arg("HEAD~1")
        .arg("HEAD")
        .assert()
        .success()
        .stdout(predicate::str::contains("Diffing Language Summaries"))
        .stdout(predicate::str::contains("Rust"));
}
