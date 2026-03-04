//! Deep tests for tool-schema generation (Wave 47).
//!
//! Covers: determinism across formats, cross-format consistency,
//! parameter type coverage, edge cases (bare/deeply-nested commands),
//! pretty vs compact equivalence, and structural invariants.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{ToolSchemaFormat, build_tool_schema, render_output};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Command exercising every parameter type.
fn all_types_cmd() -> Command {
    Command::new("alltypes")
        .version("1.0.0")
        .about("Command with all parameter types")
        .arg(
            Arg::new("name")
                .long("name")
                .required(true)
                .help("A string param"),
        )
        .arg(
            Arg::new("flag")
                .long("flag")
                .action(ArgAction::SetTrue)
                .help("A boolean param"),
        )
        .arg(
            Arg::new("level")
                .long("level")
                .action(ArgAction::Count)
                .help("An integer param"),
        )
        .arg(
            Arg::new("items")
                .long("items")
                .action(ArgAction::Append)
                .help("An array param"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["fast", "slow", "balanced"])
                .help("An enum param"),
        )
}

/// Command with three levels of nesting.
fn deeply_nested_cmd() -> Command {
    Command::new("root")
        .version("0.1.0")
        .about("Root with deep nesting")
        .subcommand(
            Command::new("level1").about("First level").subcommand(
                Command::new("level2").about("Second level").subcommand(
                    Command::new("level3")
                        .about("Third level")
                        .arg(Arg::new("deep-arg").long("deep-arg").help("Deep argument")),
                ),
            ),
        )
}

/// Command with many subcommands.
fn many_subcommands_cmd() -> Command {
    static SUBS: &[(&str, &str)] = &[
        ("sub0", "Subcommand 0"),
        ("sub1", "Subcommand 1"),
        ("sub2", "Subcommand 2"),
        ("sub3", "Subcommand 3"),
        ("sub4", "Subcommand 4"),
        ("sub5", "Subcommand 5"),
        ("sub6", "Subcommand 6"),
        ("sub7", "Subcommand 7"),
        ("sub8", "Subcommand 8"),
        ("sub9", "Subcommand 9"),
    ];
    let mut cmd = Command::new("multi")
        .version("3.0.0")
        .about("Many subcommands");
    for &(name, about) in SUBS {
        cmd = cmd.subcommand(
            Command::new(name)
                .about(about)
                .arg(Arg::new("arg").long("arg").help("An argument")),
        );
    }
    cmd
}

/// Command with only boolean flags.
fn bool_only_cmd() -> Command {
    Command::new("flags")
        .version("0.0.1")
        .about("Only boolean flags")
        .arg(Arg::new("a").long("a").action(ArgAction::SetTrue))
        .arg(Arg::new("b").long("b").action(ArgAction::SetTrue))
        .arg(Arg::new("c").long("c").action(ArgAction::SetFalse))
}

/// Command with no version or about.
fn bare_cmd() -> Command {
    Command::new("bare")
}

// ===========================================================================
// 1. Determinism: same command → identical output for every format
// ===========================================================================

#[test]
fn determinism_all_formats() {
    let formats = [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ];
    for fmt in formats {
        let s1 = build_tool_schema(&all_types_cmd());
        let s2 = build_tool_schema(&all_types_cmd());
        let r1 = render_output(&s1, fmt, false).unwrap();
        let r2 = render_output(&s2, fmt, false).unwrap();
        assert_eq!(r1, r2, "non-deterministic for format {fmt:?}");
    }
}

#[test]
fn determinism_pretty_all_formats() {
    let formats = [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ];
    for fmt in formats {
        let schema = build_tool_schema(&all_types_cmd());
        let a = render_output(&schema, fmt, true).unwrap();
        let b = render_output(&schema, fmt, true).unwrap();
        assert_eq!(a, b, "pretty output non-deterministic for {fmt:?}");
    }
}

// ===========================================================================
// 2. Cross-format consistency: same tools appear in all formats
// ===========================================================================

