//! Deep CLI tests for `tokmd diff` – file-based diffing of lang receipts.
//!
//! These tests exercise diff via synthetic JSON fixtures (no git required).

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/// Minimal lang receipt JSON with the given rows and totals.
fn lang_receipt_json(rows: &[(&str, usize, usize, usize, usize, usize)]) -> String {
    let mut total_code = 0usize;
    let mut total_lines = 0usize;
    let mut total_files = 0usize;
    let mut total_bytes = 0usize;
    let mut total_tokens = 0usize;

    let row_json: Vec<String> = rows
        .iter()
        .map(|(lang, code, lines, files, bytes, tokens)| {
            total_code += code;
            total_lines += lines;
            total_files += files;
            total_bytes += bytes;
            total_tokens += tokens;
            let avg = if *files > 0 { lines / files } else { 0 };
            format!(
                r#"{{"lang":"{}","code":{},"lines":{},"files":{},"bytes":{},"tokens":{},"avg_lines":{}}}"#,
                lang, code, lines, files, bytes, tokens, avg
            )
        })
        .collect();

    let total_avg = if total_files > 0 {
        total_lines / total_files
    } else {
        0
    };

    format!(
        r#"{{
  "schema_version": 2,
  "generated_at_ms": 0,
  "tool": {{"name":"tokmd","version":"0.0.0-test"}},
  "mode": "lang",
  "status": "complete",
  "warnings": [],
  "scan": {{
    "paths": ["."],
    "excluded": [],
    "config": "auto",
    "hidden": false,
    "no_ignore": false,
    "no_ignore_parent": false,
    "no_ignore_dot": false,
    "no_ignore_vcs": false,
    "treat_doc_strings_as_comments": false,
    "redact": "none"
  }},
  "args": {{
    "format": "json",
    "top": 0,
    "with_files": false,
    "children": "collapse"
  }},
  "rows": [{}],
  "total": {{
    "code": {},
    "lines": {},
    "files": {},
    "bytes": {},
    "tokens": {},
    "avg_lines": {}
  }},
  "with_files": false,
  "children": "collapse",
  "top": 0
}}"#,
        row_json.join(","),
        total_code,
        total_lines,
        total_files,
        total_bytes,
        total_tokens,
        total_avg
    )
}

/// Write a lang receipt to a temp dir and return its path.
fn write_receipt(dir: &std::path::Path, name: &str, json: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    fs::write(&p, json).expect("write fixture");
    p
}

fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

// ---------------------------------------------------------------------------
// 1. Diff via --from / --to (file paths)
// ---------------------------------------------------------------------------

#[test]
fn diff_from_to_json_produces_valid_receipt() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .expect("run diff");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["mode"], "diff");
    assert!(json["schema_version"].is_number());
    assert!(json["diff_rows"].is_array());
    assert!(json["totals"].is_object());
}

#[test]
fn diff_from_to_md_produces_markdown() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    tokmd()
        .args(["diff", "--format", "md", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"));
}

// ---------------------------------------------------------------------------
// 2. Diff via positional args
// ---------------------------------------------------------------------------

#[test]
fn diff_positional_args_work() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "a.json",
        &lang_receipt_json(&[("Python", 50, 60, 1, 2000, 100)]),
    );
    let after = write_receipt(
        dir.path(),
        "b.json",
        &lang_receipt_json(&[("Python", 80, 100, 2, 3000, 160)]),
    );

    tokmd()
        .arg("diff")
        .arg(&before)
        .arg(&after)
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"));
}

