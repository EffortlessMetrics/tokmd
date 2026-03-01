//! Integration tests for substrate building and sensor execution.
//!
//! These tests exercise the `build_substrate` function against real directories
//! and validate the full sensor pipeline end-to-end.

use std::collections::BTreeMap;
use std::fs;

use anyhow::Result;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA, SensorReport,
    ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate_builder::build_substrate;
use tokmd_settings::ScanOptions;
use tokmd_substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Threshold sensor (also used in proptest section)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Mock sensor for integration tests
// ---------------------------------------------------------------------------

struct IntegrationSensor;

#[derive(Serialize, Deserialize)]
struct IntegrationSettings {
    min_languages: usize,
}

impl EffortlessSensor for IntegrationSensor {
    type Settings = IntegrationSettings;

    fn name(&self) -> &str {
        "integration-sensor"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn run(&self, settings: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let lang_count = sub.lang_summary.len();
        let verdict = if lang_count >= settings.min_languages {
            Verdict::Pass
        } else {
            Verdict::Warn
        };
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "analyze"),
            "2024-06-15T12:00:00Z".to_string(),
            verdict,
            format!(
                "{} languages, {} files, {} code lines",
                lang_count,
                sub.files.len(),
                sub.total_code_lines
            ),
        );
        if lang_count < settings.min_languages {
            report.add_finding(Finding::new(
                "diversity",
                "low-lang-count",
                FindingSeverity::Info,
                "Low language diversity",
                format!(
                    "Found {} languages, expected at least {}",
                    lang_count, settings.min_languages
                ),
            ));
        }
        report.add_capability("lang-check", CapabilityStatus::available());
        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Substrate builder: scan crate's own source
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_returns_valid_substrate_for_own_src() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    // The substrate should have scanned Rust files
    assert!(!substrate.files.is_empty(), "should find at least one file");
    assert!(
        substrate.lang_summary.contains_key("Rust"),
        "should detect Rust"
    );
    assert!(substrate.total_code_lines > 0);
    assert!(substrate.total_bytes > 0);
    assert!(substrate.total_tokens > 0);
    assert!(substrate.diff_range.is_none());
}

#[test]
fn build_substrate_totals_are_consistent() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    // Totals must equal the sum across files
    let sum_code: usize = substrate.files.iter().map(|f| f.code).sum();
    let sum_bytes: usize = substrate.files.iter().map(|f| f.bytes).sum();
    let sum_tokens: usize = substrate.files.iter().map(|f| f.tokens).sum();
    assert_eq!(substrate.total_code_lines, sum_code);
    assert_eq!(substrate.total_bytes, sum_bytes);
    assert_eq!(substrate.total_tokens, sum_tokens);
}

#[test]
fn build_substrate_lang_summary_matches_files() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    // For each language in the summary, file counts and code totals must match
    for (lang, summary) in &substrate.lang_summary {
        let lang_files: Vec<_> = substrate.files.iter().filter(|f| &f.lang == lang).collect();
        assert_eq!(
            summary.files,
            lang_files.len(),
            "file count mismatch for {}",
            lang
        );
        let lang_code: usize = lang_files.iter().map(|f| f.code).sum();
        assert_eq!(summary.code, lang_code, "code sum mismatch for {}", lang);
        let lang_bytes: usize = lang_files.iter().map(|f| f.bytes).sum();
        assert_eq!(summary.bytes, lang_bytes, "bytes mismatch for {}", lang);
        let lang_tokens: usize = lang_files.iter().map(|f| f.tokens).sum();
        assert_eq!(summary.tokens, lang_tokens, "tokens mismatch for {}", lang);
    }
}

