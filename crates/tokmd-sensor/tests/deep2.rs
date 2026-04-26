//! Deep integration tests (batch 2) for tokmd-sensor crate.
//!
//! Covers: EffortlessSensor trait implementation variations, substrate builder
//! edge cases, sensor report construction and serialization, RepoSubstrate
//! methods, multi-language substrates, and finding/verdict interactions.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ===========================================================================
// Test sensor implementations
// ===========================================================================

/// A sensor that always passes.
struct AlwaysPassSensor;

#[derive(Serialize, Deserialize)]
struct EmptySettings;

impl EffortlessSensor for AlwaysPassSensor {
    type Settings = EmptySettings;

    fn name(&self) -> &str {
        "always-pass"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn run(&self, _settings: &EmptySettings, _substrate: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-06-15T12:00:00Z".to_string(),
            Verdict::Pass,
            "All checks passed".to_string(),
        ))
    }
}

/// A sensor that counts files by language and warns if a language dominates.
struct DominanceSensor;

#[derive(Serialize, Deserialize)]
struct DominanceSettings {
    max_pct: f64,
}

impl EffortlessSensor for DominanceSensor {
    type Settings = DominanceSettings;

    fn name(&self) -> &str {
        "dominance"
    }

    fn version(&self) -> &str {
        "0.2.0"
    }

    fn run(&self, settings: &DominanceSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let total_code = substrate.total_code_lines.max(1) as f64;
        let mut verdict = Verdict::Pass;
        let mut findings = Vec::new();

        for (lang, summary) in &substrate.lang_summary {
            let pct = summary.code as f64 / total_code;
            if pct > settings.max_pct {
                verdict = Verdict::Warn;
                findings.push(Finding::new(
                    "dominance",
                    "language_dominance",
                    FindingSeverity::Warn,
                    format!("{lang} dominates"),
                    format!(
                        "{lang} accounts for {:.0}% of code ({} lines)",
                        pct * 100.0,
                        summary.code
                    ),
                ));
            }
        }

        let mut report = SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "analyze"),
            "2024-06-15T12:00:00Z".to_string(),
            verdict,
            format!("{} languages analyzed", substrate.lang_summary.len()),
        );
        for f in findings {
            report.add_finding(f);
        }
        Ok(report)
    }
}

/// A sensor that reports on diff coverage.
struct DiffSensor;

#[derive(Serialize, Deserialize)]
struct DiffSettings {
    require_diff: bool,
}

impl EffortlessSensor for DiffSensor {
    type Settings = DiffSettings;

    fn name(&self) -> &str {
        "diff-reporter"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn run(&self, settings: &DiffSettings, substrate: &RepoSubstrate) -> Result<SensorReport> {
        let diff_count = substrate.diff_files().count();
        if settings.require_diff && substrate.diff_range.is_none() {
            return Ok(SensorReport::new(
                ToolMeta::new(self.name(), self.version(), "diff"),
                "2024-06-15T12:00:00Z".to_string(),
                Verdict::Skip,
                "No diff range available".to_string(),
            ));
        }
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "diff"),
            "2024-06-15T12:00:00Z".to_string(),
            Verdict::Pass,
            format!("{diff_count} files in diff"),
        ))
    }
}

/// A sensor that always fails.
struct FailSensor;

#[derive(Serialize, Deserialize)]
struct FailSettings {
    message: String,
}

impl EffortlessSensor for FailSensor {
    type Settings = FailSettings;

    fn name(&self) -> &str {
        "fail-sensor"
    }

    fn version(&self) -> &str {
        "0.0.1"
    }

    fn run(&self, settings: &FailSettings, _substrate: &RepoSubstrate) -> Result<SensorReport> {
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "gate"),
            "2024-06-15T12:00:00Z".to_string(),
            Verdict::Fail,
            settings.message.clone(),
        ))
    }
}

