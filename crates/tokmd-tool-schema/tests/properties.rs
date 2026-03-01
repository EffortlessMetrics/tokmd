//! Property-based and extended tests for tool-schema generation.
//!
//! Covers determinism, round-trip deserialization, structural invariants
//! across all formats, and proptest-driven property verification.

use clap::{Arg, ArgAction, Command};
use proptest::prelude::*;
use serde_json::Value;
use tokmd_tool_schema::{
    ToolDefinition, ToolSchemaFormat, ToolSchemaOutput, build_tool_schema, render_output,
};

// ── helpers ─────────────────────────────────────────────────────────────

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

const ALL_FORMATS: [ToolSchemaFormat; 4] = [
    ToolSchemaFormat::Openai,
    ToolSchemaFormat::Anthropic,
    ToolSchemaFormat::Jsonschema,
    ToolSchemaFormat::Clap,
];

// ── 1. Valid JSON for every format ──────────────────────────────────────

#[test]
fn all_formats_produce_valid_json() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);

    for fmt in ALL_FORMATS {
        for pretty in [false, true] {
            let output = render_output(&schema, fmt, pretty)
                .unwrap_or_else(|e| panic!("render failed for {fmt:?} pretty={pretty}: {e}"));
            let _: Value = serde_json::from_str(&output)
                .unwrap_or_else(|e| panic!("invalid JSON for {fmt:?} pretty={pretty}: {e}"));
        }
    }
}

// ── 2. All expected commands appear ─────────────────────────────────────

#[test]
fn all_expected_commands_appear_in_all_formats() {
    let cmd = rich_cmd();
    let schema = build_tool_schema(&cmd);
    let expected = ["rich", "scan", "export"];

    // Check the ToolSchemaOutput level
    let tool_names: Vec<&str> = schema.tools.iter().map(|t| t.name.as_str()).collect();
    for name in &expected {
        assert!(
            tool_names.contains(name),
            "missing tool {name} in schema output"
        );
    }

    // Check each rendered format
    for fmt in ALL_FORMATS {
        let output = render_output(&schema, fmt, false).unwrap();
        let v: Value = serde_json::from_str(&output).unwrap();

        let tools_array = match fmt {
            ToolSchemaFormat::Openai => v["functions"].as_array(),
            ToolSchemaFormat::Anthropic => v["tools"].as_array(),
            ToolSchemaFormat::Jsonschema => v["tools"].as_array(),
            ToolSchemaFormat::Clap => v["tools"].as_array(),
        };
        let tools_array = tools_array.unwrap_or_else(|| panic!("no tools array in {fmt:?} output"));

        let names: Vec<&str> = tools_array
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        for name in &expected {
            assert!(
                names.contains(name),
                "missing tool {name} in {fmt:?} rendered output"
            );
        }
    }
}

// ── 3. Determinism ──────────────────────────────────────────────────────

#[test]
fn schema_generation_is_deterministic() {
    let schema_a = build_tool_schema(&rich_cmd());
    let schema_b = build_tool_schema(&rich_cmd());

    for fmt in ALL_FORMATS {
        let output_a = render_output(&schema_a, fmt, false).unwrap();
        let output_b = render_output(&schema_b, fmt, false).unwrap();
        assert_eq!(output_a, output_b, "non-deterministic output for {fmt:?}");
    }
}

#[test]
fn repeated_rendering_is_byte_identical() {
    let schema = build_tool_schema(&rich_cmd());

    for fmt in ALL_FORMATS {
        let first = render_output(&schema, fmt, true).unwrap();
        let second = render_output(&schema, fmt, true).unwrap();
        assert_eq!(first, second, "rendering not stable for {fmt:?}");
    }
}

// ── 4. Round-trip deserialization ────────────────────────────────────────

#[test]
fn clap_format_round_trips_through_json() {
    let schema = build_tool_schema(&rich_cmd());
    let json_str = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
    let deserialized: ToolSchemaOutput = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.name, schema.name);
    assert_eq!(deserialized.version, schema.version);
    assert_eq!(deserialized.description, schema.description);
    assert_eq!(deserialized.schema_version, schema.schema_version);
    assert_eq!(deserialized.tools.len(), schema.tools.len());

    for (orig, deser) in schema.tools.iter().zip(deserialized.tools.iter()) {
        assert_eq!(orig.name, deser.name);
        assert_eq!(orig.description, deser.description);
        assert_eq!(orig.parameters.len(), deser.parameters.len());
        for (op, dp) in orig.parameters.iter().zip(deser.parameters.iter()) {
            assert_eq!(op.name, dp.name);
            assert_eq!(op.param_type, dp.param_type);
            assert_eq!(op.required, dp.required);
            assert_eq!(op.default, dp.default);
            assert_eq!(op.enum_values, dp.enum_values);
            assert_eq!(op.description, dp.description);
        }
    }
}

