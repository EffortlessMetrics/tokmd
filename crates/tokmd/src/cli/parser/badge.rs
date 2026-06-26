//! Badge command parser types.
//!
//! This module owns the clap contract for `tokmd badge` while the parent parser
//! module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

use super::AnalysisPreset;

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd badge --metric lines\n  tokmd badge --metric hotspot --preset risk --output badge.svg"
)]
pub struct BadgeArgs {
    /// Inputs to analyze (run dir, receipt.json, export.jsonl, or paths).
    #[arg(value_name = "INPUT", default_value = ".")]
    pub inputs: Vec<PathBuf>,

    /// Metric to render.
    #[arg(long, value_enum)]
    pub metric: BadgeMetric,

    /// Optional analysis preset to use for the badge.
    #[arg(long, value_enum)]
    pub preset: Option<AnalysisPreset>,

    /// Force-enable git-based metrics.
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "no_git")]
    pub git: bool,

    /// Disable git-based metrics.
    #[arg(long = "no-git", action = clap::ArgAction::SetTrue, conflicts_with = "git")]
    pub no_git: bool,

    /// Limit how many commits are scanned for git metrics.
    #[arg(long)]
    pub max_commits: Option<usize>,

    /// Limit files per commit when scanning git history.
    #[arg(long)]
    pub max_commit_files: Option<usize>,

    /// Output file for the badge (defaults to stdout).
    #[arg(long, visible_alias = "out")]
    pub output: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BadgeMetric {
    Lines,
    Tokens,
    Bytes,
    Doc,
    Blank,
    Hotspot,
}