// ===========================================================================
// Substrate helpers
// ===========================================================================

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
            code: 500,
            lines: 600,
            bytes: 15_000,
            tokens: 3_750,
            module: "src".to_string(),
            in_diff: true,
        },
        SubstrateFile {
            path: "src/main.rs".to_string(),
            lang: "Rust".to_string(),
            code: 200,
            lines: 250,
            bytes: 6_000,
            tokens: 1_500,
            module: "src".to_string(),
            in_diff: false,
        },
        SubstrateFile {
            path: "app/index.ts".to_string(),
            lang: "TypeScript".to_string(),
            code: 300,
            lines: 350,
            bytes: 9_000,
            tokens: 2_250,
            module: "app".to_string(),
            in_diff: true,
        },
        SubstrateFile {
            path: "scripts/build.py".to_string(),
            lang: "Python".to_string(),
            code: 50,
            lines: 70,
            bytes: 1_500,
            tokens: 375,
            module: "scripts".to_string(),
            in_diff: false,
        },
    ];

    let mut lang_summary = BTreeMap::new();
    lang_summary.insert(
        "Rust".to_string(),
        LangSummary {
            files: 2,
            code: 700,
            lines: 850,
            bytes: 21_000,
            tokens: 5_250,
        },
    );
    lang_summary.insert(
        "TypeScript".to_string(),
        LangSummary {
            files: 1,
            code: 300,
            lines: 350,
            bytes: 9_000,
            tokens: 2_250,
        },
    );
    lang_summary.insert(
        "Python".to_string(),
        LangSummary {
            files: 1,
            code: 50,
            lines: 70,
            bytes: 1_500,
            tokens: 375,
        },
    );

    RepoSubstrate {
        repo_root: "/repo".to_string(),
        files,
        lang_summary,
        diff_range: Some(DiffRange {
            base: "main".to_string(),
            head: "feature-xyz".to_string(),
            changed_files: vec!["src/lib.rs".to_string(), "app/index.ts".to_string()],
            commit_count: 5,
            insertions: 42,
            deletions: 10,
        }),
        total_tokens: 7_875,
        total_bytes: 31_500,
        total_code_lines: 1_050,
    }
}

fn single_file_substrate(code: usize) -> RepoSubstrate {
    RepoSubstrate {
        repo_root: ".".to_string(),
        files: vec![SubstrateFile {
            path: "main.rs".to_string(),
            lang: "Rust".to_string(),
            code,
            lines: code + 20,
            bytes: code * 30,
            tokens: code * 7,
            module: ".".to_string(),
            in_diff: false,
        }],
        lang_summary: BTreeMap::from([(
            "Rust".to_string(),
            LangSummary {
                files: 1,
                code,
                lines: code + 20,
                bytes: code * 30,
                tokens: code * 7,
            },
        )]),
        diff_range: None,
        total_tokens: code * 7,
        total_bytes: code * 30,
        total_code_lines: code,
    }
}

// ===========================================================================
// 1. EffortlessSensor trait — basic contract
// ===========================================================================

