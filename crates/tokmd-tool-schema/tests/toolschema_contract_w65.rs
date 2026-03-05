//! Contract tests for tokmd-tool-schema (Wave 65).
//!
//! Covers: schema generation contracts, format-specific structural invariants,
//! serde round-tripping, parameter extraction fidelity, cross-format parity,
//! BDD-style given/when/then flows, edge cases, and determinism properties.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use std::collections::BTreeSet;
use tokmd_tool_schema::{
    ParameterSchema, ToolDefinition, ToolSchemaFormat, ToolSchemaOutput, TOOL_SCHEMA_VERSION,
    build_tool_schema, render_output,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A realistic CLI with mixed argument types.
fn realistic_cli() -> Command {
    Command::new("myapp")
        .version("2.5.0")
        .about("A realistic CLI tool")
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
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "markdown"])
                        .default_value("json")
                        .help("Output format"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .action(ArgAction::SetTrue)
                        .help("Enable verbose mode"),
                )
                .arg(
                    Arg::new("depth")
                        .long("depth")
                        .action(ArgAction::Count)
                        .help("Recursion depth level"),
                )
                .arg(
                    Arg::new("exclude")
                        .long("exclude")
                        .action(ArgAction::Append)
                        .help("Patterns to exclude"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export results")
                .arg(Arg::new("output").long("output").help("Output file path")),
        )
}

/// Empty command — no about, no version, no args.
fn empty_cmd() -> Command {
    Command::new("empty")
}

/// Command whose only subcommand has zero arguments.
fn argless_subcmd() -> Command {
    Command::new("wrapper")
        .version("0.1.0")
        .about("Wrapper")
        .subcommand(Command::new("noop").about("Does nothing at all"))
}

/// Command with SetFalse action.
fn setfalse_cmd() -> Command {
    Command::new("negator")
        .version("1.0.0")
        .about("Has set-false arg")
        .arg(
            Arg::new("no_color")
                .long("no-color")
                .action(ArgAction::SetFalse)
                .help("Disable color output"),
        )
}

/// Helper: parse rendered JSON.
fn parse(s: &str) -> Value {
    serde_json::from_str(s).expect("output must be valid JSON")
}

/// All four formats.
const ALL_FORMATS: [ToolSchemaFormat; 4] = [
    ToolSchemaFormat::Openai,
    ToolSchemaFormat::Anthropic,
    ToolSchemaFormat::Jsonschema,
    ToolSchemaFormat::Clap,
];

// ===========================================================================
// 1. Build schema — envelope contracts
// ===========================================================================

#[test]
fn contract_schema_version_matches_constant() {
    let schema = build_tool_schema(&realistic_cli());
    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
}

#[test]
fn contract_name_reflects_root_command() {
    let schema = build_tool_schema(&realistic_cli());
    assert_eq!(schema.name, "myapp");
}

#[test]
fn contract_version_propagated_from_command() {
    let schema = build_tool_schema(&realistic_cli());
    assert_eq!(schema.version, "2.5.0");
}

#[test]
fn contract_description_from_about() {
    let schema = build_tool_schema(&realistic_cli());
    assert_eq!(schema.description, "A realistic CLI tool");
}

#[test]
fn contract_tools_include_root_and_subcommands() {
    let schema = build_tool_schema(&realistic_cli());
    let names: BTreeSet<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains("myapp"), "root command missing");
    assert!(names.contains("scan"), "scan subcommand missing");
    assert!(names.contains("export"), "export subcommand missing");
}

#[test]
fn contract_help_subcommand_excluded() {
    let schema = build_tool_schema(&realistic_cli());
    assert!(
        !schema.tools.iter().any(|t| t.name == "help"),
        "help subcommand must be filtered out"
    );
}

