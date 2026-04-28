//! Wave-76 CLI integration tests for `tokmd init` and `tokmd check-ignore`.
//!
//! Focuses on output messages, template switching, error paths,
//! and check-ignore behaviour with temp-dir fixtures.

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
// init: output messages
// ---------------------------------------------------------------------------

#[test]
fn init_stderr_contains_initialized_message() {
    let dir = tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Initialized"));
}

#[test]
fn init_default_stderr_contains_template_hint() {
    let dir = tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success()
        .stderr(predicate::str::contains("--template"));
}

#[test]
fn init_non_default_template_omits_hint() {
    let dir = tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["init", "--non-interactive", "--template", "rust"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Hint").not());
}

#[test]
fn init_stderr_contains_ready_message() {
    let dir = tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Ready!"));
}

// ---------------------------------------------------------------------------
// init: template switching on existing file
// ---------------------------------------------------------------------------

#[test]
fn init_force_switches_template_on_existing_file() {
    let dir = tempdir().unwrap();
    // Create with Rust template
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["init", "--non-interactive", "--template", "rust"])
        .assert()
        .success();
    let c1 = std::fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(c1.contains("Rust"));

    // Overwrite with Python template
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "init",
            "--non-interactive",
            "--force",
            "--template",
            "python",
        ])
        .assert()
        .success();
    let c2 = std::fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(c2.contains("Python"));
    assert!(!c2.contains("Rust"));
}

// ---------------------------------------------------------------------------
// init: --print with each template
// ---------------------------------------------------------------------------

#[test]
fn init_print_rust_contains_target() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "rust", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target/"));
}

#[test]
fn init_print_node_contains_node_modules() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "node", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("node_modules/"));
}

// ---------------------------------------------------------------------------
// init: error paths
// ---------------------------------------------------------------------------

#[test]
fn init_into_nonexistent_dir_fails_gracefully() {
    let dir = tempdir().unwrap();
    let bad_path = dir.path().join("does_not_exist");
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .args(["init", "--non-interactive", "--dir"])
        .arg(&bad_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn init_refuses_overwrite_with_error_message() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".tokeignore"), "# existing\n").unwrap();
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

// ---------------------------------------------------------------------------
// check-ignore: behaviour with temp fixtures
// ---------------------------------------------------------------------------

#[test]
fn check_ignore_nonexistent_path_reports_not_ignored() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("no_such_file.rs");
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("check-ignore")
        .arg(&missing)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn check_ignore_verbose_shows_detail() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("phantom.txt");
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["check-ignore", "--verbose"])
        .arg(&missing)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("does not exist"));
}