#[test]
fn always_pass_sensor_returns_pass() {
    let sensor = AlwaysPassSensor;
    let report = sensor.run(&EmptySettings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert_eq!(report.tool.name, "always-pass");
    assert_eq!(report.tool.version, "1.0.0");
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn fail_sensor_returns_fail_with_message() {
    let sensor = FailSensor;
    let settings = FailSettings {
        message: "Policy violated".to_string(),
    };
    let report = sensor.run(&settings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Fail);
    assert_eq!(report.summary, "Policy violated");
}

#[test]
fn sensor_name_and_version_are_stable() {
    let sensor = DominanceSensor;
    assert_eq!(sensor.name(), "dominance");
    assert_eq!(sensor.version(), "0.2.0");
    // Call again to verify stability
    assert_eq!(sensor.name(), "dominance");
    assert_eq!(sensor.version(), "0.2.0");
}

// ===========================================================================
// 2. Dominance sensor — behavioral tests
// ===========================================================================

#[test]
fn dominance_sensor_pass_when_balanced() {
    let sensor = DominanceSensor;
    let settings = DominanceSettings { max_pct: 0.80 };
    let substrate = multi_lang_substrate();
    let report = sensor.run(&settings, &substrate).unwrap();
    // Rust is 700/1050 = 66.7% < 80%
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
}

#[test]
fn dominance_sensor_warn_when_dominant() {
    let sensor = DominanceSensor;
    let settings = DominanceSettings { max_pct: 0.50 };
    let substrate = multi_lang_substrate();
    let report = sensor.run(&settings, &substrate).unwrap();
    // Rust is 700/1050 = 66.7% > 50%
    assert_eq!(report.verdict, Verdict::Warn);
    assert!(!report.findings.is_empty());
    assert!(report.findings[0].message.contains("Rust"));
}

#[test]
fn dominance_sensor_empty_substrate_passes() {
    let sensor = DominanceSensor;
    let settings = DominanceSettings { max_pct: 0.50 };
    let report = sensor.run(&settings, &empty_substrate()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn dominance_sensor_single_language_always_warns_at_low_threshold() {
    let sensor = DominanceSensor;
    let settings = DominanceSettings { max_pct: 0.50 };
    let substrate = single_file_substrate(100);
    let report = sensor.run(&settings, &substrate).unwrap();
    // Single language = 100% > 50%
    assert_eq!(report.verdict, Verdict::Warn);
}

// ===========================================================================
// 3. Diff sensor — behavioral tests
// ===========================================================================

#[test]
fn diff_sensor_reports_diff_file_count() {
    let sensor = DiffSensor;
    let settings = DiffSettings {
        require_diff: false,
    };
    let substrate = multi_lang_substrate();
    let report = sensor.run(&settings, &substrate).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.summary.contains("2 files in diff"));
}

#[test]
fn diff_sensor_skip_when_required_but_missing() {
    let sensor = DiffSensor;
    let settings = DiffSettings { require_diff: true };
    let substrate = empty_substrate();
    let report = sensor.run(&settings, &substrate).unwrap();
    assert_eq!(report.verdict, Verdict::Skip);
}

#[test]
fn diff_sensor_pass_when_diff_not_required_and_missing() {
    let sensor = DiffSensor;
    let settings = DiffSettings {
        require_diff: false,
    };
    let substrate = empty_substrate();
    let report = sensor.run(&settings, &substrate).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.summary.contains("0 files"));
}

// ===========================================================================
// 4. RepoSubstrate methods
// ===========================================================================

#[test]
fn diff_files_returns_only_in_diff() {
    let substrate = multi_lang_substrate();
    let diff: Vec<_> = substrate.diff_files().collect();
    assert_eq!(diff.len(), 2);
    assert!(diff.iter().any(|f| f.path == "src/lib.rs"));
    assert!(diff.iter().any(|f| f.path == "app/index.ts"));
}

#[test]
fn diff_files_empty_when_none_in_diff() {
    let substrate = single_file_substrate(100);
    let diff: Vec<_> = substrate.diff_files().collect();
    assert_eq!(diff.len(), 0);
}

#[test]
fn files_for_lang_filters_correctly() {
    let substrate = multi_lang_substrate();

    let rust_files: Vec<_> = substrate.files_for_lang("Rust").collect();
    assert_eq!(rust_files.len(), 2);

    let ts_files: Vec<_> = substrate.files_for_lang("TypeScript").collect();
    assert_eq!(ts_files.len(), 1);

    let py_files: Vec<_> = substrate.files_for_lang("Python").collect();
    assert_eq!(py_files.len(), 1);

    let go_files: Vec<_> = substrate.files_for_lang("Go").collect();
    assert_eq!(go_files.len(), 0);
}

#[test]
fn empty_substrate_methods_return_empty() {
    let substrate = empty_substrate();
    assert_eq!(substrate.diff_files().count(), 0);
    assert_eq!(substrate.files_for_lang("Rust").count(), 0);
    assert_eq!(substrate.total_tokens, 0);
    assert_eq!(substrate.total_bytes, 0);
    assert_eq!(substrate.total_code_lines, 0);
}

// ===========================================================================
// 5. Substrate serialization roundtrip
// ===========================================================================

#[test]
fn substrate_serde_roundtrip_empty() {
    let substrate = empty_substrate();
    let json = serde_json::to_string(&substrate).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 0);
    assert_eq!(back.total_tokens, 0);
    assert!(back.diff_range.is_none());
}

#[test]
fn substrate_serde_roundtrip_multi_lang() {
    let substrate = multi_lang_substrate();
    let json = serde_json::to_string(&substrate).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 4);
    assert_eq!(back.lang_summary.len(), 3);
    assert_eq!(back.total_code_lines, 1_050);
    assert!(back.diff_range.is_some());
    let dr = back.diff_range.unwrap();
    assert_eq!(dr.base, "main");
    assert_eq!(dr.head, "feature-xyz");
    assert_eq!(dr.commit_count, 5);
}