#[test]
fn contract_help_and_version_args_excluded() {
    let schema = build_tool_schema(&realistic_cli());
    for tool in &schema.tools {
        assert!(
            !tool.parameters.iter().any(|p| p.name == "help"),
            "help arg leaked in tool '{}'",
            tool.name
        );
        assert!(
            !tool.parameters.iter().any(|p| p.name == "version"),
            "version arg leaked in tool '{}'",
            tool.name
        );
    }
}

// ===========================================================================
// 2. Parameter extraction fidelity
// ===========================================================================

#[test]
fn param_string_type_extracted() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path.param_type, "string");
}

#[test]
fn param_boolean_from_set_true() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let verbose = scan
        .parameters
        .iter()
        .find(|p| p.name == "verbose")
        .unwrap();
    assert_eq!(verbose.param_type, "boolean");
}

#[test]
fn param_boolean_from_set_false() {
    let schema = build_tool_schema(&setfalse_cmd());
    let root = &schema.tools[0];
    let no_color = root
        .parameters
        .iter()
        .find(|p| p.name == "no_color")
        .unwrap();
    assert_eq!(no_color.param_type, "boolean");
}

#[test]
fn param_integer_from_count() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let depth = scan
        .parameters
        .iter()
        .find(|p| p.name == "depth")
        .unwrap();
    assert_eq!(depth.param_type, "integer");
}

#[test]
fn param_array_from_append() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let exclude = scan
        .parameters
        .iter()
        .find(|p| p.name == "exclude")
        .unwrap();
    assert_eq!(exclude.param_type, "array");
}

#[test]
fn param_required_flag_correct() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    assert!(
        scan.parameters
            .iter()
            .find(|p| p.name == "path")
            .unwrap()
            .required
    );
    assert!(
        !scan
            .parameters
            .iter()
            .find(|p| p.name == "verbose")
            .unwrap()
            .required
    );
}

#[test]
fn param_enum_values_captured() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let format_param = scan
        .parameters
        .iter()
        .find(|p| p.name == "format")
        .unwrap();
    let enums = format_param.enum_values.as_ref().unwrap();
    assert_eq!(enums, &["json", "csv", "markdown"]);
}

#[test]
fn param_default_value_captured() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|p| p.name == "scan").unwrap();
    let format_param = scan
        .parameters
        .iter()
        .find(|p| p.name == "format")
        .unwrap();
    assert_eq!(format_param.default.as_deref(), Some("json"));
}

#[test]
fn param_description_from_help() {
    let schema = build_tool_schema(&realistic_cli());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path.description.as_deref(), Some("Directory to scan"));
}

#[test]
fn param_without_help_has_none_description() {
    let cmd = Command::new("x")
        .version("1.0.0")
        .arg(Arg::new("bare_arg").long("bare-arg"));
    let schema = build_tool_schema(&cmd);
    let p = schema.tools[0]
        .parameters
        .iter()
        .find(|p| p.name == "bare_arg")
        .unwrap();
    assert!(p.description.is_none());
}

// ===========================================================================
// 3. Format-specific structural contracts
// ===========================================================================

#[test]
fn openai_root_key_is_functions() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    assert!(v.get("functions").is_some());
    assert!(v["functions"].is_array());
}

#[test]
fn openai_each_function_has_parameters_type_object() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    for func in v["functions"].as_array().unwrap() {
        assert_eq!(func["parameters"]["type"], "object");
    }
}

#[test]
fn openai_required_array_only_contains_required_params() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    let scan_fn = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "scan")
        .unwrap();
    let required: Vec<&str> = scan_fn["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(required.contains(&"path"));
    assert!(!required.contains(&"verbose"));
    assert!(!required.contains(&"format"));
}

#[test]
fn anthropic_root_key_is_tools() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());
    assert!(v.get("tools").is_some());
    assert!(v["tools"].is_array());
}

#[test]
fn anthropic_uses_input_schema_not_parameters() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());
    for tool in v["tools"].as_array().unwrap() {
        assert!(tool.get("input_schema").is_some());
        assert!(
            tool.get("parameters").is_none(),
            "Anthropic format should use 'input_schema' not 'parameters'"
        );
    }
}

