#![cfg(feature = "git")]
//! Deep CLI integration tests for complex commands: context, handoff, sensor,
//! baseline, diff.

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

// ===========================================================================
// Context command
// ===========================================================================

#[test]
fn context_list_mode_produces_file_paths() {
    tokmd_cmd()
        .args(["context", "--mode", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn context_json_mode_produces_valid_json_with_schema_version() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .expect("context --mode json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    assert_eq!(json["schema_version"].as_u64(), Some(4));
    assert_eq!(json["mode"].as_str(), Some("context"));
    assert!(json["files"].is_array());
    assert!(json["tool"]["name"].as_str() == Some("tokmd"));
}

#[test]
fn context_budget_respects_token_limit() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json", "--budget", "1000"])
        .output()
        .expect("context --budget 1000");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let budget = json["budget_tokens"].as_u64().unwrap();
    let used = json["used_tokens"].as_u64().unwrap();
    assert_eq!(budget, 1000);
    assert!(
        used <= budget,
        "used_tokens ({used}) should not exceed budget_tokens ({budget})"
    );
}

#[test]
fn context_invalid_strategy_fails_gracefully() {
    tokmd_cmd()
        .args(["context", "--strategy", "nonexistent_strategy_xyz"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("invalid").or(predicate::str::contains("possible values")),
        );
}

#[test]
fn context_help_exits_zero() {
    tokmd_cmd()
        .args(["context", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("context"));
}

#[test]
fn context_json_files_have_path_and_tokens() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let files = json["files"].as_array().unwrap();
    assert!(!files.is_empty());
    for f in files {
        assert!(f["path"].is_string(), "file row needs path");
        assert!(f["tokens"].is_number(), "file row needs tokens");
    }
}

#[test]
fn context_json_utilization_within_bounds() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let pct = json["utilization_pct"].as_f64().unwrap();
    assert!(
        (0.0..=100.0).contains(&pct),
        "utilization_pct out of range: {pct}"
    );
}

#[test]
fn context_bundle_mode_produces_output() {
    tokmd_cmd()
        .args(["context", "--mode", "bundle"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// Handoff command
// ===========================================================================

#[test]
fn handoff_produces_manifest_and_artifacts() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("map.jsonl").exists());
    assert!(out_dir.join("intelligence.json").exists());
    assert!(out_dir.join("code.txt").exists());
}

#[test]
fn handoff_manifest_is_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_json_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid manifest JSON");

    assert_eq!(json["schema_version"].as_u64(), Some(5));
    assert_eq!(json["mode"].as_str(), Some("handoff"));
    assert!(json["budget_tokens"].is_number());
    assert!(json["used_tokens"].is_number());
    assert!(json["artifacts"].is_array());
    assert!(json["included_files"].is_array());
}

#[test]
fn handoff_preset_minimal_works() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_minimal_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .args(["--preset", "minimal"])
        .assert()
        .success();

    let intel = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&intel).unwrap();
    assert!(json["tree"].is_string());
    // minimal preset should not have complexity or derived
    assert!(json["complexity"].is_null());
    assert!(json["derived"].is_null());
}

#[test]
fn handoff_preset_standard_works() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_standard_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .args(["--preset", "standard"])
        .assert()
        .success();

    let intel = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&intel).unwrap();
    assert!(json["tree"].is_string());
    assert!(json["complexity"].is_object());
    assert!(json["derived"].is_object());
}

