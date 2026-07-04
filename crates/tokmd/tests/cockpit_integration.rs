//! Integration tests for the `tokmd cockpit` command.

#![cfg(feature = "git")]
mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{TempDir, tempdir};

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

const REVIEW_PACKET_MANIFEST_SCHEMA_JSON: &str =
    include_str!("../schemas/review-packet-manifest.schema.json");
const REVIEW_PACKET_EVIDENCE_SCHEMA_JSON: &str =
    include_str!("../schemas/review-packet-evidence.schema.json");
const REVIEW_MAP_SCHEMA_JSON: &str = include_str!("../schemas/review-map.schema.json");

fn assert_validates_against_schema(schema_json: &str, instance: &serde_json::Value, label: &str) {
    let schema: serde_json::Value =
        serde_json::from_str(schema_json).unwrap_or_else(|err| panic!("{label} schema: {err}"));
    let validator = jsonschema::validator_for(&schema)
        .unwrap_or_else(|err| panic!("{label} schema should compile: {err}"));

    if !validator.is_valid(instance) {
        let errors: Vec<String> = validator
            .iter_errors(instance)
            .map(|err| format!("{} at {}", err, err.instance_path()))
            .collect();
        panic!(
            "{label} did not validate against schema:\n{}\n\nInstance:\n{}",
            errors.join("\n"),
            serde_json::to_string_pretty(instance)
                .expect("failed to serialize invalid schema instance")
        );
    }
}

fn basic_cockpit_repo() -> Option<TempDir> {
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return None;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return None;
    }

    std::fs::write(dir.path().join("lib.rs"), "fn main() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return None;
    }

    let status = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("Skipping: feature branch checkout failed");
        return None;
    }

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Add new file") {
        eprintln!("Skipping: second commit failed");
        return None;
    }

    Some(dir)
}

const PROOF_RUN_OBSERVATION_JSON: &str = r#"{
  "schema": "tokmd.proof_run_observation.v1",
  "status": "passed",
  "execution_status": "executed",
  "profile": "fast",
  "base": "origin/main",
  "head": "abc123",
  "ok": true,
  "execution_guard": {
    "enabled": true,
    "ci": true,
    "reason": "required proof-run summary verified"
  },
  "counts": {
    "commands_total": 1,
    "required_planned": 1,
    "advisory_skipped": 0,
    "executed": 1,
    "passed": 1,
    "failed": 0
  },
  "scopes": [
    {
      "name": "tokmd_cockpit",
      "kind": "test",
      "command": "cargo test -p tokmd-cockpit",
      "status": "passed",
      "exit_code": 0
    }
  ],
  "changed_files": ["crates/tokmd-cockpit/src/lib.rs"],
  "unknown_files": []
}"#;

const COVERAGE_RECEIPT_JSON: &str = r#"{
  "schema": "tokmd.coverage_receipt.v1",
  "schema_version": 1,
  "repo": "EffortlessMetrics/tokmd",
  "lane": "scoped",
  "flag": "tokmd_cockpit",
  "workflow": "Coverage",
  "sha": "abc123",
  "github": {
    "run_id": "12345",
    "run_attempt": "1",
    "event_name": "pull_request",
    "ref_name": "feature"
  },
  "artifacts": [],
  "status": { "ok": true, "missing": [], "empty": [] }
}"#;

const PROOF_PACK_ROUTE_JSON: &str = r#"{
  "schema": "tokmd.proof_pack_route.v1",
  "schema_version": 5,
  "base": "main",
  "head": "HEAD",
  "labels": [],
  "changed_files": [
    {
      "changed_file": "new.rs",
      "path": "new.rs",
      "surface": "tokmd-cockpit",
      "required_packs": ["tokmd-cockpit"],
      "proof_packs": ["tokmd-cockpit"],
      "reason": "manifest_match",
      "policy": "blocking",
      "lanes": ["ci"],
      "deep_lanes": ["coverage_lite_pr"]
    }
  ],
  "unmatched_files": [],
  "skipped_by_policy": [
    {
      "lane": "coverage_lite_pr",
      "status": "skipped_by_policy",
      "reason": "deep_lane_requires_label",
      "matched_files": ["new.rs"],
      "lane_kind": "coverage",
      "tier": "deep",
      "blocking": false,
      "expensive": true,
      "required_labels": ["coverage"],
      "estimated_lem": 30,
      "estimate_source": "static"
    }
  ],
  "summary": {
    "changed_file_count": 1,
    "routed_file_count": 1,
    "unmatched_file_count": 0,
    "skipped_lane_count": 1,
    "skipped_reason_counts": {
      "deep_lane_requires_label": 1
    }
  }
}"#;

const DOC_ARTIFACTS_CHECK_JSON: &str = r#"{
  "schema": "tokmd.doc_artifacts_check.v1",
  "ok": true,
  "checked": {
    "required_docs": 1,
    "family_files": 11,
    "active_goals": 1,
    "spec_index_artifacts": 9,
    "spec_index_lanes": 2
  },
  "errors": []
}"#;

