//! Deep tests for tool-schema generation (Wave 43).

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{
    ParameterSchema, TOOL_SCHEMA_VERSION, ToolDefinition, ToolSchemaFormat, ToolSchemaOutput,
    build_tool_schema, render_output,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal command with no subcommands.
fn simple_cmd() -> Command {
    Command::new("simple")
        .version("0.1.0")
        .about("A simple command")
        .arg(Arg::new("path").help("Target path").required(true).index(1))
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
}

/// Rich command exercising many clap features.
fn rich_cmd() -> Command {
    Command::new("rich")
        .version("2.3.4")
        .about("A feature-rich command")
        .arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .value_parser(["json", "csv", "markdown"])
                .default_value("json")
                .help("Output format"),
        )
        .arg(
            Arg::new("count")
                .long("count")
                .short('c')
                .action(ArgAction::Count)
                .help("Verbosity level"),
        )
        .arg(
            Arg::new("include")
                .long("include")
                .short('i')
                .action(ArgAction::Append)
                .help("Paths to include"),
        )
        .arg(
            Arg::new("no-color")
                .long("no-color")
                .action(ArgAction::SetFalse)
                .help("Disable color"),
        )
        .subcommand(
            Command::new("analyze")
                .about("Run analysis")
                .arg(
                    Arg::new("preset")
                        .long("preset")
                        .required(true)
                        .value_parser(["health", "risk", "deep"])
                        .help("Analysis preset"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .help("Output file"),
                ),
        )
        .subcommand(
            Command::new("export").about("Export data").subcommand(
                Command::new("csv")
                    .about("Export as CSV")
                    .arg(Arg::new("delimiter").long("delimiter").default_value(",")),
            ),
        )
}

/// Command with no args, no version, no about.
fn bare_cmd() -> Command {
    Command::new("bare")
}

// ===========================================================================
// OpenAI format tests
// ===========================================================================

#[test]
fn openai_output_contains_functions_key() {
    let schema = build_tool_schema(&simple_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("functions").is_some(), "missing 'functions' key");
}

#[test]
fn openai_functions_array_length_matches_tools() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let fns = v["functions"].as_array().unwrap();
    assert_eq!(fns.len(), schema.tools.len());
}

#[test]
fn openai_required_params_in_required_array() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let analyze_fn = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "analyze")
        .expect("analyze function missing");
    let required = analyze_fn["parameters"]["required"].as_array().unwrap();
    assert!(
        required.iter().any(|r| r.as_str() == Some("preset")),
        "preset should be required"
    );
}

#[test]
fn openai_enum_values_present() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let analyze_fn = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "analyze")
        .unwrap();
    let preset_prop = &analyze_fn["parameters"]["properties"]["preset"];
    let enums = preset_prop["enum"].as_array().unwrap();
    assert_eq!(enums.len(), 3);
    assert!(enums.contains(&Value::String("health".into())));
    assert!(enums.contains(&Value::String("risk".into())));
    assert!(enums.contains(&Value::String("deep".into())));
}

#[test]
fn openai_pretty_output_is_valid_json() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    let _: Value = serde_json::from_str(&json).expect("pretty output should be valid JSON");
    assert!(json.contains('\n'), "pretty output should contain newlines");
}

// ===========================================================================
// Anthropic format tests
// ===========================================================================

#[test]
fn anthropic_output_contains_tools_key() {
    let schema = build_tool_schema(&simple_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("tools").is_some(), "missing 'tools' key");
}

#[test]
fn anthropic_uses_input_schema_not_parameters() {
    let schema = build_tool_schema(&simple_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    for tool in v["tools"].as_array().unwrap() {
        assert!(
            tool.get("input_schema").is_some(),
            "Anthropic tool should use input_schema"
        );
        assert!(
            tool.get("parameters").is_none(),
            "Anthropic tool should not have parameters"
        );
    }
}

#[test]
fn anthropic_required_params_in_required_array() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let analyze_tool = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "analyze")
        .unwrap();
    let required = analyze_tool["input_schema"]["required"].as_array().unwrap();
    assert!(required.iter().any(|r| r.as_str() == Some("preset")));
}

#[test]
fn anthropic_enum_values_present() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let root_tool = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "rich")
        .unwrap();
    let format_prop = &root_tool["input_schema"]["properties"]["format"];
    let enums = format_prop["enum"].as_array().unwrap();
    assert_eq!(enums.len(), 3);
}