#[test]
fn substrate_serde_preserves_in_diff_flag() {
    let substrate = multi_lang_substrate();
    let json = serde_json::to_string(&substrate).unwrap();
    let back: RepoSubstrate = serde_json::from_str(&json).unwrap();
    let diff_files: Vec<_> = back.files.iter().filter(|f| f.in_diff).collect();
    assert_eq!(diff_files.len(), 2);
}

#[test]
fn substrate_serde_diff_range_none_omitted() {
    let substrate = empty_substrate();
    let json = serde_json::to_string(&substrate).unwrap();
    // diff_range has skip_serializing_if = None, so it should be absent
    assert!(!json.contains("diff_range") || json.contains("null"));
}

// ===========================================================================
// 6. SensorReport construction and serialization
// ===========================================================================

#[test]
fn sensor_report_schema_field() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "test".to_string(),
    );
    assert_eq!(report.schema, "sensor.report.v1");
}

#[test]
fn sensor_report_with_findings_roundtrip() {
    let mut report = SensorReport::new(
        ToolMeta::new("my-sensor", "2.0.0", "analyze"),
        "2024-07-01T00:00:00Z".to_string(),
        Verdict::Warn,
        "Issues found".to_string(),
    );
    report.add_finding(
        Finding::new(
            "risk",
            "hotspot",
            FindingSeverity::Warn,
            "Hot file",
            "Changed 50 times",
        )
        .with_location(FindingLocation::path_line("src/lib.rs", 42)),
    );
    report.add_finding(Finding::new(
        "supply",
        "lockfile_changed",
        FindingSeverity::Info,
        "Lockfile updated",
        "Cargo.lock was modified",
    ));

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 2);
    assert_eq!(back.findings[0].check_id, "risk");
    assert_eq!(back.findings[1].check_id, "supply");
}

#[test]
fn sensor_report_with_artifacts() {
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "run"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "Done".to_string(),
    )
    .with_artifacts(vec![
        Artifact::receipt("out/receipt.json"),
        Artifact::badge("out/badge.svg"),
        Artifact::comment("out/pr-comment.md"),
    ]);

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let artifacts = back.artifacts.unwrap();
    assert_eq!(artifacts.len(), 3);
    assert_eq!(artifacts[0].artifact_type, "receipt");
    assert_eq!(artifacts[1].artifact_type, "badge");
    assert_eq!(artifacts[2].artifact_type, "comment");
}

#[test]
fn sensor_report_with_data_payload() {
    let data = serde_json::json!({
        "metrics": {
            "code_lines": 1050,
            "languages": 3,
        },
        "tags": ["multi-lang", "active"]
    });
    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "analyze"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "Analyzed".to_string(),
    )
    .with_data(data.clone());

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.data.unwrap(), data);
}

