//! Property-based tests for sensor trait invariants.
//!
//! Verifies that sensors produce valid reports regardless of substrate content.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_envelope::{SensorReport, ToolMeta, Verdict, SENSOR_REPORT_SCHEMA};
use tokmd_sensor::EffortlessSensor;
use tokmd_substrate::{LangSummary, RepoSubstrate, SubstrateFile};

// ── Strategies ───────────────────────────────────────────────────

fn arb_substrate_file() -> impl Strategy<Value = SubstrateFile> {
    (
        "[a-z]+(/[a-z]+){0,3}\\.[a-z]{1,4}",
        prop::sample::select(vec![
            "Rust", "Python", "TypeScript", "Go", "Java", "C", "Shell",
        ]),
        0usize..5_000,
        0usize..10_000,
        0usize..500_000,
        0usize..125_000,
        "[a-z]+(/[a-z]+){0,2}",
        any::<bool>(),
    )
        .prop_map(
            |(path, lang, code, lines, bytes, tokens, module, in_diff)| SubstrateFile {
                path,
                lang: lang.to_string(),
                code,
                lines,
                bytes,
                tokens,
                module,
                in_diff,
            },
        )
}

fn arb_substrate() -> impl Strategy<Value = RepoSubstrate> {
    proptest::collection::vec(arb_substrate_file(), 0..20).prop_map(|files| {
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
    })
}

/// A generic sensor for property testing.
struct PropSensor;

#[derive(serde::Serialize, serde::Deserialize)]
struct PropSettings;

impl EffortlessSensor for PropSensor {
    type Settings = PropSettings;
    fn name(&self) -> &str {
        "prop-sensor"
    }
    fn version(&self) -> &str {
        "0.0.1"
    }
    fn run(&self, _settings: &PropSettings, sub: &RepoSubstrate) -> anyhow::Result<SensorReport> {
        let verdict = if sub.files.is_empty() {
            Verdict::Skip
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-01-01T00:00:00Z".to_string(),
            verdict,
            format!("{} files scanned", sub.files.len()),
        ))
    }
}

// ── Properties ───────────────────────────────────────────────────

proptest! {
    #[test]
    fn report_always_has_valid_schema(sub in arb_substrate()) {
        let report = PropSensor.run(&PropSettings, &sub).unwrap();
        prop_assert_eq!(report.schema.as_str(), SENSOR_REPORT_SCHEMA);
    }

    #[test]
    fn report_tool_meta_always_matches(sub in arb_substrate()) {
        let report = PropSensor.run(&PropSettings, &sub).unwrap();
        prop_assert_eq!(report.tool.name.as_str(), "prop-sensor");
        prop_assert_eq!(report.tool.version.as_str(), "0.0.1");
    }

    #[test]
    fn empty_substrate_always_skips(sub in arb_substrate()) {
        let report = PropSensor.run(&PropSettings, &sub).unwrap();
        if sub.files.is_empty() {
            prop_assert_eq!(report.verdict, Verdict::Skip);
        } else {
            prop_assert_eq!(report.verdict, Verdict::Pass);
        }
    }

    #[test]
    fn report_json_roundtrips(sub in arb_substrate()) {
        let report = PropSensor.run(&PropSettings, &sub).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.verdict, report.verdict);
        prop_assert_eq!(back.schema, report.schema);
        prop_assert_eq!(back.tool.name, report.tool.name);
    }

    #[test]
    fn report_is_deterministic(sub in arb_substrate()) {
        let r1 = PropSensor.run(&PropSettings, &sub).unwrap();
        let r2 = PropSensor.run(&PropSettings, &sub).unwrap();
        let j1 = serde_json::to_string(&r1).unwrap();
        let j2 = serde_json::to_string(&r2).unwrap();
        prop_assert_eq!(j1, j2);
    }

    #[test]
    fn substrate_totals_consistent(sub in arb_substrate()) {
        let computed_code: usize = sub.files.iter().map(|f| f.code).sum();
        let computed_tokens: usize = sub.files.iter().map(|f| f.tokens).sum();
        let computed_bytes: usize = sub.files.iter().map(|f| f.bytes).sum();
        prop_assert_eq!(sub.total_code_lines, computed_code);
        prop_assert_eq!(sub.total_tokens, computed_tokens);
        prop_assert_eq!(sub.total_bytes, computed_bytes);
    }

    #[test]
    fn lang_summary_files_count_matches(sub in arb_substrate()) {
        let total_from_summary: usize = sub.lang_summary.values().map(|l| l.files).sum();
        prop_assert_eq!(total_from_summary, sub.files.len());
    }
}
