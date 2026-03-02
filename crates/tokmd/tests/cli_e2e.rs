//! Comprehensive end-to-end tests for CLI commands.
//!
//! Covers: badge, init, completions, tools, and JSON/JSONL/CSV output
//! validation for lang, module, and export commands.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// A) tokmd badge — SVG badge generation
// ===========================================================================

#[test]
fn badge_lines_metric_should_contain_svg_element() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
}

#[test]
fn badge_lines_should_produce_well_formed_svg_with_xmlns() {
    let output = tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .output()
        .expect("Failed to run badge");

    assert!(output.status.success());
    let svg = String::from_utf8_lossy(&output.stdout);
    assert!(
        svg.contains("xmlns"),
        "SVG should include xmlns attribute for valid SVG"
    );
}

#[test]
fn badge_with_metric_tokens_should_produce_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

#[test]
fn badge_with_metric_bytes_should_produce_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "bytes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_with_metric_doc_should_produce_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "doc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_with_metric_blank_should_produce_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "blank"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_invalid_metric_should_fail() {
    tokmd_cmd()
        .args(["badge", "--metric", "nonexistent_metric_xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn badge_out_flag_should_write_svg_file() {
    let dir = tempdir().unwrap();
    let out_path = dir.path().join("output.svg");

    tokmd_cmd()
        .args(["badge", "--metric", "lines", "--out"])
        .arg(&out_path)
        .assert()
        .success()
        .stdout(""); // stdout should be empty when writing to file

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("<svg"), "File should contain SVG content");
    assert!(
        content.contains("</svg>"),
        "File should contain closing SVG tag"
    );
}

// ===========================================================================
// B) tokmd init — tokeignore generation
// ===========================================================================

#[test]
fn init_print_should_output_tokeignore_template() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn init_print_should_contain_common_ignore_patterns() {
    let output = tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .output()
        .expect("Failed to run init --print");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Common patterns that should appear in a tokeignore template
    let has_pattern = stdout.contains("node_modules")
        || stdout.contains("target")
        || stdout.contains(".git")
        || stdout.contains("vendor")
        || stdout.contains("dist");
    assert!(
        has_pattern,
        "tokeignore template should contain at least one common ignore pattern"
    );
}

#[test]
fn init_print_rust_template_should_contain_target() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "rust", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target/"));
}

#[test]
fn init_print_node_template_should_contain_node_modules() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "node", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("node_modules/"));
}

#[test]
fn init_non_interactive_should_create_tokeignore_file() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success();

    let tokeignore = dir.path().join(".tokeignore");
    assert!(
        tokeignore.exists(),
        ".tokeignore file should be created by init"
    );

    let content = std::fs::read_to_string(&tokeignore).unwrap();
    assert!(
        !content.is_empty(),
        ".tokeignore should not be empty after init"
    );
}

#[test]
fn init_should_refuse_overwrite_without_force() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".tokeignore"), "# existing content\n").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .failure();
}

// ===========================================================================
// C) tokmd completions — Shell completions
// ===========================================================================

#[test]
fn completions_bash_should_produce_nonempty_output() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_should_produce_nonempty_output() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_should_produce_nonempty_output() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_should_produce_nonempty_output() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_bash_should_reference_tokmd() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_zsh_should_reference_tokmd() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_fish_should_reference_tokmd() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_powershell_should_reference_tokmd() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

// ===========================================================================
// D) tokmd tools — LLM tool definitions
// ===========================================================================

#[test]
fn tools_openai_format_should_produce_valid_json_with_functions() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("Failed to run tools --format openai");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("OpenAI output should be valid JSON");

    assert!(
        parsed.get("functions").is_some(),
        "OpenAI format should have 'functions' key"
    );
    let functions = parsed["functions"].as_array().unwrap();
    assert!(
        !functions.is_empty(),
        "Should have at least one function definition"
    );
}

