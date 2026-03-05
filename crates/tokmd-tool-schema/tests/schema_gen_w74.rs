//! W74 – Tool-schema generation tests.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{
    build_tool_schema, render_output, ToolDefinition, ToolSchemaFormat, ToolSchemaOutput,
    TOOL_SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_cmd() -> Command {
    Command::new("demo")
        .version("0.1.0")
        .about("Demo tool")
        .subcommand(
            Command::new("scan")
                .about("Scan a repo")
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
                        .help("Verbose output"),
                )
                .arg(
                    Arg::new("depth")
                        .long("depth")
                        .action(ArgAction::Count)
                        .help("Depth level"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export data")
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "tsv"])
                        .default_value("json")
                        .help("Output format"),
                ),
        )
}

fn multi_subcmd() -> Command {
    Command::new("multi")
        .version("2.0.0")
        .about("Multi-sub")
        .subcommand(Command::new("alpha").about("Alpha"))
        .subcommand(Command::new("beta").about("Beta"))
        .subcommand(Command::new("gamma").about("Gamma"))
}

fn no_subcmd() -> Command {
    Command::new("bare")
        .version("0.0.1")
        .about("Bare tool")
        .arg(Arg::new("input").required(true).help("Input file"))
}

fn cmd_with_append_arg() -> Command {
    Command::new("appender")
        .version("0.2.0")
        .about("Append demo")
        .arg(
            Arg::new("tags")
                .long("tag")
                .action(ArgAction::Append)
                .help("Tags to attach"),
        )
}

fn parse_json(s: &str) -> Value {
    serde_json::from_str(s).expect("valid JSON")
}

fn find_tool<'a>(schema: &'a ToolSchemaOutput, name: &str) -> &'a ToolDefinition {
    schema
        .tools
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("tool '{name}' not found"))
}

// ---------------------------------------------------------------------------
// 1. Schema envelope metadata
// ---------------------------------------------------------------------------

#[test]
fn w74_schema_version_matches_constant() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
}

#[test]
fn w74_schema_captures_name_version_description() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.name, "demo");
    assert_eq!(schema.version, "0.1.0");
    assert_eq!(schema.description, "Demo tool");
}

// ---------------------------------------------------------------------------
// 2. Subcommand enumeration
// ---------------------------------------------------------------------------

#[test]
fn w74_subcommands_listed_as_tools() {
    let schema = build_tool_schema(&simple_cmd());
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"scan"));
    assert!(names.contains(&"export"));
}

#[test]
fn w74_help_subcommand_excluded() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(!schema.tools.iter().any(|t| t.name == "help"));
}

#[test]
fn w74_root_command_included_as_tool() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(schema.tools.iter().any(|t| t.name == "demo"));
}

#[test]
fn w74_multiple_subcommands_counted() {
    let schema = build_tool_schema(&multi_subcmd());
    // root + 3 subcommands
    assert_eq!(schema.tools.len(), 4);
}

// ---------------------------------------------------------------------------
// 3. Parameter types
// ---------------------------------------------------------------------------

#[test]
fn w74_string_param_type() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = find_tool(&schema, "scan");
    let path_param = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path_param.param_type, "string");
}

#[test]
fn w74_boolean_param_type() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = find_tool(&schema, "scan");
    let verbose = scan.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert_eq!(verbose.param_type, "boolean");
}

#[test]
fn w74_count_param_becomes_integer() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = find_tool(&schema, "scan");
    let depth = scan.parameters.iter().find(|p| p.name == "depth").unwrap();
    assert_eq!(depth.param_type, "integer");
}

#[test]
fn w74_append_param_becomes_array() {
    let schema = build_tool_schema(&cmd_with_append_arg());
    let root = find_tool(&schema, "appender");
    let tags = root.parameters.iter().find(|p| p.name == "tags").unwrap();
    assert_eq!(tags.param_type, "array");
}

// ---------------------------------------------------------------------------
// 4. Required vs optional + defaults + enums
// ---------------------------------------------------------------------------

#[test]
fn w74_required_flag_propagated() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = find_tool(&schema, "scan");
    let path_param = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path_param.required);
    let verbose = scan.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert!(!verbose.required);
}

