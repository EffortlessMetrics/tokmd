//! Wave 48 deep tests for tokmd-sensor.
//!
//! Covers EffortlessSensor trait implementation, RepoSubstrate construction
//! and field access, substrate builder patterns, sensor report envelope format,
//! property tests for required fields, and edge cases.

use std::collections::BTreeMap;

use anyhow::Result;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA,
    SensorReport, ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ============================================================================
// Test sensor implementations
// ============================================================================

/// A minimal sensor that always passes.
struct PassSensor;

#[derive(Serialize, Deserialize)]
struct PassSettings;

impl EffortlessSensor for PassSensor {
    type Settings = PassSettings;
    fn name(&self) -> &str {
        "pass-sensor"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn run(&self, _: &Self::Settings, _sub: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "all clear".to_string(),
        ))
    }
}

/// A threshold sensor that warns when code lines exceed a limit.
struct ThresholdSensor;

#[derive(Serialize, Deserialize)]
struct ThresholdSettings {
    max_code_lines: usize,
}

impl EffortlessSensor for ThresholdSensor {
    type Settings = ThresholdSettings;
    fn name(&self) -> &str {
        "threshold"
    }
    fn version(&self) -> &str {
        "0.2.0"
    }
    fn run(&self, settings: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let verdict = if sub.total_code_lines > settings.max_code_lines {
            Verdict::Fail
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-06-15T12:00:00Z".to_string(),
            verdict,
            format!(
                "{} code lines (max: {})",
                sub.total_code_lines, settings.max_code_lines
            ),
        ))
    }
}

/// A sensor that counts languages.
struct LangCountSensor;

#[derive(Serialize, Deserialize)]
struct LangCountSettings {
    min_languages: usize,
}

impl EffortlessSensor for LangCountSensor {
    type Settings = LangCountSettings;
    fn name(&self) -> &str {
        "lang-count"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn run(&self, settings: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let count = sub.lang_summary.len();
        let verdict = if count >= settings.min_languages {
            Verdict::Pass
        } else {
            Verdict::Warn
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "audit"),
            "2024-03-01T00:00:00Z".to_string(),
            verdict,
            format!(
                "{} languages found (min: {})",
                count, settings.min_languages
            ),
        ))
    }
}

/// A sensor that reports diff-file count.
struct DiffSensor;

#[derive(Serialize, Deserialize)]
struct DiffSettings;

impl EffortlessSensor for DiffSensor {
    type Settings = DiffSettings;
    fn name(&self) -> &str {
        "diff-sensor"
    }
    fn version(&self) -> &str {
        "0.3.0"
    }
    fn run(&self, _: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let diff_count = sub.diff_files().count();
        let verdict = if diff_count == 0 {
            Verdict::Skip
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "diff"),
            "2024-09-01T00:00:00Z".to_string(),
            verdict,
            format!("{} files in diff", diff_count),
        ))
    }
}

