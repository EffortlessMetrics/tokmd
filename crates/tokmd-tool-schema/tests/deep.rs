//! Deep tests for `tokmd-tool-schema`.
//!
//! Focuses on cross-format structural contracts, Unicode handling,
//! edge-case parameter combinations, serialization fidelity, and
//! semantic correctness of the generated AI tool schemas.

use clap::{Arg, ArgAction, Command};
use serde_json::Value;
use tokmd_tool_schema::{
    ParameterSchema, TOOL_SCHEMA_VERSION, ToolDefinition, ToolSchemaFormat, ToolSchemaOutput,
    build_tool_schema, render_output,
};

// ── helpers ──────────────────────────────────────────────────────────────

/// A CLI command with Unicode descriptions and argument help text.
fn unicode_cmd() -> Command {
    Command::new("uni")
        .version("1.0.0")
        .about("Outil d'analyse de dépôts — 日本語テスト 🚀")
        .subcommand(
            Command::new("scän")
                .about("Scän les fichiers — スキャン")
                .arg(
                    Arg::new("chemin")
                        .long("chemin")
                        .required(true)
                        .help("Le chemin d'accès — パス指定 📂"),
                ),
        )
}

/// Command with every ArgAction variant for exhaustive type mapping.
fn all_arg_actions_cmd() -> Command {
    Command::new("actions")
        .version("0.1.0")
        .about("All arg actions")
        .subcommand(
            Command::new("run")
                .about("Run task")
                .arg(
                    Arg::new("set-true")
                        .long("set-true")
                        .action(ArgAction::SetTrue)
                        .help("A boolean (SetTrue)"),
                )
                .arg(
                    Arg::new("set-false")
                        .long("set-false")
                        .action(ArgAction::SetFalse)
                        .help("A boolean (SetFalse)"),
                )
                .arg(
                    Arg::new("count")
                        .short('v')
                        .long("verbose")
                        .action(ArgAction::Count)
                        .help("Verbosity counter"),
                )
                .arg(
                    Arg::new("append")
                        .long("append")
                        .action(ArgAction::Append)
                        .help("Repeated values"),
                )
                .arg(
                    Arg::new("plain")
                        .long("plain")
                        .action(ArgAction::Set)
                        .help("Plain string value"),
                ),
        )
}

/// Command with many subcommands to stress tool listing.
fn many_subcommands_cmd() -> Command {
    let mut cmd = Command::new("fleet").version("3.0.0").about("Fleet CLI");
    // Use leaked strings since clap's Command::new requires 'static str.
    for i in 0..20 {
        let name: &'static str = Box::leak(format!("cmd-{i}").into_boxed_str());
        let about: &'static str = Box::leak(format!("Subcommand number {i}").into_boxed_str());
        let help: &'static str = Box::leak(format!("Input for cmd-{i}").into_boxed_str());
        cmd = cmd.subcommand(
            Command::new(name).about(about).arg(
                Arg::new("input")
                    .long("input")
                    .required(i % 2 == 0)
                    .help(help),
            ),
        );
    }
    cmd
}

/// Command with a mix of required and optional params and defaults.
fn mixed_params_cmd() -> Command {
    Command::new("mix")
        .version("2.0.0")
        .about("Mixed params")
        .subcommand(
            Command::new("deploy")
                .about("Deploy service")
                .arg(
                    Arg::new("target")
                        .long("target")
                        .required(true)
                        .help("Deploy target"),
                )
                .arg(
                    Arg::new("region")
                        .long("region")
                        .default_value("us-east-1")
                        .help("AWS region"),
                )
                .arg(
                    Arg::new("env")
                        .long("env")
                        .value_parser(["dev", "staging", "prod"])
                        .default_value("dev")
                        .help("Environment"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Dry run"),
                )
                .arg(Arg::new("tags").long("tags").action(ArgAction::Append)),
        )
}

/// Minimal command with no version, no about, no args.
fn bare_cmd() -> Command {
    Command::new("bare")
}

/// Command with a single positional argument.
fn positional_cmd() -> Command {
    Command::new("pos")
        .version("1.0.0")
        .about("Positional test")
        .subcommand(
            Command::new("read")
                .about("Read a file")
                .arg(Arg::new("file").required(true).help("File to read")),
        )
}

// ── 1. OpenAI format: structural contracts ──────────────────────────────

#[test]
fn openai_functions_array_is_not_empty_for_nonempty_cmd() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let fns = v["functions"].as_array().unwrap();
    assert!(fns.len() >= 2, "should have root + deploy");
}

