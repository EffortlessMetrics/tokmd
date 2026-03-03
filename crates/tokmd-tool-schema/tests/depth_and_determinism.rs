//! Tests for deeply nested subcommands, determinism guarantees,
//! and edge cases not covered by other test files.
//!
//! Focus areas:
//! - Deeply nested subcommand trees (3+ levels)
//! - Schema determinism across repeated builds
//! - Commands with all parameter action types simultaneously
//! - Format-specific key exclusion guarantees
//! - Zero-tool and single-tool edge cases

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{ToolSchemaFormat, ToolSchemaOutput, build_tool_schema, render_output};

// ── helpers ──────────────────────────────────────────────────────────────

/// A command with a deeply nested subcommand tree (3 levels deep).
fn deeply_nested_cmd() -> Command {
    Command::new("deep")
        .version("1.0.0")
        .about("Deep nesting test")
        .subcommand(
            Command::new("level1")
                .about("First level")
                .arg(
                    Arg::new("flag1")
                        .long("flag1")
                        .action(ArgAction::SetTrue)
                        .help("Level 1 flag"),
                )
                .subcommand(
                    Command::new("level2")
                        .about("Second level")
                        .arg(
                            Arg::new("count")
                                .long("count")
                                .action(ArgAction::Count)
                                .help("A counter"),
                        )
                        .subcommand(
                            Command::new("level3").about("Third level").arg(
                                Arg::new("value")
                                    .long("value")
                                    .required(true)
                                    .help("A required value"),
                            ),
                        ),
                ),
        )
}

/// A command with every parameter action type used simultaneously.
fn all_actions_cmd() -> Command {
    Command::new("all-actions")
        .version("1.0.0")
        .about("All parameter action types")
        .arg(
            Arg::new("string-param")
                .long("string-param")
                .help("A plain string"),
        )
        .arg(
            Arg::new("bool-true")
                .long("bool-true")
                .action(ArgAction::SetTrue)
                .help("A SetTrue flag"),
        )
        .arg(
            Arg::new("bool-false")
                .long("bool-false")
                .action(ArgAction::SetFalse)
                .help("A SetFalse flag"),
        )
        .arg(
            Arg::new("counter")
                .long("counter")
                .short('c')
                .action(ArgAction::Count)
                .help("A counter"),
        )
        .arg(
            Arg::new("list")
                .long("list")
                .action(ArgAction::Append)
                .help("An appendable list"),
        )
        .arg(
            Arg::new("with-enum")
                .long("with-enum")
                .value_parser(["alpha", "beta", "gamma"])
                .default_value("alpha")
                .help("Enum with default"),
        )
        .arg(
            Arg::new("required-val")
                .long("required-val")
                .required(true)
                .help("A required string"),
        )
}

