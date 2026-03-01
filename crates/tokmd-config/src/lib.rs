//! # tokmd-config
//!
//! **Tier 4 (Configuration)**
//!
//! This crate defines the CLI arguments and configuration file structures.
//! Currently it couples strict configuration schemas with Clap CLI parsing.
//!
//! ## What belongs here
//! * Clap `Parser`, `Args`, `Subcommand` structs
//! * Configuration file struct definitions (Serde)
//! * Default values and enums
//!
//! ## What does NOT belong here
//! * Business logic
//! * I/O operations (except config file parsing)
//! * Higher-tier crate dependencies
//!
//! ## Future Direction
//! * Split into `tokmd-settings` (pure config) and `tokmd-cli` (Clap parsing)

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
pub use tokmd_settings::{Profile, UserConfig};
pub use tokmd_tool_schema::ToolSchemaFormat;
pub use tokmd_types::{
    AnalysisFormat, ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode,
    TableFormat,
};

/// `tokmd` — a small, cross-platform, chat-friendly wrapper around `tokei`.
///
/// Default mode (no subcommand) prints a language summary.
#[derive(Parser, Debug)]
#[command(name = "tokmd", version, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Default options for the implicit `lang` mode (when no subcommand is provided).
    #[command(flatten)]
    pub lang: CliLangArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Configuration profile to use (e.g., "llm_safe", "ci").
    #[arg(long, visible_alias = "view", global = true)]
    pub profile: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct GlobalArgs {
    /// Exclude pattern(s) using gitignore syntax. Repeatable.
    ///
    /// Examples:
    ///   --exclude target
    ///   --exclude "**/*.min.js"
    #[arg(
        long = "exclude",
        visible_alias = "ignore",
        value_name = "PATTERN",
        global = true
    )]
    pub excluded: Vec<String>,

    /// Whether to load `tokei.toml` / `.tokeirc`.
    #[arg(long, value_enum, default_value_t = ConfigMode::Auto)]
    pub config: ConfigMode,

    /// Count hidden files and directories.
    #[arg(long)]
    pub hidden: bool,

    /// Don't respect ignore files (.gitignore, .ignore, etc.).
    ///
    /// Implies --no-ignore-parent, --no-ignore-dot, and --no-ignore-vcs.
    #[arg(long)]
    pub no_ignore: bool,

    /// Don't respect ignore files in parent directories.
    #[arg(long)]
    pub no_ignore_parent: bool,

    /// Don't respect .ignore and .tokeignore files (including in parent directories).
    #[arg(long)]
    pub no_ignore_dot: bool,

    /// Don't respect VCS ignore files (.gitignore, .hgignore, etc.), including in parents.
    #[arg(long, visible_alias = "no-ignore-git")]
    pub no_ignore_vcs: bool,

    /// Treat doc strings as comments (language-dependent).
    #[arg(long)]
    pub treat_doc_strings_as_comments: bool,

    /// Verbose output (repeat for more detail).
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Disable progress spinners.
    #[arg(long, global = true)]
    pub no_progress: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Language summary (default).
    Lang(CliLangArgs),

    /// Module summary (group by path prefixes like `crates/<name>` or `packages/<name>`).
    Module(CliModuleArgs),

    /// Export a file-level dataset (CSV / JSONL / JSON).
    Export(CliExportArgs),

    /// Analyze receipts or paths to produce derived metrics.
    Analyze(CliAnalyzeArgs),

    /// Render a simple SVG badge for a metric.
    Badge(BadgeArgs),

    /// Write a `.tokeignore` template to the target directory.
    Init(InitArgs),

    /// Generate shell completions.
    Completions(CompletionsArgs),

    /// Run a full scan and save receipts to a state directory.
    Run(RunArgs),

    /// Compare two receipts or runs.
    Diff(DiffArgs),

    /// Pack files into an LLM context window within a token budget.
    Context(CliContextArgs),

    /// Check why a file is being ignored (for troubleshooting).
    CheckIgnore(CliCheckIgnoreArgs),

    /// Output CLI schema as JSON for AI agents.
    Tools(ToolsArgs),

    /// Evaluate policy rules against analysis receipts.
    Gate(CliGateArgs),

    /// Generate PR cockpit metrics for code review.
    Cockpit(CockpitArgs),

    /// Generate a complexity baseline for trend tracking.
    Baseline(BaselineArgs),

    /// Bundle codebase for LLM handoff.
    Handoff(HandoffArgs),

    /// Run as a conforming sensor, producing a SensorReport.
    Sensor(SensorArgs),
}

