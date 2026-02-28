//! BDD-style scenario tests for the `EffortlessSensor` trait.
//!
//! Each test follows Given/When/Then structure to verify sensor contract behaviour.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA, SensorReport,
    ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_substrate::{LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Mock sensors
// ---------------------------------------------------------------------------

/// A lines-of-code threshold sensor that reports Pass/Warn/Fail.
struct LocThresholdSensor;

#[derive(Serialize, Deserialize)]
struct LocThresholdSettings {
    warn_threshold: usize,
    fail_threshold: usize,
}

impl EffortlessSensor for LocThresholdSensor {
    type Settings = LocThresholdSettings;

    fn name(&self) -> &str {
        "loc-threshold"
    }

    fn version(&self) -> &str {
        "0.2.0"
    }

    fn run(&self, settings: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let verdict = if sub.total_code_lines >= settings.fail_threshold {
            Verdict::Fail
        } else if sub.total_code_lines >= settings.warn_threshold {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-06-01T00:00:00Z".to_string(),
            verdict,
            format!("{} code lines scanned", sub.total_code_lines),
        ))
    }
}

/// A sensor that always returns Skip (e.g., no relevant files).
struct SkipSensor;

#[derive(Serialize, Deserialize)]
struct EmptySettings;

impl EffortlessSensor for SkipSensor {
    type Settings = EmptySettings;

    fn name(&self) -> &str {
        "skip-sensor"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn run(&self, _: &Self::Settings, _: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-06-01T00:00:00Z".to_string(),
            Verdict::Skip,
            "No relevant files".to_string(),
        ))
    }
}

/// A sensor that always returns an error.
struct FailingSensor;

impl EffortlessSensor for FailingSensor {
    type Settings = EmptySettings;

    fn name(&self) -> &str {
        "failing-sensor"
    }

    fn version(&self) -> &str {
        "0.0.1"
    }

    fn run(&self, _: &Self::Settings, _: &RepoSubstrate) -> Result<SensorReport> {
        anyhow::bail!("sensor internal error: config not found")
    }
}

/// A sensor that produces findings based on diff files.
struct DiffAwareSensor;

#[derive(Serialize, Deserialize)]
struct DiffAwareSettings {
    max_changed_lines: usize,
}

impl EffortlessSensor for DiffAwareSensor {
    type Settings = DiffAwareSettings;

    fn name(&self) -> &str {
        "diff-aware"
    }

    fn version(&self) -> &str {
        "0.3.0"
    }

    fn run(&self, settings: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let diff_files: Vec<&SubstrateFile> = sub.diff_files().collect();
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "review"),
            "2024-06-01T00:00:00Z".to_string(),
            Verdict::Pass,
            format!("{} files in diff", diff_files.len()),
        );

        for f in &diff_files {
            if f.code > settings.max_changed_lines {
                report.add_finding(Finding::new(
                    "size",
                    "large-file",
                    FindingSeverity::Warn,
                    "Large diff file",
                    format!("{} has {} code lines", f.path, f.code),
                ));
                report.verdict = Verdict::Warn;
            }
        }
        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_substrate() -> RepoSubstrate {
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

fn single_file_substrate(code: usize) -> RepoSubstrate {
    RepoSubstrate {
        repo_root: ".".to_string(),
        files: vec![SubstrateFile {
            path: "src/lib.rs".to_string(),
            lang: "Rust".to_string(),
            code,
            lines: code + 20,
            bytes: code * 30,
            tokens: code * 4,
            module: "src".to_string(),
            in_diff: false,
        }],
        lang_summary: BTreeMap::from([(
            "Rust".to_string(),
            LangSummary {
                files: 1,
                code,
                lines: code + 20,
                bytes: code * 30,
                tokens: code * 4,
            },
        )]),
        diff_range: None,
        total_tokens: code * 4,
        total_bytes: code * 30,
        total_code_lines: code,
    }
}

fn multi_lang_substrate() -> RepoSubstrate {
    let files = vec![
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
            code: 50,
            lines: 70,
            bytes: 1500,
            tokens: 375,
            module: "src".to_string(),
            in_diff: false,
        },
        SubstrateFile {
            path: "tests/test.py".to_string(),
            lang: "Python".to_string(),
            code: 80,
            lines: 100,
            bytes: 2400,
            tokens: 600,
            module: "tests".to_string(),
            in_diff: true,
        },
    ];
    let mut lang_summary = BTreeMap::new();
    lang_summary.insert(
        "Rust".to_string(),
        LangSummary {
            files: 2,
            code: 250,
            lines: 320,
            bytes: 7500,
            tokens: 1875,
        },
    );
    lang_summary.insert(
        "Python".to_string(),
        LangSummary {
            files: 1,
            code: 80,
            lines: 100,
            bytes: 2400,
            tokens: 600,
        },
    );
    RepoSubstrate {
        repo_root: "/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(tokmd_substrate::DiffRange {
            base: "main".to_string(),
            head: "feature".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "tests/test.py".to_string()],
            commit_count: 5,
            insertions: 30,
            deletions: 10,
        }),
        total_tokens: 2475,
        total_bytes: 9900,
        total_code_lines: 330,
    }
}

