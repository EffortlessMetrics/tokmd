//! Determinism hardening integration tests (wave 45).
//!
//! Complements `determinism.rs` and `determinism_regression.rs` with focused
//! hardening scenarios: multi-run byte stability across 5 consecutive runs,
//! cross-format row-count consistency, recursive JSON key ordering, silent
//! truncation detection, sort-order validation, path normalization in every
//! format, and redaction determinism through the CLI.
//!
//! Run with: `cargo test -p tokmd --test determinism_hardening`

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Normalize non-deterministic envelope fields so that byte-level comparison
/// is meaningful.
fn normalize_envelope(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\s*\d+"#).expect("valid regex");
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();
    let re_export_ts =
        regex::Regex::new(r#""export_generated_at_ms":\s*\d+"#).expect("valid regex");
    let s = re_export_ts
        .replace_all(&s, r#""export_generated_at_ms":0"#)
        .to_string();
    let re_ver = regex::Regex::new(r#"("tool":\s*\{\s*"name":\s*"tokmd",\s*"version":\s*")[^"]+"#)
        .expect("valid regex");
    re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string()
}

/// Recursively verify all JSON object keys are in alphabetical order.
fn assert_keys_sorted(v: &Value, path: &str) {
    match v {
        Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            for pair in keys.windows(2) {
                assert!(
                    pair[0] <= pair[1],
                    "JSON keys not sorted at {path}: {:?} > {:?}",
                    pair[0],
                    pair[1]
                );
            }
            for (k, val) in map {
                assert_keys_sorted(val, &format!("{path}.{k}"));
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                assert_keys_sorted(val, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

// ===========================================================================
// 1. Five-run byte stability — every JSON-producing command must be identical
// ===========================================================================

#[test]
fn hardening_five_run_lang_json_byte_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let results: Vec<String> = (0..5).map(|_| run()).collect();
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(&results[0], r, "lang JSON run 0 vs run {i} differ");
    }
}

#[test]
fn hardening_five_run_module_json_byte_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let results: Vec<String> = (0..5).map(|_| run()).collect();
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(&results[0], r, "module JSON run 0 vs run {i} differ");
    }
}

#[test]
fn hardening_five_run_export_json_byte_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let results: Vec<String> = (0..5).map(|_| run()).collect();
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(&results[0], r, "export JSON run 0 vs run {i} differ");
    }
}

// ===========================================================================
// 2. Cross-format row-count consistency (JSON vs JSONL vs CSV)
// ===========================================================================

#[test]
fn hardening_export_row_counts_agree_across_formats() {
    // JSON
    let json_out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&json_out.stdout).expect("parse JSON");
    let json_rows = json["rows"].as_array().expect("rows array").len();

    // JSONL — first line is meta envelope, remaining lines are data rows
    let jsonl_out = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    let jsonl_text = String::from_utf8_lossy(&jsonl_out.stdout);
    let jsonl_rows = jsonl_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| !l.contains(r#""type":"meta""#))
        .count();

    // CSV — first line is header
    let csv_out = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    let csv_text = String::from_utf8_lossy(&csv_out.stdout);
    let csv_rows = csv_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
        .saturating_sub(1);

    assert!(json_rows > 0, "expected at least one export row");
    assert_eq!(
        json_rows, jsonl_rows,
        "JSON row count ({json_rows}) != JSONL row count ({jsonl_rows})"
    );
    assert_eq!(
        json_rows, csv_rows,
        "JSON row count ({json_rows}) != CSV row count ({csv_rows})"
    );
}

// ===========================================================================
// 3. Recursive JSON key ordering (BTreeMap invariant)
// ===========================================================================

#[test]
fn hardening_lang_json_keys_recursively_sorted() {
    let out = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    assert_keys_sorted(&v, "$");
}

#[test]
fn hardening_module_json_keys_recursively_sorted() {
    let out = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    assert_keys_sorted(&v, "$");
}

#[test]
fn hardening_export_json_keys_recursively_sorted() {
    let out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    assert_keys_sorted(&v, "$");
}

// ===========================================================================
// 4. No silent truncation — row count stable across runs
// ===========================================================================

#[test]
fn hardening_export_no_silent_truncation() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        let v: Value = serde_json::from_slice(&o.stdout).expect("parse");
        v["rows"].as_array().expect("rows").len()
    };
    let counts: Vec<usize> = (0..3).map(|_| run()).collect();
    assert!(counts[0] > 0, "expected at least one row");
    assert_eq!(
        counts[0], counts[1],
        "row count drifted between run 0 and 1"
    );
    assert_eq!(
        counts[1], counts[2],
        "row count drifted between run 1 and 2"
    );
}

