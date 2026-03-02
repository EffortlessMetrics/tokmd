//! Documentation synchronization tests.
//!
//! These tests verify that documentation files stay in sync with code:
//! - docs/schema.json is valid JSON Schema Draft 7
//! - docs/SCHEMA.md version table matches code constants
//! - docs/reference-cli.md covers all CLI subcommands
//! - schema.json `const` values match Rust schema version constants

use std::path::PathBuf;

use serde_json::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn schema_json_is_valid_json_schema_draft7() {
    let schema_path = workspace_root().join("docs").join("schema.json");
    let content = std::fs::read_to_string(&schema_path).expect("docs/schema.json should exist");
    let schema: Value =
        serde_json::from_str(&content).expect("docs/schema.json should be valid JSON");
    assert_eq!(
        schema["$schema"].as_str().unwrap(),
        "http://json-schema.org/draft-07/schema#",
        "docs/schema.json should declare JSON Schema Draft 7"
    );
    assert!(
        schema["title"].is_string(),
        "docs/schema.json should have a title"
    );
    assert!(
        schema["definitions"].is_object(),
        "docs/schema.json should have a definitions section"
    );
    jsonschema::validator_for(&schema)
        .expect("docs/schema.json should compile as a valid JSON Schema");
}

#[test]
fn schema_json_contains_all_receipt_definitions() {
    let schema_path = workspace_root().join("docs").join("schema.json");
    let content = std::fs::read_to_string(&schema_path).unwrap();
    let schema: Value = serde_json::from_str(&content).unwrap();
    let defs = schema["definitions"].as_object().unwrap();
    let required = [
        "LangReceipt",
        "ModuleReceipt",
        "ExportReceipt",
        "AnalysisReceipt",
        "CockpitReceipt",
    ];
    for name in &required {
        assert!(
            defs.contains_key(*name),
            "docs/schema.json missing definition for {name}"
        );
    }
}

#[test]
fn schema_json_version_consts_match_code_constants() {
    let schema_path = workspace_root().join("docs").join("schema.json");
    let content = std::fs::read_to_string(&schema_path).unwrap();
    let schema: Value = serde_json::from_str(&content).unwrap();
    let cases: &[(&str, u32)] = &[
        ("LangReceipt", tokmd_types::SCHEMA_VERSION),
        ("ModuleReceipt", tokmd_types::SCHEMA_VERSION),
        ("ExportReceipt", tokmd_types::SCHEMA_VERSION),
        (
            "AnalysisReceipt",
            tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        ),
        (
            "CockpitReceipt",
            tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION,
        ),
    ];
    for (def_name, expected) in cases {
        let const_val = &schema["definitions"][def_name]["properties"]["schema_version"]["const"];
        assert_eq!(
            const_val.as_u64().unwrap_or(0),
            *expected as u64,
            "schema.json {def_name}.schema_version const ({}) != code constant ({expected})",
            const_val
        );
    }
}

#[test]
fn schema_md_version_table_matches_code_constants() {
    let md_path = workspace_root().join("docs").join("SCHEMA.md");
    let content = std::fs::read_to_string(&md_path).expect("docs/SCHEMA.md should exist");
    let expected: &[(&str, u32)] = &[
        ("SCHEMA_VERSION", tokmd_types::SCHEMA_VERSION),
        (
            "CONTEXT_SCHEMA_VERSION",
            tokmd_types::CONTEXT_SCHEMA_VERSION,
        ),
        (
            "CONTEXT_BUNDLE_SCHEMA_VERSION",
            tokmd_types::CONTEXT_BUNDLE_SCHEMA_VERSION,
        ),
        (
            "ANALYSIS_SCHEMA_VERSION",
            tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        ),
        (
            "COCKPIT_SCHEMA_VERSION",
            tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION,
        ),
        (
            "HANDOFF_SCHEMA_VERSION",
            tokmd_types::HANDOFF_SCHEMA_VERSION,
        ),
    ];
    for (constant_name, code_value) in expected {
        let pattern = format!("`{constant_name}`");
        let row = content
            .lines()
            .find(|line| line.contains(&pattern))
            .unwrap_or_else(|| panic!("docs/SCHEMA.md missing row for {constant_name}"));
        let cols: Vec<&str> = row.split('|').map(str::trim).collect();
        let doc_version: u32 = cols[2].trim().parse().unwrap_or_else(|_| {
            panic!(
                "Cannot parse version from SCHEMA.md for {constant_name}: '{}'",
                cols[2]
            )
        });
        assert_eq!(
            doc_version, *code_value,
            "docs/SCHEMA.md says {constant_name} = {doc_version}, but code says {code_value}"
        );
    }
}

#[test]
fn reference_cli_has_sections_for_all_subcommands() {
    let ref_path = workspace_root().join("docs").join("reference-cli.md");
    let content = std::fs::read_to_string(&ref_path).expect("docs/reference-cli.md should exist");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("--help")
        .output()
        .expect("tokmd --help should succeed");
    let help = String::from_utf8_lossy(&output.stdout);
    let mut subcommands = Vec::new();
    let mut in_commands = false;
    for line in help.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Commands:") {
            in_commands = true;
            continue;
        }
        if in_commands {
            if trimmed.is_empty()
                || trimmed.starts_with("Arguments:")
                || trimmed.starts_with("Options:")
            {
                break;
            }
            if let Some(name) = trimmed.split_whitespace().next() {
                if name != "help" {
                    subcommands.push(name.to_string());
                }
            }
        }
    }
    assert!(
        !subcommands.is_empty(),
        "Failed to parse subcommands from --help output"
    );
    for cmd in &subcommands {
        let section_pattern = format!("tokmd {cmd}");
        let default_pattern = "tokmd` (Default";
        let found = content.contains(&section_pattern)
            || (cmd == "lang" && content.contains(default_pattern));
        assert!(
            found,
            "docs/reference-cli.md missing section for subcommand `tokmd {cmd}`"
        );
    }
}

#[test]
fn reference_cli_help_blocks_reference_real_subcommands() {
    let ref_path = workspace_root().join("docs").join("reference-cli.md");
    let content = std::fs::read_to_string(&ref_path).unwrap();
    let mut help_blocks: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("<!-- HELP:") {
            if let Some(name) = rest.strip_suffix("-->") {
                help_blocks.push(name.trim().to_string());
            }
        }
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("--help")
        .output()
        .expect("tokmd --help should succeed");
    let help = String::from_utf8_lossy(&output.stdout);
    for block_name in &help_blocks {
        assert!(
            help.contains(block_name),
            "reference-cli.md has HELP block for '{block_name}' which is not a known subcommand"
        );
    }
}
