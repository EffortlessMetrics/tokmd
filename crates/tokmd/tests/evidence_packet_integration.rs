#![cfg(feature = "analysis")]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

const EVIDENCE_PACKET_SCHEMA_JSON: &str = include_str!("../schemas/evidence-packet.schema.json");

fn write_sensor_artifacts(root: &std::path::Path, analyze_json: &str) {
    let sensor_dir = root.join("sensors").join("tokmd");
    std::fs::create_dir_all(&sensor_dir).unwrap();
    std::fs::write(sensor_dir.join("analyze.md"), "# Bun UB analyze\n").unwrap();
    std::fs::write(sensor_dir.join("analyze.json"), analyze_json).unwrap();
    std::fs::write(sensor_dir.join("context.md"), "# Context\n").unwrap();
}

fn init_repo_with_scope() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    assert!(common::init_git_repo(dir.path()));
    let scope_dir = dir.path().join("src").join("runtime").join("api");
    std::fs::create_dir_all(&scope_dir).unwrap();
    std::fs::write(scope_dir.join("MarkdownObject.rs"), "pub fn old() {}\n").unwrap();
    assert!(common::git_add_commit(dir.path(), "initial"));
    std::fs::write(
        scope_dir.join("MarkdownObject.rs"),
        "pub fn old() {}\npub fn new_boundary() {}\n",
    )
    .unwrap();
    assert!(common::git_add_commit(dir.path(), "change api"));
    dir
}

fn valid_analyze_json(status: &str, warnings: &[&str], preset: &str) -> String {
    serde_json::json!({
        "status": status,
        "warnings": warnings,
        "args": {
            "preset": preset
        },
        "source": {
            "inputs": ["src/runtime/api/MarkdownObject.rs"]
        }
    })
    .to_string()
}

fn read_manifest(root: &std::path::Path) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(root.join("sensors").join("tokmd").join("manifest.json")).unwrap(),
    )
    .unwrap()
}

fn assert_validates_against_schema(manifest: &Value) {
    let schema: Value = serde_json::from_str(EVIDENCE_PACKET_SCHEMA_JSON).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    if !validator.is_valid(manifest) {
        let errors: Vec<String> = validator
            .iter_errors(manifest)
            .map(|err| format!("{} at {}", err, err.instance_path()))
            .collect();
        panic!(
            "evidence packet manifest did not validate:\n{}\n\n{}",
            errors.join("\n"),
            serde_json::to_string_pretty(manifest).unwrap()
        );
    }
}

#[test]
fn evidence_packet_manifest_complete_when_artifacts_match() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(dir.path(), &valid_analyze_json("complete", &[], "bun-ub"));

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "evidence-packet",
            "--base",
            "main",
            "--head",
            "HEAD",
            "src/runtime/api/MarkdownObject.rs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"complete\""));

    let manifest = read_manifest(dir.path());
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["schema"], "tokmd.evidence-packet/v1");
    assert_eq!(manifest["preset"], "bun-ub");
    assert_eq!(manifest["base"], "main");
    assert_eq!(manifest["head"], "HEAD");
    assert_eq!(manifest["paths"][0], "src/runtime/api/MarkdownObject.rs");
    assert_eq!(manifest["status"], "complete");
    assert_eq!(
        manifest["artifacts"]["analyze_md"],
        "sensors/tokmd/analyze.md"
    );
    assert!(
        manifest["non_claims"][0]
            .as_str()
            .unwrap()
            .contains("does not prove UB exists or is absent")
    );
    assert!(manifest["reproduce"].as_array().unwrap().iter().any(|cmd| {
        cmd.as_str()
            .unwrap()
            .contains("tokmd analyze --preset bun-ub --format json")
    }));
}

#[test]
fn evidence_packet_manifest_partial_preserves_analyze_warnings() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(
        dir.path(),
        &valid_analyze_json("partial", &["git scan failed: sample"], "bun-ub"),
    );

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "evidence-packet",
            "--base",
            "main",
            "--head",
            "HEAD",
            "src/runtime/api/MarkdownObject.rs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"partial\""));

    let manifest = read_manifest(dir.path());
    assert_eq!(manifest["status"], "partial");
    let warnings = manifest["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w == "analyze.json status is partial")
    );
    assert!(warnings.iter().any(|w| w == "git scan failed: sample"));
}

#[test]
fn evidence_packet_manifest_failed_when_required_artifact_missing() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    let sensor_dir = dir.path().join("sensors").join("tokmd");
    std::fs::create_dir_all(&sensor_dir).unwrap();
    std::fs::write(
        sensor_dir.join("analyze.json"),
        valid_analyze_json("complete", &[], "bun-ub"),
    )
    .unwrap();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "evidence-packet",
            "--base",
            "main",
            "--head",
            "HEAD",
            "src/runtime/api/MarkdownObject.rs",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("evidence packet failed"));

    let manifest = read_manifest(dir.path());
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["status"], "failed");
    assert!(manifest["errors"].as_array().unwrap().iter().any(|err| {
        err.as_str()
            .unwrap()
            .contains("required artifact analyze_md missing")
    }));
}

#[test]
fn evidence_packet_manifest_failed_when_analyze_preset_mismatches() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(dir.path(), &valid_analyze_json("complete", &[], "receipt"));

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "evidence-packet",
            "--base",
            "main",
            "--head",
            "HEAD",
            "src/runtime/api/MarkdownObject.rs",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("preset 'receipt'"));

    let manifest = read_manifest(dir.path());
    assert_eq!(manifest["status"], "failed");
    assert!(manifest["errors"].as_array().unwrap().iter().any(|err| {
        err.as_str()
            .unwrap()
            .contains("does not match requested preset")
    }));
}