// ===========================================================================
// JSON Schema format tests
// ===========================================================================

#[test]
fn jsonschema_contains_schema_draft_ref() {
    let schema = build_tool_schema(&simple_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["$schema"].as_str().unwrap(),
        "https://json-schema.org/draft-07/schema#"
    );
}

#[test]
fn jsonschema_contains_schema_version() {
    let schema = build_tool_schema(&simple_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        TOOL_SCHEMA_VERSION as u64
    );
}

#[test]
fn jsonschema_tools_array_present() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["tools"].is_array());
    assert!(!v["tools"].as_array().unwrap().is_empty());
}

#[test]
fn jsonschema_default_values_emitted() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let root_tool = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "rich")
        .unwrap();
    let format_prop = &root_tool["parameters"]["properties"]["format"];
    assert_eq!(format_prop["default"].as_str().unwrap(), "json");
}

// ===========================================================================
// Schema building tests
// ===========================================================================

#[test]
fn schema_includes_all_subcommands() {
    let schema = build_tool_schema(&rich_cmd());
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"rich"), "root command missing");
    assert!(names.contains(&"analyze"), "analyze subcommand missing");
    assert!(names.contains(&"export"), "export subcommand missing");
}

#[test]
fn help_subcommand_excluded() {
    let cmd = Command::new("test").subcommand(Command::new("help").about("Show help"));
    let schema = build_tool_schema(&cmd);
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(
        !names.contains(&"help"),
        "help subcommand should be filtered out"
    );
}

#[test]
fn schema_includes_all_parameters_with_types() {
    let schema = build_tool_schema(&rich_cmd());
    let root = schema.tools.iter().find(|t| t.name == "rich").unwrap();
    let param_names: Vec<&str> = root.parameters.iter().map(|p| p.name.as_str()).collect();
    assert!(param_names.contains(&"format"));
    assert!(param_names.contains(&"count"));
    assert!(param_names.contains(&"include"));
    assert!(param_names.contains(&"no-color"));
}

#[test]
fn boolean_param_type_for_set_true() {
    let schema = build_tool_schema(&simple_cmd());
    let root = schema.tools.iter().find(|t| t.name == "simple").unwrap();
    let verbose = root
        .parameters
        .iter()
        .find(|p| p.name == "verbose")
        .unwrap();
    assert_eq!(verbose.param_type, "boolean");
}

#[test]
fn boolean_param_type_for_set_false() {
    let schema = build_tool_schema(&rich_cmd());
    let root = schema.tools.iter().find(|t| t.name == "rich").unwrap();
    let no_color = root
        .parameters
        .iter()
        .find(|p| p.name == "no-color")
        .unwrap();
    assert_eq!(no_color.param_type, "boolean");
}

#[test]
fn integer_param_type_for_count() {
    let schema = build_tool_schema(&rich_cmd());
    let root = schema.tools.iter().find(|t| t.name == "rich").unwrap();
    let count = root.parameters.iter().find(|p| p.name == "count").unwrap();
    assert_eq!(count.param_type, "integer");
}

#[test]
fn array_param_type_for_append() {
    let schema = build_tool_schema(&rich_cmd());
    let root = schema.tools.iter().find(|t| t.name == "rich").unwrap();
    let include = root
        .parameters
        .iter()
        .find(|p| p.name == "include")
        .unwrap();
    assert_eq!(include.param_type, "array");
}

#[test]
fn string_param_type_for_regular_arg() {
    let schema = build_tool_schema(&simple_cmd());
    let root = schema.tools.iter().find(|t| t.name == "simple").unwrap();
    let path = root.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path.param_type, "string");
}

#[test]
fn required_vs_optional_params() {
    let schema = build_tool_schema(&simple_cmd());
    let root = schema.tools.iter().find(|t| t.name == "simple").unwrap();
    let path = root.parameters.iter().find(|p| p.name == "path").unwrap();
    let verbose = root
        .parameters
        .iter()
        .find(|p| p.name == "verbose")
        .unwrap();
    assert!(path.required, "path should be required");
    assert!(!verbose.required, "verbose should be optional");
}