#[test]
fn all_formats_round_trip_through_serde_value() {
    let schema = build_tool_schema(&rich_cmd());

    for fmt in ALL_FORMATS {
        let json_str = render_output(&schema, fmt, false).unwrap();
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        let re_serialized = serde_json::to_string(&parsed).unwrap();
        let re_parsed: Value = serde_json::from_str(&re_serialized).unwrap();
        assert_eq!(parsed, re_parsed, "Value round-trip failed for {fmt:?}");
    }
}

// ── 5. Every tool has name, description, and parameters ─────────────────

fn assert_tool_structure(tools: &[Value], format_name: &str) {
    assert!(!tools.is_empty(), "{format_name}: tools array is empty");
    for tool in tools {
        assert!(
            tool["name"].is_string(),
            "{format_name}: tool missing 'name'"
        );
        assert!(
            tool["description"].is_string(),
            "{format_name}: tool '{}' missing 'description'",
            tool["name"]
        );
    }
}

#[test]
fn every_tool_has_name_description_parameters_openai() {
    let schema = build_tool_schema(&rich_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&output).unwrap();
    let tools = v["functions"].as_array().unwrap();
    assert_tool_structure(tools, "OpenAI");

    for tool in tools {
        assert!(
            tool["parameters"].is_object(),
            "OpenAI: tool '{}' missing 'parameters'",
            tool["name"]
        );
        assert_eq!(
            tool["parameters"]["type"].as_str(),
            Some("object"),
            "OpenAI: parameters should be type 'object'"
        );
        assert!(
            tool["parameters"]["properties"].is_object(),
            "OpenAI: tool '{}' missing 'parameters.properties'",
            tool["name"]
        );
        assert!(
            tool["parameters"]["required"].is_array(),
            "OpenAI: tool '{}' missing 'parameters.required'",
            tool["name"]
        );
    }
}

#[test]
fn every_tool_has_name_description_parameters_anthropic() {
    let schema = build_tool_schema(&rich_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&output).unwrap();
    let tools = v["tools"].as_array().unwrap();
    assert_tool_structure(tools, "Anthropic");

    for tool in tools {
        assert!(
            tool["input_schema"].is_object(),
            "Anthropic: tool '{}' missing 'input_schema'",
            tool["name"]
        );
        assert_eq!(
            tool["input_schema"]["type"].as_str(),
            Some("object"),
            "Anthropic: input_schema should be type 'object'"
        );
        assert!(
            tool["input_schema"]["properties"].is_object(),
            "Anthropic: tool '{}' missing 'input_schema.properties'",
            tool["name"]
        );
        assert!(
            tool["input_schema"]["required"].is_array(),
            "Anthropic: tool '{}' missing 'input_schema.required'",
            tool["name"]
        );
    }
}

#[test]
fn every_tool_has_name_description_parameters_jsonschema() {
    let schema = build_tool_schema(&rich_cmd());
    let output = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&output).unwrap();
    let tools = v["tools"].as_array().unwrap();
    assert_tool_structure(tools, "JSON Schema");

    for tool in tools {
        assert!(
            tool["parameters"].is_object(),
            "JSON Schema: tool '{}' missing 'parameters'",
            tool["name"]
        );
        assert_eq!(
            tool["parameters"]["type"].as_str(),
            Some("object"),
            "JSON Schema: parameters should be type 'object'"
        );
    }
}

#[test]
fn every_tool_definition_struct_has_name_and_description() {
    let schema = build_tool_schema(&rich_cmd());
    for tool in &schema.tools {
        assert!(!tool.name.is_empty(), "tool name is empty");
        // description may be empty but must exist
        let _ = &tool.description;
    }
}

// ── 6. Snapshot tests (insta) ───────────────────────────────────────────