#[test]
fn openai_each_function_has_name_description_parameters() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    for func in v["functions"].as_array().unwrap() {
        assert!(func.get("name").and_then(|n| n.as_str()).is_some());
        assert!(func.get("description").is_some());
        assert!(func.get("parameters").is_some());
        assert_eq!(func["parameters"]["type"], "object");
        assert!(func["parameters"].get("properties").is_some());
        assert!(func["parameters"].get("required").is_some());
    }
}

#[test]
fn openai_has_no_tools_or_input_schema_keys() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("tools").is_none());
    for func in v["functions"].as_array().unwrap() {
        assert!(func.get("input_schema").is_none());
    }
}

// ── 2. Anthropic format: structural contracts ───────────────────────────

#[test]
fn anthropic_tools_array_is_not_empty() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let tools = v["tools"].as_array().unwrap();
    assert!(tools.len() >= 2);
}

#[test]
fn anthropic_each_tool_has_input_schema_not_parameters() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    for tool in v["tools"].as_array().unwrap() {
        assert!(tool.get("name").and_then(|n| n.as_str()).is_some());
        assert!(tool.get("description").is_some());
        assert!(tool.get("input_schema").is_some());
        assert_eq!(tool["input_schema"]["type"], "object");
        // Anthropic tools should not have a bare "parameters" key
        assert!(tool.get("parameters").is_none());
    }
}

#[test]
fn anthropic_has_no_functions_key() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("functions").is_none());
}

// ── 3. JSON Schema format: structural contracts ─────────────────────────

#[test]
fn jsonschema_has_draft_reference_and_envelope() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["$schema"].as_str().unwrap(),
        "https://json-schema.org/draft-07/schema#"
    );
    assert!(v.get("schema_version").is_some());
    assert!(v.get("name").is_some());
    assert!(v.get("version").is_some());
    assert!(v.get("description").is_some());
    assert!(v.get("tools").is_some());
}

#[test]
fn jsonschema_tools_have_parameters_with_type_object() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    for tool in v["tools"].as_array().unwrap() {
        assert_eq!(tool["parameters"]["type"], "object");
    }
}

#[test]
fn jsonschema_includes_default_values_in_properties() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let deploy = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "deploy")
        .unwrap();
    let region = &deploy["parameters"]["properties"]["region"];
    assert_eq!(region["default"].as_str(), Some("us-east-1"));
}

// ── 4. Schema is valid JSON (all formats) ───────────────────────────────

#[test]
fn all_formats_produce_valid_json_for_complex_cmd() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let json_str = render_output(&schema, fmt, false).unwrap();
        serde_json::from_str::<Value>(&json_str)
            .unwrap_or_else(|e| panic!("invalid JSON for {fmt:?}: {e}"));
    }
}

#[test]
fn all_formats_produce_valid_json_for_bare_cmd() {
    let schema = build_tool_schema(&bare_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let json_str = render_output(&schema, fmt, false).unwrap();
        serde_json::from_str::<Value>(&json_str)
            .unwrap_or_else(|e| panic!("invalid JSON for {fmt:?} on bare cmd: {e}"));
    }
}

// ── 5. Required fields present in every tool ────────────────────────────

#[test]
fn every_tool_definition_has_name_and_description() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    for tool in &schema.tools {
        assert!(!tool.name.is_empty(), "tool name must not be empty");
        // description can be empty but field must exist (it's a String)
        let _ = &tool.description;
    }
}

// ── 6. Parameter types are correct ──────────────────────────────────────

