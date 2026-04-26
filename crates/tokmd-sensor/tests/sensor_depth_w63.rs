//! Depth tests (w63) for tokmd-sensor: EffortlessSensor trait, substrate builder,
//! report lifecycle, capability reporting, multi-sensor composition, determinism,
//! and property-based verification.

use std::collections::BTreeMap;

use anyhow::Result;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA,
    SensorReport, ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ===========================================================================
// Helper: build substrate fixtures
// ===========================================================================

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines: code + 20,
        bytes: code * 30,
        tokens: code * 7,
        module: path
            .rsplit_once('/')
            .map(|(m, _)| m)
            .unwrap_or("")
            .to_string(),
        in_diff,
    }
}

fn make_lang(files: usize, code: usize) -> LangSummary {
    LangSummary {
        files,
        code,
        lines: code + files * 20,
        bytes: code * 30,
        tokens: code * 7,
    }
}

fn empty_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/empty".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    }
}

fn single_file_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![make_file("src/lib.rs", "Rust", 100, false)],
        lang_summary: {
            let mut m = BTreeMap::new();
            m.insert("Rust".to_string(), make_lang(1, 100));
            m
        },
        diff_range: None,
        total_tokens: 700,
        total_bytes: 3000,
        total_code_lines: 100,
    }
}

fn multi_lang_substrate() -> RepoSubstrate {
    let files = vec![
        make_file("src/lib.rs", "Rust", 200, true),
        make_file("src/main.rs", "Rust", 80, false),
        make_file("app/index.ts", "TypeScript", 150, true),
        make_file("app/util.ts", "TypeScript", 60, false),
        make_file("scripts/build.py", "Python", 40, false),
    ];
    let total_code: usize = files.iter().map(|f| f.code).sum();
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_bytes: usize = files.iter().map(|f| f.bytes).sum();
    let mut lang_summary = BTreeMap::new();
    lang_summary.insert("Rust".to_string(), make_lang(2, 280));
    lang_summary.insert("TypeScript".to_string(), make_lang(2, 210));
    lang_summary.insert("Python".to_string(), make_lang(1, 40));
    RepoSubstrate {
        repo_root: "/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "app/index.ts".to_string()],
            commit_count: 5,
            insertions: 30,
            deletions: 10,
        }),
        total_tokens,
        total_bytes,
        total_code_lines: total_code,
    }
}

// ===========================================================================
// Test sensor implementations
// ===========================================================================

#[derive(Serialize, Deserialize)]
struct ThresholdSettings {
    threshold: usize,
}

/// A threshold-based sensor that warns when code lines exceed the threshold.
struct ThresholdSensor;

impl EffortlessSensor for ThresholdSensor {
    type Settings = ThresholdSettings;
    fn name(&self) -> &str {
        "threshold"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn run(&self, settings: &ThresholdSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let verdict = if substrate.total_code_lines > settings.threshold {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2025-06-01T00:00:00Z".to_string(),
            verdict,
            format!(
                "{} code lines vs threshold {}",
                substrate.total_code_lines, settings.threshold
            ),
        ))
    }
}

/// A sensor that emits no-op/empty settings.
#[derive(Serialize, Deserialize)]
struct UnitSettings;

/// A sensor that always passes with no findings.
struct NoOpSensor;

impl EffortlessSensor for NoOpSensor {
    type Settings = UnitSettings;
    fn name(&self) -> &str {
        "noop"
    }
    fn version(&self) -> &str {
        "0.0.1"
    }
    fn run(&self, _s: &UnitSettings, _sub: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2025-06-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "No-op".to_string(),
        ))
    }
}

/// A sensor that always returns an error.
struct ErrorSensor;

impl EffortlessSensor for ErrorSensor {
    type Settings = UnitSettings;
    fn name(&self) -> &str {
        "error-sensor"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn run(&self, _s: &UnitSettings, _sub: &RepoSubstrate) -> Result<SensorReport> {
        anyhow::bail!("sensor failed intentionally")
    }
}

/// A sensor that produces findings based on diff files.
struct DiffFindingSensor;

#[derive(Serialize, Deserialize)]
struct DiffSettings {
    max_diff_files: usize,
}

impl EffortlessSensor for DiffFindingSensor {
    type Settings = DiffSettings;
    fn name(&self) -> &str {
        "diff-finder"
    }
    fn version(&self) -> &str {
        "2.0.0"
    }
    fn run(&self, settings: &DiffSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let diff_files: Vec<_> = substrate.diff_files().collect();
        let verdict = if diff_files.len() > settings.max_diff_files {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "diff-check"),
            "2025-06-01T00:00:00Z".to_string(),
            verdict,
            format!("{} diff files", diff_files.len()),
        );
        for f in &diff_files {
            report.add_finding(Finding::new(
                "diff",
                "changed_file",
                FindingSeverity::Info,
                "File changed",
                format!("{} was modified", f.path),
            ));
        }
        Ok(report)
    }
}