/// A command with many subcommands (10+).
fn many_subcommands_cmd() -> Command {
    Command::new("multi")
        .version("2.0.0")
        .about("Many subs")
        .subcommand(
            Command::new("cmd-0")
                .about("Subcommand 0")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-1")
                .about("Subcommand 1")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-2")
                .about("Subcommand 2")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-3")
                .about("Subcommand 3")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-4")
                .about("Subcommand 4")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-5")
                .about("Subcommand 5")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-6")
                .about("Subcommand 6")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-7")
                .about("Subcommand 7")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-8")
                .about("Subcommand 8")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-9")
                .about("Subcommand 9")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-10")
                .about("Subcommand 10")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
        .subcommand(
            Command::new("cmd-11")
                .about("Subcommand 11")
                .arg(Arg::new("input").long("input").help("Input path")),
        )
}

// ── Scenario: Deeply nested subcommands ─────────────────────────────────

#[test]
fn given_deeply_nested_cmd_when_schema_built_then_only_direct_children_included() {
    // build_tool_schema only includes direct subcommands, not nested ones
    let schema = build_tool_schema(&deeply_nested_cmd());
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();

    // Root + level1 (direct child) are included
    assert!(names.contains(&"deep"), "root should be present");
    assert!(
        names.contains(&"level1"),
        "direct subcommand should be present"
    );
    // level2 and level3 are nested — not direct subcommands of root
    assert!(
        !names.contains(&"level2"),
        "nested subcommands should not appear at top level"
    );
    assert!(
        !names.contains(&"level3"),
        "deeply nested subcommands should not appear at top level"
    );
}

#[test]
fn given_deeply_nested_cmd_when_rendered_then_valid_json_in_all_formats() {
    let schema = build_tool_schema(&deeply_nested_cmd());
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

// ── Scenario: All action types in one command ───────────────────────────

#[test]
fn given_all_actions_cmd_when_schema_built_then_each_type_correctly_inferred() {
    let schema = build_tool_schema(&all_actions_cmd());
    let root = &schema.tools[0];

    let find = |name: &str| root.parameters.iter().find(|p| p.name == name).unwrap();

    assert_eq!(find("string-param").param_type, "string");
    assert_eq!(find("bool-true").param_type, "boolean");
    assert_eq!(find("bool-false").param_type, "boolean");
    assert_eq!(find("counter").param_type, "integer");
    assert_eq!(find("list").param_type, "array");
    assert_eq!(find("with-enum").param_type, "string");
    assert_eq!(find("required-val").param_type, "string");
}

#[test]
fn given_all_actions_cmd_when_schema_built_then_required_and_optional_correct() {
    let schema = build_tool_schema(&all_actions_cmd());
    let root = &schema.tools[0];

    let find = |name: &str| root.parameters.iter().find(|p| p.name == name).unwrap();

    assert!(find("required-val").required);
    assert!(!find("string-param").required);
    assert!(!find("bool-true").required);
    assert!(!find("counter").required);
    assert!(!find("list").required);
}

#[test]
fn given_all_actions_cmd_when_rendered_openai_then_required_array_has_only_required() {
    let schema = build_tool_schema(&all_actions_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let root_func = &v["functions"][0];
    let required: Vec<&str> = root_func["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    assert!(required.contains(&"required-val"));
    assert_eq!(required.len(), 1, "only required-val should be required");
}

// ── Scenario: Many subcommands ──────────────────────────────────────────

#[test]
fn given_many_subcommands_when_schema_built_then_all_included() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    // 12 subcommands + 1 root = 13 tools
    assert_eq!(schema.tools.len(), 13);
    for i in 0..12 {
        let name = format!("cmd-{i}");
        assert!(
            schema.tools.iter().any(|t| t.name == name),
            "missing subcommand {}",
            name
        );
    }
}

#[test]
fn given_many_subcommands_when_rendered_then_all_formats_produce_valid_json() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let json_str = render_output(&schema, fmt, false).unwrap();
        let v: Value = serde_json::from_str(&json_str).unwrap();
        // Verify structure has correct tool count
        match fmt {
            ToolSchemaFormat::Openai => {
                assert_eq!(v["functions"].as_array().unwrap().len(), 13);
            }
            ToolSchemaFormat::Anthropic => {
                assert_eq!(v["tools"].as_array().unwrap().len(), 13);
            }
            ToolSchemaFormat::Jsonschema => {
                assert_eq!(v["tools"].as_array().unwrap().len(), 13);
            }
            ToolSchemaFormat::Clap => {
                let deser: ToolSchemaOutput = serde_json::from_str(&json_str).unwrap();
                assert_eq!(deser.tools.len(), 13);
            }
        }
    }
}

// ── Scenario: Determinism — repeated builds identical ───────────────────

#[test]
fn given_same_command_when_schema_built_ten_times_then_all_identical() {
    let outputs: Vec<String> = (0..10)
        .map(|_| {
            let cmd = all_actions_cmd();
            let schema = build_tool_schema(&cmd);
            render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap()
        })
        .collect();

    let first = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(first, output, "iteration {} diverged from first", i);
    }
}

#[test]
fn given_same_command_when_schema_rendered_in_all_formats_then_each_format_deterministic() {
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let cmd = all_actions_cmd();
        let schema = build_tool_schema(&cmd);
        let a = render_output(&schema, fmt, true).unwrap();
        let b = render_output(&schema, fmt, true).unwrap();
        assert_eq!(a, b, "non-deterministic output for {:?}", fmt);
    }
}

// ── Scenario: Format-specific key exclusion guarantees ──────────────────