// ---------------------------------------------------------------------------
// Substrate builder: missing directory
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_errors_on_nonexistent_directory() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nonexistent");
    let result = build_substrate(
        missing.to_string_lossy().as_ref(),
        &ScanOptions::default(),
        &[],
        2,
        None,
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Substrate builder: empty directory
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_returns_empty_for_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let substrate = build_substrate(
        dir.path().to_string_lossy().as_ref(),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    assert!(substrate.files.is_empty());
    assert!(substrate.lang_summary.is_empty());
    assert_eq!(substrate.total_code_lines, 0);
    assert_eq!(substrate.total_bytes, 0);
    assert_eq!(substrate.total_tokens, 0);
}

// ---------------------------------------------------------------------------
// Substrate builder: with diff range
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_marks_diff_files() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let diff = DiffRange {
        base: "main".to_string(),
        head: "HEAD".to_string(),
        changed_files: vec!["src/lib.rs".to_string()],
        commit_count: 1,
        insertions: 3,
        deletions: 1,
    };
    let substrate =
        build_substrate(manifest_dir, &ScanOptions::default(), &[], 2, Some(diff)).unwrap();

    assert!(substrate.diff_range.is_some());
    let diff_files: Vec<&str> = substrate
        .files
        .iter()
        .filter(|f| f.in_diff)
        .map(|f| f.path.as_str())
        .collect();
    assert!(
        diff_files.contains(&"src/lib.rs"),
        "src/lib.rs should be marked in diff"
    );

    // Files not in the changed list should NOT be in diff
    let non_diff: Vec<&str> = substrate
        .files
        .iter()
        .filter(|f| !f.in_diff)
        .map(|f| f.path.as_str())
        .collect();
    for path in &non_diff {
        assert_ne!(*path, "src/lib.rs");
    }
}

#[test]
fn build_substrate_diff_range_no_matching_files() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let diff = DiffRange {
        base: "main".to_string(),
        head: "HEAD".to_string(),
        changed_files: vec!["nonexistent/file.rs".to_string()],
        commit_count: 1,
        insertions: 0,
        deletions: 0,
    };
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        Some(diff),
    )
    .unwrap();

    // No files should be marked in_diff
    let diff_count = substrate.files.iter().filter(|f| f.in_diff).count();
    assert_eq!(diff_count, 0, "no files should match a nonexistent path");
}

// ---------------------------------------------------------------------------
// Substrate builder: temp dir with known content
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_scans_temp_dir_with_rust_file() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    let substrate = build_substrate(
        dir.path().to_string_lossy().as_ref(),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    assert!(!substrate.files.is_empty());
    assert!(substrate.lang_summary.contains_key("Rust"));
    assert!(substrate.total_code_lines > 0);
}

#[test]
fn build_substrate_scans_multiple_languages() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("script.py"),
        "def hello():\n    print('hello')\n",
    )
    .unwrap();

    let substrate = build_substrate(
        dir.path().to_string_lossy().as_ref(),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    assert!(
        substrate.lang_summary.len() >= 2,
        "expected at least Rust and Python"
    );
    assert!(substrate.lang_summary.contains_key("Rust"));
    assert!(substrate.lang_summary.contains_key("Python"));
}

// ---------------------------------------------------------------------------
// End-to-end: build substrate → run sensor
// ---------------------------------------------------------------------------

#[test]
fn end_to_end_substrate_to_sensor_report() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let sensor = IntegrationSensor;
    let settings = IntegrationSettings { min_languages: 1 };
    let report = sensor.run(&settings, &substrate).unwrap();

    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(report.tool.name, "integration-sensor");
    assert_eq!(report.tool.version, "1.0.0");
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.summary.contains("Rust") || report.summary.contains("1 languages"));
    assert!(report.findings.is_empty());
    let caps = report.capabilities.unwrap();
    assert!(caps.contains_key("lang-check"));
}

#[test]
fn end_to_end_sensor_warns_on_low_diversity() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let sensor = IntegrationSensor;
    // Set min_languages very high to trigger warning
    let settings = IntegrationSettings { min_languages: 100 };
    let report = sensor.run(&settings, &substrate).unwrap();

    assert_eq!(report.verdict, Verdict::Warn);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].check_id, "diversity");
    assert_eq!(report.findings[0].code, "low-lang-count");
}

// ---------------------------------------------------------------------------
// End-to-end: empty substrate → sensor report
// ---------------------------------------------------------------------------

