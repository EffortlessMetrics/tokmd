//! Depth tests for tokmd-sensor: EffortlessSensor trait, substrate builder, and reports.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingSeverity, SensorReport, ToolMeta,
    Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ──────────────────────────────────────────────────────────────────────
// Test sensor implementations
// ──────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct SimpleSettings {
    threshold: usize,
}

struct SimpleSensor;

impl EffortlessSensor for SimpleSensor {
    type Settings = SimpleSettings;

    fn name(&self) -> &str {
        "simple"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn run(&self, settings: &SimpleSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let verdict = if substrate.total_code_lines > settings.threshold {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2025-01-01T00:00:00Z".to_string(),
            verdict,
            format!(
                "{} lines vs threshold {}",
                substrate.total_code_lines, settings.threshold
            ),
        ))
    }
}

/// A sensor that always fails.
struct FailingSensor;

#[derive(Serialize, Deserialize)]
struct EmptySettings;

impl EffortlessSensor for FailingSensor {
    type Settings = EmptySettings;

    fn name(&self) -> &str {
        "failing"
    }

    fn version(&self) -> &str {
        "0.0.1"
    }

    fn run(&self, _settings: &EmptySettings, _substrate: &RepoSubstrate) -> Result<SensorReport> {
        anyhow::bail!("intentional sensor failure")
    }
}

/// A sensor that produces findings.
struct FindingSensor;

#[derive(Serialize, Deserialize)]
struct FindingSettings {
    max_lines_per_file: usize,
}

impl EffortlessSensor for FindingSensor {
    type Settings = FindingSettings;

    fn name(&self) -> &str {
        "file-checker"
    }

    fn version(&self) -> &str {
        "2.0.0"
    }

    fn run(&self, settings: &FindingSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "analyze"),
            "2025-06-01T12:00:00Z".to_string(),
            Verdict::Pass,
            "file size check".to_string(),
        );

        for file in &substrate.files {
            if file.lines > settings.max_lines_per_file {
                report.verdict = Verdict::Warn;
                report.add_finding(Finding {
                    check_id: "size".to_string(),
                    code: "large-file".to_string(),
                    severity: FindingSeverity::Warn,
                    title: "Large file detected".to_string(),
                    message: format!("{}: {} lines", file.path, file.lines),
                    location: None,
                    evidence: None,
                    docs_url: None,
                    fingerprint: None,
                });
            }
        }

        Ok(report)
    }
}

/// A sensor that reports capabilities.
struct CapabilitySensor;

impl EffortlessSensor for CapabilitySensor {
    type Settings = EmptySettings;

    fn name(&self) -> &str {
        "cap-sensor"
    }

    fn version(&self) -> &str {
        "3.0.0"
    }

    fn run(&self, _settings: &EmptySettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "scan"),
            "2025-06-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "capability report".to_string(),
        );

        report.add_capability(
            "rust-analysis",
            CapabilityStatus {
                status: if substrate.lang_summary.contains_key("Rust") {
                    CapabilityState::Available
                } else {
                    CapabilityState::Unavailable
                },
                reason: None,
            },
        );
        report.add_capability(
            "git-history",
            CapabilityStatus {
                status: if substrate.diff_range.is_some() {
                    CapabilityState::Available
                } else {
                    CapabilityState::Skipped
                },
                reason: Some("no diff range provided".to_string()),
            },
        );

        Ok(report)
    }
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

