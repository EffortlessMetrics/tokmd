#![cfg(feature = "analysis")]
//! CLI output-format verification tests (w76).
//!
//! Verifies every command's output format options using isolated temp
//! directories with fixture `.rs` / `.py` / `.js` files.
//!
//! ~30 tests covering: lang, module, export, analyze, badge, context.

use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp directory populated with small `.rs`, `.py`, and `.js` files
/// plus a `.git/` marker so the `ignore` crate honours gitignore rules.
fn make_fixture() -> TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    let p = dir.path();

    std::fs::create_dir_all(p.join("src")).unwrap();
    std::fs::create_dir_all(p.join("lib")).unwrap();
    std::fs::create_dir_all(p.join(".git")).unwrap();

    std::fs::write(
        p.join("src").join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    std::fs::write(
        p.join("lib").join("utils.py"),
        "def greet(name):\n    return f\"Hello, {name}\"\n\nif __name__ == '__main__':\n    greet('world')\n",
    )
    .unwrap();

    std::fs::write(
        p.join("lib").join("index.js"),
        "function add(a, b) {\n  return a + b;\n}\nmodule.exports = { add };\n",
    )
    .unwrap();

    dir
}

fn tokmd(dir: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir);
    cmd
}

fn run_stdout(dir: &Path, args: &[&str]) -> String {
    let output = tokmd(dir).args(args).output().expect("execute tokmd");
    assert!(
        output.status.success(),
        "tokmd {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("invalid UTF-8")
}

fn run_json(dir: &Path, args: &[&str]) -> Value {
    let stdout = run_stdout(dir, args);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not valid JSON for `{}`:\n{e}\n---\n{stdout}",
            args.join(" ")
        )
    })
}

// ===========================================================================
// 1. tokmd lang
// ===========================================================================

#[test]
fn lang_format_markdown_has_pipes_and_separator() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["lang", "--format", "md"]);

    assert!(
        out.contains('|'),
        "Markdown output must contain pipe characters"
    );
    assert!(
        out.lines()
            .any(|l| l.contains("|---") || l.contains("|:--")),
        "Markdown output must contain separator row"
    );
}

#[test]
fn lang_format_markdown_contains_expected_columns() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["lang", "--format", "md"]);

    assert!(out.contains("Lang"), "header must include Lang");
    assert!(out.contains("Code"), "header must include Code");
    assert!(out.contains("Tokens"), "header must include Tokens");
}

#[test]
fn lang_format_markdown_has_data_rows() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["lang", "--format", "md"]);

    assert!(out.contains("Rust"), "must list Rust language");
    assert!(out.contains("Python"), "must list Python language");
    assert!(out.contains("JavaScript"), "must list JavaScript language");
}

#[test]
fn lang_format_tsv_has_tabs_and_consistent_columns() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["lang", "--format", "tsv"]);
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(lines.len() >= 2, "need header + at least one data row");
    let header_cols = lines[0].split('\t').count();
    assert!(
        header_cols >= 3,
        "header should have >=3 tab-separated columns"
    );

    for (i, line) in lines[1..].iter().enumerate() {
        let cols = line.split('\t').count();
        assert_eq!(cols, header_cols, "TSV row {i} column count mismatch");
    }
}

#[test]
fn lang_format_tsv_contains_expected_headers() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["lang", "--format", "tsv"]);
    let header = out.lines().next().expect("must have header line");

    assert!(header.contains("Lang"), "TSV header must include Lang");
    assert!(header.contains("Code"), "TSV header must include Code");
}

#[test]
fn lang_format_json_is_parseable() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["lang", "--format", "json"]);

    assert!(json.is_object(), "top-level must be an object");
}

#[test]
fn lang_format_json_has_envelope_fields() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["lang", "--format", "json"]);

    assert_eq!(json["mode"], "lang", "mode must be 'lang'");
    assert!(
        json["schema_version"].is_number(),
        "must have schema_version"
    );
    assert!(json["rows"].is_array(), "must have rows array");
    assert!(json["tool"].is_object(), "must have tool metadata");
}

#[test]
fn lang_format_json_rows_contain_expected_fields() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["lang", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows is array");

    assert!(!rows.is_empty(), "rows must be non-empty");
    let first = &rows[0];
    assert!(first["lang"].is_string(), "row must have lang field");
    assert!(first["code"].is_number(), "row must have code field");
    assert!(first["files"].is_number(), "row must have files field");
}

// ===========================================================================
// 2. tokmd module
// ===========================================================================

