//! W62 depth tests for tool-schema generation.
//!
//! ~50 tests covering: OpenAI, Anthropic, JSON Schema and Clap formats,
//! command tree traversal, parameter type mapping, required/optional,
//! enum handling, nested subcommands, determinism, and property-based tests.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{TOOL_SCHEMA_VERSION, ToolSchemaFormat, build_tool_schema, render_output};

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn simple_cmd() -> Command {
    Command::new("mytool")
        .version("2.0.0")
        .about("A test tool")
        .subcommand(
            Command::new("scan")
                .about("Scan files")
                .arg(
                    Arg::new("path")
                        .long("path")
                        .required(true)
                        .help("Target path"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .action(ArgAction::SetTrue)
                        .help("Verbose output"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export results")
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "tsv"])
                        .default_value("json")
                        .help("Output format"),
                )
                .arg(
                    Arg::new("count")
                        .long("count")
                        .action(ArgAction::Count)
                        .help("Verbosity count"),
                ),
        )
}

fn nested_cmd() -> Command {
    Command::new("parent")
        .version("1.0.0")
        .about("Parent cmd")
        .subcommand(
            Command::new("child").about("Child cmd").subcommand(
                Command::new("grandchild")
                    .about("Grandchild cmd")
                    .arg(Arg::new("depth").long("depth").help("Depth level")),
            ),
        )
}

fn empty_cmd() -> Command {
    Command::new("empty")
        .version("0.1.0")
        .about("No subcommands")
}

fn multi_arg_cmd() -> Command {
    Command::new("multi")
        .version("1.0.0")
        .about("Multi-arg command")
        .subcommand(
            Command::new("run")
                .about("Run task")
                .arg(
                    Arg::new("items")
                        .long("items")
                        .action(ArgAction::Append)
                        .help("Items to process"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Dry run mode"),
                )
                .arg(
                    Arg::new("no-color")
                        .long("no-color")
                        .action(ArgAction::SetFalse)
                        .help("Disable color"),
                )
                .arg(
                    Arg::new("input")
                        .long("input")
                        .required(true)
                        .help("Input file"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .default_value("out.json")
                        .help("Output file"),
                ),
        )
}

fn parse_json(s: &str) -> Value {
    serde_json::from_str(s).expect("valid JSON")
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. build_tool_schema basics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_schema_name_matches_command() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.name, "mytool");
}

#[test]
fn w62_schema_version_matches_constant() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.schema_version, TOOL_SCHEMA_VERSION);
}

#[test]
fn w62_schema_version_number() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.version, "2.0.0");
}

#[test]
fn w62_schema_description() {
    let schema = build_tool_schema(&simple_cmd());
    assert_eq!(schema.description, "A test tool");
}

#[test]
fn w62_schema_includes_root_tool() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(schema.tools.iter().any(|t| t.name == "mytool"));
}

#[test]
fn w62_schema_includes_subcommands() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(schema.tools.iter().any(|t| t.name == "scan"));
    assert!(schema.tools.iter().any(|t| t.name == "export"));
}

#[test]
fn w62_schema_excludes_help_subcommand() {
    let schema = build_tool_schema(&simple_cmd());
    assert!(!schema.tools.iter().any(|t| t.name == "help"));
}

#[test]
fn w62_empty_cmd_has_root_tool_only() {
    let schema = build_tool_schema(&empty_cmd());
    assert_eq!(schema.tools.len(), 1);
    assert_eq!(schema.tools[0].name, "empty");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Parameter type mapping
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_set_true_maps_to_boolean() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let verbose = scan
        .parameters
        .iter()
        .find(|p| p.name == "verbose")
        .unwrap();
    assert_eq!(verbose.param_type, "boolean");
}

#[test]
fn w62_count_maps_to_integer() {
    let schema = build_tool_schema(&simple_cmd());
    let export = schema.tools.iter().find(|t| t.name == "export").unwrap();
    let count = export
        .parameters
        .iter()
        .find(|p| p.name == "count")
        .unwrap();
    assert_eq!(count.param_type, "integer");
}

#[test]
fn w62_append_maps_to_array() {
    let schema = build_tool_schema(&multi_arg_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let items = run.parameters.iter().find(|p| p.name == "items").unwrap();
    assert_eq!(items.param_type, "array");
}

#[test]
fn w62_set_false_maps_to_boolean() {
    let schema = build_tool_schema(&multi_arg_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let no_color = run
        .parameters
        .iter()
        .find(|p| p.name == "no-color")
        .unwrap();
    assert_eq!(no_color.param_type, "boolean");
}

#[test]
fn w62_regular_arg_maps_to_string() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path.param_type, "string");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Required vs optional parameters
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_required_param_marked_true() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.required);
}

#[test]
fn w62_optional_param_marked_false() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let verbose = scan
        .parameters
        .iter()
        .find(|p| p.name == "verbose")
        .unwrap();
    assert!(!verbose.required);
}