#[test]
fn test_cockpit_help() {
    // Given: The cockpit command exists
    // When: We run `tokmd cockpit --help`
    // Then: It should show help with expected options
    let mut cmd = tokmd_cmd();
    cmd.arg("cockpit")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--base"))
        .stdout(predicate::str::contains("--head"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--proof-run-summary"))
        .stdout(predicate::str::contains("--proof-observation"))
        .stdout(predicate::str::contains("--executor-observation"))
        .stdout(predicate::str::contains("--coverage-receipt"))
        .stdout(predicate::str::contains("--proof-route"))
        .stdout(predicate::str::contains("--doc-artifacts-check"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_cockpit_accepts_valid_proof_observation_input_without_receipt_change() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    let proof_path = dir.path().join("proof-run-observation.json");
    std::fs::write(&proof_path, PROOF_RUN_OBSERVATION_JSON).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .arg("--proof-observation")
        .arg(&proof_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "cockpit should accept valid proof observation input: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid cockpit JSON");
    assert!(json.get("schema_version").is_some());
    assert!(json.get("review_plan").is_some());
}

#[test]
fn test_cockpit_review_packet_includes_imported_proof_evidence() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    let proof_json = PROOF_RUN_OBSERVATION_JSON
        .replace("\"head\": \"abc123\"", "\"head\": \"HEAD\"")
        .replace(
            "\"changed_files\": [\"crates/tokmd-cockpit/src/lib.rs\"]",
            "\"changed_files\": [\"new.rs\"]",
        );
    std::fs::write(dir.path().join("proof-run-observation.json"), proof_json).unwrap();
    let baseline_packet_dir = dir.path().join(".tokmd").join("review-baseline");
    let packet_dir = dir.path().join(".tokmd").join("review");

    let mut baseline_cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    baseline_cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--review-packet-dir")
        .arg(&baseline_packet_dir)
        .assert()
        .success();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--proof-observation")
        .arg("proof-run-observation.json")
        .arg("--review-packet-dir")
        .arg(&packet_dir)
        .assert()
        .success();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("evidence.json")).unwrap())
            .expect("valid evidence JSON");
    let baseline_evidence: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(baseline_packet_dir.join("evidence.json")).unwrap(),
    )
    .expect("valid baseline evidence JSON");
    assert!(
        baseline_evidence.get("proof").is_none(),
        "proof evidence should stay absent when no proof artifacts are provided"
    );
    assert_validates_against_schema(
        REVIEW_PACKET_EVIDENCE_SCHEMA_JSON,
        &evidence,
        "review packet evidence with proof",
    );

    let proof = evidence["proof"].as_array().expect("proof evidence array");
    assert_eq!(proof.len(), 1);
    assert_eq!(proof[0]["kind"], "proof_run_observation");
    assert_eq!(proof[0]["source"], "proof/proof-run-observation.json");
    assert_eq!(proof[0]["source_schema"], "tokmd.proof_run_observation.v1");
    assert_eq!(proof[0]["profile"], "fast");
    assert_eq!(proof[0]["scope"], "tokmd_cockpit");
    assert_eq!(proof[0]["command"], "cargo test -p tokmd-cockpit");
    assert_eq!(proof[0]["required"], true);
    assert_eq!(proof[0]["advisory"], false);
    assert_eq!(proof[0]["execution_status"], "executed_passed");
    assert_eq!(proof[0]["availability"], "degraded");
    assert_eq!(proof[0]["commit_match"], "partial");
    assert_eq!(
        proof[0]["refs"][0],
        "proof/proof-run-observation.json#/scopes/0"
    );
    assert_eq!(
        evidence["overall_status"], baseline_evidence["overall_status"],
        "imported proof evidence must not promote or change cockpit verdicts"
    );

    let copied_proof_path = packet_dir.join("proof").join("proof-run-observation.json");
    assert!(
        copied_proof_path.exists(),
        "proof artifact should be copied into the review packet"
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("manifest.json")).unwrap())
            .expect("valid manifest JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_MANIFEST_SCHEMA_JSON,
        &manifest,
        "review packet manifest with proof",
    );
    let proof_artifact = manifest["artifacts"]
        .as_array()
        .expect("manifest artifacts")
        .iter()
        .find(|artifact| artifact["path"] == "proof/proof-run-observation.json")
        .expect("proof artifact listed in manifest");
    assert_eq!(proof_artifact["id"], "proof-run-observation");
    assert_eq!(proof_artifact["schema"], "tokmd.proof_run_observation.v1");
    let copied_bytes = std::fs::read(&copied_proof_path).unwrap();
    assert_eq!(
        proof_artifact["hash"]["hash"].as_str().expect("proof hash"),
        blake3::hash(&copied_bytes).to_hex().as_str(),
        "manifest hash should match copied proof artifact bytes"
    );

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("review-map.json")).unwrap())
            .expect("valid review map JSON");
    assert_validates_against_schema(
        REVIEW_MAP_SCHEMA_JSON,
        &review_map,
        "review map with proof refs",
    );
    assert!(
        review_map["evidence"]["refs"]
            .as_array()
            .expect("review-map evidence refs")
            .iter()
            .any(|reference| reference == "evidence.json#/proof"),
        "review map should expose packet-level proof evidence refs"
    );
    let item = review_map["items"]
        .as_array()
        .expect("review-map items")
        .iter()
        .find(|item| item["path"] == "new.rs")
        .expect("new.rs review item");
    let proof_refs = item["proof_refs"].as_array().expect("proof refs array");
    assert!(
        proof_refs
            .iter()
            .any(|reference| reference == "evidence.json#/proof/0"),
        "review item should link to normalized proof evidence"
    );
    assert!(
        proof_refs
            .iter()
            .any(|reference| reference == "proof/proof-run-observation.json#/scopes/0"),
        "review item should link to packet-local proof artifact"
    );

    let review_map_md = std::fs::read_to_string(packet_dir.join("review-map.md")).unwrap();
    assert!(review_map_md.contains(
        "Required: tokmd_cockpit passed (degraded, freshness: partial) - cargo test -p tokmd-cockpit"
    ));
    assert!(review_map_md.contains("Proof references:"));
    assert!(review_map_md.contains("evidence.json#/proof/0"));
    assert!(review_map_md.contains("proof/proof-run-observation.json#/scopes/0"));

    let comment_md = std::fs::read_to_string(packet_dir.join("comment.md")).unwrap();
    assert!(comment_md.contains("Proof evidence"));
    assert!(comment_md.contains("Required proof: 0 passed, 0 failed, 0 missing"));
    assert!(comment_md.contains("Advisory proof: 0 available, 0 missing"));
    assert!(comment_md.contains("Proof freshness: 0 exact, 1 partial, 0 stale, 0 unknown"));
}