#[test]
fn module_format_markdown_has_table_structure() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["module", "--format", "md"]);

    assert!(out.contains('|'), "Markdown must have pipes");
    assert!(
        out.lines()
            .any(|l| l.contains("|---") || l.contains("|:--")),
        "must have separator"
    );
    assert!(out.contains("Module"), "header must include Module");
    assert!(out.contains("Code"), "header must include Code");
}

#[test]
fn module_format_markdown_shows_directories() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["module", "--format", "md"]);

    let has_module = out.contains("src") || out.contains("lib") || out.contains("(root)");
    assert!(
        has_module,
        "module output must show directory-based modules"
    );
}

#[test]
fn module_format_tsv_has_consistent_columns() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["module", "--format", "tsv"]);
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(lines.len() >= 2, "need header + data");
    let header_cols = lines[0].split('\t').count();

    for (i, line) in lines[1..].iter().enumerate() {
        let cols = line.split('\t').count();
        assert_eq!(
            cols, header_cols,
            "module TSV row {i} column count mismatch"
        );
    }
}

#[test]
fn module_format_tsv_header_has_module_field() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["module", "--format", "tsv"]);
    let header = out.lines().next().unwrap();

    assert!(header.contains("Module"), "TSV header must include Module");
}

#[test]
fn module_format_json_is_parseable_with_mode() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["module", "--format", "json"]);

    assert_eq!(json["mode"], "module", "mode must be 'module'");
    assert!(json["rows"].is_array(), "must have rows");
}

#[test]
fn module_format_json_rows_have_module_key() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["module", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows");

    assert!(!rows.is_empty(), "must have at least one module row");
    let first = &rows[0];
    assert!(
        first["module"].is_string(),
        "each row must have a module field"
    );
    assert!(first["code"].is_number(), "each row must have code field");
}

// ===========================================================================
// 3. tokmd export
// ===========================================================================

#[test]
fn export_format_jsonl_each_line_is_valid_json() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["export", "--format", "jsonl"]);
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(!lines.is_empty(), "JSONL must have at least one line");
    for (i, line) in lines.iter().enumerate() {
        let v: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {i} is not valid JSON: {e}"));
        assert!(v.is_object(), "each JSONL line must be an object");
    }
}

#[test]
fn export_format_jsonl_data_rows_have_path_and_lang() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["export", "--format", "jsonl"]);
    // Skip the meta line (type: "meta") and find a data row
    let data_row = out
        .lines()
        .filter(|l| !l.trim().is_empty())
        .find(|l| l.contains("\"type\":\"row\"") || l.contains("\"type\": \"row\""))
        .expect("must have at least one data row");
    let v: Value = serde_json::from_str(data_row).unwrap();

    assert!(v["path"].is_string(), "JSONL data row must have path field");
    assert!(v["lang"].is_string(), "JSONL data row must have lang field");
    assert!(v["code"].is_number(), "JSONL data row must have code field");
}

#[test]
fn export_format_csv_has_comma_separated_header() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["export", "--format", "csv"]);
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(lines.len() >= 2, "CSV needs header + data rows");
    let header = lines[0];
    assert!(header.contains(','), "CSV header must be comma-separated");
    assert!(header.contains("path"), "CSV header must include path");
    assert!(header.contains("lang"), "CSV header must include lang");
}

#[test]
fn export_format_csv_rows_have_consistent_columns() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["export", "--format", "csv"]);
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();

    let header_cols = lines[0].split(',').count();
    for (i, line) in lines[1..].iter().enumerate() {
        let cols = line.split(',').count();
        assert_eq!(cols, header_cols, "CSV row {i} column count mismatch");
    }
}

#[test]
fn export_format_json_is_parseable_with_envelope() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["export", "--format", "json"]);

    assert_eq!(json["mode"], "export", "mode must be 'export'");
    assert!(
        json["schema_version"].is_number(),
        "must have schema_version"
    );
    assert!(json["rows"].is_array(), "must have rows");
}

#[test]
fn export_format_json_rows_have_file_fields() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["export", "--format", "json"]);
    let rows = json["rows"].as_array().expect("rows");

    assert!(!rows.is_empty(), "export rows must be non-empty");
    let first = &rows[0];
    assert!(first["path"].is_string(), "row must have path");
    assert!(first["lang"].is_string(), "row must have lang");
    assert!(first["code"].is_number(), "row must have code");
}

// ===========================================================================
// 4. tokmd analyze
// ===========================================================================

#[test]
fn analyze_format_markdown_is_non_empty_with_structure() {
    let dir = make_fixture();
    let out = run_stdout(
        dir.path(),
        &["analyze", "--format", "md", "--preset", "receipt"],
    );

    assert!(!out.trim().is_empty(), "analyze md must produce output");
    let has_structure = out.contains('#') || out.contains('|') || out.contains("---");
    assert!(
        has_structure,
        "analyze md should have markdown structure (headers or tables)"
    );
}

