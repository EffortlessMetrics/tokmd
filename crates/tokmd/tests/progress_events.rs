//! Integration tests for machine-readable progress events on the scan-summary
//! command family (`lang`, `module`, `export`) and the multi-step orchestrators
//! (`cockpit`, `packet generate`).
//!
//! See `docs/specs/progress-events.md` for the behavior contract. These tests
//! assert that, when `TOKMD_PROGRESS_EVENTS` is set, each command emits
//! `tokmd.progress` events to **stderr** while keeping **stdout** free of
//! progress noise, and that the events are absent when the variable is unset.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const UPDATE_LINE: &str = r#"{"event":"tokmd.progress","kind":"update","message":"Scanning codebase...","schema_version":1}"#;
const FINISH_LINE: &str =
    r#"{"event":"tokmd.progress","kind":"finish","message":"done","schema_version":1}"#;

fn fixture_dir() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    std::fs::write(tmp.path().join("main.rs"), "fn main() {\n    // body\n}\n")?;
    Ok(tmp)
}

fn tokmd_in(dir: &std::path::Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir)
        .env_remove("TOKMD_PROFILE")
        .env_remove("TOKMD_CONFIG")
        .env_remove("TOKMD_NO_PROGRESS");
    cmd
}

/// The progress contract is identical across the scan-summary family, so assert
/// it uniformly: events on stderr, no progress noise on stdout.
fn assert_emits_progress(subcommand: &str) -> TestResult {
    let tmp = fixture_dir()?;
    tokmd_in(tmp.path())
        .arg(subcommand)
        .arg("--no-progress")
        .arg(".")
        .env("TOKMD_PROGRESS_EVENTS", "1")
        .assert()
        .success()
        .stderr(predicate::str::contains(UPDATE_LINE))
        .stderr(predicate::str::contains(FINISH_LINE))
        .stdout(predicate::str::contains("tokmd.progress").not());
    Ok(())
}

#[test]
fn lang_emits_progress_events_on_stderr() -> TestResult {
    assert_emits_progress("lang")
}

#[test]
fn module_emits_progress_events_on_stderr() -> TestResult {
    assert_emits_progress("module")
}

#[test]
fn export_emits_progress_events_on_stderr() -> TestResult {
    assert_emits_progress("export")
}

#[test]
fn lang_emits_no_progress_events_when_unset() -> TestResult {
    let tmp = fixture_dir()?;
    tokmd_in(tmp.path())
        .arg("lang")
        .arg(".")
        .env_remove("TOKMD_PROGRESS_EVENTS")
        .assert()
        .success()
        .stderr(predicate::str::contains("tokmd.progress").not());
    Ok(())
}

#[test]
fn export_emits_no_progress_events_when_unset() -> TestResult {
    let tmp = fixture_dir()?;
    tokmd_in(tmp.path())
        .arg("export")
        .arg(".")
        .env_remove("TOKMD_PROGRESS_EVENTS")
        .assert()
        .success()
        .stderr(predicate::str::contains("tokmd.progress").not());
    Ok(())
}

// ---------------------------------------------------------------------------
// Multi-step orchestrators: `cockpit` and `packet generate`.
//
// These commands delegate or sequence several steps. They must frame their
// orchestrator-level stages as `tokmd.progress` events on stderr (when
// TOKMD_PROGRESS_EVENTS is set) while keeping the machine-readable receipt /
// manifest on stdout. They are git-backed, so each gates on git availability.
// ---------------------------------------------------------------------------

/// Build a two-branch git repo (main + feature with one added file) so cockpit
/// has a real diff to analyze. Returns `None` when git is unavailable so the
/// caller can skip gracefully.
#[cfg(feature = "git")]
fn cockpit_repo() -> Result<Option<tempfile::TempDir>, Box<dyn std::error::Error>> {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return Ok(None);
    }
    let dir = tempfile::tempdir()?;
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return Ok(None);
    }
    std::fs::write(dir.path().join("lib.rs"), "fn lib() {}\n")?;
    if !common::git_add_commit(dir.path(), "initial") {
        eprintln!("Skipping: initial commit failed");
        return Ok(None);
    }
    let checked_out = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !checked_out {
        eprintln!("Skipping: feature branch checkout failed");
        return Ok(None);
    }
    std::fs::write(dir.path().join("new.rs"), "fn new() {}\n")?;
    if !common::git_add_commit(dir.path(), "add new file") {
        eprintln!("Skipping: second commit failed");
        return Ok(None);
    }
    Ok(Some(dir))
}

