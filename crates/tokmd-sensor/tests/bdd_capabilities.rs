//! BDD-style tests for sensor capability reporting, report chaining, and
//! substrate query methods.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA,
    SensorReport, ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Capability-reporting sensor
// ---------------------------------------------------------------------------

struct CapabilitySensor;

#[derive(Serialize, Deserialize)]
struct CapabilitySettings {
    git_available: bool,
    content_available: bool,
}

impl EffortlessSensor for CapabilitySensor {
    type Settings = CapabilitySettings;

    fn name(&self) -> &str {
        "capability-sensor"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn run(&self, settings: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "audit"),
            "2024-07-01T00:00:00Z".to_string(),
            Verdict::Pass,
            format!("{} files audited", sub.files.len()),
        );

        report.add_capability("scan", CapabilityStatus::available());
        if settings.git_available {
            report.add_capability("git-history", CapabilityStatus::available());
        } else {
            report.add_capability(
                "git-history",
                CapabilityStatus::unavailable("git not installed"),
            );
        }
        if settings.content_available {
            report.add_capability("content-scan", CapabilityStatus::available());
        } else {
            report.add_capability(
                "content-scan",
                CapabilityStatus::skipped("not enabled in settings"),
            );
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

fn multi_file_substrate() -> RepoSubstrate {
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
        SubstrateFile {
            path: "README.md".to_string(),
            lang: "Markdown".to_string(),
            code: 30,
            lines: 50,
            bytes: 900,
            tokens: 225,
            module: ".".to_string(),
            in_diff: false,
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
    lang_summary.insert(
        "Markdown".to_string(),
        LangSummary {
            files: 1,
            code: 30,
            lines: 50,
            bytes: 900,
            tokens: 225,
        },
    );
    RepoSubstrate {
        repo_root: "/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "tests/test.py".to_string()],
            commit_count: 3,
            insertions: 20,
            deletions: 5,
        }),
        total_tokens: 2700,
        total_bytes: 10800,
        total_code_lines: 360,
    }
}

// ---------------------------------------------------------------------------
// Scenario: Multiple capabilities reported
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_all_features_available_when_reporting_then_all_capabilities_are_available() {
    let substrate = empty_substrate();
    let sensor = CapabilitySensor;
    let settings = CapabilitySettings {
        git_available: true,
        content_available: true,
    };

    let report = sensor.run(&settings, &substrate).unwrap();

    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["scan"].status, CapabilityState::Available);
    assert_eq!(caps["git-history"].status, CapabilityState::Available);
    assert_eq!(caps["content-scan"].status, CapabilityState::Available);
}

#[test]
fn scenario_given_git_unavailable_when_reporting_then_capability_shows_unavailable_with_reason() {
    let substrate = empty_substrate();
    let sensor = CapabilitySensor;
    let settings = CapabilitySettings {
        git_available: false,
        content_available: true,
    };

    let report = sensor.run(&settings, &substrate).unwrap();

    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["git-history"].status, CapabilityState::Unavailable);
    assert_eq!(
        caps["git-history"].reason.as_deref(),
        Some("git not installed")
    );
}

#[test]
fn scenario_given_content_skipped_when_reporting_then_capability_shows_skipped_with_reason() {
    let substrate = empty_substrate();
    let sensor = CapabilitySensor;
    let settings = CapabilitySettings {
        git_available: true,
        content_available: false,
    };

    let report = sensor.run(&settings, &substrate).unwrap();

    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["content-scan"].status, CapabilityState::Skipped);
    assert_eq!(
        caps["content-scan"].reason.as_deref(),
        Some("not enabled in settings")
    );
}

// ---------------------------------------------------------------------------
// Scenario: Report chaining with_data and with_artifacts
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_sensor_report_when_chaining_with_data_then_data_is_attached() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-07-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "test".to_string(),
    )
    .with_data(serde_json::json!({"metric": 42, "details": "ok"}));

    assert!(report.data.is_some());
    assert_eq!(report.data.as_ref().unwrap()["metric"], 42);
    assert_eq!(report.data.as_ref().unwrap()["details"], "ok");
}