// ============================================================================
// Helpers
// ============================================================================

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
                code: 50,
                lines: 60,
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
        ],
        lang_summary: BTreeMap::from([
            (
                "Rust".to_string(),
                LangSummary {
                    files: 2,
                    code: 250,
                    lines: 310,
                    bytes: 7500,
                    tokens: 1875,
                },
            ),
            (
                "Python".to_string(),
                LangSummary {
                    files: 1,
                    code: 80,
                    lines: 100,
                    bytes: 2400,
                    tokens: 600,
                },
            ),
        ]),
        diff_range: Some(DiffRange {
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

fn substrate_from_files(files: Vec<SubstrateFile>) -> RepoSubstrate {
    let mut lang_summary: BTreeMap<String, LangSummary> = BTreeMap::new();
    let mut total_tokens = 0usize;
    let mut total_bytes = 0usize;
    let mut total_code_lines = 0usize;

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

        total_tokens += f.tokens;
        total_bytes += f.bytes;
        total_code_lines += f.code;
    }

    RepoSubstrate {
        repo_root: ".".to_string(),
        files,
        lang_summary,
        diff_range: None,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

// ============================================================================
// 1. EffortlessSensor trait implementation validation
// ============================================================================

#[test]
fn pass_sensor_returns_pass() {
    let sensor = PassSensor;
    let report = sensor.run(&PassSettings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert_eq!(report.tool.name, "pass-sensor");
    assert_eq!(report.tool.version, "1.0.0");
}

#[test]
fn threshold_sensor_pass_below_limit() {
    let sensor = ThresholdSensor;
    let settings = ThresholdSettings {
        max_code_lines: 1000,
    };
    let report = sensor.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn threshold_sensor_fail_above_limit() {
    let sensor = ThresholdSensor;
    let settings = ThresholdSettings {
        max_code_lines: 100,
    };
    let report = sensor.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Fail);
}

#[test]
fn lang_count_sensor_pass_enough_languages() {
    let sensor = LangCountSensor;
    let settings = LangCountSettings { min_languages: 2 };
    let report = sensor.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn lang_count_sensor_warn_not_enough() {
    let sensor = LangCountSensor;
    let settings = LangCountSettings { min_languages: 5 };
    let report = sensor.run(&settings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
}

#[test]
fn diff_sensor_pass_with_diff() {
    let sensor = DiffSensor;
    let report = sensor.run(&DiffSettings, &sample_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn diff_sensor_skip_no_diff() {
    let sensor = DiffSensor;
    let report = sensor.run(&DiffSettings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Skip);
}

#[test]
fn sensor_name_and_version_accessors() {
    let s1 = PassSensor;
    assert_eq!(s1.name(), "pass-sensor");
    assert_eq!(s1.version(), "1.0.0");

    let s2 = ThresholdSensor;
    assert_eq!(s2.name(), "threshold");
    assert_eq!(s2.version(), "0.2.0");

    let s3 = LangCountSensor;
    assert_eq!(s3.name(), "lang-count");
    assert_eq!(s3.version(), "0.1.0");

    let s4 = DiffSensor;
    assert_eq!(s4.name(), "diff-sensor");
    assert_eq!(s4.version(), "0.3.0");
}

// ============================================================================
// 2. RepoSubstrate construction and field access
// ============================================================================

#[test]
fn substrate_field_access_totals() {
    let sub = sample_substrate();
    assert_eq!(sub.total_code_lines, 330);
    assert_eq!(sub.total_bytes, 9900);
    assert_eq!(sub.total_tokens, 2475);
}

#[test]
fn substrate_field_access_files() {
    let sub = sample_substrate();
    assert_eq!(sub.files.len(), 3);
    assert_eq!(sub.files[0].path, "src/lib.rs");
    assert_eq!(sub.files[2].lang, "Python");
}

#[test]
fn substrate_diff_files_iterator() {
    let sub = sample_substrate();
    let diff: Vec<_> = sub.diff_files().collect();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().all(|f| f.in_diff));
}

#[test]
fn substrate_files_for_lang() {
    let sub = sample_substrate();
    let rust: Vec<_> = sub.files_for_lang("Rust").collect();
    assert_eq!(rust.len(), 2);
    let python: Vec<_> = sub.files_for_lang("Python").collect();
    assert_eq!(python.len(), 1);
    let go: Vec<_> = sub.files_for_lang("Go").collect();
    assert_eq!(go.len(), 0);
}

#[test]
fn substrate_lang_summary_keys() {
    let sub = sample_substrate();
    let keys: Vec<_> = sub.lang_summary.keys().collect();
    // BTreeMap: alphabetical order
    assert_eq!(keys, vec!["Python", "Rust"]);
}

#[test]
fn substrate_diff_range_fields() {
    let sub = sample_substrate();
    let dr = sub.diff_range.as_ref().unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature");
    assert_eq!(dr.changed_files.len(), 2);
    assert_eq!(dr.commit_count, 5);
    assert_eq!(dr.insertions, 30);
    assert_eq!(dr.deletions, 10);
}

// ============================================================================
// 3. Substrate builder patterns (manual construction)
// ============================================================================

#[test]
fn substrate_from_files_computes_totals() {
    let files = vec![
        SubstrateFile {
            path: "a.rs".to_string(),
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            bytes: 3000,
            tokens: 750,
            module: "root".to_string(),
            in_diff: false,
        },
        SubstrateFile {
            path: "b.py".to_string(),
            lang: "Python".to_string(),
            code: 50,
            lines: 60,
            bytes: 1500,
            tokens: 375,
            module: "root".to_string(),
            in_diff: false,
        },
    ];
    let sub = substrate_from_files(files);
    assert_eq!(sub.total_code_lines, 150);
    assert_eq!(sub.total_bytes, 4500);
    assert_eq!(sub.total_tokens, 1125);
    assert_eq!(sub.lang_summary.len(), 2);
}

#[test]
fn substrate_from_files_empty() {
    let sub = substrate_from_files(vec![]);
    assert_eq!(sub.total_code_lines, 0);
    assert_eq!(sub.total_bytes, 0);
    assert_eq!(sub.total_tokens, 0);
    assert!(sub.files.is_empty());
    assert!(sub.lang_summary.is_empty());
}

#[test]
fn substrate_from_files_lang_aggregation() {
    let files = vec![
        SubstrateFile {
            path: "a.rs".to_string(),
            lang: "Rust".to_string(),
            code: 100,
            lines: 120,
            bytes: 3000,
            tokens: 750,
            module: "src".to_string(),
            in_diff: false,
        },
        SubstrateFile {
            path: "b.rs".to_string(),
            lang: "Rust".to_string(),
            code: 200,
            lines: 250,
            bytes: 6000,
            tokens: 1500,
            module: "src".to_string(),
            in_diff: false,
        },
    ];
    let sub = substrate_from_files(files);
    let rust = sub.lang_summary.get("Rust").unwrap();
    assert_eq!(rust.files, 2);
    assert_eq!(rust.code, 300);
    assert_eq!(rust.lines, 370);
    assert_eq!(rust.bytes, 9000);
    assert_eq!(rust.tokens, 2250);
}

// ============================================================================
// 4. Sensor report envelope format
// ============================================================================

#[test]
fn report_has_schema_v1() {
    let sensor = PassSensor;
    let report = sensor.run(&PassSettings, &empty_substrate()).unwrap();
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(report.schema, "sensor.report.v1");
}

#[test]
fn report_serde_roundtrip() {
    let sensor = ThresholdSensor;
    let settings = ThresholdSettings {
        max_code_lines: 500,
    };
    let report = sensor.run(&settings, &sample_substrate()).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, report.schema);
    assert_eq!(back.verdict, report.verdict);
    assert_eq!(back.tool.name, report.tool.name);
    assert_eq!(back.summary, report.summary);
}

#[test]
fn report_with_findings() {
    let mut report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Warn,
        "has findings".to_string(),
    );
    report.findings.push(Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "Hot file detected",
        "src/lib.rs is frequently modified".to_string(),
    ));
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].check_id, "risk");
    assert_eq!(report.findings[0].code, "hotspot");
}

#[test]
fn report_with_artifacts() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "artifacts test".to_string(),
    )
    .with_artifacts(vec![
        Artifact::receipt("output.json"),
        Artifact::receipt("report.md"),
    ]);
    let arts = report.artifacts.as_ref().unwrap();
    assert_eq!(arts.len(), 2);
}

