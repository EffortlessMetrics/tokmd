//! Run command parser types.
//!
//! This module owns the clap contract for `tokmd run` while the parent parser
//! module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::Args;

use super::{AnalysisPreset, RedactMode};

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd run --name baseline\n  tokmd run crates --analysis health --output-dir .runs/tokmd"
)]
pub struct RunArgs {
    /// Paths to scan.
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Output directory for artifacts (defaults to `.runs/tokmd` inside the repo, or system temp if not possible).
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Tag or name for this run.
    #[arg(long)]
    pub name: Option<String>,

    /// Also emit analysis receipts using this preset.
    #[arg(long, value_enum)]
    pub analysis: Option<AnalysisPreset>,

    /// Redact paths (and optionally module names) for safer copy/paste into LLMs.
    #[arg(long, value_enum)]
    pub redact: Option<RedactMode>,
}
