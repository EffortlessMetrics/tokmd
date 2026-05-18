#![cfg(feature = "git")]
mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

#[test]
fn test_handoff_creates_expected_files() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_output");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .success();

    // Verify core artifacts exist
    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("map.jsonl").exists());
    assert!(out_dir.join("intelligence.json").exists());
    assert!(out_dir.join("code.txt").exists());
    assert!(out_dir.join("work-order.md").exists());
}

#[test]
fn test_handoff_manifest_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_json");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .success();

    let manifest_content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&manifest_content).expect("manifest.json should be valid JSON");

    // Verify required fields
    assert_eq!(parsed["schema_version"].as_u64(), Some(5));
    assert!(parsed["generated_at_ms"].is_number());
    assert!(parsed["tool"]["name"].as_str() == Some("tokmd"));
    assert!(parsed["mode"].as_str() == Some("handoff"));
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["output_dir"].is_string());
    assert!(parsed["capabilities"].is_array());
    assert!(parsed["artifacts"].is_array());
    assert!(parsed["included_files"].is_array());
    assert!(parsed["excluded_paths"].is_array());
    assert!(parsed["excluded_patterns"].is_array());

    let artifacts = parsed["artifacts"].as_array().unwrap();
    let map = artifacts.iter().find(|a| a["name"] == "map").unwrap();
    assert!(map["hash"]["algo"] == "blake3");
    assert!(map["hash"]["hash"].is_string());
    let work_order = artifacts
        .iter()
        .find(|a| a["name"] == "work-order")
        .unwrap();
    assert_eq!(work_order["path"].as_str(), Some("work-order.md"));
    assert_eq!(work_order["hash"]["algo"].as_str(), Some("blake3"));
}