#[test]
fn w74_default_value_captured() {
    let schema = build_tool_schema(&simple_cmd());
    let export = find_tool(&schema, "export");
    let fmt = export.parameters.iter().find(|p| p.name == "format").unwrap();
    assert_eq!(fmt.default.as_deref(), Some("json"));
}

#[test]
fn w74_enum_values_captured() {
    let schema = build_tool_schema(&simple_cmd());
    let export = find_tool(&schema, "export");
    let fmt = export.parameters.iter().find(|p| p.name == "format").unwrap();
    let enums = fmt.enum_values.as_ref().expect("enum_values present");
    assert!(enums.contains(&"json".to_string()));
    assert!(enums.contains(&"csv".to_string()));
    assert!(enums.contains(&"tsv".to_string()));
}

#[test]
fn w74_help_and_version_args_filtered() {
    let schema = build_tool_schema(&simple_cmd());
    for tool in &schema.tools {
        assert!(!tool.parameters.iter().any(|p| p.name == "help"));
        assert!(!tool.parameters.iter().any(|p| p.name == "version"));
    }
}

// ---------------------------------------------------------------------------
// 5. OpenAI format rendering
// ---------------------------------------------------------------------------

#[test]
fn w74_openai_output_is_valid_json() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    parse_json(&out); // must not panic
}

#[test]
fn w74_openai_has_functions_array() {
    let schema = build_tool_schema(&simple_cmd());
    let v = parse_json(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    assert!(v["functions"].is_array());
    assert!(!v["functions"].as_array().unwrap().is_empty());
}

#[test]
fn w74_openai_function_has_parameters_object() {
    let schema = build_tool_schema(&simple_cmd());
    let v = parse_json(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    for func in v["functions"].as_array().unwrap() {
        assert_eq!(func["parameters"]["type"], "object");
        assert!(func["parameters"]["properties"].is_object());
        assert!(func["parameters"]["required"].is_array());
    }
}

// ---------------------------------------------------------------------------
// 6. Anthropic format rendering
// ---------------------------------------------------------------------------

#[test]
fn w74_anthropic_has_tools_with_input_schema() {
    let schema = build_tool_schema(&simple_cmd());
    let v = parse_json(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());
    assert!(v["tools"].is_array());
    for tool in v["tools"].as_array().unwrap() {
        assert!(tool.get("input_schema").is_some());
        assert_eq!(tool["input_schema"]["type"], "object");
    }
}

// ---------------------------------------------------------------------------
// 7. JSON Schema format rendering
// ---------------------------------------------------------------------------

#[test]
fn w74_jsonschema_has_dollar_schema_field() {
    let schema = build_tool_schema(&simple_cmd());
    let v = parse_json(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());
    assert_eq!(v["$schema"], "https://json-schema.org/draft-07/schema#");
}

#[test]
fn w74_jsonschema_includes_schema_version() {
    let schema = build_tool_schema(&simple_cmd());
    let v = parse_json(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());
    assert_eq!(v["schema_version"], TOOL_SCHEMA_VERSION);
}

// ---------------------------------------------------------------------------
// 8. Pretty printing
// ---------------------------------------------------------------------------

#[test]
fn w74_pretty_output_contains_newlines() {
    let schema = build_tool_schema(&simple_cmd());
    let compact = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let pretty = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    assert!(pretty.contains('\n'));
    assert!(pretty.len() > compact.len());
}

// ---------------------------------------------------------------------------
// 9. Bare command (no subcommands)
// ---------------------------------------------------------------------------

#[test]
fn w74_bare_command_produces_single_tool() {
    let schema = build_tool_schema(&no_subcmd());
    assert_eq!(schema.tools.len(), 1);
    assert_eq!(schema.tools[0].name, "bare");
    assert_eq!(schema.tools[0].parameters.len(), 1);
    assert!(schema.tools[0].parameters[0].required);
}

// ---------------------------------------------------------------------------
// 10. Clap (raw) format
// ---------------------------------------------------------------------------

#[test]
fn w74_clap_format_round_trips() {
    let schema = build_tool_schema(&simple_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let decoded: ToolSchemaOutput = serde_json::from_str(&json_str).unwrap();
    assert_eq!(decoded.name, schema.name);
    assert_eq!(decoded.tools.len(), schema.tools.len());
}
