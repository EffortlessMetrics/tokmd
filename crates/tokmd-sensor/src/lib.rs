//! # tokmd-sensor
//!
//! **Tier 1 (Sensor Contract)**
//!
//! Defines the `EffortlessSensor` trait and provides the substrate builder
//! that runs the tokei scan + git diff once.
//!
//! ## What belongs here
//! * `EffortlessSensor` trait
//! * `build_substrate()` function
//!
//! ## What does NOT belong here
//! * Sensor implementations (those go in their respective crates)
//! * CLI parsing

pub mod substrate_builder;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokmd_envelope::SensorReport;
use tokmd_substrate::RepoSubstrate;

/// Trait for effortless code quality sensors.
///
/// A sensor receives a pre-built `RepoSubstrate` (shared context from
/// a single tokei scan + git diff) and produces a `SensorReport`.
///
/// # Design
///
/// - **Settings**: Each sensor defines its own settings type.
/// - **Substrate**: Shared context eliminates redundant I/O across sensors.
/// - **Report**: Standardized envelope for cross-fleet aggregation.
pub trait EffortlessSensor {
    /// Settings type for this sensor (must be JSON-serializable).
    type Settings: Serialize + DeserializeOwned;

    /// Sensor name (e.g., "tokmd", "coverage-bot").
    fn name(&self) -> &str;

    /// Sensor version (e.g., "1.5.0").
    fn version(&self) -> &str;

    /// Run the sensor with the given settings and substrate.
    fn run(&self, settings: &Self::Settings, substrate: &RepoSubstrate) -> Result<SensorReport>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tokmd_envelope::{SensorReport, ToolMeta, Verdict};
    use tokmd_substrate::{LangSummary, RepoSubstrate, SubstrateFile};

    /// A trivial test sensor for verifying the trait.
    struct DummySensor;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct DummySettings {
        threshold: usize,
    }

    impl EffortlessSensor for DummySensor {
        type Settings = DummySettings;

        fn name(&self) -> &str {
            "dummy"
        }

        fn version(&self) -> &str {
            "0.1.0"
        }

        fn run(
            &self,
            settings: &DummySettings,
            substrate: &RepoSubstrate,
        ) -> Result<SensorReport> {
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

    fn sample_substrate() -> RepoSubstrate {
        RepoSubstrate {
            repo_root: ".".to_string(),
            files: vec![SubstrateFile {
                path: "src/lib.rs".to_string(),
                lang: "Rust".to_string(),
                code: 100,
                lines: 120,
                bytes: 3000,
                tokens: 750,
                module: "src".to_string(),
                in_diff: false,
            }],
            lang_summary: BTreeMap::from([(
                "Rust".to_string(),
                LangSummary {
                    files: 1,
                    code: 100,
                    lines: 120,
                    bytes: 3000,
                    tokens: 750,
                },
            )]),
            diff_range: None,
            total_tokens: 750,
            total_bytes: 3000,
            total_code_lines: 100,
        }
    }

    #[test]
    fn dummy_sensor_pass() {
        let sensor = DummySensor;
        let settings = DummySettings { threshold: 200 };
        let substrate = sample_substrate();
        let report = sensor.run(&settings, &substrate).unwrap();
        assert_eq!(report.verdict, Verdict::Pass);
    }

    #[test]
    fn dummy_sensor_warn() {
        let sensor = DummySensor;
        let settings = DummySettings { threshold: 50 };
        let substrate = sample_substrate();
        let report = sensor.run(&settings, &substrate).unwrap();
        assert_eq!(report.verdict, Verdict::Warn);
    }

    #[test]
    fn sensor_name_and_version() {
        let sensor = DummySensor;
        assert_eq!(sensor.name(), "dummy");
        assert_eq!(sensor.version(), "0.1.0");
    }
}