#[test]
fn test_handoff_links_review_and_proof_artifacts() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_links");
    let review_dir = dir.path().join("review");
    let proof_dir = dir.path().join("proof");
    fs::create_dir_all(&review_dir).unwrap();
    fs::create_dir_all(&proof_dir).unwrap();

    fs::write(review_dir.join("comment.md"), "summary").unwrap();
    fs::write(review_dir.join("review-map.md"), "review map").unwrap();
    fs::write(
        review_dir.join("review-map.json"),
        r#"{
          "schema":"tokmd.review_map.v1",
          "item_count":1,
          "evidence":{
            "summary":{
              "available":2,
              "missing":1,
              "degraded":0,
              "stale":0,
              "skipped":0,
              "unavailable":1
            }
          },
          "items":[
            {
              "path":"docs/handoff.md",
              "reason":"handoff behavior changed"
            }
          ]
        }"#,
    )
    .unwrap();
    fs::write(review_dir.join("evidence.json"), "{}").unwrap();
    fs::write(review_dir.join("manifest.json"), "{}").unwrap();
    fs::write(review_dir.join("cockpit.json"), "{}").unwrap();

    let review_check = proof_dir.join("review-packet-check.json");
    let affected = proof_dir.join("affected.json");
    let proof_plan = proof_dir.join("proof-plan.json");
    fs::write(
        &review_check,
        r#"{"schema":"tokmd.review_packet_check.v1","ok":true,"artifact_count":7,"hashes_verified":7}"#,
    )
    .unwrap();
    fs::write(
        &affected,
        r#"{
          "schema":"tokmd.affected.v1",
          "changed_files":["docs/handoff.md"],
          "scopes":[{"name":"user_guides"}],
          "unknown_files":["docs/unrouted.md"]
        }"#,
    )
    .unwrap();
    fs::write(
        &proof_plan,
        r#"{
          "schema":"tokmd.proof_plan.v1",
          "commands":[
            {"command":"cargo xtask docs --check","required":true},
            {"command":"cargo xtask proof --profile affected --plan","required":false}
          ]
        }"#,
    )
    .unwrap();

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--review-packet-dir")
        .arg(&review_dir)
        .arg("--review-packet-check")
        .arg(&review_check)
        .arg("--affected")
        .arg(&affected)
        .arg("--proof-plan")
        .arg(&proof_plan)
        .assert()
        .success();

    let review_links: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("review-links.json")).unwrap())
            .unwrap();
    assert_eq!(
        review_links["schema"].as_str(),
        Some("tokmd.handoff_review_links.v1")
    );
    assert_eq!(review_links["semantics"]["copied"].as_bool(), Some(false));
    assert_eq!(
        review_links["review_packet_check"]["exists"].as_bool(),
        Some(true)
    );
    assert!(
        review_links["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "review_map_md" && entry["exists"] == true)
    );

    let proof_links: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("proof-links.json")).unwrap())
            .unwrap();
    assert_eq!(
        proof_links["schema"].as_str(),
        Some("tokmd.handoff_proof_links.v1")
    );
    assert!(
        proof_links["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == "proof_plan" && entry["exists"] == true)
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("manifest.json")).unwrap()).unwrap();
    let artifacts = manifest["artifacts"].as_array().unwrap();
    assert!(
        artifacts
            .iter()
            .any(|entry| entry["path"] == "review-links.json")
    );
    assert!(
        artifacts
            .iter()
            .any(|entry| entry["path"] == "proof-links.json")
    );

    let work_order = fs::read_to_string(out_dir.join("work-order.md")).unwrap();
    assert!(work_order.contains("review-links.json"));
    assert!(work_order.contains("proof-links.json"));
    assert!(!work_order.contains("Read `work-order.md`"));
    assert!(
        work_order.contains("Treat linked review and proof receipts as external evidence handles")
    );
    assert!(work_order.contains("## Linked Evidence Summary"));
    assert!(work_order.contains("Review packet verifier: ok=true"));
    assert!(work_order.contains("Review map: 1 item(s)"));
    assert!(work_order.contains("`docs/handoff.md`: handoff behavior changed"));
    assert_sections_in_order(
        &work_order,
        &[
            "## Changed Surfaces",
            "## Linked Evidence",
            "## Linked Evidence Summary",
            "## Review Evidence",
            "## Proof Expectations",
            "## Missing / Stale / Degraded Evidence",
            "## Included Files",
            "## Agent Stop Conditions",
            "## Agent Guardrails",
        ],
    );
    assert!(work_order.contains("Changed files to inspect first:"));
    assert!(work_order.contains("Review packet verifier: linked and ok."));
    assert!(work_order.contains("Run expected proof before claiming done:"));
    assert!(work_order.contains("Review evidence missing: 1"));
    assert!(work_order.contains("Review evidence unavailable: 1"));
    assert!(
        work_order.contains("Affected proof: 1 changed file(s), 1 scope(s), 1 unknown file(s)")
    );
    assert!(work_order.contains("Proof plan: 2 command(s), 1 required, 1 advisory"));
    assert!(work_order.contains("A proof plan is planned evidence, not execution proof."));
    assert!(work_order.contains(
        "Affected proof has 1 unknown file(s); update proof routing before trusting scoped proof."
    ));
    assert!(
        work_order.contains(
            "Stop if the linked review-packet verifier is missing, unreadable, or failing."
        )
    );
    assert!(
        work_order
            .contains("Stop before claiming proof if affected routing still has unknown files.")
    );
    assert!(work_order.contains(
        "Stop before claiming done until required proof commands are run or explicitly deferred."
    ));
    assert!(
        work_order.contains(
            "Treat missing, stale, degraded, skipped, or unavailable evidence as work to resolve, not as passing proof."
        )
    );
}

fn assert_sections_in_order(content: &str, sections: &[&str]) {
    let mut previous = 0;
    for section in sections {
        let index = content[previous..]
            .find(section)
            .map(|offset| previous + offset)
            .unwrap_or_else(|| panic!("missing section `{section}`"));
        assert!(
            index >= previous,
            "section `{section}` appeared before the previous section"
        );
        previous = index;
    }
}

#[test]
fn test_handoff_intelligence_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_intel");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .success();

    let intel_content = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&intel_content).expect("intelligence.json should be valid JSON");

    // Verify required fields
    assert!(parsed["tree"].is_string());
    assert!(parsed["tree_depth"].is_number());
    assert!(parsed["warnings"].is_array());
    assert!(parsed.get("schema_version").is_none());
    assert!(parsed.get("generated_at_ms").is_none());
    assert!(parsed.get("capabilities").is_none());
}

#[test]
fn test_handoff_excludes_output_dir_from_scan() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();

    let out_dir = root.join(".handoff");
    fs::create_dir_all(&out_dir).unwrap();
    fs::write(out_dir.join("should_skip.rs"), "fn skip() {}").unwrap();

    let mut cmd = tokmd_cmd();
    cmd.current_dir(root);
    cmd.arg("handoff")
        .arg(root)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--force")
        .assert()
        .success();

    let manifest_content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();
    let excluded = parsed["excluded_paths"].as_array().unwrap();
    assert!(
        excluded.iter().any(|p| p["path"] == ".handoff"),
        "manifest should record output dir exclusion"
    );

    let map_content = fs::read_to_string(out_dir.join("map.jsonl")).unwrap();
    assert!(
        !map_content.contains("should_skip.rs"),
        "map should not include files from output directory"
    );
}

