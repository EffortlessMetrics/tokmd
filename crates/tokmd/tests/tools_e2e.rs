//! E2E tests for the `tokmd tools` CLI command.
//!
//! Tests exercise tool-schema generation in OpenAI, Anthropic, and
//! JSON Schema formats, verifying structure and content.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Helper: run `tokmd tools --format <fmt> --pretty` and return parsed JSON.
fn run_tools(format: &str) -> Value {
    let output = tokmd_cmd()
        .args(["tools", "--format", format, "--pretty"])
        .output()
        .expect("failed to execute tokmd tools");
    assert!(output.status.success(), "tokmd tools exited with failure");
    let stdout = String::from_utf8(output.stdout).unwrap();
    serde_json::from_str(&stdout).expect("output should be valid JSON")
}

// ---------------------------------------------------------------------------
// 1. OpenAI format produces valid JSON with functions array
// ---------------------------------------------------------------------------

#[test]
fn openai_produces_valid_json_with_functions_array() {
    let json = run_tools("openai");
    let functions = json
        .get("functions")
        .expect("OpenAI output must have 'functions' key");
    assert!(functions.is_array(), "'functions' must be an array");
    assert!(
        !functions.as_array().unwrap().is_empty(),
        "functions array must not be empty"
    );
}

// ---------------------------------------------------------------------------
// 2. Anthropic format produces valid JSON
// ---------------------------------------------------------------------------

#[test]
fn anthropic_produces_valid_json() {
    let json = run_tools("anthropic");
    let tools = json
        .get("tools")
        .expect("Anthropic output must have 'tools' key");
    assert!(tools.is_array(), "'tools' must be an array");
    assert!(
        !tools.as_array().unwrap().is_empty(),
        "tools array must not be empty"
    );
}

// ---------------------------------------------------------------------------
// 3. JSON Schema format produces valid JSON Schema
// ---------------------------------------------------------------------------

#[test]
fn jsonschema_produces_valid_json_schema() {
    let json = run_tools("jsonschema");
    assert!(
        json.get("$schema").is_some(),
        "JSON Schema output must have '$schema' key"
    );
    assert!(
        json.get("tools").is_some(),
        "JSON Schema output must have 'tools' key"
    );
    assert!(
        json.get("schema_version").is_some(),
        "JSON Schema output must have 'schema_version'"
    );
    assert!(
        json.get("name").is_some(),
        "JSON Schema output must have 'name'"
    );
}

// ---------------------------------------------------------------------------
// 4. Output contains expected tool names
// ---------------------------------------------------------------------------

#[test]
fn output_contains_expected_tool_names() {
    let json = run_tools("jsonschema");
    let tools = json["tools"].as_array().expect("tools must be an array");
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    for expected in ["lang", "module", "export"] {
        assert!(
            names.contains(&expected),
            "expected tool '{}' not found in {:?}",
            expected,
            names
        );
    }
}

// ---------------------------------------------------------------------------
// 5. OpenAI format entries have function-style structure
// ---------------------------------------------------------------------------

#[test]
fn openai_entries_have_function_structure() {
    let json = run_tools("openai");
    let functions = json["functions"].as_array().unwrap();

    for func in functions {
        assert!(func.get("name").is_some(), "each function must have 'name'");
        assert!(
            func.get("description").is_some(),
            "each function must have 'description'"
        );
        assert!(
            func.get("parameters").is_some(),
            "each function must have 'parameters'"
        );
        let params = &func["parameters"];
        assert_eq!(
            params["type"].as_str(),
            Some("object"),
            "parameters.type must be 'object'"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Anthropic format has proper input_schema structure
// ---------------------------------------------------------------------------

#[test]
fn anthropic_has_input_schema_structure() {
    let json = run_tools("anthropic");
    let tools = json["tools"].as_array().unwrap();

    for tool in tools {
        assert!(tool.get("name").is_some(), "each tool must have 'name'");
        assert!(
            tool.get("description").is_some(),
            "each tool must have 'description'"
        );
        let input_schema = tool
            .get("input_schema")
            .expect("Anthropic tool must have 'input_schema'");
        assert_eq!(
            input_schema["type"].as_str(),
            Some("object"),
            "input_schema.type must be 'object'"
        );
        assert!(
            input_schema.get("properties").is_some(),
            "input_schema must have 'properties'"
        );
    }
}

// ---------------------------------------------------------------------------
// 7. Each tool has name, description, and parameters
// ---------------------------------------------------------------------------

#[test]
fn each_tool_has_name_description_parameters() {
    // Verify across all three formats.
    for format in ["openai", "anthropic", "jsonschema"] {
        let json = run_tools(format);

        let tools_key = match format {
            "openai" => "functions",
            _ => "tools",
        };
        let tools = json[tools_key]
            .as_array()
            .unwrap_or_else(|| panic!("{format}: '{tools_key}' must be an array"));

        for tool in tools {
            let name = tool["name"]
                .as_str()
                .unwrap_or_else(|| panic!("{format}: tool must have string 'name'"));
            assert!(
                tool.get("description").is_some(),
                "{format}/{name}: missing 'description'"
            );

            let params_key = match format {
                "anthropic" => "input_schema",
                _ => "parameters",
            };
            assert!(
                tool.get(params_key).is_some(),
                "{format}/{name}: missing '{params_key}'"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Default format (no --format) produces jsonschema output
// ---------------------------------------------------------------------------

#[test]
fn default_format_produces_jsonschema() {
    let output = tokmd_cmd()
        .args(["tools", "--pretty"])
        .output()
        .expect("failed to execute tokmd tools");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    // Default is jsonschema, so it should have $schema key.
    assert!(
        json.get("$schema").is_some(),
        "default format should produce JSON Schema with '$schema' key"
    );
    assert!(json.get("tools").is_some());
}

// ---------------------------------------------------------------------------
// 9. Pretty flag produces indented output
// ---------------------------------------------------------------------------

#[test]
fn pretty_flag_produces_indented_output() {
    tokmd_cmd()
        .args(["tools", "--format", "openai", "--pretty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("  "));
}

// ---------------------------------------------------------------------------
// 10. Compact output (no --pretty) is single-line
// ---------------------------------------------------------------------------

#[test]
fn compact_output_is_single_line() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("failed to execute tokmd tools");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 1, "compact output should be a single line");
}

// ---------------------------------------------------------------------------
// 11. Clap format produces valid JSON with tools array
// ---------------------------------------------------------------------------

#[test]
fn clap_format_produces_valid_json() {
    let json = run_tools("clap");
    assert!(json.get("tools").is_some(), "clap output must have 'tools'");
    assert!(
        json.get("schema_version").is_some(),
        "clap output must have 'schema_version'"
    );
    assert!(json.get("name").is_some(), "clap output must have 'name'");
}
