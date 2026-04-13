#![cfg(feature = "analysis")]
//! W72 – Documentation synchronization tests (CLI binary side).
//!
//! These tests run the actual `tokmd` binary and verify that help output,
//! version output, and subcommand discovery match documentation.

mod common;

use assert_cmd::Command;

const README_MD: &str = include_str!("../../../README.md");
const REFERENCE_CLI_MD: &str = include_str!("../../../docs/reference-cli.md");
const ROOT_CARGO_TOML: &str = include_str!("../../../Cargo.toml");

/// Build a `tokmd` command pointed at the test fixtures.
fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Parse subcommand names from `tokmd --help` output.
fn parse_help_subcommands(help: &str) -> Vec<String> {
    let mut cmds = Vec::new();
    let mut in_commands = false;
    for line in help.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Commands:") {
            in_commands = true;
            continue;
        }
        if in_commands {
            if trimmed.is_empty()
                || trimmed.starts_with("Arguments:")
                || trimmed.starts_with("Options:")
            {
                break;
            }
            if let Some(name) = trimmed.split_whitespace().next()
                && name != "help"
            {
                cmds.push(name.to_string());
            }
        }
    }
    cmds
}

// ===========================================================================
// 1. tokmd --help mentions all documented subcommands
// ===========================================================================

/// Subcommands listed in README.md Commands table.
const DOCUMENTED_SUBCOMMANDS: &[&str] = &[
    "lang",
    "module",
    "context",
    "export",
    "run",
    "analyze",
    "badge",
    "diff",
    "cockpit",
    "sensor",
    "gate",
    "baseline",
    "handoff",
    "tools",
    "init",
    "check-ignore",
    "completions",
];

#[test]
fn help_output_mentions_all_documented_subcommands() {
    let output = tokmd_cmd()
        .arg("--help")
        .output()
        .expect("tokmd --help should succeed");
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);

    for cmd in DOCUMENTED_SUBCOMMANDS {
        assert!(
            help.contains(cmd),
            "`tokmd --help` should mention subcommand `{cmd}`"
        );
    }
}

// ===========================================================================
// 2. Every subcommand in help output is documented somewhere
// ===========================================================================

#[test]
fn every_help_subcommand_is_in_readme() {
    let output = tokmd_cmd()
        .arg("--help")
        .output()
        .expect("tokmd --help should succeed");
    let help = String::from_utf8_lossy(&output.stdout);
    let subcommands = parse_help_subcommands(&help);
    assert!(
        !subcommands.is_empty(),
        "Failed to parse subcommands from --help"
    );

    for cmd in &subcommands {
        let pattern = format!("tokmd {cmd}");
        let found =
            README_MD.contains(&pattern) || (cmd == "lang" && README_MD.contains("| `tokmd`"));
        assert!(
            found,
            "Subcommand `{cmd}` from --help is not documented in README.md"
        );
    }
}

#[test]
fn every_help_subcommand_is_in_reference_cli() {
    let output = tokmd_cmd()
        .arg("--help")
        .output()
        .expect("tokmd --help should succeed");
    let help = String::from_utf8_lossy(&output.stdout);
    let subcommands = parse_help_subcommands(&help);

    for cmd in &subcommands {
        let pattern = format!("tokmd {cmd}");
        let default_pattern = "tokmd` (Default";
        let found = REFERENCE_CLI_MD.contains(&pattern)
            || (cmd == "lang" && REFERENCE_CLI_MD.contains(default_pattern));
        assert!(
            found,
            "Subcommand `{cmd}` from --help is not in docs/reference-cli.md"
        );
    }
}

// ===========================================================================
// 3. Version output matches Cargo.toml version
// ===========================================================================

fn cargo_toml_version() -> String {
    for line in ROOT_CARGO_TOML.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version = \"") && trimmed.ends_with('"') {
            return trimmed
                .strip_prefix("version = \"")
                .unwrap()
                .trim_end_matches('"')
                .to_string();
        }
    }
    panic!("Could not find workspace version in Cargo.toml");
}

#[test]
fn version_output_matches_cargo_toml() {
    let output = tokmd_cmd()
        .arg("--version")
        .output()
        .expect("tokmd --version should succeed");
    assert!(output.status.success());
    let version_line = String::from_utf8_lossy(&output.stdout);
    let expected = cargo_toml_version();
    assert!(
        version_line.contains(&expected),
        "`tokmd --version` output ({}) should contain Cargo.toml version ({expected})",
        version_line.trim()
    );
}

// ===========================================================================
// 4. About text is present and non-empty
// ===========================================================================

#[test]
fn help_output_has_about_text() {
    let output = tokmd_cmd()
        .arg("--help")
        .output()
        .expect("tokmd --help should succeed");
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    // The first non-empty line should be the about text
    let first_line = help.lines().find(|l| !l.trim().is_empty());
    assert!(first_line.is_some(), "--help output should have content");
    assert!(
        !first_line.unwrap().trim().is_empty(),
        "--help about text should be non-empty"
    );
}

#[test]
fn help_output_contains_usage_line() {
    let output = tokmd_cmd()
        .arg("--help")
        .output()
        .expect("tokmd --help should succeed");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        help.contains("Usage:"),
        "--help output should contain a Usage: line"
    );
}

// ===========================================================================
// 5. Each documented subcommand has working --help
// ===========================================================================

#[test]
fn lang_subcommand_help_works() {
    tokmd_cmd().args(["lang", "--help"]).assert().success();
}

#[test]
fn module_subcommand_help_works() {
    tokmd_cmd().args(["module", "--help"]).assert().success();
}

#[test]
fn export_subcommand_help_works() {
    tokmd_cmd().args(["export", "--help"]).assert().success();
}

#[test]
fn analyze_subcommand_help_works() {
    tokmd_cmd().args(["analyze", "--help"]).assert().success();
}

#[test]
fn badge_subcommand_help_works() {
    tokmd_cmd().args(["badge", "--help"]).assert().success();
}

#[test]
fn context_subcommand_help_works() {
    tokmd_cmd().args(["context", "--help"]).assert().success();
}

#[test]
fn tools_subcommand_help_works() {
    tokmd_cmd().args(["tools", "--help"]).assert().success();
}

#[test]
fn gate_subcommand_help_works() {
    tokmd_cmd().args(["gate", "--help"]).assert().success();
}

#[test]
fn completions_subcommand_help_works() {
    tokmd_cmd()
        .args(["completions", "--help"])
        .assert()
        .success();
}

#[test]
fn init_subcommand_help_works() {
    tokmd_cmd().args(["init", "--help"]).assert().success();
}
