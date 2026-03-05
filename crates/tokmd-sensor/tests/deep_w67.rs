//! Deep tests for tokmd-sensor: EffortlessSensor trait + substrate builder (W67)

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingLocation, FindingSeverity, SensorReport, ToolMeta,
    Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_file(path: &str, lang: &str, code: usize, in_diff: bool) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines: code + 20,
        bytes: code * 30,
        tokens: code * 8,
        module: path
            .rsplit_once('/')
            .map(|(m, _)| m)
            .unwrap_or(".")
            .to_string(),
        in_diff,
    }
}

fn make_substrate(files: Vec<SubstrateFile>, diff: Option<DiffRange>) -> RepoSubstrate {
    let mut lang_summary: BTreeMap<String, LangSummary> = BTreeMap::new();
    for f in &files {
        let e = lang_summary.entry(f.lang.clone()).or_insert(LangSummary {
            files: 0,
            code: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        });
        e.files += 1;
        e.code += f.code;
        e.lines += f.lines;
        e.bytes += f.bytes;
        e.tokens += f.tokens;
    }
    let total_tokens = files.iter().map(|f| f.tokens).sum();
    let total_bytes = files.iter().map(|f| f.bytes).sum();
    let total_code_lines = files.iter().map(|f| f.code).sum();
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files,
        lang_summary,
        diff_range: diff,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

fn sample_substrate() -> RepoSubstrate {
    make_substrate(
        vec![
            make_file("src/lib.rs", "Rust", 100, false),
            make_file("src/main.rs", "Rust", 50, true),
            make_file("tests/test.py", "Python", 30, false),
        ],
        None,
    )
}

// ---------------------------------------------------------------------------
// Test sensors
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct ThresholdSettings {
    max_code_lines: usize,
}

struct LineSensor;

impl EffortlessSensor for LineSensor {
    type Settings = ThresholdSettings;
    fn name(&self) -> &str {
        "line-sensor"
    }
    fn version(&self) -> &str {
        "0.2.0"
    }
    fn run(&self, s: &ThresholdSettings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let verdict = if sub.total_code_lines > s.max_code_lines {
            Verdict::Fail
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2025-01-01T00:00:00Z".to_string(),
            verdict,
            format!("{} code lines", sub.total_code_lines),
        ))
    }
}

#[derive(Serialize, Deserialize)]
struct EmptySettings;

struct AlwaysPassSensor;

impl EffortlessSensor for AlwaysPassSensor {
    type Settings = EmptySettings;
    fn name(&self) -> &str {
        "always-pass"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn run(&self, _: &EmptySettings, _sub: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "noop"),
            "2025-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "Nothing to report".to_string(),
        ))
    }
}

struct FindingSensor;

impl EffortlessSensor for FindingSensor {
    type Settings = EmptySettings;
    fn name(&self) -> &str {
        "finding-sensor"
    }
    fn version(&self) -> &str {
        "0.3.0"
    }
    fn run(&self, _: &EmptySettings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "scan"),
            "2025-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            format!("Found {} files", sub.files.len()),
        );
        for f in &sub.files {
            report.add_finding(
                Finding::new(
                    "review",
                    "file-found",
                    FindingSeverity::Info,
                    &f.path,
                    format!("{} lines of {}", f.code, f.lang),
                )
                .with_location(FindingLocation::path(&f.path)),
            );
        }
        Ok(report)
    }
}

struct DiffAwareSensor;

