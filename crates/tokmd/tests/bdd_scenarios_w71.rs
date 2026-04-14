//! BDD-style scenario tests describing real user workflows.
//!
//! Each test follows the **Given / When / Then** pattern encoded in the
//! function name and documented with inline comments.  Tests create their own
//! hermetic fixtures so they are fully deterministic and independent.

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

/// Build a `Command` that runs against the shared hermetic fixture root.
fn tokmd_fixture() -> Command {
    let mut cmd = tokmd();
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Create a temp directory with a `.git` marker so `ignore` crate honours rules.
fn hermetic_dir() -> tempfile::TempDir {
    let dir = tempdir().expect("create temp dir");
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");
    dir
}

/// Write a minimal Rust source file.
fn write_rs(dir: &std::path::Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), body).expect("failed to write rs file");
}

// ===========================================================================
// 1. Rust project → lang shows Rust
// ===========================================================================

#[test]
fn given_rust_project_when_lang_then_shows_rust() {
    // Given: a directory containing .rs files
    let dir = hermetic_dir();
    write_rs(
        dir.path(),
        "lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    );

    // When: `tokmd lang` is run
    // Then: output contains "Rust"
    tokmd()
        .arg("lang")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

// ===========================================================================
// 2. Multi-language → JSON contains all languages
// ===========================================================================

#[test]
fn given_multi_language_when_lang_json_then_all_languages_present() {
    // Given: files in multiple languages
    let dir = hermetic_dir();
    write_rs(dir.path(), "main.rs", "fn main() {}");
    std::fs::write(dir.path().join("app.py"), "print('hello')").expect("failed to write app.py");
    std::fs::write(dir.path().join("index.js"), "console.log('hi');")
        .expect("failed to write index.js");

    // When: `tokmd lang --format json`
    let output = tokmd()
        .args(["lang", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: every language appears in the rows
    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    let rows = json["rows"].as_array().expect("rows array");
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(langs.contains(&"Rust"), "should contain Rust: {langs:?}");
    assert!(
        langs.contains(&"Python"),
        "should contain Python: {langs:?}"
    );
    assert!(
        langs.contains(&"JavaScript"),
        "should contain JavaScript: {langs:?}"
    );
}

// ===========================================================================
// 3. Empty directory → valid but empty receipt
// ===========================================================================

#[test]
fn given_empty_dir_when_lang_then_empty_receipt() {
    // Given: a completely empty directory (with .git marker)
    let dir = hermetic_dir();

    // When: `tokmd lang --format json`
    let output = tokmd()
        .args(["lang", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: valid JSON with zero or empty rows
    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert!(json["rows"].is_array());
    let rows = json["rows"]
        .as_array()
        .expect("expected rows to be an array");
    assert!(rows.is_empty(), "empty dir should produce empty rows");
}

// ===========================================================================
// 4. Exclude patterns filter out files
// ===========================================================================

#[test]
fn given_exclude_tests_when_lang_then_test_files_excluded() {
    // Given: a project with both lib and test files
    let dir = hermetic_dir();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).expect("failed to create src dir");
    std::fs::write(
        src.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .expect("failed to write lib.rs");
    let tests = dir.path().join("tests");
    std::fs::create_dir_all(&tests).expect("failed to create tests dir");
    std::fs::write(
        tests.join("test_add.rs"),
        "fn test() { assert_eq!(2, 1+1); }",
    )
    .expect("failed to write test_add.rs");

    // When: `tokmd --exclude tests export --format json`
    let output = tokmd()
        .args(["--exclude", "tests", "export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: test file should not appear in the rows
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("test_add.rs"),
        "test file should be excluded"
    );
    assert!(stdout.contains("lib.rs"), "lib file should remain");
}

// ===========================================================================
// 5. Module depth limiting groups correctly
// ===========================================================================

#[test]
fn given_nested_modules_when_module_depth1_then_top_level_grouping() {
    // Given: nested directory structure with files at different levels
    let dir = hermetic_dir();
    let src = dir.path().join("src");
    let lib = dir.path().join("lib");
    std::fs::create_dir_all(&src).expect("failed to create src dir");
    std::fs::create_dir_all(&lib).expect("failed to create lib dir");
    write_rs(&src, "main.rs", "fn main() { let x = 1; }");
    write_rs(&lib, "util.rs", "pub fn helper() { let y = 2; }");
    write_rs(dir.path(), "build.rs", "fn build() { let z = 3; }");

    // When: `tokmd module --module-depth 1 --format json`
    let output = tokmd()
        .args(["module", "--module-depth", "1", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: modules are grouped at top level
    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    let rows = json["rows"].as_array().expect("rows array");
    let modules: Vec<&str> = rows.iter().filter_map(|r| r["module"].as_str()).collect();
    assert!(
        modules.contains(&"src"),
        "should have src module: {modules:?}"
    );
    assert!(
        modules.contains(&"lib"),
        "should have lib module: {modules:?}"
    );
    assert!(
        modules.contains(&"(root)"),
        "should have root module: {modules:?}"
    );
}

// ===========================================================================
// 6. Export CSV lists all files
// ===========================================================================

#[test]
fn given_large_repo_when_export_csv_then_all_files_listed() {
    // Given: a project with multiple files
    let dir = hermetic_dir();
    let files = ["alpha.rs", "beta.rs", "gamma.rs"];
    for name in &files {
        write_rs(dir.path(), name, "fn f() { let x = 1; }");
    }

    // When: `tokmd export --format csv`
    let output = tokmd()
        .args(["export", "--format", "csv"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: every file appears in CSV output
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for name in &files {
        assert!(stdout.contains(name), "{name} should appear in CSV");
    }
}

// ===========================================================================
// 7. Diff between two receipts shows changes
// ===========================================================================

#[test]
fn given_json_receipt_when_diff_then_changes_shown() {
    // Given: two receipt files from separate runs
    let dir = hermetic_dir();
    write_rs(dir.path(), "lib.rs", "fn one() {}");

    let tmp = tempdir().expect("failed to create temp dir");
    let run1 = tmp.path().join("run1");
    let run2 = tmp.path().join("run2");

    // Run 1
    tokmd()
        .args(["run", "--output-dir"])
        .arg(&run1)
        .arg(".")
        .current_dir(dir.path())
        .assert()
        .success();

    // Modify code for second run
    write_rs(
        dir.path(),
        "lib.rs",
        "fn one() {}\nfn two() {}\nfn three() {}",
    );

    // Run 2
    tokmd()
        .args(["run", "--output-dir"])
        .arg(&run2)
        .arg(".")
        .current_dir(dir.path())
        .assert()
        .success();

    // When: `tokmd diff --from run1/receipt.json --to run2/receipt.json --format json`
    let output = tokmd()
        .args(["diff", "--format", "json", "--from"])
        .arg(run1.join("receipt.json"))
        .arg("--to")
        .arg(run2.join("receipt.json"))
        .output()
        .expect("run diff");

    // Then: valid JSON diff receipt is produced
    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert_eq!(json["mode"], "diff");
}

// ===========================================================================
// 8. Config via --exclude flag applies during scan
// ===========================================================================

#[test]
fn given_exclude_flag_when_scan_then_exclusion_applied() {
    // Given: a project with Rust and Python files
    let dir = hermetic_dir();
    write_rs(dir.path(), "main.rs", "fn main() { let x = 1; }");
    std::fs::write(dir.path().join("script.py"), "print('hello')")
        .expect("failed to write script.py");

    // When: `tokmd --exclude *.py export --format json`
    let output = tokmd()
        .args(["--exclude", "*.py", "export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: python file is excluded
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("script.py"),
        "script.py should be excluded by --exclude flag"
    );
    assert!(stdout.contains("main.rs"), "main.rs should remain");
}

// ===========================================================================
// 9. Unicode filenames – paths normalized
// ===========================================================================

#[test]
fn given_unicode_filenames_when_scan_then_paths_normalized() {
    // Given: a file with a Unicode name
    let dir = hermetic_dir();
    std::fs::write(dir.path().join("café.rs"), "fn greet() { let x = 1; }")
        .expect("failed to write cafe.rs");

    // When: `tokmd export --format json`
    let output = tokmd()
        .args(["export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: the file appears with forward-slash normalized path
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("café.rs"),
        "unicode filename should appear in output"
    );
    // Paths should use forward slashes even on Windows
    assert!(
        !stdout.contains('\\'),
        "paths should not contain backslashes"
    );
}

// ===========================================================================
// 10. Hidden files excluded by default
// ===========================================================================

#[test]
fn given_hidden_files_when_scan_default_then_hidden_excluded() {
    // Given: a directory with a hidden file
    let dir = hermetic_dir();
    write_rs(dir.path(), "visible.rs", "fn vis() { let x = 1; }");
    write_rs(dir.path(), ".hidden.rs", "fn hid() { let x = 1; }");

    // When: `tokmd export --format json` (default ignores hidden files)
    let output = tokmd()
        .args(["export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    // Then: hidden file should not appear
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(".hidden.rs"),
        "hidden file should be excluded by default"
    );
    assert!(stdout.contains("visible.rs"), "visible file should remain");
}

// ===========================================================================
// 11. .gitignore respected
// ===========================================================================

#[test]
fn given_gitignore_when_scan_then_gitignored_files_excluded() {
    // Given: fixture root has .gitignore that excludes hidden_by_git.rs
    // When: `tokmd export`
    // Then: hidden_by_git.rs should not appear
    tokmd_fixture()
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden_by_git.rs").not());
}

// ===========================================================================
// 12. JSON format always produces valid JSON
// ===========================================================================

#[test]
fn given_format_json_when_lang_then_valid_json() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let _: Value = serde_json::from_slice(&output.stdout).expect("valid JSON from lang");
}

#[test]
fn given_format_json_when_module_then_valid_json() {
    let output = tokmd_fixture()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let _: Value = serde_json::from_slice(&output.stdout).expect("valid JSON from module");
}

#[test]
fn given_format_json_when_export_then_valid_json() {
    let output = tokmd_fixture()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let _: Value = serde_json::from_slice(&output.stdout).expect("valid JSON from export");
}

#[test]
fn given_format_json_when_analyze_then_valid_json() {
    let output = tokmd_fixture()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let _: Value = serde_json::from_slice(&output.stdout).expect("valid JSON from analyze");
}

// ===========================================================================
// 13. JSON outputs have schema_version
// ===========================================================================

#[test]
fn given_format_json_when_lang_then_has_schema_version() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert!(
        json["schema_version"].is_number(),
        "lang JSON should have schema_version"
    );
}

#[test]
fn given_format_json_when_module_then_has_schema_version() {
    let output = tokmd_fixture()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "module JSON should have schema_version"
    );
}

#[test]
fn given_format_json_when_export_then_has_schema_version() {
    let output = tokmd_fixture()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "export JSON should have schema_version"
    );
}

#[test]
fn given_format_json_when_analyze_then_has_schema_version() {
    let output = tokmd_fixture()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "analyze JSON should have schema_version"
    );
}

// ===========================================================================
// 14. Determinism – same input → identical output
// ===========================================================================

#[test]
fn given_same_input_when_run_twice_then_identical_output() {
    // Given: a fixed project
    let dir = hermetic_dir();
    write_rs(
        dir.path(),
        "lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    );
    write_rs(dir.path(), "main.rs", "fn main() { println!(\"hi\"); }");

    // When: run `tokmd lang --format json` twice
    let run = |_round: u8| -> String {
        let output = tokmd()
            .args(["lang", "--format", "json"])
            .current_dir(dir.path())
            .output()
            .expect("run tokmd");
        assert!(output.status.success());
        let json: Value =
            serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
        // Strip volatile fields (generated_at_ms, tool version)
        let rows = &json["rows"];
        serde_json::to_string(rows).expect("failed to serialize rows")
    };

    let first = run(1);
    let second = run(2);

    // Then: row data is byte-identical
    assert_eq!(
        first, second,
        "deterministic output: rows must be identical"
    );
}

// ===========================================================================
// 15. check-ignore explains why a file is ignored
// ===========================================================================

#[test]
fn given_check_ignore_when_file_excluded_then_explains_why() {
    // Given: a temp dir with a file and an --exclude pattern
    let dir = hermetic_dir();
    write_rs(dir.path(), "hello.rs", "fn main() {}");

    // When: `tokmd --exclude hello.rs check-ignore hello.rs`
    // Then: the output mentions hello.rs and indicates it is ignored
    tokmd()
        .current_dir(dir.path())
        .args(["--exclude", "hello.rs", "check-ignore", "hello.rs"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ignored"));
}

// ===========================================================================
// 16. Export JSONL – each line is valid JSON
// ===========================================================================

#[test]
fn given_project_when_export_jsonl_then_every_line_valid_json() {
    let dir = hermetic_dir();
    write_rs(dir.path(), "a.rs", "fn a() { let x = 1; }");
    write_rs(dir.path(), "b.rs", "fn b() { let y = 2; }");

    let output = tokmd()
        .args(["export", "--format", "jsonl"])
        .current_dir(dir.path())
        .output()
        .expect("run tokmd");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for (i, line) in stdout.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let _: Value = serde_json::from_str(line).unwrap_or_else(|e| panic!("line {}: {e}", i + 1));
    }
}

// ===========================================================================
// 17. Badge command produces SVG
// ===========================================================================

#[test]
fn given_project_when_badge_then_outputs_svg() {
    tokmd_fixture()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

// ===========================================================================
// 18. Init command creates .tokeignore
// ===========================================================================

#[test]
fn given_empty_dir_when_init_then_tokeignore_created() {
    let dir = hermetic_dir();

    tokmd()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(
        dir.path().join(".tokeignore").exists(),
        ".tokeignore should be created"
    );
}

// ===========================================================================
// 19. Init --print just prints template
// ===========================================================================

#[test]
fn given_init_print_when_run_then_prints_template_without_file() {
    let dir = hermetic_dir();

    tokmd()
        .args(["init", "--print", "--non-interactive"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("target/"));

    // Should NOT create the file when --print is used
    assert!(
        !dir.path().join(".tokeignore").exists(),
        "--print should not create file"
    );
}

// ===========================================================================
// 20. Tools command generates AI schemas
// ===========================================================================

#[test]
fn given_tools_openai_when_run_then_valid_function_schema() {
    let output = tokmd_fixture()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert!(json.get("functions").is_some(), "should have functions key");
}

// ===========================================================================
// 21. Completions generate shell script
// ===========================================================================

#[test]
fn given_bash_shell_when_completions_then_outputs_script() {
    tokmd_fixture()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 22. Version flag shows semver
// ===========================================================================

#[test]
fn given_version_flag_when_run_then_shows_semver() {
    tokmd_fixture()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").expect("failed to compile regex"));
}

// ===========================================================================
// 23. Module default format outputs markdown table
// ===========================================================================

#[test]
fn given_project_when_module_default_then_markdown_table() {
    let dir = hermetic_dir();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).expect("failed to create src dir");
    write_rs(&src, "lib.rs", "pub fn f() { let x = 1; }");

    tokmd()
        .arg("module")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
}

// ===========================================================================
// 24. Analyze receipt preset produces derived metrics
// ===========================================================================

#[test]
fn given_project_when_analyze_receipt_then_has_derived_metrics() {
    let output = tokmd_fixture()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert!(json["derived"].is_object(), "should have derived metrics");
}

#[test]
fn given_project_when_analyze_estimate_then_has_effort_metrics() {
    let output = tokmd_fixture()
        .args(["analyze", "--preset", "estimate", "--format", "json"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert!(json["effort"].is_object(), "should have effort metrics");
    assert!(
        json["effort"]["drivers"].is_array(),
        "should have effort drivers"
    );
}

// ===========================================================================
// 25. Exclude with wildcard removes all matching files
// ===========================================================================

#[test]
fn given_exclude_wildcard_when_lang_json_then_language_absent() {
    // Given: fixture root with Rust and JavaScript files
    // When: exclude all .rs files
    let output = tokmd_fixture()
        .args(["--exclude", "*.rs", "lang", "--format", "json"])
        .output()
        .expect("run");

    // Then: Rust should not appear but other languages should remain
    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    let rows = json["rows"]
        .as_array()
        .expect("expected rows to be an array");
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(
        !langs.contains(&"Rust"),
        "Rust should be excluded: {langs:?}"
    );
    // Fixture has .js and .md files, at least one non-Rust language should remain
    assert!(
        !langs.is_empty(),
        "non-Rust languages should remain: {langs:?}"
    );
}

// ===========================================================================
// 26. Export JSON has rows array
// ===========================================================================

#[test]
fn given_project_when_export_json_then_has_rows_array() {
    let dir = hermetic_dir();
    write_rs(dir.path(), "lib.rs", "pub fn f() { let x = 1; }");

    let output = tokmd()
        .args(["export", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .expect("run");

    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    let rows = json["rows"].as_array().expect("rows should be array");
    assert!(!rows.is_empty(), "should have at least one file row");
    for row in rows {
        assert!(row["path"].is_string(), "each row should have a path");
        assert!(row["code"].is_number(), "each row should have code count");
    }
}

// ===========================================================================
// 27. Lang JSON has mode field set correctly
// ===========================================================================

#[test]
fn given_lang_json_when_run_then_mode_is_lang() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert_eq!(json["mode"], "lang");
}

// ===========================================================================
// 28. Module JSON has total object
// ===========================================================================

#[test]
fn given_module_json_when_run_then_has_total() {
    let output = tokmd_fixture()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    assert!(json["total"].is_object(), "should have total");
    assert!(
        json["total"]["code"].is_number(),
        "total should have code count"
    );
}

// ===========================================================================
// 29. TSV format uses tabs
// ===========================================================================

#[test]
fn given_lang_tsv_when_run_then_tab_separated() {
    let output = tokmd_fixture()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('\t'), "TSV should contain tabs");
    assert!(
        stdout.contains("Lang\tCode\tLines"),
        "TSV should have header"
    );
}

// ===========================================================================
// 30. No-ignore-vcs reveals gitignored files
// ===========================================================================

#[test]
fn given_gitignored_file_when_no_ignore_vcs_then_file_visible() {
    // Given: fixture root has hidden_by_git.rs in .gitignore
    // When: --no-ignore-vcs is used
    tokmd_fixture()
        .args(["--no-ignore-vcs", "export"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden_by_git.rs"));
}

// ===========================================================================
// 31. Multiple exclude patterns compose correctly
// ===========================================================================

#[test]
fn given_multiple_excludes_when_lang_then_all_patterns_applied() {
    // Given: fixture root with mixed languages
    // When: exclude both .rs and .js
    let output = tokmd_fixture()
        .args([
            "--exclude",
            "*.rs",
            "--exclude",
            "*.js",
            "lang",
            "--format",
            "json",
        ])
        .output()
        .expect("run");

    // Then: neither Rust nor JavaScript should appear
    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("failed to parse JSON from stdout");
    let rows = json["rows"]
        .as_array()
        .expect("expected rows to be an array");
    let langs: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(
        !langs.contains(&"Rust"),
        "Rust should be excluded: {langs:?}"
    );
    assert!(
        !langs.contains(&"JavaScript"),
        "JavaScript should be excluded: {langs:?}"
    );
}

// ===========================================================================
// 32. Check-ignore with --exclude on temp dir
// ===========================================================================

#[test]
fn given_check_ignore_with_exclude_when_file_matches_then_reports_match() {
    let dir = hermetic_dir();
    write_rs(dir.path(), "hello.rs", "fn main() {}");

    tokmd()
        .current_dir(dir.path())
        .args(["--exclude", "hello.rs", "check-ignore", "hello.rs"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ignored"));
}
