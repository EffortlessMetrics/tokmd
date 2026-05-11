//! Sensor command parser types.
//!
//! This module owns the clap/serde contract for `tokmd sensor` while the parent
//! parser module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct SensorArgs {
    /// Base reference to compare from (default: main).
    #[arg(long, default_value = "main")]
    pub base: String,

    /// Head reference to compare to (default: HEAD).
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Output file for the sensor report.
    #[arg(
        long,
        value_name = "PATH",
        default_value = "artifacts/tokmd/report.json"
    )]
    pub output: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = SensorFormat::Json)]
    pub format: SensorFormat,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SensorFormat {
    /// JSON sensor report.
    #[default]
    Json,
    /// Markdown summary.
    Md,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_format_default_is_json() {
        assert_eq!(SensorFormat::default(), SensorFormat::Json);
    }
}
