//! Schema validation tests for tokmd JSON outputs.
//!
//! These tests verify that the actual CLI output conforms to the JSON schema
//! defined in `docs/schema.json`.

use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

/// Load the JSON schema from docs/schema.json
fn load_schema() -> Value {
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs")
        .join("schema.json");

    let schema_content =
        std::fs::read_to_string(&schema_path).expect("Failed to read schema.json");

    serde_json::from_str(&schema_content).expect("Failed to parse schema.json")
}

/// Build a validator for a specific definition in the schema
fn build_validator_for_definition(
    schema: &Value,
    definition: &str,
) -> jsonschema::Validator {
    // Create a schema that references the specific definition
    let ref_schema = serde_json::json!({
        "$ref": format!("#/definitions/{}", definition),
        "definitions": schema["definitions"]
    });

    jsonschema::validator_for(&ref_schema).expect("Failed to compile schema")
}

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    cmd.current_dir(&fixtures);
    cmd
}

#[test]
fn test_lang_receipt_validates_against_schema() {
    let schema = load_schema();
    let validator = build_validator_for_definition(&schema, "LangReceipt");

    let output = tokmd_cmd()
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute tokmd");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON output");

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect();
        panic!(
            "LangReceipt validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap()
        );
    }
}

#[test]
fn test_module_receipt_validates_against_schema() {
    let schema = load_schema();
    let validator = build_validator_for_definition(&schema, "ModuleReceipt");

    let output = tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute tokmd");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON output");

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect();
        panic!(
            "ModuleReceipt validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap()
        );
    }
}

#[test]
fn test_export_receipt_validates_against_schema() {
    let schema = load_schema();
    let validator = build_validator_for_definition(&schema, "ExportReceipt");

    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute tokmd");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON output");

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect();
        panic!(
            "ExportReceipt validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap()
        );
    }
}

#[test]
fn test_export_meta_validates_against_schema() {
    let schema = load_schema();
    let validator = build_validator_for_definition(&schema, "ExportMeta");

    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .output()
        .expect("Failed to execute tokmd");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // The first line of JSONL output is the meta record
    let first_line = stdout.lines().next().expect("No output lines");
    let json: Value = serde_json::from_str(first_line).expect("Failed to parse meta JSON");

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect();
        panic!(
            "ExportMeta validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap()
        );
    }
}

#[test]
fn test_export_row_validates_against_schema() {
    let schema = load_schema();
    let validator = build_validator_for_definition(&schema, "ExportRow");

    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .output()
        .expect("Failed to execute tokmd");

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");

    // Skip the first line (meta) and validate data rows
    for (i, line) in stdout.lines().skip(1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let json: Value = serde_json::from_str(line).expect("Failed to parse row JSON");

        if !validator.is_valid(&json) {
            let error_messages: Vec<String> = validator
                .iter_errors(&json)
                .map(|e| format!("{} at {}", e, e.instance_path))
                .collect();
            panic!(
                "ExportRow validation failed on row {}:\n{}\n\nOutput:\n{}",
                i + 1,
                error_messages.join("\n"),
                serde_json::to_string_pretty(&json).unwrap()
            );
        }
    }
}

#[test]
fn test_schema_version_matches_constant() {
    // Verify that the schema versions in schema.json match SCHEMA_VERSION in code
    let schema = load_schema();

    // Check LangReceipt schema_version const
    let lang_version =
        &schema["definitions"]["LangReceipt"]["properties"]["schema_version"]["const"];
    assert_eq!(
        lang_version
            .as_u64()
            .expect("schema_version should be integer"),
        2,
        "LangReceipt schema_version should be 2"
    );

    // Check ModuleReceipt schema_version const
    let module_version =
        &schema["definitions"]["ModuleReceipt"]["properties"]["schema_version"]["const"];
    assert_eq!(
        module_version
            .as_u64()
            .expect("schema_version should be integer"),
        2,
        "ModuleReceipt schema_version should be 2"
    );

    // Check ExportReceipt schema_version const
    let export_version =
        &schema["definitions"]["ExportReceipt"]["properties"]["schema_version"]["const"];
    assert_eq!(
        export_version
            .as_u64()
            .expect("schema_version should be integer"),
        2,
        "ExportReceipt schema_version should be 2"
    );

    // Check ExportMeta schema_version const
    let meta_version =
        &schema["definitions"]["ExportMeta"]["properties"]["schema_version"]["const"];
    assert_eq!(
        meta_version
            .as_u64()
            .expect("schema_version should be integer"),
        2,
        "ExportMeta schema_version should be 2"
    );
}
