#![cfg(feature = "analysis")]
//! Comprehensive determinism verification tests (wave 51).
//!
//! These tests ensure byte-stable output across runs, correct path
//! normalization, deterministic sort ordering, schema version stability,
//! and metadata consistency for every major output format.
//!
//! Run with: `cargo test -p tokmd --test determinism_hardening_w51`

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Normalize non-deterministic envelope fields (timestamps, tool version)
/// so byte-level comparison is meaningful.
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
    let s = re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string();
    let re_elapsed = regex::Regex::new(r#""elapsed_ms":\s*\d+"#).expect("valid regex");
    re_elapsed.replace_all(&s, r#""elapsed_ms":0"#).to_string()
}

/// Normalize Markdown/TSV non-deterministic lines (timestamps, versions).
fn normalize_text(output: &str) -> String {
    let re_ts = regex::Regex::new(r"(?m)^.*generated.*$").expect("valid regex");
    let s = re_ts.replace_all(output, "TIMESTAMP_LINE").to_string();
    let re_ver = regex::Regex::new(r"tokmd\s+\d+\.\d+\.\d+[^\s]*").expect("valid regex");
    re_ver.replace_all(&s, "tokmd 0.0.0").to_string()
}

// ===========================================================================
// 1. Output stability tests — repeated runs produce identical bytes
// ===========================================================================

#[test]
fn w51_lang_json_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "lang --format json failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "lang JSON must be byte-stable across runs");
}

#[test]
fn w51_module_json_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "module --format json failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "module JSON must be byte-stable across runs");
}

#[test]
fn w51_export_json_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "export --format json failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "export JSON must be byte-stable across runs");
}

#[test]
fn w51_analyze_receipt_json_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "analyze --preset receipt failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "analyze receipt JSON must be byte-stable across runs");
}

#[test]
fn w51_lang_md_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "md"])
            .output()
            .expect("run");
        assert!(o.status.success(), "lang --format md failed");
        normalize_text(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "lang Markdown must be byte-stable across runs");
}

#[test]
fn w51_lang_tsv_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "tsv"])
            .output()
            .expect("run");
        assert!(o.status.success(), "lang --format tsv failed");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "lang TSV must be byte-stable across runs");
}

#[test]
fn w51_export_csv_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "csv"])
            .output()
            .expect("run");
        assert!(o.status.success(), "export --format csv failed");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "export CSV must be byte-stable across runs");
}

#[test]
fn w51_export_jsonl_output_stable() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        assert!(o.status.success(), "export --format jsonl failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "output must not be empty");
    assert_eq!(a, b, "export JSONL must be byte-stable across runs");
}

// ===========================================================================
// 2. Path normalization tests — no backslashes anywhere in output
// ===========================================================================

#[test]
fn w51_json_output_contains_no_backslashes_in_paths() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let stdout = String::from_utf8_lossy(&o.stdout);
    // Scan the entire JSON string for backslash-letter patterns typical of
    // Windows paths (e.g., `\\src`, `\\main`). Escaped JSON chars like `\n`
    // or `\"` are not path separators.
    let re = regex::Regex::new(r#"\\[a-zA-Z]"#).expect("valid regex");
    for m in re.find_iter(&stdout) {
        // Allow JSON escape sequences
        let prev_char = stdout[..m.start()].chars().last().unwrap_or(' ');
        if prev_char != '\\' {
            panic!(
                "JSON output contains likely Windows path backslash near: ...{}...",
                &stdout[m.start().saturating_sub(20)..stdout.len().min(m.end() + 20)]
            );
        }
    }
}

#[test]
fn w51_module_keys_use_forward_slashes() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");

    for row in rows {
        let module = row["module"].as_str().expect("module field");
        assert!(
            !module.contains('\\'),
            "module key contains backslash: {module}"
        );
    }
}

#[test]
fn w51_export_file_paths_use_forward_slashes() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");

    for row in rows {
        let path = row["path"].as_str().expect("path field");
        assert!(
            !path.contains('\\'),
            "export path contains backslash: {path}"
        );
    }
}

#[test]
fn w51_analyze_paths_use_forward_slashes() {
    let o = tokmd_cmd()
        .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

    // Check source path
    if let Some(source) = json.get("source").and_then(|s| s.as_str()) {
        assert!(
            !source.contains('\\'),
            "analyze source path has backslash: {source}"
        );
    }

    // Recursively check all string values for backslash path separators
    fn check_no_backslash_paths(v: &Value, path: &str) {
        match v {
            Value::String(s) => {
                // Skip version strings and hash values
                if !path.contains("version")
                    && !path.contains("hash")
                    && !path.contains("blake3")
                    && !path.contains("integrity")
                    && s.contains('\\')
                    && s.contains(std::path::MAIN_SEPARATOR)
                {
                    // Only flag if it looks like a file path
                    if s.contains(".rs") || s.contains(".js") || s.contains(".md") {
                        panic!("backslash in path-like string at {path}: {s}");
                    }
                }
            }
            Value::Object(map) => {
                for (k, val) in map {
                    check_no_backslash_paths(val, &format!("{path}.{k}"));
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    check_no_backslash_paths(val, &format!("{path}[{i}]"));
                }
            }
            _ => {}
        }
    }
    check_no_backslash_paths(&json, "$");
}

