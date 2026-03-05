//! Deep tests for tokmd-tool-schema (W69).
//!
//! Covers OpenAI, Anthropic, JSON Schema, and Clap format rendering,
//! deterministic output, envelope metadata, and parameter introspection.

use tokmd_tool_schema::{
    build_tool_schema, render_output, ToolSchemaFormat, TOOL_SCHEMA_VERSION,
};

use clap::{Arg, ArgAction, Command};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_cmd() -> Command {
    Command::new("mycli")
        .version("2.0.0")
        .about("A sample CLI")
        .subcommand(
            Command::new("scan")
                .about("Scan a directory")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .required(true)
                        .help("Target path"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(ArgAction::SetTrue)
                        .help("Enable verbose output"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export results")
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "tsv"])
                        .help("Output format"),
                )
                .arg(
                    Arg::new("count")
                        .long("count")
                        .action(ArgAction::Count)
                        .help("Verbosity level"),
                ),
        )
}

fn minimal_cmd() -> Command {
    Command::new("tiny").version("0.1.0").about("Tiny tool")
}

fn multi_arg_cmd() -> Command {
    Command::new("multi")
        .version("1.0.0")
        .about("Multi-arg tool")
        .arg(
            Arg::new("items")
                .long("items")
                .action(ArgAction::Append)
                .help("Repeated items"),
        )
        .arg(
            Arg::new("flag_a")
                .long("flag-a")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("flag_b")
                .long("flag-b")
                .action(ArgAction::SetFalse),
        )
        .arg(
            Arg::new("name")
                .long("name")
                .default_value("world")
                .help("Greeting target"),
        )
}

// ---------------------------------------------------------------------------
// 1. Schema version constant
// ---------------------------------------------------------------------------

#[test]
fn w69_schema_version_is_positive() {
    assert!(TOOL_SCHEMA_VERSION > 0);
}

// ---------------------------------------------------------------------------
// 2. build_tool_schema envelope fields
// ---------------------------------------------------------------------------

#[test]
fn w69_build_schema_envelope_fields() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.name, "mycli");
    assert_eq!(schema.version, "2.0.0");
    assert_eq!(schema.description, "A sample CLI");
    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// 3. Subcommands are collected (help excluded)
// ---------------------------------------------------------------------------

#[test]
fn w69_build_schema_collects_subcommands() {
    let schema = build_tool_schema(&simple_cmd());
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"scan"));
    assert!(names.contains(&"export"));
    assert!(!names.contains(&"help"));
}

// ---------------------------------------------------------------------------
// 4. Root command included as a tool
// ---------------------------------------------------------------------------

#[test]
fn w69_root_command_included() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(schema.tools.iter().any(|t| t.name == "mycli"));
}

// ---------------------------------------------------------------------------
// 5. Minimal command with no subcommands
// ---------------------------------------------------------------------------

#[test]
fn w69_minimal_cmd_single_tool() {
    let schema = build_tool_schema(&minimal_cmd());
    assert_eq!(schema.tools.len(), 1);
    assert_eq!(schema.tools[0].name, "tiny");
}

// ---------------------------------------------------------------------------
// 6. Parameter type detection – boolean
// ---------------------------------------------------------------------------

#[test]
fn w69_param_type_boolean() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let verbose = scan.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert_eq!(verbose.param_type, "boolean");
}

// ---------------------------------------------------------------------------
// 7. Parameter type detection – count → integer
// ---------------------------------------------------------------------------

#[test]
fn w69_param_type_count_integer() {
    let schema = build_tool_schema(&simple_cmd());
    let export = schema.tools.iter().find(|t| t.name == "export").unwrap();
    let count = export.parameters.iter().find(|p| p.name == "count").unwrap();
    assert_eq!(count.param_type, "integer");
}

// ---------------------------------------------------------------------------
// 8. Parameter type detection – append → array
// ---------------------------------------------------------------------------

#[test]
fn w69_param_type_append_array() {
    let schema = build_tool_schema(&multi_arg_cmd());
    let root = &schema.tools[0];
    let items = root.parameters.iter().find(|p| p.name == "items").unwrap();
    assert_eq!(items.param_type, "array");
}

// ---------------------------------------------------------------------------
// 9. Parameter type detection – SetFalse → boolean
// ---------------------------------------------------------------------------

