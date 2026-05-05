#![cfg(feature = "analysis")]

//! Wave 69 — deep end-to-end CLI integration tests.
//!
//! ~40 tests covering all major subcommands, output formats, flag combinations,
//! error paths, JSON structure validation, and edge cases.
//! Each test invokes the real `tokmd` binary via `assert_cmd` against hermetic
//! fixture directories.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

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

/// Create a temp dir with mixed-language fixture files and a .git marker.
fn create_mixed_lang_tempdir() -> TempDir {
    let dir = tempfile::tempdir().expect("create tempdir");

    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    std::fs::write(
        dir.path().join("app.js"),
        "function greet() {\n  console.log('hi');\n}\ngreet();\n",
    )
    .unwrap();

    std::fs::write(
        dir.path().join("lib.py"),
        "def hello():\n    print('hello')\n\nhello()\n",
    )
    .unwrap();

    std::fs::write(
        dir.path().join("README.md"),
        "# Project\n\nA sample project.\n",
    )
    .unwrap();

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src").join("util.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    std::fs::create_dir_all(dir.path().join("lib")).unwrap();
    std::fs::write(
        dir.path().join("lib").join("helper.js"),
        "module.exports = { help: () => 'ok' };\n",
    )
    .unwrap();

    // .git marker so ignore crate works
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    dir
}

/// Create a minimal temp dir with a single Rust file.
fn create_minimal_tempdir() -> TempDir {
    let dir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    dir
}

// ===========================================================================
// 1. tokmd lang — mixed language files
// ===========================================================================

#[test]
fn w69_lang_mixed_dir_detects_rust() {
    let dir = create_mixed_lang_tempdir();
    tokmd_bare()
        .arg("lang")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn w69_lang_mixed_dir_detects_javascript() {
    let dir = create_mixed_lang_tempdir();
    tokmd_bare()
        .arg("lang")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("JavaScript"));
}

#[test]
fn w69_lang_mixed_dir_detects_python() {
    let dir = create_mixed_lang_tempdir();
    tokmd_bare()
        .arg("lang")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Python"));
}

#[test]
fn w69_lang_mixed_dir_detects_at_least_three_languages() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(tokmd_bare().arg("lang").current_dir(dir.path()));
    // Markdown has 0 code lines so it may be omitted; verify we get >= 3 rows
    let data_lines = out
        .lines()
        .filter(|l| l.starts_with('|') && !l.contains("---"))
        .count();
    // Header + total + at least 3 language rows = at least 5
    assert!(
        data_lines >= 5,
        "should detect at least 3 languages, got output:\n{}",
        out
    );
}

// ===========================================================================
// 2. tokmd lang --format json — valid JSON with schema_version
// ===========================================================================

#[test]
fn w69_lang_json_valid_with_schema_version() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "lang");
}

#[test]
fn w69_lang_json_rows_have_required_fields() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    let rows = json["rows"].as_array().expect("rows array");
    assert!(rows.len() >= 3, "should detect at least 3 languages");
    for row in rows {
        assert!(row["lang"].is_string(), "each row needs lang");
        assert!(row["code"].is_number(), "each row needs code");
        assert!(row["files"].is_number(), "each row needs files");
    }
}

#[test]
fn w69_lang_json_has_total() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert!(json["total"].is_object(), "should have total");
    assert!(json["total"]["code"].is_number());
}

#[test]
fn w69_lang_json_has_args() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert!(json["args"].is_object(), "should have args envelope");
}

// ===========================================================================
// 3. tokmd lang --format tsv — tab-separated output
// ===========================================================================

#[test]
fn w69_lang_tsv_contains_tabs() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["lang", "--format", "tsv"])
            .current_dir(dir.path()),
    );
    assert!(out.contains('\t'), "TSV should contain tab characters");
}

#[test]
fn w69_lang_tsv_has_header_row() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["lang", "--format", "tsv"])
            .current_dir(dir.path()),
    );
    let first_line = out.lines().next().expect("at least one line");
    assert!(
        first_line.contains("Lang") || first_line.contains("lang"),
        "TSV header should contain language column name"
    );
}

#[test]
fn w69_lang_tsv_multiple_rows() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["lang", "--format", "tsv"])
            .current_dir(dir.path()),
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 4, "should have header + 3+ language rows");
}

// ===========================================================================
// 4. tokmd module — module breakdown output
// ===========================================================================

#[test]
fn w69_module_default_has_module_column() {
    let dir = create_mixed_lang_tempdir();
    tokmd_bare()
        .arg("module")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
}

#[test]
fn w69_module_json_has_rows() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["module", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert!(json["rows"].is_array());
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "should have at least one module row");
}

#[test]
fn w69_module_json_rows_have_module_field() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["module", "--format", "json"])
            .current_dir(dir.path()),
    );
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        assert!(row["module"].is_string(), "each row needs module field");
        assert!(row["code"].is_number(), "each row needs code field");
    }
}

// ===========================================================================
// 5. tokmd module --module-depth 1 — depth limiting
// ===========================================================================