#[test]
fn w51_export_paths_sorted_lexicographically_on_forward_slash() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");

    // Within same code-line bucket, paths must be in ascending lex order
    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_path = pair[0]["path"].as_str().unwrap();
        let b_path = pair[1]["path"].as_str().unwrap();

        if a_code == b_code {
            assert!(
                a_path <= b_path,
                "within same code bucket, paths must be lex-sorted: \
                 {a_path} should come before {b_path}"
            );
        }
        // All paths should use forward slashes for this comparison to work
        assert!(!a_path.contains('\\'), "path has backslash: {a_path}");
    }
}

// ===========================================================================
// 3. Sort order verification
// ===========================================================================

#[test]
fn w51_lang_rows_sorted_by_code_desc() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");
    assert!(!rows.is_empty(), "lang must produce at least one row");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        assert!(
            a_code >= b_code,
            "lang rows not sorted by code descending: {} < {}",
            a_code,
            b_code
        );
    }
}

#[test]
fn w51_lang_rows_tiebreak_by_name_asc() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        if a_code == b_code {
            let a_name = pair[0]["lang"].as_str().unwrap();
            let b_name = pair[1]["lang"].as_str().unwrap();
            assert!(
                a_name <= b_name,
                "lang tie-break must be ascending by name: {a_name} > {b_name}"
            );
        }
    }
}

#[test]
fn w51_module_rows_sorted_by_code_desc() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");
    assert!(!rows.is_empty(), "module must produce at least one row");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        assert!(
            a_code >= b_code,
            "module rows not sorted by code descending: {} < {}",
            a_code,
            b_code
        );
    }
}

#[test]
fn w51_export_rows_deterministic_order() {
    let o = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");
    assert!(!rows.is_empty(), "export must produce at least one row");

    // Export rows must be sorted by code desc, then path asc
    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_path = pair[0]["path"].as_str().unwrap();
        let b_path = pair[1]["path"].as_str().unwrap();

        assert!(
            a_code > b_code || (a_code == b_code && a_path <= b_path),
            "export rows must be sorted desc by code, asc by path: \
             {a_path}({a_code}) vs {b_path}({b_code})"
        );
    }
}

#[test]
fn w51_lang_totals_equal_row_sums() {
    let o = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

    let rows = json["rows"].as_array().expect("rows array");
    let total = &json["total"];

    let sum_code: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap()).sum();
    let sum_lines: u64 = rows.iter().map(|r| r["lines"].as_u64().unwrap()).sum();
    let sum_files: u64 = rows.iter().map(|r| r["files"].as_u64().unwrap()).sum();

    assert_eq!(
        sum_code,
        total["code"].as_u64().unwrap(),
        "code total mismatch"
    );
    assert_eq!(
        sum_lines,
        total["lines"].as_u64().unwrap(),
        "lines total mismatch"
    );
    assert_eq!(
        sum_files,
        total["files"].as_u64().unwrap(),
        "files total mismatch"
    );
}

// ===========================================================================
// 4. Schema version stability
// ===========================================================================

#[test]
fn w51_core_receipts_schema_version_is_2() {
    for (cmd, mode_label) in [
        (vec!["lang", "--format", "json"], "lang"),
        (vec!["module", "--format", "json"], "module"),
        (vec!["export", "--format", "json"], "export"),
    ] {
        let o = tokmd_cmd().args(&cmd).output().expect("run");
        assert!(o.status.success(), "{mode_label} command failed");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        assert_eq!(
            json["schema_version"], 2,
            "{mode_label} schema_version must be 2"
        );
    }
}

#[test]
fn w51_analyze_schema_version_is_9() {
    let o = tokmd_cmd()
        .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success(), "analyze command failed");
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert_eq!(
        json["schema_version"], 9,
        "analysis schema_version must be 9"
    );
}

#[test]
fn w51_schema_version_is_number_not_string() {
    for (cmd, label) in [
        (vec!["lang", "--format", "json"], "lang"),
        (vec!["module", "--format", "json"], "module"),
        (vec!["export", "--format", "json"], "export"),
        (
            vec!["analyze", ".", "--preset", "receipt", "--format", "json"],
            "analyze",
        ),
    ] {
        let o = tokmd_cmd().args(&cmd).output().expect("run");
        assert!(o.status.success(), "{label} command failed");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        assert!(
            json["schema_version"].is_number(),
            "{label} schema_version must be a number, got: {}",
            json["schema_version"]
        );
        assert!(
            !json["schema_version"].is_string(),
            "{label} schema_version must not be a string"
        );
    }
}

// ===========================================================================
// 5. Metadata stability
// ===========================================================================

