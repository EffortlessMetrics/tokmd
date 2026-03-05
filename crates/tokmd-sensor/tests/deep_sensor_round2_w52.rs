//! Deep round-2 tests for tokmd-sensor (W52).
//!
//! Covers EffortlessSensor trait semantics, substrate builder construction,
//! sensor report generation, and metadata invariants.

use std::collections::BTreeMap;

use anyhow::Result;
use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA, SensorReport,
    ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A test sensor that mirrors substrate totals into findings.
struct MetricsSensor;

#[derive(serde::Serialize, serde::Deserialize)]
struct MetricsSettings {
    code_threshold: usize,
    token_threshold: usize,
}

impl EffortlessSensor for MetricsSensor {
    type Settings = MetricsSettings;

    fn name(&self) -> &str {
        "metrics"
    }

    fn version(&self) -> &str {
        "2.0.0"
    }

    fn run(&self, settings: &MetricsSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let mut findings = Vec::new();

        if substrate.total_code_lines > settings.code_threshold {
            findings.push(Finding::new(
                "size",
                "code_exceeds",
                FindingSeverity::Warn,
                "Code size exceeded",
                format!(
                    "{} lines > threshold {}",
                    substrate.total_code_lines, settings.code_threshold
                ),
            ));
        }

        if substrate.total_tokens > settings.token_threshold {
            findings.push(Finding::new(
                "size",
                "tokens_exceeds",
                FindingSeverity::Info,
                "Token count exceeded",
                format!(
                    "{} tokens > threshold {}",
                    substrate.total_tokens, settings.token_threshold
                ),
            ));
        }

        let verdict = if findings.iter().any(|f| f.severity == FindingSeverity::Warn) {
            Verdict::Warn
        } else {
            Verdict::Pass
        };

        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "analyze"),
            "2025-01-15T12:00:00Z".to_string(),
            verdict,
            format!("{} findings", findings.len()),
        );
        for f in findings {
            report.add_finding(f);
        }
        Ok(report)
    }
}

/// A sensor that always skips.
struct SkipSensor;

#[derive(serde::Serialize, serde::Deserialize)]
struct EmptySettings;

impl EffortlessSensor for SkipSensor {
    type Settings = EmptySettings;

    fn name(&self) -> &str {
        "skip-bot"
    }

    fn version(&self) -> &str {
        "0.0.1"
    }

    fn run(&self, _settings: &EmptySettings, _substrate: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2025-01-15T12:00:00Z".to_string(),
            Verdict::Skip,
            "Nothing to do".to_string(),
        ))
    }
}

fn minimal_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: ".".to_string(),
        files: vec![],
        lang_summary: BTreeMap::new(),
        diff_range: None,
        total_tokens: 0,
        total_bytes: 0,
        total_code_lines: 0,
    }
}

fn full_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/workspace/project".to_string(),
        files: vec![
            SubstrateFile {
                path: "src/lib.rs".to_string(),
                lang: "Rust".to_string(),
                code: 500,
                lines: 600,
                bytes: 15000,
                tokens: 3750,
                module: "src".to_string(),
                in_diff: true,
            },
            SubstrateFile {
                path: "src/main.rs".to_string(),
                lang: "Rust".to_string(),
                code: 80,
                lines: 100,
                bytes: 2400,
                tokens: 600,
                module: "src".to_string(),
                in_diff: false,
            },
            SubstrateFile {
                path: "tests/integration.py".to_string(),
                lang: "Python".to_string(),
                code: 200,
                lines: 250,
                bytes: 6000,
                tokens: 1500,
                module: "tests".to_string(),
                in_diff: true,
            },
        ],
        lang_summary: BTreeMap::from([
            (
                "Rust".to_string(),
                LangSummary {
                    files: 2,
                    code: 580,
                    lines: 700,
                    bytes: 17400,
                    tokens: 4350,
                },
            ),
            (
                "Python".to_string(),
                LangSummary {
                    files: 1,
                    code: 200,
                    lines: 250,
                    bytes: 6000,
                    tokens: 1500,
                },
            ),
        ]),
        diff_range: Some(DiffRange {
            base: "v1.0.0".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "tests/integration.py".to_string()],
            commit_count: 7,
            insertions: 42,
            deletions: 15,
        }),
        total_tokens: 5850,
        total_bytes: 23400,
        total_code_lines: 780,
    }
}

// ---------------------------------------------------------------------------
// Tests: EffortlessSensor trait basics
// ---------------------------------------------------------------------------

#[test]
fn sensor_name_returns_expected_identifier() {
    let s = MetricsSensor;
    assert_eq!(s.name(), "metrics");
}

#[test]
fn sensor_version_returns_semver_string() {
    let s = MetricsSensor;
    let ver = s.version();
    assert!(
        ver.split('.').count() == 3,
        "version should be semver: {ver}"
    );
}

