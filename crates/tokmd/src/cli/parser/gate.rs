//! Gate command parser types.
//!
//! This module owns the clap/serde contract for `tokmd gate` while the parent
//! parser module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

use super::AnalysisPreset;

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd gate\n  tokmd gate analysis.json --policy policy.toml\n  tokmd gate --preset health\n  tokmd gate --format json\n  tokmd gate --fail-fast\n  tokmd gate --baseline baseline.json"
)]
pub struct CliGateArgs {
    /// Input analysis receipt or path to scan.
    #[arg(value_name = "INPUT")]
    pub input: Option<PathBuf>,

    /// Path to policy file (TOML format).
    #[arg(long)]
    pub policy: Option<PathBuf>,

    /// Path to baseline receipt for ratchet comparison.
    ///
    /// When provided, gate will evaluate ratchet rules comparing current
    /// metrics against the baseline values.
    #[arg(long, value_name = "PATH")]
    pub baseline: Option<PathBuf>,

    /// Path to ratchet config file (TOML format).
    ///
    /// Defines rules for comparing current metrics against baseline.
    /// Can also be specified inline in tokmd.toml under [[gate.ratchet]].
    #[arg(long, value_name = "PATH")]
    pub ratchet_config: Option<PathBuf>,

    /// Analysis preset (for compute-then-gate mode).
    #[arg(long, value_enum)]
    pub preset: Option<AnalysisPreset>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = GateFormat::Text)]
    pub format: GateFormat,

    /// Fail fast on first error.
    #[arg(long)]
    pub fail_fast: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum GateFormat {
    /// Human-readable text output.
    #[default]
    Text,
    /// JSON output.
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_format_default_is_text() {
        assert_eq!(GateFormat::default(), GateFormat::Text);
    }
}