#[test]
fn enum_values_populated_for_value_parser() {
    let schema = build_tool_schema(&rich_cmd());
    let root = schema.tools.iter().find(|t| t.name == "rich").unwrap();
    let format = root.parameters.iter().find(|p| p.name == "format").unwrap();
    let enums = format
        .enum_values
        .as_ref()
        .expect("enum_values should be Some");
    assert_eq!(enums, &["json", "csv", "markdown"]);
}

#[test]
fn enum_values_none_for_free_text_arg() {
    let schema = build_tool_schema(&simple_cmd());
    let root = schema.tools.iter().find(|t| t.name == "simple").unwrap();
    let path = root.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.enum_values.is_none());
}

// ===========================================================================
// Serde roundtrip tests
// ===========================================================================

#[test]
fn serde_roundtrip_tool_schema_output() {
    let schema = build_tool_schema(&rich_cmd());
    let json = serde_json::to_string(&schema).unwrap();
    let deserialized: ToolSchemaOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, schema.name);
    assert_eq!(deserialized.version, schema.version);
    assert_eq!(deserialized.tools.len(), schema.tools.len());
}

#[test]
fn serde_roundtrip_tool_definition() {
    let def = ToolDefinition {
        name: "test".into(),
        description: "A test tool".into(),
        parameters: vec![ParameterSchema {
            name: "arg1".into(),
            description: Some("First argument".into()),
            param_type: "string".into(),
            required: true,
            default: None,
            enum_values: Some(vec!["a".into(), "b".into()]),
        }],
    };
    let json = serde_json::to_string(&def).unwrap();
    let rt: ToolDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.name, "test");
    assert_eq!(rt.parameters.len(), 1);
    assert_eq!(rt.parameters[0].enum_values.as_ref().unwrap().len(), 2);
}

#[test]
fn serde_roundtrip_parameter_schema_optional_fields() {
    let param = ParameterSchema {
        name: "flag".into(),
        description: None,
        param_type: "boolean".into(),
        required: false,
        default: None,
        enum_values: None,
    };
    let json = serde_json::to_string(&param).unwrap();
    // skip_serializing_if: optional fields should be absent
    assert!(!json.contains("description"));
    assert!(!json.contains("default"));
    assert!(!json.contains("enum_values"));
    let rt: ParameterSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.name, "flag");
    assert!(rt.description.is_none());
}

#[test]
fn serde_roundtrip_format_enum() {
    let fmt = ToolSchemaFormat::Openai;
    let json = serde_json::to_string(&fmt).unwrap();
    let rt: ToolSchemaFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(rt, ToolSchemaFormat::Openai);
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn bare_command_produces_valid_schema() {
    let schema = build_tool_schema(&bare_cmd());
    assert_eq!(schema.name, "bare");
    assert_eq!(schema.version, "unknown");
    assert_eq!(schema.description, "");
    assert_eq!(schema.tools.len(), 1); // root only
    assert!(schema.tools[0].parameters.is_empty());
}

#[test]
fn clap_format_is_valid_json() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["name"].as_str().unwrap(), "rich");
}

#[test]
fn clap_format_pretty() {
    let schema = build_tool_schema(&rich_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Clap, true).unwrap();
    assert!(json.contains('\n'));
    let _: Value = serde_json::from_str(&json).unwrap();
}

#[test]
fn default_format_is_jsonschema() {
    assert_eq!(ToolSchemaFormat::default(), ToolSchemaFormat::Jsonschema);
}

#[test]
fn nested_subcommand_csv_appears_as_tool() {
    let schema = build_tool_schema(&rich_cmd());
    // export's subcommand csv is nested, so only top-level subcommands appear
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    // csv is a child of export, not a direct child of rich — so it should NOT appear
    assert!(
        !names.contains(&"csv"),
        "nested subcommands should not appear at top level"
    );
}

#[test]
fn help_and_version_args_excluded() {
    let schema = build_tool_schema(&simple_cmd());
    let root = schema.tools.iter().find(|t| t.name == "simple").unwrap();
    let param_names: Vec<&str> = root.parameters.iter().map(|p| p.name.as_str()).collect();
    assert!(!param_names.contains(&"help"));
    assert!(!param_names.contains(&"version"));
}

#[test]
fn tool_schema_version_constant() {
    assert_eq!(TOOL_SCHEMA_VERSION, 1);
}
