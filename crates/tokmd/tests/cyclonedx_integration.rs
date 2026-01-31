//! Integration tests for CycloneDX export format.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn tokmd() -> Command {
    cargo_bin_cmd!("tokmd")
}

#[test]
fn test_cyclonedx_export_valid_json() {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "CycloneDX export should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // CycloneDX required fields
    assert_eq!(
        parsed["bomFormat"], "CycloneDX",
        "bomFormat should be CycloneDX"
    );
    assert!(
        parsed.get("specVersion").is_some(),
        "Should have specVersion"
    );
}

#[test]
fn test_cyclonedx_spec_version() {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();

    // Check spec version is 1.6
    assert_eq!(parsed["specVersion"], "1.6", "specVersion should be 1.6");
}

#[test]
fn test_cyclonedx_has_components() {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();

    // Should have components array
    assert!(
        parsed.get("components").is_some(),
        "Should have components array"
    );
    assert!(
        parsed["components"].is_array(),
        "components should be an array"
    );
}

#[test]
fn test_cyclonedx_component_structure() {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();

    let components = parsed["components"].as_array().unwrap();

    // If there are components, check their structure
    if !components.is_empty() {
        let first = &components[0];

        // Required fields per CycloneDX spec
        assert!(first.get("type").is_some(), "Component should have type");
        assert!(first.get("name").is_some(), "Component should have name");
    }
}

#[test]
fn test_cyclonedx_metadata() {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();

    // Should have metadata
    assert!(
        parsed.get("metadata").is_some(),
        "Should have metadata object"
    );

    let metadata = &parsed["metadata"];

    // metadata should have tools array
    if let Some(tools) = metadata.get("tools") {
        assert!(
            tools.is_array() || tools.is_object(),
            "tools should be array or object"
        );
    }
}

#[test]
fn test_cyclonedx_to_file() {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("bom.json");

    tokmd()
        .args([
            "export",
            "--format",
            "cyclonedx",
            "--out",
            output_path.to_str().unwrap(),
            ".",
        ])
        .assert()
        .success();

    // Verify file was created and is valid
    assert!(output_path.exists(), "Output file should exist");

    let content = fs::read_to_string(&output_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).expect("File should contain valid JSON");

    assert_eq!(parsed["bomFormat"], "CycloneDX");
}

#[test]
fn test_cyclonedx_serial_number() {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();

    // serialNumber should be a URN UUID if present
    if let Some(serial) = parsed.get("serialNumber") {
        let serial_str = serial.as_str().unwrap();
        assert!(
            serial_str.starts_with("urn:uuid:"),
            "serialNumber should be a URN UUID"
        );
    }
}
