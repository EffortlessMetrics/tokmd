//! Schema validation tests for tokmd JSON outputs.
//!
//! These tests verify that the actual CLI output conforms to the JSON schema
//! defined in the embedded schemas. Schemas are embedded at compile time using
//! `include_str!` to ensure tests work in packaged crate environments (e.g., Nix).

use anyhow::{Context, Result};
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;
use tempfile::tempdir;

/// Embedded JSON schema for tokmd receipts (from schemas/schema.json)
const SCHEMA_JSON: &str = include_str!("../schemas/schema.json");

/// Embedded JSON schema for handoff manifests (from schemas/handoff.schema.json)
const HANDOFF_SCHEMA_JSON: &str = include_str!("../schemas/handoff.schema.json");

/// Embedded JSON schema for sensor.report.v1 envelope (from schemas/sensor.report.v1.schema.json)
const SENSOR_REPORT_SCHEMA_JSON: &str = include_str!("../schemas/sensor.report.v1.schema.json");

/// Load the JSON schema from embedded content
fn load_schema() -> Result<Value> {
    serde_json::from_str(SCHEMA_JSON).context("Failed to parse embedded schema.json")
}

/// Load the handoff JSON schema from embedded content
fn load_handoff_schema() -> Result<Value> {
    serde_json::from_str(HANDOFF_SCHEMA_JSON)
        .context("Failed to parse embedded handoff.schema.json")
}

/// Load the sensor.report.v1 JSON schema from embedded content
fn load_sensor_report_schema() -> Result<Value> {
    serde_json::from_str(SENSOR_REPORT_SCHEMA_JSON)
        .context("Failed to parse embedded sensor.report.v1.schema.json")
}

/// Build a validator for a specific definition in the schema
fn build_validator_for_definition(
    schema: &Value,
    definition: &str,
) -> Result<jsonschema::Validator> {
    // Create a schema that references the specific definition
    let ref_schema = serde_json::json!({
        "$ref": format!("#/definitions/{}", definition),
        "definitions": schema["definitions"]
    });

    jsonschema::validator_for(&ref_schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))
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
fn test_lang_receipt_validates_against_schema() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "LangReceipt")?;

    let output = tokmd_cmd().arg("--format").arg("json").output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "LangReceipt validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_module_receipt_validates_against_schema() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "ModuleReceipt")?;

    let output = tokmd_cmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "ModuleReceipt validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_export_receipt_validates_against_schema() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "ExportReceipt")?;

    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("json")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "ExportReceipt validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_export_meta_validates_against_schema() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "ExportMeta")?;

    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // The first line of JSONL output is the meta record
    let first_line = stdout.lines().next().context("No output lines")?;
    let json: Value = serde_json::from_str(first_line).context("Failed to parse meta JSON")?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "ExportMeta validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_export_row_validates_against_schema() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "ExportRow")?;

    let output = tokmd_cmd()
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Skip the first line (meta) and validate data rows
    for (i, line) in stdout.lines().skip(1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let json: Value = serde_json::from_str(line).context("Failed to parse row JSON")?;

        if !validator.is_valid(&json) {
            let error_messages: Vec<String> = validator
                .iter_errors(&json)
                .map(|e| format!("{} at {}", e, e.instance_path()))
                .collect();
            panic!(
                "ExportRow validation failed on row {}:\n{}\n\nOutput:\n{}",
                i + 1,
                error_messages.join("\n"),
                serde_json::to_string_pretty(&json).unwrap_or_default()
            );
        }
    }
    Ok(())
}

#[test]
fn test_analysis_receipt_validates_against_schema() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "AnalysisReceipt")?;

    // Test with the default 'receipt' preset
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--format")
        .arg("json")
        .arg("--preset")
        .arg("receipt")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "AnalysisReceipt validation failed (preset=receipt):\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_analysis_receipt_health_preset_validates() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "AnalysisReceipt")?;

    // Test with the 'health' preset which includes TODO density
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--format")
        .arg("json")
        .arg("--preset")
        .arg("health")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "AnalysisReceipt validation failed (preset=health):\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_analysis_receipt_supply_preset_validates() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "AnalysisReceipt")?;

    // Test with the 'supply' preset which includes assets and dependencies
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--format")
        .arg("json")
        .arg("--preset")
        .arg("supply")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "AnalysisReceipt validation failed (preset=supply):\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_analysis_receipt_with_context_window_validates() -> Result<()> {
    let schema = load_schema()?;
    let validator = build_validator_for_definition(&schema, "AnalysisReceipt")?;

    // Test with a context window to exercise the context_window report
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--format")
        .arg("json")
        .arg("--preset")
        .arg("receipt")
        .arg("--window")
        .arg("128000")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let json: Value = serde_json::from_str(&stdout)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "AnalysisReceipt validation failed (with --window):\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }

    // Verify the context_window field is present
    assert!(
        json["derived"]["context_window"].is_object(),
        "Expected context_window to be present when --window is specified"
    );
    Ok(())
}