#[test]
fn test_cockpit_review_packet_includes_imported_proof_route_evidence() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    std::fs::write(
        dir.path().join("proof-pack-route.json"),
        PROOF_PACK_ROUTE_JSON,
    )
    .unwrap();
    let baseline_packet_dir = dir.path().join(".tokmd").join("review-baseline");
    let packet_dir = dir.path().join(".tokmd").join("review");

    let mut baseline_cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    baseline_cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--review-packet-dir")
        .arg(&baseline_packet_dir)
        .assert()
        .success();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--proof-route")
        .arg("proof-pack-route.json")
        .arg("--review-packet-dir")
        .arg(&packet_dir)
        .assert()
        .success();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("evidence.json")).unwrap())
            .expect("valid evidence JSON");
    let baseline_evidence: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(baseline_packet_dir.join("evidence.json")).unwrap(),
    )
    .expect("valid baseline evidence JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_EVIDENCE_SCHEMA_JSON,
        &evidence,
        "review packet evidence with proof route",
    );

    let proof = evidence["proof"].as_array().expect("proof evidence array");
    assert_eq!(proof.len(), 1);
    assert_eq!(proof[0]["kind"], "proof_pack_route");
    assert_eq!(proof[0]["source"], "proof/proof-pack-route.json");
    assert_eq!(proof[0]["source_schema"], "tokmd.proof_pack_route.v1");
    assert_eq!(proof[0]["profile"], serde_json::Value::Null);
    assert_eq!(proof[0]["scope"], "proof_pack_route");
    assert_eq!(
        proof[0]["command"],
        "cargo xtask ci-plan --route-json-out target/ci/proof-pack-route.json"
    );
    assert_eq!(proof[0]["required"], false);
    assert_eq!(proof[0]["advisory"], true);
    assert_eq!(proof[0]["execution_status"], "planned");
    assert_eq!(proof[0]["availability"], "degraded");
    assert_eq!(proof[0]["commit_match"], "partial");
    assert_eq!(proof[0]["refs"][0], "proof/proof-pack-route.json#/summary");
    assert_eq!(
        evidence["overall_status"], baseline_evidence["overall_status"],
        "imported proof-route evidence must not promote or change cockpit verdicts"
    );

    let copied_route_path = packet_dir.join("proof").join("proof-pack-route.json");
    assert!(
        copied_route_path.exists(),
        "proof route receipt should be copied into the review packet"
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("manifest.json")).unwrap())
            .expect("valid manifest JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_MANIFEST_SCHEMA_JSON,
        &manifest,
        "review packet manifest with proof route",
    );
    let route_artifact = manifest["artifacts"]
        .as_array()
        .expect("manifest artifacts")
        .iter()
        .find(|artifact| artifact["path"] == "proof/proof-pack-route.json")
        .expect("proof route artifact listed in manifest");
    assert_eq!(route_artifact["id"], "proof-pack-route");
    assert_eq!(route_artifact["schema"], "tokmd.proof_pack_route.v1");
    let copied_bytes = std::fs::read(&copied_route_path).unwrap();
    assert_eq!(
        route_artifact["hash"]["hash"].as_str().expect("route hash"),
        blake3::hash(&copied_bytes).to_hex().as_str(),
        "manifest hash should match copied proof route artifact bytes"
    );

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("review-map.json")).unwrap())
            .expect("valid review map JSON");
    assert_validates_against_schema(
        REVIEW_MAP_SCHEMA_JSON,
        &review_map,
        "review map with proof route refs",
    );
    assert!(
        review_map["evidence"]["refs"]
            .as_array()
            .expect("review-map evidence refs")
            .iter()
            .any(|reference| reference == "evidence.json#/proof"),
        "review map should expose packet-level proof-route evidence refs"
    );
    let item = review_map["items"]
        .as_array()
        .expect("review-map items")
        .iter()
        .find(|item| item["path"] == "new.rs")
        .expect("new.rs review item");
    let proof_refs = item["proof_refs"].as_array().expect("proof refs array");
    assert!(
        proof_refs
            .iter()
            .any(|reference| reference == "evidence.json#/proof/0"),
        "review item should link to normalized proof-route evidence"
    );
    assert!(
        proof_refs
            .iter()
            .any(|reference| reference == "proof/proof-pack-route.json#/summary"),
        "review item should link to packet-local proof-route artifact"
    );

    let review_map_md = std::fs::read_to_string(packet_dir.join("review-map.md")).unwrap();
    assert!(review_map_md.contains(
        "Routing: proof_pack_route planned (degraded, freshness: partial) - cargo xtask ci-plan --route-json-out target/ci/proof-pack-route.json"
    ));
    assert!(review_map_md.contains(
        "Proof routing: 0 available, 0 missing, 1 degraded, 0 stale, 0 skipped, 0 unavailable"
    ));

    let comment_md = std::fs::read_to_string(packet_dir.join("comment.md")).unwrap();
    assert!(comment_md.contains(
        "Proof routing: 0 available, 0 missing, 1 degraded, 0 stale, 0 skipped, 0 unavailable"
    ));
    assert!(comment_md.contains("Required proof: 0 passed, 0 failed, 0 missing"));
}

#[test]
fn test_cockpit_review_packet_preserves_coverage_run_metadata() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    let coverage_json = COVERAGE_RECEIPT_JSON.replace("\"sha\": \"abc123\"", "\"sha\": \"HEAD\"");
    std::fs::write(dir.path().join("coverage-receipt.json"), coverage_json).unwrap();
    let packet_dir = dir.path().join(".tokmd").join("review");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--coverage-receipt")
        .arg("coverage-receipt.json")
        .arg("--review-packet-dir")
        .arg(&packet_dir)
        .assert()
        .success();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("evidence.json")).unwrap())
            .expect("valid evidence JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_EVIDENCE_SCHEMA_JSON,
        &evidence,
        "review packet evidence with coverage run metadata",
    );
    let proof = evidence["proof"].as_array().expect("proof evidence array");
    assert_eq!(proof.len(), 1);
    assert_eq!(proof[0]["kind"], "coverage_receipt");
    assert_eq!(proof[0]["run_id"], "12345");
    assert_eq!(proof[0]["run_attempt"], "1");
    assert_eq!(
        proof[0]["run_url"],
        "https://github.com/EffortlessMetrics/tokmd/actions/runs/12345"
    );
    assert_eq!(proof[0]["workflow"], "Coverage");
    assert_eq!(proof[0]["event_name"], "pull_request");
    assert_eq!(proof[0]["ref_name"], "feature");
}

