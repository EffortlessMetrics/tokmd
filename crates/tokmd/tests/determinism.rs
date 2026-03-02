//! End-to-end determinism hardening tests.
//!
//! These tests verify the core invariant: same input yields byte-identical output
//! across repeated runs for every output format.

mod common;

use assert_cmd::Command;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Normalize non-deterministic fields (timestamps, tool version) so we can
/// compare outputs byte-for-byte.
fn normalize_envelope(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\d+"#).expect("valid regex");
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();
    let re_ver =
        regex::Regex::new(r#"("tool":\{"name":"tokmd","version":")[^"]+"#).expect("valid regex");
    re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string()
}

// ---------------------------------------------------------------------------
// 1. Format stability: repeated runs produce identical bytes
// ---------------------------------------------------------------------------

#[test]
fn lang_json_is_deterministic() {
    let out = |_| {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = out(0);
    let b = out(1);
    assert_eq!(a, b, "lang JSON must be byte-stable across runs");
}

#[test]
fn lang_md_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "md"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(
        run(),
        run(),
        "lang Markdown must be byte-stable across runs"
    );
}

#[test]
fn lang_tsv_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "tsv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(run(), run(), "lang TSV must be byte-stable across runs");
}

#[test]
fn module_json_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    assert_eq!(run(), run(), "module JSON must be byte-stable across runs");
}

#[test]
fn module_md_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "md"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(
        run(),
        run(),
        "module Markdown must be byte-stable across runs"
    );
}

#[test]
fn module_tsv_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "tsv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(run(), run(), "module TSV must be byte-stable across runs");
}

#[test]
fn export_jsonl_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    assert_eq!(run(), run(), "export JSONL must be byte-stable across runs");
}

#[test]
fn export_csv_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "csv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(run(), run(), "export CSV must be byte-stable across runs");
}

#[test]
fn export_json_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    assert_eq!(run(), run(), "export JSON must be byte-stable across runs");
}

// ---------------------------------------------------------------------------
// 2. BTreeMap ordering: JSON keys are always sorted
// ---------------------------------------------------------------------------

#[test]
fn lang_json_keys_are_sorted() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

    // Verify any nested maps in row objects have sorted keys.
    if let Some(rows) = json.get("rows").and_then(|v| v.as_array()) {
        for row in rows {
            if let Some(map) = row.as_object() {
                let row_keys: Vec<&String> = map.keys().collect();
                let mut row_sorted = row_keys.clone();
                row_sorted.sort();
                assert_eq!(
                    row_keys, row_sorted,
                    "row keys must be alphabetically sorted"
                );
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
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
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

// ---------------------------------------------------------------------------
// 3. Sorting invariants: descending by code, then by name
// ---------------------------------------------------------------------------

#[test]
fn lang_rows_sorted_by_code_desc_then_name_asc() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_lang = pair[0]["lang"].as_str().unwrap();
        let b_lang = pair[1]["lang"].as_str().unwrap();

        assert!(
            a_code > b_code || (a_code == b_code && a_lang <= b_lang),
            "lang rows must be sorted desc by code, asc by name: \
             {a_lang}({a_code}) should come before {b_lang}({b_code})"
        );
    }
}

#[test]
fn module_rows_sorted_by_code_desc_then_name_asc() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_mod = pair[0]["module"].as_str().unwrap();
        let b_mod = pair[1]["module"].as_str().unwrap();

        assert!(
            a_code > b_code || (a_code == b_code && a_mod <= b_mod),
            "module rows must be sorted desc by code, asc by name: \
             {a_mod}({a_code}) should come before {b_mod}({b_code})"
        );
    }
}

#[test]
fn export_rows_sorted_by_code_desc_then_path_asc() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_path = pair[0]["path"].as_str().unwrap();
        let b_path = pair[1]["path"].as_str().unwrap();

        assert!(
            a_code > b_code || (a_code == b_code && a_path <= b_path),
            "export rows must be sorted desc by code, asc by path: \
             {a_path}({a_code}) should come before {b_path}({b_code})"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Receipt envelope determinism
// ---------------------------------------------------------------------------

#[test]
fn lang_receipt_has_required_envelope_fields() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert!(
        json.get("schema_version").is_some(),
        "missing schema_version"
    );
    assert!(
        json.get("generated_at_ms").is_some(),
        "missing generated_at_ms"
    );
    assert!(json.get("tool").is_some(), "missing tool");
    assert!(json.get("mode").is_some(), "missing mode");
    assert_eq!(json["mode"], "lang");
    assert_eq!(json["schema_version"], 2);
}

