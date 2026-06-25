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

fn write_syntax_artifact(root: &std::path::Path) {
    write_syntax_artifact_with(
        root,
        "complete",
        &[],
        &[],
        &["src/runtime/api/MarkdownObject.rs"],
    );
}

fn write_syntax_artifact_with(
    root: &std::path::Path,
    status: &str,
    warnings: &[&str],
    errors: &[&str],
    paths: &[&str],
) {
    write_syntax_artifact_with_receipts(
        root,
        status,
        warnings,
        errors,
        paths,
        serde_json::json!([]),
    );
}

fn write_syntax_artifact_with_receipts(
    root: &std::path::Path,
    status: &str,
    warnings: &[&str],
    errors: &[&str],
    paths: &[&str],
    receipts: Value,
) {
    let sensor_dir = root.join("sensors").join("tokmd");
    std::fs::write(
        sensor_dir.join("syntax.json"),
        serde_json::json!({
            "schema": "tokmd.syntax_receipts.v1",
            "status": status,
            "paths": paths,
            "receipts": receipts,
            "warnings": warnings,
            "errors": errors
        })
        .to_string(),
    )
    .unwrap();
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
    assert!(manifest["artifacts"].get("syntax_json").is_none());
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
fn evidence_packet_manifest_records_optional_syntax_artifact() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(dir.path(), &valid_analyze_json("complete", &[], "bun-ub"));
    write_syntax_artifact(dir.path());

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
    assert_eq!(
        manifest["artifacts"]["syntax_json"],
        "sensors/tokmd/syntax.json"
    );
    assert!(
        manifest["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cmd| { cmd.as_str().unwrap().contains("tokmd syntax --no-progress") })
    );
}

#[test]
fn evidence_packet_manifest_ranks_syntax_review_signals() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(dir.path(), &valid_analyze_json("complete", &[], "bun-ub"));
    write_syntax_artifact_with_receipts(
        dir.path(),
        "complete",
        &[],
        &[],
        &["src/runtime/api/MarkdownObject.rs"],
        serde_json::json!([
            {
                "schema": "tokmd.syntax_receipt.v1",
                "path": "src/runtime/api/MarkdownObject.rs",
                "status": "complete",
                "review_signals": [
                    {
                        "category": "public_surface",
                        "severity": "medium",
                        "score": 40,
                        "kind": "public_function",
                        "reason": "public or exported symbol changed",
                        "evidence": "pub fn new_boundary"
                    },
                    {
                        "category": "panic_seam",
                        "severity": "high",
                        "score": 95,
                        "kind": "expect_call",
                        "reason": "panic-like seam near review scope",
                        "evidence": "expect"
                    }
                ]
            }
        ]),
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
        .stdout(predicate::str::contains("\"review_priority\""));

    let manifest = read_manifest(dir.path());
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["status"], "complete");
    let items = manifest["review_priority"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["rank"], 1);
    assert_eq!(items[0]["category"], "panic_seam");
    assert_eq!(items[0]["severity"], "high");
    assert_eq!(items[0]["score"], 95);
    assert_eq!(
        items[0]["refs"][0],
        "sensors/tokmd/syntax.json#/receipts/0/review_signals/1"
    );
    assert_eq!(items[1]["rank"], 2);
    assert_eq!(items[1]["category"], "public_surface");
}

#[test]
fn evidence_packet_manifest_preserves_syntax_warnings_as_partial() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(dir.path(), &valid_analyze_json("complete", &[], "bun-ub"));
    write_syntax_artifact_with(
        dir.path(),
        "partial",
        &["parser recovered with syntax errors"],
        &[],
        &["src/runtime/api/MarkdownObject.rs"],
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
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["status"], "partial");
    let warnings = manifest["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w == "syntax_json status is partial")
    );
    assert!(
        warnings
            .iter()
            .any(|w| { w == "syntax_json warning: parser recovered with syntax errors" })
    );
}

#[test]
fn evidence_packet_manifest_preserves_failed_syntax_artifact_as_partial() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(dir.path(), &valid_analyze_json("complete", &[], "bun-ub"));
    write_syntax_artifact_with(
        dir.path(),
        "failed",
        &[],
        &["parser unavailable for language"],
        &["src/runtime/api/MarkdownObject.rs"],
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
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["status"], "partial");
    assert_eq!(
        manifest["artifacts"]["syntax_json"],
        "sensors/tokmd/syntax.json"
    );
    let warnings = manifest["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w == "syntax_json status is failed"),
        "failed syntax status should be explicit: {warnings:?}"
    );
    assert!(
        warnings
            .iter()
            .any(|w| w == "syntax_json error: parser unavailable for language"),
        "syntax errors should be preserved in packet warnings: {warnings:?}"
    );
    assert!(
        manifest["errors"].as_array().unwrap().is_empty(),
        "advisory syntax failure should not make required analyze/context evidence invalid"
    );
}

#[test]
fn evidence_packet_manifest_preserves_malformed_syntax_artifact_as_partial() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    write_sensor_artifacts(dir.path(), &valid_analyze_json("complete", &[], "bun-ub"));
    std::fs::write(
        dir.path().join("sensors").join("tokmd").join("syntax.json"),
        "{not json",
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
        .success()
        .stdout(predicate::str::contains("\"status\": \"partial\""));

    let manifest = read_manifest(dir.path());
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["status"], "partial");
    assert_eq!(
        manifest["artifacts"]["syntax_json"],
        "sensors/tokmd/syntax.json"
    );
    assert!(
        manifest["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| {
                warning
                    .as_str()
                    .unwrap()
                    .contains("failed to parse syntax_json")
            })
    );
    assert!(
        manifest["errors"].as_array().unwrap().is_empty(),
        "malformed optional syntax evidence should degrade the packet without hiding required artifact validity"
    );
}

#[test]
fn evidence_packet_manifest_warns_for_explicit_missing_syntax_artifact() {
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
            "--syntax-json",
            "sensors/tokmd/syntax.json",
            "src/runtime/api/MarkdownObject.rs",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"partial\""));

    let manifest = read_manifest(dir.path());
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["status"], "partial");
    assert_eq!(
        manifest["artifacts"]["syntax_json"],
        "sensors/tokmd/syntax.json"
    );
    assert!(manifest["warnings"].as_array().unwrap().iter().any(|err| {
        err.as_str()
            .unwrap()
            .contains("optional artifact syntax_json missing")
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

#[test]
fn evidence_packet_manifest_failed_when_analyze_json_is_utf16le() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    let sensor_dir = dir.path().join("sensors").join("tokmd");
    std::fs::create_dir_all(&sensor_dir).unwrap();
    std::fs::write(sensor_dir.join("analyze.md"), "# Bun UB analyze\n").unwrap();
    std::fs::write(sensor_dir.join("context.md"), "# Context\n").unwrap();
    std::fs::write(sensor_dir.join("analyze.json"), [0xFF_u8, 0xFE, b'{', b'}']).unwrap();

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
    assert_eq!(manifest["status"], "failed");
    assert!(manifest["errors"].as_array().unwrap().iter().any(|err| {
        err.as_str().unwrap().contains("UTF-16LE") && err.as_str().unwrap().contains("PowerShell")
    }));
}
