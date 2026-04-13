//! Integration tests for CycloneDX export format.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;
use std::error::Error;
use std::fs;
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn Error>>;

fn tokmd() -> Command {
    cargo_bin_cmd!("tokmd")
}

#[test]
fn test_cyclonedx_export_valid_json() -> TestResult {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()?;

    assert!(output.status.success(), "CycloneDX export should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)?;

    // CycloneDX required fields
    assert_eq!(
        parsed["bomFormat"], "CycloneDX",
        "bomFormat should be CycloneDX"
    );
    assert!(
        parsed.get("specVersion").is_some(),
        "Should have specVersion"
    );

    Ok(())
}

#[test]
fn test_cyclonedx_spec_version() -> TestResult {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)?;

    // Check spec version is 1.6
    assert_eq!(parsed["specVersion"], "1.6", "specVersion should be 1.6");

    Ok(())
}

#[test]
fn test_cyclonedx_has_components() -> TestResult {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)?;

    // Should have components array
    assert!(
        parsed.get("components").is_some(),
        "Should have components array"
    );
    assert!(
        parsed["components"].is_array(),
        "components should be an array"
    );

    Ok(())
}

#[test]
fn test_cyclonedx_component_structure() -> TestResult {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)?;

    let components = parsed
        .get("components")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "components field should be a valid JSON array",
            )
        })?;

    // If there are components, check their structure
    if !components.is_empty() {
        let first = &components[0];

        // Required fields per CycloneDX spec
        assert!(first.get("type").is_some(), "Component should have type");
        assert!(first.get("name").is_some(), "Component should have name");
    }

    Ok(())
}

#[test]
fn test_cyclonedx_metadata() -> TestResult {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)?;

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

    Ok(())
}

#[test]
fn test_cyclonedx_to_file() -> TestResult {
    let dir = TempDir::new()?;
    let output_path = dir.path().join("bom.json");
    let output_path = output_path.to_str().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Path should be valid UTF-8",
        )
    })?;

    tokmd()
        .args(["export", "--format", "cyclonedx", "--out", output_path, "."])
        .assert()
        .success();

    // Verify file was created and is valid
    assert!(
        dir.path().join("bom.json").exists(),
        "Output file should exist"
    );

    let content = fs::read_to_string(dir.path().join("bom.json"))?;
    let parsed: Value = serde_json::from_str(&content)?;

    assert_eq!(parsed["bomFormat"], "CycloneDX");

    Ok(())
}

#[test]
fn test_cyclonedx_serial_number() -> TestResult {
    let output = tokmd()
        .args(["export", "--format", "cyclonedx", "."])
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout)?;

    // serialNumber should be a URN UUID if present
    if let Some(serial) = parsed.get("serialNumber") {
        let serial_str = serial.as_str().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "serialNumber field should be a string",
            )
        })?;
        assert!(
            serial_str.starts_with("urn:uuid:"),
            "serialNumber should be a URN UUID"
        );
    }

    Ok(())
}