/// A sensor that attaches capabilities and artifacts.
struct FullSensor;

impl EffortlessSensor for FullSensor {
    type Settings = UnitSettings;
    fn name(&self) -> &str {
        "full-sensor"
    }
    fn version(&self) -> &str {
        "3.0.0"
    }
    fn run(&self, _s: &UnitSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let mut caps = BTreeMap::new();
        caps.insert("loc_count".to_string(), CapabilityStatus::available());
        if substrate.diff_range.is_some() {
            caps.insert("diff_analysis".to_string(), CapabilityStatus::available());
        } else {
            caps.insert(
                "diff_analysis".to_string(),
                CapabilityStatus::skipped("no diff range"),
            );
        }
        caps.insert(
            "coverage".to_string(),
            CapabilityStatus::unavailable("no coverage tool"),
        );

        let report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "full"),
            "2025-06-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "Full analysis".to_string(),
        )
        .with_capabilities(caps)
        .with_artifacts(vec![
            Artifact::receipt("output/receipt.json").with_id("main"),
            Artifact::badge("output/badge.svg"),
        ])
        .with_data(serde_json::json!({
            "total_code": substrate.total_code_lines,
            "languages": substrate.lang_summary.len(),
        }));
        Ok(report)
    }
}

/// A sensor that returns Skip verdict for empty repos.
struct SkipOnEmptySensor;

impl EffortlessSensor for SkipOnEmptySensor {
    type Settings = UnitSettings;
    fn name(&self) -> &str {
        "skip-empty"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn run(&self, _s: &UnitSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        if substrate.files.is_empty() {
            Ok(SensorReport::new(
                ToolMeta::new(self.name(), self.version(), "check"),
                "2025-06-01T00:00:00Z".to_string(),
                Verdict::Skip,
                "No files to analyze".to_string(),
            ))
        } else {
            Ok(SensorReport::new(
                ToolMeta::new(self.name(), self.version(), "check"),
                "2025-06-01T00:00:00Z".to_string(),
                Verdict::Pass,
                format!("{} files analyzed", substrate.files.len()),
            ))
        }
    }
}

// ===========================================================================
// 1. Sensor trait implementation verification
// ===========================================================================

#[test]
fn sensor_name_returns_expected() {
    assert_eq!(ThresholdSensor.name(), "threshold");
    assert_eq!(NoOpSensor.name(), "noop");
    assert_eq!(ErrorSensor.name(), "error-sensor");
    assert_eq!(DiffFindingSensor.name(), "diff-finder");
    assert_eq!(FullSensor.name(), "full-sensor");
}

#[test]
fn sensor_version_returns_expected() {
    assert_eq!(ThresholdSensor.version(), "1.0.0");
    assert_eq!(NoOpSensor.version(), "0.0.1");
    assert_eq!(ErrorSensor.version(), "0.1.0");
    assert_eq!(DiffFindingSensor.version(), "2.0.0");
    assert_eq!(FullSensor.version(), "3.0.0");
}

#[test]
fn sensor_name_is_nonempty() {
    let sensors: Vec<&str> = vec![
        ThresholdSensor.name(),
        NoOpSensor.name(),
        ErrorSensor.name(),
        DiffFindingSensor.name(),
        FullSensor.name(),
        SkipOnEmptySensor.name(),
    ];
    for name in sensors {
        assert!(!name.is_empty(), "sensor name must not be empty");
    }
}

#[test]
fn sensor_version_is_semver_like() {
    let versions = vec![
        ThresholdSensor.version(),
        NoOpSensor.version(),
        ErrorSensor.version(),
        DiffFindingSensor.version(),
        FullSensor.version(),
    ];
    for v in versions {
        let parts: Vec<&str> = v.split('.').collect();
        assert_eq!(parts.len(), 3, "version must have 3 parts: {v}");
        for p in &parts {
            assert!(p.parse::<u32>().is_ok(), "version part not numeric: {p}");
        }
    }
}

#[test]
fn threshold_sensor_pass_when_below() {
    let s = ThresholdSensor;
    let settings = ThresholdSettings { threshold: 500 };
    let report = s.run(&settings, &single_file_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn threshold_sensor_warn_when_above() {
    let s = ThresholdSensor;
    let settings = ThresholdSettings { threshold: 50 };
    let report = s.run(&settings, &single_file_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
}

#[test]
fn threshold_sensor_pass_at_exact_boundary() {
    let s = ThresholdSensor;
    let settings = ThresholdSettings { threshold: 100 };
    let report = s.run(&settings, &single_file_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

// ===========================================================================
// 2. Report generation lifecycle
// ===========================================================================

#[test]
fn report_has_schema_field() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn report_has_tool_meta() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    assert_eq!(report.tool.name, "noop");
    assert_eq!(report.tool.version, "0.0.1");
    assert_eq!(report.tool.mode, "check");
}

#[test]
fn report_has_timestamp() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    assert!(!report.generated_at.is_empty());
    assert!(report.generated_at.contains('T'));
}

#[test]
fn report_has_summary() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    assert!(!report.summary.is_empty());
}

#[test]
fn report_findings_initially_empty_for_noop() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn report_serde_roundtrip() {
    let report = ThresholdSensor
        .run(
            &ThresholdSettings { threshold: 10 },
            &multi_lang_substrate(),
        )
        .unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, report.schema);
    assert_eq!(back.verdict, report.verdict);
    assert_eq!(back.summary, report.summary);
    assert_eq!(back.tool.name, report.tool.name);
}

#[test]
fn report_json_contains_required_fields() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val.get("schema").is_some());
    assert!(val.get("tool").is_some());
    assert!(val.get("generated_at").is_some());
    assert!(val.get("verdict").is_some());
    assert!(val.get("summary").is_some());
    assert!(val.get("findings").is_some());
}