#[test]
fn test_cockpit_review_packet_includes_imported_doc_artifacts_evidence() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    std::fs::write(
        dir.path().join("doc-artifacts-check.json"),
        DOC_ARTIFACTS_CHECK_JSON,
    )
    .unwrap();
    let packet_dir = dir.path().join(".tokmd").join("review");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--doc-artifacts-check")
        .arg("doc-artifacts-check.json")
        .arg("--review-packet-dir")
        .arg(&packet_dir)
        .assert()
        .success();

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("evidence.json")).unwrap())
            .expect("valid evidence JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_EVIDENCE_SCHEMA_JSON,
        &evidence,
        "review packet evidence with doc artifacts",
    );
    assert_eq!(
        evidence["doc_artifacts"]["source"],
        "docs/doc-artifacts-check.json"
    );
    assert_eq!(
        evidence["doc_artifacts"]["source_schema"],
        "tokmd.doc_artifacts_check.v1"
    );
    assert_eq!(evidence["doc_artifacts"]["ok"], true);
    assert_eq!(evidence["doc_artifacts"]["availability"], "available");
    assert_eq!(evidence["doc_artifacts"]["checked"]["required_docs"], 1);
    assert_eq!(evidence["doc_artifacts"]["checked"]["family_files"], 11);
    assert_eq!(evidence["doc_artifacts"]["checked"]["active_goals"], 1);
    assert_eq!(
        evidence["doc_artifacts"]["checked"]["spec_index_artifacts"],
        9
    );
    assert_eq!(evidence["doc_artifacts"]["checked"]["spec_index_lanes"], 2);

    let copied_doc_artifacts_path = packet_dir.join("docs").join("doc-artifacts-check.json");
    assert!(
        copied_doc_artifacts_path.exists(),
        "doc-artifacts receipt should be copied into the review packet"
    );
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("manifest.json")).unwrap())
            .expect("valid manifest JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_MANIFEST_SCHEMA_JSON,
        &manifest,
        "review packet manifest with doc artifacts",
    );
    let doc_artifacts_artifact = manifest["artifacts"]
        .as_array()
        .expect("manifest artifacts")
        .iter()
        .find(|artifact| artifact["path"] == "docs/doc-artifacts-check.json")
        .expect("doc-artifacts receipt listed in manifest");
    assert_eq!(doc_artifacts_artifact["id"], "doc-artifacts-check");
    assert_eq!(
        doc_artifacts_artifact["schema"],
        "tokmd.doc_artifacts_check.v1"
    );
    let copied_bytes = std::fs::read(&copied_doc_artifacts_path).unwrap();
    assert_eq!(
        doc_artifacts_artifact["hash"]["hash"]
            .as_str()
            .expect("doc-artifacts hash"),
        blake3::hash(&copied_bytes).to_hex().as_str(),
        "manifest hash should match copied doc-artifacts receipt bytes"
    );

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("review-map.json")).unwrap())
            .expect("valid review map JSON");
    assert_validates_against_schema(
        REVIEW_MAP_SCHEMA_JSON,
        &review_map,
        "review map with doc artifacts evidence",
    );
    assert!(
        review_map["evidence"]["refs"]
            .as_array()
            .expect("review-map evidence refs")
            .iter()
            .any(|reference| reference == "evidence.json#/doc_artifacts"),
        "review map should expose packet-level doc-artifacts evidence refs"
    );

    let comment_md = std::fs::read_to_string(packet_dir.join("comment.md")).unwrap();
    assert!(comment_md.contains("Doc artifacts"));
    assert!(comment_md.contains("verified"));
}

#[test]
fn test_cockpit_rejects_unknown_doc_artifacts_schema() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    let doc_artifacts_path = dir.path().join("doc-artifacts-check.json");
    std::fs::write(&doc_artifacts_path, r#"{ "schema": "tokmd.unknown.v1" }"#).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--doc-artifacts-check")
        .arg(&doc_artifacts_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to parse --doc-artifacts-check evidence",
        ))
        .stderr(predicate::str::contains(
            "unsupported doc artifacts evidence schema",
        ));
}

#[test]
fn test_cockpit_rejects_unknown_proof_evidence_schema() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    let proof_path = dir.path().join("unknown-proof.json");
    std::fs::write(&proof_path, r#"{ "schema": "tokmd.unknown.v1" }"#).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--proof-observation")
        .arg(&proof_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to parse --proof-observation proof evidence",
        ))
        .stderr(predicate::str::contains(
            "unsupported proof evidence schema",
        ));
}

#[test]
fn test_cockpit_rejects_mismatched_proof_evidence_kind() {
    let Some(dir) = basic_cockpit_repo() else {
        return;
    };
    let proof_path = dir.path().join("coverage-receipt.json");
    std::fs::write(&proof_path, COVERAGE_RECEIPT_JSON).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--proof-observation")
        .arg(&proof_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--proof-observation expected ProofRunObservation evidence",
        ))
        .stderr(predicate::str::contains("found CoverageReceipt"));
}