// ===========================================================================
// 5. Sort ordering validation
// ===========================================================================

#[test]
fn hardening_lang_rows_sorted_desc_code_then_asc_name() {
    let out = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_name = pair[0]["lang"].as_str().unwrap();
        let b_name = pair[1]["lang"].as_str().unwrap();

        assert!(
            a_code > b_code || (a_code == b_code && a_name <= b_name),
            "lang sort violated: {a_name}({a_code}) before {b_name}({b_code})"
        );
    }
}

#[test]
fn hardening_module_rows_sorted_desc_code_then_asc_name() {
    let out = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_name = pair[0]["module"].as_str().unwrap();
        let b_name = pair[1]["module"].as_str().unwrap();

        assert!(
            a_code > b_code || (a_code == b_code && a_name <= b_name),
            "module sort violated: {a_name}({a_code}) before {b_name}({b_code})"
        );
    }
}

#[test]
fn hardening_export_rows_sorted_desc_code_then_asc_path() {
    let out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_path = pair[0]["path"].as_str().unwrap();
        let b_path = pair[1]["path"].as_str().unwrap();

        assert!(
            a_code > b_code || (a_code == b_code && a_path <= b_path),
            "export sort violated: {a_path}({a_code}) before {b_path}({b_code})"
        );
    }
}

// ===========================================================================
// 6. Path normalization — no backslashes in any output format
// ===========================================================================

#[test]
fn hardening_export_json_no_backslash_in_paths() {
    let out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows");

    for row in rows {
        let path = row["path"].as_str().unwrap();
        assert!(!path.contains('\\'), "backslash in export path: {path}");

        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('\\'),
            "backslash in export module: {module}"
        );
    }
}

#[test]
fn hardening_module_json_no_backslash_in_modules() {
    let out = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows");

    for row in rows {
        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('\\'),
            "backslash in module name: {module}"
        );
    }
}

#[test]
fn hardening_export_tsv_no_backslash_in_paths() {
    let out = tokmd_cmd()
        .args(["export", "--format", "tsv"])
        .output()
        .expect("run");
    let text = String::from_utf8_lossy(&out.stdout);

    for (i, line) in text.lines().enumerate().skip(1) {
        // First column is the path
        if let Some(path) = line.split('\t').next() {
            assert!(
                !path.contains('\\'),
                "backslash in TSV path at line {i}: {path}"
            );
        }
    }
}

#[test]
fn hardening_export_csv_no_backslash_in_paths() {
    let out = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    let text = String::from_utf8_lossy(&out.stdout);

    for (i, line) in text.lines().enumerate().skip(1) {
        // First column is the path (may be quoted)
        if let Some(path) = line.split(',').next() {
            let path = path.trim_matches('"');
            assert!(
                !path.contains('\\'),
                "backslash in CSV path at line {i}: {path}"
            );
        }
    }
}

// ===========================================================================
// 7. Redaction determinism via CLI
// ===========================================================================

#[test]
fn hardening_redact_produces_deterministic_output() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json", "--redact", "paths"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    let c = run();
    assert_eq!(a, b, "redacted export run 1 vs 2 differ");
    assert_eq!(b, c, "redacted export run 2 vs 3 differ");

    // Verify paths are actually redacted (hashed, not plaintext)
    let v: Value = serde_json::from_str(&a).expect("parse");
    let rows = v["rows"].as_array().expect("rows");
    assert!(!rows.is_empty(), "redacted export must have rows");
    for row in rows {
        let path = row["path"].as_str().unwrap();
        // Redacted paths are hex hashes, not readable source paths
        assert!(
            !path.starts_with("src/") && !path.starts_with("script"),
            "path appears unredacted: {path}"
        );
    }
}