#[test]
fn report_optional_fields_absent_when_none() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val.get("artifacts").is_none());
    assert!(val.get("capabilities").is_none());
    assert!(val.get("data").is_none());
}

// ===========================================================================
// 3. Capability reporting
// ===========================================================================

#[test]
fn full_sensor_reports_capabilities_with_diff() {
    let report = FullSensor
        .run(&UnitSettings, &multi_lang_substrate())
        .unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["loc_count"].status, CapabilityState::Available);
    assert_eq!(caps["diff_analysis"].status, CapabilityState::Available);
    assert_eq!(caps["coverage"].status, CapabilityState::Unavailable);
}

#[test]
fn full_sensor_skips_diff_without_range() {
    let report = FullSensor
        .run(&UnitSettings, &single_file_substrate())
        .unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["diff_analysis"].status, CapabilityState::Skipped);
}

#[test]
fn capability_unavailable_has_reason() {
    let report = FullSensor
        .run(&UnitSettings, &single_file_substrate())
        .unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert!(caps["coverage"].reason.is_some());
    assert!(
        caps["coverage"]
            .reason
            .as_ref()
            .unwrap()
            .contains("coverage")
    );
}

#[test]
fn capability_skipped_has_reason() {
    let report = FullSensor
        .run(&UnitSettings, &single_file_substrate())
        .unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    let diff_cap = &caps["diff_analysis"];
    assert_eq!(diff_cap.status, CapabilityState::Skipped);
    assert!(diff_cap.reason.is_some());
}

#[test]
fn capability_available_has_no_reason() {
    let report = FullSensor
        .run(&UnitSettings, &multi_lang_substrate())
        .unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert!(caps["loc_count"].reason.is_none());
}

#[test]
fn capabilities_serialized_in_json() {
    let report = FullSensor
        .run(&UnitSettings, &multi_lang_substrate())
        .unwrap();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    let caps = val.get("capabilities").unwrap().as_object().unwrap();
    assert!(caps.contains_key("loc_count"));
    assert!(caps.contains_key("diff_analysis"));
    assert!(caps.contains_key("coverage"));
}

// ===========================================================================
// 4. Artifacts in reports
// ===========================================================================

