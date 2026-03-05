//! Depth tests for tokmd-tool-schema — W56 tooling coverage.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{
    TOOL_SCHEMA_VERSION, ToolSchemaFormat, ToolSchemaOutput, build_tool_schema, render_output,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_cmd() -> Command {
    Command::new("demo")
        .version("0.1.0")
        .about("Demo tool")
        .subcommand(
            Command::new("run")
                .about("Run something")
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .action(ArgAction::SetTrue)
                        .help("Enable verbose output"),
                )
                .arg(
                    Arg::new("count")
                        .long("count")
                        .action(ArgAction::Count)
                        .help("Verbosity level"),
                )
                .arg(
                    Arg::new("input")
                        .long("input")
                        .required(true)
                        .help("Input path"),
                ),
        )
}

fn cmd_with_enum() -> Command {
    Command::new("enumtool")
        .version("2.0.0")
        .about("Tool with enum arg")
        .subcommand(
            Command::new("format").about("Choose format").arg(
                Arg::new("fmt")
                    .long("fmt")
                    .value_parser(["json", "csv", "yaml"])
                    .default_value("json")
                    .help("Output format"),
            ),
        )
}

fn cmd_no_subcommands() -> Command {
    Command::new("bare")
        .version("0.0.1")
        .about("No subcommands")
}

fn cmd_no_params_subcommand() -> Command {
    Command::new("empty")
        .version("1.0.0")
        .about("Has a sub with no args")
        .subcommand(Command::new("noop").about("Does nothing"))
}

fn cmd_deeply_nested() -> Command {
    Command::new("deep")
        .version("3.0.0")
        .about("Nested commands")
        .subcommand(
            Command::new("level1").about("L1").subcommand(
                Command::new("level2")
                    .about("L2")
                    .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue)),
            ),
        )
}

fn cmd_with_append() -> Command {
    Command::new("multi")
        .version("1.0.0")
        .about("Multiple values")
        .subcommand(
            Command::new("collect").about("Collect items").arg(
                Arg::new("item")
                    .long("item")
                    .action(ArgAction::Append)
                    .help("Items to collect"),
            ),
        )
}

fn parse_json(s: &str) -> Value {
    serde_json::from_str(s).expect("valid JSON")
}

// ---------------------------------------------------------------------------
// build_tool_schema — structure
// ---------------------------------------------------------------------------

#[test]
fn schema_includes_root_command() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(schema.tools.iter().any(|t| t.name == "demo"));
}

#[test]
fn schema_version_matches_constant() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
}

#[test]
fn schema_name_matches_command() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.name, "demo");
}

#[test]
fn schema_version_string_from_command() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.version, "0.1.0");
}

#[test]
fn schema_description_from_about() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.description, "Demo tool");
}

#[test]
fn schema_excludes_help_subcommand() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(!schema.tools.iter().any(|t| t.name == "help"));
}

#[test]
fn schema_skips_help_and_version_args() {
    let schema = build_tool_schema(&simple_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    assert!(!run.parameters.iter().any(|p| p.name == "help"));
    assert!(!run.parameters.iter().any(|p| p.name == "version"));
}

// ---------------------------------------------------------------------------
// Parameter type mapping
// ---------------------------------------------------------------------------

#[test]
fn set_true_maps_to_boolean() {
    let schema = build_tool_schema(&simple_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let verbose = run.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert_eq!(verbose.param_type, "boolean");
}

#[test]
fn count_maps_to_integer() {
    let schema = build_tool_schema(&simple_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let count = run.parameters.iter().find(|p| p.name == "count").unwrap();
    assert_eq!(count.param_type, "integer");
}

#[test]
fn append_maps_to_array() {
    let schema = build_tool_schema(&cmd_with_append());
    let collect = schema.tools.iter().find(|t| t.name == "collect").unwrap();
    let item = collect
        .parameters
        .iter()
        .find(|p| p.name == "item")
        .unwrap();
    assert_eq!(item.param_type, "array");
}

#[test]
fn string_arg_maps_to_string() {
    let schema = build_tool_schema(&simple_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let input = run.parameters.iter().find(|p| p.name == "input").unwrap();
    assert_eq!(input.param_type, "string");
}

#[test]
fn required_flag_set_correctly() {
    let schema = build_tool_schema(&simple_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let input = run.parameters.iter().find(|p| p.name == "input").unwrap();
    assert!(input.required);
    let verbose = run.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert!(!verbose.required);
}

#[test]
fn enum_values_populated() {
    let schema = build_tool_schema(&cmd_with_enum());
    let fmt_tool = schema.tools.iter().find(|t| t.name == "format").unwrap();
    let fmt_param = fmt_tool
        .parameters
        .iter()
        .find(|p| p.name == "fmt")
        .unwrap();
    let enums = fmt_param.enum_values.as_ref().unwrap();
    assert_eq!(enums, &["json", "csv", "yaml"]);
}

#[test]
fn default_value_captured() {
    let schema = build_tool_schema(&cmd_with_enum());
    let fmt_tool = schema.tools.iter().find(|t| t.name == "format").unwrap();
    let fmt_param = fmt_tool
        .parameters
        .iter()
        .find(|p| p.name == "fmt")
        .unwrap();
    assert_eq!(fmt_param.default.as_deref(), Some("json"));
}

#[test]
fn description_populated() {
    let schema = build_tool_schema(&simple_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let verbose = run.parameters.iter().find(|p| p.name == "verbose").unwrap();
    assert_eq!(
        verbose.description.as_deref(),
        Some("Enable verbose output")
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn no_subcommands_has_only_root_tool() {
    let schema = build_tool_schema(&cmd_no_subcommands());
    assert_eq!(schema.tools.len(), 1);
    assert_eq!(schema.tools[0].name, "bare");
}

#[test]
fn subcommand_with_no_params_has_empty_parameters() {
    let schema = build_tool_schema(&cmd_no_params_subcommand());
    let noop = schema.tools.iter().find(|t| t.name == "noop").unwrap();
    assert!(noop.parameters.is_empty());
}

#[test]
fn deeply_nested_only_shows_direct_subcommands() {
    let schema = build_tool_schema(&cmd_deeply_nested());
    // Only root + level1 should appear (not level2, since it's nested deeper)
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"deep"));
    assert!(names.contains(&"level1"));
}

#[test]
fn command_without_version_uses_unknown() {
    let cmd = Command::new("nover").about("No version");
    let schema = build_tool_schema(&cmd);
    assert_eq!(schema.version, "unknown");
}

#[test]
fn command_without_about_uses_empty_string() {
    let cmd = Command::new("nodesc").version("1.0.0");
    let schema = build_tool_schema(&cmd);
    assert_eq!(schema.description, "");
}

// ---------------------------------------------------------------------------
// Render — JSON Schema format
// ---------------------------------------------------------------------------

#[test]
fn jsonschema_has_schema_field() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&output);
    assert_eq!(v["$schema"], "https://json-schema.org/draft-07/schema#");
}

#[test]
fn jsonschema_tools_have_parameters_object() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&output);
    for tool in v["tools"].as_array().unwrap() {
        assert_eq!(tool["parameters"]["type"], "object");
    }
}

#[test]
fn jsonschema_required_array_correct() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&output);
    let run_tool = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "run")
        .unwrap();
    let required = run_tool["parameters"]["required"].as_array().unwrap();
    assert!(required.iter().any(|r| r == "input"));
    assert!(!required.iter().any(|r| r == "verbose"));
}

