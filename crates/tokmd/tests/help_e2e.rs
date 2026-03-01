//! E2E tests verifying that every CLI subcommand prints help and exits 0.

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

// ── tokmd --help ─────────────────────────────────────────────────────

#[test]
fn top_level_help_exits_successfully() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("tokmd"));
}

// ── tokmd lang --help ────────────────────────────────────────────────

#[test]
fn lang_help_exits_successfully() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

// ── tokmd module --help ──────────────────────────────────────────────

#[test]
fn module_help_exits_successfully() {
    tokmd_cmd().args(["module", "--help"]).assert().success();
}

// ── tokmd export --help ──────────────────────────────────────────────

#[test]
fn export_help_exits_successfully() {
    tokmd_cmd().args(["export", "--help"]).assert().success();
}

// ── tokmd analyze --help ─────────────────────────────────────────────

#[test]
fn analyze_help_exits_successfully() {
    tokmd_cmd().args(["analyze", "--help"]).assert().success();
}

// ── tokmd diff --help ────────────────────────────────────────────────

#[test]
fn diff_help_exits_successfully() {
    tokmd_cmd().args(["diff", "--help"]).assert().success();
}

// ── tokmd cockpit --help ─────────────────────────────────────────────

#[test]
fn cockpit_help_exits_successfully() {
    tokmd_cmd().args(["cockpit", "--help"]).assert().success();
}

// ── tokmd gate --help ────────────────────────────────────────────────

#[test]
fn gate_help_exits_successfully() {
    tokmd_cmd().args(["gate", "--help"]).assert().success();
}

// ── tokmd context --help ─────────────────────────────────────────────

#[test]
fn context_help_exits_successfully() {
    tokmd_cmd().args(["context", "--help"]).assert().success();
}

// ── tokmd handoff --help ─────────────────────────────────────────────

#[test]
fn handoff_help_exits_successfully() {
    tokmd_cmd().args(["handoff", "--help"]).assert().success();
}

// ── tokmd sensor --help ──────────────────────────────────────────────

#[test]
fn sensor_help_exits_successfully() {
    tokmd_cmd().args(["sensor", "--help"]).assert().success();
}

// ── tokmd baseline --help ────────────────────────────────────────────

#[test]
fn baseline_help_exits_successfully() {
    tokmd_cmd().args(["baseline", "--help"]).assert().success();
}

// ── tokmd tools --help ───────────────────────────────────────────────

#[test]
fn tools_help_exits_successfully() {
    tokmd_cmd().args(["tools", "--help"]).assert().success();
}

// ── tokmd completions --help ─────────────────────────────────────────

#[test]
fn completions_help_exits_successfully() {
    tokmd_cmd()
        .args(["completions", "--help"])
        .assert()
        .success();
}