#[test]
fn full_sensor_includes_artifacts() {
    let report = FullSensor
        .run(&UnitSettings, &multi_lang_substrate())
        .unwrap();
    let arts = report.artifacts.as_ref().unwrap();
    assert_eq!(arts.len(), 2);
}

#[test]
fn artifact_receipt_has_id() {
    let report = FullSensor
        .run(&UnitSettings, &multi_lang_substrate())
        .unwrap();
    let arts = report.artifacts.as_ref().unwrap();
    let receipt = arts.iter().find(|a| a.artifact_type == "receipt").unwrap();
    assert_eq!(receipt.id.as_deref(), Some("main"));
    assert_eq!(receipt.path, "output/receipt.json");
}

#[test]
fn artifact_badge_has_no_id() {
    let report = FullSensor
        .run(&UnitSettings, &multi_lang_substrate())
        .unwrap();
    let arts = report.artifacts.as_ref().unwrap();
    let badge = arts.iter().find(|a| a.artifact_type == "badge").unwrap();
    assert!(badge.id.is_none());
}

// ===========================================================================
// 5. Data payload
// ===========================================================================

#[test]
fn full_sensor_data_payload_present() {
    let report = FullSensor
        .run(&UnitSettings, &multi_lang_substrate())
        .unwrap();
    let data = report.data.as_ref().unwrap();
    assert!(data.get("total_code").is_some());
    assert!(data.get("languages").is_some());
}

#[test]
fn full_sensor_data_matches_substrate() {
    let sub = multi_lang_substrate();
    let report = FullSensor.run(&UnitSettings, &sub).unwrap();
    let data = report.data.as_ref().unwrap();
    assert_eq!(
        data["total_code"].as_u64().unwrap(),
        sub.total_code_lines as u64
    );
    assert_eq!(
        data["languages"].as_u64().unwrap(),
        sub.lang_summary.len() as u64
    );
}

// ===========================================================================
// 6. Empty scan input handling
// ===========================================================================