#[test]
fn report_with_capabilities() {
    let mut caps = BTreeMap::new();
    caps.insert(
        "git".to_string(),
        CapabilityStatus {
            status: CapabilityState::Available,
            reason: None,
        },
    );
    caps.insert(
        "content".to_string(),
        CapabilityStatus {
            status: CapabilityState::Unavailable,
            reason: Some("feature disabled".to_string()),
        },
    );
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "caps test".to_string(),
    )
    .with_capabilities(caps.clone());
    let report_caps = report.capabilities.as_ref().unwrap();
    assert_eq!(report_caps.len(), 2);
    assert_eq!(
        report_caps.get("git").unwrap().status,
        CapabilityState::Available
    );
    assert_eq!(
        report_caps.get("content").unwrap().status,
        CapabilityState::Unavailable
    );
}

#[test]
fn report_with_data_payload() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "data test".to_string(),
    )
    .with_data(serde_json::json!({"lines": 100, "quality": "high"}));
    let data = report.data.as_ref().unwrap();
    assert_eq!(data["lines"], 100);
    assert_eq!(data["quality"], "high");
}

// ============================================================================
// 5. Property: sensor reports always have required fields
// ============================================================================

fn arb_substrate_file() -> impl Strategy<Value = SubstrateFile> {
    (
        "[a-z]{1,5}/[a-z]{1,5}\\.[a-z]{1,3}",
        prop_oneof!["Rust", "Python", "Go", "JavaScript"],
        0usize..5_000,
        any::<bool>(),
    )
        .prop_map(|(path, lang, code, in_diff)| SubstrateFile {
            path,
            lang,
            code,
            lines: code + code / 5,
            bytes: code * 30,
            tokens: code * 4,
            module: "mod".to_string(),
            in_diff,
        })
}

