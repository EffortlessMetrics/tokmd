//! BDD-style scenario tests for tool-schema generation.
//!
//! Each test reads as a Given/When/Then scenario covering one behavioural
//! facet of the schema builder or renderer.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{TOOL_SCHEMA_VERSION, ToolSchemaFormat, build_tool_schema, render_output};

// ── helpers ──────────────────────────────────────────────────────────────

fn minimal_cmd() -> Command {
    Command::new("app").version("0.1.0").about("A minimal app")
}

fn rich_cmd() -> Command {
    Command::new("rich")
        .version("2.0.0")
        .about("A rich CLI")
        .subcommand(
            Command::new("scan")
                .about("Scan a directory")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .required(true)
                        .help("Directory to scan"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
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
                    Arg::new("flag")
                        .long("flag")
                        .action(ArgAction::SetTrue)
                        .help("A boolean flag"),
                )
                .arg(
                    Arg::new("items")
                        .long("items")
                        .action(ArgAction::Append)
                        .help("Repeatable item list"),
                ),
        )
        .subcommand(
            Command::new("export").about("Export results").arg(
                Arg::new("output")
                    .long("output")
                    .required(true)
                    .help("Output file path"),
            ),
        )
}

// ── Scenario: Schema envelope metadata ──────────────────────────────────

#[test]
fn schema_envelope_contains_version_name_and_description() {
    // Given a command with version and about text
    let cmd = minimal_cmd();

    // When we build the schema
    let schema = build_tool_schema(&cmd);

    // Then the envelope captures all metadata
    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
    assert_eq!(schema.name, "app");
    assert_eq!(schema.version, "0.1.0");
    assert_eq!(schema.description, "A minimal app");
}

#[test]
fn schema_version_without_version_says_unknown() {
    // Given a command without a version
    let cmd = Command::new("bare").about("No version");

    // When we build the schema
    let schema = build_tool_schema(&cmd);

    // Then version falls back to "unknown"
    assert_eq!(schema.version, "unknown");
}

#[test]
fn schema_description_without_about_is_empty() {
    // Given a command without about text
    let cmd = Command::new("bare").version("1.0.0");

    // When we build the schema
    let schema = build_tool_schema(&cmd);

    // Then description is empty
    assert_eq!(schema.description, "");
}

// ── Scenario: Root command and subcommands ──────────────────────────────

#[test]
fn root_command_is_first_tool() {
    // Given a CLI with subcommands
    let cmd = rich_cmd();

    // When we build the schema
    let schema = build_tool_schema(&cmd);

    // Then the root command appears as the first tool
    assert_eq!(schema.tools[0].name, "rich");
}

#[test]
fn subcommands_are_included_in_order() {
    // Given a CLI with scan and export subcommands
    let cmd = rich_cmd();

    // When we build the schema
    let schema = build_tool_schema(&cmd);

    // Then both subcommands appear after root
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"scan"));
    assert!(names.contains(&"export"));
}

#[test]
fn help_subcommand_is_excluded() {
    // Given a CLI (clap auto-generates a `help` subcommand)
    let cmd = rich_cmd();

    // When we build the schema
    let schema = build_tool_schema(&cmd);

    // Then the help subcommand is filtered out
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(!names.contains(&"help"));
}

#[test]
fn command_with_no_subcommands_has_only_root() {
    // Given a minimal CLI with no subcommands
    let cmd = minimal_cmd();

    // When we build the schema
    let schema = build_tool_schema(&cmd);

    // Then there is exactly one tool (the root)
    assert_eq!(schema.tools.len(), 1);
    assert_eq!(schema.tools[0].name, "app");
}

// ── Scenario: Parameter type inference ──────────────────────────────────

#[test]
fn boolean_flag_has_type_boolean() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let flag = scan.parameters.iter().find(|p| p.name == "flag").unwrap();
    assert_eq!(flag.param_type, "boolean");
}

#[test]
fn count_arg_has_type_integer() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let verbose = scan
        .parameters
        .iter()
        .find(|p| p.name == "verbose")
        .unwrap();
    assert_eq!(verbose.param_type, "integer");
}

#[test]
fn append_arg_has_type_array() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let items = scan.parameters.iter().find(|p| p.name == "items").unwrap();
    assert_eq!(items.param_type, "array");
}

#[test]
fn regular_arg_has_type_string() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path.param_type, "string");
}

// ── Scenario: Required / optional / defaults / enums ────────────────────

#[test]
fn required_parameter_is_marked_required() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.required);
}

#[test]
fn optional_parameter_is_not_required() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let flag = scan.parameters.iter().find(|p| p.name == "flag").unwrap();
    assert!(!flag.required);
}

#[test]
fn default_value_is_captured() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let format = scan.parameters.iter().find(|p| p.name == "format").unwrap();
    assert_eq!(format.default.as_deref(), Some("json"));
}

#[test]
fn enum_values_are_captured() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let format = scan.parameters.iter().find(|p| p.name == "format").unwrap();
    let enums = format.enum_values.as_ref().unwrap();
    assert_eq!(enums, &["json", "csv", "md"]);
}

#[test]
fn parameter_without_possible_values_has_no_enum() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.enum_values.is_none());
}

#[test]
fn help_text_becomes_description() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();

    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path.description.as_deref(), Some("Directory to scan"));
}

