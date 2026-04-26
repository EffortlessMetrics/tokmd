//! W74 – Sensor pipeline integration tests.
//!
//! Tests the `EffortlessSensor` trait, substrate building, sensor report
//! generation, capability reporting, and metadata completeness.

use std::collections::BTreeMap;

use anyhow::Result;
use tokmd_envelope::{
    CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Minimal sensor for exercising the trait contract.
struct StubSensor {
    name: &'static str,
    version: &'static str,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StubSettings {
    code_threshold: usize,
}

impl EffortlessSensor for StubSensor {
    type Settings = StubSettings;

    fn name(&self) -> &str {
        self.name
    }

    fn version(&self) -> &str {
        self.version
    }

    fn run(&self, settings: &StubSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let verdict = if substrate.total_code_lines > settings.code_threshold {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2025-01-15T12:00:00Z".to_string(),
            verdict,
            format!(
                "{} code lines (threshold: {})",
                substrate.total_code_lines, settings.code_threshold
            ),
        );
        report.add_capability("scan", CapabilityStatus::available());
        if substrate.diff_range.is_some() {
            report.add_capability("diff", CapabilityStatus::available());
        } else {
            report.add_capability("diff", CapabilityStatus::skipped("no diff range provided"));
        }
        Ok(report)
    }
}

fn default_sensor() -> StubSensor {
    StubSensor {
        name: "test-sensor",
        version: "0.1.0",
    }
}

fn sample_substrate() -> RepoSubstrate {
    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files: vec![
            SubstrateFile {
                path: "src/lib.rs".to_string(),
                lang: "Rust".to_string(),
                code: 200,
                lines: 250,
                bytes: 6000,
                tokens: 1500,
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
                path: "tests/smoke.rs".to_string(),
                lang: "Rust".to_string(),
                code: 40,
                lines: 50,
                bytes: 1200,
                tokens: 300,
                module: "tests".to_string(),
                in_diff: false,
            },
        ],
        lang_summary: BTreeMap::from([(
            "Rust".to_string(),
            LangSummary {
                files: 3,
                code: 320,
                lines: 400,
                bytes: 9600,
                tokens: 2400,
            },
        )]),
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["src/lib.rs".to_string()],
            commit_count: 2,
            insertions: 15,
            deletions: 3,
        }),
        total_tokens: 2400,
        total_bytes: 9600,
        total_code_lines: 320,
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

// ---------------------------------------------------------------------------
// 1. EffortlessSensor trait basics
// ---------------------------------------------------------------------------

#[test]
fn sensor_name_returns_expected_value() {
    let s = default_sensor();
    assert_eq!(s.name(), "test-sensor");
}

#[test]
fn sensor_version_returns_expected_value() {
    let s = default_sensor();
    assert_eq!(s.version(), "0.1.0");
}

#[test]
fn sensor_run_returns_pass_below_threshold() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 500,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn sensor_run_returns_warn_above_threshold() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 100,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
}

#[test]
fn sensor_run_on_empty_substrate_passes() {
    let s = default_sensor();
    let settings = StubSettings { code_threshold: 0 };
    // 0 code lines is NOT > 0, so should pass
    let report = s.run(&settings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

// ---------------------------------------------------------------------------
// 2. Report metadata completeness
// ---------------------------------------------------------------------------

#[test]
fn report_schema_is_sensor_report_v1() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 1000,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(report.schema, "sensor.report.v1");
}

#[test]
fn report_tool_meta_matches_sensor() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 1000,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.tool.name, "test-sensor");
    assert_eq!(report.tool.version, "0.1.0");
    assert_eq!(report.tool.mode, "check");
}

#[test]
fn report_generated_at_is_iso8601() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 1000,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    assert!(report.generated_at.contains('T'));
    assert!(report.generated_at.ends_with('Z'));
}

#[test]
fn report_summary_contains_code_lines() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 1000,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    assert!(report.summary.contains("320"));
    assert!(report.summary.contains("1000"));
}

// ---------------------------------------------------------------------------
// 3. Capability reporting
// ---------------------------------------------------------------------------

#[test]
fn capabilities_present_when_diff_range_exists() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 1000,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    let caps = report
        .capabilities
        .as_ref()
        .expect("capabilities should be set");
    assert_eq!(caps["scan"].status, CapabilityState::Available);
    assert_eq!(caps["diff"].status, CapabilityState::Available);
}

#[test]
fn capabilities_diff_skipped_without_diff_range() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 1000,
    };
    let report = s.run(&settings, &empty_substrate()).unwrap();
    let caps = report
        .capabilities
        .as_ref()
        .expect("capabilities should be set");
    assert_eq!(caps["scan"].status, CapabilityState::Available);
    assert_eq!(caps["diff"].status, CapabilityState::Skipped);
    assert!(caps["diff"].reason.as_ref().unwrap().contains("no diff"));
}

#[test]
fn capabilities_unavailable_variant_works() {
    let cap = CapabilityStatus::unavailable("git not found");
    assert_eq!(cap.status, CapabilityState::Unavailable);
    assert_eq!(cap.reason.as_deref(), Some("git not found"));
}

// ---------------------------------------------------------------------------
// 4. Serde roundtrip of sensor report
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_serde_roundtrip() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 100,
    };
    let report = s.run(&settings, &sample_substrate()).unwrap();
    let json = serde_json::to_string_pretty(&report).unwrap();
    let deser: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.schema, report.schema);
    assert_eq!(deser.verdict, report.verdict);
    assert_eq!(deser.tool.name, report.tool.name);
    assert_eq!(deser.summary, report.summary);
    let caps = deser.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 2);
}

#[test]
fn sensor_report_with_findings_roundtrip() {
    let s = default_sensor();
    let settings = StubSettings {
        code_threshold: 100,
    };
    let mut report = s.run(&settings, &sample_substrate()).unwrap();
    report.add_finding(
        Finding::new(
            "risk",
            "hotspot",
            FindingSeverity::Warn,
            "Churn",
            "high churn",
        )
        .with_location(FindingLocation::path("src/lib.rs"))
        .with_fingerprint("test-sensor"),
    );
    report.add_finding(Finding::new(
        "contract",
        "schema_changed",
        FindingSeverity::Info,
        "Schema",
        "version bumped",
    ));

    let json = serde_json::to_string(&report).unwrap();
    let deser: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.findings.len(), 2);
    assert_eq!(deser.findings[0].check_id, "risk");
    assert!(deser.findings[0].fingerprint.is_some());
    assert!(deser.findings[1].fingerprint.is_none());
}

// ---------------------------------------------------------------------------
// 5. Substrate builder (integration – scans own crate)
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_scans_own_crate() {
    use tokmd_sensor::substrate_builder::build_substrate;
    use tokmd_settings::ScanOptions;

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    assert!(!substrate.files.is_empty(), "should find source files");
    assert!(
        substrate.lang_summary.contains_key("Rust"),
        "should detect Rust"
    );
    assert!(substrate.total_code_lines > 0);
    assert!(substrate.total_tokens > 0);
    assert!(substrate.total_bytes > 0);
    assert!(substrate.diff_range.is_none());
}