#[test]
fn w62_default_value_captured() {
    let schema = build_tool_schema(&simple_cmd());
    let export = schema.tools.iter().find(|t| t.name == "export").unwrap();
    let format = export
        .parameters
        .iter()
        .find(|p| p.name == "format")
        .unwrap();
    assert_eq!(format.default.as_deref(), Some("json"));
}

#[test]
fn w62_no_default_is_none() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.default.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Enum parameter handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_enum_values_captured() {
    let schema = build_tool_schema(&simple_cmd());
    let export = schema.tools.iter().find(|t| t.name == "export").unwrap();
    let format = export
        .parameters
        .iter()
        .find(|p| p.name == "format")
        .unwrap();
    let enums = format.enum_values.as_ref().expect("should have enums");
    assert_eq!(enums, &["json", "csv", "tsv"]);
}

#[test]
fn w62_non_enum_has_no_values() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert!(path.enum_values.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Nested subcommand handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_nested_child_appears_as_tool() {
    let schema = build_tool_schema(&nested_cmd());
    assert!(schema.tools.iter().any(|t| t.name == "child"));
}

#[test]
fn w62_nested_grandchild_not_top_level() {
    // build_tool_schema only traverses one level of subcommands
    let schema = build_tool_schema(&nested_cmd());
    assert!(!schema.tools.iter().any(|t| t.name == "grandchild"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Help/version filtering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_help_arg_excluded_from_parameters() {
    let schema = build_tool_schema(&simple_cmd());
    for tool in &schema.tools {
        assert!(
            !tool.parameters.iter().any(|p| p.name == "help"),
            "tool {} should not have help param",
            tool.name
        );
    }
}

#[test]
fn w62_version_arg_excluded_from_parameters() {
    let schema = build_tool_schema(&simple_cmd());
    for tool in &schema.tools {
        assert!(
            !tool.parameters.iter().any(|p| p.name == "version"),
            "tool {} should not have version param",
            tool.name
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. JSON Schema format rendering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_jsonschema_is_valid_json() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let _: Value = parse_json(&out);
}

#[test]
fn w62_jsonschema_has_schema_ref() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&out);
    assert!(v["$schema"].as_str().unwrap().contains("json-schema.org"));
}

#[test]
fn w62_jsonschema_has_tools_array() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&out);
    assert!(v["tools"].is_array());
}

#[test]
fn w62_jsonschema_tool_has_parameters_object() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&out);
    let tools = v["tools"].as_array().unwrap();
    for tool in tools {
        assert_eq!(tool["parameters"]["type"], "object");
    }
}

#[test]
fn w62_jsonschema_required_array() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&out);
    let scan = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "scan")
        .unwrap();
    let required = scan["parameters"]["required"].as_array().unwrap();
    assert!(required.iter().any(|r| r == "path"));
    assert!(!required.iter().any(|r| r == "verbose"));
}

#[test]
fn w62_jsonschema_enum_in_properties() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v = parse_json(&out);
    let export = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "export")
        .unwrap();
    let enums = export["parameters"]["properties"]["format"]["enum"]
        .as_array()
        .unwrap();
    assert_eq!(enums.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. OpenAI format rendering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_openai_has_functions_key() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v = parse_json(&out);
    assert!(v["functions"].is_array());
}

#[test]
fn w62_openai_function_has_name_description_parameters() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v = parse_json(&out);
    for func in v["functions"].as_array().unwrap() {
        assert!(func["name"].is_string());
        assert!(func["description"].is_string());
        assert!(func["parameters"].is_object());
    }
}

#[test]
fn w62_openai_parameters_type_is_object() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v = parse_json(&out);
    for func in v["functions"].as_array().unwrap() {
        assert_eq!(func["parameters"]["type"], "object");
    }
}

#[test]
fn w62_openai_no_input_schema_key() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v = parse_json(&out);
    for func in v["functions"].as_array().unwrap() {
        assert!(func.get("input_schema").is_none());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Anthropic format rendering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_anthropic_has_tools_key() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v = parse_json(&out);
    assert!(v["tools"].is_array());
}

#[test]
fn w62_anthropic_uses_input_schema() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v = parse_json(&out);
    for tool in v["tools"].as_array().unwrap() {
        assert!(
            tool["input_schema"].is_object(),
            "Anthropic uses input_schema"
        );
    }
}

#[test]
fn w62_anthropic_no_parameters_key() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v = parse_json(&out);
    for tool in v["tools"].as_array().unwrap() {
        assert!(
            tool.get("parameters").is_none(),
            "Anthropic should not have parameters key"
        );
    }
}

#[test]
fn w62_anthropic_input_schema_type_is_object() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v = parse_json(&out);
    for tool in v["tools"].as_array().unwrap() {
        assert_eq!(tool["input_schema"]["type"], "object");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Clap format rendering
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_clap_format_is_valid_json() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let _: Value = parse_json(&out);
}