#[test]
fn tools_anthropic_format_should_produce_valid_json_with_tools() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("Failed to run tools --format anthropic");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value =
        serde_json::from_str(&stdout).expect("Anthropic output should be valid JSON");

    assert!(
        parsed.get("tools").is_some(),
        "Anthropic format should have 'tools' key"
    );
    let tools = parsed["tools"].as_array().unwrap();
    assert!(
        !tools.is_empty(),
        "Should have at least one tool definition"
    );

    // Anthropic tools should have input_schema
    for tool in tools {
        assert!(
            tool.get("input_schema").is_some(),
            "Each Anthropic tool should have 'input_schema'"
        );
    }
}

#[test]
fn tools_jsonschema_format_should_produce_valid_json_with_tools() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("Failed to run tools --format jsonschema");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value =
        serde_json::from_str(&stdout).expect("JSON Schema output should be valid JSON");

    assert!(
        parsed.get("tools").is_some(),
        "JSON Schema format should have 'tools' key"
    );
    assert!(
        parsed.get("schema_version").is_some(),
        "Should have schema_version"
    );

    let tools = parsed["tools"].as_array().unwrap();
    assert!(
        !tools.is_empty(),
        "Should have at least one tool definition"
    );

    // Each tool should have name and description
    for tool in tools {
        assert!(
            tool.get("name").is_some(),
            "Each tool should have a 'name' field"
        );
        assert!(
            tool.get("description").is_some(),
            "Each tool should have a 'description' field"
        );
    }
}

#[test]
fn tools_openai_functions_should_have_parameters() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("Failed to run tools --format openai");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();

    let functions = parsed["functions"].as_array().unwrap();
    for func in functions {
        assert!(
            func.get("parameters").is_some(),
            "Each OpenAI function should have 'parameters'"
        );
        assert!(
            func.get("name").is_some(),
            "Each OpenAI function should have 'name'"
        );
    }
}

#[test]
fn tools_should_include_known_commands() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("Failed to run tools");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();

    let tools = parsed["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(names.contains(&"lang"), "Should include 'lang' command");
    assert!(names.contains(&"module"), "Should include 'module' command");
    assert!(names.contains(&"export"), "Should include 'export' command");
}

// ===========================================================================
// E) tokmd lang --json — JSON output validation
// ===========================================================================

#[test]
fn lang_json_should_produce_valid_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: Value = serde_json::from_str(&stdout).expect("lang JSON output must be valid JSON");
}

#[test]
fn lang_json_should_have_schema_version() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["schema_version"].is_number(),
        "schema_version field should be present and numeric"
    );
}

#[test]
fn lang_json_should_have_rows_array() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(json["rows"].is_array(), "rows should be an array");
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "rows should not be empty for a fixture");
}

#[test]
fn lang_json_should_have_mode_lang() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["mode"], "lang", "mode field should be 'lang'");
}

#[test]
fn lang_json_should_have_tool_metadata() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(json["tool"].is_object(), "tool metadata should be present");
    assert!(
        json["tool"]["name"].is_string(),
        "tool.name should be present"
    );
    assert!(
        json["tool"]["version"].is_string(),
        "tool.version should be present"
    );
}

#[test]
fn lang_json_should_have_total_section() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(json["total"].is_object(), "total section should be present");
    assert!(
        json["total"]["code"].is_number(),
        "total.code should be numeric"
    );
}

#[test]
fn lang_json_rows_should_have_required_fields() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        assert!(row["lang"].is_string(), "each row should have lang field");
        assert!(row["code"].is_number(), "each row should have code field");
        assert!(row["lines"].is_number(), "each row should have lines field");
    }
}

#[test]
fn lang_json_should_have_generated_at_ms() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run lang --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms should be present"
    );
}

// ===========================================================================
// F) tokmd module --json — JSON output validation
// ===========================================================================

#[test]
fn module_json_should_produce_valid_json() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: Value = serde_json::from_str(&stdout).expect("module JSON output must be valid JSON");
}

#[test]
fn module_json_should_have_schema_version() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["schema_version"].is_number(),
        "schema_version field should be present and numeric"
    );
}