#[test]
fn w69_module_depth_1_limits_output() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["module", "--format", "json", "--module-depth", "1"])
            .current_dir(dir.path()),
    );
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        let module = row["module"].as_str().unwrap_or("");
        let depth = module.split('/').count();
        assert!(
            depth <= 1,
            "depth-1 module '{}' has {} segments",
            module,
            depth
        );
    }
}

#[test]
fn w69_module_depth_1_json_valid() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["module", "--format", "json", "--module-depth", "1"])
            .current_dir(dir.path()),
    );
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "module");
}

// ===========================================================================
// 6. tokmd export --format jsonl — one JSON per line
// ===========================================================================

#[test]
fn w69_export_jsonl_each_line_valid_json() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "jsonl"])
            .current_dir(dir.path()),
    );
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "should have meta + at least one data row");
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} is not valid JSON: {}", i + 1, e));
    }
}

#[test]
fn w69_export_jsonl_has_meta_record() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "jsonl"])
            .current_dir(dir.path()),
    );
    let first = out.lines().next().expect("at least one line");
    let v: Value = serde_json::from_str(first).expect("meta is JSON");
    assert!(
        v.get("schema_version").is_some() || v.get("tool").is_some(),
        "first line should be a meta record"
    );
}

#[test]
fn w69_export_jsonl_data_rows_have_path() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "jsonl"])
            .current_dir(dir.path()),
    );
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    // Skip meta record (first line), check data rows
    for line in &lines[1..] {
        let v: Value = serde_json::from_str(line).unwrap();
        assert!(
            v.get("path").is_some() || v.get("lang").is_some(),
            "data rows should have path or lang"
        );
    }
}

// ===========================================================================
// 7. tokmd export --format csv — CSV with header
// ===========================================================================

#[test]
fn w69_export_csv_has_header_and_rows() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "csv"])
            .current_dir(dir.path()),
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 2, "should have header + at least one row");
}

#[test]
fn w69_export_csv_header_has_columns() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "csv"])
            .current_dir(dir.path()),
    );
    let header = out.lines().next().unwrap();
    assert!(
        header.contains("path") || header.contains("lang"),
        "CSV header should contain column names"
    );
}

#[test]
fn w69_export_csv_rows_are_comma_separated() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "csv"])
            .current_dir(dir.path()),
    );
    for line in out.lines() {
        assert!(line.contains(','), "CSV lines should contain commas");
    }
}

// ===========================================================================
// 8. tokmd --help — expected subcommands
// ===========================================================================

#[test]
fn w69_help_lists_all_subcommands() {
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
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("tools"))
        .stdout(predicate::str::contains("gate"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("check-ignore"));
}

#[test]
fn w69_help_contains_usage_section() {
    tokmd_bare()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

// ===========================================================================
// 9. tokmd lang --help — subcommand help
// ===========================================================================

#[test]
fn w69_lang_help_shows_options() {
    tokmd_bare()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn w69_module_help_shows_depth_option() {
    tokmd_bare()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--module-depth"));
}

#[test]
fn w69_export_help_shows_format_option() {
    tokmd_bare()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("jsonl").or(predicate::str::contains("csv")));
}

// ===========================================================================
// 10. tokmd --version — verify version output
// ===========================================================================

#[test]
fn w69_version_shows_semver() {
    tokmd_bare()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

#[test]
fn w69_version_contains_tokmd() {
    tokmd_bare()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

// ===========================================================================
// 11. tokmd lang --exclude — verify exclude patterns work
// ===========================================================================

#[test]
fn w69_exclude_md_removes_markdown() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["lang", "--format", "json", "--exclude", "*.md"])
            .current_dir(dir.path()),
    );
    let json: Value = serde_json::from_str(&out).unwrap();
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        let lang = row["lang"].as_str().unwrap_or("");
        assert_ne!(
            lang, "Markdown",
            "Markdown should be excluded by --exclude '*.md'"
        );
    }
}

#[test]
fn w69_exclude_rs_removes_rust() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["lang", "--format", "json", "--exclude", "*.rs"])
            .current_dir(dir.path()),
    );
    let json: Value = serde_json::from_str(&out).unwrap();
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        let lang = row["lang"].as_str().unwrap_or("");
        assert_ne!(lang, "Rust", "Rust should be excluded by --exclude '*.rs'");
    }
}

// ===========================================================================
// 12. tokmd lang default sort — descending by code lines
// ===========================================================================

#[test]
fn w69_lang_json_rows_sorted_by_code_descending() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    let rows = json["rows"].as_array().unwrap();
    let codes: Vec<u64> = rows.iter().filter_map(|r| r["code"].as_u64()).collect();
    for window in codes.windows(2) {
        assert!(
            window[0] >= window[1],
            "rows should be sorted descending by code: {:?}",
            codes
        );
    }
}

// ===========================================================================
// 13. Error cases
// ===========================================================================