#[test]
fn end_to_end_empty_substrate_sensor_warns() {
    let dir = tempfile::tempdir().unwrap();
    let substrate = build_substrate(
        dir.path().to_string_lossy().as_ref(),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let sensor = IntegrationSensor;
    let settings = IntegrationSettings { min_languages: 1 };
    let report = sensor.run(&settings, &substrate).unwrap();

    // 0 languages < 1 required
    assert_eq!(report.verdict, Verdict::Warn);
    assert_eq!(report.findings.len(), 1);
    assert!(report.summary.contains("0 languages"));
}

// ---------------------------------------------------------------------------
// Substrate serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn substrate_serde_roundtrip() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let json = serde_json::to_string(&substrate).unwrap();
    let restored: RepoSubstrate = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.files.len(), substrate.files.len());
    assert_eq!(restored.total_code_lines, substrate.total_code_lines);
    assert_eq!(restored.total_bytes, substrate.total_bytes);
    assert_eq!(restored.total_tokens, substrate.total_tokens);
    assert_eq!(restored.lang_summary.len(), substrate.lang_summary.len());
}

// ---------------------------------------------------------------------------
// Substrate file paths use forward slashes
// ---------------------------------------------------------------------------

#[test]
fn substrate_paths_use_forward_slashes() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(manifest_dir, &ScanOptions::default(), &[], 2, None).unwrap();

    for f in &substrate.files {
        assert!(
            !f.path.contains('\\'),
            "path should use forward slashes: {}",
            f.path
        );
    }
}

// ---------------------------------------------------------------------------
// Multiple sensors on same substrate
// ---------------------------------------------------------------------------

struct CountSensor;

#[derive(Serialize, Deserialize)]
struct CountSettings;

impl EffortlessSensor for CountSensor {
    type Settings = CountSettings;

    fn name(&self) -> &str {
        "count-sensor"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn run(&self, _: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "count"),
            "2024-06-15T12:00:00Z".to_string(),
            Verdict::Pass,
            format!("{} files", sub.files.len()),
        ))
    }
}

#[test]
fn multiple_sensors_share_same_substrate() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    // Run two different sensors on the same substrate
    let sensor_a = IntegrationSensor;
    let report_a = sensor_a
        .run(&IntegrationSettings { min_languages: 1 }, &substrate)
        .unwrap();

    let sensor_b = CountSensor;
    let report_b = sensor_b.run(&CountSettings, &substrate).unwrap();

    // Both should succeed with different tool metadata
    assert_eq!(report_a.tool.name, "integration-sensor");
    assert_eq!(report_b.tool.name, "count-sensor");
    assert_eq!(report_a.verdict, Verdict::Pass);
    assert_eq!(report_b.verdict, Verdict::Pass);
}

// ---------------------------------------------------------------------------
// Property-based tests
// ---------------------------------------------------------------------------

fn arb_substrate_file() -> impl Strategy<Value = SubstrateFile> {
    (
        "[a-z]{1,5}/[a-z]{1,5}\\.[a-z]{1,3}",
        prop_oneof!["Rust", "Python", "Go", "JavaScript"],
        0usize..10_000,
    )
        .prop_map(|(path, lang, code)| SubstrateFile {
            path,
            lang,
            code,
            lines: code + code / 5,
            bytes: code * 30,
            tokens: code * 4,
            module: "mod".to_string(),
            in_diff: false,
        })
}

fn arb_substrate() -> impl Strategy<Value = RepoSubstrate> {
    prop::collection::vec(arb_substrate_file(), 0..20).prop_map(|files| {
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
        let total_tokens = files.iter().map(|f| f.tokens).sum();
        let total_bytes = files.iter().map(|f| f.bytes).sum();
        let total_code_lines = files.iter().map(|f| f.code).sum();
        RepoSubstrate {
            repo_root: ".".to_string(),
            files,
            lang_summary,
            diff_range: None,
            total_tokens,
            total_bytes,
            total_code_lines,
        }
    })
}