#[test]
fn scenario_given_sensor_report_when_chaining_with_artifacts_then_artifacts_are_attached() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-07-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "test".to_string(),
    )
    .with_artifacts(vec![
        Artifact::receipt("out/lang.json"),
        Artifact::receipt("out/module.json"),
    ]);

    let artifacts = report.artifacts.as_ref().unwrap();
    assert_eq!(artifacts.len(), 2);
}

#[test]
fn scenario_given_sensor_report_when_chaining_both_then_both_present() {
    let report = SensorReport::new(
        ToolMeta::new("combo", "2.0.0", "full"),
        "2024-07-01T00:00:00Z".to_string(),
        Verdict::Warn,
        "combo test".to_string(),
    )
    .with_data(serde_json::json!({"score": 85}))
    .with_artifacts(vec![Artifact::receipt("out/report.json")]);

    assert!(report.data.is_some());
    assert!(report.artifacts.is_some());
    assert_eq!(report.verdict, Verdict::Warn);
}

// ---------------------------------------------------------------------------
// Scenario: Capabilities survive JSON roundtrip
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_report_with_all_capability_states_when_serialized_then_roundtrips() {
    let substrate = empty_substrate();
    let sensor = CapabilitySensor;
    let settings = CapabilitySettings {
        git_available: false,
        content_available: false,
    };

    let report = sensor.run(&settings, &substrate).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let restored: SensorReport = serde_json::from_str(&json).unwrap();

    let caps = restored.capabilities.as_ref().unwrap();
    assert_eq!(caps["scan"].status, CapabilityState::Available);
    assert_eq!(caps["git-history"].status, CapabilityState::Unavailable);
    assert_eq!(caps["content-scan"].status, CapabilityState::Skipped);

    // Reasons preserved
    assert!(caps["git-history"].reason.is_some());
    assert!(caps["content-scan"].reason.is_some());
    assert!(caps["scan"].reason.is_none());
}

// ---------------------------------------------------------------------------
// Scenario: Substrate query methods
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_multi_lang_substrate_when_querying_files_for_lang_then_correct_files_returned() {
    let substrate = multi_file_substrate();

    let rust_files: Vec<&SubstrateFile> = substrate.files_for_lang("Rust").collect();
    assert_eq!(rust_files.len(), 2);
    assert!(rust_files.iter().all(|f| f.lang == "Rust"));

    let python_files: Vec<&SubstrateFile> = substrate.files_for_lang("Python").collect();
    assert_eq!(python_files.len(), 1);
    assert_eq!(python_files[0].path, "tests/test.py");

    let go_files: Vec<&SubstrateFile> = substrate.files_for_lang("Go").collect();
    assert_eq!(go_files.len(), 0);
}

#[test]
fn scenario_given_substrate_with_diff_range_when_querying_diff_files_then_only_diff_files_returned()
{
    let substrate = multi_file_substrate();

    let diff_files: Vec<&SubstrateFile> = substrate.diff_files().collect();
    assert_eq!(diff_files.len(), 2);

    let paths: Vec<&str> = diff_files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"src/lib.rs"));
    assert!(paths.contains(&"tests/test.py"));
    assert!(!paths.contains(&"src/main.rs"));
    assert!(!paths.contains(&"README.md"));
}

#[test]
fn scenario_given_substrate_without_diff_when_querying_diff_files_then_empty() {
    let substrate = empty_substrate();

    let diff_files: Vec<&SubstrateFile> = substrate.diff_files().collect();
    assert!(diff_files.is_empty());
}