#[test]
fn w69_error_nonexistent_directory() {
    let bogus = std::env::temp_dir().join("tokmd_w69_nonexistent_does_not_exist");
    // Pass the nonexistent path as a positional argument
    tokmd_bare()
        .args(["lang", bogus.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn w69_error_invalid_format_for_lang() {
    tokmd_cmd()
        .args(["lang", "--format", "nonexistent_format_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn w69_error_invalid_format_for_export() {
    tokmd_cmd()
        .args(["export", "--format", "nonexistent_format_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn w69_error_unknown_subcommand() {
    tokmd_bare()
        .arg("nonexistent_subcommand_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 14. tokmd completions — verify generation
// ===========================================================================

#[test]
fn w69_completions_bash_produces_output() {
    tokmd_bare()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn w69_completions_powershell_produces_output() {
    tokmd_bare()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn w69_completions_zsh_produces_output() {
    tokmd_bare()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn w69_completions_fish_produces_output() {
    tokmd_bare()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 15. tokmd init — .tokeignore generation
// ===========================================================================

#[test]
fn w69_init_print_generates_tokeignore() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn w69_init_print_contains_ignore_patterns() {
    let out = stdout_of(tokmd_cmd().args(["init", "--print", "--non-interactive"]));
    assert!(
        out.contains('#') || out.contains("node_modules") || out.contains("target"),
        "init output should contain ignore patterns or comments"
    );
}

// ===========================================================================
// Additional deep tests — determinism, envelope, cross-command consistency
// ===========================================================================

#[test]
fn w69_lang_default_format_is_markdown_table() {
    let dir = create_mixed_lang_tempdir();
    tokmd_bare()
        .arg("lang")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn w69_module_default_format_is_markdown_table() {
    let dir = create_mixed_lang_tempdir();
    tokmd_bare()
        .arg("module")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("|"));
}

#[test]
fn w69_lang_json_deterministic_across_runs() {
    let dir = create_mixed_lang_tempdir();
    let json1 = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    let json2 = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert_eq!(json1["rows"], json2["rows"], "rows should be deterministic");
    assert_eq!(
        json1["total"], json2["total"],
        "total should be deterministic"
    );
}

#[test]
fn w69_export_jsonl_deterministic_across_runs() {
    let dir = create_mixed_lang_tempdir();
    let out1 = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "jsonl"])
            .current_dir(dir.path()),
    );
    let out2 = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "jsonl"])
            .current_dir(dir.path()),
    );
    let lines1: Vec<&str> = out1.lines().filter(|l| !l.trim().is_empty()).collect();
    let lines2: Vec<&str> = out2.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        lines1.len(),
        lines2.len(),
        "line count should be deterministic"
    );
    // Compare data rows (skip meta which may have timestamps)
    for (l1, l2) in lines1.iter().skip(1).zip(lines2.iter().skip(1)) {
        assert_eq!(l1, l2, "data rows should be deterministic");
    }
}

#[test]
fn w69_lang_minimal_dir_succeeds() {
    let dir = create_minimal_tempdir();
    tokmd_bare()
        .arg("lang")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn w69_module_json_total_present() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["module", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert!(json["total"].is_object(), "total should be present");
    assert!(
        json["total"]["code"].is_number(),
        "total should have code count"
    );
}

#[test]
fn w69_export_csv_consistent_column_count() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "csv"])
            .current_dir(dir.path()),
    );
    let lines: Vec<&str> = out.lines().collect();
    if lines.len() >= 2 {
        let header_cols = lines[0].split(',').count();
        for (i, line) in lines.iter().skip(1).enumerate() {
            let cols = line.split(',').count();
            assert_eq!(
                cols,
                header_cols,
                "row {} has {} columns but header has {}",
                i + 1,
                cols,
                header_cols
            );
        }
    }
}

#[test]
fn w69_lang_json_schema_version_is_positive() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    let sv = json["schema_version"]
        .as_u64()
        .expect("schema_version is u64");
    assert!(sv > 0, "schema_version should be positive");
}

#[test]
fn w69_module_json_schema_version_matches_lang() {
    let dir = create_mixed_lang_tempdir();
    let lang_json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    let mod_json = json_of(
        tokmd_bare()
            .args(["module", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert_eq!(
        lang_json["schema_version"], mod_json["schema_version"],
        "core receipts should share schema_version"
    );
}

#[test]
fn w69_lang_json_generated_at_present() {
    let dir = create_mixed_lang_tempdir();
    let json = json_of(
        tokmd_bare()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path()),
    );
    assert!(
        json.get("generated_at_ms").is_some() || json.get("generated_at").is_some(),
        "JSON receipt should have a timestamp field"
    );
}

#[test]
fn w69_export_jsonl_meta_has_tool_info() {
    let dir = create_mixed_lang_tempdir();
    let out = stdout_of(
        tokmd_bare()
            .args(["export", "--format", "jsonl"])
            .current_dir(dir.path()),
    );
    let first = out.lines().next().expect("at least one line");
    let v: Value = serde_json::from_str(first).expect("meta is JSON");
    assert!(
        v.get("tool").is_some() || v.get("schema_version").is_some(),
        "meta record should have tool info or schema_version"
    );
}