#[test]
fn module_receipt_has_required_envelope_fields() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert!(
        json.get("schema_version").is_some(),
        "missing schema_version"
    );
    assert_eq!(json["mode"], "module");
    assert_eq!(json["schema_version"], 2);
}

#[test]
fn export_json_receipt_has_required_envelope_fields() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert!(
        json.get("schema_version").is_some(),
        "missing schema_version"
    );
    assert_eq!(json["mode"], "export");
    assert_eq!(json["schema_version"], 2);
}

// ---------------------------------------------------------------------------
// 5. Path normalization in output: no backslashes
// ---------------------------------------------------------------------------

#[test]
fn export_paths_use_forward_slashes() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");

    for row in rows {
        let path = row["path"].as_str().unwrap();
        assert!(!path.contains('\\'), "path contains backslash: {path}");
    }
}

#[test]
fn export_modules_use_forward_slashes() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");

    for row in rows {
        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('\\'),
            "module key contains backslash: {module}"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Schema version stability
// ---------------------------------------------------------------------------

#[test]
fn schema_version_constants_match_expected() {
    assert_eq!(tokmd_types::SCHEMA_VERSION, 2, "core schema version");
    assert_eq!(
        tokmd_types::HANDOFF_SCHEMA_VERSION,
        5,
        "handoff schema version"
    );
    assert_eq!(
        tokmd_types::CONTEXT_BUNDLE_SCHEMA_VERSION,
        2,
        "context bundle schema version"
    );
    assert_eq!(
        tokmd_types::CONTEXT_SCHEMA_VERSION,
        4,
        "context schema version"
    );
    assert_eq!(
        tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION,
        3,
        "cockpit schema version"
    );
    assert_eq!(
        tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        8,
        "analysis schema version"
    );
}

// ---------------------------------------------------------------------------
// 7. Multiple runs produce same row count
// ---------------------------------------------------------------------------

#[test]
fn lang_row_count_is_stable() {
    let count = || -> usize {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        json.get("rows")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
    };
    assert_eq!(count(), count(), "lang row count must be stable");
}

#[test]
fn export_row_count_is_stable() {
    let count = || -> usize {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        json.get("rows")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
    };
    assert_eq!(count(), count(), "export row count must be stable");
}

// ---------------------------------------------------------------------------
// 8. Redaction determinism via CLI
// ---------------------------------------------------------------------------

#[test]
fn redacted_export_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json", "--redact", "paths"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    assert_eq!(
        run(),
        run(),
        "redacted export JSON must be byte-stable across runs"
    );
}

#[test]
fn redacted_paths_are_hashed_not_plaintext() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json", "--redact", "paths"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json
        .get("rows")
        .and_then(|v| v.as_array())
        .expect("rows array");

    for row in rows {
        let path = row["path"].as_str().unwrap();
        // Redacted paths should not contain directory separators
        assert!(
            !path.contains('/') || path.starts_with('('),
            "redacted path looks un-redacted: {path}"
        );
    }
}

// ---------------------------------------------------------------------------
// 9. JSONL meta record consistency
// ---------------------------------------------------------------------------

#[test]
fn export_jsonl_meta_record_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
        let first_line = stdout.lines().next().unwrap_or("").to_string();
        normalize_envelope(&first_line)
    };
    assert_eq!(
        run(),
        run(),
        "JSONL meta record must be byte-stable across runs"
    );
}

// ---------------------------------------------------------------------------
// 10. Totals consistency: totals match sum of rows
// ---------------------------------------------------------------------------

#[test]
fn lang_totals_match_row_sums() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

    let rows = json["rows"].as_array().expect("rows array");
    let total = &json["total"];

    let sum_code: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap()).sum();
    let sum_lines: u64 = rows.iter().map(|r| r["lines"].as_u64().unwrap()).sum();
    let sum_files: u64 = rows.iter().map(|r| r["files"].as_u64().unwrap()).sum();

    assert_eq!(sum_code, total["code"].as_u64().unwrap(), "code total");
    assert_eq!(sum_lines, total["lines"].as_u64().unwrap(), "lines total");
    assert_eq!(sum_files, total["files"].as_u64().unwrap(), "files total");
}
