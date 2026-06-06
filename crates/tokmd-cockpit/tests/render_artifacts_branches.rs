//! Branch-coverage tests for `tokmd_cockpit::render::write_artifacts` and
//! (under the `git` feature) `write_sensor_artifacts`.
//!
//! `write_artifacts` maps `evidence.overall_status` to an envelope
//! `Verdict` for `report.json`. Of the five `GateStatus` arms only
//! `Pass` and `Skipped` were exercised by existing tests
//! (`cargo llvm-cov` reported `Fail`, `Warn`, `Pending` at 0 hits each).
//! `write_sensor_artifacts` was not exercised at all.
//!
//! These tests assert the artifact files appear on disk and that
//! `report.json` carries the expected `verdict` string for each
//! `GateStatus`.

use std::fs;

use tempfile::TempDir;

use tokmd_cockpit::render::write_sensor_artifacts;
use tokmd_cockpit::render::{render_comment_md, write_artifacts};
use tokmd_cockpit::*;
use tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION;

fn base_meta() -> GateMeta {
    GateMeta {
        status: GateStatus::Pass,
        source: EvidenceSource::RanLocal,
        commit_match: CommitMatch::Exact,
        scope: ScopeCoverage {
            relevant: vec![],
            tested: vec![],
            ratio: 1.0,
            lines_relevant: None,
            lines_tested: None,
        },
        evidence_commit: None,
        evidence_generated_at_ms: None,
    }
}

fn base_mutation() -> MutationGate {
    MutationGate {
        meta: GateMeta {
            status: GateStatus::Skipped,
            ..base_meta()
        },
        survivors: vec![],
        killed: 0,
        timeout: 0,
        unviable: 0,
    }
}

fn receipt_with_status(status: GateStatus) -> CockpitReceipt {
    CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 0,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
            files_changed: 2,
            insertions: 30,
            deletions: 5,
            net_lines: 25,
            churn_velocity: 0.0,
            change_concentration: 0.0,
        },
        composition: Composition {
            code_pct: 1.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        },
        code_health: CodeHealth {
            score: 80,
            grade: "B".to_string(),
            large_files_touched: 0,
            avg_file_size: 100,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Medium,
            score: 25,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: Evidence {
            overall_status: status,
            mutation: base_mutation(),
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan: vec![],
        trend: None,
    }
}

