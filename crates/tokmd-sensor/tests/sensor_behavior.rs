//! Deeper tests for `EffortlessSensor` trait implementations,
//! substrate building, and sensor report envelope construction.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA, SensorReport,
    ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── Test sensors ────────────────────────────────────────────────

/// Sensor that counts files per language.
struct LangCountSensor;

#[derive(Serialize, Deserialize)]
struct LangCountSettings {
    max_languages: usize,
}

impl EffortlessSensor for LangCountSensor {
    type Settings = LangCountSettings;

    fn name(&self) -> &str {
        "lang-count"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn run(&self, settings: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let lang_count = sub.lang_summary.len();
        let verdict = if lang_count > settings.max_languages {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-01-15T12:00:00Z".to_string(),
            verdict,
            format!("{lang_count} languages detected"),
        ))
    }
}

/// Sensor that enriches report with findings and artifacts.
struct RichSensor;

#[derive(Serialize, Deserialize)]
struct RichSettings;

impl EffortlessSensor for RichSensor {
    type Settings = RichSettings;

    fn name(&self) -> &str {
        "rich-sensor"
    }

    fn version(&self) -> &str {
        "2.1.0"
    }

    fn run(&self, _: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "review"),
            "2024-06-15T00:00:00Z".to_string(),
            Verdict::Pass,
            format!("{} files scanned", sub.files.len()),
        );

        for f in &sub.files {
            if f.code > 500 {
                report.add_finding(Finding::new(
                    "complexity",
                    "large-file",
                    FindingSeverity::Warn,
                    "File exceeds 500 LOC",
                    format!("{}: {} lines", f.path, f.code),
                ));
                report.verdict = Verdict::Warn;
            }
        }

        report.add_capability("file-size-check", CapabilityStatus::available());
        report.add_capability(
            "complexity-check",
            CapabilityStatus::unavailable("not configured"),
        );

        Ok(report)
    }
}

// ── Helpers ─────────────────────────────────────────────────────

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

fn multi_lang_substrate() -> RepoSubstrate {
    let files = vec![
        SubstrateFile {
            path: "src/lib.rs".to_string(),
            lang: "Rust".to_string(),
            code: 600,
            lines: 700,
            bytes: 18000,
            tokens: 4200,
            module: "src".to_string(),
            in_diff: true,
        },
        SubstrateFile {
            path: "src/util.rs".to_string(),
            lang: "Rust".to_string(),
            code: 100,
            lines: 130,
            bytes: 3000,
            tokens: 700,
            module: "src".to_string(),
            in_diff: false,
        },
        SubstrateFile {
            path: "app.py".to_string(),
            lang: "Python".to_string(),
            code: 200,
            lines: 250,
            bytes: 6000,
            tokens: 1400,
            module: "".to_string(),
            in_diff: true,
        },
        SubstrateFile {
            path: "index.ts".to_string(),
            lang: "TypeScript".to_string(),
            code: 150,
            lines: 180,
            bytes: 4500,
            tokens: 1050,
            module: "".to_string(),
            in_diff: false,
        },
    ];

    let mut lang_summary = BTreeMap::new();
    lang_summary.insert(
        "Rust".to_string(),
        LangSummary {
            files: 2,
            code: 700,
            lines: 830,
            bytes: 21000,
            tokens: 4900,
        },
    );
    lang_summary.insert(
        "Python".to_string(),
        LangSummary {
            files: 1,
            code: 200,
            lines: 250,
            bytes: 6000,
            tokens: 1400,
        },
    );
    lang_summary.insert(
        "TypeScript".to_string(),
        LangSummary {
            files: 1,
            code: 150,
            lines: 180,
            bytes: 4500,
            tokens: 1050,
        },
    );

    RepoSubstrate {
        repo_root: "/project".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "app.py".to_string()],
            commit_count: 3,
            insertions: 50,
            deletions: 20,
        }),
        total_tokens: 7350,
        total_bytes: 31500,
        total_code_lines: 1050,
    }
}

