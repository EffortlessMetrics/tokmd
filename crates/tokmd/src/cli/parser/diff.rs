//! Diff command parser types.
//!
//! This module owns the clap/serde contract for `tokmd diff` while the parent
//! parser module keeps the top-level command dispatch shape.

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd diff .runs/baseline .runs/current\n  tokmd diff --from main --to HEAD"
)]
pub struct DiffArgs {
    /// Base receipt/run or git ref to compare from.
    #[arg(long)]
    pub from: Option<String>,

    /// Target receipt/run or git ref to compare to.
    #[arg(long)]
    pub to: Option<String>,

    /// Two refs/paths to compare (positional).
    #[arg(value_name = "REF", num_args = 2)]
    pub refs: Vec<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = DiffFormat::Md)]
    pub format: DiffFormat,

    /// Compact output for narrow terminals (summary table only).
    #[arg(long)]
    pub compact: bool,

    /// Color policy for terminal output.
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    pub color: ColorMode,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DiffFormat {
    /// Markdown table output.
    #[default]
    Md,
    /// JSON receipt with envelope metadata.
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ColorMode {
    /// Enable color when stdout is a TTY and color env vars allow it.
    #[default]
    Auto,
    /// Always emit ANSI color.
    Always,
    /// Never emit ANSI color.
    Never,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_format_default_is_md() {
        assert_eq!(DiffFormat::default(), DiffFormat::Md);
    }

    #[test]
    fn diff_format_serde_roundtrip() {
        for variant in [DiffFormat::Md, DiffFormat::Json] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: DiffFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn color_mode_default_is_auto() {
        assert_eq!(ColorMode::default(), ColorMode::Auto);
    }
}