fn arb_substrate() -> impl Strategy<Value = RepoSubstrate> {
    prop::collection::vec(arb_substrate_file(), 0..20).prop_map(substrate_from_files)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn prop_pass_sensor_always_has_schema(sub in arb_substrate()) {
        let sensor = PassSensor;
        let report = sensor.run(&PassSettings, &sub).unwrap();
        prop_assert_eq!(&report.schema, SENSOR_REPORT_SCHEMA);
        prop_assert!(!report.tool.name.is_empty());
        prop_assert!(!report.tool.version.is_empty());
        prop_assert!(!report.generated_at.is_empty());
    }

    #[test]
    fn prop_threshold_sensor_verdict_consistent(
        sub in arb_substrate(),
        max in 0usize..10_000
    ) {
        let sensor = ThresholdSensor;
        let settings = ThresholdSettings { max_code_lines: max };
        let report = sensor.run(&settings, &sub).unwrap();
        if sub.total_code_lines > max {
            prop_assert_eq!(report.verdict, Verdict::Fail);
        } else {
            prop_assert_eq!(report.verdict, Verdict::Pass);
        }
    }

    #[test]
    fn prop_lang_count_sensor_consistent(
        sub in arb_substrate(),
        min in 0usize..10
    ) {
        let sensor = LangCountSensor;
        let settings = LangCountSettings { min_languages: min };
        let report = sensor.run(&settings, &sub).unwrap();
        if sub.lang_summary.len() >= min {
            prop_assert_eq!(report.verdict, Verdict::Pass);
        } else {
            prop_assert_eq!(report.verdict, Verdict::Warn);
        }
    }

    #[test]
    fn prop_diff_sensor_skip_when_no_diff(
        files in prop::collection::vec(arb_substrate_file(), 0..10)
    ) {
        let files_no_diff: Vec<SubstrateFile> = files.into_iter().map(|mut f| {
            f.in_diff = false;
            f
        }).collect();
        let sub = substrate_from_files(files_no_diff);
        let sensor = DiffSensor;
        let report = sensor.run(&DiffSettings, &sub).unwrap();
        prop_assert_eq!(report.verdict, Verdict::Skip);
    }

    #[test]
    fn prop_report_serde_roundtrip(sub in arb_substrate()) {
        let sensor = PassSensor;
        let report = sensor.run(&PassSettings, &sub).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.schema, &report.schema);
        prop_assert_eq!(back.verdict, report.verdict);
        prop_assert_eq!(&back.tool.name, &report.tool.name);
    }

    #[test]
    fn prop_substrate_totals_consistent(sub in arb_substrate()) {
        let sum_code: usize = sub.files.iter().map(|f| f.code).sum();
        let sum_bytes: usize = sub.files.iter().map(|f| f.bytes).sum();
        let sum_tokens: usize = sub.files.iter().map(|f| f.tokens).sum();
        prop_assert_eq!(sub.total_code_lines, sum_code);
        prop_assert_eq!(sub.total_bytes, sum_bytes);
        prop_assert_eq!(sub.total_tokens, sum_tokens);
    }

    #[test]
    fn prop_substrate_lang_summary_consistent(sub in arb_substrate()) {
        for (lang, summary) in &sub.lang_summary {
            let lang_files: Vec<_> = sub.files_for_lang(lang).collect();
            prop_assert_eq!(summary.files, lang_files.len());
            let code_sum: usize = lang_files.iter().map(|f| f.code).sum();
            prop_assert_eq!(summary.code, code_sum);
        }
    }
}

// ============================================================================
// 6. Edge cases: empty substrate, missing optional fields
// ============================================================================

#[test]
fn edge_empty_substrate_totals_zero() {
    let sub = empty_substrate();
    assert_eq!(sub.total_code_lines, 0);
    assert_eq!(sub.total_bytes, 0);
    assert_eq!(sub.total_tokens, 0);
    assert!(sub.files.is_empty());
    assert!(sub.lang_summary.is_empty());
}

#[test]
fn edge_empty_substrate_no_diff_range() {
    let sub = empty_substrate();
    assert!(sub.diff_range.is_none());
}

#[test]
fn edge_empty_substrate_diff_files_empty() {
    let sub = empty_substrate();
    assert_eq!(sub.diff_files().count(), 0);
}

#[test]
fn edge_empty_substrate_files_for_lang_empty() {
    let sub = empty_substrate();
    assert_eq!(sub.files_for_lang("Rust").count(), 0);
}

#[test]
fn edge_sensor_on_empty_substrate() {
    let sensor = ThresholdSensor;
    let settings = ThresholdSettings { max_code_lines: 0 };
    // 0 > 0 is false, so should pass
    let report = sensor.run(&settings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn edge_report_no_findings() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "no findings".to_string(),
    );
    assert!(report.findings.is_empty());
    assert!(report.artifacts.is_none());
    assert!(report.capabilities.is_none());
    assert!(report.data.is_none());
}

#[test]
fn edge_report_serde_omits_none_fields() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "minimal".to_string(),
    );
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("artifacts"));
    assert!(!json.contains("capabilities"));
    assert!(!json.contains("\"data\""));
}

#[test]
fn edge_substrate_serde_roundtrip_empty() {
    let sub = empty_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 0);
    assert_eq!(back.total_code_lines, 0);
    assert!(back.diff_range.is_none());
}

#[test]
fn edge_substrate_serde_roundtrip_with_diff() {
    let sub = sample_substrate();
    let json = serde_json::to_string(&sub).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 3);
    assert_eq!(back.total_code_lines, 330);
    let dr = back.diff_range.as_ref().unwrap();
    assert_eq!(dr.changed_files.len(), 2);
}

#[test]
fn edge_verdict_default_is_pass() {
    let v: Verdict = Default::default();
    assert_eq!(v, Verdict::Pass);
}

#[test]
fn edge_all_verdicts_serde_roundtrip() {
    for verdict in [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ] {
        let json = serde_json::to_string(&verdict).unwrap();
        let back: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, verdict);
    }
}