#[test]
fn cross_format_same_tool_count() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    let tool_count = schema.tools.len();

    let openai: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap())
            .unwrap();
    let anthropic: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap())
            .unwrap();
    let jsonschema: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap())
            .unwrap();

    assert_eq!(openai["functions"].as_array().unwrap().len(), tool_count);
    assert_eq!(anthropic["tools"].as_array().unwrap().len(), tool_count);
    assert_eq!(jsonschema["tools"].as_array().unwrap().len(), tool_count);
}

#[test]
fn cross_format_same_tool_names() {
    let schema = build_tool_schema(&many_subcommands_cmd());

    let openai: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap())
            .unwrap();
    let anthropic: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap())
            .unwrap();

    let openai_names: Vec<&str> = openai["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    let anthropic_names: Vec<&str> = anthropic["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    assert_eq!(openai_names, anthropic_names);
}

// ===========================================================================
// 3. All formats produce valid JSON
// ===========================================================================

#[test]
fn all_formats_produce_valid_json() {
    let formats = [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ];
    for cmd in [all_types_cmd(), bare_cmd(), deeply_nested_cmd()] {
        let schema = build_tool_schema(&cmd);
        for fmt in formats {
            for pretty in [true, false] {
                let output = render_output(&schema, fmt, pretty).unwrap();
                let parsed: Result<Value, _> = serde_json::from_str(&output);
                assert!(
                    parsed.is_ok(),
                    "invalid JSON for cmd={}, fmt={fmt:?}, pretty={pretty}",
                    cmd.get_name()
                );
            }
        }
    }
}

// ===========================================================================
// 4. Pretty vs compact parse to identical structure
// ===========================================================================

#[test]
fn pretty_vs_compact_equivalent_structure() {
    let formats = [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ];
    let schema = build_tool_schema(&all_types_cmd());
    for fmt in formats {
        let compact: Value =
            serde_json::from_str(&render_output(&schema, fmt, false).unwrap()).unwrap();
        let pretty: Value =
            serde_json::from_str(&render_output(&schema, fmt, true).unwrap()).unwrap();
        assert_eq!(compact, pretty, "structure differs for {fmt:?}");
    }
}

// ===========================================================================
// 5. Parameter type coverage in rendered output
// ===========================================================================

#[test]
fn openai_all_param_types_present() {
    let schema = build_tool_schema(&all_types_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap())
            .unwrap();
    let root = &v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "alltypes")
        .unwrap()["parameters"]["properties"];

    assert_eq!(root["name"]["type"].as_str().unwrap(), "string");
    assert_eq!(root["flag"]["type"].as_str().unwrap(), "boolean");
    assert_eq!(root["level"]["type"].as_str().unwrap(), "integer");
    assert_eq!(root["items"]["type"].as_str().unwrap(), "array");
    assert!(root["mode"]["enum"].is_array());
}

#[test]
fn anthropic_all_param_types_present() {
    let schema = build_tool_schema(&all_types_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap())
            .unwrap();
    let root = &v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "alltypes")
        .unwrap()["input_schema"]["properties"];

    assert_eq!(root["name"]["type"].as_str().unwrap(), "string");
    assert_eq!(root["flag"]["type"].as_str().unwrap(), "boolean");
    assert_eq!(root["level"]["type"].as_str().unwrap(), "integer");
    assert_eq!(root["items"]["type"].as_str().unwrap(), "array");
    assert!(root["mode"]["enum"].is_array());
}

#[test]
fn jsonschema_all_param_types_present() {
    let schema = build_tool_schema(&all_types_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap())
            .unwrap();
    let root = &v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "alltypes")
        .unwrap()["parameters"]["properties"];

    assert_eq!(root["name"]["type"].as_str().unwrap(), "string");
    assert_eq!(root["flag"]["type"].as_str().unwrap(), "boolean");
    assert_eq!(root["level"]["type"].as_str().unwrap(), "integer");
    assert_eq!(root["items"]["type"].as_str().unwrap(), "array");
    assert!(root["mode"]["enum"].is_array());
}

// ===========================================================================
// 6. Structural invariants per format
// ===========================================================================

