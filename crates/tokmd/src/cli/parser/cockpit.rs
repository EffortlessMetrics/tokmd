use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd cockpit --base origin/main --head HEAD --format comment\n  tokmd cockpit --base origin/main --head HEAD --review-packet-dir .tokmd/review\n  tokmd cockpit --base origin/main --head HEAD --proof-run-summary target/proof/proof-run-summary.json --review-packet-dir .tokmd/review\n  tokmd cockpit --base origin/main --head HEAD --doc-artifacts-check target/docs/doc-artifacts-check.json --review-packet-dir .tokmd/review"
)]
pub struct CockpitArgs {
    /// Base reference to compare from (default: main).
    #[arg(long, default_value = "main")]
    pub base: String,

    /// Head reference to compare to (default: HEAD).
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Output format.
    #[arg(long, value_enum, default_value_t = CockpitFormat::Json)]
    pub format: CockpitFormat,

    /// Output file (stdout if omitted).
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Write cockpit artifacts (`cockpit.json`, `report.json`, `comment.md`) to directory.
    #[arg(long, value_name = "DIR")]
    pub artifacts_dir: Option<PathBuf>,

    /// Write review packet artifacts (`manifest.json`, `cockpit.json`, `evidence.json`, `review-map.json`, `review-map.md`, `comment.md`) to directory.
    #[arg(long, value_name = "DIR")]
    pub review_packet_dir: Option<PathBuf>,

    /// Path to baseline receipt for trend comparison.
    ///
    /// When provided, cockpit will compute delta metrics showing how
    /// the current state compares to the baseline.
    #[arg(long, value_name = "PATH")]
    pub baseline: Option<PathBuf>,

    /// Import required proof-run summary evidence into review packets.
    #[arg(long, value_name = "PATH")]
    pub proof_run_summary: Option<PathBuf>,

    /// Import proof-run observation evidence into review packets.
    #[arg(long, value_name = "PATH")]
    pub proof_observation: Option<PathBuf>,

    /// Import proof-executor observation evidence into review packets.
    #[arg(long, value_name = "PATH")]
    pub executor_observation: Option<PathBuf>,

    /// Import coverage receipt evidence into review packets.
    #[arg(long, value_name = "PATH")]
    pub coverage_receipt: Option<PathBuf>,

    /// Import proof-pack route evidence into review packets.
    #[arg(long, value_name = "PATH")]
    pub proof_route: Option<PathBuf>,

    /// Import doc-artifacts checker receipt evidence into review packets.
    #[arg(long, value_name = "PATH")]
    pub doc_artifacts_check: Option<PathBuf>,

    /// Diff range syntax: two-dot (default) or three-dot.
    #[arg(long, value_enum, default_value_t = DiffRangeMode::TwoDot)]
    pub diff_range: DiffRangeMode,

    /// Run in sensor mode for CI integration.
    ///
    /// When enabled:
    /// - Writes only sensor.report.v1 envelope to artifacts_dir/report.json
    /// - Exits 0 if receipt written successfully (verdict in envelope instead of exit code)
    #[arg(long)]
    pub sensor_mode: bool,
}

#[derive(Args, Debug, Clone)]
pub struct BaselineArgs {
    /// Target path to analyze.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output path for baseline file.
    #[arg(long, default_value = ".tokmd/baseline.json")]
    pub output: PathBuf,

    /// Include determinism baseline (hash build artifacts).
    #[arg(long)]
    pub determinism: bool,

    /// Force overwrite existing baseline.
    #[arg(long, short)]
    pub force: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CockpitFormat {
    /// JSON output with full metrics.
    #[default]
    Json,
    /// Markdown output for human readability.
    Md,
    /// Compact PR comment markdown.
    Comment,
    /// Section-based output for PR template filling.
    Sections,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DiffRangeMode {
    /// Two-dot syntax (A..B) - direct diff between commits.
    #[default]
    TwoDot,
    /// Three-dot syntax (A...B) - diff from merge-base.
    ThreeDot,
}
