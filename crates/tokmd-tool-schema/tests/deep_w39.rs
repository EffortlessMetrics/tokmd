//! Wave-39 deep tests for tokmd-tool-schema: schema generation, format
//! rendering, structure validation, and command tree traversal.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::*;

fn make_multi_cmd() -> Command {
    Command::new("myapp")
        .version("2.0.0")
        .about("My test app")
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(ArgAction::Count)
                .help("Verbosity level"),
        )
        .subcommand(
            Command::new("scan")
                .about("Scan files")
                .arg(
                    Arg::new("path")
                        .required(true)
                        .help("Path to scan"),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "md"])
                        .help("Output format"),
                )
                .arg(
                    Arg::new("recursive")
                        .long("recursive")
                        .action(ArgAction::SetTrue)
                        .help("Scan recursively"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export data")
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .action(ArgAction::Append)
                        .help("Tags to apply"),
                ),
        )
}

// ── build_tool_schema ────────────────────────────────────────────────────

#[test]
fn schema_captures_name_and_version() {
    let cmd = make_multi_cmd();
    let schema = build_tool_schema(&cmd);
    assert_eq!(schema.name, "myapp");
    assert_eq!(schema.version, "2.0.0");
    assert_eq!(schema.description, "My test app");
    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
}

#[test]
fn schema_includes_root_and_subcommands() {
    let cmd = make_multi_cmd();
    let schema = build_tool_schema(&cmd);
    // root + scan + export = 3 tools
    assert_eq!(schema.tools.len(), 3);
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"myapp"));
    assert!(names.contains(&"scan"));
    assert!(names.contains(&"export"));
}

#[test]
fn help_subcommand_is_excluded() {
    let cmd = Command::new("app")
        .subcommand(Command::new("help").about("Show help"))
        .subcommand(Command::new("real").about("Real command"));
    let schema = build_tool_schema(&cmd);
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(!names.contains(&"help"));
    assert!(names.contains(&"real"));
}

#[test]
fn boolean_param_detected_from_set_true() {
    let cmd = make_multi_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let recursive = scan.parameters.iter().find(|p| p.name == "recursive").unwrap();
    assert_eq!(recursive.param_type, "boolean");
    assert!(!recursive.required);
}

#[test]
fn count_param_detected_as_integer() {
    let cmd = make_multi_cmd();
    let schema = build_tool_schema(&cmd);
    let root = schema.tools.iter().find(|t| t.name == "myapp").unwrap();
    let verbose = root.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert_eq!(verbose.param_type, "integer");
}

#[test]
fn append_param_detected_as_array() {
    let cmd = make_multi_cmd();
    let schema = build_tool_schema(&cmd);
    let export = schema.tools.iter().find(|t| t.name == "export").unwrap();
    let tags = export.parameters.iter().find(|p| p.name == "tags").unwrap();
    assert_eq!(tags.param_type, "array");
}

#[test]
fn required_param_marked_correctly() {
    let cmd = make_multi_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.required);
}

#[test]
fn enum_values_captured() {
    let cmd = make_multi_cmd();
    let schema = build_tool_schema(&cmd);
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let format = scan.parameters.iter().find(|p| p.name == "format").unwrap();
    let enums = format.enum_values.as_ref().unwrap();
    assert_eq!(enums, &["json", "csv", "md"]);
}

// ── OpenAI format ────────────────────────────────────────────────────────

#[test]
fn openai_output_has_functions_array() {
    let schema = build_tool_schema(&make_multi_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let fns = parsed["functions"].as_array().unwrap();
    assert!(fns.len() >= 2);
    // Each function should have name + parameters
    for f in fns {
        assert!(f["name"].is_string());
        assert!(f["parameters"]["type"].as_str() == Some("object"));
    }
}

#[test]
fn openai_required_params_in_array() {
    let schema = build_tool_schema(&make_multi_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let scan_fn = parsed["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "scan")
        .unwrap();
    let required = scan_fn["parameters"]["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v.as_str() == Some("path")));
}

// ── Anthropic format ─────────────────────────────────────────────────────

#[test]
fn anthropic_uses_input_schema() {
    let schema = build_tool_schema(&make_multi_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let tools = parsed["tools"].as_array().unwrap();
    for tool in tools {
        assert!(
            tool.get("input_schema").is_some(),
            "Anthropic tools must use input_schema"
        );
        assert!(tool["input_schema"]["type"].as_str() == Some("object"));
    }
}

// ── JSON Schema format ──────────────────────────────────────────────────

#[test]
fn jsonschema_has_draft7_ref() {
    let schema = build_tool_schema(&make_multi_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed["$schema"].as_str().unwrap(),
        "https://json-schema.org/draft-07/schema#"
    );
    assert_eq!(parsed["schema_version"].as_u64().unwrap(), 1);
}

#[test]
fn jsonschema_enum_values_in_properties() {
    let schema = build_tool_schema(&make_multi_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let scan_tool = parsed["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "scan")
        .unwrap();
    let format_prop = &scan_tool["parameters"]["properties"]["format"];
    let enums = format_prop["enum"].as_array().unwrap();
    assert_eq!(enums.len(), 3);
}

// ── Clap (raw) format ───────────────────────────────────────────────────

#[test]
fn clap_format_serialises_full_struct() {
    let schema = build_tool_schema(&make_multi_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["tools"].is_array());
    assert!(parsed["schema_version"].is_number());
    assert_eq!(parsed["name"].as_str().unwrap(), "myapp");
}

// ── Pretty vs compact ──────────────────────────────────────────────────

#[test]
fn pretty_output_is_indented() {
    let schema = build_tool_schema(&make_multi_cmd());
    let pretty = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    let compact = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    assert!(pretty.contains('\n'));
    assert!(pretty.len() > compact.len());
}

// ── Command with no args ────────────────────────────────────────────────

#[test]
fn command_with_no_args_produces_empty_parameters() {
    let cmd = Command::new("empty").about("Nothing here");
    let schema = build_tool_schema(&cmd);
    let root = &schema.tools[0];
    assert!(root.parameters.is_empty());
}

// ── Command with default value ──────────────────────────────────────────

#[test]
fn default_value_captured() {
    let cmd = Command::new("app").arg(
        Arg::new("level")
            .long("level")
            .default_value("info")
            .help("Log level"),
    );
    let schema = build_tool_schema(&cmd);
    let param = &schema.tools[0].parameters[0];
    assert_eq!(param.default.as_deref(), Some("info"));
}

// ── ToolSchemaFormat enum ───────────────────────────────────────────────

#[test]
fn tool_schema_format_default_is_jsonschema() {
    assert_eq!(ToolSchemaFormat::default(), ToolSchemaFormat::Jsonschema);
}