#[test]
fn set_true_yields_boolean_type() {
    let schema = build_tool_schema(&all_arg_actions_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let p = run
        .parameters
        .iter()
        .find(|p| p.name == "set-true")
        .unwrap();
    assert_eq!(p.param_type, "boolean");
}

#[test]
fn set_false_yields_boolean_type() {
    let schema = build_tool_schema(&all_arg_actions_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let p = run
        .parameters
        .iter()
        .find(|p| p.name == "set-false")
        .unwrap();
    assert_eq!(p.param_type, "boolean");
}

#[test]
fn count_yields_integer_type() {
    let schema = build_tool_schema(&all_arg_actions_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let p = run.parameters.iter().find(|p| p.name == "count").unwrap();
    assert_eq!(p.param_type, "integer");
}

#[test]
fn append_yields_array_type() {
    let schema = build_tool_schema(&all_arg_actions_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let p = run.parameters.iter().find(|p| p.name == "append").unwrap();
    assert_eq!(p.param_type, "array");
}

#[test]
fn set_yields_string_type() {
    let schema = build_tool_schema(&all_arg_actions_cmd());
    let run = schema.tools.iter().find(|t| t.name == "run").unwrap();
    let p = run.parameters.iter().find(|p| p.name == "plain").unwrap();
    assert_eq!(p.param_type, "string");
}

// ── 7. All CLI subcommands produce schemas ──────────────────────────────

#[test]
fn twenty_subcommands_all_appear_in_tools() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    // root + 20 subcommands
    assert_eq!(schema.tools.len(), 21);
    for i in 0..20 {
        let name = format!("cmd-{i}");
        assert!(
            schema.tools.iter().any(|t| t.name == name),
            "missing subcommand {name}"
        );
    }
}

#[test]
fn subcommands_with_alternating_required_are_correct() {
    let schema = build_tool_schema(&many_subcommands_cmd());
    for i in 0..20 {
        let name = format!("cmd-{i}");
        let tool = schema.tools.iter().find(|t| t.name == name).unwrap();
        let input = tool.parameters.iter().find(|p| p.name == "input").unwrap();
        let expect_required = i % 2 == 0;
        assert_eq!(
            input.required, expect_required,
            "cmd-{i} input.required should be {expect_required}"
        );
    }
}

// ── 8. Serialization roundtrip ──────────────────────────────────────────

#[test]
fn clap_format_roundtrip_preserves_all_fields() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Clap, true).unwrap();
    let rt: ToolSchemaOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(rt.schema_version, TOOL_SCHEMA_VERSION);
    assert_eq!(rt.name, "mix");
    assert_eq!(rt.version, "2.0.0");
    assert_eq!(rt.description, "Mixed params");
    assert_eq!(rt.tools.len(), schema.tools.len());

    let deploy = rt.tools.iter().find(|t| t.name == "deploy").unwrap();
    assert_eq!(deploy.parameters.len(), 5);

    let env_p = deploy.parameters.iter().find(|p| p.name == "env").unwrap();
    assert_eq!(env_p.default.as_deref(), Some("dev"));
    assert_eq!(
        env_p.enum_values.as_ref().unwrap(),
        &["dev", "staging", "prod"]
    );
}

#[test]
fn tool_definition_roundtrip_via_serde() {
    let original = ToolDefinition {
        name: "test-tool".into(),
        description: "A tool for testing".into(),
        parameters: vec![
            ParameterSchema {
                name: "input".into(),
                description: Some("The input path".into()),
                param_type: "string".into(),
                required: true,
                default: None,
                enum_values: None,
            },
            ParameterSchema {
                name: "mode".into(),
                description: None,
                param_type: "string".into(),
                required: false,
                default: Some("fast".into()),
                enum_values: Some(vec!["fast".into(), "slow".into()]),
            },
        ],
    };
    let json = serde_json::to_string(&original).unwrap();
    let rt: ToolDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.name, original.name);
    assert_eq!(rt.parameters.len(), 2);
    assert_eq!(rt.parameters[1].default.as_deref(), Some("fast"));
}

#[test]
fn parameter_schema_skip_serializing_none_fields() {
    let p = ParameterSchema {
        name: "x".into(),
        description: None,
        param_type: "string".into(),
        required: false,
        default: None,
        enum_values: None,
    };
    let json = serde_json::to_string(&p).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(
        v.get("description").is_none(),
        "None description should be skipped"
    );
    assert!(v.get("default").is_none(), "None default should be skipped");
    assert!(
        v.get("enum_values").is_none(),
        "None enum_values should be skipped"
    );
}

// ── 9. Deterministic output ─────────────────────────────────────────────

#[test]
fn deterministic_across_multiple_builds_and_formats() {
    for _ in 0..5 {
        let cmd = mixed_params_cmd();
        let schema = build_tool_schema(&cmd);
        let a = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
        let cmd2 = mixed_params_cmd();
        let schema2 = build_tool_schema(&cmd2);
        let b = render_output(&schema2, ToolSchemaFormat::Openai, false).unwrap();
        assert_eq!(a, b, "OpenAI output must be deterministic across builds");
    }
}

#[test]
fn deterministic_pretty_vs_compact_parse_equivalence() {
    let schema = build_tool_schema(&mixed_params_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let pretty = render_output(&schema, fmt, true).unwrap();
        let compact = render_output(&schema, fmt, false).unwrap();
        let pretty_v: Value = serde_json::from_str(&pretty).unwrap();
        let compact_v: Value = serde_json::from_str(&compact).unwrap();
        assert_eq!(
            pretty_v, compact_v,
            "pretty vs compact must parse equal for {fmt:?}"
        );
    }
}

// ── 10. Unicode in descriptions ─────────────────────────────────────────

#[test]
fn unicode_in_about_is_preserved_in_schema() {
    let schema = build_tool_schema(&unicode_cmd());
    assert!(schema.description.contains("日本語テスト"));
    assert!(schema.description.contains("🚀"));
}