// ── Sensor trait behavior tests ─────────────────────────────────

#[test]
fn sensor_name_and_version_are_stable_across_invocations() {
    let s1 = LangCountSensor;
    let s2 = LangCountSensor;
    assert_eq!(s1.name(), s2.name());
    assert_eq!(s1.version(), s2.version());
    assert_eq!(s1.name(), "lang-count");
    assert_eq!(s1.version(), "1.0.0");
}

#[test]
fn sensor_pass_on_empty_substrate() {
    let sensor = LangCountSensor;
    let settings = LangCountSettings { max_languages: 5 };
    let report = sensor.run(&settings, &empty_substrate()).unwrap();

    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.summary.contains("0"));
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn sensor_warns_when_language_count_exceeds_threshold() {
    let sensor = LangCountSensor;
    let settings = LangCountSettings { max_languages: 2 };
    let sub = multi_lang_substrate();

    let report = sensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
    assert!(report.summary.contains("3"));
}

#[test]
fn sensor_passes_when_language_count_within_threshold() {
    let sensor = LangCountSensor;
    let settings = LangCountSettings { max_languages: 10 };
    let sub = multi_lang_substrate();

    let report = sensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn sensor_on_boundary_language_count() {
    let sensor = LangCountSensor;
    // Exactly 3 languages, threshold = 3 → should pass (not >)
    let settings = LangCountSettings { max_languages: 3 };
    let sub = multi_lang_substrate();

    let report = sensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

// ── Substrate building tests ────────────────────────────────────

#[test]
fn substrate_diff_files_filter_works_in_sensor_context() {
    let sub = multi_lang_substrate();
    let diff_files: Vec<_> = sub.diff_files().collect();

    assert_eq!(diff_files.len(), 2);
    assert!(diff_files.iter().all(|f| f.in_diff));
}

#[test]
fn substrate_files_for_lang_in_sensor_context() {
    let sub = multi_lang_substrate();

    assert_eq!(sub.files_for_lang("Rust").count(), 2);
    assert_eq!(sub.files_for_lang("Python").count(), 1);
    assert_eq!(sub.files_for_lang("TypeScript").count(), 1);
    assert_eq!(sub.files_for_lang("Go").count(), 0);
}

#[test]
fn substrate_totals_are_consistent() {
    let sub = multi_lang_substrate();

    let computed_code: usize = sub.files.iter().map(|f| f.code).sum();
    assert_eq!(sub.total_code_lines, computed_code);

    let computed_bytes: usize = sub.files.iter().map(|f| f.bytes).sum();
    assert_eq!(sub.total_bytes, computed_bytes);

    let computed_tokens: usize = sub.files.iter().map(|f| f.tokens).sum();
    assert_eq!(sub.total_tokens, computed_tokens);
}

// ── Sensor report envelope construction ─────────────────────────

#[test]
fn report_has_correct_schema_identifier() {
    let report = SensorReport::new(
        ToolMeta::new("test", "0.1.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "test summary".to_string(),
    );
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn report_tool_meta_fields_preserved() {
    let report = SensorReport::new(
        ToolMeta::new("my-sensor", "3.2.1", "review"),
        "2024-06-01T00:00:00Z".to_string(),
        Verdict::Warn,
        "some summary".to_string(),
    );

    assert_eq!(report.tool.name, "my-sensor");
    assert_eq!(report.tool.version, "3.2.1");
    assert_eq!(report.tool.mode, "review");
}

#[test]
fn report_with_findings_roundtrips_through_json() {
    let sensor = RichSensor;
    let sub = multi_lang_substrate();
    let report = sensor.run(&RichSettings, &sub).unwrap();

    // Has findings for the large file (600 LOC > 500)
    assert_eq!(report.verdict, Verdict::Warn);
    assert!(!report.findings.is_empty());

    let json = serde_json::to_string_pretty(&report).unwrap();
    let restored: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.schema, report.schema);
    assert_eq!(restored.verdict, report.verdict);
    assert_eq!(restored.findings.len(), report.findings.len());
    assert_eq!(restored.findings[0].check_id, "complexity");
    assert_eq!(restored.findings[0].code, "large-file");
}

#[test]
fn report_capabilities_roundtrip() {
    let sensor = RichSensor;
    let sub = multi_lang_substrate();
    let report = sensor.run(&RichSettings, &sub).unwrap();

    let caps = report.capabilities.as_ref().unwrap();
    assert!(caps.contains_key("file-size-check"));
    assert!(caps.contains_key("complexity-check"));

    let json = serde_json::to_string(&report).unwrap();
    let restored: SensorReport = serde_json::from_str(&json).unwrap();
    let restored_caps = restored.capabilities.unwrap();
    assert_eq!(restored_caps.len(), 2);
}

#[test]
fn report_with_artifacts_and_data() {
    let mut report = SensorReport::new(
        ToolMeta::new("bundler", "1.0.0", "bundle"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "bundled".to_string(),
    );
    report = report
        .with_artifacts(vec![
            Artifact::receipt("out/receipt.json"),
            Artifact::receipt("out/summary.md"),
        ])
        .with_data(serde_json::json!({
            "total_files": 42,
            "version": "1.0"
        }));

    let json = serde_json::to_string(&report).unwrap();
    let restored: SensorReport = serde_json::from_str(&json).unwrap();

    let artifacts = restored.artifacts.unwrap();
    assert_eq!(artifacts.len(), 2);

    let data = restored.data.unwrap();
    assert_eq!(data["total_files"], 42);
    assert_eq!(data["version"], "1.0");
}

#[test]
fn minimal_report_has_no_optional_fields() {
    let report = SensorReport::new(
        ToolMeta::new("minimal", "0.0.1", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "ok".to_string(),
    );

    assert!(report.findings.is_empty());
    assert!(report.artifacts.is_none());
    assert!(report.capabilities.is_none());
    assert!(report.data.is_none());

    // JSON should omit optional None fields
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("artifacts"));
    assert!(!json.contains("capabilities"));
    assert!(!json.contains("\"data\""));
}

#[test]
fn all_verdict_variants_work_in_sensor_report() {
    for verdict in [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ] {
        let report = SensorReport::new(
            ToolMeta::new("test", "0.1.0", "check"),
            "2024-01-01T00:00:00Z".to_string(),
            verdict,
            format!("verdict: {verdict:?}"),
        );
        assert_eq!(report.verdict, verdict);

        let json = serde_json::to_string(&report).unwrap();
        let restored: SensorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.verdict, verdict);
    }
}

#[test]
fn sensor_report_json_is_deterministic() {
    let sensor = RichSensor;
    let sub = multi_lang_substrate();

    let r1 = sensor.run(&RichSettings, &sub).unwrap();
    let r2 = sensor.run(&RichSettings, &sub).unwrap();

    let j1 = serde_json::to_string_pretty(&r1).unwrap();
    let j2 = serde_json::to_string_pretty(&r2).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn settings_json_roundtrip() {
    let settings = LangCountSettings { max_languages: 42 };
    let json = serde_json::to_string(&settings).unwrap();
    let restored: LangCountSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.max_languages, 42);
}

// ── Multiple sensors on same substrate ──────────────────────────

#[test]
fn multiple_sensors_produce_independent_results() {
    let sub = multi_lang_substrate();

    let lang_sensor = LangCountSensor;
    let lang_report = lang_sensor
        .run(&LangCountSettings { max_languages: 1 }, &sub)
        .unwrap();

    let rich_sensor = RichSensor;
    let rich_report = rich_sensor.run(&RichSettings, &sub).unwrap();

    // Both warn but for different reasons
    assert_eq!(lang_report.verdict, Verdict::Warn);
    assert_eq!(rich_report.verdict, Verdict::Warn);
    assert_ne!(lang_report.tool.name, rich_report.tool.name);
    assert!(lang_report.findings.is_empty());
    assert!(!rich_report.findings.is_empty());
}