#[test]
fn test_schema_version_matches_constant() -> Result<()> {
    // Verify that the schema versions in schema.json match SCHEMA_VERSION in code
    let schema = load_schema()?;

    // Check LangReceipt schema_version const
    let lang_version =
        &schema["definitions"]["LangReceipt"]["properties"]["schema_version"]["const"];
    assert_eq!(
        lang_version
            .as_u64()
            .context("schema_version should be integer")?,
        2,
        "LangReceipt schema_version should be 2"
    );

    // Check ModuleReceipt schema_version const
    let module_version =
        &schema["definitions"]["ModuleReceipt"]["properties"]["schema_version"]["const"];
    assert_eq!(
        module_version
            .as_u64()
            .context("schema_version should be integer")?,
        2,
        "ModuleReceipt schema_version should be 2"
    );

    // Check ExportReceipt schema_version const
    let export_version =
        &schema["definitions"]["ExportReceipt"]["properties"]["schema_version"]["const"];
    assert_eq!(
        export_version
            .as_u64()
            .context("schema_version should be integer")?,
        2,
        "ExportReceipt schema_version should be 2"
    );

    // Check ExportMeta schema_version const
    let meta_version =
        &schema["definitions"]["ExportMeta"]["properties"]["schema_version"]["const"];
    assert_eq!(
        meta_version
            .as_u64()
            .context("schema_version should be integer")?,
        2,
        "ExportMeta schema_version should be 2"
    );

    // Check AnalysisReceipt schema_version const
    let analysis_version =
        &schema["definitions"]["AnalysisReceipt"]["properties"]["schema_version"]["const"];
    assert_eq!(
        analysis_version
            .as_u64()
            .context("schema_version should be integer")?,
        5,
        "AnalysisReceipt schema_version should be 5"
    );
    Ok(())
}

#[test]
fn test_handoff_manifest_validates_against_schema() -> Result<()> {
    let schema = load_handoff_schema()?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile handoff schema: {}", e))?;

    let dir = tempdir()?;
    let out_dir = dir.path().join("handoff_out");

    let output = tokmd_cmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd handoff failed: {}", stderr);
    }

    let manifest_content =
        std::fs::read_to_string(out_dir.join("manifest.json")).context("read manifest.json")?;
    let json: Value = serde_json::from_str(&manifest_content)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "Handoff manifest validation failed:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

// =============================================================================
// sensor.report.v1 Schema Validation Tests
// =============================================================================

#[test]
fn test_sensor_report_schema_is_valid_json_schema() -> Result<()> {
    // Verify the schema itself is valid JSON Schema
    let schema = load_sensor_report_schema()?;
    jsonschema::validator_for(&schema)
        .map_err(|e| anyhow::anyhow!("sensor.report.v1 schema is not valid: {}", e))?;
    Ok(())
}

#[test]
fn test_sensor_report_example_pass_validates() -> Result<()> {
    let schema = load_sensor_report_schema()?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;

    // Read the pass example from test data
    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("sensor_report")
        .join("pass.json");

    let content = std::fs::read_to_string(&example_path)
        .with_context(|| format!("Failed to read {}", example_path.display()))?;
    let json: Value = serde_json::from_str(&content)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "sensor.report.v1 pass example validation failed:\n{}",
            error_messages.join("\n")
        );
    }
    Ok(())
}

#[test]
fn test_sensor_report_example_fail_validates() -> Result<()> {
    let schema = load_sensor_report_schema()?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;

    // Read the fail example from test data
    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("sensor_report")
        .join("fail.json");

    let content = std::fs::read_to_string(&example_path)
        .with_context(|| format!("Failed to read {}", example_path.display()))?;
    let json: Value = serde_json::from_str(&content)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "sensor.report.v1 fail example validation failed:\n{}",
            error_messages.join("\n")
        );
    }
    Ok(())
}

#[test]
fn test_envelope_struct_validates_against_sensor_report_v1() -> Result<()> {
    use tokmd_envelope::{CapabilityStatus, SensorReport, ToolMeta, Verdict};

    let schema = load_sensor_report_schema()?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;

    // Create a SensorReport using the Rust struct
    let mut report = SensorReport::new(
        ToolMeta::tokmd("1.5.0", "cockpit"),
        "2024-01-15T10:30:00Z".to_string(),
        Verdict::Pass,
        "Test summary".to_string(),
    );

    // Add a capability
    report.add_capability("mutation", CapabilityStatus::available());

    // Serialize to JSON
    let json: Value = serde_json::to_value(&report)?;

    if !validator.is_valid(&json) {
        let error_messages: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| format!("{} at {}", e, e.instance_path()))
            .collect();
        panic!(
            "SensorReport struct does not validate against schema:\n{}\n\nOutput:\n{}",
            error_messages.join("\n"),
            serde_json::to_string_pretty(&json).unwrap_or_default()
        );
    }
    Ok(())
}

#[test]
fn test_envelope_output_determinism() -> Result<()> {
    use tokmd_envelope::{
        Artifact, CapabilityStatus, Finding, FindingSeverity, GateItem, GateResults, SensorReport,
        ToolMeta, Verdict, findings,
    };

    // Create identical reports twice
    let build_report = || {
        let mut caps = std::collections::BTreeMap::new();
        caps.insert("mutation".to_string(), CapabilityStatus::available());
        caps.insert(
            "coverage".to_string(),
            CapabilityStatus::unavailable("missing"),
        );

        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-15T10:30:00Z".to_string(),
            Verdict::Warn,
            "Test summary".to_string(),
        );

        report.add_finding(Finding::new(
            findings::risk::CHECK_ID,
            findings::risk::HOTSPOT,
            FindingSeverity::Warn,
            "Hotspot",
            "Message",
        ));

        let gates = GateResults::new(Verdict::Warn, vec![GateItem::new("test", Verdict::Warn)]);
        report = report.with_data(serde_json::json!({ "gates": gates }));
        report = report.with_capabilities(caps);
        report = report.with_artifacts(vec![Artifact::receipt("report.json")]);

        report
    };

    let report1 = build_report();
    let report2 = build_report();

    let json1 = serde_json::to_string_pretty(&report1)?;
    let json2 = serde_json::to_string_pretty(&report2)?;

    assert_eq!(
        json1, json2,
        "SensorReport serialization should be deterministic"
    );

    Ok(())
}