#[test]
fn diff_positional_json_format() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "a.json",
        &lang_receipt_json(&[("Go", 300, 400, 5, 15000, 750)]),
    );
    let after = write_receipt(
        dir.path(),
        "b.json",
        &lang_receipt_json(&[("Go", 350, 460, 6, 17000, 875)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json"])
        .arg(&before)
        .arg(&after)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "diff");
    let rows = json["diff_rows"].as_array().unwrap();
    assert!(!rows.is_empty());
}

// ---------------------------------------------------------------------------
// 3. Identical inputs -> zero deltas
// ---------------------------------------------------------------------------

#[test]
fn diff_identical_receipts_json_zero_deltas() {
    let dir = tempdir().unwrap();
    let receipt = lang_receipt_json(&[("Rust", 500, 600, 10, 25000, 1250)]);
    let a = write_receipt(dir.path(), "a.json", &receipt);
    let b = write_receipt(dir.path(), "b.json", &receipt);

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let totals = &json["totals"];
    assert_eq!(totals["delta_code"], 0);
    assert_eq!(totals["delta_lines"], 0);
    assert_eq!(totals["delta_files"], 0);
    assert_eq!(totals["delta_bytes"], 0);
    assert_eq!(totals["delta_tokens"], 0);
}

#[test]
fn diff_identical_receipts_md_has_header() {
    let dir = tempdir().unwrap();
    let receipt = lang_receipt_json(&[("Rust", 500, 600, 10, 25000, 1250)]);
    let a = write_receipt(dir.path(), "a.json", &receipt);
    let b = write_receipt(dir.path(), "b.json", &receipt);

    tokmd()
        .args(["diff", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"));
}

// ---------------------------------------------------------------------------
// 4. Completely different inputs (language added / removed)
// ---------------------------------------------------------------------------

#[test]
fn diff_language_added_shows_positive_delta() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[
            ("Rust", 100, 120, 2, 5000, 250),
            ("Python", 50, 60, 1, 2000, 100),
        ]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let totals = &json["totals"];
    assert_eq!(totals["delta_code"], 50);
    assert_eq!(totals["delta_files"], 1);

    let rows = json["diff_rows"].as_array().unwrap();
    let py_row = rows.iter().find(|r| r["lang"] == "Python");
    assert!(py_row.is_some(), "Python row should appear in diff");
    assert_eq!(py_row.unwrap()["old_code"], 0);
    assert_eq!(py_row.unwrap()["new_code"], 50);
}

#[test]
fn diff_language_removed_shows_negative_delta() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[
            ("Rust", 100, 120, 2, 5000, 250),
            ("Python", 50, 60, 1, 2000, 100),
        ]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let totals = &json["totals"];
    assert_eq!(totals["delta_code"], -50);

    let rows = json["diff_rows"].as_array().unwrap();
    let py_row = rows.iter().find(|r| r["lang"] == "Python");
    assert!(py_row.is_some());
    assert_eq!(py_row.unwrap()["new_code"], 0);
    assert_eq!(py_row.unwrap()["delta_code"], -50);
}

#[test]
fn diff_completely_different_languages() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[("Java", 400, 500, 8, 20000, 1000)]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[("Kotlin", 300, 380, 6, 15000, 750)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let rows = json["diff_rows"].as_array().unwrap();
    assert!(rows.len() >= 2, "should have rows for both Java and Kotlin");

    let java = rows.iter().find(|r| r["lang"] == "Java").unwrap();
    assert_eq!(java["new_code"], 0);

    let kotlin = rows.iter().find(|r| r["lang"] == "Kotlin").unwrap();
    assert_eq!(kotlin["old_code"], 0);
}

// ---------------------------------------------------------------------------
// 5. Diff JSON receipt schema fields
// ---------------------------------------------------------------------------

#[test]
fn diff_json_has_from_source_and_to_source() {
    let dir = tempdir().unwrap();
    let a = write_receipt(
        dir.path(),
        "a.json",
        &lang_receipt_json(&[("Rust", 10, 12, 1, 500, 25)]),
    );
    let b = write_receipt(
        dir.path(),
        "b.json",
        &lang_receipt_json(&[("Rust", 20, 24, 1, 1000, 50)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["from_source"].is_string());
    assert!(json["to_source"].is_string());
    assert!(json["tool"]["name"].is_string());
    assert!(json["tool"]["version"].is_string());
}

#[test]
fn diff_json_diff_rows_have_all_metric_fields() {
    let dir = tempdir().unwrap();
    let a = write_receipt(
        dir.path(),
        "a.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let b = write_receipt(
        dir.path(),
        "b.json",
        &lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .output()
        .unwrap();

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let row = &json["diff_rows"][0];

    for field in [
        "lang",
        "old_code",
        "new_code",
        "delta_code",
        "old_lines",
        "new_lines",
        "delta_lines",
        "old_files",
        "new_files",
        "delta_files",
        "old_bytes",
        "new_bytes",
        "delta_bytes",
        "old_tokens",
        "new_tokens",
        "delta_tokens",
    ] {
        assert!(row.get(field).is_some(), "diff row missing field: {field}");
    }
}

#[test]
fn diff_json_totals_have_all_metric_fields() {
    let dir = tempdir().unwrap();
    let a = write_receipt(
        dir.path(),
        "a.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let b = write_receipt(
        dir.path(),
        "b.json",
        &lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .output()
        .unwrap();

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let totals = &json["totals"];

    for field in [
        "old_code",
        "new_code",
        "delta_code",
        "old_lines",
        "new_lines",
        "delta_lines",
        "old_files",
        "new_files",
        "delta_files",
        "old_bytes",
        "new_bytes",
        "delta_bytes",
        "old_tokens",
        "new_tokens",
        "delta_tokens",
    ] {
        assert!(totals.get(field).is_some(), "totals missing field: {field}");
    }
}

// ---------------------------------------------------------------------------
// 6. Multi-language diffs
// ---------------------------------------------------------------------------

#[test]
fn diff_multi_language_growth() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[
            ("Rust", 100, 120, 2, 5000, 250),
            ("Python", 50, 60, 1, 2000, 100),
        ]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[
            ("Rust", 200, 240, 4, 10000, 500),
            ("Python", 80, 100, 2, 3200, 160),
        ]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let totals = &json["totals"];
    assert_eq!(totals["delta_code"], 130); // (200-100) + (80-50)
    assert_eq!(totals["delta_files"], 3); // (4-2) + (2-1)
}

#[test]
fn diff_multi_language_md_mentions_language_movement() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[
            ("Rust", 100, 120, 2, 5000, 250),
            ("Python", 50, 60, 1, 2000, 100),
        ]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[
            ("Rust", 200, 240, 4, 10000, 500),
            ("Python", 80, 100, 2, 3200, 160),
        ]),
    );

    tokmd()
        .args(["diff", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .assert()
        .success()
        .stdout(predicate::str::contains("### Language Movement"));
}

// ---------------------------------------------------------------------------
// 7. Compact mode
// ---------------------------------------------------------------------------

#[test]
fn diff_compact_mode_excludes_language_breakdown() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    tokmd()
        .args(["diff", "--compact", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .assert()
        .success()
        .stdout(predicate::str::contains("Language Breakdown").not());
}

// ---------------------------------------------------------------------------
// 8. Color modes
// ---------------------------------------------------------------------------

#[test]
fn diff_color_never_no_ansi() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    let output = tokmd()
        .args(["diff", "--color", "never", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        !stdout.contains("\x1b["),
        "color=never should produce no ANSI escapes"
    );
}

#[test]
fn diff_color_always_emits_ansi_escapes() {
    let dir = tempdir().unwrap();
    let before = write_receipt(
        dir.path(),
        "before.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_receipt(
        dir.path(),
        "after.json",
        &lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    let output = tokmd()
        .args(["diff", "--color", "always", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("\x1b["),
        "color=always should produce ANSI escapes"
    );
}

// ---------------------------------------------------------------------------
// 9. Error cases
// ---------------------------------------------------------------------------

#[test]
fn diff_missing_from_fails() {
    tokmd()
        .args(["diff", "--to", "nonexistent.json"])
        .assert()
        .failure();
}

#[test]
fn diff_missing_both_args_fails() {
    tokmd().arg("diff").assert().failure();
}

#[test]
fn diff_single_positional_arg_fails() {
    let dir = tempdir().unwrap();
    let a = write_receipt(
        dir.path(),
        "a.json",
        &lang_receipt_json(&[("Rust", 10, 12, 1, 500, 25)]),
    );

    tokmd().arg("diff").arg(&a).assert().failure();
}

// ---------------------------------------------------------------------------
// 10. Receipt.json pointing to lang.json sibling
// ---------------------------------------------------------------------------

#[test]
fn diff_via_receipt_json_resolves_to_sibling_lang_json() {
    let dir = tempdir().unwrap();

    let run1 = dir.path().join("run1");
    fs::create_dir_all(&run1).unwrap();
    fs::write(
        run1.join("lang.json"),
        lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    )
    .unwrap();
    fs::write(run1.join("receipt.json"), r#"{"artifacts":["lang.json"]}"#).unwrap();

    let run2 = dir.path().join("run2");
    fs::create_dir_all(&run2).unwrap();
    fs::write(
        run2.join("lang.json"),
        lang_receipt_json(&[("Rust", 200, 240, 3, 10000, 500)]),
    )
    .unwrap();
    fs::write(run2.join("receipt.json"), r#"{"artifacts":["lang.json"]}"#).unwrap();

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(run1.join("receipt.json"))
        .arg("--to")
        .arg(run2.join("receipt.json"))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "diff");
    assert_eq!(json["totals"]["delta_code"], 100);
}

#[test]
fn diff_via_directory_resolves_to_lang_json() {
    let dir = tempdir().unwrap();

    let run1 = dir.path().join("run1");
    fs::create_dir_all(&run1).unwrap();
    fs::write(
        run1.join("lang.json"),
        lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    )
    .unwrap();

    let run2 = dir.path().join("run2");
    fs::create_dir_all(&run2).unwrap();
    fs::write(
        run2.join("lang.json"),
        lang_receipt_json(&[("Rust", 150, 180, 2, 7500, 375)]),
    )
    .unwrap();

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&run1)
        .arg("--to")
        .arg(&run2)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["totals"]["delta_code"], 50);
}

// ---------------------------------------------------------------------------
// 11. Empty receipts (no languages)
// ---------------------------------------------------------------------------

#[test]
fn diff_empty_receipts_produces_zero_totals() {
    let dir = tempdir().unwrap();
    let empty = lang_receipt_json(&[]);
    let a = write_receipt(dir.path(), "a.json", &empty);
    let b = write_receipt(dir.path(), "b.json", &empty);

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["totals"]["delta_code"], 0);
    let rows = json["diff_rows"].as_array().unwrap();
    assert!(rows.is_empty());
}

#[test]
fn diff_from_empty_to_populated() {
    let dir = tempdir().unwrap();
    let a = write_receipt(dir.path(), "a.json", &lang_receipt_json(&[]));
    let b = write_receipt(
        dir.path(),
        "b.json",
        &lang_receipt_json(&[("Rust", 100, 120, 2, 5000, 250)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["totals"]["delta_code"], 100);
    assert_eq!(json["totals"]["new_code"], 100);
    assert_eq!(json["totals"]["old_code"], 0);
}
