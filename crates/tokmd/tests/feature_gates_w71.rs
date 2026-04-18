//! CLI-level feature gate boundary tests.
//!
//! Verifies that the tokmd CLI correctly surfaces feature availability
//! through JSON output structure, help text, and graceful degradation.
//! Enforces the "no green by omission" invariant at the CLI boundary.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

fn fixture() -> std::path::PathBuf {
    common::fixture_root().to_path_buf()
}

// -- help text feature visibility --

/// Analyze help must list all known presets so users know what's available.
#[cfg(feature = "analysis")]
#[test]
fn analyze_help_lists_preset_values() {
    tokmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("receipt"))
        .stdout(predicate::str::contains("deep"));
}

/// Analyze help mentions format options.
#[cfg(feature = "analysis")]
#[test]
fn analyze_help_mentions_format_option() {
    tokmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

/// Analyze help mentions the git override flag.
#[cfg(feature = "analysis")]
#[test]
fn analyze_help_mentions_git_flag() {
    tokmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("git"));
}

// -- JSON output structure tests --

/// Analyze receipt preset JSON output includes warnings array.
#[cfg(feature = "analysis")]
#[test]
fn analyze_receipt_json_includes_warnings_array() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json.get("warnings").unwrap().is_array(),
        "warnings must be an array"
    );
}

/// Analyze receipt preset JSON output includes status field.
#[cfg(feature = "analysis")]
#[test]
fn analyze_receipt_json_includes_status_field() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let status = json.get("status").unwrap().as_str().unwrap();
    assert!(
        status == "complete" || status == "partial",
        "status must be 'complete' or 'partial', got: {status}"
    );
}

/// Deep preset JSON output has schema_version and derived section.
#[cfg(feature = "analysis")]
#[test]
fn analyze_deep_json_has_schema_and_derived() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "deep", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("schema_version").is_some());
    assert!(json.get("derived").is_some(), "deep must produce derived");
}

/// Deep preset JSON includes tool.name = "tokmd".
#[cfg(feature = "analysis")]
#[test]
fn analyze_deep_json_tool_name() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "deep", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let tool_name = json["tool"]["name"].as_str().unwrap();
    assert_eq!(tool_name, "tokmd");
}

/// Health preset JSON always has warnings array (possibly non-empty
/// if content/walk features are missing).
#[cfg(feature = "analysis")]
#[test]
fn analyze_health_json_has_warnings() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "health", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("warnings").unwrap().is_array());
}

// -- graceful degradation --

/// Invalid preset name returns non-zero exit code.
#[cfg(feature = "analysis")]
#[test]
fn analyze_invalid_preset_fails() {
    tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "nonexistent_w71"])
        .assert()
        .failure();
}

/// Analyze with --no-git flag suppresses git enricher (deep preset).
#[cfg(feature = "analysis")]
#[test]
fn analyze_deep_no_git_flag_suppresses_git() {
    let output = tokmd()
        .current_dir(fixture())
        .args([
            "analyze", "--preset", "deep", "--format", "json", "--no-git",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json.get("git").unwrap().is_null() || json.get("git").is_none(),
        "git must be null/absent with --no-git"
    );
}

/// Analyze receipt preset exits successfully -- it requires no optional features.
#[cfg(feature = "analysis")]
#[test]
fn analyze_receipt_always_succeeds() {
    tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .assert()
        .success();
}

/// Analyze with --preset deep includes all available features.
#[cfg(feature = "analysis")]
#[test]
fn analyze_deep_includes_available_features() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "deep", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(json.get("derived").is_some(), "deep must include derived");
    assert_eq!(json["mode"], "analysis");
}

// -- all presets produce valid JSON --

/// Every preset produces parseable JSON with required envelope fields.
#[cfg(feature = "analysis")]
#[test]
fn all_presets_produce_valid_json_envelope() {
    let presets = [
        "receipt",
        "health",
        "risk",
        "supply",
        "architecture",
        "security",
        "deep",
    ];
    for preset in presets {
        let output = tokmd()
            .current_dir(fixture())
            .args(["analyze", "--preset", preset, "--format", "json"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "preset '{preset}' must succeed, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "preset '{preset}' must produce valid JSON: {e}\nstdout: {}",
                String::from_utf8_lossy(&output.stdout)
            )
        });
        assert!(
            json.get("schema_version").is_some(),
            "preset '{preset}' must have schema_version"
        );
        assert!(
            json.get("warnings").is_some(),
            "preset '{preset}' must have warnings"
        );
        assert!(
            json.get("status").is_some(),
            "preset '{preset}' must have status"
        );
        assert!(
            json.get("tool").is_some(),
            "preset '{preset}' must have tool"
        );
    }
}

/// Receipt-mode JSON never includes git, entropy, or assets sections.
#[cfg(feature = "analysis")]
#[test]
fn receipt_json_excludes_optional_sections() {
    let output = tokmd()
        .current_dir(fixture())
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(
        json.get("git").unwrap().is_null(),
        "receipt must not include git"
    );
    assert!(
        json.get("entropy").unwrap().is_null(),
        "receipt must not include entropy"
    );
    assert!(
        json.get("assets").unwrap().is_null(),
        "receipt must not include assets"
    );
}