proptest! {
    /// The LocThreshold sensor never panics on any substrate.
    #[test]
    fn prop_loc_sensor_never_panics(sub in arb_substrate()) {
        let sensor = LocThresholdSensor;
        let settings = LocThresholdSettings { warn_threshold: 500, fail_threshold: 1000 };
        let _report = sensor.run(&settings, &sub).unwrap();
    }

    /// The verdict is deterministic for a given substrate.
    #[test]
    fn prop_sensor_verdict_is_deterministic(sub in arb_substrate()) {
        let sensor = LocThresholdSensor;
        let settings = LocThresholdSettings { warn_threshold: 500, fail_threshold: 1000 };
        let r1 = sensor.run(&settings, &sub).unwrap();
        let r2 = sensor.run(&settings, &sub).unwrap();
        prop_assert_eq!(r1.verdict, r2.verdict);
    }

    /// The sensor report always has the correct schema.
    #[test]
    fn prop_report_schema_is_stable(sub in arb_substrate()) {
        let sensor = LocThresholdSensor;
        let settings = LocThresholdSettings { warn_threshold: 100, fail_threshold: 500 };
        let report = sensor.run(&settings, &sub).unwrap();
        prop_assert_eq!(report.schema.as_str(), SENSOR_REPORT_SCHEMA);
        prop_assert_eq!(report.tool.name, "loc-threshold");
    }

    /// Substrate totals equal the sum of file-level values.
    #[test]
    fn prop_substrate_totals_are_sums(sub in arb_substrate()) {
        let sum_code: usize = sub.files.iter().map(|f| f.code).sum();
        let sum_bytes: usize = sub.files.iter().map(|f| f.bytes).sum();
        let sum_tokens: usize = sub.files.iter().map(|f| f.tokens).sum();
        prop_assert_eq!(sub.total_code_lines, sum_code);
        prop_assert_eq!(sub.total_bytes, sum_bytes);
        prop_assert_eq!(sub.total_tokens, sum_tokens);
    }

    /// The substrate survives a JSON roundtrip.
    #[test]
    fn prop_substrate_serde_roundtrip(sub in arb_substrate()) {
        let json = serde_json::to_string(&sub).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.files.len(), sub.files.len());
        prop_assert_eq!(back.total_code_lines, sub.total_code_lines);
        prop_assert_eq!(back.lang_summary.len(), sub.lang_summary.len());
    }

    /// A sensor with a very high threshold always passes.
    #[test]
    fn prop_high_threshold_always_passes(sub in arb_substrate()) {
        let sensor = LocThresholdSensor;
        let settings = LocThresholdSettings {
            warn_threshold: usize::MAX,
            fail_threshold: usize::MAX,
        };
        let report = sensor.run(&settings, &sub).unwrap();
        prop_assert_eq!(report.verdict, Verdict::Pass);
    }

    /// A sensor with threshold 0 never passes (unless substrate has 0 code lines).
    #[test]
    fn prop_zero_threshold_warns_or_fails(sub in arb_substrate()) {
        let sensor = LocThresholdSensor;
        let settings = LocThresholdSettings {
            warn_threshold: 0,
            fail_threshold: 0,
        };
        let report = sensor.run(&settings, &sub).unwrap();
        // 0 >= 0 is true, so fail_threshold triggers Fail
        prop_assert_eq!(report.verdict, Verdict::Fail);
    }
}

/// Sensor that attaches per-language findings for coverage reporting.
struct LangCoverageSensor;

#[derive(Serialize, Deserialize)]
struct LangCoverageSettings;

impl EffortlessSensor for LangCoverageSensor {
    type Settings = LangCoverageSettings;

    fn name(&self) -> &str {
        "lang-coverage"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn run(&self, _: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "audit"),
            "2024-06-15T12:00:00Z".to_string(),
            Verdict::Pass,
            format!("{} languages audited", sub.lang_summary.len()),
        );
        for (lang, summary) in &sub.lang_summary {
            report.add_finding(Finding::new(
                "coverage",
                "lang-stats",
                FindingSeverity::Info,
                format!("{} stats", lang),
                format!(
                    "{}: {} files, {} code lines",
                    lang, summary.files, summary.code
                ),
            ));
        }
        Ok(report)
    }
}