fn make_substrate(files: Vec<SubstrateFile>, diff_range: Option<DiffRange>) -> RepoSubstrate {
    let mut lang_summary: BTreeMap<String, LangSummary> = BTreeMap::new();
    for f in &files {
        let entry = lang_summary.entry(f.lang.clone()).or_insert(LangSummary {
            files: 0,
            code: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        });
        entry.files += 1;
        entry.code += f.code;
        entry.lines += f.lines;
        entry.bytes += f.bytes;
        entry.tokens += f.tokens;
    }
    let total_tokens: usize = files.iter().map(|f| f.tokens).sum();
    let total_bytes: usize = files.iter().map(|f| f.bytes).sum();
    let total_code_lines: usize = files.iter().map(|f| f.code).sum();

    RepoSubstrate {
        repo_root: ".".to_string(),
        files,
        lang_summary,
        diff_range,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

fn file(path: &str, lang: &str, code: usize, lines: usize) -> SubstrateFile {
    SubstrateFile {
        path: path.to_string(),
        lang: lang.to_string(),
        code,
        lines,
        bytes: lines * 30,
        tokens: code * 5,
        module: path.split('/').next().unwrap_or("root").to_string(),
        in_diff: false,
    }
}

fn single_file_substrate() -> RepoSubstrate {
    make_substrate(vec![file("src/lib.rs", "Rust", 100, 120)], None)
}

fn multi_file_substrate() -> RepoSubstrate {
    make_substrate(
        vec![
            file("src/main.rs", "Rust", 200, 250),
            file("src/lib.rs", "Rust", 150, 180),
            file("tests/test.py", "Python", 80, 100),
        ],
        None,
    )
}

fn empty_substrate() -> RepoSubstrate {
    make_substrate(vec![], None)
}

// ──────────────────────────────────────────────────────────────────────
// 1. EffortlessSensor trait implementation tests
// ──────────────────────────────────────────────────────────────────────

#[test]
fn sensor_name_returns_static_str() {
    let s = SimpleSensor;
    assert_eq!(s.name(), "simple");
}

#[test]
fn sensor_version_returns_semver() {
    let s = SimpleSensor;
    let v = s.version();
    assert_eq!(v.split('.').count(), 3, "version should be semver");
}

#[test]
fn sensor_run_pass_when_under_threshold() {
    let s = SimpleSensor;
    let settings = SimpleSettings { threshold: 500 };
    let report = s.run(&settings, &single_file_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn sensor_run_warn_when_over_threshold() {
    let s = SimpleSensor;
    let settings = SimpleSettings { threshold: 50 };
    let report = s.run(&settings, &single_file_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
}

#[test]
fn sensor_run_pass_at_exact_threshold() {
    let s = SimpleSensor;
    let settings = SimpleSettings { threshold: 100 };
    let report = s.run(&settings, &single_file_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn failing_sensor_returns_error() {
    let s = FailingSensor;
    let result = s.run(&EmptySettings, &single_file_substrate());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("intentional"), "error: {msg}");
}

#[test]
fn multiple_sensor_impls_coexist() {
    let s1 = SimpleSensor;
    let s2 = FailingSensor;
    let s3 = FindingSensor;
    assert_ne!(s1.name(), s2.name());
    assert_ne!(s2.name(), s3.name());
    assert_ne!(s1.name(), s3.name());
}

// ──────────────────────────────────────────────────────────────────────
// 2. Sensor report generation
// ──────────────────────────────────────────────────────────────────────

#[test]
fn report_schema_is_sensor_report_v1() {
    let s = SimpleSensor;
    let report = s
        .run(&SimpleSettings { threshold: 999 }, &single_file_substrate())
        .unwrap();
    assert_eq!(report.schema, "sensor.report.v1");
}

#[test]
fn report_tool_meta_matches_sensor() {
    let s = SimpleSensor;
    let report = s
        .run(&SimpleSettings { threshold: 999 }, &single_file_substrate())
        .unwrap();
    assert_eq!(report.tool.name, "simple");
    assert_eq!(report.tool.version, "1.0.0");
    assert_eq!(report.tool.mode, "check");
}

#[test]
fn report_summary_contains_line_count() {
    let s = SimpleSensor;
    let report = s
        .run(&SimpleSettings { threshold: 999 }, &single_file_substrate())
        .unwrap();
    assert!(
        report.summary.contains("100"),
        "summary should mention line count"
    );
}

#[test]
fn report_generated_at_is_populated() {
    let s = SimpleSensor;
    let report = s
        .run(&SimpleSettings { threshold: 999 }, &single_file_substrate())
        .unwrap();
    assert!(!report.generated_at.is_empty());
}

#[test]
fn report_json_roundtrip() {
    let s = SimpleSensor;
    let report = s
        .run(&SimpleSettings { threshold: 999 }, &single_file_substrate())
        .unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.verdict, report.verdict);
    assert_eq!(back.summary, report.summary);
    assert_eq!(back.schema, report.schema);
}

#[test]
fn report_with_artifacts() {
    let s = SimpleSensor;
    let report = s
        .run(&SimpleSettings { threshold: 999 }, &single_file_substrate())
        .unwrap()
        .with_artifacts(vec![Artifact {
            id: None,
            artifact_type: "receipt".to_string(),
            path: "output.json".to_string(),
            mime: Some("application/json".to_string()),
        }]);
    assert!(report.artifacts.is_some());
    assert_eq!(report.artifacts.as_ref().unwrap().len(), 1);
}

#[test]
fn report_with_data_payload() {
    let s = SimpleSensor;
    let data = serde_json::json!({"key": "value", "count": 42});
    let report = s
        .run(&SimpleSettings { threshold: 999 }, &single_file_substrate())
        .unwrap()
        .with_data(data.clone());
    assert_eq!(report.data, Some(data));
}

// ──────────────────────────────────────────────────────────────────────
// 3. Finding sensor tests
// ──────────────────────────────────────────────────────────────────────

#[test]
fn finding_sensor_no_findings_when_all_small() {
    let s = FindingSensor;
    let settings = FindingSettings {
        max_lines_per_file: 500,
    };
    let report = s.run(&settings, &single_file_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
}

#[test]
fn finding_sensor_warns_on_large_file() {
    let substrate = make_substrate(vec![file("big.rs", "Rust", 1000, 1200)], None);
    let s = FindingSensor;
    let settings = FindingSettings {
        max_lines_per_file: 500,
    };
    let report = s.run(&settings, &substrate).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].code, "large-file");
    assert!(report.findings[0].message.contains("big.rs"));
}

#[test]
fn finding_sensor_multiple_findings() {
    let substrate = make_substrate(
        vec![
            file("a.rs", "Rust", 600, 700),
            file("b.rs", "Rust", 800, 900),
            file("c.rs", "Rust", 10, 15),
        ],
        None,
    );
    let s = FindingSensor;
    let settings = FindingSettings {
        max_lines_per_file: 100,
    };
    let report = s.run(&settings, &substrate).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
    assert_eq!(report.findings.len(), 2);
}

// ──────────────────────────────────────────────────────────────────────
// 4. Capability reporting
// ──────────────────────────────────────────────────────────────────────

#[test]
fn capability_sensor_rust_available() {
    let s = CapabilitySensor;
    let report = s.run(&EmptySettings, &single_file_substrate()).unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["rust-analysis"].status, CapabilityState::Available);
}

#[test]
fn capability_sensor_git_skipped_without_diff() {
    let s = CapabilitySensor;
    let report = s.run(&EmptySettings, &single_file_substrate()).unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["git-history"].status, CapabilityState::Skipped);
    assert!(caps["git-history"].reason.is_some());
}

#[test]
fn capability_sensor_git_available_with_diff() {
    let diff = DiffRange {
        base: "main".to_string(),
        head: "HEAD".to_string(),
        changed_files: vec!["src/lib.rs".to_string()],
        commit_count: 1,
        insertions: 5,
        deletions: 2,
    };
    let substrate = make_substrate(vec![file("src/lib.rs", "Rust", 100, 120)], Some(diff));
    let s = CapabilitySensor;
    let report = s.run(&EmptySettings, &substrate).unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["git-history"].status, CapabilityState::Available);
}

#[test]
fn capability_sensor_rust_unavailable_for_python_only() {
    let substrate = make_substrate(vec![file("main.py", "Python", 50, 60)], None);
    let s = CapabilitySensor;
    let report = s.run(&EmptySettings, &substrate).unwrap();
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["rust-analysis"].status, CapabilityState::Unavailable);
}