#[test]
fn arg_without_help_has_no_description() {
    let cmd = Command::new("t")
        .version("0.0.1")
        .subcommand(Command::new("sub").arg(Arg::new("bare").long("bare")));

    let schema = build_tool_schema(&cmd);
    let sub = schema.tools.iter().find(|t| t.name == "sub").unwrap();
    let bare = sub.parameters.iter().find(|p| p.name == "bare").unwrap();
    assert!(bare.description.is_none());
}

// ── Scenario: help and version args are excluded ────────────────────────

#[test]
fn help_and_version_args_are_filtered_out() {
    let cmd = minimal_cmd();
    let schema = build_tool_schema(&cmd);

    let root = &schema.tools[0];
    let names: Vec<&str> = root.parameters.iter().map(|p| p.name.as_str()).collect();
    assert!(!names.contains(&"help"));
    assert!(!names.contains(&"version"));
}

// ── Scenario: OpenAI format ─────────────────────────────────────────────

#[test]
fn openai_output_wraps_tools_in_functions_array() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert!(v["functions"].is_array());
    assert!(!v["functions"].as_array().unwrap().is_empty());
}

#[test]
fn openai_tool_has_name_description_parameters() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let func = &v["functions"][0];
    assert!(func["name"].is_string());
    assert!(func["description"].is_string());
    assert!(func["parameters"]["type"].as_str() == Some("object"));
    assert!(func["parameters"]["properties"].is_object());
    assert!(func["parameters"]["required"].is_array());
}

#[test]
fn openai_required_param_listed_in_required_array() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    // Find the scan function
    let scan = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "scan")
        .unwrap();

    let required = scan["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();

    assert!(required.contains(&"path"));
    assert!(!required.contains(&"flag"));
}

// ── Scenario: Anthropic format ──────────────────────────────────────────

#[test]
fn anthropic_output_wraps_tools_in_tools_array() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert!(v["tools"].is_array());
}

#[test]
fn anthropic_tool_uses_input_schema_key() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let tool = &v["tools"][0];
    assert!(tool["input_schema"].is_object());
    assert_eq!(tool["input_schema"]["type"].as_str(), Some("object"));
}

#[test]
fn anthropic_does_not_have_parameters_key() {
    // Anthropic uses `input_schema`, not `parameters`
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let tool = &v["tools"][0];
    assert!(tool.get("parameters").is_none());
}

// ── Scenario: JSON Schema format ────────────────────────────────────────

#[test]
fn jsonschema_output_has_schema_draft_reference() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        v["$schema"].as_str(),
        Some("https://json-schema.org/draft-07/schema#")
    );
}

#[test]
fn jsonschema_output_contains_envelope_metadata() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        v["schema_version"].as_u64(),
        Some(TOOL_SCHEMA_VERSION as u64)
    );
    assert_eq!(v["name"].as_str(), Some("rich"));
    assert_eq!(v["version"].as_str(), Some("2.0.0"));
    assert_eq!(v["description"].as_str(), Some("A rich CLI"));
}

#[test]
fn jsonschema_tool_parameters_include_default_value() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let scan = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "scan")
        .unwrap();

    let format_prop = &scan["parameters"]["properties"]["format"];
    assert_eq!(format_prop["default"].as_str(), Some("json"));
}

// ── Scenario: Clap raw format ───────────────────────────────────────────

#[test]
fn clap_format_is_serde_round_trippable() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();

    // Should deserialize back into ToolSchemaOutput
    let deserialized: tokmd_tool_schema::ToolSchemaOutput =
        serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.name, "rich");
    assert_eq!(deserialized.schema_version, TOOL_SCHEMA_VERSION);
    assert_eq!(deserialized.tools.len(), schema.tools.len());
}

// ── Scenario: Pretty vs compact rendering ───────────────────────────────

#[test]
fn pretty_output_contains_newlines() {
    let cmd = minimal_cmd();
    let schema = build_tool_schema(&cmd);
    let pretty = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    let compact = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();

    assert!(pretty.contains('\n'));
    assert!(!compact.contains('\n'));
}

#[test]
fn pretty_and_compact_parse_to_same_value() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);

    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let pretty: Value =
            serde_json::from_str(&render_output(&schema, fmt, true).unwrap()).unwrap();
        let compact: Value =
            serde_json::from_str(&render_output(&schema, fmt, false).unwrap()).unwrap();
        assert_eq!(pretty, compact, "Mismatch for format {:?}", fmt);
    }
}

// ── Scenario: SetFalse action ───────────────────────────────────────────

#[test]
fn set_false_action_is_boolean() {
    let cmd = Command::new("t").version("1.0.0").arg(
        Arg::new("no-color")
            .long("no-color")
            .action(ArgAction::SetFalse)
            .help("Disable color"),
    );

    let schema = build_tool_schema(&cmd);
    let root = &schema.tools[0];
    let param = root
        .parameters
        .iter()
        .find(|p| p.name == "no-color")
        .unwrap();
    assert_eq!(param.param_type, "boolean");
}

// ── Scenario: Empty subcommand with no args ─────────────────────────────

#[test]
fn subcommand_with_no_args_has_empty_parameters() {
    let cmd = Command::new("app")
        .version("1.0.0")
        .subcommand(Command::new("status").about("Show status"));

    let schema = build_tool_schema(&cmd);
    let status = schema.tools.iter().find(|t| t.name == "status").unwrap();
    assert!(status.parameters.is_empty());
}

// ── Scenario: ToolSchemaFormat default ──────────────────────────────────

#[test]
fn default_format_is_jsonschema() {
    let fmt: ToolSchemaFormat = Default::default();
    assert_eq!(fmt, ToolSchemaFormat::Jsonschema);
}
