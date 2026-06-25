use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

const TOKMD_PACKETS_SCHEMA_JSON: &str = include_str!("../schemas/tokmd-packets.schema.json");

fn workspace_fixture_bundle() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tokmd-packets/minimal")
        .canonicalize()
        .expect("workspace fixture bundle exists")
}

fn sibling_derived_fixture_bundle() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tokmd-packets/sibling-derived")
        .canonicalize()
        .expect("sibling-derived fixture bundle exists")
}

#[test]
fn render_handoff_preset_from_fixture_bundle() {
    let bundle = workspace_fixture_bundle();
    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.args([
        "render",
        "--from-packets",
        bundle.to_str().expect("utf-8 bundle path"),
        "--preset",
        "bun-ub-handoff",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("## Candidate Identity"))
        .stdout(predicate::str::contains("seed-42"))
        .stdout(predicate::str::contains("## Limitations"))
        .stdout(predicate::str::contains(
            "missing required section: test or witness target",
        ))
        .stdout(predicate::str::contains("## Non-claims"))
        .stdout(predicate::str::contains(
            "Does not prove undefined behavior.",
        ));
}

#[test]
fn render_handoff_derives_sections_from_sibling_bundle_files() {
    let bundle = sibling_derived_fixture_bundle();
    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.args([
        "render",
        "--from-packets",
        bundle.to_str().expect("utf-8 bundle path"),
        "--preset",
        "bun-ub-handoff",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("## Candidate Identity"))
        .stdout(predicate::str::contains("seed-42"))
        .stdout(predicate::str::contains("## Bundle source inputs"))
        .stdout(predicate::str::contains("manual-candidates.json"))
        .stdout(predicate::str::contains("## Limitations"))
        .stdout(predicate::str::contains("## Non-claims"));
}

#[test]
fn render_review_map_ingests_cards_json() {
    let bundle = sibling_derived_fixture_bundle();
    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.args([
        "render",
        "--from-packets",
        bundle.to_str().expect("utf-8 bundle path"),
        "--preset",
        "bun-ub-review-map",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ReviewCard rc-17"))
        .stdout(predicate::str::contains("Explicit No Posting Boundary"))
        .stdout(predicate::str::contains("## Limitations"));
}

#[test]
fn render_absent_preset_inputs_emits_limitation() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = serde_json::json!({
        "schema": "tokmd.packets/v1",
        "inputs_absent": ["manual-candidates.json"],
        "non_claims": ["Does not prove UB."],
        "preset_inputs": {}
    });
    std::fs::write(
        dir.path().join("tokmd-packets.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.args([
        "render",
        "--from-packets",
        dir.path().to_str().unwrap(),
        "--preset",
        "bun-ub-review-map",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("preset_inputs"))
        .stdout(predicate::str::contains("## Limitations"));
}

#[test]
fn render_rejects_invalid_manifest_schema() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = serde_json::json!({
        "schema": "tokmd.packets/v0",
        "non_claims": ["Does not prove UB."]
    });
    std::fs::write(
        dir.path().join("tokmd-packets.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.args([
        "render",
        "--from-packets",
        dir.path().to_str().unwrap(),
        "--preset",
        "bun-ub-handoff",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("failed schema validation"));
}

#[test]
fn render_rejects_unknown_preset() {
    let bundle = workspace_fixture_bundle();
    let mut cmd = Command::cargo_bin("tokmd").unwrap();
    cmd.args([
        "render",
        "--from-packets",
        bundle.to_str().expect("utf-8 bundle path"),
        "--preset",
        "not-a-real-preset",
    ]);
    cmd.assert().failure();
}

#[test]
fn fixture_manifest_matches_schema_id() {
    let manifest_path = workspace_fixture_bundle().join("tokmd-packets.json");
    let raw = std::fs::read_to_string(&manifest_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(value["schema"], "tokmd.packets/v1");

    let schema: serde_json::Value = serde_json::from_str(TOKMD_PACKETS_SCHEMA_JSON).unwrap();
    let compiled = jsonschema::validator_for(&schema).expect("valid schema");
    let output = compiled.validate(&value);
    assert!(
        output.is_ok(),
        "fixture manifest failed schema validation: {output:?}"
    );
}