impl EffortlessSensor for DiffAwareSensor {
    type Settings = EmptySettings;
    fn name(&self) -> &str {
        "diff-aware"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn run(&self, _: &EmptySettings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let diff_count = sub.diff_files().count();
        let verdict = if diff_count > 0 {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "diff"),
            "2025-01-01T00:00:00Z".to_string(),
            verdict,
            format!("{diff_count} files in diff"),
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests: trait identity
// ---------------------------------------------------------------------------

#[test]
fn sensor_name_returns_correct_id() {
    assert_eq!(LineSensor.name(), "line-sensor");
    assert_eq!(AlwaysPassSensor.name(), "always-pass");
    assert_eq!(FindingSensor.name(), "finding-sensor");
    assert_eq!(DiffAwareSensor.name(), "diff-aware");
}

#[test]
fn sensor_version_returns_semver() {
    let v = LineSensor.version();
    assert_eq!(v.split('.').count(), 3, "version must be semver");
}

// ---------------------------------------------------------------------------
// Tests: threshold sensor verdicts
// ---------------------------------------------------------------------------

#[test]
fn line_sensor_pass_under_threshold() {
    let sub = sample_substrate(); // 180 total code lines
    let settings = ThresholdSettings {
        max_code_lines: 500,
    };
    let report = LineSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn line_sensor_fail_over_threshold() {
    let sub = sample_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 100,
    };
    let report = LineSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Fail);
}

#[test]
fn line_sensor_pass_at_exact_threshold() {
    let sub = sample_substrate(); // total = 180
    let settings = ThresholdSettings {
        max_code_lines: 180,
    };
    let report = LineSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

// ---------------------------------------------------------------------------
// Tests: always-pass sensor
// ---------------------------------------------------------------------------

#[test]
fn always_pass_returns_pass_with_empty_substrate() {
    let sub = make_substrate(vec![], None);
    let report = AlwaysPassSensor.run(&EmptySettings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
}

#[test]
fn always_pass_schema_is_sensor_report_v1() {
    let sub = sample_substrate();
    let report = AlwaysPassSensor.run(&EmptySettings, &sub).unwrap();
    assert_eq!(report.schema, "sensor.report.v1");
}

// ---------------------------------------------------------------------------
// Tests: finding sensor
// ---------------------------------------------------------------------------

#[test]
fn finding_sensor_creates_one_finding_per_file() {
    let sub = sample_substrate();
    let report = FindingSensor.run(&EmptySettings, &sub).unwrap();
    assert_eq!(report.findings.len(), sub.files.len());
}

#[test]
fn finding_sensor_each_finding_has_location() {
    let sub = sample_substrate();
    let report = FindingSensor.run(&EmptySettings, &sub).unwrap();
    for finding in &report.findings {
        assert!(
            finding.location.is_some(),
            "every finding must have a location"
        );
    }
}

#[test]
fn finding_sensor_locations_match_file_paths() {
    let sub = sample_substrate();
    let report = FindingSensor.run(&EmptySettings, &sub).unwrap();
    let file_paths: Vec<&str> = sub.files.iter().map(|f| f.path.as_str()).collect();
    for finding in &report.findings {
        let loc_path = &finding.location.as_ref().unwrap().path;
        assert!(file_paths.contains(&loc_path.as_str()));
    }
}

// ---------------------------------------------------------------------------
// Tests: diff-aware sensor
// ---------------------------------------------------------------------------

#[test]
fn diff_aware_sensor_pass_when_no_diff_files() {
    let sub = make_substrate(vec![make_file("a.rs", "Rust", 10, false)], None);
    let report = DiffAwareSensor.run(&EmptySettings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn diff_aware_sensor_warn_when_diff_files_exist() {
    let sub = make_substrate(
        vec![
            make_file("a.rs", "Rust", 10, true),
            make_file("b.rs", "Rust", 20, false),
        ],
        None,
    );
    let report = DiffAwareSensor.run(&EmptySettings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
    assert!(report.summary.contains("1 files in diff"));
}

// ---------------------------------------------------------------------------
// Tests: tool metadata propagation
// ---------------------------------------------------------------------------

#[test]
fn report_tool_meta_matches_sensor() {
    let sub = sample_substrate();
    let report = LineSensor
        .run(
            &ThresholdSettings {
                max_code_lines: 999,
            },
            &sub,
        )
        .unwrap();
    assert_eq!(report.tool.name, "line-sensor");
    assert_eq!(report.tool.version, "0.2.0");
    assert_eq!(report.tool.mode, "check");
}

// ---------------------------------------------------------------------------
// Tests: determinism
// ---------------------------------------------------------------------------

#[test]
fn sensor_output_deterministic_across_runs() {
    let sub = sample_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 500,
    };
    let r1 = serde_json::to_string(&LineSensor.run(&settings, &sub).unwrap()).unwrap();
    let r2 = serde_json::to_string(&LineSensor.run(&settings, &sub).unwrap()).unwrap();
    assert_eq!(r1, r2, "same input must produce identical JSON");
}

#[test]
fn finding_sensor_deterministic_across_runs() {
    let sub = sample_substrate();
    let r1 = serde_json::to_string(&FindingSensor.run(&EmptySettings, &sub).unwrap()).unwrap();
    let r2 = serde_json::to_string(&FindingSensor.run(&EmptySettings, &sub).unwrap()).unwrap();
    assert_eq!(r1, r2);
}

// ---------------------------------------------------------------------------
// Tests: settings serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn threshold_settings_serde_roundtrip() {
    let s = ThresholdSettings { max_code_lines: 42 };
    let json = serde_json::to_string(&s).unwrap();
    let back: ThresholdSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_code_lines, 42);
}

#[test]
fn empty_settings_serde_roundtrip() {
    let s = EmptySettings;
    let json = serde_json::to_string(&s).unwrap();
    let back: EmptySettings = serde_json::from_str(&json).unwrap();
    let _ = back; // just confirm deserialization succeeded
}

// ---------------------------------------------------------------------------
// Tests: capability reporting through sensors
// ---------------------------------------------------------------------------

#[test]
fn sensor_can_attach_capabilities_to_report() {
    let sub = sample_substrate();
    let mut report = AlwaysPassSensor.run(&EmptySettings, &sub).unwrap();
    report.add_capability("loc-check", CapabilityStatus::available());
    report.add_capability("git-analysis", CapabilityStatus::unavailable("no git"));
    let caps = report.capabilities.unwrap();
    assert_eq!(caps.len(), 2);
    assert_eq!(
        caps["loc-check"].status,
        tokmd_envelope::CapabilityState::Available
    );
    assert_eq!(
        caps["git-analysis"].status,
        tokmd_envelope::CapabilityState::Unavailable
    );
}

#[test]
fn sensor_can_attach_artifacts_to_report() {
    let sub = sample_substrate();
    let report = AlwaysPassSensor
        .run(&EmptySettings, &sub)
        .unwrap()
        .with_artifacts(vec![Artifact::receipt("out/receipt.json")]);
    let arts = report.artifacts.unwrap();
    assert_eq!(arts.len(), 1);
    assert_eq!(arts[0].artifact_type, "receipt");
}

#[test]
fn sensor_report_serde_roundtrip_preserves_all_fields() {
    let sub = sample_substrate();
    let report = FindingSensor.run(&EmptySettings, &sub).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool.name, "finding-sensor");
    assert_eq!(back.verdict, Verdict::Warn);
    assert_eq!(back.findings.len(), sub.files.len());
    assert_eq!(back.schema, "sensor.report.v1");
}
