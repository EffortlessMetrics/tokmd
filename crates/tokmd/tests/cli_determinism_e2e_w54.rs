#![cfg(feature = "analysis")]
//! Determinism end-to-end tests verifying that repeated CLI invocations
//! produce identical output.  Timestamps and tool versions are normalized
//! before comparison so that only true non-determinism triggers failures.

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Normalize non-deterministic envelope fields (timestamps, tool version).
fn normalize_envelope(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\s*\d+"#).expect("valid regex");
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();
    let re_ver = regex::Regex::new(r#"("tool":\s*\{"name":\s*"tokmd",\s*"version":\s*")[^"]+"#)
        .expect("valid regex");
    re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string()
}

/// Run a command twice and assert outputs are identical (after normalization).
fn assert_deterministic(args: &[&str], label: &str) {
    let run = || {
        let o = tokmd_cmd().args(args).output().expect("run");
        assert!(o.status.success(), "{label} failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "{label} must be byte-stable across runs");
}

/// Run a command three times and assert all outputs are identical.
fn assert_deterministic_triple(args: &[&str], label: &str) {
    let run = || {
        let o = tokmd_cmd().args(args).output().expect("run");
        assert!(o.status.success(), "{label} failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    let c = run();
    assert_eq!(a, b, "{label} must be stable (run 1 vs 2)");
    assert_eq!(b, c, "{label} must be stable (run 2 vs 3)");
}

// ===========================================================================
// 1. Format stability: repeated runs produce identical bytes
// ===========================================================================

#[test]
fn lang_json_deterministic() {
    assert_deterministic_triple(&["lang", "--format", "json"], "lang JSON");
}

#[test]
fn lang_md_deterministic() {
    assert_deterministic(&["lang", "--format", "md"], "lang Markdown");
}

#[test]
fn lang_tsv_deterministic() {
    assert_deterministic(&["lang", "--format", "tsv"], "lang TSV");
}

#[test]
fn module_json_deterministic() {
    assert_deterministic_triple(&["module", "--format", "json"], "module JSON");
}

#[test]
fn module_md_deterministic() {
    assert_deterministic(&["module", "--format", "md"], "module Markdown");
}

#[test]
fn module_tsv_deterministic() {
    assert_deterministic(&["module", "--format", "tsv"], "module TSV");
}

#[test]
fn export_jsonl_deterministic() {
    assert_deterministic(&["export", "--format", "jsonl"], "export JSONL");
}

#[test]
fn export_csv_deterministic() {
    assert_deterministic(&["export", "--format", "csv"], "export CSV");
}

#[test]
fn export_json_deterministic() {
    assert_deterministic_triple(&["export", "--format", "json"], "export JSON");
}

#[test]
fn analyze_receipt_json_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["analyze", "--preset", "receipt", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "analyze receipt JSON failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "analyze receipt JSON must be byte-stable across runs");
}

#[test]
fn run_default_deterministic() {
    // `run` writes artifacts to a directory; verify it always succeeds
    let run = || {
        let o = tokmd_cmd().arg("run").output().expect("run");
        o.status.success()
    };
    assert!(run(), "run must succeed (first)");
    assert!(run(), "run must succeed (second)");
}

// ===========================================================================
// 2. JSON key ordering: BTreeMap guarantees sorted keys
// ===========================================================================

#[test]
fn analyze_json_all_keys_recursively_sorted() {
    let o = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_json_keys_sorted(&json, "root");
}

fn assert_json_keys_sorted(json: &Value, context: &str) {
    if let Some(obj) = json.as_object() {
        let keys: Vec<&String> = obj.keys().collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "keys not sorted in {context}");
        for (k, v) in obj {
            assert_json_keys_sorted(v, &format!("{context}.{k}"));
        }
    } else if let Some(arr) = json.as_array() {
        for (i, v) in arr.iter().enumerate() {
            assert_json_keys_sorted(v, &format!("{context}[{i}]"));
        }
    }
}

#[test]
fn lang_json_keys_are_sorted() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    if let Some(rows) = json.get("rows").and_then(|v| v.as_array()) {
        for row in rows {
            if let Some(map) = row.as_object() {
                let keys: Vec<&String> = map.keys().collect();
                let mut sorted = keys.clone();
                sorted.sort();
                assert_eq!(keys, sorted, "lang row keys must be sorted");
            }
        }
    }
}

#[test]
fn module_json_keys_are_sorted() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    if let Some(rows) = json.get("rows").and_then(|v| v.as_array()) {
        for row in rows {
            if let Some(map) = row.as_object() {
                let keys: Vec<&String> = map.keys().collect();
                let mut sorted = keys.clone();
                sorted.sort();
                assert_eq!(keys, sorted, "module row keys must be sorted");
            }
        }
    }
}

#[test]
fn export_json_keys_are_sorted() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    if let Some(rows) = json.get("rows").and_then(|v| v.as_array()) {
        for row in rows {
            if let Some(map) = row.as_object() {
                let keys: Vec<&String> = map.keys().collect();
                let mut sorted = keys.clone();
                sorted.sort();
                assert_eq!(keys, sorted, "export row keys must be sorted");
            }
        }
    }
}

#[test]
fn analyze_json_top_level_keys_sorted() {
    let o = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    if let Some(obj) = json.as_object() {
        let keys: Vec<&String> = obj.keys().collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "analyze top-level keys must be sorted");
    }
}