#[test]
fn noop_sensor_on_empty_substrate() {
    let report = NoOpSensor.run(&UnitSettings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn threshold_sensor_pass_on_empty_substrate() {
    let report = ThresholdSensor
        .run(&ThresholdSettings { threshold: 0 }, &empty_substrate())
        .unwrap();
    // 0 is not > 0, so Pass
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn skip_empty_sensor_skips_on_empty() {
    let report = SkipOnEmptySensor
        .run(&UnitSettings, &empty_substrate())
        .unwrap();
    assert_eq!(report.verdict, Verdict::Skip);
    assert!(report.summary.contains("No files"));
}

#[test]
fn skip_empty_sensor_passes_on_nonempty() {
    let report = SkipOnEmptySensor
        .run(&UnitSettings, &single_file_substrate())
        .unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn diff_sensor_on_empty_substrate_no_findings() {
    let report = DiffFindingSensor
        .run(&DiffSettings { max_diff_files: 0 }, &empty_substrate())
        .unwrap();
    assert!(report.findings.is_empty());
    assert_eq!(report.verdict, Verdict::Pass);
}

// ===========================================================================
// 7. Sensor error handling
// ===========================================================================

#[test]
fn error_sensor_returns_err() {
    let result = ErrorSensor.run(&UnitSettings, &empty_substrate());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("intentionally"));
}

#[test]
fn error_sensor_on_nonempty_still_errors() {
    let result = ErrorSensor.run(&UnitSettings, &multi_lang_substrate());
    assert!(result.is_err());
}

// ===========================================================================
// 8. Diff-based findings
// ===========================================================================

#[test]
fn diff_sensor_produces_findings_for_changed_files() {
    let report = DiffFindingSensor
        .run(&DiffSettings { max_diff_files: 0 }, &multi_lang_substrate())
        .unwrap();
    assert_eq!(report.findings.len(), 2);
    assert_eq!(report.verdict, Verdict::Warn);
}

#[test]
fn diff_sensor_finding_messages_reference_paths() {
    let report = DiffFindingSensor
        .run(
            &DiffSettings { max_diff_files: 10 },
            &multi_lang_substrate(),
        )
        .unwrap();
    let messages: Vec<&str> = report.findings.iter().map(|f| f.message.as_str()).collect();
    assert!(messages.iter().any(|m| m.contains("src/lib.rs")));
    assert!(messages.iter().any(|m| m.contains("app/index.ts")));
}

#[test]
fn diff_sensor_passes_when_within_limit() {
    let report = DiffFindingSensor
        .run(
            &DiffSettings { max_diff_files: 10 },
            &multi_lang_substrate(),
        )
        .unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn diff_sensor_no_diff_files_when_no_range() {
    let report = DiffFindingSensor
        .run(
            &DiffSettings { max_diff_files: 0 },
            &single_file_substrate(),
        )
        .unwrap();
    assert!(report.findings.is_empty());
    assert_eq!(report.verdict, Verdict::Pass);
}

// ===========================================================================
// 9. Multi-sensor composition
// ===========================================================================

#[test]
fn multiple_sensors_share_substrate() {
    let sub = multi_lang_substrate();
    let r1 = ThresholdSensor
        .run(&ThresholdSettings { threshold: 1000 }, &sub)
        .unwrap();
    let r2 = NoOpSensor.run(&UnitSettings, &sub).unwrap();
    let r3 = DiffFindingSensor
        .run(&DiffSettings { max_diff_files: 10 }, &sub)
        .unwrap();
    assert_eq!(r1.verdict, Verdict::Pass);
    assert_eq!(r2.verdict, Verdict::Pass);
    assert_eq!(r3.verdict, Verdict::Pass);
}

#[test]
fn multi_sensor_mixed_verdicts() {
    let sub = multi_lang_substrate();
    let r1 = ThresholdSensor
        .run(&ThresholdSettings { threshold: 10 }, &sub)
        .unwrap();
    let r2 = NoOpSensor.run(&UnitSettings, &sub).unwrap();
    assert_eq!(r1.verdict, Verdict::Warn);
    assert_eq!(r2.verdict, Verdict::Pass);
}

#[test]
fn multi_sensor_all_pass_on_empty() {
    let sub = empty_substrate();
    let r1 = ThresholdSensor
        .run(&ThresholdSettings { threshold: 100 }, &sub)
        .unwrap();
    let r2 = NoOpSensor.run(&UnitSettings, &sub).unwrap();
    let r3 = DiffFindingSensor
        .run(&DiffSettings { max_diff_files: 5 }, &sub)
        .unwrap();
    assert_eq!(r1.verdict, Verdict::Pass);
    assert_eq!(r2.verdict, Verdict::Pass);
    assert_eq!(r3.verdict, Verdict::Pass);
}

#[test]
fn multi_sensor_aggregate_worst_verdict() {
    let sub = multi_lang_substrate();
    let reports: Vec<SensorReport> = vec![
        ThresholdSensor
            .run(&ThresholdSettings { threshold: 10 }, &sub)
            .unwrap(),
        NoOpSensor.run(&UnitSettings, &sub).unwrap(),
        SkipOnEmptySensor.run(&UnitSettings, &sub).unwrap(),
    ];
    let has_warn = reports.iter().any(|r| r.verdict == Verdict::Warn);
    assert!(has_warn, "at least one sensor should warn");
}

#[test]
fn multi_sensor_total_findings() {
    let sub = multi_lang_substrate();
    let r1 = DiffFindingSensor
        .run(&DiffSettings { max_diff_files: 0 }, &sub)
        .unwrap();
    let r2 = NoOpSensor.run(&UnitSettings, &sub).unwrap();
    let total_findings = r1.findings.len() + r2.findings.len();
    assert_eq!(total_findings, 2);
}

// ===========================================================================
// 10. Determinism: same scan produces same sensor report
// ===========================================================================

#[test]
fn determinism_threshold_sensor() {
    let sub = multi_lang_substrate();
    let settings = ThresholdSettings { threshold: 100 };
    let r1 = serde_json::to_string(&ThresholdSensor.run(&settings, &sub).unwrap()).unwrap();
    let r2 = serde_json::to_string(&ThresholdSensor.run(&settings, &sub).unwrap()).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn determinism_noop_sensor() {
    let sub = empty_substrate();
    let r1 = serde_json::to_string(&NoOpSensor.run(&UnitSettings, &sub).unwrap()).unwrap();
    let r2 = serde_json::to_string(&NoOpSensor.run(&UnitSettings, &sub).unwrap()).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn determinism_diff_sensor() {
    let sub = multi_lang_substrate();
    let settings = DiffSettings { max_diff_files: 5 };
    let r1 = serde_json::to_string(&DiffFindingSensor.run(&settings, &sub).unwrap()).unwrap();
    let r2 = serde_json::to_string(&DiffFindingSensor.run(&settings, &sub).unwrap()).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn determinism_full_sensor() {
    let sub = multi_lang_substrate();
    let r1 = serde_json::to_string(&FullSensor.run(&UnitSettings, &sub).unwrap()).unwrap();
    let r2 = serde_json::to_string(&FullSensor.run(&UnitSettings, &sub).unwrap()).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn determinism_skip_empty_sensor() {
    let sub = empty_substrate();
    let r1 = serde_json::to_string(&SkipOnEmptySensor.run(&UnitSettings, &sub).unwrap()).unwrap();
    let r2 = serde_json::to_string(&SkipOnEmptySensor.run(&UnitSettings, &sub).unwrap()).unwrap();
    assert_eq!(r1, r2);
}

// ===========================================================================
// 11. Settings serialization
// ===========================================================================

#[test]
fn settings_serde_roundtrip_threshold() {
    let s = ThresholdSettings { threshold: 42 };
    let json = serde_json::to_string(&s).unwrap();
    let back: ThresholdSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.threshold, 42);
}

#[test]
fn settings_serde_roundtrip_diff() {
    let s = DiffSettings { max_diff_files: 7 };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_diff_files, 7);
}

#[test]
fn settings_serde_roundtrip_unit() {
    let s = UnitSettings;
    let json = serde_json::to_string(&s).unwrap();
    let back: UnitSettings = serde_json::from_str(&json).unwrap();
    let _ = back; // unit type, just verify roundtrip
}

// ===========================================================================
// 12. Property tests
// ===========================================================================

proptest! {
    #[test]
    fn prop_report_always_has_schema(threshold in 0usize..10000) {
        let sub = single_file_substrate();
        let report = ThresholdSensor.run(&ThresholdSettings { threshold }, &sub).unwrap();
        prop_assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    }

    #[test]
    fn prop_report_always_has_tool_name(threshold in 0usize..10000) {
        let sub = single_file_substrate();
        let report = ThresholdSensor.run(&ThresholdSettings { threshold }, &sub).unwrap();
        prop_assert!(!report.tool.name.is_empty());
    }

    #[test]
    fn prop_report_verdict_is_pass_or_warn(threshold in 0usize..10000) {
        let sub = single_file_substrate();
        let report = ThresholdSensor.run(&ThresholdSettings { threshold }, &sub).unwrap();
        prop_assert!(report.verdict == Verdict::Pass || report.verdict == Verdict::Warn);
    }

    #[test]
    fn prop_report_summary_not_empty(threshold in 0usize..10000) {
        let sub = single_file_substrate();
        let report = ThresholdSensor.run(&ThresholdSettings { threshold }, &sub).unwrap();
        prop_assert!(!report.summary.is_empty());
    }

    #[test]
    fn prop_report_serde_roundtrip(threshold in 0usize..10000) {
        let sub = single_file_substrate();
        let report = ThresholdSensor.run(&ThresholdSettings { threshold }, &sub).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.verdict, report.verdict);
        prop_assert_eq!(back.schema, report.schema);
    }

    #[test]
    fn prop_report_json_has_required_keys(threshold in 0usize..10000) {
        let sub = single_file_substrate();
        let report = ThresholdSensor.run(&ThresholdSettings { threshold }, &sub).unwrap();
        let val: serde_json::Value = serde_json::to_value(&report).unwrap();
        prop_assert!(val.get("schema").is_some());
        prop_assert!(val.get("tool").is_some());
        prop_assert!(val.get("verdict").is_some());
        prop_assert!(val.get("summary").is_some());
        prop_assert!(val.get("findings").is_some());
    }

    #[test]
    fn prop_determinism(threshold in 0usize..10000) {
        let sub = single_file_substrate();
        let settings = ThresholdSettings { threshold };
        let r1 = serde_json::to_string(&ThresholdSensor.run(&settings, &sub).unwrap()).unwrap();
        let r2 = serde_json::to_string(&ThresholdSensor.run(&settings, &sub).unwrap()).unwrap();
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn prop_diff_findings_count_matches_diff_files(max in 0usize..20) {
        let sub = multi_lang_substrate();
        let report = DiffFindingSensor.run(&DiffSettings { max_diff_files: max }, &sub).unwrap();
        let diff_count = sub.diff_files().count();
        prop_assert_eq!(report.findings.len(), diff_count);
    }
}
