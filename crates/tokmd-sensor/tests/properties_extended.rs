//! Extended property tests for sensor: diff-aware sensors, capability invariants,
//! and report chaining.

use std::collections::BTreeMap;

use anyhow::Result;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingSeverity, SENSOR_REPORT_SCHEMA,
    SensorReport, ToolMeta, Verdict,
};
use tokmd_sensor::EffortlessSensor;
use tokmd_sensor::substrate::{DiffRange, LangSummary, RepoSubstrate, SubstrateFile};

// ---------------------------------------------------------------------------
// Test sensors
// ---------------------------------------------------------------------------

struct DiffCountSensor;

#[derive(Serialize, Deserialize)]
struct DiffCountSettings;

impl EffortlessSensor for DiffCountSensor {
    type Settings = DiffCountSettings;

    fn name(&self) -> &str {
        "diff-count"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn run(&self, _: &Self::Settings, sub: &RepoSubstrate) -> Result<SensorReport> {
        let diff_count = sub.diff_files().count();
        let verdict = if diff_count > 0 {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-07-01T00:00:00Z".to_string(),
            verdict,
            format!("{} files in diff", diff_count),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arbitrary generators
// ---------------------------------------------------------------------------

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

fn arb_substrate_with_diff() -> impl Strategy<Value = RepoSubstrate> {
    prop::collection::vec(arb_substrate_file(), 0..15).prop_map(|files| {
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
        let changed_files: Vec<String> = files
            .iter()
            .filter(|f| f.in_diff)
            .map(|f| f.path.clone())
            .collect();
        let diff_range = if changed_files.is_empty() {
            None
        } else {
            Some(DiffRange {
                base: "main".to_string(),
                head: "feature".to_string(),
                changed_files,
                commit_count: 1,
                insertions: 10,
                deletions: 5,
            })
        };
        RepoSubstrate {
            repo_root: ".".to_string(),
            files,
            lang_summary,
            diff_range,
            total_tokens,
            total_bytes,
            total_code_lines,
        }
    })
}

proptest! {
    // ── Diff-aware sensor: diff_files count matches in_diff count ─────

    #[test]
    fn diff_files_count_matches_in_diff_flag(sub in arb_substrate_with_diff()) {
        let diff_count = sub.diff_files().count();
        let in_diff_count = sub.files.iter().filter(|f| f.in_diff).count();
        prop_assert_eq!(diff_count, in_diff_count);
    }

    // ── Diff-aware sensor never panics ───────────────────────────────

    #[test]
    fn diff_count_sensor_never_panics(sub in arb_substrate_with_diff()) {
        let sensor = DiffCountSensor;
        let _report = sensor.run(&DiffCountSettings, &sub).unwrap();
    }

    // ── files_for_lang partition covers all files ────────────────────

    #[test]
    fn files_for_lang_covers_all_files(sub in arb_substrate_with_diff()) {
        let mut total_via_lang = 0usize;
        for lang in sub.lang_summary.keys() {
            total_via_lang += sub.files_for_lang(lang).count();
        }
        prop_assert_eq!(total_via_lang, sub.files.len());
    }

    // ── Report with_data preserves verdict ──────────────────────────

    #[test]
    fn with_data_preserves_verdict(
        verdict in prop::sample::select(vec![
            Verdict::Pass, Verdict::Fail, Verdict::Warn,
            Verdict::Skip, Verdict::Pending,
        ]),
    ) {
        let report = SensorReport::new(
            ToolMeta::new("prop-test", "1.0.0", "check"),
            "2024-01-01T00:00:00Z".to_string(),
            verdict,
            "test".to_string(),
        ).with_data(serde_json::json!({"key": "value"}));

        prop_assert_eq!(report.verdict, verdict);
        prop_assert!(report.data.is_some());
    }

    // ── Report with_artifacts preserves other fields ─────────────────

    #[test]
    fn with_artifacts_preserves_schema(
        name in "[a-z][a-z0-9-]{1,15}",
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
    ) {
        let report = SensorReport::new(
            ToolMeta::new(&name, &version, "check"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "test".to_string(),
        ).with_artifacts(vec![Artifact::receipt("out.json")]);

        prop_assert_eq!(&report.schema, SENSOR_REPORT_SCHEMA);
        prop_assert_eq!(&report.tool.name, &name);
        prop_assert!(report.artifacts.is_some());
    }

    // ── Capability status roundtrip ──────────────────────────────────

    #[test]
    fn capability_status_roundtrips_through_json(
        reason in "[a-z ]{5,30}",
        state in prop::sample::select(vec![
            CapabilityState::Available,
            CapabilityState::Unavailable,
            CapabilityState::Skipped,
        ]),
    ) {
        let status = CapabilityStatus {
            status: state,
            reason: Some(reason.clone()),
        };
        let json = serde_json::to_string(&status).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.status, state);
        prop_assert_eq!(back.reason.as_deref(), Some(reason.as_str()));
    }

    // ── Finding fields are preserved ─────────────────────────────────

    #[test]
    fn finding_preserves_all_fields(
        check_id in "[a-z-]{3,15}",
        code in "[a-z-]{3,15}",
        title in "[a-zA-Z ]{5,30}",
        message in "[a-zA-Z0-9 ]{10,50}",
    ) {
        let finding = Finding::new(
            &check_id,
            &code,
            FindingSeverity::Warn,
            &title,
            message.clone(),
        );
        prop_assert_eq!(&finding.check_id, &check_id);
        prop_assert_eq!(&finding.code, &code);
        prop_assert_eq!(&finding.title, &title);
        prop_assert_eq!(&finding.message, &message);
    }
}
