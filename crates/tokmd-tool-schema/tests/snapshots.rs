//! Snapshot tests for tool-schema rendering.
//!
//! Each snapshot locks down the exact JSON output so regressions in shape,
//! key naming, or ordering are caught automatically.

use clap::{Arg, ArgAction, Command};
use tokmd_tool_schema::{ToolSchemaFormat, build_tool_schema, render_output};

// ── fixture ─────────────────────────────────────────────────────────────

/// A representative CLI with mixed parameter types, enums, defaults, and
/// required/optional args across two subcommands.
fn fixture_cmd() -> Command {
    Command::new("example")
        .version("3.5.0")
        .about("Example CLI for snapshot testing")
        .subcommand(
            Command::new("analyze")
                .about("Run analysis on input")
                .arg(
                    Arg::new("input")
                        .long("input")
                        .required(true)
                        .help("Path to the input file"),
                )
                .arg(
                    Arg::new("preset")
                        .long("preset")
                        .value_parser(["quick", "deep", "full"])
                        .default_value("quick")
                        .help("Analysis preset"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Run without side effects"),
                )
                .arg(
                    Arg::new("verbosity")
                        .short('v')
                        .long("verbosity")
                        .action(ArgAction::Count)
                        .help("Increase verbosity"),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .action(ArgAction::Append)
                        .help("Tags to attach"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export results")
                .arg(
                    Arg::new("output")
                        .long("output")
                        .required(true)
                        .help("Output path"),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv"])
                        .default_value("json")
                        .help("Export format"),
                ),
        )
}

// ── OpenAI snapshots ────────────────────────────────────────────────────

#[test]
fn snapshot_openai_format() {
    let schema = build_tool_schema(&fixture_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    insta::assert_snapshot!(rendered);
}

// ── Anthropic snapshots ─────────────────────────────────────────────────

#[test]
fn snapshot_anthropic_format() {
    let schema = build_tool_schema(&fixture_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Anthropic, true).unwrap();
    insta::assert_snapshot!(rendered);
}

// ── JSON Schema snapshots ───────────────────────────────────────────────

#[test]
fn snapshot_jsonschema_format() {
    let schema = build_tool_schema(&fixture_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    insta::assert_snapshot!(rendered);
}

// ── Clap raw snapshots ──────────────────────────────────────────────────

#[test]
fn snapshot_clap_format() {
    let schema = build_tool_schema(&fixture_cmd());
    let rendered = render_output(&schema, ToolSchemaFormat::Clap, true).unwrap();
    insta::assert_snapshot!(rendered);
}

// ── Minimal command snapshot ────────────────────────────────────────────

#[test]
fn snapshot_minimal_cmd_jsonschema() {
    let cmd = Command::new("bare").version("0.0.1").about("Bare CLI");
    let schema = build_tool_schema(&cmd);
    let rendered = render_output(&schema, ToolSchemaFormat::Jsonschema, true).unwrap();
    insta::assert_snapshot!(rendered);
}

// ── Single parameter edge-cases ─────────────────────────────────────────

#[test]
fn snapshot_single_boolean_param_openai() {
    let cmd = Command::new("flag-only")
        .version("1.0.0")
        .about("Only a flag")
        .arg(
            Arg::new("enable")
                .long("enable")
                .action(ArgAction::SetTrue)
                .help("Enable the feature"),
        );
    let schema = build_tool_schema(&cmd);
    let rendered = render_output(&schema, ToolSchemaFormat::Openai, true).unwrap();
    insta::assert_snapshot!(rendered);
}
