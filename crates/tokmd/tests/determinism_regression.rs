//! Determinism regression tests v2.
//!
//! This module is the definitive regression suite for output determinism.
//! It verifies byte-stability, path normalization, timestamp isolation,
//! and sorting invariants across all receipt-producing commands including
//! `analyze`.
//!
//! Run with: `cargo test -p tokmd --test determinism_regression`

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Normalize non-deterministic fields (timestamps, tool version) for
/// byte-level comparison of serialized output.
fn normalize_envelope(output: &str) -> String {
    // Handle both compact ("key":val) and pretty-printed ("key": val) JSON
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

/// Recursively zero out `generated_at_ms` in a JSON value tree so that
/// structural equality comparison ignores only the timestamp.
fn zero_timestamps(v: &mut Value) {
    match v {
        Value::Object(map) => {
            if let Some(ts) = map.get_mut("generated_at_ms") {
                *ts = Value::Number(0.into());
            }
            if let Some(ts) = map.get_mut("export_generated_at_ms") {
                *ts = Value::Number(0.into());
            }
            for val in map.values_mut() {
                zero_timestamps(val);
            }
        }
        Value::Array(arr) => arr.iter_mut().for_each(zero_timestamps),
        _ => {}
    }
}

// ===========================================================================
// 1. Ordering determinism: byte-identical output across runs
// ===========================================================================

#[test]
fn lang_json_byte_identical_across_runs() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(
        a, b,
        "lang --format json must be byte-identical across runs"
    );
}

#[test]
fn module_json_byte_identical_across_runs() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(
        a, b,
        "module --format json must be byte-identical across runs"
    );
}

#[test]
fn export_jsonl_byte_identical_across_runs() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(
        a, b,
        "export --format jsonl must be byte-identical across runs"
    );
}

#[test]
fn analyze_receipt_json_byte_identical_across_runs() {
    let run = || {
        let o = tokmd_cmd()
            .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
            .output()
            .expect("run");
        assert!(
            o.status.success(),
            "analyze failed: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(
        a, b,
        "analyze --preset receipt must be byte-identical across runs"
    );
}

// ===========================================================================
// 2. Path normalization: no backslashes in any path field
// ===========================================================================

#[test]
fn export_json_no_backslash_in_path_or_module() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");

    for (i, row) in rows.iter().enumerate() {
        let path = row["path"].as_str().expect("row field should be a string");
        assert!(!path.contains('\\'), "row[{i}].path has backslash: {path}");
        let module = row["module"].as_str().expect("row field should be a string");
        assert!(
            !module.contains('\\'),
            "row[{i}].module has backslash: {module}"
        );
    }
}

#[test]
fn module_json_no_backslash_in_module_keys() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");

    for (i, row) in rows.iter().enumerate() {
        let module = row["module"].as_str().expect("row field should be a string");
        assert!(
            !module.contains('\\'),
            "row[{i}].module has backslash: {module}"
        );
    }
}

/// Collect all string values from a JSON tree with their key paths.
fn collect_path_like_strings(v: &Value, out: &mut Vec<(String, String)>, prefix: &str) {
    match v {
        Value::String(s) => out.push((prefix.to_string(), s.clone())),
        Value::Object(map) => {
            for (k, val) in map {
                collect_path_like_strings(val, out, &format!("{prefix}.{k}"));
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                collect_path_like_strings(val, out, &format!("{prefix}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn analyze_json_no_backslash_in_path_fields() {
    let o = tokmd_cmd()
        .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

    let mut strings = Vec::new();
    collect_path_like_strings(&json, &mut strings, "");
    for (key, val) in &strings {
        if key.contains("path") || key.contains("root") || key.contains("dir") {
            assert!(
                !val.contains('\\'),
                "path-like field {key} contains backslash: {val}"
            );
        }
    }
}

#[test]
fn lang_json_no_backslash_in_path_fields() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

    let mut strings = Vec::new();
    collect_path_like_strings(&json, &mut strings, "");
    for (key, val) in &strings {
        if key.contains("path") || key.contains("root") || key.contains("dir") {
            assert!(
                !val.contains('\\'),
                "path-like field {key} contains backslash: {val}"
            );
        }
    }
}

// ===========================================================================
// 3. Timestamp stability: generated_at_ms is the ONLY varying field
// ===========================================================================

#[test]
fn lang_timestamp_is_only_nondeterministic_field() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        serde_json::from_slice::<Value>(&o.stdout).expect("valid JSON")
    };
    let mut a = run();
    let mut b = run();

    assert!(
        a["generated_at_ms"].is_number(),
        "generated_at_ms must exist"
    );
    assert!(
        b["generated_at_ms"].is_number(),
        "generated_at_ms must exist"
    );

    zero_timestamps(&mut a);
    zero_timestamps(&mut b);
    assert_eq!(a, b, "only generated_at_ms should differ between lang runs");
}

#[test]
fn module_timestamp_is_only_nondeterministic_field() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        serde_json::from_slice::<Value>(&o.stdout).expect("valid JSON")
    };
    let mut a = run();
    let mut b = run();

    assert!(a["generated_at_ms"].is_number());
    assert!(b["generated_at_ms"].is_number());

    zero_timestamps(&mut a);
    zero_timestamps(&mut b);
    assert_eq!(
        a, b,
        "only generated_at_ms should differ between module runs"
    );
}

