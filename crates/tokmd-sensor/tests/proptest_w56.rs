use proptest::prelude::*;
use std::collections::BTreeMap;
use tokmd_envelope::{SensorReport, ToolMeta, Verdict};
use tokmd_sensor::EffortlessSensor;
use tokmd_substrate::{LangSummary, RepoSubstrate, SubstrateFile};

/// A test sensor for property testing.
struct PropSensor;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct PropSettings {
    threshold: usize,
    label: String,
}

impl EffortlessSensor for PropSensor {
    type Settings = PropSettings;

    fn name(&self) -> &str {
        "prop-sensor"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn run(
        &self,
        settings: &PropSettings,
        substrate: &RepoSubstrate,
    ) -> anyhow::Result<SensorReport> {
        let verdict = if substrate.total_code_lines > settings.threshold {
            Verdict::Warn
        } else {
            Verdict::Pass
        };
        Ok(SensorReport::new(
            ToolMeta::new(self.name(), self.version(), "check"),
            "2024-01-01T00:00:00Z".to_string(),
            verdict,
            format!(
                "{} code lines (threshold: {})",
                substrate.total_code_lines, settings.threshold
            ),
        ))
    }
}

fn make_substrate(code_lines: usize) -> RepoSubstrate {
    RepoSubstrate {
        repo_root: ".".to_string(),
        files: vec![SubstrateFile {
            path: "src/lib.rs".to_string(),
            lang: "Rust".to_string(),
            code: code_lines,
            lines: code_lines + 20,
            bytes: code_lines * 30,
            tokens: code_lines * 7,
            module: "src".to_string(),
            in_diff: false,
        }],
        lang_summary: BTreeMap::from([(
            "Rust".to_string(),
            LangSummary {
                files: 1,
                code: code_lines,
                lines: code_lines + 20,
                bytes: code_lines * 30,
                tokens: code_lines * 7,
            },
        )]),
        diff_range: None,
        total_tokens: code_lines * 7,
        total_bytes: code_lines * 30,
        total_code_lines: code_lines,
    }
}

proptest! {
    /// Sensor name is stable across calls.
    #[test]
    fn sensor_name_is_stable(_dummy in 0..5u8) {
        let sensor = PropSensor;
        let n1 = sensor.name();
        let n2 = sensor.name();
        prop_assert_eq!(n1, n2);
    }

    /// Sensor version is stable across calls.
    #[test]
    fn sensor_version_is_stable(_dummy in 0..5u8) {
        let sensor = PropSensor;
        let v1 = sensor.version();
        let v2 = sensor.version();
        prop_assert_eq!(v1, v2);
    }

    /// Settings serialization round-trips.
    #[test]
    fn settings_serde_roundtrip(
        threshold in 0usize..10_000,
        label in "[a-zA-Z0-9_]{1,30}",
    ) {
        let settings = PropSettings { threshold, label };
        let json = serde_json::to_string(&settings).unwrap();
        let back: PropSettings = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(settings, back);
    }

    /// Sensor run is deterministic for the same inputs.
    #[test]
    fn sensor_run_is_deterministic(
        code_lines in 0usize..1000,
        threshold in 0usize..2000,
    ) {
        let sensor = PropSensor;
        let settings = PropSettings { threshold, label: "test".to_string() };
        let substrate = make_substrate(code_lines);
        let r1 = sensor.run(&settings, &substrate).unwrap();
        let r2 = sensor.run(&settings, &substrate).unwrap();
        prop_assert_eq!(r1.verdict, r2.verdict);
        prop_assert_eq!(r1.summary, r2.summary);
    }

    /// Verdict is Pass when code_lines <= threshold, Warn otherwise.
    #[test]
    fn verdict_matches_threshold(
        code_lines in 0usize..1000,
        threshold in 0usize..2000,
    ) {
        let sensor = PropSensor;
        let settings = PropSettings { threshold, label: "test".to_string() };
        let substrate = make_substrate(code_lines);
        let report = sensor.run(&settings, &substrate).unwrap();
        if code_lines > threshold {
            prop_assert_eq!(report.verdict, Verdict::Warn);
        } else {
            prop_assert_eq!(report.verdict, Verdict::Pass);
        }
    }

    /// Report summary is non-empty.
    #[test]
    fn report_summary_nonempty(
        code_lines in 0usize..500,
        threshold in 0usize..1000,
    ) {
        let sensor = PropSensor;
        let settings = PropSettings { threshold, label: "test".to_string() };
        let substrate = make_substrate(code_lines);
        let report = sensor.run(&settings, &substrate).unwrap();
        prop_assert!(!report.summary.is_empty());
    }

    /// Report serializes to valid JSON.
    #[test]
    fn report_serializes_to_json(
        code_lines in 0usize..500,
        threshold in 0usize..1000,
    ) {
        let sensor = PropSensor;
        let settings = PropSettings { threshold, label: "test".to_string() };
        let substrate = make_substrate(code_lines);
        let report = sensor.run(&settings, &substrate).unwrap();
        let json = serde_json::to_string(&report);
        prop_assert!(json.is_ok(), "Report should serialize to JSON");
    }

    /// SensorReport serde round-trip preserves verdict and summary.
    #[test]
    fn report_serde_roundtrip(
        code_lines in 0usize..500,
        threshold in 0usize..1000,
    ) {
        let sensor = PropSensor;
        let settings = PropSettings { threshold, label: "test".to_string() };
        let substrate = make_substrate(code_lines);
        let report = sensor.run(&settings, &substrate).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(report.verdict, back.verdict);
        prop_assert_eq!(report.summary, back.summary);
    }
}
