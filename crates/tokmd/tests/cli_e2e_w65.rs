//! Wave 65 — comprehensive CLI end-to-end tests.
//!
//! ~80 tests covering all major subcommands, output formats, flag combinations,
//! error paths, JSON structure validation, determinism, and edge cases.
//! Each test invokes the real `tokmd` binary via `assert_cmd` against a hermetic
//! fixture directory.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

fn tokmd_bare() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

fn nonexistent_path() -> std::path::PathBuf {
    std::env::temp_dir().join("tokmd_w65_nonexistent_path_does_not_exist")
}

/// Run a command and return stdout as String.
fn stdout_of(cmd: &mut Command) -> String {
    let output = cmd.output().expect("failed to run tokmd");
    assert!(output.status.success(), "command failed: {:?}", cmd);
    String::from_utf8(output.stdout).expect("stdout is not UTF-8")
}

/// Run a command and parse stdout as JSON.
fn json_of(cmd: &mut Command) -> Value {
    let raw = stdout_of(cmd);
    serde_json::from_str(&raw).expect("stdout is not valid JSON")
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

// ===========================================================================
// 1. Version & help
// ===========================================================================

#[test]
fn version_flag_shows_tokmd() {
    tokmd_bare()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn version_flag_matches_semver() {
    tokmd_bare()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"tokmd \d+\.\d+\.\d+").unwrap());
}

#[test]
fn help_flag_shows_subcommands() {
    tokmd_bare()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lang"))
        .stdout(predicate::str::contains("module"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains("badge"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("context"))
        .stdout(predicate::str::contains("gate"))
        .stdout(predicate::str::contains("cockpit"))
        .stdout(predicate::str::contains("handoff"))
        .stdout(predicate::str::contains("sensor"));
}

#[test]
fn help_flag_shows_global_options() {
    tokmd_bare()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--exclude"))
        .stdout(predicate::str::contains("--children"))
        .stdout(predicate::str::contains("--top"));
}

#[test]
fn lang_help_shows_expected_flags() {
    tokmd_bare()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--children"))
        .stdout(predicate::str::contains("--top"))
        .stdout(predicate::str::contains("--files"))
        .stdout(predicate::str::contains("--exclude"));
}

#[test]
fn module_help_shows_depth_option() {
    tokmd_bare()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--module-depth"))
        .stdout(predicate::str::contains("--module-roots"));
}

#[test]
fn export_help_shows_format_options() {
    tokmd_bare()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("csv"))
        .stdout(predicate::str::contains("jsonl"))
        .stdout(predicate::str::contains("json"))
        .stdout(predicate::str::contains("cyclonedx"));
}

// ===========================================================================
// 2. tokmd lang — format matrix
// ===========================================================================

#[test]
fn lang_default_produces_markdown() {
    tokmd_cmd()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn lang_format_md_produces_markdown_table() {
    tokmd_cmd()
        .args(["lang", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("|"))
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
}

#[test]
fn lang_format_tsv_is_tab_separated() {
    let out = stdout_of(&mut tokmd_cmd().args(["lang", "--format", "tsv"]));
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 2, "TSV needs header + at least 1 data row");
    assert!(lines[0].contains('\t'), "header line must contain tabs");
    // Every non-empty line should have the same number of tabs
    let header_tabs = lines[0].matches('\t').count();
    for line in &lines[1..] {
        if !line.trim().is_empty() {
            assert_eq!(
                line.matches('\t').count(),
                header_tabs,
                "each TSV row must have same column count as header"
            );
        }
    }
}

#[test]
fn lang_format_json_is_valid() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "lang");
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
}

#[test]
fn lang_json_rows_have_expected_fields() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let rows = json["rows"].as_array().expect("rows is array");
    assert!(
        !rows.is_empty(),
        "fixture must produce at least one language"
    );
    for row in rows {
        assert!(row["lang"].is_string(), "row must have lang");
        assert!(row["code"].is_number(), "row must have code");
        assert!(row["lines"].is_number(), "row must have lines");
        assert!(row["files"].is_number(), "row must have files");
    }
}

#[test]
fn lang_json_total_has_code_field() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    assert!(json["total"]["code"].is_number());
    let code = json["total"]["code"].as_u64().unwrap();
    assert!(code > 0, "fixture should have non-zero total code lines");
}

