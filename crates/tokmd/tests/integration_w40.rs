#![cfg(feature = "analysis")]

//! Wave-40 CLI integration tests.
//!
//! End-to-end tests that invoke the real `tokmd` binary and verify
//! stdout output for all major commands and flag combinations.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ============================================================================
// 1. tokmd . produces valid markdown to stdout
// ============================================================================

#[test]
fn lang_default_produces_markdown_table() {
    tokmd_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn lang_explicit_produces_markdown_table() {
    tokmd_cmd()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
}

// ============================================================================
// 2. tokmd --json produces valid JSON (via lang --format json)
// ============================================================================

#[test]
fn lang_json_is_valid() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["mode"], "lang");
    assert!(json["schema_version"].is_number());
}

#[test]
fn lang_json_rows_have_required_fields() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["lang"].is_string());
        assert!(row["code"].is_number());
        assert!(row["files"].is_number());
        assert!(row["lines"].is_number());
    }
}

#[test]
fn lang_json_total_present() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["total"].is_object());
    assert!(json["total"]["code"].is_number());
    assert!(json["total"]["files"].is_number());
}

// ============================================================================
// 3. tokmd module --depth 2 produces correct depth
// ============================================================================

#[test]
fn module_json_has_correct_depth() {
    let output = tokmd_cmd()
        .args(["module", "--module-depth", "2", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "module");
    assert_eq!(json["module_depth"], 2);
}

#[test]
fn module_depth_1_json() {
    let output = tokmd_cmd()
        .args(["module", "--module-depth", "1", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["module_depth"], 1);
}

#[test]
fn module_default_markdown() {
    tokmd_cmd()
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
}

// ============================================================================
// 4. tokmd export --format jsonl produces valid JSONL
// ============================================================================

#[test]
fn export_jsonl_each_line_valid_json() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need meta + data rows");
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} invalid JSON: {e}", i + 1));
    }
}

#[test]
fn export_csv_has_header_and_data() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2);
    assert!(lines[0].contains("path"));
    assert!(lines[0].contains("code"));
}

// ============================================================================
// 5. tokmd analyze --preset receipt produces analysis receipt
// ============================================================================

#[test]
fn analyze_receipt_json_has_derived() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object());
}

#[test]
fn analyze_receipt_markdown() {
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ============================================================================
// 6. tokmd badge produces valid SVG
// ============================================================================

#[test]
fn badge_lines_outputs_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_tokens_outputs_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

// ============================================================================
// 7. tokmd init produces .tokeignore content
// ============================================================================

#[test]
fn init_print_produces_tokeignore() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ============================================================================
// 8. tokmd completions bash produces shell completions
// ============================================================================

#[test]
fn completions_bash_produces_script() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_produces_script() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_produces_script() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ============================================================================
// 9. Idempotent output (run twice, same result)
// ============================================================================

#[test]
fn lang_json_idempotent() {
    let out1 = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("first run");
    let out2 = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("second run");

    assert!(out1.status.success());
    assert!(out2.status.success());

    let mut j1: Value = serde_json::from_slice(&out1.stdout).unwrap();
    let mut j2: Value = serde_json::from_slice(&out2.stdout).unwrap();

    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2, "two runs should produce identical JSON");
}

#[test]
fn module_json_idempotent() {
    let out1 = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("first run");
    let out2 = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("second run");

    assert!(out1.status.success());
    assert!(out2.status.success());

    let mut j1: Value = serde_json::from_slice(&out1.stdout).unwrap();
    let mut j2: Value = serde_json::from_slice(&out2.stdout).unwrap();

    strip_volatile(&mut j1);
    strip_volatile(&mut j2);
    assert_eq!(j1, j2);
}

#[test]
fn export_jsonl_idempotent() {
    let out1 = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("first run");
    let out2 = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("second run");

    assert!(out1.status.success());
    assert!(out2.status.success());

    let lines1: Vec<&str> = std::str::from_utf8(&out1.stdout)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    let lines2: Vec<&str> = std::str::from_utf8(&out2.stdout)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    assert_eq!(lines1.len(), lines2.len());

    for (i, (l1, l2)) in lines1.iter().zip(lines2.iter()).enumerate() {
        let mut v1: Value = serde_json::from_str(l1).unwrap();
        let mut v2: Value = serde_json::from_str(l2).unwrap();
        strip_volatile(&mut v1);
        strip_volatile(&mut v2);
        assert_eq!(v1, v2, "JSONL line {i} should be identical");
    }
}

// ============================================================================
// 10. Additional coverage
// ============================================================================

#[test]
fn version_flag_shows_semver() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

#[test]
fn lang_tsv_contains_tabs() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'));
}

#[test]
fn export_json_format_valid() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["mode"], "export");
    assert!(json["rows"].is_array());
}

// ============================================================================
// Helpers
// ============================================================================

fn strip_volatile(v: &mut Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("generated_at_ms");
        obj.remove("scan_duration_ms");
        obj.remove("export_generated_at_ms");
        for (_, child) in obj.iter_mut() {
            strip_volatile(child);
        }
    }
    if let Some(arr) = v.as_array_mut() {
        for child in arr.iter_mut() {
            strip_volatile(child);
        }
    }
}