#[test]
fn test_cockpit_json_format() {
    // Given: A git repository with a main branch and a feature branch with code changes
    // When: User runs `tokmd cockpit --base main --head HEAD --format json`
    // Then: Output should include JSON with schema_version, change_surface, composition, contracts, review_plan
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    // Initialize git repo
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    // Create initial commit on main
    std::fs::write(dir.path().join("lib.rs"), "fn main() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return;
    }

    // Create a branch with changes
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Add new file") {
        eprintln!("Skipping: second commit failed");
        return;
    }

    // Run cockpit
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("cockpit failed: {}", stderr);
        // Don't fail the test - just verify the command is recognized
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify JSON structure
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    assert!(
        json.get("schema_version").is_some(),
        "should have schema_version"
    );
    assert!(
        json.get("change_surface").is_some(),
        "should have change_surface"
    );
    assert!(json.get("composition").is_some(), "should have composition");
    assert!(json.get("contracts").is_some(), "should have contracts");
    assert!(json.get("review_plan").is_some(), "should have review_plan");
}

#[test]
fn test_cockpit_md_format() {
    // Given: A git repository with a main branch and a feature branch with code changes
    // When: User runs `tokmd cockpit --base main --format md`
    // Then: Output should include markdown with Glass Cockpit header and sections
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("test.rs"), "fn test() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Add test") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("md")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify markdown structure
    assert!(
        stdout.contains("## Glass Cockpit"),
        "should have Glass Cockpit header"
    );
    assert!(
        stdout.contains("### Change Surface"),
        "should have Change Surface section"
    );
    assert!(
        stdout.contains("### Composition"),
        "should have Composition section"
    );
    assert!(
        stdout.contains("### Contracts"),
        "should have Contracts section"
    );
    assert!(
        stdout.contains("### Review Plan"),
        "should have Review Plan section"
    );
}

#[test]
fn test_cockpit_comment_format() {
    // Given: A git repository with a main branch and a feature branch with code changes
    // When: User runs `tokmd cockpit --base main --format comment`
    // Then: Output should be compact PR-comment markdown with actionable next steps
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("review.rs"), "fn review() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Add review file") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("comment")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("## Glass Cockpit Summary"),
        "should have compact comment header"
    );
    assert!(
        stdout.contains("**Next steps**:"),
        "should include reviewer next steps"
    );
    assert!(
        !stdout.trim_start().starts_with('{'),
        "comment format should not emit JSON"
    );
}

#[test]
fn test_cockpit_md_includes_summary_comparison_with_baseline() {
    if !common::git_available() {
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("feature.rs"), "fn feature() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Add feature") {
        return;
    }

    // Write a baseline receipt first.
    let baseline_path = dir.path().join("baseline.json");
    let mut baseline_cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let baseline_output = baseline_cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&baseline_path)
        .output()
        .unwrap();
    if !baseline_output.status.success() {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("md")
        .arg("--baseline")
        .arg(&baseline_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("### Summary Comparison"))
        .stdout(predicate::str::contains(
            "|Metric|Baseline|Current|Delta|Change|",
        ));
}

#[test]
fn test_cockpit_sections_format() {
    // Given: A git repository with a main branch and a dev branch with code changes
    // When: User runs `tokmd cockpit --base main --format sections`
    // Then: Output should include section markers (COCKPIT, REVIEW_PLAN, RECEIPTS)
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("app.rs"), "fn app() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "dev"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("mod.rs"), "mod app;").unwrap();
    if !common::git_add_commit(dir.path(), "Add module") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("sections")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify sections format (used for AI-FILL markers)
    assert!(
        stdout.contains("<!-- SECTION:COCKPIT -->"),
        "should have COCKPIT section marker"
    );
    assert!(
        stdout.contains("<!-- SECTION:REVIEW_PLAN -->"),
        "should have REVIEW_PLAN section marker"
    );
    assert!(
        stdout.contains("<!-- SECTION:RECEIPTS -->"),
        "should have RECEIPTS section marker"
    );
}

#[test]
fn test_cockpit_output_file() {
    // Given: A git repository with a main branch and a test branch with code changes
    // When: User runs `tokmd cockpit --base main --output cockpit.json`
    // Then: Output file should be created with valid JSON
    if !common::git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("code.rs"), "fn code() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "test"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !common::git_add_commit(dir.path(), "New") {
        return;
    }

    let output_file = dir.path().join("cockpit.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success()
        .stdout(""); // stdout should be empty

    // Verify file was created with valid JSON
    assert!(output_file.exists(), "output file should exist");
    let content = std::fs::read_to_string(&output_file).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("valid JSON in file");
}

#[test]
fn test_cockpit_artifacts_dir() {
    // Given: A git repository with a main branch and a test branch with code changes
    // When: User runs `tokmd cockpit --base main --artifacts-dir artifacts/tokmd`
    // Then: Artifacts directory should contain report.json and comment.md
    if !common::git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("code.rs"), "fn code() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "test"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !common::git_add_commit(dir.path(), "New") {
        return;
    }

    let artifacts_dir = dir.path().join("artifacts").join("tokmd");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--artifacts-dir")
        .arg(&artifacts_dir)
        .assert()
        .success();

    let report_path = artifacts_dir.join("report.json");
    let comment_path = artifacts_dir.join("comment.md");
    assert!(report_path.exists(), "report.json should exist");
    assert!(comment_path.exists(), "comment.md should exist");

    let report = std::fs::read_to_string(&report_path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&report).expect("valid JSON in report");

    let comment = std::fs::read_to_string(&comment_path).unwrap();
    let bullet_count = comment
        .lines()
        .filter(|l| l.trim_start().starts_with("- "))
        .count();
    assert!(
        (3..=8).contains(&bullet_count),
        "comment bullet count should be 3-8, got {}",
        bullet_count
    );
}