#[test]
fn w51_json_output_has_args_metadata() {
    for (cmd, label) in [
        (vec!["lang", "--format", "json"], "lang"),
        (vec!["module", "--format", "json"], "module"),
        (vec!["export", "--format", "json"], "export"),
    ] {
        let o = tokmd_cmd().args(&cmd).output().expect("run");
        assert!(o.status.success(), "{label} command failed");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        assert!(
            json.get("args").is_some(),
            "{label} must have args metadata"
        );
        assert!(json["args"].is_object(), "{label} args must be an object");
    }
}

#[test]
fn w51_timestamp_is_consistent_format() {
    for (cmd, label) in [
        (vec!["lang", "--format", "json"], "lang"),
        (vec!["module", "--format", "json"], "module"),
        (vec!["export", "--format", "json"], "export"),
    ] {
        let o = tokmd_cmd().args(&cmd).output().expect("run");
        assert!(o.status.success(), "{label} command failed");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

        let ts = json
            .get("generated_at_ms")
            .unwrap_or_else(|| panic!("{label} missing generated_at_ms"));
        assert!(
            ts.is_number(),
            "{label} generated_at_ms must be a number, got: {ts}"
        );
        let ms = ts.as_u64().expect("timestamp as u64");
        // Sanity: timestamp should be after 2020-01-01 in ms
        assert!(
            ms > 1_577_836_800_000,
            "{label} generated_at_ms looks invalid: {ms}"
        );
    }
}

#[test]
fn w51_tool_version_present_and_nonempty() {
    for (cmd, label) in [
        (vec!["lang", "--format", "json"], "lang"),
        (vec!["module", "--format", "json"], "module"),
        (vec!["export", "--format", "json"], "export"),
    ] {
        let o = tokmd_cmd().args(&cmd).output().expect("run");
        assert!(o.status.success(), "{label} command failed");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");

        let tool = json
            .get("tool")
            .unwrap_or_else(|| panic!("{label} missing tool"));
        assert!(tool.is_object(), "{label} tool must be an object");

        let name = tool
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{label} missing tool.name"));
        assert_eq!(name, "tokmd", "{label} tool.name must be 'tokmd'");

        let version = tool
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{label} missing tool.version"));
        assert!(
            !version.is_empty(),
            "{label} tool.version must not be empty"
        );
        // Version should look like semver (N.N.N)
        assert!(
            version.contains('.'),
            "{label} tool.version doesn't look like semver: {version}"
        );
    }
}

// ===========================================================================
// 6. Additional determinism hardening
// ===========================================================================

#[test]
fn w51_export_csv_column_count_stable() {
    let o = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let stdout = String::from_utf8_lossy(&o.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "CSV must have header + at least one row");

    let header_cols = lines[0].split(',').count();
    for (i, line) in lines.iter().enumerate().skip(1) {
        let cols = line.split(',').count();
        assert_eq!(
            cols, header_cols,
            "CSV row {i} has {cols} columns, expected {header_cols}"
        );
    }
}

#[test]
fn w51_export_jsonl_every_line_is_valid_json() {
    let o = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let stdout = String::from_utf8_lossy(&o.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(!lines.is_empty(), "JSONL output must not be empty");

    for (i, line) in lines.iter().enumerate() {
        assert!(
            serde_json::from_str::<Value>(line).is_ok(),
            "JSONL line {i} is not valid JSON: {line}"
        );
    }
}

#[test]
fn w51_module_tiebreak_by_name_asc() {
    let o = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let rows = json["rows"].as_array().expect("rows array");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        if a_code == b_code {
            let a_mod = pair[0]["module"].as_str().unwrap();
            let b_mod = pair[1]["module"].as_str().unwrap();
            assert!(
                a_mod <= b_mod,
                "module tie-break must be ascending by name: {a_mod} > {b_mod}"
            );
        }
    }
}

#[test]
fn w51_json_keys_are_sorted_in_all_commands() {
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

    for (cmd, label) in [
        (vec!["lang", "--format", "json"], "lang"),
        (vec!["module", "--format", "json"], "module"),
        (vec!["export", "--format", "json"], "export"),
    ] {
        let o = tokmd_cmd().args(&cmd).output().expect("run");
        assert!(o.status.success(), "{label} command failed");
        let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        assert_keys_sorted(&json, &format!("${label}"));
    }
}

#[test]
fn w51_analyze_mode_field_is_analysis() {
    let o = tokmd_cmd()
        .args(["analyze", ".", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    assert!(o.status.success());
    let json: Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    assert_eq!(
        json["mode"], "analysis",
        "analyze mode field must be 'analysis'"
    );
}

#[test]
fn w51_export_row_count_matches_across_formats() {
    // JSON
    let json_out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(json_out.status.success());
    let json: Value = serde_json::from_slice(&json_out.stdout).expect("parse JSON");
    let json_rows = json["rows"].as_array().expect("rows array").len();

    // CSV (minus header)
    let csv_out = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    assert!(csv_out.status.success());
    let csv_text = String::from_utf8_lossy(&csv_out.stdout);
    let csv_rows = csv_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
        .saturating_sub(1);

    assert_eq!(
        json_rows, csv_rows,
        "JSON rows ({json_rows}) must match CSV rows ({csv_rows})"
    );
}