#[test]
fn test_handoff_budget_enforcement() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_budget");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--budget")
        .arg("1k") // Small budget
        .assert()
        .success();

    let manifest_content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();

    // Verify budget not exceeded
    assert!(
        used <= budget,
        "used_tokens ({}) should not exceed budget_tokens ({})",
        used,
        budget
    );
}

#[test]
fn test_handoff_graceful_no_git() {
    // Run with --no-git flag
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_no_git");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-git")
        .assert()
        .success();

    // Verify all artifacts still created
    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("map.jsonl").exists());
    assert!(out_dir.join("intelligence.json").exists());
    assert!(out_dir.join("code.txt").exists());
    assert!(out_dir.join("work-order.md").exists());

    // Verify capabilities show git as skipped
    let manifest_content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();

    let caps = parsed["capabilities"].as_array().unwrap();
    let git_cap = caps.iter().find(|c| c["name"] == "git").unwrap();
    assert_eq!(git_cap["status"], "skipped");
}

#[test]
fn test_handoff_directory_already_exists_without_force() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_exists");

    // First run - should succeed
    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .success();

    // Second run without --force - should fail
    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not empty").or(predicate::str::contains("--force")));
}

#[test]
fn test_handoff_force_overwrites() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_force");

    // First run
    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .success();

    // Second run with --force - should succeed
    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn test_handoff_preset_minimal() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_minimal");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--preset")
        .arg("minimal")
        .assert()
        .success();

    let intel_content = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&intel_content).unwrap();

    // Minimal preset should have tree but not complexity or derived
    assert!(parsed["tree"].is_string());
    assert!(parsed["complexity"].is_null());
    assert!(parsed["derived"].is_null());
}

#[test]
fn test_handoff_preset_standard() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_standard");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--preset")
        .arg("standard")
        .assert()
        .success();

    let intel_content = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&intel_content).unwrap();

    // Standard preset should have tree, complexity, and derived
    assert!(parsed["tree"].is_string());
    assert!(parsed["complexity"].is_object());
    assert!(parsed["derived"].is_object());
}

#[test]
fn test_handoff_map_jsonl_format() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_map");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .success();

    let map_content = fs::read_to_string(out_dir.join("map.jsonl")).unwrap();

    // Each line should be valid JSON
    for line in map_content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let _parsed: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Invalid JSON line: {}\nLine: {}", e, line));
    }
}

#[test]
fn test_handoff_code_txt_has_content() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_code");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .assert()
        .success();

    let code_content = fs::read_to_string(out_dir.join("code.txt")).unwrap();

    // code.txt should have file markers
    assert!(
        code_content.contains("// ==="),
        "code.txt should contain file markers"
    );
}

#[test]
fn test_handoff_compress_strips_blanks() {
    let dir = tempdir().unwrap();
    let out_dir_normal = dir.path().join("handoff_normal");
    let out_dir_compress = dir.path().join("handoff_compress");

    // Run without compress
    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir_normal)
        .assert()
        .success();

    // Run with compress
    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir_compress)
        .arg("--compress")
        .assert()
        .success();

    let normal_size = fs::metadata(out_dir_normal.join("code.txt")).unwrap().len();
    let compress_size = fs::metadata(out_dir_compress.join("code.txt"))
        .unwrap()
        .len();

    // Compressed should be smaller or equal (depends on blank line count in source)
    assert!(
        compress_size <= normal_size,
        "Compressed ({}) should be <= normal ({})",
        compress_size,
        normal_size
    );
}

#[test]
fn test_handoff_strategy_greedy() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_greedy");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--strategy")
        .arg("greedy")
        .assert()
        .success();

    let manifest_content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();

    assert_eq!(parsed["strategy"].as_str(), Some("greedy"));
}

#[test]
fn test_handoff_strategy_spread() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_spread");

    let mut cmd = tokmd_cmd();
    cmd.arg("handoff")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--strategy")
        .arg("spread")
        .assert()
        .success();

    let manifest_content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();

    assert_eq!(parsed["strategy"].as_str(), Some("spread"));
}
