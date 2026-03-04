//! End-to-end tests for `tokmd tools` — format variants, tool name
//! enumeration, and structural validation.

mod common;

use assert_cmd::Command;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// OpenAI format
// ---------------------------------------------------------------------------

#[test]
fn tools_openai_each_function_has_name_and_description() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let funcs = json["functions"].as_array().expect("functions array");

    for func in funcs {
        assert!(func["name"].is_string(), "each function needs a name");
        assert!(
            func["description"].is_string(),
            "each function needs a description"
        );
    }
}

#[test]
fn tools_openai_contains_lang_and_export() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let names: Vec<&str> = json["functions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();

    assert!(names.contains(&"lang"), "should contain lang");
    assert!(names.contains(&"export"), "should contain export");
}

// ---------------------------------------------------------------------------
// Anthropic format
// ---------------------------------------------------------------------------

#[test]
fn tools_anthropic_each_tool_has_name_and_input_schema() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let tools = json["tools"].as_array().expect("tools array");

    for tool in tools {
        assert!(tool["name"].is_string(), "each tool needs a name");
        assert!(
            tool["input_schema"].is_object(),
            "each tool needs input_schema"
        );
    }
}

#[test]
fn tools_anthropic_contains_module_and_analyze() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let names: Vec<&str> = json["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(names.contains(&"module"), "should contain module");
    assert!(names.contains(&"analyze"), "should contain analyze");
}

// ---------------------------------------------------------------------------
// JSON Schema format
// ---------------------------------------------------------------------------

#[test]
fn tools_jsonschema_has_name_and_tools_array() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["name"].is_string(), "envelope should have name");
    assert!(
        json["schema_version"].is_number(),
        "envelope should have schema_version"
    );

    let tools = json["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty());

    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["parameters"].is_object());
    }
}

#[test]
fn tools_jsonschema_contains_context_and_gate() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let names: Vec<&str> = json["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(names.contains(&"context"), "should contain context");
    assert!(names.contains(&"gate"), "should contain gate");
}

// ---------------------------------------------------------------------------
// Clap format
// ---------------------------------------------------------------------------

#[test]
fn tools_clap_format_has_tools_with_parameters() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "clap"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());

    let tools = json["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty());
    for tool in tools {
        assert!(tool["name"].is_string());
    }
}

// ---------------------------------------------------------------------------
// Error case
// ---------------------------------------------------------------------------

#[test]
fn tools_invalid_format_fails() {
    tokmd_cmd()
        .args(["tools", "--format", "yaml"])
        .assert()
        .failure();
}