// ---------------------------------------------------------------------------
// Render — OpenAI format
// ---------------------------------------------------------------------------

#[test]
fn openai_has_functions_key() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v = parse_json(&output);
    assert!(v.get("functions").is_some());
}

#[test]
fn openai_functions_have_name_and_parameters() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v = parse_json(&output);
    for func in v["functions"].as_array().unwrap() {
        assert!(func.get("name").is_some());
        assert!(func.get("parameters").is_some());
    }
}

// ---------------------------------------------------------------------------
// Render — Anthropic format
// ---------------------------------------------------------------------------

#[test]
fn anthropic_has_tools_key() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v = parse_json(&output);
    assert!(v.get("tools").is_some());
}

#[test]
fn anthropic_tools_use_input_schema() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v = parse_json(&output);
    for tool in v["tools"].as_array().unwrap() {
        assert!(tool.get("input_schema").is_some());
        assert_eq!(tool["input_schema"]["type"], "object");
    }
}

// ---------------------------------------------------------------------------
// Render — Clap (raw) format
// ---------------------------------------------------------------------------

#[test]
fn clap_format_round_trips_through_serde() {
    let schema = build_tool_schema(&simple_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let deserialized: ToolSchemaOutput = serde_json::from_str(&output).unwrap();
    assert_eq!(deserialized.name, "demo");
    assert_eq!(deserialized.tools.len(), schema.tools.len());
}

// ---------------------------------------------------------------------------
// Pretty vs compact
// ---------------------------------------------------------------------------

#[test]
fn pretty_output_is_longer_than_compact() {
    let schema = build_tool_schema(&simple_cmd());
    let compact = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let pretty = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    assert!(pretty.len() > compact.len());
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn schema_output_is_deterministic() {
    let a = render_output(
        &build_tool_schema(&simple_cmd()),
        ToolSchemaFormat::Jsonschema,
        false,
    )
    .unwrap();
    let b = render_output(
        &build_tool_schema(&simple_cmd()),
        ToolSchemaFormat::Jsonschema,
        false,
    )
    .unwrap();
    assert_eq!(a, b);
}

#[test]
fn all_formats_produce_deterministic_output() {
    let formats = [
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Clap,
    ];
    for fmt in formats {
        let a = render_output(&build_tool_schema(&simple_cmd()), fmt, false).unwrap();
        let b = render_output(&build_tool_schema(&simple_cmd()), fmt, false).unwrap();
        assert_eq!(a, b, "format {fmt:?} should be deterministic");
    }
}

#[test]
fn all_formats_produce_valid_json() {
    let formats = [
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Clap,
    ];
    for fmt in formats {
        let output = render_output(&build_tool_schema(&simple_cmd()), fmt, false).unwrap();
        let parsed: Result<Value, _> = serde_json::from_str(&output);
        assert!(parsed.is_ok(), "format {fmt:?} should produce valid JSON");
    }
}
