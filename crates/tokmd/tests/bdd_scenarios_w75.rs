#![cfg(feature = "analysis")]

//! BDD-style scenario tests documenting user-facing behaviour.
//!
//! Each test follows the **Given / When / Then** pattern encoded in the
//! function name and inline comments.  Tests use isolated temp dirs for
//! determinism and independence.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `Command` pointing at the compiled `tokmd` binary.
fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Build a `Command` running against the shared hermetic fixture root.
fn tokmd_fixture() -> Command {
    let mut cmd = tokmd();
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Create a temp directory with `.git` marker so the `ignore` crate honours rules.
fn hermetic_dir() -> tempfile::TempDir {
    let dir = tempdir().expect("create temp dir");
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");
    dir
}

/// Write a file into `dir`.
fn write_file(dir: &std::path::Path, name: &str, body: &str) {
    if let Some(parent) = dir.join(name).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(dir.join(name), body).unwrap();
}

/// Run tokmd with args, assert success, parse JSON.
fn run_json(dir: &std::path::Path, args: &[&str]) -> Value {
    let output = tokmd()
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run tokmd");
    assert!(output.status.success(), "tokmd failed: {:?}", output.status);
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}

// ===========================================================================
// Scenario 1: Fresh Rust project scan
// ===========================================================================

#[test]
fn given_rust_project_when_lang_json_then_receipt_has_rust_with_code_gt_zero() {
    // Given: a directory with Cargo.toml and src/main.rs
    let dir = hermetic_dir();
    write_file(
        dir.path(),
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write_file(
        dir.path(),
        "src/main.rs",
        "fn main() {\n    println!(\"hello\");\n}\n",
    );

    // When: `tokmd lang --format json`
    let json = run_json(dir.path(), &["lang", "--format", "json"]);

    // Then: receipt contains Rust and total code lines > 0
    let rows = json["rows"].as_array().expect("rows array");
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(langs.contains(&"Rust"), "should contain Rust: {langs:?}");
    let total_code = json["total"]["code"].as_u64().unwrap_or(0);
    assert!(
        total_code > 0,
        "total code lines should be > 0, got {total_code}"
    );
}

// ===========================================================================
// Scenario 2: Multi-language project
// ===========================================================================

#[test]
fn given_multi_lang_project_when_lang_json_then_all_languages_present_sorted_desc() {
    // Given: a directory with .rs, .py, .js files
    let dir = hermetic_dir();
    write_file(
        dir.path(),
        "main.rs",
        "fn main() {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n}\n",
    );
    write_file(dir.path(), "app.py", "x = 1\n");
    write_file(dir.path(), "index.js", "let y = 2;\n");

    // When: `tokmd lang --format json`
    let json = run_json(dir.path(), &["lang", "--format", "json"]);

    // Then: rows for Rust, Python, JavaScript
    let rows = json["rows"].as_array().expect("rows array");
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(langs.contains(&"Rust"), "missing Rust: {langs:?}");
    assert!(langs.contains(&"Python"), "missing Python: {langs:?}");
    assert!(
        langs.contains(&"JavaScript"),
        "missing JavaScript: {langs:?}"
    );

    // And: sorted descending by code lines
    let codes: Vec<u64> = rows.iter().filter_map(|r| r["code"].as_u64()).collect();
    for w in codes.windows(2) {
        assert!(
            w[0] >= w[1],
            "rows should be sorted desc by code: {codes:?}"
        );
    }
}

// ===========================================================================
// Scenario 3: Module breakdown
// ===========================================================================

#[test]
fn given_nested_dirs_when_module_depth1_then_modules_listed() {
    // Given: a directory with foo/bar.rs and baz/qux.rs
    let dir = hermetic_dir();
    write_file(dir.path(), "foo/bar.rs", "fn bar() { let x = 1; }\n");
    write_file(dir.path(), "baz/qux.rs", "fn qux() { let y = 2; }\n");

    // When: `tokmd module --format json --module-depth 1`
    let json = run_json(
        dir.path(),
        &["module", "--format", "json", "--module-depth", "1"],
    );

    // Then: modules for foo and baz
    let rows = json["rows"].as_array().expect("rows array");
    let modules: Vec<&str> = rows.iter().filter_map(|r| r["module"].as_str()).collect();
    assert!(
        modules.iter().any(|m| m.contains("foo")),
        "should have foo module: {modules:?}"
    );
    assert!(
        modules.iter().any(|m| m.contains("baz")),
        "should have baz module: {modules:?}"
    );
}

// ===========================================================================
// Scenario 4: Export inventory
// ===========================================================================

#[test]
fn given_multiple_files_when_export_jsonl_then_each_line_valid_json_with_fields() {
    // Given: a directory with multiple files
    let dir = hermetic_dir();
    write_file(dir.path(), "alpha.rs", "fn a() { let x = 1; }\n");
    write_file(dir.path(), "beta.rs", "fn b() { let y = 2; }\n");
    write_file(dir.path(), "gamma.rs", "fn g() { let z = 3; }\n");

    // When: `tokmd export --format jsonl`
    let output = tokmd()
        .args(["export", "--format", "jsonl"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: each line is valid JSON with path, language, code fields
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "should have meta + data lines, got {}",
        lines.len()
    );

    for (i, line) in lines.iter().enumerate() {
        let v: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} not valid JSON: {e}", i + 1));
        // Data rows (after meta) should have path, lang, code
        if v.get("schema_version").is_none() {
            assert!(v["path"].is_string(), "line {}: missing path", i + 1);
            assert!(v["lang"].is_string(), "line {}: missing lang", i + 1);
            assert!(v["code"].is_number(), "line {}: missing code", i + 1);
        }
    }
}

// ===========================================================================
// Scenario 5: Context packing
// ===========================================================================

#[test]
fn given_small_project_when_context_json_budget_then_tokens_within_budget() {
    // Given: a small project with a few files
    let dir = hermetic_dir();
    write_file(
        dir.path(),
        "lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    );
    write_file(
        dir.path(),
        "util.rs",
        "pub fn sub(a: i32, b: i32) -> i32 { a - b }\n",
    );

    // When: `tokmd context --mode json --budget 10000`
    let json = run_json(
        dir.path(),
        &["context", "--mode", "json", "--budget", "10000"],
    );

    // Then: total tokens in context <= 10000
    let used = json["used_tokens"].as_u64().expect("used_tokens");
    let budget = json["budget_tokens"].as_u64().expect("budget_tokens");
    assert!(used <= 10000, "used_tokens ({used}) should be <= 10000");
    assert_eq!(
        budget, 10000,
        "budget_tokens should reflect requested budget"
    );
}

// ===========================================================================
// Scenario 6: Badge generation
// ===========================================================================

#[test]
fn given_source_code_when_badge_then_valid_svg_output() {
    // Given: a directory with source code
    // When: `tokmd badge`
    // Then: valid SVG output containing <svg
    tokmd_fixture()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

// ===========================================================================
// Scenario 7: Analysis presets
// ===========================================================================

#[test]
fn given_source_when_analyze_receipt_then_derived_metrics_present() {
    // Given: a directory with source code
    // When: `tokmd analyze --format json --preset receipt`
    let json = run_json(
        common::fixture_root(),
        &["analyze", "--preset", "receipt", "--format", "json"],
    );

    // Then: derived metrics including density and distribution
    assert_eq!(json["mode"], "analysis", "mode should be 'analysis'");
    assert!(json.get("source").is_some(), "should have source section");
    assert!(json.get("derived").is_some(), "should have derived section");
    assert!(
        json["schema_version"].is_number(),
        "should have schema_version"
    );
}

// ===========================================================================
// Scenario 8: Check-ignore explanation
// ===========================================================================

#[test]
fn given_exclude_when_check_ignore_then_explains_ignored() {
    // Given: a directory with a source file and an --exclude pattern
    let dir = hermetic_dir();
    write_file(dir.path(), "ignored_file.rs", "fn f() { let x = 1; }\n");

    // When: `tokmd --exclude ignored_file.rs check-ignore ignored_file.rs`
    // Then: output explains the file is ignored
    tokmd()
        .current_dir(dir.path())
        .args([
            "--exclude",
            "ignored_file.rs",
            "check-ignore",
            "ignored_file.rs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ignored"));
}

// ===========================================================================
// Scenario 9: Rust-only project has no Python or JavaScript
// ===========================================================================

#[test]
fn given_rust_only_when_lang_json_then_no_other_languages() {
    // Given: a directory with only Rust files
    let dir = hermetic_dir();
    write_file(dir.path(), "main.rs", "fn main() { let x = 1; }\n");

    // When: `tokmd lang --format json`
    let json = run_json(dir.path(), &["lang", "--format", "json"]);

    // Then: only Rust appears
    let rows = json["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 1, "should have exactly one language row");
    assert_eq!(rows[0]["lang"], "Rust");
}

// ===========================================================================
// Scenario 10: Empty directory produces empty receipt
// ===========================================================================

#[test]
fn given_empty_dir_when_lang_json_then_empty_rows() {
    // Given: an empty directory
    let dir = hermetic_dir();

    // When: `tokmd lang --format json`
    let json = run_json(dir.path(), &["lang", "--format", "json"]);

    // Then: valid receipt with empty rows
    let rows = json["rows"].as_array().expect("rows");
    assert!(rows.is_empty(), "empty dir should produce empty rows");
}

// ===========================================================================
// Scenario 11: Export JSON has path and code fields per file
// ===========================================================================

#[test]
fn given_files_when_export_json_then_rows_have_path_and_code() {
    // Given: several source files
    let dir = hermetic_dir();
    write_file(dir.path(), "a.rs", "fn a() { let x = 1; }\n");
    write_file(dir.path(), "b.rs", "fn b() { let y = 2; }\n");

    // When: `tokmd export --format json`
    let json = run_json(dir.path(), &["export", "--format", "json"]);

    // Then: each row has path and code count
    let rows = json["rows"].as_array().expect("rows");
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["path"].is_string(), "row should have path");
        assert!(row["code"].is_number(), "row should have code");
    }
}

// ===========================================================================
// Scenario 12: Lang JSON receipt has mode field
// ===========================================================================

#[test]
fn given_project_when_lang_json_then_mode_is_lang() {
    // Given: fixture root with source files
    // When: `tokmd lang --format json`
    let json = run_json(common::fixture_root(), &["lang", "--format", "json"]);

    // Then: mode is "lang"
    assert_eq!(json["mode"], "lang");
}

// ===========================================================================
// Scenario 13: Module JSON has total object with code count
// ===========================================================================

#[test]
fn given_project_when_module_json_then_total_has_code() {
    // Given: project with source files
    let dir = hermetic_dir();
    write_file(dir.path(), "src/lib.rs", "pub fn f() { let x = 1; }\n");

    // When: `tokmd module --format json`
    let json = run_json(dir.path(), &["module", "--format", "json"]);

    // Then: total object with code count
    assert!(json["total"].is_object(), "should have total");
    assert!(json["total"]["code"].is_number(), "total should have code");
}

// ===========================================================================
// Scenario 14: Schema version present in JSON outputs
// ===========================================================================

#[test]
fn given_project_when_lang_json_then_has_schema_version() {
    let json = run_json(common::fixture_root(), &["lang", "--format", "json"]);
    assert!(
        json["schema_version"].is_number(),
        "should have schema_version"
    );
}

#[test]
fn given_project_when_export_json_then_has_schema_version() {
    let json = run_json(common::fixture_root(), &["export", "--format", "json"]);
    assert!(
        json["schema_version"].is_number(),
        "should have schema_version"
    );
}

// ===========================================================================
// Scenario 15: Determinism — same input produces identical output
// ===========================================================================

#[test]
fn given_same_input_when_lang_json_twice_then_rows_identical() {
    // Given: a fixed project
    let dir = hermetic_dir();
    write_file(
        dir.path(),
        "lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    );
    write_file(dir.path(), "main.rs", "fn main() { println!(\"hi\"); }\n");

    // When: run twice
    let run = || -> String {
        let json = run_json(dir.path(), &["lang", "--format", "json"]);
        serde_json::to_string(&json["rows"]).unwrap()
    };

    let first = run();
    let second = run();

    // Then: row data is identical
    assert_eq!(first, second, "deterministic: rows must match across runs");
}

// ===========================================================================
// Scenario 16: Exclude pattern filters files
// ===========================================================================

#[test]
fn given_exclude_flag_when_export_then_excluded_files_absent() {
    // Given: Rust and Python files
    let dir = hermetic_dir();
    write_file(dir.path(), "main.rs", "fn main() { let x = 1; }\n");
    write_file(dir.path(), "script.py", "print('hi')\n");

    // When: `tokmd --exclude *.py export --format json`
    let output = tokmd()
        .args(["--exclude", "*.py", "export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run");

    // Then: Python file absent, Rust file present
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("script.py"),
        "script.py should be excluded"
    );
    assert!(stdout.contains("main.rs"), "main.rs should remain");
}

// ===========================================================================
// Scenario 17: Hidden files excluded by default
// ===========================================================================

#[test]
fn given_hidden_file_when_default_scan_then_hidden_excluded() {
    // Given: a visible file and a hidden file
    let dir = hermetic_dir();
    write_file(dir.path(), "visible.rs", "fn vis() { let x = 1; }\n");
    write_file(dir.path(), ".hidden.rs", "fn hid() { let y = 2; }\n");

    // When: `tokmd export --format json`
    let output = tokmd()
        .args(["export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run");

    // Then: hidden file absent
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(".hidden.rs"),
        "hidden file should be excluded"
    );
    assert!(stdout.contains("visible.rs"), "visible file should remain");
}

// ===========================================================================
// Scenario 18: Badge with metric=tokens outputs SVG
// ===========================================================================

#[test]
fn given_source_when_badge_tokens_then_svg() {
    let dir = hermetic_dir();
    write_file(dir.path(), "lib.rs", "pub fn f() { let x = 1; }\n");

    tokmd()
        .args(["badge", "--metric", "tokens"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

// ===========================================================================
// Scenario 19: Context mode=list outputs file list
// ===========================================================================

#[test]
fn given_files_when_context_list_then_outputs_file_paths() {
    // Given: project with files
    let dir = hermetic_dir();
    write_file(dir.path(), "main.rs", "fn main() { let x = 1; }\n");

    // When: `tokmd context --mode list`
    tokmd()
        .args(["context", "--mode", "list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"));
}

// ===========================================================================
// Scenario 20: Init creates .tokeignore
// ===========================================================================

#[test]
fn given_empty_dir_when_init_then_tokeignore_created() {
    // Given: an empty directory
    let dir = hermetic_dir();

    // When: `tokmd init`
    tokmd()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Then: .tokeignore exists
    assert!(
        dir.path().join(".tokeignore").exists(),
        ".tokeignore should be created"
    );
}

// ===========================================================================
// Scenario 21: TSV format uses tab separators
// ===========================================================================

#[test]
fn given_project_when_lang_tsv_then_tab_separated_header() {
    // Given: fixture root
    // When: `tokmd lang --format tsv`
    let output = tokmd_fixture()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("run");

    // Then: output contains tab-separated header
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('\t'), "TSV should contain tabs");
    assert!(
        stdout.contains("Lang\tCode\tLines"),
        "TSV should have header"
    );
}

// ===========================================================================
// Scenario 22: Version flag shows semver
// ===========================================================================

#[test]
fn given_version_flag_when_run_then_semver_shown() {
    tokmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ===========================================================================
// Scenario 23: Tools command generates OpenAI function schema
// ===========================================================================

#[test]
fn given_tools_openai_when_run_then_valid_schema_json() {
    // When: `tokmd tools --format openai`
    let output = tokmd_fixture()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("run");

    // Then: valid JSON with functions key
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("functions").is_some(), "should have functions key");
}

// ===========================================================================
// Scenario 24: Completions generates shell script
// ===========================================================================

#[test]
fn given_bash_shell_when_completions_then_script_output() {
    tokmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// Scenario 25: No-ignore-vcs reveals gitignored files
// ===========================================================================

#[test]
fn given_gitignored_file_when_no_ignore_vcs_then_visible() {
    // Given: a project with .gitignore that hides a file
    let dir = hermetic_dir();
    write_file(dir.path(), ".gitignore", "secret.rs\n");
    write_file(dir.path(), "secret.rs", "fn secret() { let x = 1; }\n");
    write_file(dir.path(), "public.rs", "fn public() { let y = 2; }\n");

    // When: `tokmd --no-ignore-vcs export`
    tokmd()
        .args(["--no-ignore-vcs", "export"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("secret.rs"));
}

// ===========================================================================
// Scenario 26: Multiple exclude patterns compose correctly
// ===========================================================================

#[test]
fn given_multiple_excludes_when_lang_then_all_patterns_applied() {
    // Given: Rust, Python, and JavaScript files
    let dir = hermetic_dir();
    write_file(dir.path(), "main.rs", "fn main() { let x = 1; }\n");
    write_file(dir.path(), "app.py", "x = 1\n");
    write_file(dir.path(), "index.js", "let y = 2;\n");

    // When: exclude both .rs and .py
    let json = run_json(
        dir.path(),
        &[
            "--exclude",
            "*.rs",
            "--exclude",
            "*.py",
            "lang",
            "--format",
            "json",
        ],
    );

    // Then: only JavaScript remains
    let rows = json["rows"].as_array().unwrap();
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(
        !langs.contains(&"Rust"),
        "Rust should be excluded: {langs:?}"
    );
    assert!(
        !langs.contains(&"Python"),
        "Python should be excluded: {langs:?}"
    );
    assert!(
        langs.contains(&"JavaScript"),
        "JavaScript should remain: {langs:?}"
    );
}

// ===========================================================================
// Scenario 27: Context budget enforcement
// ===========================================================================

#[test]
fn given_project_when_context_tight_budget_then_used_le_budget() {
    // Given: project with source files
    let dir = hermetic_dir();
    write_file(dir.path(), "a.rs", "fn a() { let x = 1; }\n");
    write_file(dir.path(), "b.rs", "fn b() { let y = 2; }\n");
    write_file(dir.path(), "c.rs", "fn c() { let z = 3; }\n");

    // When: `tokmd context --mode json --budget 500`
    let json = run_json(
        dir.path(),
        &["context", "--mode", "json", "--budget", "500"],
    );

    // Then: used_tokens <= budget_tokens
    let budget = json["budget_tokens"].as_u64().expect("budget_tokens");
    let used = json["used_tokens"].as_u64().expect("used_tokens");
    assert!(
        used <= budget,
        "used_tokens ({used}) should not exceed budget_tokens ({budget})"
    );
}

// ===========================================================================
// Scenario 28: Unicode filenames in export
// ===========================================================================

#[test]
fn given_unicode_filename_when_export_then_path_normalized() {
    // Given: a file with a Unicode name
    let dir = hermetic_dir();
    write_file(dir.path(), "café.rs", "fn greet() { let x = 1; }\n");

    // When: `tokmd export --format json`
    let output = tokmd()
        .args(["export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run");

    // Then: unicode filename appears with forward-slash paths
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("café.rs"), "unicode filename should appear");
    assert!(!stdout.contains('\\'), "paths should use forward slashes");
}

// ===========================================================================
// Scenario 29: Analyze receipt has schema_version
// ===========================================================================

#[test]
fn given_project_when_analyze_json_then_schema_version_is_number() {
    let json = run_json(
        common::fixture_root(),
        &["analyze", "--preset", "receipt", "--format", "json"],
    );
    assert!(
        json["schema_version"].is_number(),
        "should have numeric schema_version"
    );
}

// ===========================================================================
// Scenario 30: Context JSON has utilization_pct
// ===========================================================================

#[test]
fn given_project_when_context_json_then_has_utilization_pct() {
    // Given: project with source files
    let dir = hermetic_dir();
    write_file(dir.path(), "main.rs", "fn main() { println!(\"hi\"); }\n");

    // When: `tokmd context --mode json --budget 10000`
    let json = run_json(
        dir.path(),
        &["context", "--mode", "json", "--budget", "10000"],
    );

    // Then: utilization_pct is a number between 0 and 100
    let pct = json["utilization_pct"].as_f64().expect("utilization_pct");
    assert!(
        (0.0..=100.0).contains(&pct),
        "utilization_pct should be 0..100, got {pct}"
    );
}