#[test]
fn snapshot_rich_cmd_openai() {
    let schema = build_tool_schema(&rich_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn snapshot_rich_cmd_anthropic() {
    let schema = build_tool_schema(&rich_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Anthropic, true).unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn snapshot_rich_cmd_jsonschema() {
    let schema = build_tool_schema(&rich_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    insta::assert_snapshot!(rendered);
}

#[test]
fn snapshot_rich_cmd_clap() {
    let schema = build_tool_schema(&rich_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Clap, true).unwrap();
    insta::assert_snapshot!(rendered);
}

// ── proptest: arbitrary commands ─────────────────────────────────────────

fn arb_action() -> impl Strategy<Value = ArgAction> {
    prop_oneof![
        Just(ArgAction::SetTrue),
        Just(ArgAction::SetFalse),
        Just(ArgAction::Count),
        Just(ArgAction::Append),
        Just(ArgAction::Set),
    ]
}

fn arb_arg() -> impl Strategy<Value = (String, ArgAction, bool)> {
    ("[a-z][a-z0-9_]{0,11}", arb_action(), any::<bool>())
}

fn arb_subcmd() -> impl Strategy<Value = (String, Vec<(String, ArgAction, bool)>)> {
    ("[a-z][a-z0-9]{0,7}", prop::collection::vec(arb_arg(), 0..5))
}

fn build_cmd_from_parts(
    name: &str,
    subcmds: &[(String, Vec<(String, ArgAction, bool)>)],
) -> Command {
    let mut cmd = Command::new(name.to_string())
        .version("1.0.0")
        .about("Generated CLI");
    for (sc_name, args) in subcmds {
        let mut sc = Command::new(sc_name.clone()).about("A subcommand");
        // Deduplicate arg names within this subcommand
        let mut seen = std::collections::BTreeSet::new();
        for (arg_name, action, required) in args {
            if !seen.insert(arg_name.clone()) {
                continue;
            }
            let mut arg = Arg::new(arg_name.clone())
                .long(arg_name.clone())
                .action(action.clone());
            // Only Set/Append can be required; bool/count are flags
            if *required && matches!(action, ArgAction::Set | ArgAction::Append) {
                arg = arg.required(true);
            }
            sc = sc.arg(arg);
        }
        cmd = cmd.subcommand(sc);
    }
    cmd
}

proptest! {
    #[test]
    fn prop_schema_always_valid_json(
        subcmds in prop::collection::vec(arb_subcmd(), 0..4)
    ) {
        let cmd = build_cmd_from_parts("testcli", &subcmds);
        let schema = build_tool_schema(&cmd);

        for fmt in &ALL_FORMATS {
            let output = render_output(&schema, *fmt, false).unwrap();
            let _: Value = serde_json::from_str(&output)
                .unwrap_or_else(|e| panic!("invalid JSON for {fmt:?}: {e}"));
        }
    }

    #[test]
    fn prop_schema_is_deterministic(
        subcmds in prop::collection::vec(arb_subcmd(), 0..3)
    ) {
        let cmd_a = build_cmd_from_parts("testcli", &subcmds);
        let cmd_b = build_cmd_from_parts("testcli", &subcmds);
        let schema_a = build_tool_schema(&cmd_a);
        let schema_b = build_tool_schema(&cmd_b);

        for fmt in &ALL_FORMATS {
            let out_a = render_output(&schema_a, *fmt, false).unwrap();
            let out_b = render_output(&schema_b, *fmt, false).unwrap();
            prop_assert_eq!(&out_a, &out_b, "non-deterministic for {:?}", fmt);
        }
    }

    #[test]
    fn prop_tool_count_equals_one_plus_subcommands(
        subcmds in prop::collection::vec(arb_subcmd(), 0..4)
    ) {
        let cmd = build_cmd_from_parts("testcli", &subcmds);
        let schema = build_tool_schema(&cmd);
        // Unique subcommand names (clap deduplicates)
        let unique_subcmd_names: std::collections::BTreeSet<_> =
            subcmds.iter().map(|(n, _)| n.as_str()).collect();
        // root + unique subcommands
        prop_assert_eq!(schema.tools.len(), 1 + unique_subcmd_names.len());
    }

    #[test]
    fn prop_every_tool_has_non_empty_name(
        subcmds in prop::collection::vec(arb_subcmd(), 0..4)
    ) {
        let cmd = build_cmd_from_parts("testcli", &subcmds);
        let schema = build_tool_schema(&cmd);
        for tool in &schema.tools {
            prop_assert!(!tool.name.is_empty());
        }
    }

    #[test]
    fn prop_clap_format_round_trips(
        subcmds in prop::collection::vec(arb_subcmd(), 0..3)
    ) {
        let cmd = build_cmd_from_parts("testcli", &subcmds);
        let schema = build_tool_schema(&cmd);
        let json_str = render_output(&schema, ToolSchemaFormat::Clap, false).unwrap();
        let deser: ToolSchemaOutput = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(schema.tools.len(), deser.tools.len());
        prop_assert_eq!(&schema.name, &deser.name);
    }

    #[test]
    fn prop_param_types_are_valid(
        subcmds in prop::collection::vec(arb_subcmd(), 0..4)
    ) {
        let cmd = build_cmd_from_parts("testcli", &subcmds);
        let schema = build_tool_schema(&cmd);
        let valid_types = ["string", "boolean", "integer", "array"];
        for tool in &schema.tools {
            for param in &tool.parameters {
                prop_assert!(
                    valid_types.contains(&param.param_type.as_str()),
                    "unexpected param type: {}",
                    param.param_type
                );
            }
        }
    }
}