#[test]
fn analyze_format_markdown_mentions_metrics() {
    let dir = make_fixture();
    let out = run_stdout(
        dir.path(),
        &["analyze", "--format", "md", "--preset", "receipt"],
    );

    let out_lower = out.to_lowercase();
    let has_metric = out_lower.contains("density")
        || out_lower.contains("distribution")
        || out_lower.contains("cocomo")
        || out_lower.contains("code");
    assert!(
        has_metric,
        "analyze md receipt should mention at least one metric keyword"
    );
}

#[test]
fn analyze_format_json_is_parseable() {
    let dir = make_fixture();
    let json = run_json(
        dir.path(),
        &["analyze", "--format", "json", "--preset", "receipt"],
    );

    assert!(json.is_object(), "analyze JSON must be an object");
}

#[test]
fn analyze_format_json_has_schema_version() {
    let dir = make_fixture();
    let json = run_json(
        dir.path(),
        &["analyze", "--format", "json", "--preset", "receipt"],
    );

    assert!(
        json["schema_version"].is_number(),
        "analyze JSON must have schema_version"
    );
}

#[test]
fn analyze_format_json_has_mode_field() {
    let dir = make_fixture();
    let json = run_json(
        dir.path(),
        &["analyze", "--format", "json", "--preset", "receipt"],
    );

    assert!(
        json["mode"].is_string(),
        "analyze JSON must have mode field"
    );
}

// ===========================================================================
// 5. tokmd badge
// ===========================================================================

#[test]
fn badge_default_output_is_svg() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["badge", "--metric", "lines"]);

    assert!(
        out.contains("<svg"),
        "badge output must contain SVG element"
    );
    assert!(out.contains("</svg>"), "badge SVG must be complete");
}

#[test]
fn badge_svg_is_non_empty_and_has_namespace() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["badge", "--metric", "lines"]);

    assert!(!out.trim().is_empty(), "badge output must be non-empty");
    assert!(out.contains("xmlns"), "SVG should declare xmlns namespace");
}

#[test]
fn badge_svg_contains_metric_value() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["badge", "--metric", "lines"]);

    assert!(
        out.chars().any(|c| c.is_ascii_digit()),
        "badge SVG should contain at least one digit (the metric value)"
    );
}

// ===========================================================================
// 6. tokmd context
// ===========================================================================

#[test]
fn context_mode_list_produces_file_paths() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["context", "--mode", "list"]);

    assert!(!out.trim().is_empty(), "context list must produce output");
    let has_file = out.contains("main.rs") || out.contains("utils.py") || out.contains("index.js");
    assert!(has_file, "context list should mention fixture files");
}

#[test]
fn context_mode_list_is_not_json() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["context", "--mode", "list"]);

    let parsed: Result<Value, _> = serde_json::from_str(&out);
    assert!(
        parsed.is_err(),
        "context list mode should NOT be JSON (it is human-readable)"
    );
}

#[test]
fn context_mode_json_is_parseable() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["context", "--mode", "json"]);

    assert!(json.is_object(), "context JSON must be an object");
}

#[test]
fn context_mode_json_has_schema_and_files() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["context", "--mode", "json"]);

    assert!(
        json["schema_version"].is_number(),
        "context JSON must have schema_version"
    );
    assert!(
        json["files"].is_array(),
        "context JSON must have files array"
    );
}

#[test]
fn context_mode_json_has_budget_fields() {
    let dir = make_fixture();
    let json = run_json(dir.path(), &["context", "--mode", "json"]);

    assert!(
        json["budget_tokens"].is_number(),
        "context JSON must have budget_tokens"
    );
    assert!(
        json["used_tokens"].is_number(),
        "context JSON must have used_tokens"
    );
}

#[test]
fn context_mode_bundle_concatenates_content() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["context", "--mode", "bundle"]);

    assert!(!out.trim().is_empty(), "bundle mode must produce output");
    let has_content =
        out.contains("println") || out.contains("greet") || out.contains("module.exports");
    assert!(
        has_content,
        "bundle should contain file content from fixtures"
    );
}

#[test]
fn context_mode_bundle_is_not_json() {
    let dir = make_fixture();
    let out = run_stdout(dir.path(), &["context", "--mode", "bundle"]);

    let parsed: Result<Value, _> = serde_json::from_str(&out);
    assert!(
        parsed.is_err(),
        "context bundle mode should NOT be raw JSON"
    );
}