#[test]
fn anthropic_input_schema_type_is_object() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());
    for tool in v["tools"].as_array().unwrap() {
        assert_eq!(tool["input_schema"]["type"], "object");
    }
}

#[test]
fn jsonschema_has_dollar_schema_field() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());
    assert_eq!(v["$schema"], "https://json-schema.org/draft-07/schema#");
}

#[test]
fn jsonschema_has_schema_version() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(TOOL_SCHEMA_VERSION)
    );
}

#[test]
fn jsonschema_includes_default_in_properties() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());
    let scan = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "scan")
        .unwrap();
    assert_eq!(
        scan["parameters"]["properties"]["format"]["default"],
        "json"
    );
}

#[test]
fn clap_format_round_trips_via_serde() {
    let schema = build_tool_schema(&realistic_cli());
    let json = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let deserialized: ToolSchemaOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, schema.name);
    assert_eq!(deserialized.version, schema.version);
    assert_eq!(deserialized.tools.len(), schema.tools.len());
}

#[test]
fn clap_format_preserves_parameter_details() {
    let schema = build_tool_schema(&realistic_cli());
    let json = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let rt: ToolSchemaOutput = serde_json::from_str(&json).unwrap();
    let scan = rt.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.required);
    assert_eq!(path.param_type, "string");
}

// ===========================================================================
// 4. Cross-format parity
// ===========================================================================