// ===========================================================================
// 3. Ordering invariants: descending by code, then by name
// ===========================================================================

#[test]
fn lang_rows_sorted_desc_code_asc_name() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_lang = pair[0]["lang"].as_str().unwrap();
        let b_lang = pair[1]["lang"].as_str().unwrap();
        assert!(
            a_code > b_code || (a_code == b_code && a_lang <= b_lang),
            "lang sort violated: {a_lang}({a_code}) before {b_lang}({b_code})"
        );
    }
}

#[test]
fn module_rows_sorted_desc_code_asc_name() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_mod = pair[0]["module"].as_str().unwrap();
        let b_mod = pair[1]["module"].as_str().unwrap();
        assert!(
            a_code > b_code || (a_code == b_code && a_mod <= b_mod),
            "module sort violated: {a_mod}({a_code}) before {b_mod}({b_code})"
        );
    }
}

#[test]
fn export_rows_sorted_desc_code_asc_path() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
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
// 4. Row count stability
// ===========================================================================

#[test]
fn lang_row_count_stable() {
    let count = || -> usize {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["rows"].as_array().unwrap().len()
    };
    assert_eq!(count(), count(), "lang row count must be stable");
}

#[test]
fn module_row_count_stable() {
    let count = || -> usize {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["rows"].as_array().unwrap().len()
    };
    assert_eq!(count(), count(), "module row count must be stable");
}

#[test]
fn export_row_count_stable() {
    let count = || -> usize {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["rows"].as_array().unwrap().len()
    };
    assert_eq!(count(), count(), "export row count must be stable");
}

// ===========================================================================
// 5. Totals consistency
// ===========================================================================

#[test]
fn lang_totals_equal_sum_of_rows() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    let total = &json["total"];

    let sum_code: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap()).sum();
    let sum_lines: u64 = rows.iter().map(|r| r["lines"].as_u64().unwrap()).sum();
    let sum_files: u64 = rows.iter().map(|r| r["files"].as_u64().unwrap()).sum();

    assert_eq!(sum_code, total["code"].as_u64().unwrap(), "code total");
    assert_eq!(sum_lines, total["lines"].as_u64().unwrap(), "lines total");
    assert_eq!(sum_files, total["files"].as_u64().unwrap(), "files total");
}

// ===========================================================================
// 6. Timestamp is the only varying field
// ===========================================================================

#[test]
fn lang_json_only_timestamps_vary() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();

    // Before normalization they may differ
    let a_norm = normalize_envelope(&a);
    let b_norm = normalize_envelope(&b);
    assert_eq!(
        a_norm, b_norm,
        "after normalizing timestamps and version, outputs must be identical"
    );
}

#[test]
fn export_json_only_timestamps_vary() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    let a_norm = normalize_envelope(&a);
    let b_norm = normalize_envelope(&b);
    assert_eq!(
        a_norm, b_norm,
        "after normalizing timestamps, export outputs must be identical"
    );
}

#[test]
fn module_json_only_timestamps_vary() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    let a_norm = normalize_envelope(&a);
    let b_norm = normalize_envelope(&b);
    assert_eq!(
        a_norm, b_norm,
        "after normalizing timestamps, module outputs must be identical"
    );
}

// ===========================================================================
// 7. Redaction determinism
// ===========================================================================

#[test]
fn redacted_export_deterministic() {
    assert_deterministic(
        &["export", "--format", "json", "--redact", "paths"],
        "redacted export",
    );
}

// ===========================================================================
// 8. Children mode determinism
// ===========================================================================

#[test]
fn children_collapse_deterministic() {
    assert_deterministic(
        &["lang", "--format", "json", "--children", "collapse"],
        "children collapse",
    );
}

#[test]
fn children_separate_deterministic() {
    assert_deterministic(
        &["lang", "--format", "json", "--children", "separate"],
        "children separate",
    );
}