#[test]
fn export_timestamp_is_only_nondeterministic_field() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        serde_json::from_slice::<Value>(&o.stdout).expect("valid JSON")
    };
    let mut a = run();
    let mut b = run();

    assert!(a["generated_at_ms"].is_number());
    assert!(b["generated_at_ms"].is_number());

    zero_timestamps(&mut a);
    zero_timestamps(&mut b);
    assert_eq!(
        a, b,
        "only generated_at_ms should differ between export runs"
    );
}

#[test]
fn analyze_timestamp_is_only_nondeterministic_field() {
    let run = || {
        let o = tokmd_cmd()
            .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        serde_json::from_slice::<Value>(&o.stdout).expect("valid JSON")
    };
    let mut a = run();
    let mut b = run();

    assert!(a["generated_at_ms"].is_number());
    assert!(b["generated_at_ms"].is_number());

    zero_timestamps(&mut a);
    zero_timestamps(&mut b);
    assert_eq!(
        a, b,
        "only generated_at_ms should differ between analyze runs"
    );
}

// ===========================================================================
// 4. Sorting order invariants
// ===========================================================================

#[test]
fn lang_rows_descending_code_ascending_name() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().expect("code row should be a u64 number");
        let b_code = pair[1]["code"].as_u64().expect("code row should be a u64 number");
        let a_name = pair[0]["lang"].as_str().expect("row field should be a string");
        let b_name = pair[1]["lang"].as_str().expect("row field should be a string");

        assert!(
            a_code > b_code || (a_code == b_code && a_name <= b_name),
            "lang sort violated: {a_name}({a_code}) before {b_name}({b_code})"
        );
    }
}

#[test]
fn module_rows_descending_code_ascending_module() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().expect("code row should be a u64 number");
        let b_code = pair[1]["code"].as_u64().expect("code row should be a u64 number");
        let a_mod = pair[0]["module"].as_str().expect("row field should be a string");
        let b_mod = pair[1]["module"].as_str().expect("row field should be a string");

        assert!(
            a_code > b_code || (a_code == b_code && a_mod <= b_mod),
            "module sort violated: {a_mod}({a_code}) before {b_mod}({b_code})"
        );
    }
}

#[test]
fn export_rows_descending_code_ascending_path() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().expect("code row should be a u64 number");
        let b_code = pair[1]["code"].as_u64().expect("code row should be a u64 number");
        let a_path = pair[0]["path"].as_str().expect("row field should be a string");
        let b_path = pair[1]["path"].as_str().expect("row field should be a string");

        assert!(
            a_code > b_code || (a_code == b_code && a_path <= b_path),
            "export sort violated: {a_path}({a_code}) before {b_path}({b_code})"
        );
    }
}

#[test]
fn analyze_derived_keys_stable_and_sorted() {
    let get_keys = || {
        let o = tokmd_cmd()
            .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        let derived = json
            .get("derived")
            .and_then(|v| v.as_object())
            .expect("derived section");
        derived.keys().cloned().collect::<Vec<_>>()
    };

    let keys1 = get_keys();
    let keys2 = get_keys();
    assert_eq!(
        keys1, keys2,
        "analyze derived keys must be stable across runs"
    );

    // BTreeMap guarantee: keys are alphabetically sorted
    let mut sorted = keys1.clone();
    sorted.sort();
    assert_eq!(keys1, sorted, "derived keys must be alphabetically sorted");
}

// ===========================================================================
// 5. Multi-format byte stability
// ===========================================================================