#[test]
fn lang_coverage_sensor_produces_per_language_findings() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("app.rs"),
        "fn main() {\n    println!(\"hi\");\n}\n",
    )
    .unwrap();
    fs::write(dir.path().join("util.py"), "def util():\n    return 42\n").unwrap();

    let substrate = build_substrate(
        dir.path().to_string_lossy().as_ref(),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let sensor = LangCoverageSensor;
    let report = sensor.run(&LangCoverageSettings, &substrate).unwrap();

    assert_eq!(report.verdict, Verdict::Pass);
    // Should have one finding per language
    assert!(
        report.findings.len() >= 2,
        "expected at least 2 findings, got {}",
        report.findings.len()
    );
    let codes: Vec<&str> = report.findings.iter().map(|f| f.code.as_str()).collect();
    assert!(codes.iter().all(|c| *c == "lang-stats"));
}

// ---------------------------------------------------------------------------
// Substrate builder: deterministic output
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_is_deterministic() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let opts = ScanOptions::default();
    let s1 = build_substrate(&format!("{}/src", manifest_dir), &opts, &[], 2, None).unwrap();
    let s2 = build_substrate(&format!("{}/src", manifest_dir), &opts, &[], 2, None).unwrap();

    // File lists must be identical
    assert_eq!(s1.files.len(), s2.files.len());
    for (a, b) in s1.files.iter().zip(s2.files.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.tokens, b.tokens);
    }
    // Totals must be identical
    assert_eq!(s1.total_code_lines, s2.total_code_lines);
    assert_eq!(s1.total_bytes, s2.total_bytes);
    assert_eq!(s1.total_tokens, s2.total_tokens);
    // Lang summary keys must be identical
    assert_eq!(
        s1.lang_summary.keys().collect::<Vec<_>>(),
        s2.lang_summary.keys().collect::<Vec<_>>()
    );
    // JSON must be byte-identical
    let j1 = serde_json::to_string(&s1).unwrap();
    let j2 = serde_json::to_string(&s2).unwrap();
    assert_eq!(j1, j2);
}

// ---------------------------------------------------------------------------
// Substrate with DiffRange survives serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn substrate_with_diff_range_serde_roundtrip() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let diff = DiffRange {
        base: "v1.0.0".to_string(),
        head: "v2.0.0".to_string(),
        changed_files: vec!["src/lib.rs".to_string()],
        commit_count: 7,
        insertions: 42,
        deletions: 13,
    };
    let substrate =
        build_substrate(manifest_dir, &ScanOptions::default(), &[], 2, Some(diff)).unwrap();

    let json = serde_json::to_string_pretty(&substrate).unwrap();
    let restored: RepoSubstrate = serde_json::from_str(&json).unwrap();

    // Core fields
    assert_eq!(restored.files.len(), substrate.files.len());
    assert_eq!(restored.total_code_lines, substrate.total_code_lines);
    // DiffRange preserved
    let dr = restored
        .diff_range
        .as_ref()
        .expect("diff_range should survive");
    assert_eq!(dr.base, "v1.0.0");
    assert_eq!(dr.head, "v2.0.0");
    assert_eq!(dr.changed_files, vec!["src/lib.rs"]);
    assert_eq!(dr.commit_count, 7);
    assert_eq!(dr.insertions, 42);
    assert_eq!(dr.deletions, 13);
    // in_diff markers preserved
    let orig_diff_count = substrate.files.iter().filter(|f| f.in_diff).count();
    let rest_diff_count = restored.files.iter().filter(|f| f.in_diff).count();
    assert_eq!(orig_diff_count, rest_diff_count);
}