#[derive(Args, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Powershell,
    Zsh,
}

#[derive(Args, Debug, Clone, Default)]
pub struct CliLangArgs {
    /// Paths to scan (directories, files, or globs). Defaults to "."
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Output format [default: md].
    #[arg(long, value_enum)]
    pub format: Option<TableFormat>,

    /// Show only the top N rows (by code lines), plus an "Other" row if needed.
    /// Use 0 to show all rows.
    #[arg(long)]
    pub top: Option<usize>,

    /// Include file counts and average lines per file.
    #[arg(long)]
    pub files: bool,

    /// How to handle embedded languages (tokei "children" / blobs) [default: collapse].
    #[arg(long, value_enum)]
    pub children: Option<ChildrenMode>,
}

#[derive(Args, Debug, Clone)]
pub struct CliModuleArgs {
    /// Paths to scan (directories, files, or globs). Defaults to "."
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Output format [default: md].
    #[arg(long, value_enum)]
    pub format: Option<TableFormat>,

    /// Show only the top N modules (by code lines), plus an "Other" row if needed.
    /// Use 0 to show all rows.
    #[arg(long)]
    pub top: Option<usize>,

    /// Treat these top-level directories as "module roots" [default: crates,packages].
    ///
    /// If a file path starts with one of these roots, the module key will include
    /// `module_depth` segments. Otherwise, the module key is the top-level directory.
    #[arg(long, value_delimiter = ',')]
    pub module_roots: Option<Vec<String>>,

    /// How many path segments to include for module roots [default: 2].
    ///
    /// Example:
    ///   crates/foo/src/lib.rs  (depth=2) => crates/foo
    ///   crates/foo/src/lib.rs  (depth=1) => crates
    #[arg(long)]
    pub module_depth: Option<usize>,

    /// Whether to include embedded languages (tokei "children" / blobs) in module totals [default: separate].
    #[arg(long, value_enum)]
    pub children: Option<ChildIncludeMode>,
}

#[derive(Args, Debug, Clone)]
pub struct CliExportArgs {
    /// Paths to scan (directories, files, or globs). Defaults to "."
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Output format [default: jsonl].
    #[arg(long, value_enum)]
    pub format: Option<ExportFormat>,

    /// Write output to this file instead of stdout.
    #[arg(long, value_name = "PATH", visible_alias = "out")]
    pub output: Option<PathBuf>,

    /// Module roots (see `tokmd module`) [default: crates,packages].
    #[arg(long, value_delimiter = ',')]
    pub module_roots: Option<Vec<String>>,

    /// Module depth (see `tokmd module`) [default: 2].
    #[arg(long)]
    pub module_depth: Option<usize>,

    /// Whether to include embedded languages (tokei "children" / blobs) [default: separate].
    #[arg(long, value_enum)]
    pub children: Option<ChildIncludeMode>,

    /// Drop rows with fewer than N code lines [default: 0].
    #[arg(long)]
    pub min_code: Option<usize>,

    /// Stop after emitting N rows (0 = unlimited) [default: 0].
    #[arg(long)]
    pub max_rows: Option<usize>,

    /// Include a meta record (JSON / JSONL only). Enabled by default.
    #[arg(long, action = clap::ArgAction::Set)]
    pub meta: Option<bool>,

    /// Redact paths (and optionally module names) for safer copy/paste into LLMs [default: none].
    #[arg(long, value_enum)]
    pub redact: Option<RedactMode>,

    /// Strip this prefix from paths before output (helps when paths are absolute).
    #[arg(long, value_name = "PATH")]
    pub strip_prefix: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
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