#[test]
fn w62_clap_has_schema_version() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let v = parse_json(&out);
    assert_eq!(v["schema_version"], TOOL_SCHEMA_VERSION);
}

#[test]
fn w62_clap_has_tools_with_parameters() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let v = parse_json(&out);
    assert!(v["tools"].is_array());
    let scan = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "scan")
        .unwrap();
    assert!(scan["parameters"].is_array());
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Pretty printing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_pretty_output_has_newlines() {
    let schema = build_tool_schema(&simple_cmd());
    let pretty = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    assert!(pretty.contains('\n'));
}

#[test]
fn w62_compact_output_has_no_newlines() {
    let schema = build_tool_schema(&simple_cmd());
    let compact = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    assert!(!compact.contains('\n'));
}

#[test]
fn w62_pretty_and_compact_parse_to_same_value() {
    let schema = build_tool_schema(&simple_cmd());
    let pretty = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    let compact = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let vp: Value = parse_json(&pretty);
    let vc: Value = parse_json(&compact);
    assert_eq!(vp, vc);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_determinism_same_cmd_same_schema() {
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
fn w62_determinism_openai_stable() {
    let a = render_output(
        &build_tool_schema(&simple_cmd()),
        ToolSchemaFormat::Openai,
        false,
    )
    .unwrap();
    let b = render_output(
        &build_tool_schema(&simple_cmd()),
        ToolSchemaFormat::Openai,
        false,
    )
    .unwrap();
    assert_eq!(a, b);
}

#[test]
fn w62_determinism_anthropic_stable() {
    let a = render_output(
        &build_tool_schema(&simple_cmd()),
        ToolSchemaFormat::Anthropic,
        false,
    )
    .unwrap();
    let b = render_output(
        &build_tool_schema(&simple_cmd()),
        ToolSchemaFormat::Anthropic,
        false,
    )
    .unwrap();
    assert_eq!(a, b);
}

#[test]
fn w62_determinism_100_iterations() {
    let reference = render_output(
        &build_tool_schema(&simple_cmd()),
        ToolSchemaFormat::Openai,
        false,
    )
    .unwrap();
    for _ in 0..100 {
        let out = render_output(
            &build_tool_schema(&simple_cmd()),
            ToolSchemaFormat::Openai,
            false,
        )
        .unwrap();
        assert_eq!(out, reference);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Cross-format structural comparison
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_all_formats_same_tool_count() {
    let schema = build_tool_schema(&simple_cmd());
    let oai = parse_json(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    let ant = parse_json(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());
    let js = parse_json(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap());

    let oai_count = oai["functions"].as_array().unwrap().len();
    let ant_count = ant["tools"].as_array().unwrap().len();
    let js_count = js["tools"].as_array().unwrap().len();

    assert_eq!(oai_count, ant_count);
    assert_eq!(ant_count, js_count);
}

#[test]
fn w62_all_formats_same_tool_names() {
    let schema = build_tool_schema(&simple_cmd());
    let oai = parse_json(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap());
    let ant = parse_json(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap());

    let oai_names: Vec<&str> = oai["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    let ant_names: Vec<&str> = ant["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(oai_names, ant_names);
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Description and help text
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_tool_description_preserved() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    assert_eq!(scan.description, "Scan files");
}

#[test]
fn w62_param_help_becomes_description() {
    let schema = build_tool_schema(&simple_cmd());
    let scan = schema.tools.iter().find(|t| t.name == "scan").unwrap();
    let path = scan.parameters.iter().find(|p| p.name == "path").unwrap();
    assert_eq!(path.description.as_deref(), Some("Target path"));
}

#[test]
fn w62_missing_version_defaults_to_unknown() {
    let cmd = Command::new("no-ver").about("No version");
    let schema = build_tool_schema(&cmd);
    assert_eq!(schema.version, "unknown");
}

#[test]
fn w62_missing_about_defaults_to_empty() {
    let cmd = Command::new("no-about").version("1.0.0");
    let schema = build_tool_schema(&cmd);
    assert_eq!(schema.description, "");
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Snapshot tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w62_snapshot_jsonschema_output() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    let v: Value = parse_json(&out);
    insta::assert_json_snapshot!("w62_jsonschema", v);
}

#[test]
fn w62_snapshot_openai_output() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    let v: Value = parse_json(&out);
    insta::assert_json_snapshot!("w62_openai", v);
}

#[test]
fn w62_snapshot_anthropic_output() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Anthropic, true).unwrap();
    let v: Value = parse_json(&out);
    insta::assert_json_snapshot!("w62_anthropic", v);
}

#[test]
fn w62_snapshot_clap_output() {
    let schema = build_tool_schema(&simple_cmd());
    let out = render_output(&schema, ToolSchemaFormat::Clap, true).unwrap();
    let v: Value = parse_json(&out);
    insta::assert_json_snapshot!("w62_clap", v);
}