// ---------------------------------------------------------------------------
// Scenario: Sensor passes when under threshold
// ---------------------------------------------------------------------------

#[test]
fn scenario_sensor_passes_when_code_lines_under_threshold() {
    // Given a substrate with 100 code lines
    let substrate = single_file_substrate(100);
    let sensor = LocThresholdSensor;
    let settings = LocThresholdSettings {
        warn_threshold: 500,
        fail_threshold: 1000,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Pass
    assert_eq!(report.verdict, Verdict::Pass);
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(report.tool.name, "loc-threshold");
    assert_eq!(report.tool.version, "0.2.0");
    assert!(report.summary.contains("100"));
}

// ---------------------------------------------------------------------------
// Scenario: Sensor warns when exceeding warn threshold
// ---------------------------------------------------------------------------

#[test]
fn scenario_sensor_warns_when_code_lines_exceed_warn_threshold() {
    // Given a substrate with 500 code lines (above warn, below fail)
    let substrate = single_file_substrate(500);
    let sensor = LocThresholdSensor;
    let settings = LocThresholdSettings {
        warn_threshold: 200,
        fail_threshold: 1000,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Warn
    assert_eq!(report.verdict, Verdict::Warn);
}

// ---------------------------------------------------------------------------
// Scenario: Sensor fails when exceeding fail threshold
// ---------------------------------------------------------------------------

#[test]
fn scenario_sensor_fails_when_code_lines_exceed_fail_threshold() {
    // Given a substrate with 2000 code lines (above fail)
    let substrate = single_file_substrate(2000);
    let sensor = LocThresholdSensor;
    let settings = LocThresholdSettings {
        warn_threshold: 200,
        fail_threshold: 1000,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Fail
    assert_eq!(report.verdict, Verdict::Fail);
}

// ---------------------------------------------------------------------------
// Scenario: Sensor handles empty substrate
// ---------------------------------------------------------------------------

#[test]
fn scenario_sensor_passes_on_empty_substrate() {
    // Given an empty substrate with 0 files
    let substrate = empty_substrate();
    let sensor = LocThresholdSensor;
    let settings = LocThresholdSettings {
        warn_threshold: 100,
        fail_threshold: 500,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Pass (0 < warn_threshold)
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.summary.contains("0"));
    assert!(report.findings.is_empty());
}

// ---------------------------------------------------------------------------
// Scenario: Skip sensor always returns Skip
// ---------------------------------------------------------------------------

#[test]
fn scenario_skip_sensor_produces_skip_verdict() {
    // Given any substrate
    let substrate = single_file_substrate(100);
    let sensor = SkipSensor;

    // When the sensor runs
    let report = sensor.run(&EmptySettings, &substrate).unwrap();

    // Then the verdict is Skip
    assert_eq!(report.verdict, Verdict::Skip);
    assert_eq!(report.tool.name, "skip-sensor");
}

// ---------------------------------------------------------------------------
// Scenario: Failing sensor propagates errors
// ---------------------------------------------------------------------------

#[test]
fn scenario_failing_sensor_returns_error() {
    // Given a failing sensor
    let substrate = empty_substrate();
    let sensor = FailingSensor;

    // When the sensor runs
    let result = sensor.run(&EmptySettings, &substrate);

    // Then it returns Err
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("config not found"));
}

// ---------------------------------------------------------------------------
// Scenario: Diff-aware sensor adds findings for large diff files
// ---------------------------------------------------------------------------

#[test]
fn scenario_diff_aware_sensor_warns_on_large_diff_files() {
    // Given a multi-lang substrate with diff files
    let substrate = multi_lang_substrate();
    let sensor = DiffAwareSensor;
    let settings = DiffAwareSettings {
        max_changed_lines: 100,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then: src/lib.rs (200 lines) is flagged, tests/test.py (80 lines) is not
    assert_eq!(report.verdict, Verdict::Warn);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].check_id, "size");
    assert_eq!(report.findings[0].code, "large-file");
    assert!(report.findings[0].message.contains("src/lib.rs"));
}

#[test]
fn scenario_diff_aware_sensor_passes_when_all_files_small() {
    // Given a substrate where all diff files are below the threshold
    let substrate = multi_lang_substrate();
    let sensor = DiffAwareSensor;
    let settings = DiffAwareSettings {
        max_changed_lines: 500,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Pass with no findings
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
}

// ---------------------------------------------------------------------------
// Scenario: Report JSON serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn scenario_sensor_report_survives_json_roundtrip() {
    // Given a sensor that produces a report with findings and artifacts
    let substrate = multi_lang_substrate();
    let sensor = DiffAwareSensor;
    let settings = DiffAwareSettings {
        max_changed_lines: 100,
    };
    let mut report = sensor.run(&settings, &substrate).unwrap();
    report = report
        .with_artifacts(vec![Artifact::receipt("out/report.json")])
        .with_data(serde_json::json!({ "files_checked": 3 }));
    report.add_capability("diff-size", CapabilityStatus::available());

    // When we serialize and deserialize
    let json = serde_json::to_string_pretty(&report).unwrap();
    let restored: SensorReport = serde_json::from_str(&json).unwrap();

    // Then all fields are preserved
    assert_eq!(restored.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(restored.tool.name, "diff-aware");
    assert_eq!(restored.verdict, Verdict::Warn);
    assert_eq!(restored.findings.len(), 1);
    assert_eq!(restored.artifacts.unwrap().len(), 1);
    assert_eq!(restored.data.unwrap()["files_checked"], 3);
    let caps = restored.capabilities.unwrap();
    assert!(caps.contains_key("diff-size"));
}

// ---------------------------------------------------------------------------
// Scenario: Settings can be deserialized from JSON
// ---------------------------------------------------------------------------

#[test]
fn scenario_settings_deserialize_from_json() {
    // Given a JSON string representing settings
    let json = r#"{ "warn_threshold": 300, "fail_threshold": 800 }"#;

    // When we deserialize
    let settings: LocThresholdSettings = serde_json::from_str(json).unwrap();

    // Then the values match
    assert_eq!(settings.warn_threshold, 300);
    assert_eq!(settings.fail_threshold, 800);
}

// ---------------------------------------------------------------------------
// Scenario: Sensor name and version are stable
// ---------------------------------------------------------------------------

#[test]
fn scenario_sensor_metadata_is_stable() {
    // Given multiple sensor instances
    let s1 = LocThresholdSensor;
    let s2 = LocThresholdSensor;

    // Then name and version are identical across instances
    assert_eq!(s1.name(), s2.name());
    assert_eq!(s1.version(), s2.version());
    assert_eq!(s1.name(), "loc-threshold");
    assert_eq!(s1.version(), "0.2.0");
}

// ---------------------------------------------------------------------------
// Scenario: Sensor with threshold at exact boundary
// ---------------------------------------------------------------------------

#[test]
fn scenario_threshold_boundary_exact_match() {
    // Given a substrate where code lines exactly equal the warn threshold
    let substrate = single_file_substrate(200);
    let sensor = LocThresholdSensor;
    let settings = LocThresholdSettings {
        warn_threshold: 200,
        fail_threshold: 500,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Warn (>= comparison)
    assert_eq!(report.verdict, Verdict::Warn);
}

#[test]
fn scenario_threshold_boundary_one_below() {
    // Given a substrate where code lines are one below the warn threshold
    let substrate = single_file_substrate(199);
    let sensor = LocThresholdSensor;
    let settings = LocThresholdSettings {
        warn_threshold: 200,
        fail_threshold: 500,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Pass
    assert_eq!(report.verdict, Verdict::Pass);
}

// ---------------------------------------------------------------------------
// Scenario: Diff-aware sensor with no diff context
// ---------------------------------------------------------------------------

#[test]
fn scenario_diff_aware_sensor_with_no_diff_context() {
    // Given a substrate with no diff range (no files marked in_diff)
    let substrate = single_file_substrate(500);
    let sensor = DiffAwareSensor;
    let settings = DiffAwareSettings {
        max_changed_lines: 10,
    };

    // When the sensor runs
    let report = sensor.run(&settings, &substrate).unwrap();

    // Then the verdict is Pass (no diff files to check)
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
    assert!(report.summary.contains("0 files in diff"));
}