#[cfg(feature = "git")]
#[test]
fn cockpit_emits_progress_events_on_stderr() -> TestResult {
    let Some(dir) = cockpit_repo()? else {
        return Ok(());
    };
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "cockpit",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--no-progress",
        ])
        .env("TOKMD_PROGRESS_EVENTS", "1")
        .env_remove("TOKMD_NO_PROGRESS")
        .assert()
        .success()
        .stderr(predicate::str::contains(r#""event":"tokmd.progress""#))
        .stderr(predicate::str::contains("Computing cockpit metrics"))
        .stderr(predicate::str::contains(FINISH_LINE))
        .stdout(predicate::str::contains("tokmd.progress").not());
    Ok(())
}

#[cfg(feature = "git")]
#[test]
fn cockpit_emits_no_progress_events_when_unset() -> TestResult {
    let Some(dir) = cockpit_repo()? else {
        return Ok(());
    };
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--head", "HEAD"])
        .env_remove("TOKMD_PROGRESS_EVENTS")
        .env_remove("TOKMD_NO_PROGRESS")
        .assert()
        .success()
        .stderr(predicate::str::contains("tokmd.progress").not());
    Ok(())
}

/// Build a git repo whose feature change touches a single scoped file, matching
/// the `packet generate` integration fixtures.
#[cfg(feature = "analysis")]
fn packet_repo() -> Result<Option<tempfile::TempDir>, Box<dyn std::error::Error>> {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return Ok(None);
    }
    let dir = tempfile::tempdir()?;
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return Ok(None);
    }
    let scope_dir = dir.path().join("src").join("runtime").join("api");
    std::fs::create_dir_all(&scope_dir)?;
    std::fs::write(scope_dir.join("MarkdownObject.rs"), "pub fn old() {}\n")?;
    if !common::git_add_commit(dir.path(), "initial") {
        eprintln!("Skipping: initial commit failed");
        return Ok(None);
    }
    std::fs::write(
        scope_dir.join("MarkdownObject.rs"),
        "pub fn old() {}\npub fn new_boundary() {}\n",
    )?;
    if !common::git_add_commit(dir.path(), "change api") {
        eprintln!("Skipping: second commit failed");
        return Ok(None);
    }
    Ok(Some(dir))
}

#[cfg(feature = "analysis")]
#[test]
fn packet_generate_emits_orchestrator_progress_events_on_stderr() -> TestResult {
    let Some(dir) = packet_repo()? else {
        return Ok(());
    };
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--no-syntax",
            "--no-progress",
            "src/runtime/api/MarkdownObject.rs",
        ])
        .env("TOKMD_PROGRESS_EVENTS", "1")
        .env_remove("TOKMD_NO_PROGRESS")
        .assert()
        .success()
        .stderr(predicate::str::contains("Generating analysis receipt"))
        .stderr(predicate::str::contains("Generating context artifact"))
        .stderr(predicate::str::contains("Indexing evidence packet"))
        .stderr(predicate::str::contains(FINISH_LINE))
        .stdout(predicate::str::contains("tokmd.progress").not());
    Ok(())
}

#[cfg(feature = "analysis")]
#[test]
fn packet_generate_emits_no_progress_events_when_unset() -> TestResult {
    let Some(dir) = packet_repo()? else {
        return Ok(());
    };
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--no-syntax",
            "--no-progress",
            "src/runtime/api/MarkdownObject.rs",
        ])
        .env_remove("TOKMD_PROGRESS_EVENTS")
        .env_remove("TOKMD_NO_PROGRESS")
        .assert()
        .success()
        .stderr(predicate::str::contains("tokmd.progress").not());
    Ok(())
}
