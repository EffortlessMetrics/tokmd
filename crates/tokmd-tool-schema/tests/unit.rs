//! Unit tests for tokmd-tool-schema.
//!
//! Tests cover schema generation from clap command trees, JSON output format
//! correctness, and OpenAI/Anthropic/JSON Schema format variants.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{
    ParameterSchema, TOOL_SCHEMA_VERSION, ToolDefinition, ToolSchemaFormat, ToolSchemaOutput,
    build_tool_schema, render_output,
};

// ── helpers ──────────────────────────────────────────────────────────────

fn simple_cmd() -> Command {
    Command::new("simple").version("1.0.0").about("Simple app")
}

fn nested_cmd() -> Command {
    Command::new("cli")
        .version("2.0.0")
        .about("CLI with nested commands")
        .subcommand(
            Command::new("scan")
                .about("Scan files")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .required(true)
                        .help("Path to scan"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(ArgAction::Count)
                        .help("Verbosity level"),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "md"])
                        .default_value("json")
                        .help("Output format"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Dry run mode"),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .action(ArgAction::Append)
                        .help("Add tags"),
                ),
        )
        .subcommand(
            Command::new("export").about("Export data").arg(
                Arg::new("output")
                    .long("output")
                    .required(true)
                    .help("Output file"),
            ),
        )
}

// ── 1. Schema envelope metadata ─────────────────────────────────────────

#[test]
fn schema_envelope_has_correct_metadata() {
    let cmd = simple_cmd();
    let schema = build_tool_schema(&cmd);

    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
    assert_eq!(schema.name, "simple");
    assert_eq!(schema.version, "1.0.0");
    assert_eq!(schema.description, "Simple app");
}

// ── 2. Root command appears as first tool ────────────────────────────────

#[test]
fn root_command_is_always_first_tool() {
    let schema = build_tool_schema(&nested_cmd());
    assert_eq!(schema.tools[0].name, "cli");
}

// ── 3. Subcommands are included ─────────────────────────────────────────

#[test]
fn all_subcommands_appear_in_tools() {
    let schema = build_tool_schema(&nested_cmd());
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"scan"));
    assert!(names.contains(&"export"));
    // help subcommand should be filtered out
    assert!(!names.contains(&"help"));
}

// ── 4. Parameter type inference ─────────────────────────────────────────

