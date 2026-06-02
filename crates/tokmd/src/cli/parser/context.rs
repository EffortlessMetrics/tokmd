use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd context --budget 128k --mode bundle --output context.txt\n  tokmd context crates/tokmd xtask --strategy spread --budget 200k"
)]
pub struct CliContextArgs {
    /// Paths to scan (directories, files, or globs). Defaults to "."
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Token budget with optional k/m/g suffix, or 'unlimited' (e.g., "128k", "1m", "1g", "unlimited").
    #[arg(long, default_value = "128k")]
    pub budget: String,

    /// Packing strategy.
    #[arg(long, value_enum, default_value_t = ContextStrategy::Greedy)]
    pub strategy: ContextStrategy,

    /// Metric to rank files by.
    #[arg(long, value_enum, default_value_t = ValueMetric::Code)]
    pub rank_by: ValueMetric,

    /// Output mode.
    #[arg(long = "mode", value_enum, default_value_t = ContextOutput::List)]
    pub output_mode: ContextOutput,

    /// Strip blank lines from bundle output.
    #[arg(long)]
    pub compress: bool,

    /// Disable smart exclusion of lockfiles, minified files, and generated artifacts.
    #[arg(long)]
    pub no_smart_exclude: bool,

    /// Module roots (see `tokmd module`).
    #[arg(long, value_delimiter = ',')]
    pub module_roots: Option<Vec<String>>,

    /// Module depth (see `tokmd module`).
    #[arg(long, visible_alias = "depth")]
    pub module_depth: Option<usize>,

    /// Enable git-based ranking (required for churn/hotspot).
    #[arg(long)]
    pub git: bool,

    /// Disable git-based ranking.
    #[arg(long = "no-git")]
    pub no_git: bool,

    /// Maximum commits to scan for git metrics.
    #[arg(long, default_value = "1000")]
    pub max_commits: usize,

    /// Maximum files per commit to process.
    #[arg(long, default_value = "100")]
    pub max_commit_files: usize,

    /// Write output to file instead of stdout.
    #[arg(long, value_name = "PATH", visible_alias = "out")]
    pub output: Option<PathBuf>,

    /// Overwrite existing output file.
    #[arg(long)]
    pub force: bool,

    /// Write bundle to directory with manifest (for large outputs).
    #[arg(long, value_name = "DIR", conflicts_with = "output")]
    pub bundle_dir: Option<PathBuf>,

    /// Warn if output exceeds N bytes (default: 10MB, 0=disable).
    #[arg(long, default_value = "10485760")]
    pub max_output_bytes: u64,

    /// Append JSONL record to log file (metadata only, not content).
    #[arg(long, value_name = "PATH")]
    pub log: Option<PathBuf>,

    /// Maximum fraction of budget a single file may consume (0.0–1.0).
    #[arg(long, default_value = "0.15")]
    pub max_file_pct: f64,

    /// Hard cap on tokens per file (overrides percentage-based cap).
    #[arg(long)]
    pub max_file_tokens: Option<usize>,

    /// Error if git scores are unavailable when using churn/hotspot ranking.
    #[arg(long)]
    pub require_git_scores: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ContextStrategy {
    /// Select files by value until budget is exhausted.
    #[default]
    Greedy,
    /// Round-robin across modules/languages for coverage, then greedy fill.
    Spread,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ValueMetric {
    /// Rank by lines of code.
    #[default]
    Code,
    /// Rank by token count.
    Tokens,
    /// Rank by git churn (requires git feature).
    Churn,
    /// Rank by hotspot score (requires git feature).
    Hotspot,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ContextOutput {
    /// Print list of selected files with stats.
    #[default]
    List,
    /// Concatenate file contents into a single bundle.
    Bundle,
    /// Output JSON receipt with selection details.
    Json,
}

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd handoff crates/tokmd xtask --out-dir .handoff --budget 128k\n  tokmd handoff . --review-packet-dir .tokmd/review --proof-route target/ci/proof-pack-route.json --proof-plan target/proof/proof-plan.json"
)]
pub struct HandoffArgs {
    /// Paths to scan (directories, files, or globs). Defaults to ".".
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Output directory for handoff artifacts.
    #[arg(long, default_value = ".handoff")]
    pub out_dir: PathBuf,

    /// Token budget with optional k/m/g suffix, or 'unlimited' (e.g., "128k", "1m", "1g", "unlimited").
    #[arg(long, default_value = "128k")]
    pub budget: String,

    /// Packing strategy for code bundle.
    #[arg(long, value_enum, default_value_t = ContextStrategy::Greedy)]
    pub strategy: ContextStrategy,

    /// Metric to rank files by for packing.
    #[arg(long, value_enum, default_value_t = ValueMetric::Hotspot)]
    pub rank_by: ValueMetric,

    /// Intelligence preset level.
    #[arg(long, value_enum, default_value_t = HandoffPreset::Risk)]
    pub preset: HandoffPreset,

    /// Module roots (see `tokmd module`).
    #[arg(long, value_delimiter = ',')]
    pub module_roots: Option<Vec<String>>,

    /// Module depth (see `tokmd module`).
    #[arg(long, visible_alias = "depth")]
    pub module_depth: Option<usize>,

    /// Overwrite existing output directory.
    #[arg(long)]
    pub force: bool,

    /// Strip blank lines from code bundle.
    #[arg(long)]
    pub compress: bool,

    /// Disable smart exclusion of lockfiles, minified files, and generated artifacts.
    #[arg(long)]
    pub no_smart_exclude: bool,

    /// Disable git-based features.
    #[arg(long = "no-git")]
    pub no_git: bool,

    /// Maximum commits to scan for git metrics.
    #[arg(long, default_value = "1000")]
    pub max_commits: usize,

    /// Maximum files per commit to process.
    #[arg(long, default_value = "100")]
    pub max_commit_files: usize,

    /// Maximum fraction of budget a single file may consume (0.0–1.0).
    #[arg(long, default_value = "0.15")]
    pub max_file_pct: f64,

    /// Hard cap on tokens per file (overrides percentage-based cap).
    #[arg(long)]
    pub max_file_tokens: Option<usize>,

    /// Link an existing cockpit review packet directory from the handoff bundle.
    #[arg(long)]
    pub review_packet_dir: Option<PathBuf>,

    /// Link an existing review-packet verifier receipt from the handoff bundle.
    #[arg(long)]
    pub review_packet_check: Option<PathBuf>,

    /// Link an existing affected-proof report from the handoff bundle.
    #[arg(long)]
    pub affected: Option<PathBuf>,

    /// Link an existing proof-plan report from the handoff bundle.
    #[arg(long)]
    pub proof_plan: Option<PathBuf>,

    /// Link an existing proof-pack route receipt from the handoff bundle.
    #[arg(long)]
    pub proof_route: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HandoffPreset {
    /// Minimal: tree + map only.
    Minimal,
    /// Standard: + complexity, derived.
    Standard,
    /// Risk: + hotspots, coupling (default).
    #[default]
    Risk,
    /// Deep: everything.
    Deep,
}