#[test]
fn openai_every_function_has_name_description_parameters() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap())
            .unwrap();
    for func in v["functions"].as_array().unwrap() {
        assert!(func["name"].is_string(), "missing name");
        assert!(func["description"].is_string(), "missing description");
        assert!(func["parameters"].is_object(), "missing parameters");
        assert_eq!(
            func["parameters"]["type"].as_str().unwrap(),
            "object",
            "parameters.type must be object"
        );
    }
}

#[test]
fn anthropic_every_tool_has_name_description_input_schema() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap())
            .unwrap();
    for tool in v["tools"].as_array().unwrap() {
        assert!(tool["name"].is_string(), "missing name");
        assert!(tool["description"].is_string(), "missing description");
        assert!(tool["input_schema"].is_object(), "missing input_schema");
        assert_eq!(
            tool["input_schema"]["type"].as_str().unwrap(),
            "object",
            "input_schema.type must be object"
        );
    }
}

#[test]
fn jsonschema_every_tool_has_name_description_parameters() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap())
            .unwrap();
    for tool in v["tools"].as_array().unwrap() {
        assert!(tool["name"].is_string(), "missing name");
        assert!(tool["description"].is_string(), "missing description");
        assert!(tool["parameters"].is_object(), "missing parameters");
    }
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn deeply_nested_only_top_level_subcommands() {
    let schema = build_tool_schema(&deeply_nested_cmd());
    let names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"root"));
    assert!(names.contains(&"level1"));
    assert!(
        !names.contains(&"level2"),
        "level2 should not appear at top level"
    );
    assert!(
        !names.contains(&"level3"),
        "level3 should not appear at top level"
    );
}

#[test]
fn bare_command_renders_all_formats() {
    let schema = build_tool_schema(&bare_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let output = render_output(&schema, fmt, false).unwrap();
        assert!(!output.is_empty(), "empty output for {fmt:?}");
    }
}

#[test]
fn bool_only_command_all_params_boolean() {
    let schema = build_tool_schema(&bool_only_cmd());
    let root = schema.tools.iter().find(|t| t.name == "flags").unwrap();
    for param in &root.parameters {
        assert_eq!(
            param.param_type, "boolean",
            "param {} should be boolean",
            param.name
        );
    }
}

#[test]
fn many_subcommands_count() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    // 10 subcommands + 1 root = 11
    assert_eq!(schema.tools.len(), 11);
}

#[test]
fn required_param_appears_in_openai_required_array() {
    let schema = build_tool_schema(&all_types_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap())
            .unwrap();
    let root = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "alltypes")
        .unwrap();
    let required = root["parameters"]["required"].as_array().unwrap();
    assert!(
        required.iter().any(|r| r.as_str() == Some("name")),
        "name should be required"
    );
    assert!(
        !required.iter().any(|r| r.as_str() == Some("flag")),
        "flag should not be required"
    );
}

#[test]
fn jsonschema_has_default_value_for_enum_param() {
    let cmd = Command::new("test").version("1.0.0").about("Test").arg(
        Arg::new("fmt")
            .long("fmt")
            .value_parser(["a", "b"])
            .default_value("a")
            .help("Format"),
    );
    let schema = build_tool_schema(&cmd);
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap())
            .unwrap();
    let root = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "test")
        .unwrap();
    let fmt_prop = &root["parameters"]["properties"]["fmt"];
    assert_eq!(fmt_prop["default"].as_str().unwrap(), "a");
    assert!(
        fmt_prop["enum"]
            .as_array()
            .unwrap()
            .contains(&Value::String("b".into()))
    );
}

#[test]
fn openai_description_field_populated_from_help() {
    let schema = build_tool_schema(&all_types_cmd());
    let v: Value =
        serde_json::from_str(&render_output(&schema, ToolSchemaFormat::Openai, false).unwrap())
            .unwrap();
    let root = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "alltypes")
        .unwrap();
    let name_desc = root["parameters"]["properties"]["name"]["description"]
        .as_str()
        .unwrap();
    assert_eq!(name_desc, "A string param");
}