    /// Limit bytes per file during content scans.
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

#[derive(Args, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Target directory (defaults to ".").
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub dir: PathBuf,

    /// Overwrite an existing `.tokeignore`.
    #[arg(long)]
    pub force: bool,

    /// Print the template to stdout instead of writing a file.
    #[arg(long)]
    pub print: bool,

    /// Which template profile to use.
    #[arg(long, value_enum, default_value_t = InitProfile::Default)]
    pub template: InitProfile,

    /// Skip interactive wizard and use defaults.
    #[arg(long)]
    pub non_interactive: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisPreset {
    Receipt,
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
pub enum BadgeMetric {
    Lines,
    Tokens,
    Bytes,
    Doc,
    Blank,
    Hotspot,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InitProfile {
    Default,
    Rust,
    Node,
    Mono,
    Python,
    Go,
    Cpp,
}

#[derive(Args, Debug, Clone)]
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
    #[arg(long)]
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
pub struct CliCheckIgnoreArgs {
    /// File path(s) to check.
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<PathBuf>,

    /// Show verbose output with rule sources.
    #[arg(long, short = 'v')]
    pub verbose: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ToolsArgs {
    /// Output format for the tool schema.
    #[arg(long, value_enum, default_value_t = ToolSchemaFormat::Jsonschema)]
    pub format: ToolSchemaFormat,

    /// Pretty-print JSON output.
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Args, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
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
    pub output: Option<std::path::PathBuf>,

    /// Write cockpit artifacts (report.json, comment.md) to directory.
    #[arg(long, value_name = "DIR")]
    pub artifacts_dir: Option<std::path::PathBuf>,

    /// Path to baseline receipt for trend comparison.
    ///
    /// When provided, cockpit will compute delta metrics showing how
    /// the current state compares to the baseline.
    #[arg(long, value_name = "PATH")]
    pub baseline: Option<std::path::PathBuf>,

    /// Diff range syntax: two-dot (default) or three-dot.
    #[arg(long, value_enum, default_value_t = DiffRangeMode::TwoDot)]
    pub diff_range: DiffRangeMode,

    /// Run in sensor mode for CI integration.
    ///
    /// When enabled:
    /// - Always writes sensor.report.v1 envelope to artifacts_dir/report.json
    /// - Exits 0 if receipt written successfully (verdict in envelope instead of exit code)
    /// - Reports capability availability for "No Green By Omission"
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
    /// Section-based output for PR template filling.
    Sections,
}

#[derive(Args, Debug, Clone)]
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
    #[arg(long)]
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
    pub output: std::path::PathBuf,

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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DiffRangeMode {
    /// Two-dot syntax (A..B) - direct diff between commits.
    #[default]
    TwoDot,
    /// Three-dot syntax (A...B) - diff from merge-base.
    ThreeDot,
}

// =============================================================================
// TOML Configuration File Structures (re-exported from tokmd-settings)
// =============================================================================

pub use tokmd_settings::{
    AnalyzeConfig, BadgeConfig, ContextConfig, ExportConfig, GateConfig, GateRule, ModuleConfig,
    RatchetRuleConfig, ScanConfig, TomlConfig, TomlResult, ViewProfile,
};

// ============================================================
// Conversions between CLI GlobalArgs and Tier-0 ScanOptions
// ============================================================

impl From<&GlobalArgs> for tokmd_settings::ScanOptions {
    fn from(g: &GlobalArgs) -> Self {
        Self {
            excluded: g.excluded.clone(),
            config: g.config,
            hidden: g.hidden,
            no_ignore: g.no_ignore,
            no_ignore_parent: g.no_ignore_parent,
            no_ignore_dot: g.no_ignore_dot,
            no_ignore_vcs: g.no_ignore_vcs,
            treat_doc_strings_as_comments: g.treat_doc_strings_as_comments,
        }
    }
}

impl From<GlobalArgs> for tokmd_settings::ScanOptions {
    fn from(g: GlobalArgs) -> Self {
        Self::from(&g)
    }
}