#[test]
fn w69_param_type_set_false_boolean() {
    let schema = build_tool_schema(&multi_arg_cmd());
    let root = &schema.tools[0];
    let flag_b = root.parameters.iter().find(|p| p.name == "flag_b").unwrap();
    assert_eq!(flag_b.param_type, "boolean");
}

// ---------------------------------------------------------------------------
// 10. Required flag propagation
// ---------------------------------------------------------------------------

#[test]
fn w69_required_flag() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.required);
    let verbose = scan.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert!(!verbose.required);
}

// ---------------------------------------------------------------------------
// 11. Enum values (possible_values)
// ---------------------------------------------------------------------------

#[test]
fn w69_enum_values_extracted() {
    let schema = build_tool_schema(&simple_cmd());
    let export = schema.tools.iter().find(|t| t.name == "export").unwrap();
    let fmt = export.parameters.iter().find(|p| p.name == "format").unwrap();
    let enums = fmt.enum_values.as_ref().expect("should have enum values");
    assert!(enums.contains(&"json".to_string()));
    assert!(enums.contains(&"csv".to_string()));
    assert!(enums.contains(&"tsv".to_string()));
}

// ---------------------------------------------------------------------------
// 12. Default value extraction
// ---------------------------------------------------------------------------

#[test]
fn w69_default_value_extracted() {
    let schema = build_tool_schema(&multi_arg_cmd());
    let root = &schema.tools[0];
    let name = root.parameters.iter().find(|p| p.name == "name").unwrap();
    assert_eq!(name.default.as_deref(), Some("world"));
}

// ---------------------------------------------------------------------------
// 13. OpenAI format – top-level key
// ---------------------------------------------------------------------------

#[test]
fn w69_openai_has_functions_key() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.get("functions").is_some());
}

// ---------------------------------------------------------------------------
// 14. OpenAI format – parameters.type is "object"
// ---------------------------------------------------------------------------

#[test]
fn w69_openai_parameters_type_object() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let func = &parsed["functions"][0];
    assert_eq!(func["parameters"]["type"], "object");
}

// ---------------------------------------------------------------------------
// 15. Anthropic format – has input_schema
// ---------------------------------------------------------------------------

#[test]
fn w69_anthropic_has_input_schema() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let tools = parsed["tools"].as_array().unwrap();
    for tool in tools {
        assert!(tool.get("input_schema").is_some());
    }
}

// ---------------------------------------------------------------------------
// 16. Anthropic format – no "functions" key (differs from OpenAI)
// ---------------------------------------------------------------------------

#[test]
fn w69_anthropic_no_functions_key() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.get("functions").is_none());
}

// ---------------------------------------------------------------------------
// 17. JSON Schema format – $schema field
// ---------------------------------------------------------------------------

#[test]
fn w69_jsonschema_has_dollar_schema() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        parsed["$schema"],
        "https://json-schema.org/draft-07/schema#"
    );
}

// ---------------------------------------------------------------------------
// 18. Clap format – serializes ToolSchemaOutput directly
// ---------------------------------------------------------------------------

#[test]
fn w69_clap_format_has_tools_array() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert!(parsed["tools"].is_array());
    assert_eq!(parsed["schema_version"], TOOL_SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// 19. Deterministic output – two calls produce identical JSON
// ---------------------------------------------------------------------------

#[test]
fn w69_deterministic_output() {
    let cmd1 = simple_cmd();
    let cmd2 = simple_cmd();
    let s1 = build_tool_schema(&cmd1);
    let s2 = build_tool_schema(&cmd2);

    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let r1 = render_output(&s1, fmt, true).unwrap();
        let r2 = render_output(&s2, fmt, true).unwrap();
        assert_eq!(r1, r2, "non-deterministic output for format {:?}", fmt);
    }
}

// ---------------------------------------------------------------------------
// 20. Pretty vs compact output
// ---------------------------------------------------------------------------

#[test]
fn w69_pretty_vs_compact() {
    let schema = build_tool_schema(&simple_cmd());
    let pretty = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    let compact = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    assert!(pretty.len() > compact.len());
    assert!(pretty.contains('\n'));
    // Both parse to the same value.
    let p: Value = serde_json::from_str(&pretty).unwrap();
    let c: Value = serde_json::from_str(&compact).unwrap();
    assert_eq!(p, c);
}
