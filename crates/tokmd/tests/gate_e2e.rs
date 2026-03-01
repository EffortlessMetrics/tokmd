//! End-to-end tests for the `tokmd gate` CLI command.

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::PathBuf;

/// Build a `tokmd` command rooted at the test fixtures directory.
fn tokmd() -> Command {
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    cmd.current_dir(&fixtures);
    cmd
}

// ── 1. No rules → error ──────────────────────────────────────────────

#[test]
fn gate_no_rules_fails() {
    tokmd()
        .arg("gate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No policy or ratchet rules"));
}

// ── 2. Policy pass (lenient threshold) ───────────────────────────────

#[test]
fn gate_policy_pass_lenient_threshold() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 999999999
"#,
    )
    .unwrap();

    tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Gate PASSED"));
}

// ── 3. Policy fail (impossible threshold) ────────────────────────────

#[test]
fn gate_policy_fail_impossible_threshold() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    // Code lines must be > 99999999 — impossible for tiny fixture.
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "min_code"
pointer = "/derived/totals/code"
op = "gte"
value = 99999999
"#,
    )
    .unwrap();

    tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Gate FAILED"));
}

// ── 4. JSON output with passing policy ───────────────────────────────

#[test]
fn gate_json_output_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 999999999
"#,
    )
    .unwrap();

    let output = tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["passed"], true);
    assert_eq!(json["total_errors"], 0);
    assert!(json["policy"].is_object());
    assert!(json["policy"]["rule_results"].is_array());
}

// ── 5. JSON output with failing policy ───────────────────────────────

#[test]
fn gate_json_output_fail() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "impossible_code"
pointer = "/derived/totals/code"
op = "gte"
value = 99999999
message = "Not enough code"
"#,
    )
    .unwrap();

    let output = tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["passed"], false);
    assert!(json["total_errors"].as_u64().unwrap() >= 1);

    // Verify rule_results structure
    let results = json["policy"]["rule_results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "impossible_code");
    assert_eq!(results[0]["passed"], false);
    assert_eq!(results[0]["message"], "Not enough code");
}

// ── 6. Multiple rules (mix of pass and fail) ─────────────────────────

#[test]
fn gate_multiple_rules_mixed() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "lenient_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 999999999

[[rules]]
name = "impossible_code"
pointer = "/derived/totals/code"
op = "gte"
value = 99999999
"#,
    )
    .unwrap();

    let output = tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    // Overall should fail because one rule fails.
    assert!(!output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["passed"], false);

    let results = json["policy"]["rule_results"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    // First rule passes, second fails.
    assert_eq!(results[0]["passed"], true);
    assert_eq!(results[1]["passed"], false);
}

// ── 7. Warning-level rule does not fail the gate ─────────────────────

#[test]
fn gate_warn_level_does_not_fail() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "soft_limit"
pointer = "/derived/totals/code"
op = "gte"
value = 99999999
level = "warn"
"#,
    )
    .unwrap();

    let output = tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["passed"], true);
    assert_eq!(json["total_errors"], 0);
    assert!(json["total_warnings"].as_u64().unwrap() >= 1);
}

// ── 8. Fail-fast stops after first error ─────────────────────────────

#[test]
fn gate_fail_fast_stops_early() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
fail_fast = true

[[rules]]
name = "first_fail"
pointer = "/derived/totals/code"
op = "gte"
value = 99999999

[[rules]]
name = "second_should_not_run"
pointer = "/derived/totals/code"
op = "gte"
value = 99999999
"#,
    )
    .unwrap();

    let output = tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // fail_fast should stop after first error → only 1 result.
    let results = json["policy"]["rule_results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "first_fail");
}

// ── 9. Policy from pre-computed JSON receipt ─────────────────────────

#[test]
fn gate_from_precomputed_receipt() {
    let tmp = tempfile::tempdir().unwrap();

    // First, generate a JSON receipt via `tokmd analyze`.
    let receipt_path = tmp.path().join("receipt.json");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    let output = cargo_bin_cmd!("tokmd")
        .current_dir(&fixtures)
        .arg("analyze")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(output.status.success());
    std::fs::write(&receipt_path, &output.stdout).unwrap();

    // Now gate against the pre-computed receipt.
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "has_derived"
pointer = "/derived"
op = "exists"
"#,
    )
    .unwrap();

    cargo_bin_cmd!("tokmd")
        .arg("gate")
        .arg(&receipt_path)
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Gate PASSED"));
}

// ── 10. Text output contains rule details on failure ─────────────────

#[test]
fn gate_text_output_shows_rule_details() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "custom_msg_rule"
pointer = "/derived/totals/code"
op = "gte"
value = 99999999
message = "Code is too small"
"#,
    )
    .unwrap();

    tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("FAIL")
                .and(predicate::str::contains("custom_msg_rule"))
                .and(predicate::str::contains("Code is too small")),
        );
}

// ── 11. Exists operator check ────────────────────────────────────────

#[test]
fn gate_exists_operator() {
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(
        &policy_path,
        r#"
[[rules]]
name = "derived_exists"
pointer = "/derived"
op = "exists"

[[rules]]
name = "no_bogus"
pointer = "/totally_nonexistent_field"
op = "exists"
negate = true
"#,
    )
    .unwrap();

    let output = tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["passed"], true);
    let results = json["policy"]["rule_results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r["passed"] == true));
}

// ── 12. Invalid policy file path → falls through to "no rules" error ─

#[test]
fn gate_invalid_policy_path_fails() {
    // When --policy points to a missing file, load_policy errors out
    // and the code falls through to the "No policy or ratchet rules" bail.
    tokmd()
        .arg("gate")
        .arg("--policy")
        .arg("nonexistent_policy.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No policy or ratchet rules"));
}