#[test]
fn cross_format_tool_count_identical() {
    let schema = build_tool_schema(&realistic_cli());
    let expected = schema.tools.len();

    let openai = parse(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    let anthropic = parse(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());
    let jsonschema = parse(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());

    assert_eq!(openai["functions"].as_array().unwrap().len(), expected);
    assert_eq!(anthropic["tools"].as_array().unwrap().len(), expected);
    assert_eq!(jsonschema["tools"].as_array().unwrap().len(), expected);
}

#[test]
fn cross_format_tool_names_identical() {
    let schema = build_tool_schema(&realistic_cli());

    let openai = parse(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    let anthropic = parse(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());

    let openai_names: BTreeSet<String> = openai["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap().to_string())
        .collect();
    let anthropic_names: BTreeSet<String> = anthropic["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(openai_names, anthropic_names);
}

#[test]
fn cross_format_property_count_consistent() {
    let schema = build_tool_schema(&realistic_cli());

    let openai = parse(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    let anthropic = parse(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());

    for tool in &schema.tools {
        let expected_param_count = tool.parameters.len();

        let oai_fn = openai["functions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"].as_str() == Some(&tool.name))
            .unwrap();
        let oai_count = oai_fn["parameters"]["properties"]
            .as_object()
            .unwrap()
            .len();

        let ant_tool = anthropic["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"].as_str() == Some(&tool.name))
            .unwrap();
        let ant_count = ant_tool["input_schema"]["properties"]
            .as_object()
            .unwrap()
            .len();

        assert_eq!(oai_count, expected_param_count, "OpenAI mismatch for {}", tool.name);
        assert_eq!(ant_count, expected_param_count, "Anthropic mismatch for {}", tool.name);
    }
}

// ===========================================================================
// 5. Determinism
// ===========================================================================

#[test]
fn determinism_all_formats_compact() {
    for fmt in ALL_FORMATS {
        let a = render_output(&build_tool_schema(&realistic_cli()), fmt, false).unwrap();
        let b = render_output(&build_tool_schema(&realistic_cli()), fmt, false).unwrap();
        assert_eq!(a, b, "compact output non-deterministic for {fmt:?}");
    }
}

#[test]
fn determinism_all_formats_pretty() {
    for fmt in ALL_FORMATS {
        let a = render_output(&build_tool_schema(&realistic_cli()), fmt, true).unwrap();
        let b = render_output(&build_tool_schema(&realistic_cli()), fmt, true).unwrap();
        assert_eq!(a, b, "pretty output non-deterministic for {fmt:?}");
    }
}

#[test]
fn pretty_and_compact_parse_to_same_value() {
    for fmt in ALL_FORMATS {
        let schema = build_tool_schema(&realistic_cli());
        let compact: Value = parse(&render_output(&schema, fmt, false).unwrap());
        let pretty: Value = parse(&render_output(&schema, fmt, true).unwrap());
        assert_eq!(compact, pretty, "pretty vs compact differ for {fmt:?}");
    }
}

#[test]
fn pretty_output_longer_than_compact() {
    for fmt in ALL_FORMATS {
        let schema = build_tool_schema(&realistic_cli());
        let compact = render_output(&schema, fmt, false).unwrap();
        let pretty = render_output(&schema, fmt, true).unwrap();
        assert!(
            pretty.len() > compact.len(),
            "pretty should be longer for {fmt:?}"
        );
    }
}

// ===========================================================================
// 6. Valid JSON — all formats × all commands
// ===========================================================================

#[test]
fn all_formats_produce_valid_json_for_all_commands() {
    let commands = [
        realistic_cli(),
        empty_cmd(),
        argless_subcmd(),
        setfalse_cmd(),
    ];
    for cmd in commands {
        let schema = build_tool_schema(&cmd);
        for fmt in ALL_FORMATS {
            for pretty in [true, false] {
                let output = render_output(&schema, fmt, pretty).unwrap();
                let result: Result<Value, _> = serde_json::from_str(&output);
                assert!(
                    result.is_ok(),
                    "invalid JSON for cmd={}, fmt={fmt:?}, pretty={pretty}: {}",
                    cmd.get_name(),
                    result.unwrap_err()
                );
            }
        }
    }
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn edge_empty_command_produces_single_root_tool() {
    let schema = build_tool_schema(&empty_cmd());
    assert_eq!(schema.tools.len(), 1);
    assert_eq!(schema.tools[0].name, "empty");
    assert!(schema.tools[0].parameters.is_empty());
}

#[test]
fn edge_empty_command_version_is_unknown() {
    let schema = build_tool_schema(&empty_cmd());
    assert_eq!(schema.version, "unknown");
}

#[test]
fn edge_empty_command_description_is_empty() {
    let schema = build_tool_schema(&empty_cmd());
    assert!(schema.description.is_empty());
}

#[test]
fn edge_argless_subcommand_has_empty_parameters() {
    let schema = build_tool_schema(&argless_subcmd());
    let noop = schema.tools.iter().find(|t| t.name == "noop").unwrap();
    assert!(noop.parameters.is_empty());
}

#[test]
fn edge_argless_subcommand_renders_empty_properties() {
    let schema = build_tool_schema(&argless_subcmd());
    for fmt in [ToolSchemaFormat::Openai, ToolSchemaFormat::Anthropic] {
        let v = parse(&render_output(&schema, fmt, false).unwrap());
        let tools_key = if fmt == ToolSchemaFormat::Openai {
            "functions"
        } else {
            "tools"
        };
        let params_key = if fmt == ToolSchemaFormat::Openai {
            "parameters"
        } else {
            "input_schema"
        };
        let noop_tool = v[tools_key]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == "noop")
            .unwrap();
        assert!(noop_tool[params_key]["properties"].as_object().unwrap().is_empty());
        assert!(noop_tool[params_key]["required"].as_array().unwrap().is_empty());
    }
}

#[test]
fn edge_nested_subcommands_only_top_level_extracted() {
    let cmd = Command::new("top")
        .version("1.0.0")
        .subcommand(
            Command::new("l1")
                .about("Level 1")
                .subcommand(Command::new("l2").about("Level 2")),
        );
    let schema = build_tool_schema(&cmd);
    let names: BTreeSet<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains("top"));
    assert!(names.contains("l1"));
    assert!(!names.contains("l2"));
}

// ===========================================================================
// 8. Serde round-trip contracts
// ===========================================================================

#[test]
fn serde_tool_definition_round_trip() {
    let td = ToolDefinition {
        name: "test".to_string(),
        description: "A test tool".to_string(),
        parameters: vec![ParameterSchema {
            name: "arg1".to_string(),
            description: Some("First arg".to_string()),
            param_type: "string".to_string(),
            required: true,
            default: None,
            enum_values: Some(vec!["a".to_string(), "b".to_string()]),
        }],
    };
    let json = serde_json::to_string(&td).unwrap();
    let rt: ToolDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.name, td.name);
    assert_eq!(rt.parameters.len(), 1);
    assert_eq!(rt.parameters[0].enum_values.as_ref().unwrap().len(), 2);
}

#[test]
fn serde_parameter_schema_skip_none_fields() {
    let ps = ParameterSchema {
        name: "x".to_string(),
        description: None,
        param_type: "boolean".to_string(),
        required: false,
        default: None,
        enum_values: None,
    };
    let json = serde_json::to_string(&ps).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("description").is_none(), "None fields should be skipped");
    assert!(v.get("default").is_none());
    assert!(v.get("enum_values").is_none());
}

