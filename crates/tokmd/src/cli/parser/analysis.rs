use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

use super::AnalysisFormat;

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd analyze --preset receipt --format md\n  tokmd analyze . --preset risk --output-dir .runs/analysis"
)]
pub struct CliAnalyzeArgs {
    /// Inputs to analyze (run dir, receipt.json, export.jsonl, or paths).
    #[arg(value_name = "INPUT", default_value = ".")]
    pub inputs: Vec<PathBuf>,

    /// Analysis preset to run [default: receipt].
    #[arg(long, value_enum)]
    pub preset: Option<AnalysisPreset>,

    /// Output format [default: md].
    #[arg(long, value_enum)]
    pub format: Option<AnalysisFormat>,

    /// Context window size (tokens) for utilization bars.
    #[arg(long)]
    pub window: Option<usize>,

    /// Force-enable git-based metrics.
    #[arg(long, action = clap::ArgAction::SetTrue, conflicts_with = "no_git")]
    pub git: bool,

    /// Disable git-based metrics.
    #[arg(long = "no-git", action = clap::ArgAction::SetTrue, conflicts_with = "git")]
    pub no_git: bool,

    /// Output directory for analysis artifacts.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Limit how many files are walked for asset/deps/content scans.
    #[arg(long)]
    pub max_files: Option<usize>,

    /// Limit total bytes read during content scans.
    #[arg(long)]
    pub max_bytes: Option<u64>,

    /// Limit bytes per file during content scans [default for file-backed scans: 131072].
    #[arg(long)]
    pub max_file_bytes: Option<u64>,

    /// Limit how many commits are scanned for git metrics.
    #[arg(long)]
    pub max_commits: Option<usize>,

    /// Limit files per commit when scanning git history.
    #[arg(long)]
    pub max_commit_files: Option<usize>,

    /// Import graph granularity [default: module].
    #[arg(long, value_enum)]
    pub granularity: Option<ImportGranularity>,

    /// Effort model for estimate calculations [default: cocomo81-basic].
    #[arg(long)]
    pub effort_model: Option<EffortModelKind>,

    /// Effort layer for report detail [default: full].
    #[arg(long)]
    pub effort_layer: Option<EffortLayer>,

    /// Base reference for effort delta computation.
    #[arg(long = "effort-base-ref")]
    pub effort_base_ref: Option<String>,

    /// Head reference for effort delta computation.
    #[arg(long = "effort-head-ref")]
    pub effort_head_ref: Option<String>,

    /// Enable Monte Carlo simulation for effort estimation.
    #[arg(long)]
    pub monte_carlo: bool,

    /// Monte Carlo iterations when effort estimation is enabled [default: 10000].
    #[arg(long = "mc-iterations")]
    pub mc_iterations: Option<usize>,

    /// Monte Carlo seed for deterministic effort estimation.
    #[arg(long = "mc-seed")]
    pub mc_seed: Option<u64>,

    /// Include function-level complexity details in output.
    #[arg(long)]
    pub detail_functions: bool,

    /// Enable near-duplicate file detection (opt-in).
    #[arg(long)]
    pub near_dup: bool,

    /// Near-duplicate similarity threshold (0.0–1.0) [default: 0.80].
    #[arg(long, default_value = "0.80")]
    pub near_dup_threshold: f64,

    /// Maximum files to analyze for near-duplicates [default: 2000].
    #[arg(long, default_value = "2000")]
    pub near_dup_max_files: usize,

    /// Near-duplicate comparison scope [default: module].
    #[arg(long, value_enum)]
    pub near_dup_scope: Option<NearDupScope>,

    /// Maximum near-duplicate pairs to emit (truncation guardrail) [default: 10000].
    #[arg(long, default_value = "10000")]
    pub near_dup_max_pairs: usize,

    /// Exclude files matching this glob pattern from near-duplicate analysis. Repeatable.
    #[arg(long, value_name = "GLOB")]
    pub near_dup_exclude: Vec<String>,

    /// Explain a metric or finding key and exit.
    #[arg(long, value_name = "KEY")]
    pub explain: Option<String>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisPreset {
    Receipt,
    Estimate,
    BunUb,
    Health,
    Risk,
    Supply,
    Architecture,
    Topics,
    Security,
    Identity,
    Git,
    Deep,
    Fun,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportGranularity {
    Module,
    File,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffortModelKind {
    Cocomo81Basic,
    Cocomo2Early,
    Ensemble,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffortLayer {
    Headline,
    Why,
    Full,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NearDupScope {
    /// Compare files within the same module.
    #[default]
    Module,
    /// Compare files within the same language.
    Lang,
    /// Compare all files globally.
    Global,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_preset_serde_roundtrip() {
        for variant in [
            AnalysisPreset::Receipt,
            AnalysisPreset::Estimate,
            AnalysisPreset::BunUb,
            AnalysisPreset::Health,
            AnalysisPreset::Risk,
            AnalysisPreset::Supply,
            AnalysisPreset::Architecture,
            AnalysisPreset::Topics,
            AnalysisPreset::Security,
            AnalysisPreset::Identity,
            AnalysisPreset::Git,
            AnalysisPreset::Deep,
            AnalysisPreset::Fun,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AnalysisPreset = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn analysis_preset_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&AnalysisPreset::Receipt).unwrap(),
            "\"receipt\""
        );
        assert_eq!(
            serde_json::to_string(&AnalysisPreset::Deep).unwrap(),
            "\"deep\""
        );
        assert_eq!(
            serde_json::to_string(&AnalysisPreset::BunUb).unwrap(),
            "\"bun-ub\""
        );
    }

    #[test]
    fn near_dup_scope_default_is_module() {
        assert_eq!(NearDupScope::default(), NearDupScope::Module);
    }
}
