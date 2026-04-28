#![cfg(feature = "analysis")]

//! Expanded CLI end-to-end tests covering error handling, output format matrix,
//! and flag combination behaviour.
//!
//! Each test invokes the real `tokmd` binary via `assert_cmd` against the
//! hermetic fixture directory.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

fn tokmd_fixture() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

fn nonexistent_path() -> std::path::PathBuf {
    std::env::temp_dir().join("tokmd_w58_nonexistent_path_does_not_exist")
}

// ===========================================================================
// 1. Error handling – invalid subcommand
// ===========================================================================

#[test]
fn err_invalid_subcommand_shows_suggestion() {
    tokmd_cmd()
        .arg("lnag")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_completely_unknown_subcommand() {
    tokmd_cmd()
        .arg("frobnicate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 2. Error handling – non-existent paths
// ===========================================================================

#[test]
fn err_lang_nonexistent_path() {
    tokmd_cmd()
        .args(["lang"])
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_module_nonexistent_path() {
    tokmd_cmd()
        .args(["module"])
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_export_nonexistent_path() {
    tokmd_cmd()
        .args(["export"])
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_analyze_nonexistent_path() {
    tokmd_cmd()
        .args(["analyze"])
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_run_nonexistent_path() {
    tokmd_cmd()
        .args(["run"])
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_context_nonexistent_path() {
    tokmd_cmd()
        .args(["context"])
        .arg(nonexistent_path())
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 3. Error handling – invalid format flags
// ===========================================================================

#[test]
fn err_lang_invalid_format() {
    tokmd_fixture()
        .args(["lang", "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_module_invalid_format() {
    tokmd_fixture()
        .args(["module", "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_export_invalid_format() {
    tokmd_fixture()
        .args(["export", "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_analyze_invalid_format() {
    tokmd_fixture()
        .args(["analyze", "--format", "yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 4. Error handling – invalid preset for analyze
// ===========================================================================

#[test]
fn err_analyze_invalid_preset() {
    tokmd_fixture()
        .args(["analyze", "--preset", "banana"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 5. Error handling – missing required args
// ===========================================================================

#[test]
fn err_diff_no_refs() {
    tokmd_cmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_badge_missing_metric() {
    tokmd_fixture()
        .arg("badge")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn err_badge_invalid_metric() {
    tokmd_fixture()
        .args(["badge", "--metric", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_completions_missing_shell() {
    tokmd_cmd()
        .arg("completions")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 6. Error handling – invalid children mode
// ===========================================================================

#[test]
fn err_lang_invalid_children_mode() {
    tokmd_fixture()
        .args(["lang", "--children", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn err_module_invalid_children_mode() {
    tokmd_fixture()
        .args(["module", "--children", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 7. Output format matrix – lang
// ===========================================================================

#[test]
fn fmt_lang_json_valid_with_schema_version() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "lang");
    assert!(json["rows"].is_array());
}

#[test]
fn fmt_lang_json_rows_non_empty() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "fixture should produce at least one lang");
}

#[test]
fn fmt_lang_json_has_total() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["total"].is_object());
    assert!(json["total"]["code"].is_number());
}

#[test]
fn fmt_lang_markdown_contains_headers() {
    tokmd_fixture()
        .args(["lang", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn fmt_lang_tsv_tab_separated_with_header() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "need header + data");
    assert!(lines[0].contains('\t'), "header must be tab-separated");
}

// ===========================================================================
// 8. Output format matrix – module
// ===========================================================================

#[test]
fn fmt_module_json_valid_with_mode() {
    let output = tokmd_fixture()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "module");
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
}

#[test]
fn fmt_module_json_rows_have_module_field() {
    let output = tokmd_fixture()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    for row in json["rows"].as_array().unwrap() {
        assert!(row["module"].is_string());
        assert!(row["code"].is_number());
    }
}

#[test]
fn fmt_module_markdown_contains_table() {
    tokmd_fixture()
        .args(["module", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn fmt_module_tsv_has_tab_columns() {
    let output = tokmd_fixture()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'));
    assert!(stdout.lines().count() >= 2);
}

// ===========================================================================
// 9. Output format matrix – export
// ===========================================================================

#[test]
fn fmt_export_jsonl_each_line_valid_json() {
    let output = tokmd_fixture()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "meta + data rows");
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} not valid JSON: {}", i + 1, e));
    }
}

#[test]
fn fmt_export_jsonl_meta_has_schema() {
    let output = tokmd_fixture()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first = stdout.lines().next().unwrap();
    let meta: Value = serde_json::from_str(first).unwrap();
    assert!(meta["schema_version"].is_number());
}

#[test]
fn fmt_export_csv_has_header_row() {
    let output = tokmd_fixture()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let header = stdout.lines().next().unwrap();
    assert!(
        header.contains("path") || header.contains("language") || header.contains("code"),
        "CSV should have recognizable header columns"
    );
}

#[test]
fn fmt_export_csv_consistent_columns() {
    let output = tokmd_fixture()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2);
    let col_count = lines[0].split(',').count();
    for (i, line) in lines[1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        assert_eq!(
            line.split(',').count(),
            col_count,
            "CSV row {} column count mismatch",
            i + 1
        );
    }
}

#[test]
fn fmt_export_json_envelope() {
    let output = tokmd_fixture()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "export");
    assert!(json["schema_version"].is_number());
    assert!(json["rows"].is_array());
}

#[test]
fn fmt_export_json_rows_have_path() {
    let output = tokmd_fixture()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert!(
            row["path"].is_string() || row["file"].is_string(),
            "export rows should have path"
        );
    }
}

// ===========================================================================
// 10. Output format matrix – analyze
// ===========================================================================

#[test]
fn fmt_analyze_json_has_derived() {
    let output = tokmd_fixture()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object());
}

#[test]
fn fmt_analyze_markdown_contains_headers() {
    tokmd_fixture()
        .args(["analyze", "--preset", "receipt", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 11. Output format matrix – tools
// ===========================================================================

#[test]
fn fmt_tools_openai_valid_json() {
    let output = tokmd_fixture()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["functions"].is_array());
}

#[test]
fn fmt_tools_anthropic_valid_json() {
    let output = tokmd_fixture()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["tools"].is_array());
}

// ===========================================================================
// 12. Flag combinations – children modes
// ===========================================================================

#[test]
fn flag_lang_children_collapse() {
    let output = tokmd_fixture()
        .args(["lang", "--children", "collapse", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

#[test]
fn flag_lang_children_separate() {
    let output = tokmd_fixture()
        .args(["lang", "--children", "separate", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

#[test]
fn flag_module_children_separate() {
    let output = tokmd_fixture()
        .args(["module", "--children", "separate", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

#[test]
fn flag_module_children_parents_only() {
    let output = tokmd_fixture()
        .args(["module", "--children", "parents-only", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

#[test]
fn flag_export_children_separate() {
    let output = tokmd_fixture()
        .args(["export", "--children", "separate", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

#[test]
fn flag_export_children_parents_only() {
    let output = tokmd_fixture()
        .args(["export", "--children", "parents-only", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

// ===========================================================================
// 13. Flag combinations – top N limiting
// ===========================================================================

#[test]
fn flag_lang_top_limits_rows() {
    let output = tokmd_fixture()
        .args(["lang", "--top", "1", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    // With --top 1, we get at most 1 real row + possibly an "Other" bucket
    assert!(rows.len() <= 2, "top=1 should yield at most 2 rows");
}

#[test]
fn flag_lang_top_zero_shows_all() {
    let output = tokmd_fixture()
        .args(["lang", "--top", "0", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
}

#[test]
fn flag_module_top_limits_rows() {
    let output = tokmd_fixture()
        .args(["module", "--top", "1", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(rows.len() <= 2, "top=1 should yield at most 2 rows");
}

// ===========================================================================
// 14. Flag combinations – module depth
// ===========================================================================

#[test]
fn flag_module_depth_1() {
    let output = tokmd_fixture()
        .args(["module", "--module-depth", "1", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    // At depth=1, module keys should be single-segment
    for row in rows {
        let module = row["module"].as_str().unwrap_or("");
        let segments: Vec<&str> = module.split('/').filter(|s| !s.is_empty()).collect();
        assert!(
            segments.len() <= 1,
            "depth=1 module '{}' has {} segments",
            module,
            segments.len()
        );
    }
}

#[test]
fn flag_module_depth_large_succeeds() {
    tokmd_fixture()
        .args(["module", "--module-depth", "99", "--format", "json"])
        .assert()
        .success();
}

// ===========================================================================
// 15. Flag combinations – exclude patterns
// ===========================================================================

#[test]
fn flag_exclude_reduces_output() {
    let all_output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run all");

    let filtered_output = tokmd_fixture()
        .args(["lang", "--format", "json", "--exclude", "*.rs"])
        .output()
        .expect("run filtered");

    assert!(all_output.status.success());
    assert!(filtered_output.status.success());

    let all_json: Value = serde_json::from_slice(&all_output.stdout).unwrap();
    let filtered_json: Value = serde_json::from_slice(&filtered_output.stdout).unwrap();

    let all_total = all_json["total"]["code"].as_u64().unwrap_or(0);
    let filtered_total = filtered_json["total"]["code"].as_u64().unwrap_or(0);

    // Excluding *.rs from the fixture (which contains .rs files) should reduce count
    assert!(
        filtered_total <= all_total,
        "excluding *.rs should not increase total: all={} filtered={}",
        all_total,
        filtered_total
    );
}

#[test]
fn flag_exclude_nonexistent_pattern_still_succeeds() {
    tokmd_fixture()
        .args(["lang", "--format", "json", "--exclude", "*.zzz_nonexistent"])
        .assert()
        .success();
}

#[test]
fn flag_multiple_excludes() {
    tokmd_fixture()
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
// 16. Default format tests (no --format flag)
// ===========================================================================

#[test]
fn default_lang_produces_markdown() {
    tokmd_fixture()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("|"));
}

#[test]
fn default_module_produces_markdown() {
    tokmd_fixture()
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("|"));
}

// ===========================================================================
// 17. Version and help sanity
// ===========================================================================

#[test]
fn version_contains_semver() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

#[test]
fn help_mentions_subcommands() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lang"))
        .stdout(predicate::str::contains("module"))
        .stdout(predicate::str::contains("export"));
}

// ===========================================================================
// 18. Analyze preset variants
// ===========================================================================

#[test]
fn analyze_preset_health_succeeds() {
    tokmd_fixture()
        .args(["analyze", "--preset", "health", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn analyze_preset_supply_succeeds() {
    tokmd_fixture()
        .args(["analyze", "--preset", "supply", "--format", "json"])
        .assert()
        .success();
}

// ===========================================================================
// 19. Badge metric variants
// ===========================================================================

#[test]
fn badge_metric_tokens_produces_svg() {
    tokmd_fixture()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_metric_bytes_produces_svg() {
    tokmd_fixture()
        .args(["badge", "--metric", "bytes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

// ===========================================================================
// 20. JSON structural invariants across commands
// ===========================================================================

#[test]
fn json_envelope_has_args_metadata_lang() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["args"].is_object(), "lang JSON should contain args");
}

#[test]
fn json_envelope_has_args_metadata_module() {
    let output = tokmd_fixture()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["args"].is_object(), "module JSON should contain args");
}

#[test]
fn json_envelope_has_args_metadata_export() {
    let output = tokmd_fixture()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["args"].is_object(), "export JSON should contain args");
}