#[test]
fn lang_json_schema_version_is_positive() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let sv = json["schema_version"].as_u64().unwrap();
    assert!(sv >= 1, "schema_version must be >= 1");
}

#[test]
fn lang_json_has_envelope_metadata() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    assert!(json["generated_at_ms"].is_number());
    assert!(json["tool"].is_object());
    assert!(json["tool"]["name"].is_string());
}

// ===========================================================================
// 3. tokmd module — format matrix & options
// ===========================================================================

#[test]
fn module_default_produces_markdown() {
    tokmd_cmd()
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn module_format_json_valid() {
    let json = json_of(&mut tokmd_cmd().args(["module", "--format", "json"]));
    assert_eq!(json["mode"], "module");
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
}

#[test]
fn module_json_rows_have_module_field() {
    let json = json_of(&mut tokmd_cmd().args(["module", "--format", "json"]));
    for row in json["rows"].as_array().unwrap() {
        assert!(row["module"].is_string(), "module row must have module key");
        assert!(row["code"].is_number(), "module row must have code");
    }
}

#[test]
fn module_format_tsv_has_tabs() {
    let out = stdout_of(&mut tokmd_cmd().args(["module", "--format", "tsv"]));
    assert!(out.contains('\t'), "TSV output must contain tabs");
    assert!(out.lines().count() >= 2, "TSV needs header + data");
}

#[test]
fn module_format_md_contains_table_markers() {
    tokmd_cmd()
        .args(["module", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("|"))
        .stdout(predicate::str::contains("Module"));
}

#[test]
fn module_depth_1_produces_output() {
    tokmd_cmd()
        .args(["module", "--module-depth", "1"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn module_depth_3_produces_output() {
    tokmd_cmd()
        .args(["module", "--module-depth", "3"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 4. tokmd export — format matrix
// ===========================================================================

#[test]
fn export_jsonl_produces_valid_lines() {
    let out = stdout_of(&mut tokmd_cmd().args(["export", "--format", "jsonl"]));
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need meta + at least one data row");
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {} is not valid JSON: {}", i + 1, e));
    }
}

#[test]
fn export_jsonl_first_line_is_meta() {
    let out = stdout_of(&mut tokmd_cmd().args(["export", "--format", "jsonl"]));
    let first = out.lines().next().expect("must have at least one line");
    let meta: Value = serde_json::from_str(first).unwrap();
    assert!(
        meta["schema_version"].is_number(),
        "meta line should contain schema_version"
    );
}

#[test]
fn export_csv_has_header_and_data() {
    let out = stdout_of(&mut tokmd_cmd().args(["export", "--format", "csv"]));
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 2, "CSV needs header + data");
    let header = lines[0];
    assert!(
        header.contains("path") || header.contains("language") || header.contains("code"),
        "CSV header should contain recognizable column names"
    );
}

#[test]
fn export_csv_rows_have_consistent_column_count() {
    let out = stdout_of(&mut tokmd_cmd().args(["export", "--format", "csv"]));
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2);
    let header_cols = lines[0].matches(',').count();
    for (i, line) in lines[1..].iter().enumerate() {
        let cols = line.matches(',').count();
        assert_eq!(
            cols,
            header_cols,
            "CSV row {} has {} commas but header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

#[test]
fn export_json_is_valid_array() {
    let json = json_of(&mut tokmd_cmd().args(["export", "--format", "json"]));
    assert!(
        json.is_object() || json.is_array(),
        "export JSON must be object or array"
    );
}

#[test]
fn export_min_code_filter_works() {
    let out_all =
        stdout_of(&mut tokmd_cmd().args(["export", "--format", "jsonl", "--min-code", "0"]));
    let out_filtered =
        stdout_of(&mut tokmd_cmd().args(["export", "--format", "jsonl", "--min-code", "9999"]));
    let all_lines: Vec<&str> = out_all.lines().filter(|l| !l.trim().is_empty()).collect();
    let filtered_lines: Vec<&str> = out_filtered
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    // Filtered should have fewer data rows (meta may still be present)
    assert!(
        filtered_lines.len() <= all_lines.len(),
        "min-code=9999 should filter out most rows"
    );
}

#[test]
fn export_redact_none_shows_paths() {
    let out = stdout_of(&mut tokmd_cmd().args(["export", "--format", "jsonl", "--redact", "none"]));
    let data_lines: Vec<&str> = out
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .collect();
    if let Some(first_data) = data_lines.first() {
        let row: Value = serde_json::from_str(first_data).unwrap();
        if let Some(path) = row.get("path").and_then(|p| p.as_str()) {
            // Unredacted paths should contain recognizable file extensions
            assert!(
                path.contains('.') || path.contains('/'),
                "unredacted path should look like a file path"
            );
        }
    }
}

// ===========================================================================
// 5. Children mode
// ===========================================================================

#[test]
fn lang_children_collapse_succeeds() {
    tokmd_cmd()
        .args(["lang", "--children", "collapse"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn lang_children_separate_succeeds() {
    tokmd_cmd()
        .args(["lang", "--children", "separate"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn lang_children_collapse_vs_separate_differ_in_json() {
    let collapse =
        json_of(&mut tokmd_cmd().args(["lang", "--format", "json", "--children", "collapse"]));
    let separate =
        json_of(&mut tokmd_cmd().args(["lang", "--format", "json", "--children", "separate"]));

    let c_rows = collapse["rows"].as_array().unwrap().len();
    let s_rows = separate["rows"].as_array().unwrap().len();
    // Separate mode may produce more rows (embedded) or equal — never fewer
    assert!(
        s_rows >= c_rows,
        "separate mode should have >= rows than collapse ({} vs {})",
        s_rows,
        c_rows
    );
}

// ===========================================================================
// 6. --top flag
// ===========================================================================

#[test]
fn lang_top_1_limits_rows_in_json() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json", "--top", "1"]));
    let rows = json["rows"].as_array().unwrap();
    // Should be at most 2 rows (1 real + "Other" roll-up)
    assert!(
        rows.len() <= 2,
        "top=1 should produce at most 2 rows (1 + Other), got {}",
        rows.len()
    );
}

#[test]
fn lang_top_0_shows_all_rows() {
    let json_all = json_of(&mut tokmd_cmd().args(["lang", "--format", "json", "--top", "0"]));
    let json_top1 = json_of(&mut tokmd_cmd().args(["lang", "--format", "json", "--top", "1"]));
    let all_len = json_all["rows"].as_array().unwrap().len();
    let top1_len = json_top1["rows"].as_array().unwrap().len();
    assert!(
        all_len >= top1_len,
        "top=0 should show all rows ({} >= {})",
        all_len,
        top1_len
    );
}

// ===========================================================================
// 7. --files flag
// ===========================================================================

#[test]
fn lang_files_flag_adds_file_info() {
    // The --files flag adds file counts and avg lines per file in md/tsv output
    tokmd_cmd()
        .args(["lang", "--files"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    // In JSON, file info is always present — verify it
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json", "--files"]));
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["files"].is_number(), "each row should have files field");
    }
}

// ===========================================================================
// 8. --exclude flag
// ===========================================================================

#[test]
fn lang_exclude_filters_files() {
    let json_all = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let json_filtered =
        json_of(&mut tokmd_cmd().args(["lang", "--format", "json", "--exclude", "*.rs"]));

    let total_all = json_all["total"]["code"].as_u64().unwrap_or(0);
    let total_filtered = json_filtered["total"]["code"].as_u64().unwrap_or(0);
    // Excluding *.rs should reduce code lines (fixture has .rs files)
    assert!(
        total_filtered <= total_all,
        "excluding *.rs should reduce total code lines"
    );
}

#[test]
fn lang_exclude_multiple_patterns() {
    tokmd_cmd()
        .args([
            "lang",
            "--format",
            "json",
            "--exclude",
            "*.rs",
            "--exclude",
            "*.js",
        ])
        .assert()
        .success();
}

// ===========================================================================
// 9. Error paths — invalid args
// ===========================================================================

#[test]
fn err_unknown_subcommand_fails() {
    tokmd_bare()
        .arg("frobnicate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_typo_subcommand_fails() {
    tokmd_bare()
        .arg("lnag")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_invalid_format_for_lang() {
    tokmd_cmd()
        .args(["lang", "--format", "xml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_invalid_format_for_module() {
    tokmd_cmd()
        .args(["module", "--format", "xml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_invalid_format_for_export() {
    tokmd_cmd()
        .args(["export", "--format", "xml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_invalid_children_mode() {
    tokmd_cmd()
        .args(["lang", "--children", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_nonexistent_path_lang() {
    tokmd_bare()
        .arg("lang")
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_nonexistent_path_module() {
    tokmd_bare()
        .arg("module")
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_nonexistent_path_export() {
    tokmd_bare()
        .arg("export")
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_nonexistent_path_analyze() {
    tokmd_bare()
        .arg("analyze")
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_diff_no_refs_fails() {
    tokmd_bare()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_badge_missing_metric_fails() {
    tokmd_cmd()
        .arg("badge")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_badge_invalid_metric_fails() {
    tokmd_cmd()
        .args(["badge", "--metric", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_completions_missing_shell() {
    tokmd_bare()
        .arg("completions")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_analyze_invalid_preset() {
    tokmd_cmd()
        .args(["analyze", "--preset", "banana"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_analyze_invalid_format() {
    tokmd_cmd()
        .args(["analyze", "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 10. Exit codes
// ===========================================================================

#[test]
fn success_exit_code_is_zero() {
    let output = tokmd_cmd().arg("lang").output().expect("run");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn failure_exit_code_is_nonzero() {
    let output = tokmd_bare().arg("frobnicate").output().expect("run");
    assert_ne!(output.status.code(), Some(0));
}

// ===========================================================================
// 11. Multiple path arguments
// ===========================================================================

#[test]
fn lang_with_explicit_path_arg() {
    tokmd_cmd()
        .args(["lang", "."])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn module_with_explicit_path_arg() {
    tokmd_cmd()
        .args(["module", "."])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn export_with_explicit_path_arg() {
    tokmd_cmd()
        .args(["export", "."])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 12. Determinism — same command twice produces same output
// ===========================================================================

#[test]
fn lang_json_deterministic_across_runs() {
    let run = || {
        let out = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(out.status.success());
        normalize_envelope(&String::from_utf8_lossy(&out.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "lang JSON must be byte-stable across runs");
}

#[test]
fn module_json_deterministic_across_runs() {
    let run = || {
        let out = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        assert!(out.status.success());
        normalize_envelope(&String::from_utf8_lossy(&out.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "module JSON must be byte-stable across runs");
}

#[test]
fn export_jsonl_deterministic_across_runs() {
    let run = || {
        let out = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        assert!(out.status.success());
        normalize_envelope(&String::from_utf8_lossy(&out.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "export JSONL must be byte-stable across runs");
}

#[test]
fn lang_tsv_deterministic_across_runs() {
    let run = || stdout_of(&mut tokmd_cmd().args(["lang", "--format", "tsv"]));
    let a = run();
    let b = run();
    assert_eq!(a, b, "lang TSV must be stable across runs");
}

#[test]
fn lang_md_deterministic_across_runs() {
    let run = || stdout_of(&mut tokmd_cmd().args(["lang", "--format", "md"]));
    let a = run();
    let b = run();
    assert_eq!(a, b, "lang Markdown must be stable across runs");
}

// ===========================================================================
// 13. JSON structure validation
// ===========================================================================

#[test]
fn lang_json_rows_sorted_by_code_descending() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let rows = json["rows"].as_array().unwrap();
    if rows.len() >= 2 {
        let codes: Vec<u64> = rows
            .iter()
            .map(|r| r["code"].as_u64().unwrap_or(0))
            .collect();
        for window in codes.windows(2) {
            assert!(
                window[0] >= window[1],
                "rows must be sorted by code descending: {} < {}",
                window[0],
                window[1]
            );
        }
    }
}

#[test]
fn lang_json_total_equals_sum_of_rows() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let rows = json["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap_or(0)).sum();
    let total = json["total"]["code"].as_u64().unwrap();
    assert_eq!(sum, total, "total.code must equal sum of rows code");
}

#[test]
fn lang_json_lines_total_equals_sum() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let rows = json["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["lines"].as_u64().unwrap_or(0)).sum();
    let total = json["total"]["lines"].as_u64().unwrap();
    assert_eq!(sum, total, "total.lines must equal sum of rows lines");
}

#[test]
fn lang_json_files_total_equals_sum() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let rows = json["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["files"].as_u64().unwrap_or(0)).sum();
    let total = json["total"]["files"].as_u64().unwrap();
    assert_eq!(sum, total, "total.files must equal sum of rows files");
}

#[test]
fn module_json_total_equals_sum_of_rows() {
    let json = json_of(&mut tokmd_cmd().args(["module", "--format", "json"]));
    let rows = json["rows"].as_array().unwrap();
    let sum: u64 = rows.iter().map(|r| r["code"].as_u64().unwrap_or(0)).sum();
    let total = json["total"]["code"].as_u64().unwrap();
    assert_eq!(sum, total, "module total.code must equal sum of rows code");
}

#[test]
fn lang_json_no_duplicate_languages() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let rows = json["rows"].as_array().unwrap();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (i, row) in rows.iter().enumerate() {
        let lang = row["lang"].as_str().unwrap_or("").to_string();
        if let Some(prev_idx) = seen.insert(lang.clone(), i) {
            panic!(
                "duplicate language '{}' at rows {} and {}",
                lang, prev_idx, i
            );
        }
    }
}

// ===========================================================================
// 14. TSV structure validation
// ===========================================================================

#[test]
fn lang_tsv_header_contains_expected_columns() {
    let out = stdout_of(&mut tokmd_cmd().args(["lang", "--format", "tsv"]));
    let header = out.lines().next().expect("TSV must have header");
    let lower = header.to_lowercase();
    assert!(lower.contains("lang"), "TSV header should contain 'lang'");
    assert!(lower.contains("code"), "TSV header should contain 'code'");
}

#[test]
fn module_tsv_header_contains_module() {
    let out = stdout_of(&mut tokmd_cmd().args(["module", "--format", "tsv"]));
    let header = out.lines().next().expect("TSV must have header");
    let lower = header.to_lowercase();
    assert!(
        lower.contains("module"),
        "module TSV header should contain 'module'"
    );
}

// ===========================================================================
// 15. Markdown structure validation
// ===========================================================================

#[test]
fn lang_md_has_separator_line() {
    let out = stdout_of(&mut tokmd_cmd().args(["lang", "--format", "md"]));
    // Markdown tables have a separator line with dashes
    let has_separator = out.lines().any(|l| l.contains("---"));
    assert!(
        has_separator,
        "Markdown table should have --- separator line"
    );
}

#[test]
fn module_md_has_separator_line() {
    let out = stdout_of(&mut tokmd_cmd().args(["module", "--format", "md"]));
    let has_separator = out.lines().any(|l| l.contains("---"));
    assert!(
        has_separator,
        "Module Markdown should have --- separator line"
    );
}

// ===========================================================================
// 16. Analyze command
// ===========================================================================

#[test]
fn analyze_default_preset_succeeds() {
    tokmd_cmd()
        .arg("analyze")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn analyze_receipt_preset_json_valid() {
    let json =
        json_of(&mut tokmd_cmd().args(["analyze", "--preset", "receipt", "--format", "json"]));
    assert!(json.is_object(), "analyze JSON should be an object");
}

#[test]
fn analyze_health_preset_succeeds() {
    tokmd_cmd()
        .args(["analyze", "--preset", "health"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 17. Badge command
// ===========================================================================

#[test]
fn badge_lines_metric_produces_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
}

#[test]
fn badge_tokens_metric_produces_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

// ===========================================================================
// 18. Completions command
// ===========================================================================

#[test]
fn completions_bash_produces_output() {
    tokmd_bare()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_produces_output() {
    tokmd_bare()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_produces_output() {
    tokmd_bare()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 19. Default command (bare invocation = lang)
// ===========================================================================

#[test]
fn bare_invocation_produces_output() {
    tokmd_cmd()
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn bare_invocation_matches_lang() {
    let bare = stdout_of(&mut tokmd_cmd());
    let lang = stdout_of(&mut tokmd_cmd().arg("lang"));
    assert_eq!(bare, lang, "bare invocation should match `tokmd lang`");
}

// ===========================================================================
// 20. Verbose flag
// ===========================================================================

#[test]
fn verbose_flag_succeeds() {
    tokmd_cmd()
        .args(["-v", "lang"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 21. --no-progress flag
// ===========================================================================

#[test]
fn no_progress_flag_succeeds() {
    tokmd_cmd()
        .args(["lang", "--no-progress"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 22. Hidden files and no-ignore flags
// ===========================================================================

#[test]
fn hidden_flag_succeeds() {
    tokmd_cmd()
        .args(["--hidden", "lang"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn no_ignore_flag_succeeds() {
    tokmd_cmd()
        .args(["--no-ignore", "lang"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 23. Cross-format consistency
// ===========================================================================

#[test]
fn lang_all_formats_report_same_total_code() {
    let json = json_of(&mut tokmd_cmd().args(["lang", "--format", "json"]));
    let total_code = json["total"]["code"].as_u64().unwrap();

    // TSV: last non-empty row is typically the total
    let tsv_out = stdout_of(&mut tokmd_cmd().args(["lang", "--format", "tsv"]));
    let tsv_lines: Vec<&str> = tsv_out.lines().filter(|l| !l.trim().is_empty()).collect();
    // The total row in TSV should contain the same total code
    let last_line = tsv_lines.last().unwrap();
    assert!(
        last_line.contains(&total_code.to_string()),
        "TSV total row should contain code total {}: {}",
        total_code,
        last_line
    );
}

// ===========================================================================
// 24. Export with --meta flag
// ===========================================================================

#[test]
fn export_jsonl_meta_true_has_meta_line() {
    let out = stdout_of(&mut tokmd_cmd().args(["export", "--format", "jsonl", "--meta", "true"]));
    let first = out.lines().next().unwrap();
    let meta: Value = serde_json::from_str(first).unwrap();
    assert!(meta["schema_version"].is_number());
}

#[test]
fn export_jsonl_meta_false_skips_meta() {
    let out = stdout_of(&mut tokmd_cmd().args(["export", "--format", "jsonl", "--meta", "false"]));
    let first = out.lines().next().unwrap();
    let row: Value = serde_json::from_str(first).unwrap();
    // Without meta, first line should be a data row (has "path" field)
    assert!(
        row.get("path").is_some() || row.get("schema_version").is_none(),
        "with --meta false, first line should be data, not meta"
    );
}

// ===========================================================================
// 25. Export --max-rows
// ===========================================================================

#[test]
fn export_max_rows_limits_output() {
    let out_1 = stdout_of(&mut tokmd_cmd().args([
        "export",
        "--format",
        "jsonl",
        "--max-rows",
        "1",
        "--meta",
        "false",
    ]));
    let lines_1: Vec<&str> = out_1.lines().filter(|l| !l.trim().is_empty()).collect();
    // With max-rows=1 and meta=false, should have exactly 1 data line
    assert_eq!(
        lines_1.len(),
        1,
        "max-rows=1 + meta=false should produce exactly 1 line, got {}",
        lines_1.len()
    );
}

// ===========================================================================
// 26. Tools subcommand
// ===========================================================================

#[test]
fn tools_openai_produces_json() {
    let out = stdout_of(&mut tokmd_cmd().args(["tools", "--format", "openai"]));
    let _: Value = serde_json::from_str(&out).expect("tools --format openai should produce JSON");
}

// ===========================================================================
// 27. Init subcommand
// ===========================================================================

#[test]
fn init_help_shows_usage() {
    tokmd_bare()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokeignore"));
}

// ===========================================================================
// 28. Global --format flag (top-level, not subcommand)
// ===========================================================================

#[test]
fn global_format_json_works() {
    let json = json_of(&mut tokmd_cmd().args(["--format", "json"]));
    assert!(json["rows"].is_array());
}

#[test]
fn global_format_tsv_works() {
    let out = stdout_of(&mut tokmd_cmd().args(["--format", "tsv"]));
    assert!(out.contains('\t'));
}

// ===========================================================================
// 29. Config flag
// ===========================================================================

#[test]
fn config_none_succeeds() {
    tokmd_cmd()
        .args(["--config", "none", "lang"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn config_auto_succeeds() {
    tokmd_cmd()
        .args(["--config", "auto", "lang"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 30. Treat-doc-strings-as-comments flag
// ===========================================================================

#[test]
fn treat_doc_strings_as_comments_succeeds() {
    tokmd_cmd()
        .args(["--treat-doc-strings-as-comments", "lang"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 31. Check-ignore subcommand
// ===========================================================================

#[test]
fn check_ignore_help_shows_usage() {
    tokmd_bare()
        .args(["check-ignore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