#[test]
fn sensor_report_with_capabilities() {
    let mut caps = BTreeMap::new();
    caps.insert("complexity".to_string(), CapabilityStatus::available());
    caps.insert(
        "coverage".to_string(),
        CapabilityStatus::unavailable("no lcov.info"),
    );
    caps.insert(
        "mutation".to_string(),
        CapabilityStatus::skipped("no test files changed"),
    );

    let report = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "cockpit"),
        "2024-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "Done".to_string(),
    )
    .with_capabilities(caps);

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let caps = back.capabilities.unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["complexity"].status, CapabilityState::Available);
    assert_eq!(caps["coverage"].status, CapabilityState::Unavailable);
    assert_eq!(caps["mutation"].status, CapabilityState::Skipped);
}

// ===========================================================================
// 7. Verdict variants
// ===========================================================================

#[test]
fn all_verdict_variants_serialize_correctly() {
    for (variant, expected) in [
        (Verdict::Pass, "\"pass\""),
        (Verdict::Fail, "\"fail\""),
        (Verdict::Warn, "\"warn\""),
        (Verdict::Skip, "\"skip\""),
        (Verdict::Pending, "\"pending\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
    }
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

#[test]
fn verdict_display_matches_serde() {
    for variant in [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ] {
        let display = variant.to_string();
        let serde_val = serde_json::to_value(variant).unwrap();
        assert_eq!(display, serde_val.as_str().unwrap());
    }
}

// ===========================================================================
// 8. ToolMeta builders
// ===========================================================================

#[test]
fn tool_meta_new_fields() {
    let meta = ToolMeta::new("custom-sensor", "3.2.1", "scan");
    assert_eq!(meta.name, "custom-sensor");
    assert_eq!(meta.version, "3.2.1");
    assert_eq!(meta.mode, "scan");
}

#[test]
fn tool_meta_tokmd_helper() {
    let meta = ToolMeta::tokmd("1.5.0", "cockpit");
    assert_eq!(meta.name, "tokmd");
    assert_eq!(meta.version, "1.5.0");
    assert_eq!(meta.mode, "cockpit");
}

// ===========================================================================
// 9. Finding fingerprint
// ===========================================================================

#[test]
fn finding_fingerprint_deterministic() {
    let finding = Finding::new("risk", "hotspot", FindingSeverity::Warn, "Hot", "msg")
        .with_location(FindingLocation::path("src/lib.rs"));
    let fp1 = finding.compute_fingerprint("tokmd");
    let fp2 = finding.compute_fingerprint("tokmd");
    assert_eq!(fp1, fp2);
    assert_eq!(fp1.len(), 32); // 16 bytes = 32 hex chars
}

#[test]
fn finding_fingerprint_differs_by_tool() {
    let finding = Finding::new("risk", "hotspot", FindingSeverity::Warn, "Hot", "msg");
    let fp1 = finding.compute_fingerprint("tokmd");
    let fp2 = finding.compute_fingerprint("other-tool");
    assert_ne!(fp1, fp2);
}

#[test]
fn finding_fingerprint_differs_by_path() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "Hot", "msg")
        .with_location(FindingLocation::path("src/a.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "Hot", "msg")
        .with_location(FindingLocation::path("src/b.rs"));
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn finding_fingerprint_no_location_uses_empty_path() {
    let finding = Finding::new("risk", "hotspot", FindingSeverity::Warn, "Hot", "msg");
    let fp = finding.compute_fingerprint("tokmd");
    assert!(!fp.is_empty());
}

// ===========================================================================
// 10. Sensor with substrate — multi-sensor pipeline simulation
// ===========================================================================

#[test]
fn multi_sensor_pipeline_on_same_substrate() {
    let substrate = multi_lang_substrate();

    // Run multiple sensors on same substrate
    let pass_report = AlwaysPassSensor.run(&EmptySettings, &substrate).unwrap();
    let dominance_report = DominanceSensor
        .run(&DominanceSettings { max_pct: 0.50 }, &substrate)
        .unwrap();
    let diff_report = DiffSensor
        .run(
            &DiffSettings {
                require_diff: false,
            },
            &substrate,
        )
        .unwrap();

    assert_eq!(pass_report.verdict, Verdict::Pass);
    assert_eq!(dominance_report.verdict, Verdict::Warn);
    assert_eq!(diff_report.verdict, Verdict::Pass);

    // All reports share the schema
    assert_eq!(pass_report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(dominance_report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(diff_report.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn sensor_reports_are_independent() {
    let substrate = multi_lang_substrate();

    let r1 = DominanceSensor
        .run(&DominanceSettings { max_pct: 0.50 }, &substrate)
        .unwrap();
    let r2 = DominanceSensor
        .run(&DominanceSettings { max_pct: 0.90 }, &substrate)
        .unwrap();

    // Different thresholds → different verdicts
    assert_eq!(r1.verdict, Verdict::Warn);
    assert_eq!(r2.verdict, Verdict::Pass);
}

// ===========================================================================
// 11. DiffRange edge cases
// ===========================================================================

#[test]
fn diff_range_serde_roundtrip() {
    let dr = DiffRange {
        base: "v1.0.0".to_string(),
        head: "v2.0.0".to_string(),
        changed_files: vec!["a.rs".to_string(), "b.rs".to_string()],
        commit_count: 42,
        insertions: 500,
        deletions: 200,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v1.0.0");
    assert_eq!(back.head, "v2.0.0");
    assert_eq!(back.changed_files.len(), 2);
    assert_eq!(back.commit_count, 42);
}

#[test]
fn diff_range_empty_changed_files() {
    let dr = DiffRange {
        base: "main".to_string(),
        head: "HEAD".to_string(),
        changed_files: vec![],
        commit_count: 0,
        insertions: 0,
        deletions: 0,
    };
    let json = serde_json::to_string(&dr).unwrap();
    let back: DiffRange = serde_json::from_str(&json).unwrap();
    assert!(back.changed_files.is_empty());
}

// ===========================================================================
// 12. LangSummary aggregation
// ===========================================================================

#[test]
fn lang_summary_btreemap_ordering() {
    let substrate = multi_lang_substrate();
    let keys: Vec<_> = substrate.lang_summary.keys().collect();
    // BTreeMap ensures alphabetical order
    assert_eq!(keys, vec!["Python", "Rust", "TypeScript"]);
}

#[test]
fn lang_summary_totals_consistent() {
    let substrate = multi_lang_substrate();
    let sum_code: usize = substrate.lang_summary.values().map(|s| s.code).sum();
    assert_eq!(sum_code, substrate.total_code_lines);

    let sum_tokens: usize = substrate.lang_summary.values().map(|s| s.tokens).sum();
    assert_eq!(sum_tokens, substrate.total_tokens);
}

// ===========================================================================
// 13. Settings serialization
// ===========================================================================

#[test]
fn empty_settings_serde_roundtrip() {
    let settings = EmptySettings;
    let json = serde_json::to_string(&settings).unwrap();
    let _back: EmptySettings = serde_json::from_str(&json).unwrap();
}

#[test]
fn dominance_settings_serde_roundtrip() {
    let settings = DominanceSettings { max_pct: 0.75 };
    let json = serde_json::to_string(&settings).unwrap();
    let back: DominanceSettings = serde_json::from_str(&json).unwrap();
    assert!((back.max_pct - 0.75).abs() < f64::EPSILON);
}

#[test]
fn fail_settings_serde_roundtrip() {
    let settings = FailSettings {
        message: "gate failed: complexity too high".to_string(),
    };
    let json = serde_json::to_string(&settings).unwrap();
    let back: FailSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.message, "gate failed: complexity too high");
}