// ---------------------------------------------------------------------------
// Scenario: Substrate with all files in diff
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_substrate_where_all_files_in_diff_then_diff_files_returns_all() {
    let files = vec![
        SubstrateFile {
            path: "a.rs".to_string(),
            lang: "Rust".to_string(),
            code: 10,
            lines: 15,
            bytes: 300,
            tokens: 40,
            module: ".".to_string(),
            in_diff: true,
        },
        SubstrateFile {
            path: "b.py".to_string(),
            lang: "Python".to_string(),
            code: 20,
            lines: 25,
            bytes: 600,
            tokens: 80,
            module: ".".to_string(),
            in_diff: true,
        },
    ];
    let substrate = RepoSubstrate {
        repo_root: ".".to_string(),
        files,
        lang_summary: BTreeMap::new(),
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "HEAD".to_string(),
            changed_files: vec!["a.rs".to_string(), "b.py".to_string()],
            commit_count: 1,
            insertions: 5,
            deletions: 0,
        }),
        total_tokens: 120,
        total_bytes: 900,
        total_code_lines: 30,
    };

    let diff_files: Vec<&SubstrateFile> = substrate.diff_files().collect();
    assert_eq!(diff_files.len(), 2);
}

// ---------------------------------------------------------------------------
// Scenario: Report with multiple findings sorted by severity
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_sensor_adding_multiple_findings_then_order_is_preserved() {
    let mut report = SensorReport::new(
        ToolMeta::new("multi-finding", "1.0.0", "check"),
        "2024-07-01T00:00:00Z".to_string(),
        Verdict::Warn,
        "multiple issues".to_string(),
    );

    report.add_finding(Finding::new(
        "security",
        "high-entropy",
        FindingSeverity::Error,
        "High entropy file",
        "secrets.env has high entropy".to_string(),
    ));
    report.add_finding(Finding::new(
        "quality",
        "todo-density",
        FindingSeverity::Warn,
        "High TODO density",
        "src/lib.rs has 15 TODOs".to_string(),
    ));
    report.add_finding(Finding::new(
        "info",
        "file-count",
        FindingSeverity::Info,
        "File count",
        "42 files scanned".to_string(),
    ));

    assert_eq!(report.findings.len(), 3);
    // Insertion order preserved
    assert_eq!(report.findings[0].code, "high-entropy");
    assert_eq!(report.findings[1].code, "todo-density");
    assert_eq!(report.findings[2].code, "file-count");
}

// ---------------------------------------------------------------------------
// Scenario: All verdict variants
// ---------------------------------------------------------------------------

#[test]
fn scenario_all_verdict_variants_can_be_used_in_reports() {
    let verdicts = [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ];

    for verdict in &verdicts {
        let report = SensorReport::new(
            ToolMeta::new("verdict-test", "1.0.0", "check"),
            "2024-07-01T00:00:00Z".to_string(),
            *verdict,
            format!("verdict: {:?}", verdict),
        );
        assert_eq!(report.verdict, *verdict);
        assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    }
}

// ---------------------------------------------------------------------------
// Scenario: ToolMeta mode field
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_different_modes_when_creating_tool_meta_then_mode_is_preserved() {
    for mode in &["check", "review", "audit", "analyze", "count"] {
        let report = SensorReport::new(
            ToolMeta::new("mode-test", "1.0.0", mode),
            "2024-07-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "mode test".to_string(),
        );
        assert_eq!(report.tool.mode, *mode);
    }
}

// ---------------------------------------------------------------------------
// Scenario: Timestamp preservation
// ---------------------------------------------------------------------------

#[test]
fn scenario_given_specific_timestamp_when_creating_report_then_timestamp_preserved() {
    let ts = "2025-01-15T10:30:45Z".to_string();
    let report = SensorReport::new(
        ToolMeta::new("ts-test", "1.0.0", "check"),
        ts.clone(),
        Verdict::Pass,
        "timestamp test".to_string(),
    );
    assert_eq!(report.generated_at, ts);
}
