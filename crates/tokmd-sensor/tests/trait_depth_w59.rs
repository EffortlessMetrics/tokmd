//! Depth tests for the `EffortlessSensor` trait contract.
//!
//! Covers: trait implementation patterns, verdict logic, settings serde,
//! report construction, edge cases, and determinism.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA, SensorReport,
    ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ── Helpers ──────────────────────────────────────────────────────

fn make_file(path: &str, lang: &str, code: usize) -> SubstrateFile {
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
            .unwrap_or("")
            .to_string(),
        in_diff: false,
    }
}

fn make_file_in_diff(path: &str, lang: &str, code: usize) -> SubstrateFile {
    let mut f = make_file(path, lang, code);
    f.in_diff = true;
    f
}

fn substrate_from_files(files: Vec<SubstrateFile>) -> RepoSubstrate {
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
        diff_range: None,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

fn empty_substrate() -> RepoSubstrate {
    substrate_from_files(vec![])
}

fn single_file_substrate() -> RepoSubstrate {
    substrate_from_files(vec![make_file("src/lib.rs", "Rust", 100)])
}

fn multi_lang_substrate() -> RepoSubstrate {
    substrate_from_files(vec![
        make_file("src/lib.rs", "Rust", 200),
        make_file("src/main.py", "Python", 150),
        make_file("src/app.ts", "TypeScript", 300),
        make_file_in_diff("src/new.rs", "Rust", 50),
    ])
}

// ── Test sensors ─────────────────────────────────────────────────

/// Threshold-based sensor that warns when code exceeds a limit.
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
        "1.0.0"
    }
    fn run(&self, settings: &ThresholdSettings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let verdict = if sub.total_code_lines > settings.max_code_lines {
            Verdict::Fail
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-06-01T00:00:00Z".to_string(),
            verdict,
            format!("{} lines", sub.total_code_lines),
        ))
    }
}

/// Language-counting sensor that produces findings per language.
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
        "0.2.0"
    }
    fn run(&self, settings: &LangCountSettings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let count = sub.lang_summary.len();
        let verdict = if count >= settings.min_languages {
            Verdict::Pass
        } else {
            Verdict::Warn
        };
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "analyze"),
            "2024-06-01T00:00:00Z".to_string(),
            verdict,
            format!("{count} languages found"),
        );
        for lang in sub.lang_summary.keys() {
            report.add_finding(Finding::new(
                "inventory",
                "language",
                FindingSeverity::Info,
                lang,
                format!("Language {lang} detected"),
            ));
        }
        Ok(report)
    }
}

/// Diff-aware sensor that only cares about changed files.
struct DiffSensor;

#[derive(Serialize, Deserialize)]
struct DiffSettings;

impl EffortlessSensor for DiffSensor {
    type Settings = DiffSettings;
    fn name(&self) -> &str {
        "diff-sensor"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn run(&self, _settings: &DiffSettings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let diff_count = sub.diff_files().count();
        let verdict = if diff_count == 0 {
            Verdict::Skip
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "diff"),
            "2024-06-01T00:00:00Z".to_string(),
            verdict,
            format!("{diff_count} files in diff"),
        ))
    }
}

/// Sensor that always fails for error path testing.
struct FailingSensor;

#[derive(Serialize, Deserialize)]
struct FailSettings {
    should_fail: bool,
}

impl EffortlessSensor for FailingSensor {
    type Settings = FailSettings;
    fn name(&self) -> &str {
        "failing"
    }
    fn version(&self) -> &str {
        "0.0.1"
    }
    fn run(&self, settings: &FailSettings, _sub: &RepoSubstrate) -> Result<SensorReport> {
        if settings.should_fail {
            anyhow::bail!("intentional failure");
        }
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-06-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "ok".to_string(),
        ))
    }
}

// ── Trait contract tests ─────────────────────────────────────────

#[test]
fn sensor_name_is_stable() {
    let s = ThresholdSensor;
    assert_eq!(s.name(), s.name(), "name() must be pure");
}

#[test]
fn sensor_version_is_stable() {
    let s = ThresholdSensor;
    assert_eq!(s.version(), s.version(), "version() must be pure");
}

#[test]
fn threshold_pass_below_limit() {
    let sub = single_file_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 500,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn threshold_fail_above_limit() {
    let sub = multi_lang_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 100,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Fail);
}

#[test]
fn threshold_pass_at_exact_boundary() {
    let sub = single_file_substrate(); // 100 lines
    let settings = ThresholdSettings {
        max_code_lines: 100,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass, "equal should pass (not >)");
}

#[test]
fn threshold_on_empty_substrate() {
    let sub = empty_substrate();
    let settings = ThresholdSettings { max_code_lines: 0 };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

// ── Lang-count sensor tests ─────────────────────────────────────

#[test]
fn lang_count_pass_when_enough() {
    let sub = multi_lang_substrate();
    let settings = LangCountSettings { min_languages: 2 };
    let report = LangCountSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert_eq!(report.findings.len(), 3); // Rust, Python, TypeScript
}

#[test]
fn lang_count_warn_when_too_few() {
    let sub = single_file_substrate();
    let settings = LangCountSettings { min_languages: 5 };
    let report = LangCountSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
}

#[test]
fn lang_count_empty_substrate() {
    let sub = empty_substrate();
    let settings = LangCountSettings { min_languages: 1 };
    let report = LangCountSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Warn);
    assert!(report.findings.is_empty());
}

#[test]
fn lang_count_findings_match_languages() {
    let sub = multi_lang_substrate();
    let settings = LangCountSettings { min_languages: 1 };
    let report = LangCountSensor.run(&settings, &sub).unwrap();
    let finding_titles: Vec<&str> = report.findings.iter().map(|f| f.title.as_str()).collect();
    // BTreeMap ensures alphabetical order
    assert_eq!(finding_titles, vec!["Python", "Rust", "TypeScript"]);
}