#[test]
fn given_openai_format_then_no_tools_or_schema_or_input_schema_keys() {
    let schema = build_tool_schema(&all_actions_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert!(v.get("tools").is_none(), "OpenAI must not have 'tools' key");
    assert!(
        v.get("$schema").is_none(),
        "OpenAI must not have '$schema' key"
    );
    // Each function uses "parameters", not "input_schema"
    for func in v["functions"].as_array().unwrap() {
        assert!(
            func.get("input_schema").is_none(),
            "OpenAI functions must not have 'input_schema'"
        );
        assert!(
            func.get("parameters").is_some(),
            "OpenAI functions must have 'parameters'"
        );
    }
}

#[test]
fn given_anthropic_format_then_no_functions_or_schema_or_parameters_keys() {
    let schema = build_tool_schema(&all_actions_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        v.get("functions").is_none(),
        "Anthropic must not have 'functions' key"
    );
    assert!(
        v.get("$schema").is_none(),
        "Anthropic must not have '$schema' key"
    );
    // Each tool uses "input_schema", not "parameters"
    for tool in v["tools"].as_array().unwrap() {
        assert!(
            tool.get("parameters").is_none(),
            "Anthropic tools must not have 'parameters'"
        );
        assert!(
            tool.get("input_schema").is_some(),
            "Anthropic tools must have 'input_schema'"
        );
    }
}

#[test]
fn given_jsonschema_format_then_has_schema_draft_and_no_functions_key() {
    let schema = build_tool_schema(&all_actions_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    assert!(
        v.get("$schema").is_some(),
        "JSON Schema must have '$schema'"
    );
    assert!(
        v.get("functions").is_none(),
        "JSON Schema must not have 'functions'"
    );
    // Tools use "parameters" with type: "object"
    for tool in v["tools"].as_array().unwrap() {
        assert_eq!(tool["parameters"]["type"].as_str(), Some("object"));
    }
}

// ── Scenario: Enum values propagate to all formats ──────────────────────

#[test]
fn given_param_with_enums_when_rendered_in_each_format_then_enums_present() {
    let schema = build_tool_schema(&all_actions_cmd());

    // OpenAI
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();
    let prop = &v["functions"][0]["parameters"]["properties"]["with-enum"];
    assert_eq!(
        prop["enum"].as_array().unwrap().len(),
        3,
        "OpenAI should have enum values"
    );

    // Anthropic
    let json_str = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();
    let prop = &v["tools"][0]["input_schema"]["properties"]["with-enum"];
    assert_eq!(
        prop["enum"].as_array().unwrap().len(),
        3,
        "Anthropic should have enum values"
    );

    // JSON Schema
    let json_str = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();
    let prop = &v["tools"][0]["parameters"]["properties"]["with-enum"];
    assert_eq!(
        prop["enum"].as_array().unwrap().len(),
        3,
        "JSON Schema should have enum values"
    );
    // JSON Schema also includes default
    assert_eq!(prop["default"].as_str(), Some("alpha"));
}

// ── Scenario: ToolSchemaOutput clone and debug ──────────────────────────

#[test]
fn given_schema_output_when_cloned_then_identical() {
    let schema = build_tool_schema(&all_actions_cmd());
    let cloned = schema.clone();

    let a = serde_json::to_string(&schema).unwrap();
    let b = serde_json::to_string(&cloned).unwrap();
    assert_eq!(a, b, "cloned schema must serialize identically");
}

#[test]
fn given_schema_output_when_debug_printed_then_does_not_panic() {
    let schema = build_tool_schema(&all_actions_cmd());
    let debug = format!("{:?}", schema);
    assert!(!debug.is_empty());
    assert!(debug.contains("all-actions"));
}

// ── Scenario: ToolSchemaFormat serde roundtrip ──────────────────────────

#[test]
fn given_tool_schema_format_when_serialized_then_roundtrips() {
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let json = serde_json::to_string(&fmt).unwrap();
        let back: ToolSchemaFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, fmt, "roundtrip failed for {:?}", fmt);
    }
}

// ── Scenario: ParameterSchema skip_serializing_if behavior ──────────────

#[test]
fn given_parameter_with_no_optional_fields_then_json_omits_them() {
    use tokmd_tool_schema::ParameterSchema;

    let param = ParameterSchema {
        name: "bare".to_string(),
        description: None,
        param_type: "string".to_string(),
        required: false,
        default: None,
        enum_values: None,
    };

    let json = serde_json::to_string(&param).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();

    assert!(
        v.get("description").is_none(),
        "None description should be omitted"
    );
    assert!(v.get("default").is_none(), "None default should be omitted");
    assert!(
        v.get("enum_values").is_none(),
        "None enum_values should be omitted"
    );
    // Required fields always present
    assert!(v.get("name").is_some());
    assert!(v.get("type").is_some());
    assert!(v.get("required").is_some());
}

// ── Scenario: Command with no about text on subcommands ─────────────────

#[test]
fn given_subcommand_without_about_then_description_is_empty() {
    let cmd = Command::new("app")
        .version("1.0.0")
        .about("Main app")
        .subcommand(Command::new("bare-sub"));

    let schema = build_tool_schema(&cmd);
    let bare = schema.tools.iter().find(|t| t.name == "bare-sub").unwrap();
    assert_eq!(bare.description, "");
}

#[test]
fn given_subcommand_without_about_when_rendered_then_description_is_empty_string() {
    let cmd = Command::new("app")
        .version("1.0.0")
        .about("Main app")
        .subcommand(Command::new("bare-sub"));

    let schema = build_tool_schema(&cmd);
    let json_str = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json_str).unwrap();

    let bare = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "bare-sub")
        .unwrap();
    assert_eq!(bare["description"].as_str(), Some(""));
}