#[test]
fn unicode_in_subcommand_about_is_preserved() {
    let schema = build_tool_schema(&unicode_cmd());
    let sub = schema.tools.iter().find(|t| t.name == "scän").unwrap();
    assert!(sub.description.contains("スキャン"));
}

#[test]
fn unicode_in_arg_help_is_preserved() {
    let schema = build_tool_schema(&unicode_cmd());
    let sub = schema.tools.iter().find(|t| t.name == "scän").unwrap();
    let p = sub.parameters.iter().find(|p| p.name == "chemin").unwrap();
    let desc = p.description.as_ref().unwrap();
    assert!(desc.contains("パス指定"));
    assert!(desc.contains("📂"));
}

#[test]
fn unicode_survives_json_roundtrip_in_all_formats() {
    let schema = build_tool_schema(&unicode_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let json = render_output(&schema, fmt, false).unwrap();
        // Verify it's valid JSON and contains the Unicode content
        let v: Value = serde_json::from_str(&json).unwrap();
        let json_text = serde_json::to_string(&v).unwrap();
        // The emoji should survive (may be escaped or literal — both valid)
        assert!(
            json_text.contains("🚀") || json_text.contains("\\ud83d\\ude80"),
            "emoji should survive JSON roundtrip for {fmt:?}"
        );
    }
}

// ── 11. Optional vs required parameters ─────────────────────────────────

#[test]
fn required_param_appears_in_openai_required_array() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let deploy = v["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "deploy")
        .unwrap();
    let required: Vec<&str> = deploy["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(required.contains(&"target"), "target is required");
    assert!(
        !required.contains(&"region"),
        "region has default, not required"
    );
    assert!(
        !required.contains(&"dry-run"),
        "dry-run is a flag, not required"
    );
    assert!(!required.contains(&"tags"), "tags is optional");
}

#[test]
fn required_param_appears_in_anthropic_required_array() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let deploy = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "deploy")
        .unwrap();
    let required: Vec<&str> = deploy["input_schema"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(required.contains(&"target"));
    assert!(!required.contains(&"env"));
}

#[test]
fn required_param_appears_in_jsonschema_required_array() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    let deploy = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "deploy")
        .unwrap();
    let required: Vec<&str> = deploy["parameters"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(required.contains(&"target"));
    assert!(!required.contains(&"dry-run"));
}

// ── Cross-format consistency ────────────────────────────────────────────

#[test]
fn same_tool_names_across_all_formats() {
    let schema = build_tool_schema(&mixed_params_cmd());

    let openai_json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let anthropic_json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let jsonschema_json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();

    let ov: Value = serde_json::from_str(&openai_json).unwrap();
    let av: Value = serde_json::from_str(&anthropic_json).unwrap();
    let jv: Value = serde_json::from_str(&jsonschema_json).unwrap();

    let openai_names: Vec<&str> = ov["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();

    let anthropic_names: Vec<&str> = av["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    let jsonschema_names: Vec<&str> = jv["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    assert_eq!(openai_names, anthropic_names);
    assert_eq!(openai_names, jsonschema_names);
}

#[test]
fn same_required_params_across_all_formats() {
    let schema = build_tool_schema(&mixed_params_cmd());

    let openai_json = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    let anthropic_json = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();
    let jsonschema_json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();

    let ov: Value = serde_json::from_str(&openai_json).unwrap();
    let av: Value = serde_json::from_str(&anthropic_json).unwrap();
    let jv: Value = serde_json::from_str(&jsonschema_json).unwrap();

    // Extract required arrays for deploy from each format
    let get_required_for = |val: &Value, tools_key: &str, params_key: &str| -> Vec<String> {
        let tool = val[tools_key]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == "deploy")
            .unwrap();
        tool[params_key]["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };

    let openai_req = get_required_for(&ov, "functions", "parameters");
    let anthropic_req = get_required_for(&av, "tools", "input_schema");
    let jsonschema_req = get_required_for(&jv, "tools", "parameters");

    assert_eq!(openai_req, anthropic_req);
    assert_eq!(openai_req, jsonschema_req);
}

// ── Schema version constant ─────────────────────────────────────────────

#[test]
fn schema_version_constant_embedded_in_jsonschema_output() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let json = render_output(&schema, ToolSchemaFormat::Jsonschema, false).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["schema_version"].as_u64().unwrap(),
        u64::from(TOOL_SCHEMA_VERSION)
    );
}

#[test]
fn schema_version_constant_is_positive() {
    const {
        assert!(TOOL_SCHEMA_VERSION > 0);
    }
}

// ── Edge cases ──────────────────────────────────────────────────────────

#[test]
fn bare_cmd_without_version_or_about() {
    let schema = build_tool_schema(&bare_cmd());
    assert_eq!(schema.name, "bare");
    assert_eq!(schema.version, "unknown");
    assert!(schema.description.is_empty());
    assert_eq!(schema.tools.len(), 1);
    assert!(schema.tools[0].parameters.is_empty());
}

#[test]
fn help_subcommand_is_excluded() {
    // clap auto-adds a "help" subcommand; verify it's filtered out
    let cmd = Command::new("app")
        .version("1.0.0")
        .subcommand(Command::new("real").about("Real command"));
    let schema = build_tool_schema(&cmd);
    assert!(
        !schema.tools.iter().any(|t| t.name == "help"),
        "help subcommand should be excluded"
    );
}

#[test]
fn help_and_version_args_are_excluded() {
    let cmd = Command::new("app")
        .version("1.0.0")
        .about("App")
        .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue));
    let schema = build_tool_schema(&cmd);
    let root = &schema.tools[0];
    assert!(
        !root
            .parameters
            .iter()
            .any(|p| p.name == "help" || p.name == "version"),
        "help and version args should be excluded"
    );
    assert_eq!(root.parameters.len(), 1);
    assert_eq!(root.parameters[0].name, "flag");
}