#[test]
fn module_json_should_have_rows_array() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(json["rows"].is_array(), "rows should be an array");
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "rows should not be empty for a fixture");
}

#[test]
fn module_json_should_have_mode_module() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(json["mode"], "module", "mode field should be 'module'");
}

#[test]
fn module_json_should_have_tool_metadata() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(json["tool"].is_object(), "tool metadata should be present");
}

#[test]
fn module_json_should_have_total_section() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(json["total"].is_object(), "total section should be present");
}

#[test]
fn module_json_rows_should_have_module_field() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        assert!(
            row["module"].is_string(),
            "each row should have a module field"
        );
        assert!(row["code"].is_number(), "each row should have code field");
    }
}

#[test]
fn module_json_should_have_generated_at_ms() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms should be present"
    );
}

#[test]
fn module_json_should_contain_root_module() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run module --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let rows = json["rows"].as_array().unwrap();
    let has_root = rows.iter().any(|r| r["module"] == "(root)");
    assert!(has_root, "module output should contain (root) entry");
}

// ===========================================================================
// G) tokmd export --format jsonl and --format csv
// ===========================================================================

#[test]
fn export_jsonl_should_produce_valid_json_on_every_line() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("Failed to run export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(
        lines.len() >= 2,
        "JSONL should have at least meta + one data row"
    );

    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {} is not valid JSON: {}", i + 1, e));
    }
}

#[test]
fn export_jsonl_first_line_should_be_meta_record() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("Failed to run export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout
        .lines()
        .next()
        .expect("should have at least one line");
    let meta: Value = serde_json::from_str(first_line).expect("first line should be valid JSON");

    assert_eq!(
        meta["type"], "meta",
        "first JSONL line should have type=meta"
    );
    assert_eq!(
        meta["mode"], "export",
        "meta record should have mode=export"
    );
    assert!(
        meta["schema_version"].is_number(),
        "meta should have schema_version"
    );
}

#[test]
fn export_jsonl_data_rows_should_have_path_and_lang() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("Failed to run export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    // Skip meta line, check data rows
    for line in lines.iter().skip(1) {
        let row: Value = serde_json::from_str(line).unwrap();
        assert!(row["path"].is_string(), "data row should have a path field");
        assert!(row["lang"].is_string(), "data row should have a lang field");
    }
}

#[test]
fn export_csv_should_have_header_row() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("Failed to run export --format csv");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().expect("CSV should have a header row");

    assert!(
        first_line.contains("path") || first_line.contains("language"),
        "CSV header should contain column names like 'path' or 'language', got: {first_line}"
    );
}

#[test]
fn export_csv_should_have_data_rows() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("Failed to run export --format csv");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(
        lines.len() >= 2,
        "CSV should have header + at least one data row, got {} lines",
        lines.len()
    );
}

#[test]
fn export_csv_should_have_consistent_column_count() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("Failed to run export --format csv");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(lines.len() >= 2, "Need header + data rows");

    let header_cols = lines[0].split(',').count();
    for (i, line) in lines.iter().skip(1).enumerate() {
        let cols = line.split(',').count();
        assert_eq!(
            cols,
            header_cols,
            "Data row {} has {} columns but header has {}",
            i + 1,
            cols,
            header_cols
        );
    }
}

#[test]
fn export_jsonl_should_have_more_lines_than_just_meta() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("Failed to run export --format jsonl");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

    assert!(
        lines.len() > 1,
        "JSONL should contain data rows beyond the meta record"
    );
}

#[test]
fn export_json_should_have_rows_with_path_field() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("Failed to run export --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let rows = json["rows"].as_array().expect("should have rows array");
    assert!(!rows.is_empty(), "should have at least one file row");

    for row in rows {
        assert!(
            row["path"].is_string(),
            "each export row should have a path"
        );
        assert!(
            row["code"].is_number(),
            "each export row should have code count"
        );
    }
}