// ──────────────────────────────────────────────────────────────────────
// 5. Edge cases and determinism
// ──────────────────────────────────────────────────────────────────────

#[test]
fn sensor_on_empty_substrate() {
    let s = SimpleSensor;
    let settings = SimpleSettings { threshold: 0 };
    let report = s.run(&settings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn sensor_deterministic_across_runs() {
    let s = SimpleSensor;
    let settings = SimpleSettings { threshold: 999 };
    let sub = single_file_substrate();
    let r1 = s.run(&settings, &sub).unwrap();
    let r2 = s.run(&settings, &sub).unwrap();
    assert_eq!(r1.verdict, r2.verdict);
    assert_eq!(r1.summary, r2.summary);
    assert_eq!(r1.schema, r2.schema);
}

#[test]
fn sensor_json_deterministic() {
    let s = SimpleSensor;
    let settings = SimpleSettings { threshold: 999 };
    let sub = single_file_substrate();
    let j1 = serde_json::to_string(&s.run(&settings, &sub).unwrap()).unwrap();
    let j2 = serde_json::to_string(&s.run(&settings, &sub).unwrap()).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn settings_json_roundtrip() {
    let settings = SimpleSettings { threshold: 42 };
    let json = serde_json::to_string(&settings).unwrap();
    let back: SimpleSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.threshold, 42);
}

#[test]
fn multi_lang_substrate_totals_correct() {
    let sub = multi_file_substrate();
    assert_eq!(sub.total_code_lines, 200 + 150 + 80);
    assert_eq!(sub.lang_summary.len(), 2);
    assert!(sub.lang_summary.contains_key("Rust"));
    assert!(sub.lang_summary.contains_key("Python"));
    assert_eq!(sub.lang_summary["Rust"].files, 2);
    assert_eq!(sub.lang_summary["Python"].files, 1);
}

#[test]
fn substrate_diff_range_none_by_default() {
    let sub = single_file_substrate();
    assert!(sub.diff_range.is_none());
}

#[test]
fn substrate_file_in_diff_flag() {
    let mut f = file("src/lib.rs", "Rust", 100, 120);
    f.in_diff = true;
    let sub = make_substrate(vec![f], None);
    assert!(sub.files[0].in_diff);
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

// ──────────────────────────────────────────────────────────────────────
// 6. Substrate builder integration (live scan)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn substrate_builder_scans_sensor_crate_src() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = tokmd_sensor::substrate_builder::build_substrate(
        &format!("{}/src", manifest_dir),
        &tokmd_settings::ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();
    assert!(!substrate.files.is_empty());
    assert!(substrate.lang_summary.contains_key("Rust"));
    assert!(substrate.total_code_lines > 0);
    assert!(substrate.total_bytes > 0);
    assert!(substrate.total_tokens > 0);
}

#[test]
fn substrate_builder_totals_match_file_sums() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = tokmd_sensor::substrate_builder::build_substrate(
        &format!("{}/src", manifest_dir),
        &tokmd_settings::ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let sum_code: usize = substrate.files.iter().map(|f| f.code).sum();
    let sum_bytes: usize = substrate.files.iter().map(|f| f.bytes).sum();
    let sum_tokens: usize = substrate.files.iter().map(|f| f.tokens).sum();
    assert_eq!(substrate.total_code_lines, sum_code);
    assert_eq!(substrate.total_bytes, sum_bytes);
    assert_eq!(substrate.total_tokens, sum_tokens);
}

#[test]
fn substrate_builder_lang_summary_matches_files() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = tokmd_sensor::substrate_builder::build_substrate(
        &format!("{}/src", manifest_dir),
        &tokmd_settings::ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    for (lang, summary) in &substrate.lang_summary {
        let lang_files: Vec<_> = substrate.files.iter().filter(|f| &f.lang == lang).collect();
        assert_eq!(summary.files, lang_files.len());
        let sum_code: usize = lang_files.iter().map(|f| f.code).sum();
        assert_eq!(summary.code, sum_code);
    }
}