#[test]
fn skip_sensor_name_and_version() {
    let s = SkipSensor;
    assert_eq!(s.name(), "skip-bot");
    assert_eq!(s.version(), "0.0.1");
}

// ---------------------------------------------------------------------------
// Tests: Sensor run with minimal substrate
// ---------------------------------------------------------------------------

#[test]
fn sensor_run_with_empty_substrate_passes() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 100,
        token_threshold: 500,
    };
    let report = sensor.run(&settings, &minimal_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
}

#[test]
fn skip_sensor_returns_skip_verdict_on_empty_substrate() {
    let sensor = SkipSensor;
    let report = sensor.run(&EmptySettings, &minimal_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Skip);
    assert_eq!(report.summary, "Nothing to do");
}

// ---------------------------------------------------------------------------
// Tests: Sensor run with full substrate
// ---------------------------------------------------------------------------

#[test]
fn sensor_warns_when_code_exceeds_threshold() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 500,
        token_threshold: 10000,
    };
    let report = sensor.run(&settings, &full_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
    assert!(report.findings.iter().any(|f| f.code == "code_exceeds"));
}

#[test]
fn sensor_reports_multiple_findings_when_both_thresholds_exceeded() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 100,
        token_threshold: 100,
    };
    let report = sensor.run(&settings, &full_substrate()).unwrap();
    assert_eq!(report.findings.len(), 2);
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.contains(&"code_exceeds"));
    assert!(codes.contains(&"tokens_exceeds"));
}

#[test]
fn sensor_passes_when_thresholds_not_exceeded() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 10000,
        token_threshold: 100000,
    };
    let report = sensor.run(&settings, &full_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
}

// ---------------------------------------------------------------------------
// Tests: Report metadata
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_has_correct_schema() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 10000,
        token_threshold: 100000,
    };
    let report = sensor.run(&settings, &full_substrate()).unwrap();
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn sensor_report_tool_meta_matches_sensor() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 10000,
        token_threshold: 100000,
    };
    let report = sensor.run(&settings, &full_substrate()).unwrap();
    assert_eq!(report.tool.name, sensor.name());
    assert_eq!(report.tool.version, sensor.version());
    assert_eq!(report.tool.mode, "analyze");
}

#[test]
fn sensor_report_has_iso8601_timestamp() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 10000,
        token_threshold: 100000,
    };
    let report = sensor.run(&settings, &full_substrate()).unwrap();
    assert!(
        report.generated_at.contains('T') && report.generated_at.ends_with('Z'),
        "timestamp should be ISO 8601: {}",
        report.generated_at
    );
}

// ---------------------------------------------------------------------------
// Tests: Settings serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn settings_serde_roundtrip() {
    let settings = MetricsSettings {
        code_threshold: 42,
        token_threshold: 1000,
    };
    let json = serde_json::to_string(&settings).unwrap();
    let back: MetricsSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.code_threshold, 42);
    assert_eq!(back.token_threshold, 1000);
}

// ---------------------------------------------------------------------------
// Tests: Sensor report serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_full_serde_roundtrip() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 100,
        token_threshold: 100,
    };
    let report = sensor.run(&settings, &full_substrate()).unwrap();
    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.schema, report.schema);
    assert_eq!(back.verdict, report.verdict);
    assert_eq!(back.findings.len(), report.findings.len());
    assert_eq!(back.tool.name, report.tool.name);
    assert_eq!(back.summary, report.summary);
}

// ---------------------------------------------------------------------------
// Tests: Sensor with capabilities
// ---------------------------------------------------------------------------

#[test]
fn sensor_can_attach_capabilities_to_report() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 10000,
        token_threshold: 100000,
    };
    let mut report = sensor.run(&settings, &full_substrate()).unwrap();
    report.add_capability("code_size", CapabilityStatus::available());
    report.add_capability(
        "token_budget",
        CapabilityStatus::available().with_reason("within limits"),
    );

    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 2);
    assert!(caps.contains_key("code_size"));
    assert!(caps.contains_key("token_budget"));
}

// ---------------------------------------------------------------------------
// Tests: Sensor with artifacts
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_with_artifacts_roundtrips() {
    let sensor = MetricsSensor;
    let settings = MetricsSettings {
        code_threshold: 10000,
        token_threshold: 100000,
    };
    let report = sensor
        .run(&settings, &full_substrate())
        .unwrap()
        .with_artifacts(vec![
            Artifact::receipt("out/metrics.json"),
            Artifact::badge("out/badge.svg"),
        ]);

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let arts = back.artifacts.unwrap();
    assert_eq!(arts.len(), 2);
    assert_eq!(arts[0].artifact_type, "receipt");
    assert_eq!(arts[1].artifact_type, "badge");
}
