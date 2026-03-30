//! Determinism regression suite – wave 40.
//!
//! Verifies byte-stable output across repeated runs, sort invariants,
//! path normalization, and structural stability for all receipt-producing
//! commands.
//!
//! Run with: `cargo test -p tokmd --test determinism_w40`

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Normalize non-deterministic envelope fields for byte-level comparison.
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

// ===========================================================================
// 1. Five-run byte-stability tests
// ===========================================================================

#[test]
fn lang_json_five_runs_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let runs: Vec<String> = (0..5).map(|_| run()).collect();
    for i in 1..5 {
        assert_eq!(
            runs[0], runs[i],
            "lang JSON run 0 vs {i} must be byte-identical"
        );
    }
}

#[test]
fn module_json_five_runs_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let runs: Vec<String> = (0..5).map(|_| run()).collect();
    for i in 1..5 {
        assert_eq!(
            runs[0], runs[i],
            "module JSON run 0 vs {i} must be byte-identical"
        );
    }
}

#[test]
fn export_json_five_runs_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let runs: Vec<String> = (0..5).map(|_| run()).collect();
    for i in 1..5 {
        assert_eq!(
            runs[0], runs[i],
            "export JSON run 0 vs {i} must be byte-identical"
        );
    }
}

// ===========================================================================
// 2. JSON key sorting (BTreeMap guarantee) – top-level envelope
// ===========================================================================

#[test]
fn lang_json_top_level_keys_sorted() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let map = json.as_object().expect("top-level object");
    let keys: Vec<&String> = map.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(
        keys, sorted,
        "lang receipt top-level keys must be alphabetically sorted"
    );
}

#[test]
fn module_json_top_level_keys_sorted() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let map = json.as_object().expect("top-level object");
    let keys: Vec<&String> = map.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(
        keys, sorted,
        "module receipt top-level keys must be alphabetically sorted"
    );
}

#[test]
fn export_json_top_level_keys_sorted() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let map = json.as_object().expect("top-level object");
    let keys: Vec<&String> = map.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(
        keys, sorted,
        "export receipt top-level keys must be alphabetically sorted"
    );
}

// ===========================================================================
// 3. Row sort invariants
// ===========================================================================

#[test]
fn lang_rows_sorted_desc_code_asc_name() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");
    assert!(!rows.is_empty(), "must have at least one row");

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
fn module_rows_sorted_consistently() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");
    assert!(!rows.is_empty(), "must have at least one row");

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
fn file_rows_sorted_consistently() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");
    assert!(!rows.is_empty(), "must have at least one row");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().expect("code row should be a u64 number");
        let b_code = pair[1]["code"].as_u64().expect("code row should be a u64 number");
        let a_path = pair[0]["path"].as_str().expect("row field should be a string");
        let b_path = pair[1]["path"].as_str().expect("row field should be a string");
        assert!(
            a_code > b_code || (a_code == b_code && a_path <= b_path),
            "file sort violated: {a_path}({a_code}) before {b_path}({b_code})"
        );
    }
}

// ===========================================================================
// 4. Path normalization – forward slashes everywhere
// ===========================================================================

#[test]
fn all_paths_use_forward_slashes_in_export() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows");

    for (i, row) in rows.iter().enumerate() {
        let path = row["path"].as_str().expect("row field should be a string");
        assert!(
            !path.contains('\\'),
            "row[{i}].path contains backslash: {path}"
        );
        let module = row["module"].as_str().expect("row field should be a string");
        assert!(
            !module.contains('\\'),
            "row[{i}].module contains backslash: {module}"
        );
    }
}

#[test]
fn no_os_path_separators_in_markdown_output() {
    let o = tokmd_cmd()
        .args(["module", "--format", "md"])
        .output()
        .expect("run");
    let stdout = String::from_utf8_lossy(&o.stdout);

    // On Windows the OS separator is backslash; module keys must use forward slashes.
    for line in stdout.lines() {
        if line.starts_with('|') && !line.contains("---") {
            assert!(
                !line.contains('\\'),
                "Markdown output contains backslash: {line}"
            );
        }
    }
}

#[test]
fn no_os_path_separators_in_tsv_output() {
    let o = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("run");
    let stdout = String::from_utf8_lossy(&o.stdout);

    for line in stdout.lines() {
        let first_col = line.split('\t').next().unwrap_or("");
        assert!(
            !first_col.contains('\\'),
            "TSV module column contains backslash: {first_col}"
        );
    }
}

#[test]
fn no_os_path_separators_in_csv_output() {
    let o = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    let stdout = String::from_utf8_lossy(&o.stdout);

    for (i, line) in stdout.lines().enumerate() {
        if i == 0 {
            continue; // skip header
        }
        assert!(
            !line.contains('\\'),
            "CSV row {i} contains backslash: {line}"
        );
    }
}

// ===========================================================================
// 5. Timestamp normalization consistency
// ===========================================================================

#[test]
fn timestamps_are_only_nondeterministic_field_lang() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        serde_json::from_slice::<Value>(&o.stdout).expect("valid JSON")
    };
    let mut a = run();
    let mut b = run();

    // Timestamps should differ (or at least exist)
    assert!(a["generated_at_ms"].is_number());
    assert!(b["generated_at_ms"].is_number());

    // Zero out timestamps
    if let Some(map) = a.as_object_mut() {
        map.insert("generated_at_ms".to_string(), Value::Number(0.into()));
    }
    if let Some(map) = b.as_object_mut() {
        map.insert("generated_at_ms".to_string(), Value::Number(0.into()));
    }
    assert_eq!(a, b, "only generated_at_ms should differ between lang runs");
}

// ===========================================================================
// 6. Structural stability – row and field counts
// ===========================================================================

#[test]
fn lang_names_deterministic_across_runs() {
    let get_names = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        json["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(|r| r["lang"].as_str().expect("row field should be a string").to_string())
            .collect::<Vec<_>>()
    };
    let a = get_names();
    let b = get_names();
    assert_eq!(a, b, "language names must be identical across runs");
    assert!(!a.is_empty(), "should have at least one language");
}

#[test]
fn export_file_kinds_deterministic() {
    let get_kinds = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        json["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(|r| r["kind"].as_str().expect("row field should be a string").to_string())
            .collect::<Vec<_>>()
    };
    let a = get_kinds();
    let b = get_kinds();
    assert_eq!(a, b, "file kinds must be identical across runs");
}

#[test]
fn export_csv_header_deterministic() {
    let get_header = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "csv"])
            .output()
            .expect("run");
        let stdout = String::from_utf8_lossy(&o.stdout);
        stdout.lines().next().unwrap_or("").to_string()
    };
    let a = get_header();
    let b = get_header();
    assert_eq!(a, b, "CSV header must be identical across runs");
    assert!(a.contains("path"), "CSV header must contain 'path' column");
}

#[test]
fn export_jsonl_line_order_deterministic() {
    let get_lines = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        let stdout = String::from_utf8_lossy(&o.stdout);
        stdout
            .lines()
            .skip(1) // skip meta record
            .map(|line| {
                let v: Value = serde_json::from_str(line).expect("valid JSON");
                v["path"].as_str().unwrap_or("").to_string()
            })
            .collect::<Vec<_>>()
    };
    let a = get_lines();
    let b = get_lines();
    assert_eq!(a, b, "JSONL row order must be identical across runs");
}

#[test]
fn top_truncation_is_deterministic() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json", "--top", "1"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    let c = run();
    assert_eq!(a, b, "--top 1 must be deterministic (1 vs 2)");
    assert_eq!(b, c, "--top 1 must be deterministic (2 vs 3)");
}