#[test]
fn positional_argument_is_included() {
    let schema = build_tool_schema(&positional_cmd());
    let read = schema.tools.iter().find(|t| t.name == "read").unwrap();
    assert_eq!(read.parameters.len(), 1);
    assert_eq!(read.parameters[0].name, "file");
    assert!(read.parameters[0].required);
}

#[test]
fn enum_values_from_value_parser_are_captured() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let deploy = schema.tools.iter().find(|t| t.name == "deploy").unwrap();
    let env_p = deploy.parameters.iter().find(|p| p.name == "env").unwrap();
    let enums = env_p.enum_values.as_ref().unwrap();
    assert_eq!(enums, &["dev", "staging", "prod"]);
}

#[test]
fn param_without_enum_has_none_enum_values() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let deploy = schema.tools.iter().find(|t| t.name == "deploy").unwrap();
    let target = deploy
        .parameters
        .iter()
        .find(|p| p.name == "target")
        .unwrap();
    assert!(target.enum_values.is_none());
}

// ── Format enum traits ──────────────────────────────────────────────────

#[test]
fn tool_schema_format_default_is_jsonschema() {
    assert_eq!(ToolSchemaFormat::default(), ToolSchemaFormat::Jsonschema);
}

#[test]
fn tool_schema_format_clone_and_eq() {
    let a = ToolSchemaFormat::Openai;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(ToolSchemaFormat::Openai, ToolSchemaFormat::Anthropic);
}

#[test]
fn tool_schema_format_debug() {
    let s = format!("{:?}", ToolSchemaFormat::Anthropic);
    assert!(s.contains("Anthropic"));
}

#[test]
fn tool_schema_format_serde_roundtrip() {
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let json = serde_json::to_string(&fmt).unwrap();
        let rt: ToolSchemaFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(fmt, rt);
    }
}

// ── Pretty output ───────────────────────────────────────────────────────

#[test]
fn pretty_output_contains_newlines_and_indentation() {
    let schema = build_tool_schema(&mixed_params_cmd());
    for fmt in [
        ToolSchemaFormat::Openai,
        ToolSchemaFormat::Anthropic,
        ToolSchemaFormat::Jsonschema,
        ToolSchemaFormat::Clap,
    ] {
        let pretty = render_output(&schema, fmt, true).unwrap();
        assert!(
            pretty.contains('\n'),
            "pretty output should have newlines for {fmt:?}"
        );
        assert!(
            pretty.contains("  "),
            "pretty output should have indentation for {fmt:?}"
        );
    }
}

#[test]
fn compact_output_has_no_pretty_indentation() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let compact = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();
    // Compact JSON should be a single line
    assert!(
        !compact.contains('\n'),
        "compact output should be single line"
    );
}

// ── ToolSchemaOutput derive traits ──────────────────────────────────────

#[test]
fn tool_schema_output_clone_produces_equal_json() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let cloned = schema.clone();
    let a = serde_json::to_string(&schema).unwrap();
    let b = serde_json::to_string(&cloned).unwrap();
    assert_eq!(a, b);
}

#[test]
fn tool_schema_output_debug_does_not_panic() {
    let schema = build_tool_schema(&mixed_params_cmd());
    let _ = format!("{:?}", schema);
}
