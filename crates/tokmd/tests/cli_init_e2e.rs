//! End-to-end tests for `tokmd init` — file creation, templates, overwrite
//! protection, and `--dir` flag.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// File creation
// ---------------------------------------------------------------------------

#[test]
fn init_creates_tokeignore_in_temp_dir() {
    let dir = tempdir().expect("Failed to create temporary directory for test");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success();

    assert!(
        dir.path().join(".tokeignore").exists(),
        ".tokeignore should be created"
    );
    let content =
        std::fs::read_to_string(dir.path().join(".tokeignore")).expect("Failed to read test file");
    assert!(!content.is_empty(), ".tokeignore should not be empty");
}

#[test]
fn init_with_dir_flag_creates_tokeignore_in_target() {
    let dir = tempdir().expect("Failed to create temporary directory for test");
    let target = dir.path().join("project");
    std::fs::create_dir_all(&target).expect("Failed to create test directory");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.args(["init", "--non-interactive", "--dir"])
        .arg(&target)
        .assert()
        .success();

    assert!(
        target.join(".tokeignore").exists(),
        ".tokeignore should be created in --dir target"
    );
}

// ---------------------------------------------------------------------------
// Overwrite protection and --force
// ---------------------------------------------------------------------------

#[test]
fn init_does_not_overwrite_existing_tokeignore() {
    let dir = tempdir().expect("Failed to create temporary directory for test");
    let existing = dir.path().join(".tokeignore");
    std::fs::write(&existing, "# my custom rules\n").expect("Failed to write to test file");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .failure();

    let content = std::fs::read_to_string(&existing).expect("Failed to read test file");
    assert_eq!(content, "# my custom rules\n", "file should be untouched");
}

#[test]
fn init_force_overwrites_existing_tokeignore() {
    let dir = tempdir().expect("Failed to create temporary directory for test");
    let existing = dir.path().join(".tokeignore");
    std::fs::write(&existing, "# old rules\n").expect("Failed to write to test file");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive", "--force"])
        .assert()
        .success();

    let content = std::fs::read_to_string(&existing).expect("Failed to read test file");
    assert_ne!(content, "# old rules\n", "file should be overwritten");
    assert!(!content.is_empty());
}

// ---------------------------------------------------------------------------
// --print flag (stdout, no file created)
// ---------------------------------------------------------------------------

#[test]
fn init_print_outputs_template_to_stdout() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn init_print_does_not_create_file() {
    let dir = tempdir().expect("Failed to create temporary directory for test");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success();

    assert!(
        !dir.path().join(".tokeignore").exists(),
        "--print should not create .tokeignore"
    );
}

// ---------------------------------------------------------------------------
// Template profiles
// ---------------------------------------------------------------------------

#[test]
fn init_print_python_template_contains_venv() {
    tokmd_cmd()
        .args([
            "init",
            "--print",
            "--template",
            "python",
            "--non-interactive",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("venv").or(predicate::str::contains("__pycache__")));
}

#[test]
fn init_print_go_template_contains_vendor() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "go", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vendor"));
}

#[test]
fn init_print_cpp_template_produces_output() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "cpp", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn init_print_mono_template_produces_output() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "mono", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
