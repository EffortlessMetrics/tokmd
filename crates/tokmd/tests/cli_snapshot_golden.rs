//! Golden snapshot tests for CLI output formats.
//!
//! These tests capture the full output of each major CLI command and format
//! combination as insta snapshots.  Any unintentional change to the output
//! format will cause a test failure, making regressions easy to spot.

mod common;

use assert_cmd::Command;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Replace dynamic values (timestamps, versions, absolute paths) with stable
/// placeholders so snapshots are deterministic across machines and runs.
fn normalize(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\d+"#).unwrap();
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();

    let re_ver = regex::Regex::new(r#"("tool":\{"name":"tokmd","version":")[^"]+"#).unwrap();
    let s = re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string();

    // Normalize --version output line (e.g. "tokmd 0.42.1" -> "tokmd <VERSION>")
    let re_version_line = regex::Regex::new(r"tokmd \d+\.\d+\.\d+").unwrap();
    let s = re_version_line
        .replace_all(&s, "tokmd <VERSION>")
        .to_string();

    // Normalize absolute paths that may leak into scan.paths
    let re_abs = regex::Regex::new(r#""paths":\["[^"]*"\]"#).unwrap();
    re_abs.replace_all(&s, r#""paths":["<ROOT>"]"#).to_string()
}

// ---------------------------------------------------------------------------
// 1. JSON output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run tokmd lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("lang_json", normalize(&stdout));
}

#[test]
fn snapshot_lang_json_structure() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run tokmd lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify structural keys are present — the snapshot captures exact values
    assert_eq!(json["mode"], "lang");
    assert!(json["schema_version"].is_number());
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert!(json["tool"].is_object());
}

// ---------------------------------------------------------------------------
// 2. Markdown output snapshots (default format)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_markdown() {
    let output = tokmd_cmd()
        .args(["lang"])
        .output()
        .expect("failed to run tokmd lang");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("lang_markdown", stdout);
}

// ---------------------------------------------------------------------------
// 3. TSV output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_tsv() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to run tokmd lang --format tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("lang_tsv", stdout);
}

// ---------------------------------------------------------------------------
// 4. Module output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_module_json() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run tokmd module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("module_json", normalize(&stdout));
}

#[test]
fn snapshot_module_markdown() {
    let output = tokmd_cmd()
        .args(["module"])
        .output()
        .expect("failed to run tokmd module");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("module_markdown", stdout);
}

#[test]
fn snapshot_module_tsv() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("failed to run tokmd module --format tsv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("module_tsv", stdout);
}

// ---------------------------------------------------------------------------
// 5. Export output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_export_json() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run tokmd export --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("export_json", normalize(&stdout));
}

#[test]
fn snapshot_export_jsonl() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run tokmd export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("export_jsonl", normalize(&stdout));
}

#[test]
fn snapshot_export_csv() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run tokmd export --format csv");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("export_csv", stdout);
}

// ---------------------------------------------------------------------------
// 6. Version output snapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_version() {
    let output = tokmd_cmd()
        .arg("--version")
        .output()
        .expect("failed to run tokmd --version");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    insta::assert_snapshot!("version", normalize(&stdout));
}

// ---------------------------------------------------------------------------
// 7. Help output snapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_help() {
    let output = tokmd_cmd()
        .arg("--help")
        .output()
        .expect("failed to run tokmd --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Normalize the version in the help header
    insta::assert_snapshot!("help", normalize(&stdout));
}