#[test]
fn handoff_budget_respects_token_limits() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_budget_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .args(["--budget", "1k"])
        .assert()
        .success();

    let content = fs::read_to_string(out_dir.join("manifest.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let budget = json["budget_tokens"].as_u64().unwrap();
    let used = json["used_tokens"].as_u64().unwrap();
    assert!(used <= budget, "used ({used}) exceeds budget ({budget})");
}

#[test]
fn handoff_help_exits_zero() {
    tokmd_cmd()
        .args(["handoff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("handoff"));
}

#[test]
fn handoff_map_jsonl_each_line_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_jsonl_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let map = fs::read_to_string(out_dir.join("map.jsonl")).unwrap();
    for line in map.lines().filter(|l| !l.trim().is_empty()) {
        let _: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Invalid JSONL line: {e}\nLine: {line}"));
    }
}

#[test]
fn handoff_force_flag_allows_overwrite() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_force_w48");

    // First run
    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    // Second run with --force
    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn handoff_without_force_on_existing_dir_fails() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_noforce_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not empty").or(predicate::str::contains("--force")));
}

// ===========================================================================
// Sensor command (requires git feature + git availability)
// ===========================================================================

#[cfg(feature = "git")]
mod sensor_tests {
    use super::*;

    fn setup_git_repo_with_feature_branch() -> Option<tempfile::TempDir> {
        if !crate::common::git_available() {
            return None;
        }

        let dir = tempdir().unwrap();
        if !crate::common::init_git_repo(dir.path()) {
            return None;
        }

        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
        if !crate::common::git_add_commit(dir.path(), "Initial commit") {
            return None;
        }

        let status = std::process::Command::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(dir.path())
            .status()
            .ok()?;
        if !status.success() {
            return None;
        }

        fs::write(
            dir.path().join("src/lib.rs"),
            "fn main() { println!(\"hello\"); }\n",
        )
        .unwrap();
        fs::write(dir.path().join("src/extra.rs"), "fn extra() {}\n").unwrap();
        if !crate::common::git_add_commit(dir.path(), "Feature changes") {
            return None;
        }

        Some(dir)
    }

    #[test]
    fn sensor_produces_report_v1_envelope() {
        let Some(dir) = setup_git_repo_with_feature_branch() else {
            eprintln!("Skipping: git setup failed");
            return;
        };

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        let output = cmd
            .current_dir(dir.path())
            .args([
                "sensor", "--base", "main", "--head", "HEAD", "--format", "json",
            ])
            .output()
            .expect("sensor json");

        if !output.status.success() {
            panic!("sensor failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("valid JSON");

        assert_eq!(json["schema"], "sensor.report.v1");
    }

    #[test]
    fn sensor_output_is_valid_json() {
        let Some(dir) = setup_git_repo_with_feature_branch() else {
            eprintln!("Skipping: git setup failed");
            return;
        };

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        let output = cmd
            .current_dir(dir.path())
            .args([
                "sensor", "--base", "main", "--head", "HEAD", "--format", "json",
            ])
            .output()
            .expect("sensor json");

        assert!(output.status.success());
        let _: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
            .expect("stdout should be valid JSON");
    }

    #[test]
    fn sensor_report_has_required_envelope_fields() {
        let Some(dir) = setup_git_repo_with_feature_branch() else {
            eprintln!("Skipping: git setup failed");
            return;
        };

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        let output = cmd
            .current_dir(dir.path())
            .args([
                "sensor", "--base", "main", "--head", "HEAD", "--format", "json",
            ])
            .output()
            .expect("sensor json");

        assert!(output.status.success());
        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

        assert_eq!(json["schema"], "sensor.report.v1");
        assert_eq!(json["tool"]["name"], "tokmd");
        assert!(json.get("data").is_some(), "envelope must have data");
        assert!(
            json.get("artifacts").is_some(),
            "envelope must have artifacts"
        );
        assert!(json.get("verdict").is_some(), "envelope must have verdict");
        assert!(
            json.get("findings").is_some(),
            "envelope must have findings"
        );
    }

    #[test]
    fn sensor_artifacts_include_receipt_and_cockpit() {
        let Some(dir) = setup_git_repo_with_feature_branch() else {
            eprintln!("Skipping: git setup failed");
            return;
        };

        let output_path = dir
            .path()
            .join("artifacts")
            .join("tokmd")
            .join("report.json");

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        let output = cmd
            .current_dir(dir.path())
            .args(["sensor", "--base", "main", "--head", "HEAD", "--output"])
            .arg(&output_path)
            .args(["--format", "json"])
            .output()
            .expect("sensor with output");

        assert!(output.status.success());
        let json: serde_json::Value =
            serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

        let artifacts = json["artifacts"].as_array().expect("artifacts array");
        let ids: Vec<&str> = artifacts.iter().filter_map(|a| a["id"].as_str()).collect();
        assert!(ids.contains(&"receipt"), "missing receipt artifact");
        assert!(ids.contains(&"cockpit"), "missing cockpit artifact");
    }

    #[test]
    fn sensor_md_format_produces_markdown() {
        let Some(dir) = setup_git_repo_with_feature_branch() else {
            eprintln!("Skipping: git setup failed");
            return;
        };

        let output_path = dir.path().join("artifacts/tokmd/report.json");

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        let output = cmd
            .current_dir(dir.path())
            .args(["sensor", "--base", "main", "--head", "HEAD", "--output"])
            .arg(&output_path)
            .args(["--format", "md"])
            .output()
            .expect("sensor md");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("## Sensor Report: tokmd"),
            "md format should contain report header"
        );
    }

    #[test]
    fn sensor_help_exits_zero() {
        Command::new(env!("CARGO_BIN_EXE_tokmd"))
            .args(["sensor", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("sensor"));
    }
}

// ===========================================================================
// Baseline command
// ===========================================================================

#[test]
fn baseline_generates_valid_json_file() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("baseline.json");

    tokmd_cmd()
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&out_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid baseline JSON");

    assert_eq!(json["baseline_version"].as_u64(), Some(1));
    assert!(
        json.get("metrics").is_some(),
        "baseline should have metrics"
    );
}

#[test]
fn baseline_output_has_expected_metrics() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("baseline_metrics.json");

    tokmd_cmd()
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&out_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let metrics = &json["metrics"];
    assert!(metrics.is_object(), "metrics should be an object");
}

#[test]
fn baseline_help_exits_zero() {
    tokmd_cmd()
        .args(["baseline", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("baseline"));
}

#[test]
fn baseline_force_overwrites_existing() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("baseline_force.json");

    // First run
    tokmd_cmd()
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    // Second run with --force should also succeed
    tokmd_cmd()
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&out_file).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("valid JSON after overwrite");
}

