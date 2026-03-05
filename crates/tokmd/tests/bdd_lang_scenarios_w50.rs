//! BDD-style scenario tests for the `lang` command.
//!
//! Each test follows the Given/When/Then pattern to verify key user-facing
//! workflows of the language summary command.

mod common;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// Scenario 1: JSON output with schema and sorted rows
// ---------------------------------------------------------------------------

#[test]
fn given_rust_project_when_lang_json_then_valid_json_with_schema_and_sorted_rows() {
    // Given: a Rust project (the fixture contains .rs files)
    // When: I run `tokmd lang --format json`
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to execute tokmd lang --format json");

    // Then: I get valid JSON with schema_version and rows sorted by code desc
    assert!(output.status.success(), "command should succeed");
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert!(
        json["schema_version"].is_number(),
        "should have schema_version"
    );
    assert_eq!(json["mode"], "lang", "mode should be 'lang'");

    let rows = json["rows"].as_array().expect("rows should be an array");
    assert!(!rows.is_empty(), "should detect at least one language");

    // Verify descending sort by code (excluding "Other" bucket)
    let code_vals: Vec<u64> = rows
        .iter()
        .filter(|r| r["lang"].as_str() != Some("Other"))
        .filter_map(|r| r["code"].as_u64())
        .collect();
    for window in code_vals.windows(2) {
        assert!(
            window[0] >= window[1],
            "rows should be sorted descending by code: {} < {}",
            window[0],
            window[1]
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Empty directory produces zero totals
// ---------------------------------------------------------------------------

#[test]
fn given_empty_directory_when_lang_then_zero_totals() {
    // Given: an empty directory
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");

    // When: I run `tokmd lang --format json`
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to execute tokmd lang on empty dir");

    // Then: I get a summary with zero totals
    assert!(
        output.status.success(),
        "command should succeed on empty dir"
    );
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    let total = &json["total"];
    assert_eq!(total["code"].as_u64().unwrap_or(0), 0, "code should be 0");
    assert_eq!(total["lines"].as_u64().unwrap_or(0), 0, "lines should be 0");
    let rows = json["rows"].as_array().expect("rows should be array");
    assert!(rows.is_empty(), "rows should be empty for empty dir");
}

// ---------------------------------------------------------------------------
// Scenario 3: --top 2 limits to 2 rows plus "Other"
// ---------------------------------------------------------------------------

#[test]
fn given_mixed_language_project_when_top_2_then_at_most_3_rows() {
    // Given: a mixed-language project (fixture has .rs, .js, .md, .toml files)
    // When: I run `tokmd lang --format json --top 2`
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "2"])
        .output()
        .expect("failed to execute tokmd lang --top 2");

    // Then: I get at most 2 primary rows plus an "Other" bucket
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");

    assert!(!rows.is_empty(), "should have at least 1 row");
    assert!(
        rows.len() <= 3,
        "with --top 2, at most 3 rows expected (2 + Other), got {}",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// Scenario 4: --children collapse merges embedded stats
// ---------------------------------------------------------------------------

#[test]
fn given_project_with_embedded_code_when_children_collapse_then_mode_recorded() {
    // Given: a project with embedded code (fixture has mixed.md with code blocks)
    // When: I run `tokmd lang --format json --children collapse`
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()
        .expect("failed to execute tokmd lang --children collapse");

    // Then: the children mode is recorded as "collapse" in args
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["args"]["children"].as_str().unwrap(),
        "collapse",
        "args should record children=collapse"
    );

    // And: no row should have "(embedded)" in the lang name
    let rows = json["rows"].as_array().expect("rows should be array");
    for row in rows {
        let lang = row["lang"].as_str().unwrap_or_default();
        assert!(
            !lang.contains("(embedded)"),
            "collapse mode should not have (embedded) rows, found: {lang}"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 5: --children separate produces separate embedded rows
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_children_separate_then_mode_recorded() {
    // Given: a project with files
    // When: I run `tokmd lang --format json --children separate`
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("failed to execute tokmd lang --children separate");

    // Then: the children mode is recorded as "separate"
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["args"]["children"].as_str().unwrap(),
        "separate",
        "args should record children=separate"
    );
}

// ---------------------------------------------------------------------------
// Scenario 6: TSV format has tab-separated columns with header
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_lang_tsv_then_tab_separated_with_header() {
    // Given: a project with source files
    // When: I run `tokmd lang --format tsv`
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to execute tokmd lang --format tsv");

    // Then: output has tab-separated columns with a header row
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(lines.len() >= 2, "need header + at least one data row");

    let header = lines[0];
    assert!(header.contains('\t'), "header should be tab-separated");
    assert!(
        header.contains("Code") || header.contains("code"),
        "header should contain Code column"
    );

    // All data rows should have the same column count as the header
    let header_cols = header.split('\t').count();
    for (i, line) in lines[1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').count();
        assert_eq!(
            cols,
            header_cols,
            "row {} has {} cols, header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 7: JSON output has expected row fields
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_lang_json_then_rows_have_expected_fields() {
    // Given: a project with source files
    // When: I run `tokmd lang --format json`
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to execute tokmd lang --format json");

    // Then: each row has lang, code, lines, blanks, comments fields
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    assert!(!rows.is_empty());

    for row in rows {
        assert!(row["lang"].is_string(), "row should have lang field");
        assert!(row["code"].is_number(), "row should have code field");
        assert!(row["lines"].is_number(), "row should have lines field");
        assert!(row["files"].is_number(), "row should have files field");
        assert!(row["bytes"].is_number(), "row should have bytes field");
    }
}

// ---------------------------------------------------------------------------
// Scenario 8: JSON output contains tool metadata
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_lang_json_then_has_tool_and_args_metadata() {
    // Given: a project
    // When: I run `tokmd lang --format json`
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to execute tokmd lang --format json");

    // Then: output contains tool and args metadata
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["tool"].is_object(), "should have tool metadata");
    assert!(json["args"].is_object(), "should have args metadata");
    assert!(
        json["generated_at_ms"].is_number(),
        "should have generated_at_ms"
    );
}

// ---------------------------------------------------------------------------
// Scenario 9: Markdown format produces table
// ---------------------------------------------------------------------------

#[test]
fn given_project_when_lang_md_then_markdown_table_output() {
    // Given: a project with source files
    // When: I run `tokmd lang --format md`
    let output = tokmd_cmd()
        .args(["lang", "--format", "md"])
        .output()
        .expect("failed to execute tokmd lang --format md");

    // Then: output contains a markdown table with pipe delimiters
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('|'),
        "markdown output should contain pipe characters"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.contains("|---") || l.contains("|:--")),
        "markdown output should contain table separator"
    );
}

// ---------------------------------------------------------------------------
// Scenario 10: Rust is detected in the fixture
// ---------------------------------------------------------------------------

#[test]
fn given_rust_project_when_lang_json_then_rust_detected() {
    // Given: a project with .rs files
    // When: I run `tokmd lang --format json`
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to execute tokmd lang --format json");

    // Then: Rust appears in the rows
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    let has_rust = rows.iter().any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(has_rust, "should detect Rust in fixture project");
}
