mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ── tokmd --version ──────────────────────────────────────────────────

#[test]
fn version_exits_successfully() {
    tokmd_cmd().arg("--version").assert().success();
}

#[test]
fn version_output_contains_version_number() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ── tokmd init ───────────────────────────────────────────────────────

#[test]
fn init_creates_tokeignore_file() {
    let tmp = tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("init")
        .arg("--non-interactive")
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(tmp.path().join(".tokeignore").exists());
}

#[test]
fn init_twice_shows_already_exists() {
    let tmp = tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();

    // First run creates the file
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("init")
        .arg("--non-interactive")
        .current_dir(tmp.path())
        .assert()
        .success();

    // Second run fails with "already exists" on stderr
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("init")
        .arg("--non-interactive")
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn init_output_mentions_tokeignore() {
    let tmp = tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();

    // init prints status to stderr; use --non-interactive to skip wizard
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("init")
        .arg("--non-interactive")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains(".tokeignore"));
}

// ── tokmd completions ────────────────────────────────────────────────

#[test]
fn completions_bash_produces_shell_script() {
    tokmd_cmd()
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_zsh_produces_shell_script() {
    tokmd_cmd()
        .arg("completions")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_fish_produces_output() {
    tokmd_cmd()
        .arg("completions")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_powershell_produces_output() {
    tokmd_cmd()
        .arg("completions")
        .arg("powershell")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

// ── tokmd check-ignore ──────────────────────────────────────────────

#[test]
fn check_ignore_with_nonexistent_path() {
    // check-ignore requires PATH argument; a missing file is still valid input
    tokmd_cmd()
        .arg("check-ignore")
        .arg("nonexistent_file.xyz")
        .assert()
        .stdout(predicate::str::contains("not ignored"));
}

#[test]
fn check_ignore_with_path_explains_status() {
    // Exit code 0 = ignored, 1 = not ignored; both are acceptable
    let assert = tokmd_cmd()
        .arg("check-ignore")
        .arg("src/main.rs")
        .assert();

    // Should produce output explaining the ignore status
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        stdout.contains("ignored") || stdout.contains("not ignored"),
        "expected ignore status in output, got: {stdout}"
    );
}

// ── tokmd lang (basic) ──────────────────────────────────────────────

#[test]
fn lang_produces_output() {
    tokmd_cmd()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn lang_json_produces_valid_json() {
    let output = tokmd_cmd()
        .arg("lang")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("--format json output should be valid JSON");
    assert!(json.is_object());
}

#[test]
fn lang_tsv_produces_tsv_output() {
    tokmd_cmd()
        .arg("lang")
        .arg("--format")
        .arg("tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("\t"));
}

#[test]
fn module_json_produces_valid_json() {
    let output = tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("--format json output should be valid JSON");
    assert!(json.is_object());
}