fn report_verdict(dir: &std::path::Path) -> String {
    let raw = fs::read_to_string(dir.join("report.json")).expect("report.json readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("report.json is valid json");
    v["verdict"]
        .as_str()
        .expect("verdict is string")
        .to_string()
}

// ---------------------------------------------------------------------------
// write_artifacts — verdict mapping for each GateStatus
// ---------------------------------------------------------------------------

#[test]
fn write_artifacts_creates_three_files() {
    let tmp = TempDir::new().unwrap();
    let receipt = receipt_with_status(GateStatus::Pass);

    write_artifacts(tmp.path(), &receipt).expect("write_artifacts succeeds");

    assert!(tmp.path().join("cockpit.json").is_file());
    assert!(tmp.path().join("report.json").is_file());
    assert!(tmp.path().join("comment.md").is_file());
}

#[test]
fn write_artifacts_pass_status_maps_to_pass_verdict() {
    let tmp = TempDir::new().unwrap();
    write_artifacts(tmp.path(), &receipt_with_status(GateStatus::Pass)).unwrap();

    assert_eq!(report_verdict(tmp.path()), "pass");
}

#[test]
fn write_artifacts_fail_status_maps_to_fail_verdict() {
    let tmp = TempDir::new().unwrap();
    write_artifacts(tmp.path(), &receipt_with_status(GateStatus::Fail)).unwrap();

    assert_eq!(report_verdict(tmp.path()), "fail");
}

#[test]
fn write_artifacts_warn_status_maps_to_warn_verdict() {
    let tmp = TempDir::new().unwrap();
    write_artifacts(tmp.path(), &receipt_with_status(GateStatus::Warn)).unwrap();

    assert_eq!(report_verdict(tmp.path()), "warn");
}

#[test]
fn write_artifacts_skipped_status_maps_to_skip_verdict() {
    let tmp = TempDir::new().unwrap();
    write_artifacts(tmp.path(), &receipt_with_status(GateStatus::Skipped)).unwrap();

    assert_eq!(report_verdict(tmp.path()), "skip");
}

#[test]
fn write_artifacts_pending_status_maps_to_pending_verdict() {
    let tmp = TempDir::new().unwrap();
    write_artifacts(tmp.path(), &receipt_with_status(GateStatus::Pending)).unwrap();

    assert_eq!(report_verdict(tmp.path()), "pending");
}

// ---------------------------------------------------------------------------
// write_artifacts — content shape
// ---------------------------------------------------------------------------

#[test]
fn write_artifacts_cockpit_json_matches_receipt_schema_version() {
    let tmp = TempDir::new().unwrap();
    let receipt = receipt_with_status(GateStatus::Pass);
    write_artifacts(tmp.path(), &receipt).unwrap();

    let raw = fs::read_to_string(tmp.path().join("cockpit.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        v["schema_version"].as_u64(),
        Some(COCKPIT_SCHEMA_VERSION as u64)
    );
    assert_eq!(v["mode"], "cockpit");
}

#[test]
fn write_artifacts_comment_md_matches_render_comment_md() {
    let tmp = TempDir::new().unwrap();
    let receipt = receipt_with_status(GateStatus::Warn);
    write_artifacts(tmp.path(), &receipt).unwrap();

    let from_file = fs::read_to_string(tmp.path().join("comment.md")).unwrap();
    let direct = render_comment_md(&receipt);
    assert_eq!(from_file, direct);
}

#[test]
fn write_artifacts_report_summary_mentions_files_changed_and_refs() {
    let tmp = TempDir::new().unwrap();
    let mut receipt = receipt_with_status(GateStatus::Pass);
    receipt.change_surface.files_changed = 7;
    receipt.change_surface.insertions = 42;
    receipt.change_surface.deletions = 3;
    receipt.code_health.score = 91;
    receipt.risk.level = RiskLevel::High;
    receipt.base_ref = "v1.0".to_string();
    receipt.head_ref = "v1.1".to_string();

    write_artifacts(tmp.path(), &receipt).unwrap();

    let raw = fs::read_to_string(tmp.path().join("report.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let summary = v["summary"].as_str().expect("summary is string");

    assert!(summary.contains("7 files changed"), "summary: {summary}");
    assert!(summary.contains("+42/-3"), "summary: {summary}");
    assert!(summary.contains("health 91/100"), "summary: {summary}");
    assert!(summary.contains("risk high"), "summary: {summary}");
    assert!(summary.contains("v1.0..v1.1"), "summary: {summary}");
}

#[test]
fn write_artifacts_creates_missing_output_directory() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("nested").join("artifacts");
    assert!(!nested.exists());

    write_artifacts(&nested, &receipt_with_status(GateStatus::Pass)).unwrap();

    assert!(nested.join("cockpit.json").is_file());
}

// ---------------------------------------------------------------------------
// write_sensor_artifacts — feature-gated git path
// ---------------------------------------------------------------------------

fn sensor_report_verdict(dir: &std::path::Path) -> String {
    let raw = fs::read_to_string(dir.join("report.json")).expect("report.json readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    v["verdict"]
        .as_str()
        .expect("verdict is string")
        .to_string()
}

#[test]
fn write_sensor_artifacts_emits_report_json_with_base_head_summary() {
    let tmp = TempDir::new().unwrap();
    let receipt = receipt_with_status(GateStatus::Pass);

    write_sensor_artifacts(tmp.path(), &receipt, "main", "HEAD").unwrap();

    let raw = fs::read_to_string(tmp.path().join("report.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let summary = v["summary"].as_str().unwrap();
    assert!(
        summary.contains("Cockpit run for main..HEAD"),
        "summary: {summary}"
    );
    assert_eq!(v["verdict"].as_str(), Some("pass"));
}

#[test]
fn write_sensor_artifacts_fail_status_maps_to_fail_verdict() {
    let tmp = TempDir::new().unwrap();
    write_sensor_artifacts(
        tmp.path(),
        &receipt_with_status(GateStatus::Fail),
        "base",
        "head",
    )
    .unwrap();
    assert_eq!(sensor_report_verdict(tmp.path()), "fail");
}

#[test]
fn write_sensor_artifacts_warn_status_maps_to_warn_verdict() {
    let tmp = TempDir::new().unwrap();
    write_sensor_artifacts(
        tmp.path(),
        &receipt_with_status(GateStatus::Warn),
        "base",
        "head",
    )
    .unwrap();
    assert_eq!(sensor_report_verdict(tmp.path()), "warn");
}

#[test]
fn write_sensor_artifacts_skipped_status_maps_to_skip_verdict() {
    let tmp = TempDir::new().unwrap();
    write_sensor_artifacts(
        tmp.path(),
        &receipt_with_status(GateStatus::Skipped),
        "base",
        "head",
    )
    .unwrap();
    assert_eq!(sensor_report_verdict(tmp.path()), "skip");
}

#[test]
fn write_sensor_artifacts_pending_status_maps_to_pending_verdict() {
    let tmp = TempDir::new().unwrap();
    write_sensor_artifacts(
        tmp.path(),
        &receipt_with_status(GateStatus::Pending),
        "base",
        "head",
    )
    .unwrap();
    assert_eq!(sensor_report_verdict(tmp.path()), "pending");
}

#[test]
fn write_sensor_artifacts_creates_missing_output_directory() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("nested").join("sensor");
    write_sensor_artifacts(
        &nested,
        &receipt_with_status(GateStatus::Pass),
        "main",
        "HEAD",
    )
    .unwrap();
    assert!(nested.join("report.json").is_file());
}