#[test]
fn analyze_receipt_markdown_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["analyze", ".", "--preset", "receipt", "--format", "md"])
            .output()
            .expect("run");
        assert!(o.status.success());
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(run(), run(), "analyze markdown must be deterministic");
}

#[test]
fn lang_csv_byte_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "csv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(run(), run(), "lang CSV must be byte-identical");
}

#[test]
fn module_tsv_byte_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "tsv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(run(), run(), "module TSV must be byte-identical");
}

#[test]
fn export_csv_byte_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "csv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    assert_eq!(run(), run(), "export CSV must be byte-identical");
}

// ===========================================================================
// 6. Structural stability across runs
// ===========================================================================

#[test]
fn row_counts_stable_across_all_commands() {
    let count = |args: &[&str]| -> usize {
        let o = tokmd_cmd().args(args).output().expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        json["rows"].as_array().map(|a| a.len()).unwrap_or(0)
    };

    let lang1 = count(&["lang", "--format", "json"]);
    let lang2 = count(&["lang", "--format", "json"]);
    assert_eq!(lang1, lang2, "lang row count unstable");
    assert!(lang1 > 0, "lang should have at least one row");

    let mod1 = count(&["module", "--format", "json"]);
    let mod2 = count(&["module", "--format", "json"]);
    assert_eq!(mod1, mod2, "module row count unstable");
    assert!(mod1 > 0, "module should have at least one row");

    let exp1 = count(&["export", "--format", "json"]);
    let exp2 = count(&["export", "--format", "json"]);
    assert_eq!(exp1, exp2, "export row count unstable");
    assert!(exp1 > 0, "export should have at least one row");
}

#[test]
fn export_jsonl_line_count_stable() {
    let count = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).lines().count()
    };
    let a = count();
    let b = count();
    assert_eq!(a, b, "export JSONL line count unstable");
    assert!(a > 1, "JSONL should have meta + data lines");
}

#[test]
fn json_keys_alphabetically_sorted_in_receipt_rows() {
    let commands: &[&[&str]] = &[
        &["lang", "--format", "json"],
        &["module", "--format", "json"],
        &["export", "--format", "json"],
    ];

    for cmd in commands {
        let o = tokmd_cmd().args(*cmd).output().expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

        if let Some(rows) = json["rows"].as_array() {
            for (i, row) in rows.iter().enumerate() {
                if let Some(map) = row.as_object() {
                    let keys: Vec<&String> = map.keys().collect();
                    let mut sorted = keys.clone();
                    sorted.sort();
                    assert_eq!(
                        keys, sorted,
                        "{cmd:?} row[{i}] keys not alphabetically sorted: {keys:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn analyze_receipt_has_stable_top_level_keys() {
    let get_top_keys = || {
        let o = tokmd_cmd()
            .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        let map = json.as_object().expect("top-level object");
        map.keys().cloned().collect::<Vec<_>>()
    };

    let keys1 = get_top_keys();
    let keys2 = get_top_keys();
    assert_eq!(keys1, keys2, "analyze top-level keys must be stable");

    // BTreeMap serialization: top-level keys are alphabetically sorted
    let mut sorted = keys1.clone();
    sorted.sort();
    assert_eq!(
        keys1, sorted,
        "analyze top-level keys must be alphabetically sorted"
    );
}

// ===========================================================================
// 7. Module key determinism
// ===========================================================================

#[test]
fn module_keys_deterministic_across_runs() {
    let get_modules = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        json["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(|r| r["module"].as_str().expect("row field should be a string").to_string())
            .collect::<Vec<_>>()
    };

    let mods1 = get_modules();
    let mods2 = get_modules();
    assert_eq!(mods1, mods2, "module keys must be identical across runs");
    assert!(!mods1.is_empty(), "should have at least one module");
}

#[test]
fn export_module_keys_match_module_command() {
    let module_mods = {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        let mut mods: Vec<String> = json["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(|r| r["module"].as_str().expect("row field should be a string").to_string())
            .collect();
        mods.sort();
        mods.dedup();
        mods
    };

    let export_mods = {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        let mut mods: Vec<String> = json["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(|r| r["module"].as_str().expect("row field should be a string").to_string())
            .collect();
        mods.sort();
        mods.dedup();
        mods
    };

    // Every module in export should appear in module command output
    for m in &export_mods {
        assert!(
            module_mods.contains(m),
            "export module key '{m}' not found in module command output"
        );
    }
}
