//! Integration tests for the `tokmd tools` command.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd() -> Command {
    Command::cargo_bin("tokmd").unwrap()
}

#[test]
fn test_tools_jsonschema_output_parses() {
    let output = tokmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check envelope metadata
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("name").is_some());
    assert!(parsed.get("tools").is_some());
}

#[test]
fn test_tools_contains_known_commands() {
    let output = tokmd()
        .args(["tools", "--format", "jsonschema", "--pretty"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    let tools = parsed["tools"]
        .as_array()
        .expect("tools should be an array");

    // Check for known commands
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(
        tool_names.contains(&"lang"),
        "Should contain 'lang' command"
    );
    assert!(
        tool_names.contains(&"module"),
        "Should contain 'module' command"
    );
    assert!(
        tool_names.contains(&"export"),
        "Should contain 'export' command"
    );
    assert!(
        tool_names.contains(&"analyze"),
        "Should contain 'analyze' command"
    );
    assert!(
        tool_names.contains(&"context"),
        "Should contain 'context' command"
    );
    assert!(
        tool_names.contains(&"tools"),
        "Should contain 'tools' command"
    );
    assert!(
        tool_names.contains(&"gate"),
        "Should contain 'gate' command"
    );
}

#[test]
fn test_tools_openai_format() {
    let output = tokmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // OpenAI format has "functions" key
    assert!(
        parsed.get("functions").is_some(),
        "OpenAI format should have 'functions' key"
    );

    let functions = parsed["functions"]
        .as_array()
        .expect("functions should be an array");
    assert!(!functions.is_empty(), "Should have at least one function");

    // Each function should have "parameters"
    for func in functions {
        assert!(
            func.get("parameters").is_some(),
            "Each function should have 'parameters'"
        );
    }
}

#[test]
fn test_tools_anthropic_format() {
    let output = tokmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Anthropic format has "tools" key
    assert!(
        parsed.get("tools").is_some(),
        "Anthropic format should have 'tools' key"
    );

    let tools = parsed["tools"]
        .as_array()
        .expect("tools should be an array");
    assert!(!tools.is_empty(), "Should have at least one tool");

    // Each tool should have "input_schema"
    for tool in tools {
        assert!(
            tool.get("input_schema").is_some(),
            "Each tool should have 'input_schema'"
        );
    }
}

#[test]
fn test_tools_clap_format() {
    let output = tokmd()
        .args(["tools", "--format", "clap"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Clap format should have our internal structure
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("tools").is_some());
}

#[test]
fn test_tools_pretty_output() {
    let compact = tokmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("Failed to execute command");

    let pretty = tokmd()
        .args(["tools", "--format", "jsonschema", "--pretty"])
        .output()
        .expect("Failed to execute command");

    // Pretty output should have newlines and be longer
    let compact_len = compact.stdout.len();
    let pretty_len = pretty.stdout.len();

    assert!(
        pretty_len > compact_len,
        "Pretty output should be longer than compact"
    );

    let pretty_str = String::from_utf8_lossy(&pretty.stdout);
    assert!(
        pretty_str.contains('\n'),
        "Pretty output should contain newlines"
    );
}

#[test]
fn test_tools_export_formats_enum() {
    let output = tokmd()
        .args(["tools", "--format", "jsonschema", "--pretty"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Find the export tool
    let tools = parsed["tools"].as_array().unwrap();
    let export_tool = tools.iter().find(|t| t["name"] == "export");

    assert!(export_tool.is_some(), "Should have export tool");

    // The export tool should exist and have parameters object
    let params = &export_tool.unwrap()["parameters"];
    assert!(
        params.is_object(),
        "Export tool should have parameters object"
    );

    // Check properties contains format
    let properties = &params["properties"];
    assert!(properties.is_object(), "Parameters should have properties");
    assert!(
        properties.get("format").is_some(),
        "Export should have format parameter"
    );
}