#[test]
fn baseline_without_determinism_has_no_determinism_field() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("baseline_nodet.json");

    tokmd_cmd()
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&out_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(
        json.get("determinism").is_none(),
        "determinism should be absent without --determinism flag"
    );
}

// ===========================================================================
// Diff command
// ===========================================================================

#[test]
fn diff_no_args_shows_error() {
    tokmd_cmd().arg("diff").assert().failure().stderr(
        predicate::str::contains("--from")
            .or(predicate::str::contains("Provide"))
            .or(predicate::str::contains("refs/paths")),
    );
}

#[test]
fn diff_same_receipt_twice_shows_zero_diff() {
    let dir = tempdir().unwrap();
    let run_dir = dir.path().join("run_for_diff");

    // Generate a run
    tokmd_cmd()
        .args(["run", "--output-dir"])
        .arg(&run_dir)
        .arg(".")
        .assert()
        .success();

    let receipt = run_dir.join("receipt.json");
    assert!(receipt.exists());

    // Diff same file against itself
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("diff")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"));
}

#[test]
fn diff_json_format_produces_valid_json() {
    let dir = tempdir().unwrap();
    let run_dir = dir.path().join("run_diff_json");

    tokmd_cmd()
        .args(["run", "--output-dir"])
        .arg(&run_dir)
        .arg(".")
        .assert()
        .success();

    let receipt = run_dir.join("receipt.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .args(["diff", "--format", "json", "--from"])
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .output()
        .expect("diff json");

    assert!(output.status.success());
    let _: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .expect("diff JSON output should be valid");
}

#[test]
fn diff_help_exits_zero() {
    tokmd_cmd()
        .args(["diff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("diff"));
}

#[test]
fn diff_compact_mode_works() {
    let dir = tempdir().unwrap();
    let run_dir = dir.path().join("run_compact");

    tokmd_cmd()
        .args(["run", "--output-dir"])
        .arg(&run_dir)
        .arg(".")
        .assert()
        .success();

    let receipt = run_dir.join("receipt.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.arg("diff")
        .arg("--compact")
        .arg("--from")
        .arg(&receipt)
        .arg("--to")
        .arg(&receipt)
        .assert()
        .success()
        .stdout(predicate::str::contains("|Metric|Value|"));
}

// ===========================================================================
// General: all subcommands accept --help
// ===========================================================================

#[test]
fn all_complex_commands_accept_help() {
    for subcmd in ["context", "handoff", "baseline", "diff"] {
        tokmd_cmd().args([subcmd, "--help"]).assert().success();
    }
}

// ===========================================================================
// General: invalid flags produce helpful errors
// ===========================================================================

#[test]
fn context_invalid_mode_produces_error() {
    tokmd_cmd()
        .args(["context", "--mode", "invalid_mode_xyz"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("invalid").or(predicate::str::contains("possible values")),
        );
}

#[test]
fn handoff_invalid_preset_produces_error() {
    let dir = tempdir().unwrap();
    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(dir.path().join("bad"))
        .args(["--preset", "nonexistent_preset"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("invalid").or(predicate::str::contains("possible values")),
        );
}

#[test]
fn diff_invalid_format_produces_error() {
    tokmd_cmd()
        .args(["diff", "--format", "bogus_format"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("invalid").or(predicate::str::contains("possible values")),
        );
}

// ===========================================================================
// JSON validity across commands
// ===========================================================================

#[test]
fn context_json_output_is_parseable() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let _: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("valid JSON");
}

#[test]
fn baseline_json_output_is_parseable() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("bl_parse.json");

    tokmd_cmd()
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = fs::read_to_string(&out_file).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).expect("valid baseline JSON");
}

#[test]
fn handoff_intelligence_json_is_parseable() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("handoff_parse_w48");

    tokmd_cmd()
        .args(["handoff", "--out-dir"])
        .arg(&out_dir)
        .assert()
        .success();

    let content = fs::read_to_string(out_dir.join("intelligence.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert!(json["tree"].is_string());
}