#[test]
fn serde_tool_schema_output_round_trip() {
    let schema = build_tool_schema(&realistic_cli());
    let json = serde_json::to_string(&schema).unwrap();
    let rt: ToolSchemaOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.schema_version, schema.schema_version);
    assert_eq!(rt.name, schema.name);
    assert_eq!(rt.tools.len(), schema.tools.len());
}

// ===========================================================================
// 9. BDD-style: Given / When / Then
// ===========================================================================

/// Given a command with mixed argument types
/// When generating OpenAI schema
/// Then each parameter type maps correctly to JSON schema types.
#[test]
fn bdd_given_mixed_args_when_openai_then_types_correct() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    let scan = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "scan")
        .unwrap();
    let props = &scan["parameters"]["properties"];

    assert_eq!(props["path"]["type"], "string");
    assert_eq!(props["verbose"]["type"], "boolean");
    assert_eq!(props["depth"]["type"], "integer");
    assert_eq!(props["exclude"]["type"], "array");
    assert!(props["format"]["enum"].is_array());
}

/// Given an empty command
/// When generating Anthropic schema
/// Then output is valid with a single tool having no properties.
#[test]
fn bdd_given_empty_cmd_when_anthropic_then_valid_single_tool() {
    let schema = build_tool_schema(&empty_cmd());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());
    let tools = v["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "empty");
    assert!(tools[0]["input_schema"]["properties"]
        .as_object()
        .unwrap()
        .is_empty());
}

/// Given a command with required and optional params
/// When generating JSON Schema format
/// Then the required array only lists required params.
#[test]
fn bdd_given_required_optional_when_jsonschema_then_required_accurate() {
    let schema = build_tool_schema(&realistic_cli());
    let v = parse(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());
    let scan = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "scan")
        .unwrap();
    let required: BTreeSet<String> = scan["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert!(required.contains("path"));
    assert!(!required.contains("verbose"));
    assert!(!required.contains("format"));
    assert!(!required.contains("depth"));
    assert!(!required.contains("exclude"));
}

/// Given a command with enum values and defaults
/// When rendering in Clap format and deserializing
/// Then enum values and defaults survive the round-trip.
#[test]
fn bdd_given_enum_defaults_when_clap_roundtrip_then_preserved() {
    let schema = build_tool_schema(&realistic_cli());
    let json = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let rt: ToolSchemaOutput = serde_json::from_str(&json).unwrap();
    let scan = rt.tools.iter().find(|t| t.name == "scan").unwrap();
    let format_param = scan
        .parameters
        .iter()
        .find(|p| p.name == "format")
        .unwrap();
    assert_eq!(format_param.default.as_deref(), Some("json"));
    assert_eq!(
        format_param.enum_values.as_ref().unwrap(),
        &["json", "csv", "markdown"]
    );
}