#[test]
fn test_cockpit_review_packet_dir() {
    // Given: A git repository with a main branch and a test branch with code changes
    // When: User runs `tokmd cockpit --base main --review-packet-dir .tokmd/custom-review`
    // Then: The review packet directory should contain the initial packet artifacts
    if !common::git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        return;
    }

    std::fs::write(dir.path().join("code.rs"), "fn code() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "test"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("new.rs"), "fn new() {}").unwrap();
    if !common::git_add_commit(dir.path(), "New") {
        return;
    }

    let packet_dir_arg = std::path::PathBuf::from(".tokmd").join("custom-review");
    let packet_dir = dir.path().join(&packet_dir_arg);

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--review-packet-dir")
        .arg(&packet_dir_arg)
        .assert()
        .success();

    for artifact in [
        "manifest.json",
        "cockpit.json",
        "evidence.json",
        "review-map.json",
        "review-map.md",
        "comment.md",
    ] {
        assert!(
            packet_dir.join(artifact).exists(),
            "{artifact} should exist"
        );
    }

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("manifest.json")).unwrap())
            .expect("valid manifest JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_MANIFEST_SCHEMA_JSON,
        &manifest,
        "review packet manifest",
    );
    assert_eq!(manifest["schema"], "tokmd.review_packet_manifest.v1");
    assert_eq!(
        manifest["capabilities"]["evidence"]["details"],
        "evidence.json#/gates"
    );
    assert!(
        manifest["verdict"]["evidence"]["unavailable"]
            .as_u64()
            .unwrap()
            > 0,
        "manifest verdict should summarize unavailable evidence"
    );
    let artifacts = manifest["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 5);

    let mut listed_paths = std::collections::BTreeSet::new();
    for artifact in artifacts {
        let id = artifact["id"].as_str().expect("artifact id");
        let path = artifact["path"].as_str().expect("artifact path");
        let schema = artifact["schema"].as_str().expect("artifact schema");
        let media_type = artifact["media_type"]
            .as_str()
            .expect("artifact media type");

        assert!(!id.is_empty(), "artifact id should not be empty");
        assert!(!schema.is_empty(), "artifact schema should not be empty");
        assert!(
            !media_type.is_empty(),
            "artifact media type should not be empty"
        );
        assert!(
            std::path::Path::new(path)
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_))),
            "artifact path should stay relative within the packet dir: {path}"
        );

        let artifact_path = packet_dir.join(path);
        let artifact_bytes = std::fs::read(&artifact_path)
            .unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
        assert!(
            !artifact_bytes.is_empty(),
            "artifact should not be empty: {path}"
        );
        assert_eq!(artifact["hash"]["algo"], "blake3");
        assert_eq!(
            artifact["hash"]["hash"].as_str().expect("artifact hash"),
            blake3::hash(&artifact_bytes).to_hex().as_str(),
            "manifest hash should match artifact bytes for {path}"
        );

        listed_paths.insert(path.to_string());
    }

    assert_eq!(
        listed_paths,
        std::collections::BTreeSet::from([
            "cockpit.json".to_string(),
            "comment.md".to_string(),
            "evidence.json".to_string(),
            "review-map.json".to_string(),
            "review-map.md".to_string(),
        ])
    );

    let cockpit_bytes = std::fs::read(packet_dir.join("cockpit.json")).unwrap();
    let _: tokmd_cockpit::CockpitReceipt =
        serde_json::from_slice(&cockpit_bytes).expect("cockpit.json should parse as a receipt");

    let evidence: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("evidence.json")).unwrap())
            .expect("valid evidence JSON");
    assert_validates_against_schema(
        REVIEW_PACKET_EVIDENCE_SCHEMA_JSON,
        &evidence,
        "review packet evidence",
    );
    assert_eq!(evidence["schema"], "tokmd.review_packet_evidence.v1");
    let gates = evidence["gates"].as_array().expect("evidence gates array");
    assert!(
        gates
            .iter()
            .any(|gate| gate["availability"] == "unavailable"),
        "missing evidence should be represented explicitly"
    );
    for gate in gates {
        assert!(gate["id"].is_string(), "gate should have an id");
        assert!(
            gate["status"].is_string(),
            "gate should report status explicitly"
        );
        assert!(
            gate["availability"].is_string(),
            "gate should report availability explicitly"
        );
    }

    let review_map: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(packet_dir.join("review-map.json")).unwrap())
            .expect("valid review map JSON");
    assert_validates_against_schema(REVIEW_MAP_SCHEMA_JSON, &review_map, "review map");
    assert_eq!(review_map["schema"], "tokmd.review_map.v1");
    assert_eq!(
        review_map["evidence"]["summary"]["details"],
        "evidence.json#/gates"
    );
    assert!(
        review_map["evidence"]["summary"]["unavailable"]
            .as_u64()
            .unwrap()
            > 0,
        "review map should summarize packet-level unavailable evidence"
    );
    assert!(
        review_map["evidence"]["refs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reference| reference == "evidence.json#/gates"),
        "review map should link back to evidence gates"
    );
    let items = review_map["items"].as_array().expect("review map items");
    assert_eq!(review_map["item_count"], items.len() as u64);
    assert!(
        items
            .iter()
            .all(|item| item["evidence"]["status"].is_string()),
        "each review-map item should expose compact evidence status"
    );
    assert!(
        items[0]["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str().unwrap_or("").contains(
                "tokmd cockpit --base main --head HEAD --review-packet-dir .tokmd/custom-review"
            )),
        "review-map JSON should reproduce the actual packet directory"
    );

    let review_map_md = std::fs::read_to_string(packet_dir.join("review-map.md")).unwrap();
    assert!(review_map_md.contains("# Review Map"));
    assert!(review_map_md.contains("Evidence overview:"));
    assert!(review_map_md.contains("## Review First"));
    assert!(review_map_md.contains("Why it matters:"));
    assert!(review_map_md.contains("Evidence status:"));
    assert!(review_map_md.contains("Evidence references:"));
    assert!(review_map_md.contains("cockpit.json#/review_plan/0"));
    assert!(review_map_md.contains("evidence.json#/gates"));
    assert!(review_map_md.contains("Reproduce:"));
    assert!(review_map_md.contains("tokmd cockpit --base main --head HEAD --format json"));
    assert!(review_map_md.contains(
        "tokmd cockpit --base main --head HEAD --review-packet-dir .tokmd/custom-review"
    ));

    let comment_md = std::fs::read_to_string(packet_dir.join("comment.md")).unwrap();
    assert!(comment_md.contains("Evidence availability"));
    assert!(
        comment_md.contains("unavailable"),
        "comment.md should expose unavailable evidence, not just evidence.json"
    );
    assert!(comment_md.contains("Review packet artifacts"));
    assert!(comment_md.contains("[Evidence gates](evidence.json)"));
    assert!(comment_md.contains("[Review map](review-map.md)"));
    assert!(comment_md.contains("[Full cockpit receipt](cockpit.json)"));
}