// ---------------------------------------------------------------------------
// Sensor report with findings + artifacts survives roundtrip
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_with_findings_survives_roundtrip() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let sensor = LangCoverageSensor;
    let mut report = sensor.run(&LangCoverageSettings, &substrate).unwrap();
    report.add_capability("audit", CapabilityStatus::available());
    report = report
        .with_data(serde_json::json!({ "scanned": true, "depth": 2 }))
        .with_artifacts(vec![
            Artifact::receipt("out/report.json"),
            Artifact::receipt("out/summary.md"),
        ]);

    let json = serde_json::to_string_pretty(&report).unwrap();
    let restored: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.verdict, report.verdict);
    assert_eq!(restored.summary, report.summary);
    assert_eq!(restored.findings.len(), report.findings.len());
    for (a, b) in restored.findings.iter().zip(report.findings.iter()) {
        assert_eq!(a.check_id, b.check_id);
        assert_eq!(a.code, b.code);
        assert_eq!(a.severity, b.severity);
    }
    assert_eq!(restored.artifacts.as_ref().unwrap().len(), 2);
    assert_eq!(restored.data.as_ref().unwrap()["scanned"], true);
    assert_eq!(restored.data.as_ref().unwrap()["depth"], 2);
    let caps = restored.capabilities.as_ref().unwrap();
    assert!(caps.contains_key("audit"));
}

// ---------------------------------------------------------------------------
// End-to-end: deterministic sensor JSON output
// ---------------------------------------------------------------------------

#[test]
fn end_to_end_sensor_json_is_deterministic() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let substrate = build_substrate(
        &format!("{}/src", manifest_dir),
        &ScanOptions::default(),
        &[],
        2,
        None,
    )
    .unwrap();

    let sensor = IntegrationSensor;
    let settings = IntegrationSettings { min_languages: 1 };
    let r1 = sensor.run(&settings, &substrate).unwrap();
    let r2 = sensor.run(&settings, &substrate).unwrap();

    let j1 = serde_json::to_string_pretty(&r1).unwrap();
    let j2 = serde_json::to_string_pretty(&r2).unwrap();
    assert_eq!(j1, j2, "sensor JSON output should be deterministic");
}

// ---------------------------------------------------------------------------
// Substrate builder: module_depth parameter
// ---------------------------------------------------------------------------

#[test]
fn build_substrate_different_module_depths() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let s_depth1 = build_substrate(manifest_dir, &ScanOptions::default(), &[], 1, None).unwrap();
    let s_depth3 = build_substrate(manifest_dir, &ScanOptions::default(), &[], 3, None).unwrap();

    // Same files regardless of module depth
    assert_eq!(s_depth1.files.len(), s_depth3.files.len());
    assert_eq!(s_depth1.total_code_lines, s_depth3.total_code_lines);
}

// ---------------------------------------------------------------------------
// Property: sensor report JSON roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_sensor_report_json_roundtrip(sub in arb_substrate()) {
        let sensor = LocThresholdSensor;
        let settings = LocThresholdSettings { warn_threshold: 500, fail_threshold: 1000 };
        let report = sensor.run(&settings, &sub).unwrap();

        let json = serde_json::to_string(&report).unwrap();
        let restored: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(restored.verdict, report.verdict);
        prop_assert_eq!(restored.summary, report.summary);
        prop_assert_eq!(restored.tool.name, report.tool.name);
        prop_assert_eq!(restored.schema, report.schema);
        prop_assert_eq!(restored.findings.len(), report.findings.len());
    }

    /// Lang summary totals match file-level sums for every language.
    #[test]
    fn prop_lang_summary_matches_files(sub in arb_substrate()) {
        for (lang, summary) in &sub.lang_summary {
            let lang_files: Vec<_> = sub.files.iter().filter(|f| &f.lang == lang).collect();
            prop_assert_eq!(summary.files, lang_files.len());
            let code_sum: usize = lang_files.iter().map(|f| f.code).sum();
            prop_assert_eq!(summary.code, code_sum);
        }
    }

    /// Sensor JSON output is deterministic for a given substrate.
    #[test]
    fn prop_sensor_json_is_deterministic(sub in arb_substrate()) {
        let sensor = LocThresholdSensor;
        let settings = LocThresholdSettings { warn_threshold: 500, fail_threshold: 1000 };
        let j1 = serde_json::to_string(&sensor.run(&settings, &sub).unwrap()).unwrap();
        let j2 = serde_json::to_string(&sensor.run(&settings, &sub).unwrap()).unwrap();
        prop_assert_eq!(j1, j2);
    }
}
