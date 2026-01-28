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

#[test]
fn test_run_default_output_creates_local_runs_dir() {
    let dir = tempdir().unwrap();
    let work_dir = dir.path().join("workdir");
    fs::create_dir_all(&work_dir).unwrap();

    // Create a minimal source file to scan
    let src_dir = work_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "fn main() {}\n").unwrap();

    // Run without --output-dir (should use .runs/tokmd/runs/<run-id>)
    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.current_dir(&work_dir)
        .arg("run")
        .arg("--name")
        .arg("test-run")
        .arg(".")
        .assert()
        .success();

    // Verify .runs/tokmd/runs/test-run directory was created
    let expected_dir = work_dir.join(".runs/tokmd/runs/test-run");
    assert!(
        expected_dir.exists(),
        ".runs/tokmd/runs/test-run directory should be created at {:?}",
        expected_dir
    );
    assert!(
        expected_dir.join("receipt.json").exists(),
        "receipt.json should exist in default location"
    );
    assert!(
        expected_dir.join("lang.json").exists(),
        "lang.json should exist in default location"
    );
    assert!(
        expected_dir.join("export.jsonl").exists(),
        "export.jsonl should exist in default location"
    );
}

#[test]
fn test_run_with_redact_flag() {
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redacted");

    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");

    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.current_dir(&fixtures)
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg(".")
        .assert()
        .success();

    // Check that export.jsonl was created and contains redacted paths
    let export_content = fs::read_to_string(output_dir.join("export.jsonl")).unwrap();

    // The meta line should contain "redact": "paths"
    assert!(
        export_content.contains(r#""redact":"paths""#),
        "export.jsonl should indicate redact mode is 'paths'"
    );

    // Paths should be hashed (16 hex chars followed by extension)
    // Check that we don't have the original .rs extension preceded by a recognizable path
    let lines: Vec<&str> = export_content.lines().collect();
    for line in lines.iter().skip(1) {
        // Skip meta line
        if line.contains(r#""type":"row""#) {
            // Paths should be hashed - they should be 16 hex chars followed by extension
            assert!(
                !line.contains("src/") && !line.contains("src\\"),
                "Redacted export should not contain original path segments"
            );
        }
    }
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
