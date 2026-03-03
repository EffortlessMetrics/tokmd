//! Additional coverage tests for `tokmd-tool-schema`.
//!
//! Targets format-specific field presence, deeply nested commands,
//! trait behaviors, and edge cases not covered by BDD/unit suites.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{ToolSchemaFormat, ToolSchemaOutput, build_tool_schema, render_output};

// ── helpers ──────────────────────────────────────────────────────────────

fn empty_cmd() -> Command {
    Command::new("empty").version("0.0.1")
}

fn multi_required_cmd() -> Command {
    Command::new("app")
        .version("1.0.0")
        .about("Multi-required test")
        .subcommand(
            Command::new("deploy")
                .about("Deploy to target")
                .arg(
                    Arg::new("target")
                        .long("target")
                        .required(true)
                        .help("Deploy target"),
                )
                .arg(
                    Arg::new("region")
                        .long("region")
                        .required(true)
                        .help("AWS region"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Simulate deploy"),
                ),
        )
}

// ── OpenAI: enum values present in properties ───────────────────────────

#[test]
fn given_openai_format_when_arg_has_enum_then_enum_in_properties() {
    let cmd = Command::new("t").version("1.0.0").subcommand(
        Command::new("export").about("Export").arg(
            Arg::new("format")
                .long("format")
                .value_parser(["json", "csv", "md"])
                .help("Output format"),
        ),
    );

    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let export = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "export")
        .unwrap();

    let format_prop = &export["parameters"]["properties"]["format"];
    let enums = format_prop["enum"].as_array().unwrap();
    assert_eq!(enums.len(), 3);
    assert!(enums.iter().any(|v| v == "json"));
    assert!(enums.iter().any(|v| v == "csv"));
    assert!(enums.iter().any(|v| v == "md"));
}

// ── Anthropic: required array lists all required params ─────────────────

#[test]
fn given_anthropic_format_when_multiple_required_then_all_in_required_array() {
    let schema = build_tool_schema(&multi_required_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let deploy = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "deploy")
        .unwrap();

    let required: Vec<&str> = deploy["input_schema"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    assert!(required.contains(&"target"));
    assert!(required.contains(&"region"));
    assert!(!required.contains(&"dry-run"));
}

// ── JSON Schema: description field in properties ────────────────────────

#[test]
fn given_jsonschema_format_when_arg_has_help_then_description_in_properties() {
    let schema = build_tool_schema(&multi_required_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let deploy = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "deploy")
        .unwrap();

    let target_prop = &deploy["parameters"]["properties"]["target"];
    assert_eq!(target_prop["description"].as_str(), Some("Deploy target"));
}

// ── Empty CLI: no subcommands, no user args ─────────────────────────────

#[test]
fn given_empty_cmd_when_schema_built_then_root_tool_has_no_parameters() {
    let schema = build_tool_schema(&empty_cmd());
    assert_eq!(schema.tools.len(), 1);
    assert_eq!(schema.tools[0].name, "empty");
    assert!(schema.tools[0].parameters.is_empty());
}

#[test]
fn given_empty_cmd_when_rendered_in_all_formats_then_valid_json() {
    let schema = build_tool_schema(&empty_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let json_str = render_output(&schema, fmt, false).unwrap();
        let _: Value = serde_json::from_str(&json_str)
            .unwrap_or_else(|e| panic!("invalid JSON for format {:?}: {e}", fmt));
    }
}

// ── ToolSchemaOutput: serde roundtrip for all formats ───────────────────

#[test]
fn given_clap_format_output_when_deserialized_then_field_values_preserved() {
    let schema = build_tool_schema(&multi_required_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let deserialized: ToolSchemaOutput = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.name, "app");
    assert_eq!(deserialized.version, "1.0.0");
    assert_eq!(deserialized.description, "Multi-required test");

    let deploy = deserialized
        .tools
        .iter()
        .find(|t| t.name == "deploy")
        .unwrap();
    assert_eq!(deploy.parameters.len(), 3);

    let target = deploy
        .parameters
        .iter()
        .find(|p| p.name == "target")
        .unwrap();
    assert!(target.required);
    assert_eq!(target.param_type, "string");
}

// ── Determinism: same command produces same schema ──────────────────────

#[test]
fn given_same_command_when_schema_rendered_twice_then_output_identical() {
    let cmd = multi_required_cmd();
    let schema = build_tool_schema(&cmd);

    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let a = render_output(&schema, fmt, false).unwrap();
        let b = render_output(&schema, fmt, false).unwrap();
        assert_eq!(a, b, "render_output must be deterministic for {:?}", fmt);
    }
}

// ── ToolSchemaFormat: Default trait ─────────────────────────────────────

#[test]
fn given_tool_schema_format_when_default_then_jsonschema() {
    let fmt: ToolSchemaFormat = Default::default();
    assert_eq!(fmt, ToolSchemaFormat::Jsonschema);
}

// ── OpenAI: does not contain top-level schema or tools keys ─────────────

#[test]
fn given_openai_format_when_rendered_then_no_schema_or_tools_key() {
    let schema = build_tool_schema(&multi_required_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert!(v.get("$schema").is_none(), "OpenAI should not have $schema");
    assert!(v.get("tools").is_none(), "OpenAI should not have tools key");
    assert!(
        v.get("functions").is_some(),
        "OpenAI must have functions key"
    );
}
