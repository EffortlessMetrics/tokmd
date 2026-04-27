#![cfg(feature = "analysis")]

//! End-to-end tests for `tokmd tools` — format variants, tool name
//! enumeration, and structural validation.

mod common;

use anyhow::Context;
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
fn tools_openai_each_function_has_name_and_description() -> anyhow::Result<()> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .context("failed to run tokmd tools --format openai")?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let funcs = json["functions"]
        .as_array()
        .context("OpenAI functions array is missing or not an array")?;

    for func in funcs {
        assert!(func["name"].is_string(), "each function needs a name");
        assert!(
            func["description"].is_string(),
            "each function needs a description"
        );
    }
    Ok(())
}

#[test]
fn tools_openai_contains_lang_and_export() -> anyhow::Result<()> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .context("failed to run tokmd tools --format openai")?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let names: Vec<&str> = json["functions"]
        .as_array()
        .context("OpenAI functions array is missing")?
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();

    assert!(names.contains(&"lang"), "should contain lang");
    assert!(names.contains(&"export"), "should contain export");
    Ok(())
}

// ---------------------------------------------------------------------------
// Anthropic format
// ---------------------------------------------------------------------------

#[test]
fn tools_anthropic_each_tool_has_name_and_input_schema() -> anyhow::Result<()> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .context("failed to run tokmd tools --format anthropic")?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let tools = json["tools"]
        .as_array()
        .context("Anthropic tools array is missing")?;

    for tool in tools {
        assert!(tool["name"].is_string(), "each tool needs a name");
        assert!(
            tool["input_schema"].is_object(),
            "each tool needs input_schema"
        );
    }
    Ok(())
}

#[test]
fn tools_anthropic_contains_module_and_analyze() -> anyhow::Result<()> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .context("failed to run tokmd tools --format anthropic")?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let names: Vec<&str> = json["tools"]
        .as_array()
        .context("Anthropic tools array is missing")?
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(names.contains(&"module"), "should contain module");
    assert!(names.contains(&"analyze"), "should contain analyze");
    Ok(())
}

// ---------------------------------------------------------------------------
// JSON Schema format
// ---------------------------------------------------------------------------

#[test]
fn tools_jsonschema_has_name_and_tools_array() -> anyhow::Result<()> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .context("failed to run tokmd tools --format jsonschema")?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["name"].is_string(), "envelope should have name");
    assert!(
        json["schema_version"].is_number(),
        "envelope should have schema_version"
    );

    let tools = json["tools"]
        .as_array()
        .context("JSON Schema tools array is missing")?;
    assert!(!tools.is_empty());

    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["parameters"].is_object());
    }
    Ok(())
}

#[test]
fn tools_jsonschema_contains_context_and_gate() -> anyhow::Result<()> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .context("failed to run tokmd tools --format jsonschema")?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let names: Vec<&str> = json["tools"]
        .as_array()
        .context("JSON Schema tools array is missing")?
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(names.contains(&"context"), "should contain context");
    assert!(names.contains(&"gate"), "should contain gate");
    Ok(())
}

// ---------------------------------------------------------------------------
// Clap format
// ---------------------------------------------------------------------------

#[test]
fn tools_clap_format_has_tools_with_parameters() -> anyhow::Result<()> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "clap"])
        .output()
        .context("failed to run tokmd tools --format clap")?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["schema_version"].is_number());

    let tools = json["tools"]
        .as_array()
        .context("Clap tools array is missing")?;
    assert!(!tools.is_empty());
    for tool in tools {
        assert!(tool["name"].is_string());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Error case
// ---------------------------------------------------------------------------

#[test]
fn tools_invalid_format_fails() -> anyhow::Result<()> {
    tokmd_cmd()
        .args(["tools", "--format", "yaml"])
        .assert()
        .failure();
    Ok(())
}
