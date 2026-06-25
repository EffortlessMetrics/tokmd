#![cfg(feature = "analysis")]

//! End-to-end coverage for the `tokmd packet generate` orchestrator.
//!
//! These tests lock the packet generation status behavior over the real
//! `analyze`, `context`, `syntax`, and `evidence-packet` surfaces: a complete
//! packet writes all artifacts and a valid manifest, `--no-syntax` omits the
//! optional artifact, and unresolved refs fail the run.

mod common;

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

const EVIDENCE_PACKET_SCHEMA_JSON: &str = include_str!("../schemas/evidence-packet.schema.json");

const SCOPE_FILE: &str = "src/runtime/api/MarkdownObject.rs";

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

fn read_manifest_at(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
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
fn packet_generate_produces_complete_packet() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--no-progress",
            SCOPE_FILE,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"complete\""));

    let sensor_dir = dir.path().join("sensors").join("tokmd");
    for artifact in ["analyze.md", "analyze.json", "context.md", "syntax.json"] {
        assert!(
            sensor_dir.join(artifact).is_file(),
            "expected packet artifact {artifact} to exist"
        );
    }

    let manifest = read_manifest_at(&sensor_dir.join("manifest.json"));
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["schema"], "tokmd.evidence-packet/v1");
    assert_eq!(manifest["preset"], "bun-ub");
    assert_eq!(manifest["base"], "main");
    assert_eq!(manifest["head"], "HEAD");
    assert_eq!(manifest["paths"][0], SCOPE_FILE);
    assert_eq!(manifest["status"], "complete");
    assert_eq!(
        manifest["artifacts"]["analyze_json"],
        "sensors/tokmd/analyze.json"
    );
    assert_eq!(
        manifest["artifacts"]["syntax_json"],
        "sensors/tokmd/syntax.json"
    );

    // The generated analyze.json must carry the fields the evidence packet
    // validates against, proving the orchestrator wired real receipts.
    let analyze: Value = read_manifest_at(&sensor_dir.join("analyze.json"));
    assert_eq!(analyze["status"], "complete");
    assert_eq!(analyze["args"]["preset"], "bun-ub");
    assert_eq!(analyze["source"]["inputs"][0], SCOPE_FILE);
}

#[test]
fn packet_generate_no_syntax_omits_optional_artifact() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--no-syntax",
            "--no-progress",
            SCOPE_FILE,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"complete\""));

    let sensor_dir = dir.path().join("sensors").join("tokmd");
    assert!(!sensor_dir.join("syntax.json").exists());

    let manifest = read_manifest_at(&sensor_dir.join("manifest.json"));
    assert_validates_against_schema(&manifest);
    assert_eq!(manifest["status"], "complete");
    assert!(manifest["artifacts"].get("syntax_json").is_none());
}

#[test]
fn packet_generate_no_syntax_clears_stale_syntax_artifact() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();
    let sensor_dir = dir.path().join("sensors").join("tokmd");
    std::fs::create_dir_all(&sensor_dir).unwrap();
    // A stale syntax.json from a prior run must not leak into a --no-syntax
    // packet via evidence-packet's directory auto-detection.
    std::fs::write(
        sensor_dir.join("syntax.json"),
        "{\"schema\":\"tokmd.syntax_receipts.v1\",\"status\":\"complete\",\"receipts\":[]}",
    )
    .unwrap();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--no-syntax",
            "--no-progress",
            SCOPE_FILE,
        ])
        .assert()
        .success();

    assert!(!sensor_dir.join("syntax.json").exists());
    let manifest = read_manifest_at(&sensor_dir.join("manifest.json"));
    assert!(manifest["artifacts"].get("syntax_json").is_none());
}

#[test]
fn packet_generate_honors_custom_output_dir() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--out",
            "review/packet",
            "--no-progress",
            SCOPE_FILE,
        ])
        .assert()
        .success();

    let packet_dir = dir.path().join("review").join("packet");
    assert!(packet_dir.join("manifest.json").is_file());
    assert!(packet_dir.join("analyze.json").is_file());
    assert!(!dir.path().join("sensors").join("tokmd").exists());
}

#[test]
fn packet_generate_non_effort_preset_does_not_force_effort() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();

    // A non-effort preset (e.g. receipt) must not force an effort request, which
    // would otherwise trip ref validation (and break non-git builds).
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--preset",
            "receipt",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--no-syntax",
            "--no-progress",
            SCOPE_FILE,
        ])
        .assert()
        .success();

    let manifest = read_manifest_at(
        &dir.path()
            .join("sensors")
            .join("tokmd")
            .join("manifest.json"),
    );
    assert_eq!(manifest["preset"], "receipt");
}

#[test]
fn packet_generate_fails_on_unresolved_ref() {
    if !common::git_available() {
        return;
    }

    let dir = init_repo_with_scope();

    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .args([
            "packet",
            "generate",
            "--base",
            "does-not-exist",
            "--head",
            "HEAD",
            "--no-progress",
            SCOPE_FILE,
        ])
        .assert()
        .failure();
}