#[test]
fn parameter_types_are_correctly_inferred() {
    let schema = build_tool_schema(&nested_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let find_param = |name: &str| -> &ParameterSchema {
        scan.parameters.iter().find(|p| p.name == name).unwrap()
    };

    // String arg
    assert_eq!(find_param("path").param_type, "string");
    // Count arg → integer
    assert_eq!(find_param("verbose").param_type, "integer");
    // SetTrue arg → boolean
    assert_eq!(find_param("dry-run").param_type, "boolean");
    // Append arg → array
    assert_eq!(find_param("tags").param_type, "array");
}

// ── 5. Required and optional flags ──────────────────────────────────────

#[test]
fn required_and_optional_params_are_distinguished() {
    let schema = build_tool_schema(&nested_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.required, "path should be required");

    let dry_run = scan
        .parameters
        .iter()
        .find(|p| p.name == "dry-run")
        .unwrap();
    assert!(!dry_run.required, "dry-run should be optional");
}

// ── 6. Default values and enum values ───────────────────────────────────

#[test]
fn default_and_enum_values_are_captured() {
    let schema = build_tool_schema(&nested_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let format = scan.parameters.iter().find(|p| p.name == "format").unwrap();

    assert_eq!(format.default.as_deref(), Some("json"));
    let enums = format.enum_values.as_ref().unwrap();
    assert_eq!(enums, &["json", "csv", "md"]);
}

// ── 7. OpenAI format structure ──────────────────────────────────────────

#[test]
fn openai_format_has_correct_structure() {
    let schema = build_tool_schema(&nested_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    // Top-level has "functions" array
    assert!(v["functions"].is_array());
    let functions = v["functions"].as_array().unwrap();
    assert!(!functions.is_empty());

    // Each function has name, description, parameters
    for func in functions {
        assert!(func["name"].is_string());
        assert!(func["description"].is_string());
        assert_eq!(func["parameters"]["type"].as_str(), Some("object"));
        assert!(func["parameters"]["properties"].is_object());
        assert!(func["parameters"]["required"].is_array());
    }

    // OpenAI should NOT have top-level "tools" key
    assert!(v.get("tools").is_none());
}

// ── 8. Anthropic format structure ───────────────────────────────────────

#[test]
fn anthropic_format_uses_input_schema() {
    let schema = build_tool_schema(&nested_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    // Top-level has "tools" array
    assert!(v["tools"].is_array());
    let tools = v["tools"].as_array().unwrap();
    assert!(!tools.is_empty());

    // Each tool has name, description, input_schema (not "parameters")
    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert!(tool["input_schema"].is_object());
        assert_eq!(tool["input_schema"]["type"].as_str(), Some("object"));
        // Anthropic format should NOT have "parameters" key on tools
        assert!(tool.get("parameters").is_none());
    }
}

// ── 9. JSON Schema format structure ─────────────────────────────────────

#[test]
fn jsonschema_format_has_draft_reference_and_envelope() {
    let schema = build_tool_schema(&nested_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        v["$schema"].as_str(),
        Some("https://json-schema.org/draft-07/schema#")
    );
    assert_eq!(
        v["schema_version"].as_u64(),
        Some(TOOL_SCHEMA_VERSION as u64)
    );
    assert_eq!(v["name"].as_str(), Some("cli"));
    assert_eq!(v["version"].as_str(), Some("2.0.0"));
    assert!(v["tools"].is_array());

    // JSON Schema format should include default values in properties
    let scan = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "scan")
        .unwrap();
    let format_prop = &scan["parameters"]["properties"]["format"];
    assert_eq!(format_prop["default"].as_str(), Some("json"));
}

// ── 10. Clap format round-trip ──────────────────────────────────────────

#[test]
fn clap_format_round_trips_through_serde() {
    let schema = build_tool_schema(&nested_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();

    let deserialized: ToolSchemaOutput = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.name, schema.name);
    assert_eq!(deserialized.version, schema.version);
    assert_eq!(deserialized.schema_version, schema.schema_version);
    assert_eq!(deserialized.tools.len(), schema.tools.len());

    // Verify tool names match
    let orig_names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    let deser_names: Vec<&str> = deserialized.tools.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(orig_names, deser_names);
}

// ── 11. Pretty vs compact output ────────────────────────────────────────

#[test]
fn pretty_and_compact_produce_equivalent_values() {
    let schema = build_tool_schema(&nested_cmd());

    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let pretty_str = render_output(&schema, fmt, true).unwrap();
        let compact_str = render_output(&schema, fmt, false).unwrap();

        // Pretty has newlines, compact does not
        assert!(pretty_str.contains('\n'), "Pretty should contain newlines");
        assert!(
            !compact_str.contains('\n'),
            "Compact should not contain newlines"
        );

        // Both parse to the same JSON value
        let pretty_val: Value = serde_json::from_str(&pretty_str).unwrap();
        let compact_val: Value = serde_json::from_str(&compact_str).unwrap();
        assert_eq!(pretty_val, compact_val, "Mismatch for format {:?}", fmt);
    }
}

// ── 12. OpenAI required array correctness ───────────────────────────────

#[test]
fn openai_required_array_only_contains_required_params() {
    let schema = build_tool_schema(&nested_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let scan = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "scan")
        .unwrap();

    let required: Vec<&str> = scan["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    assert!(required.contains(&"path"), "path should be required");
    assert!(
        !required.contains(&"verbose"),
        "verbose should not be required"
    );
    assert!(
        !required.contains(&"dry-run"),
        "dry-run should not be required"
    );
    assert!(!required.contains(&"tags"), "tags should not be required");
}

// ── 13. ToolDefinition and ParameterSchema are serializable ─────────────

#[test]
fn tool_definition_and_parameter_schema_serde_roundtrip() {
    let param = ParameterSchema {
        name: "input".to_string(),
        description: Some("Input file".to_string()),
        param_type: "string".to_string(),
        required: true,
        default: None,
        enum_values: Some(vec!["a".to_string(), "b".to_string()]),
    };

    let tool = ToolDefinition {
        name: "my-tool".to_string(),
        description: "Does things".to_string(),
        parameters: vec![param],
    };

    let json = serde_json::to_string(&tool).unwrap();
    let deser: ToolDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, "my-tool");
    assert_eq!(deser.parameters.len(), 1);
    assert_eq!(deser.parameters[0].name, "input");
    assert!(deser.parameters[0].required);
    assert_eq!(
        deser.parameters[0].enum_values,
        Some(vec!["a".to_string(), "b".to_string()])
    );
}
