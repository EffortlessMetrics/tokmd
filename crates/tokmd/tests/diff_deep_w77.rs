//! Deep tests for `tokmd diff` – W77.
//!
//! ~20 tests covering: identical receipts, added/removed/changed languages,
//! JSON and Markdown output formats, file-based diffing, live run-based diffing,
//! error handling, and schema version verification.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

/// Build a minimal but complete lang receipt JSON from row tuples.
fn lang_receipt(rows: &[(&str, usize, usize, usize, usize, usize)]) -> String {
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
    "treat_doc_strings_as_comments": false
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

/// Write JSON to a file in the given directory and return its path.
fn write_fixture(dir: &std::path::Path, name: &str, json: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    fs::write(&p, json).expect("write fixture");
    p
}

fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

// =========================================================================
// 1. Diff of same receipt → no changes
// =========================================================================

#[test]
fn diff_same_receipt_json_all_deltas_zero() {
    let dir = tempdir().unwrap();
    let receipt = lang_receipt(&[("Rust", 500, 650, 10, 25000, 1250)]);
    let path = write_fixture(dir.path(), "same.json", &receipt);

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&path)
        .arg("--to")
        .arg(&path)
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
fn diff_same_receipt_each_row_delta_is_zero() {
    let dir = tempdir().unwrap();
    let receipt = lang_receipt(&[
        ("Rust", 300, 400, 5, 15000, 750),
        ("Python", 100, 130, 3, 5000, 250),
    ]);
    let path = write_fixture(dir.path(), "same.json", &receipt);

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&path)
        .arg("--to")
        .arg(&path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    for row in json["diff_rows"].as_array().unwrap() {
        assert_eq!(row["delta_code"], 0, "row {} delta_code != 0", row["lang"]);
        assert_eq!(
            row["delta_lines"], 0,
            "row {} delta_lines != 0",
            row["lang"]
        );
        assert_eq!(
            row["delta_files"], 0,
            "row {} delta_files != 0",
            row["lang"]
        );
    }
}

// =========================================================================
// 2. Diff with added language → shows addition
// =========================================================================

#[test]
fn diff_added_language_appears_with_positive_delta() {
    let dir = tempdir().unwrap();
    let before = write_fixture(
        dir.path(),
        "before.json",
        &lang_receipt(&[("Rust", 200, 260, 4, 10000, 500)]),
    );
    let after = write_fixture(
        dir.path(),
        "after.json",
        &lang_receipt(&[
            ("Rust", 200, 260, 4, 10000, 500),
            ("Go", 80, 100, 2, 4000, 200),
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

    let rows = json["diff_rows"].as_array().unwrap();
    let go_row = rows.iter().find(|r| r["lang"] == "Go").expect("Go row");
    assert_eq!(go_row["old_code"], 0);
    assert_eq!(go_row["new_code"], 80);
    assert_eq!(go_row["delta_code"], 80);
}

#[test]
fn diff_added_language_totals_reflect_addition() {
    let dir = tempdir().unwrap();
    let before = write_fixture(
        dir.path(),
        "before.json",
        &lang_receipt(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_fixture(
        dir.path(),
        "after.json",
        &lang_receipt(&[
            ("Rust", 100, 120, 2, 5000, 250),
            ("TypeScript", 60, 75, 3, 3000, 150),
        ]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["totals"]["delta_code"], 60);
    assert_eq!(json["totals"]["delta_files"], 3);
}

// =========================================================================
// 3. Diff with removed language → shows removal
// =========================================================================

#[test]
fn diff_removed_language_appears_with_negative_delta() {
    let dir = tempdir().unwrap();
    let before = write_fixture(
        dir.path(),
        "before.json",
        &lang_receipt(&[
            ("Rust", 200, 260, 4, 10000, 500),
            ("Shell", 30, 40, 2, 1500, 75),
        ]),
    );
    let after = write_fixture(
        dir.path(),
        "after.json",
        &lang_receipt(&[("Rust", 200, 260, 4, 10000, 500)]),
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
    let shell_row = rows
        .iter()
        .find(|r| r["lang"] == "Shell")
        .expect("Shell row");
    assert_eq!(shell_row["new_code"], 0);
    assert_eq!(shell_row["delta_code"], -30);
    assert_eq!(shell_row["delta_files"], -2);
}

#[test]
fn diff_removed_language_totals_reflect_removal() {
    let dir = tempdir().unwrap();
    let before = write_fixture(
        dir.path(),
        "before.json",
        &lang_receipt(&[
            ("Rust", 100, 120, 2, 5000, 250),
            ("TOML", 20, 25, 1, 800, 50),
        ]),
    );
    let after = write_fixture(
        dir.path(),
        "after.json",
        &lang_receipt(&[("Rust", 100, 120, 2, 5000, 250)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["totals"]["delta_code"], -20);
    assert_eq!(json["totals"]["delta_files"], -1);
}

// =========================================================================
// 4. Diff with changed counts → shows delta
// =========================================================================

#[test]
fn diff_changed_counts_positive_growth() {
    let dir = tempdir().unwrap();
    let before = write_fixture(
        dir.path(),
        "before.json",
        &lang_receipt(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let after = write_fixture(
        dir.path(),
        "after.json",
        &lang_receipt(&[("Rust", 350, 450, 7, 17500, 875)]),
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

    let row = &json["diff_rows"][0];
    assert_eq!(row["delta_code"], 250);
    assert_eq!(row["delta_lines"], 330);
    assert_eq!(row["delta_files"], 5);
    assert_eq!(row["delta_bytes"], 12500);
    assert_eq!(row["delta_tokens"], 625);
}

#[test]
fn diff_changed_counts_negative_shrink() {
    let dir = tempdir().unwrap();
    let before = write_fixture(
        dir.path(),
        "before.json",
        &lang_receipt(&[("Python", 400, 520, 8, 20000, 1000)]),
    );
    let after = write_fixture(
        dir.path(),
        "after.json",
        &lang_receipt(&[("Python", 250, 320, 5, 12500, 625)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let row = &json["diff_rows"][0];
    assert_eq!(row["delta_code"], -150);
    assert_eq!(row["delta_files"], -3);
}

#[test]
fn diff_mixed_changes_across_languages() {
    let dir = tempdir().unwrap();
    let before = write_fixture(
        dir.path(),
        "before.json",
        &lang_receipt(&[
            ("Rust", 200, 260, 4, 10000, 500),
            ("Python", 100, 130, 3, 5000, 250),
        ]),
    );
    let after = write_fixture(
        dir.path(),
        "after.json",
        &lang_receipt(&[
            ("Rust", 300, 390, 6, 15000, 750),
            ("Python", 60, 78, 2, 3000, 150),
        ]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&before)
        .arg("--to")
        .arg(&after)
        .output()
        .unwrap();

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();

    let rows = json["diff_rows"].as_array().unwrap();
    let rust = rows.iter().find(|r| r["lang"] == "Rust").unwrap();
    assert_eq!(rust["delta_code"], 100);

    let py = rows.iter().find(|r| r["lang"] == "Python").unwrap();
    assert_eq!(py["delta_code"], -40);

    // net: +100 - 40 = +60
    assert_eq!(json["totals"]["delta_code"], 60);
}

// =========================================================================
// 5. Diff output format: JSON
// =========================================================================

#[test]
fn diff_json_output_has_required_envelope_fields() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("Rust", 50, 60, 1, 2500, 125)]),
    );
    let b = write_fixture(
        dir.path(),
        "b.json",
        &lang_receipt(&[("Rust", 70, 85, 2, 3500, 175)]),
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

    assert!(json["schema_version"].is_number());
    assert!(json["generated_at_ms"].is_number());
    assert_eq!(json["mode"], "diff");
    assert!(json["from_source"].is_string());
    assert!(json["to_source"].is_string());
    assert!(json["tool"]["name"].is_string());
    assert!(json["tool"]["version"].is_string());
    assert!(json["diff_rows"].is_array());
    assert!(json["totals"].is_object());
}

#[test]
fn diff_json_diff_row_fields_complete() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("C", 1000, 1200, 20, 50000, 2500)]),
    );
    let b = write_fixture(
        dir.path(),
        "b.json",
        &lang_receipt(&[("C", 1100, 1350, 22, 55000, 2750)]),
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

    let expected_fields = [
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
    ];
    for field in expected_fields {
        assert!(row.get(field).is_some(), "missing field: {field}");
    }
}

// =========================================================================
// 6. Diff output format: Markdown
// =========================================================================

#[test]
fn diff_md_output_contains_header_and_table() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let b = write_fixture(
        dir.path(),
        "b.json",
        &lang_receipt(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    tokmd()
        .args(["diff", "--format", "md", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn diff_md_compact_produces_summary_table() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let b = write_fixture(
        dir.path(),
        "b.json",
        &lang_receipt(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    tokmd()
        .args(["diff", "--compact", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .assert()
        .success()
        .stdout(predicate::str::contains("|Metric|Value|"));
}

#[test]
fn diff_md_color_never_has_no_ansi() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("Rust", 100, 120, 2, 5000, 250)]),
    );
    let b = write_fixture(
        dir.path(),
        "b.json",
        &lang_receipt(&[("Rust", 200, 240, 3, 10000, 500)]),
    );

    let output = tokmd()
        .args(["diff", "--color", "never", "--from"])
        .arg(&a)
        .arg("--to")
        .arg(&b)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // ANSI escape sequences start with ESC (0x1B)
    assert!(
        !stdout.contains('\x1b'),
        "output should have no ANSI escapes with --color never"
    );
}

// =========================================================================
// 7. Diff from files: load two receipt JSON files and diff
// =========================================================================

#[test]
fn diff_from_files_via_positional_args() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("JavaScript", 300, 400, 5, 15000, 750)]),
    );
    let b = write_fixture(
        dir.path(),
        "b.json",
        &lang_receipt(&[("JavaScript", 350, 460, 6, 17500, 875)]),
    );

    let output = tokmd()
        .args(["diff", "--format", "json"])
        .arg(&a)
        .arg(&b)
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["totals"]["delta_code"], 50);
}

#[test]
fn diff_from_directory_loads_lang_json() {
    let dir = tempdir().unwrap();

    let run_dir = dir.path().join("run_a");
    fs::create_dir_all(&run_dir).unwrap();
    fs::write(
        run_dir.join("lang.json"),
        lang_receipt(&[("Rust", 100, 120, 2, 5000, 250)]),
    )
    .unwrap();

    let run_dir_b = dir.path().join("run_b");
    fs::create_dir_all(&run_dir_b).unwrap();
    fs::write(
        run_dir_b.join("lang.json"),
        lang_receipt(&[("Rust", 200, 240, 3, 10000, 500)]),
    )
    .unwrap();

    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&run_dir)
        .arg("--to")
        .arg(&run_dir_b)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["totals"]["delta_code"], 100);
}

// =========================================================================
// 8. Diff from runs: run tokmd twice on different dirs and diff
// =========================================================================

#[test]
fn diff_from_live_runs() {
    let dir = tempdir().unwrap();

    // Create two minimal source trees
    let src_a = dir.path().join("src_a");
    fs::create_dir_all(src_a.join(".git")).unwrap();
    fs::write(src_a.join("main.rs"), "fn main() {}\n").unwrap();

    let src_b = dir.path().join("src_b");
    fs::create_dir_all(src_b.join(".git")).unwrap();
    fs::write(
        src_b.join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\nfn other() {}\n",
    )
    .unwrap();

    // Run tokmd on each
    let out_a = dir.path().join("out_a");
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(&src_a)
        .args(["run", "--output-dir"])
        .arg(&out_a)
        .arg(".")
        .assert()
        .success();

    let out_b = dir.path().join("out_b");
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(&src_b)
        .args(["run", "--output-dir"])
        .arg(&out_b)
        .arg(".")
        .assert()
        .success();

    // Diff the two runs
    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&out_a)
        .arg("--to")
        .arg(&out_b)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["diff_rows"].is_array());
    // src_b has more code than src_a
    assert!(json["totals"]["delta_code"].as_i64().unwrap() > 0);
}

// =========================================================================
// 9. Error handling: diff with invalid input
// =========================================================================

#[test]
fn diff_missing_both_args_fails() {
    tokmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--from").or(predicate::str::contains("refs/paths")));
}

#[test]
fn diff_only_from_no_to_fails() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("Rust", 10, 12, 1, 500, 25)]),
    );

    tokmd().args(["diff", "--from"]).arg(&a).assert().failure();
}

#[test]
fn diff_nonexistent_file_fails() {
    tokmd()
        .args([
            "diff",
            "--from",
            "/nonexistent/path.json",
            "--to",
            "/also/missing.json",
        ])
        .assert()
        .failure();
}

#[test]
fn diff_invalid_json_file_fails() {
    let dir = tempdir().unwrap();
    let bad = write_fixture(dir.path(), "bad.json", "this is not json");
    let good = write_fixture(
        dir.path(),
        "good.json",
        &lang_receipt(&[("Rust", 10, 12, 1, 500, 25)]),
    );

    tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(&bad)
        .arg("--to")
        .arg(&good)
        .assert()
        .failure();
}

// =========================================================================
// 10. Diff schema version matches expected
// =========================================================================

#[test]
fn diff_schema_version_matches_types_constant() {
    let dir = tempdir().unwrap();
    let a = write_fixture(
        dir.path(),
        "a.json",
        &lang_receipt(&[("Rust", 10, 12, 1, 500, 25)]),
    );
    let b = write_fixture(
        dir.path(),
        "b.json",
        &lang_receipt(&[("Rust", 20, 24, 1, 1000, 50)]),
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
    assert_eq!(
        json["schema_version"].as_u64().unwrap(),
        u64::from(tokmd_types::SCHEMA_VERSION),
        "diff schema_version should match SCHEMA_VERSION constant"
    );
}