// ── Diff-aware sensor tests ─────────────────────────────────────

#[test]
fn diff_sensor_skip_when_no_diff_files() {
    let sub = single_file_substrate(); // no in_diff files
    let report = DiffSensor.run(&DiffSettings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Skip);
}

#[test]
fn diff_sensor_pass_when_diff_files_exist() {
    let sub = multi_lang_substrate(); // has one in_diff file
    let report = DiffSensor.run(&DiffSettings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.summary.contains("1 files in diff"));
}

// ── Error path tests ─────────────────────────────────────────────

#[test]
fn failing_sensor_returns_error() {
    let sub = empty_substrate();
    let settings = FailSettings { should_fail: true };
    let result = FailingSensor.run(&settings, &sub);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("intentional failure"));
}

#[test]
fn failing_sensor_succeeds_when_not_failing() {
    let sub = empty_substrate();
    let settings = FailSettings { should_fail: false };
    let report = FailingSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

// ── Report envelope correctness ──────────────────────────────────

#[test]
fn report_has_correct_schema() {
    let sub = single_file_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 1000,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn report_tool_meta_matches_sensor() {
    let sub = single_file_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 1000,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    assert_eq!(report.tool.name, "threshold");
    assert_eq!(report.tool.version, "1.0.0");
    assert_eq!(report.tool.mode, "check");
}

#[test]
fn report_findings_empty_by_default() {
    let sub = single_file_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 1000,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn report_serde_roundtrip() {
    let sub = multi_lang_substrate();
    let settings = LangCountSettings { min_languages: 1 };
    let report = LangCountSensor.run(&settings, &sub).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.verdict, report.verdict);
    assert_eq!(back.findings.len(), report.findings.len());
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn report_with_artifacts() {
    let sub = single_file_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 1000,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    let report = report.with_artifacts(vec![Artifact::receipt("out/receipt.json")]);
    assert!(report.artifacts.is_some());
    assert_eq!(report.artifacts.as_ref().unwrap().len(), 1);
}

#[test]
fn report_with_capabilities() {
    let sub = single_file_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 1000,
    };
    let mut report = ThresholdSensor.run(&settings, &sub).unwrap();
    report.add_capability("git", CapabilityStatus::available());
    report.add_capability("content", CapabilityStatus::unavailable("not compiled"));
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 2);
    assert!(caps.contains_key("git"));
    assert!(caps.contains_key("content"));
}

#[test]
fn report_with_data_payload() {
    let sub = single_file_substrate();
    let settings = ThresholdSettings {
        max_code_lines: 1000,
    };
    let report = ThresholdSensor.run(&settings, &sub).unwrap();
    let report = report.with_data(serde_json::json!({"extra": true}));
    assert!(report.data.is_some());
}

// ── Determinism tests ────────────────────────────────────────────

#[test]
fn same_inputs_produce_identical_reports() {
    let sub = multi_lang_substrate();
    let settings = LangCountSettings { min_languages: 1 };
    let r1 = LangCountSensor.run(&settings, &sub).unwrap();
    let r2 = LangCountSensor.run(&settings, &sub).unwrap();
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "deterministic: same inputs → same JSON");
}

#[test]
fn finding_order_is_deterministic() {
    let sub = multi_lang_substrate();
    let settings = LangCountSettings { min_languages: 1 };
    let r1 = LangCountSensor.run(&settings, &sub).unwrap();
    let r2 = LangCountSensor.run(&settings, &sub).unwrap();
    let titles1: Vec<&str> = r1.findings.iter().map(|f| f.title.as_str()).collect();
    let titles2: Vec<&str> = r2.findings.iter().map(|f| f.title.as_str()).collect();
    assert_eq!(titles1, titles2);
}

// ── Settings serde tests ─────────────────────────────────────────

#[test]
fn threshold_settings_roundtrip() {
    let s = ThresholdSettings { max_code_lines: 42 };
    let json = serde_json::to_string(&s).unwrap();
    let back: ThresholdSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_code_lines, 42);
}

#[test]
fn diff_settings_unit_struct_roundtrip() {
    let s = DiffSettings;
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(json, "null");
    let _back: DiffSettings = serde_json::from_str(&json).unwrap();
}

// ── Multiple sensors on same substrate ───────────────────────────

#[test]
fn multiple_sensors_share_substrate() {
    let sub = multi_lang_substrate();
    let r1 = ThresholdSensor
        .run(
            &ThresholdSettings {
                max_code_lines: 1000,
            },
            &sub,
        )
        .unwrap();
    let r2 = LangCountSensor
        .run(&LangCountSettings { min_languages: 1 }, &sub)
        .unwrap();
    let r3 = DiffSensor.run(&DiffSettings, &sub).unwrap();

    // All sensors ran on the same substrate without issues
    assert_eq!(r1.verdict, Verdict::Pass);
    assert_eq!(r2.verdict, Verdict::Pass);
    assert_eq!(r3.verdict, Verdict::Pass);

    // Substrate is unchanged (immutable borrow)
    assert_eq!(sub.total_code_lines, 700);
    assert_eq!(sub.lang_summary.len(), 3);
}

#[test]
fn substrate_with_diff_range_propagates_to_sensors() {
    let mut sub = multi_lang_substrate();
    sub.diff_range = Some(DiffRange {
        base: "main".to_string(),
        head: "feature".to_string(),
        changed_files: vec!["src/new.rs".to_string()],
        commit_count: 2,
        insertions: 10,
        deletions: 3,
    });
    let report = DiffSensor.run(&DiffSettings, &sub).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}
