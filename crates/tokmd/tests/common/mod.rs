//! Shared test utilities for tokmd integration tests.
//!
//! This module provides hermetic test fixtures that work correctly across all
//! environments, including cargo-mutants which copies the crate to a temp
//! directory without the parent `.git/` marker.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

static FIXTURE_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Returns path to a hermetic copy of test fixtures with `.git/` marker.
///
/// The fixture is initialized once per test process using `OnceLock`.
/// This ensures that:
/// 1. The `ignore` crate honors `.gitignore` rules (requires `.git/` marker)
/// 2. Tests work in cargo-mutants environment (no parent `.git/`)
/// 3. Fixture is only copied once for efficiency
pub fn fixture_root() -> &'static Path {
    FIXTURE_ROOT
        .get_or_init(|| {
            let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("data");

            let dst = std::env::temp_dir().join(format!(
                "tokmd-fixtures-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));

            let _ = std::fs::remove_dir_all(&dst);
            copy_dir_recursive(&src, &dst).expect("copy test fixtures");
            std::fs::create_dir_all(dst.join(".git")).expect("create .git marker");

            dst
        })
        .as_path()
}

// ---------------------------------------------------------------------------
// Git helpers – shared by sensor_integration and cockpit_integration tests.
// ---------------------------------------------------------------------------

/// Returns `true` when `git --version` succeeds on the current `PATH`.
pub fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Initialise a fresh git repo in `dir` with default branch `main`.
pub fn init_git_repo(dir: &Path) -> bool {
    let commands = [
        vec!["init"],
        vec!["symbolic-ref", "HEAD", "refs/heads/main"],
        vec!["config", "user.email", "test@test.com"],
        vec!["config", "user.name", "Test User"],
    ];

    for args in &commands {
        let status = Command::new("git").args(args).current_dir(dir).status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            return false;
        }
    }
    true
}

/// Stage everything and commit with the given `message`.
pub fn git_add_commit(dir: &Path, message: &str) -> bool {
    let commands = [vec!["add", "."], vec!["commit", "-m", message]];

    for args in &commands {
        let status = Command::new("git").args(args).current_dir(dir).status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            return false;
        }
    }
    true
}

/// Return the HEAD commit SHA, or `None` on failure.
pub fn git_head(dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Write a minimal `mutants-summary.json` that satisfies the mutation gate.
///
/// The sensor command checks for this file to decide whether mutation testing
/// already ran.  Writing it with the current HEAD commit prevents
/// cargo-mutants from being invoked during the test.
pub fn write_mutants_summary(dir: &Path, commit: &str, scope: &str, status: &str) {
    let json = format!(
        r#"{{"commit":"{}","scope":"{}","status":"{}","survivors":[],"killed":0,"timeout":0,"unviable":0}}"#,
        commit, scope, status
    );
    let mutants_dir = dir.join("mutants.out");
    std::fs::create_dir_all(&mutants_dir).expect("create mutants.out dir");
    std::fs::write(mutants_dir.join("mutants-summary.json"), json)
        .expect("write mutants-summary.json");
}
