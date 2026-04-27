#![cfg(feature = "analysis")]

//! W73 – CLI error handling and help output tests (~30 tests).
//!
//! Validates that:
//! - Invalid flags / formats / missing args produce clear errors
//! - Every subcommand exposes its expected flags in `--help`
//! - `--version` / `-V` emit the Cargo.toml version string

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// =========================================================================
// 1. Error cases
// =========================================================================

#[test]
fn error_lang_invalid_format() {
    tokmd_cmd()
        .args(["lang", "--format", "invalid_format"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn error_unknown_flag() {
    tokmd_cmd().arg("--unknown-flag").assert().failure().stderr(
        predicate::str::contains("unexpected argument").or(predicate::str::contains("error")),
    );
}

#[test]
fn error_export_nonexistent_path() {
    tokmd_cmd()
        .args(["export", "--format", "json", "--path", "/nonexistent_w73"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn error_diff_no_args() {
    tokmd_cmd().arg("diff").assert().failure().stderr(
        predicate::str::contains("Provide either two positional refs")
            .or(predicate::str::contains("--from"))
            .or(predicate::str::contains("error")),
    );
}

#[test]
fn error_gate_nonexistent_policy() {
    tokmd_cmd()
        .args(["gate", "--policy", "nonexistent_w73.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn error_module_invalid_format() {
    tokmd_cmd()
        .args(["module", "--format", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn error_analyze_invalid_preset() {
    tokmd_cmd()
        .args(["analyze", "--preset", "bogus_preset"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// =========================================================================
// 2. Help output – root
// =========================================================================

#[test]
fn help_root_contains_subcommands() {
    let assert = tokmd_cmd().arg("--help").assert().success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    for subcmd in &[
        "lang",
        "module",
        "export",
        "analyze",
        "badge",
        "diff",
        "context",
        "gate",
        "handoff",
        "completions",
        "run",
        "init",
    ] {
        assert!(
            stdout.contains(subcmd),
            "root --help missing subcommand `{subcmd}`"
        );
    }
}

#[test]
fn help_root_mentions_version_flag() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--version").or(predicate::str::contains("-V")));
}

// =========================================================================
// 2. Help output – subcommands
// =========================================================================

#[test]
fn help_lang_mentions_format_and_children() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format").and(predicate::str::contains("--children")));
}

#[test]
fn help_module_mentions_depth_and_format() {
    tokmd_cmd()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("--module-depth")
                .or(predicate::str::contains("--depth"))
                .and(predicate::str::contains("--format")),
        );
}

#[test]
fn help_export_mentions_format_variants() {
    let assert = tokmd_cmd().args(["export", "--help"]).assert().success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    assert!(
        stdout.contains("--format"),
        "export --help missing --format"
    );
    // At least two of the three well-known export formats should appear
    let hits = ["jsonl", "csv", "json"]
        .iter()
        .filter(|f| stdout.contains(**f))
        .count();
    assert!(
        hits >= 2,
        "export --help should mention at least 2 of jsonl/csv/json, found {hits}"
    );
}

#[test]
fn help_analyze_mentions_preset() {
    tokmd_cmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--preset"));
}

#[test]
fn help_context_mentions_mode_and_budget() {
    tokmd_cmd()
        .args(["context", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--mode").and(predicate::str::contains("--budget")));
}

#[test]
fn help_handoff_mentions_preset() {
    tokmd_cmd()
        .args(["handoff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--preset"));
}

#[test]
fn help_completions_mentions_shell_types() {
    let assert = tokmd_cmd()
        .args(["completions", "--help"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);

    for shell in &["bash", "zsh", "fish", "powershell", "elvish"] {
        assert!(
            stdout.to_lowercase().contains(shell),
            "completions --help missing shell type `{shell}`"
        );
    }
}

#[test]
fn help_diff_mentions_from_and_to() {
    tokmd_cmd()
        .args(["diff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--from").and(predicate::str::contains("--to")));
}

#[test]
fn help_gate_mentions_policy() {
    tokmd_cmd()
        .args(["gate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--policy"));
}

#[test]
fn help_badge_exists() {
    tokmd_cmd()
        .args(["badge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn help_run_exists() {
    tokmd_cmd()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn help_init_exists() {
    tokmd_cmd()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn help_check_ignore_exists() {
    tokmd_cmd()
        .args(["check-ignore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn help_tools_exists() {
    tokmd_cmd()
        .args(["tools", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn help_cockpit_exists() {
    tokmd_cmd()
        .args(["cockpit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn help_sensor_exists() {
    tokmd_cmd()
        .args(["sensor", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn help_baseline_exists() {
    tokmd_cmd()
        .args(["baseline", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// =========================================================================
// 3. Version output
// =========================================================================

#[test]
fn version_long_flag_matches_cargo() {
    let version = env!("CARGO_PKG_VERSION");
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(version));
}

#[test]
fn version_short_flag_matches_cargo() {
    let version = env!("CARGO_PKG_VERSION");
    tokmd_cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(version));
}

#[test]
fn version_long_and_short_agree() {
    let long = tokmd_cmd().arg("--version").output().expect("--version");
    let short = tokmd_cmd().arg("-V").output().expect("-V");
    assert_eq!(long.stdout, short.stdout, "--version and -V should agree");
}

#[test]
fn error_completions_invalid_shell() {
    tokmd_cmd()
        .args(["completions", "invalid_shell"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