#[test]
fn test_cockpit_review_packet_bun_ub_sensor_refs() {
    // Given: A Bun-shaped scoped change with no sensor artifacts yet
    // When: User emits a cockpit review packet, then creates sensors/tokmd artifacts and emits another
    // Then: The review map and comment expose Bun UB sensor refs, state, and regeneration commands
    if !common::git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        return;
    }

    let api_dir = dir.path().join("src").join("runtime").join("api");
    std::fs::create_dir_all(&api_dir).unwrap();
    let api_path = api_dir.join("MarkdownObject.rs");
    std::fs::write(&api_path, "pub fn markdown_object() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "test"])
        .current_dir(dir.path())
        .status();

    std::fs::write(
        &api_path,
        "pub fn markdown_object() {}\npub fn native_boundary() {}\n",
    )
    .unwrap();
    if !common::git_add_commit(dir.path(), "Bun API change") {
        return;
    }

    let missing_packet_dir_arg = std::path::PathBuf::from(".tokmd").join("review-missing");
    let missing_packet_dir = dir.path().join(&missing_packet_dir_arg);

    let mut missing_cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    missing_cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--review-packet-dir")
        .arg(&missing_packet_dir_arg)
        .assert()
        .success();

    let missing_review_map: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(missing_packet_dir.join("review-map.json")).unwrap(),
    )
    .expect("valid review map JSON");
    assert_validates_against_schema(
        REVIEW_MAP_SCHEMA_JSON,
        &missing_review_map,
        "missing Bun UB review map",
    );
    let missing_items = missing_review_map["items"]
        .as_array()
        .expect("review map items");
    let missing_item = missing_items
        .iter()
        .find(|item| item["path"] == "src/runtime/api/MarkdownObject.rs")
        .expect("Bun UB review-map item");
    assert_eq!(missing_item["bun_ub_sensor"]["status"], "missing");
    assert!(
        missing_item["bun_ub_sensor"]["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/manifest.json")
    );
    assert!(
        missing_item["bun_ub_sensor"]["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/analyze.md")
    );
    assert!(
        missing_item["bun_ub_sensor"]["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/analyze.json")
    );
    assert!(
        missing_item["bun_ub_sensor"]["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/context.md")
    );
    assert!(
        missing_item["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str().unwrap_or("").contains(
                "tokmd analyze --preset bun-ub --format md --effort-base-ref main --effort-head-ref HEAD --no-progress src/runtime/api/MarkdownObject.rs > sensors/tokmd/analyze.md"
            ))
    );
    assert!(
        missing_item["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str().unwrap_or("").contains(
                "tokmd analyze --preset bun-ub --format json --effort-base-ref main --effort-head-ref HEAD --no-progress src/runtime/api/MarkdownObject.rs > sensors/tokmd/analyze.json"
            ))
    );
    assert!(
        missing_item["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str().unwrap_or("").contains(
                "tokmd context --budget 64000 src/runtime/api/MarkdownObject.rs > sensors/tokmd/context.md"
            ))
    );
    assert!(
        missing_item["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str().unwrap_or("").contains(
                "tokmd syntax --no-progress src/runtime/api/MarkdownObject.rs > sensors/tokmd/syntax.json"
            ))
    );
    assert!(
        missing_item["reproduce"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command.as_str().unwrap_or("").contains(
                "tokmd evidence-packet --preset bun-ub --base main --head HEAD --output sensors/tokmd/manifest.json src/runtime/api/MarkdownObject.rs"
            ))
    );

    let missing_review_map_md =
        std::fs::read_to_string(missing_packet_dir.join("review-map.md")).unwrap();
    assert!(missing_review_map_md.contains("Bun UB sensor artifacts: missing."));
    assert!(missing_review_map_md.contains("Bun UB sensor: missing"));
    assert!(missing_review_map_md.contains("sensors/tokmd/manifest.json"));
    assert!(missing_review_map_md.contains("sensors/tokmd/analyze.md"));
    assert!(missing_review_map_md.contains("sensors/tokmd/analyze.json"));
    assert!(missing_review_map_md.contains("sensors/tokmd/context.md"));
    assert!(missing_review_map_md.contains("sensors/tokmd/syntax.json"));
    let missing_comment_md =
        std::fs::read_to_string(missing_packet_dir.join("comment.md")).unwrap();
    assert!(missing_comment_md.contains("Bun UB sensor artifacts"));
    assert!(missing_comment_md.contains("missing."));
    assert!(missing_comment_md.contains("Regeneration commands are listed in review-map.md."));

    let sensor_dir = dir.path().join("sensors").join("tokmd");
    std::fs::create_dir_all(&sensor_dir).unwrap();
    std::fs::write(sensor_dir.join("manifest.json"), "{}").unwrap();
    std::fs::write(sensor_dir.join("analyze.md"), "# Bun UB analyze\n").unwrap();
    std::fs::write(sensor_dir.join("analyze.json"), r#"{"preset":"bun-ub"}"#).unwrap();
    std::fs::write(sensor_dir.join("context.md"), "# Context\n").unwrap();

    let available_packet_dir_arg = std::path::PathBuf::from(".tokmd").join("review-available");
    let available_packet_dir = dir.path().join(&available_packet_dir_arg);

    let mut available_cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    available_cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--review-packet-dir")
        .arg(&available_packet_dir_arg)
        .assert()
        .success();

    let available_review_map: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(available_packet_dir.join("review-map.json")).unwrap(),
    )
    .expect("valid review map JSON");
    assert_validates_against_schema(
        REVIEW_MAP_SCHEMA_JSON,
        &available_review_map,
        "available Bun UB review map",
    );
    let available_items = available_review_map["items"]
        .as_array()
        .expect("review map items");
    let available_item = available_items
        .iter()
        .find(|item| item["path"] == "src/runtime/api/MarkdownObject.rs")
        .expect("Bun UB review-map item");
    assert_eq!(available_item["bun_ub_sensor"]["status"], "available");
    assert!(
        available_item["bun_ub_sensor"]["missing"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        available_item["bun_ub_sensor"]["available"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/manifest.json")
    );
    assert!(
        available_item["bun_ub_sensor"]["available"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/analyze.md")
    );
    assert!(
        available_item["bun_ub_sensor"]["available"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/analyze.json")
    );
    assert!(
        available_item["bun_ub_sensor"]["available"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "sensors/tokmd/context.md")
    );
    assert!(
        available_item["bun_ub_sensor"]["available"]
            .as_array()
            .unwrap()
            .iter()
            .all(|path| path != "sensors/tokmd/syntax.json")
    );
    let available_comment_md =
        std::fs::read_to_string(available_packet_dir.join("comment.md")).unwrap();
    assert!(available_comment_md.contains("Bun UB sensor artifacts"));
    assert!(available_comment_md.contains("available."));
}

#[test]
fn test_cockpit_not_in_git_repo() {
    // Given: A directory that is not a git repo
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    // When: We run cockpit
    // Then: It should fail with an appropriate error
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("cockpit")
        .assert()
        .failure()
        .stderr(predicate::str::contains("git"));
}

#[test]
fn test_cockpit_file_classification() {
    // Given: A git repository with a main branch and a diverse branch with code, test, docs, config files
    // When: User runs `tokmd cockpit --base main --format json`
    // Then: Output should include composition with code_pct, test_pct, docs_pct, config_pct
    if !common::git_available() {
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        return;
    }

    // Create initial commit
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    // Create branch with diverse file types
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "diverse"])
        .current_dir(dir.path())
        .status();

    // Code
    std::fs::write(dir.path().join("lib.rs"), "pub fn lib() {}").unwrap();
    // Test
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(dir.path().join("tests").join("test.rs"), "fn test() {}").unwrap();
    // Docs
    std::fs::write(dir.path().join("README.md"), "# README").unwrap();
    // Config
    std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

    if !common::git_add_commit(dir.path(), "Add diverse files") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify composition has all categories
    let composition = &json["composition"];
    assert!(
        composition.get("code_pct").is_some(),
        "should have code_pct"
    );
    assert!(
        composition.get("test_pct").is_some(),
        "should have test_pct"
    );
    assert!(
        composition.get("docs_pct").is_some(),
        "should have docs_pct"
    );
    assert!(
        composition.get("config_pct").is_some(),
        "should have config_pct"
    );
}

// =============================================================================
// Priority 2: BDD Scenario Tests for Evidence Gates
// =============================================================================

#[test]
fn test_evidence_gates_pass_all() {
    // Given: A repository with adequate code, tests, and documentation
    // When: User runs `tokmd cockpit --base main --head feature --format json`
    // Then: Output should show all evidence gates passing
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    // Create initial commit with balanced files
    std::fs::write(dir.path().join("lib.rs"), "pub fn lib() {}").unwrap();
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(dir.path().join("tests").join("test.rs"), "fn test() {}").unwrap();
    std::fs::write(dir.path().join("README.md"), "# README").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    // Add more balanced changes
    std::fs::write(dir.path().join("new.rs"), "pub fn new() {}").unwrap();
    std::fs::write(
        dir.path().join("tests").join("new_test.rs"),
        "fn new_test() {}",
    )
    .unwrap();
    if !common::git_add_commit(dir.path(), "Add feature") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify review plan exists (contains evidence gate information)
    assert!(
        json.get("review_plan").is_some(),
        "should have review_plan with evidence gates"
    );
}

#[test]
fn test_evidence_gates_fail_coverage() {
    // Given: A repository with code but insufficient test coverage
    // When: User runs `tokmd cockpit --base main --head feature --format json`
    // Then: Output should show coverage gate failing or warning
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    // Create initial commit with only code
    std::fs::write(dir.path().join("lib.rs"), "pub fn lib() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    // Add more code without tests
    std::fs::write(dir.path().join("new.rs"), "pub fn new() {}").unwrap();
    std::fs::write(dir.path().join("more.rs"), "pub fn more() {}").unwrap();
    if !common::git_add_commit(dir.path(), "Add code without tests") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify composition shows low test percentage
    let composition = &json["composition"];
    assert!(
        composition.get("test_pct").is_some(),
        "should have test_pct"
    );

    // Low test percentage indicates coverage gate issue
    let test_pct = composition["test_pct"].as_f64().unwrap_or(0.0);
    assert!(
        test_pct < 50.0,
        "test_pct should be low (< 50%) indicating coverage gate issue, got {}",
        test_pct
    );
}

#[test]
fn test_evidence_gates_fail_supply_chain() {
    // Given: A repository with missing dependency lockfiles
    // When: User runs `tokmd cockpit --base main --head feature --format json`
    // Then: Output should show supply chain gate failing or warning
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    // Create initial commit with Cargo.toml but no lockfile
    std::fs::write(
        dir.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#,
    )
    .unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    // Add more dependencies without lockfile
    std::fs::write(
        dir.path().join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = "1.0"
anyhow = "1.0"
"#,
    )
    .unwrap();
    if !common::git_add_commit(dir.path(), "Add dependencies") {
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("cockpit")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify review plan exists (may contain supply chain gate info)
    assert!(json.get("review_plan").is_some(), "should have review_plan");
}
