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

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
pub use tokmd_types::{
    ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode, TableFormat,
};

/// `tokmd` — a small, cross-platform, chat-friendly wrapper around `tokei`.
///
/// Default mode (no subcommand) prints a language summary.
#[derive(Parser, Debug)]
#[command(name = "tokmd", version, about, long_about = None)]
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
    #[arg(long = "exclude", visible_alias = "ignore", value_name = "PATTERN")]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub profiles: BTreeMap<String, Profile>,
    pub repos: BTreeMap<String, String>, // "owner/repo" -> "profile_name"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    // Shared
    pub format: Option<String>, // "json", "md", "tsv", "csv", "jsonl"
    pub top: Option<usize>,

    // Lang
    pub files: Option<bool>,

    // Module / Export
    pub module_roots: Option<Vec<String>>,
    pub module_depth: Option<usize>,
    pub min_code: Option<usize>,
    pub max_rows: Option<usize>,
    pub redact: Option<RedactMode>,
    pub meta: Option<bool>,

    // "children" can be ChildrenMode or ChildIncludeMode string
    pub children: Option<String>,
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
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,

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
    #[arg(long)]
    pub out: Option<PathBuf>,
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
pub enum AnalysisFormat {
    Md,
    Json,
    Jsonld,
    Xml,
    Svg,
    Mermaid,
    Obj,
    Midi,
    Tree,
    Html,
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

    /// Token budget with optional k/m suffix (e.g., "128k", "1m", "50000").
    #[arg(long, default_value = "128k")]
    pub budget: String,

    /// Packing strategy.
    #[arg(long, value_enum, default_value_t = ContextStrategy::Greedy)]
    pub strategy: ContextStrategy,

    /// Metric to rank files by.
    #[arg(long, value_enum, default_value_t = ValueMetric::Code)]
    pub rank_by: ValueMetric,

    /// Output mode.
    #[arg(long, value_enum, default_value_t = ContextOutput::List)]
    pub output: ContextOutput,

    /// Strip blank lines from bundle output.
    #[arg(long)]
    pub compress: bool,

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
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,

    /// Overwrite existing output file.
    #[arg(long)]
    pub force: bool,

    /// Write bundle to directory with manifest (for large outputs).
    #[arg(long, value_name = "DIR", conflicts_with = "out")]
    pub bundle_dir: Option<PathBuf>,

    /// Warn if output exceeds N bytes (default: 10MB, 0=disable).
    #[arg(long, default_value = "10485760")]
    pub max_output_bytes: u64,

    /// Append JSONL record to log file (metadata only, not content).
    #[arg(long, value_name = "PATH")]
    pub log: Option<PathBuf>,
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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ToolSchemaFormat {
    /// OpenAI function calling format.
    Openai,
    /// Anthropic tool use format.
    Anthropic,
    /// JSON Schema Draft 7 format.
    #[default]
    Jsonschema,
    /// Raw clap structure dump.
    Clap,
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

    /// Token budget with optional k/m suffix (e.g., "128k", "1m", "50000").
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

    /// Disable git-based features.
    #[arg(long = "no-git")]
    pub no_git: bool,

    /// Maximum commits to scan for git metrics.
    #[arg(long, default_value = "1000")]
    pub max_commits: usize,

    /// Maximum files per commit to process.
    #[arg(long, default_value = "100")]
    pub max_commit_files: usize,
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
pub enum DiffRangeMode {
    /// Two-dot syntax (A..B) - direct diff between commits.
    #[default]
    TwoDot,
    /// Three-dot syntax (A...B) - diff from merge-base.
    ThreeDot,
}

// =============================================================================
// TOML Configuration File Structures
// =============================================================================

/// Root TOML configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TomlConfig {
    /// Scan settings (applies to all commands).
    pub scan: ScanConfig,

    /// Module command settings.
    pub module: ModuleConfig,

    /// Export command settings.
    pub export: ExportConfig,

    /// Analyze command settings.
    pub analyze: AnalyzeConfig,

    /// Context command settings.
    pub context: ContextConfig,

    /// Badge command settings.
    pub badge: BadgeConfig,

    /// Gate command settings.
    pub gate: GateConfig,

    /// Named view profiles (e.g., [view.llm], [view.ci]).
    #[serde(default)]
    pub view: HashMap<String, ViewProfile>,
}

/// Scan settings shared by all commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanConfig {
    /// Paths to scan (default: ["."])
    pub paths: Option<Vec<String>>,

    /// Glob patterns to exclude.
    pub exclude: Option<Vec<String>>,

    /// Include hidden files and directories.
    pub hidden: Option<bool>,

    /// Config file strategy for tokei: "auto" or "none".
    pub config: Option<String>,

    /// Disable all ignore files.
    pub no_ignore: Option<bool>,

    /// Disable parent directory ignore file traversal.
    pub no_ignore_parent: Option<bool>,

    /// Disable .ignore/.tokeignore files.
    pub no_ignore_dot: Option<bool>,

    /// Disable .gitignore files.
    pub no_ignore_vcs: Option<bool>,

    /// Treat doc comments as comments instead of code.
    pub doc_comments: Option<bool>,
}

/// Module command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ModuleConfig {
    /// Root directories for module grouping.
    pub roots: Option<Vec<String>>,

    /// Depth for module grouping.
    pub depth: Option<usize>,

    /// Children handling: "collapse" or "separate".
    pub children: Option<String>,
}

/// Export command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportConfig {
    /// Minimum lines of code to include.
    pub min_code: Option<usize>,

    /// Maximum rows in output.
    pub max_rows: Option<usize>,

    /// Redaction mode: "none", "paths", or "all".
    pub redact: Option<String>,

    /// Output format: "jsonl", "csv", "json", "cyclonedx".
    pub format: Option<String>,

    /// Children handling: "collapse" or "separate".
    pub children: Option<String>,
}

/// Analyze command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalyzeConfig {
    /// Analysis preset.
    pub preset: Option<String>,

    /// Context window size for utilization analysis.
    pub window: Option<usize>,

    /// Output format.
    pub format: Option<String>,

    /// Force git metrics on/off.
    pub git: Option<bool>,

    /// Max files for asset/deps/content scans.
    pub max_files: Option<usize>,

    /// Max total bytes for content scans.
    pub max_bytes: Option<u64>,

    /// Max bytes per file for content scans.
    pub max_file_bytes: Option<u64>,

    /// Max commits for git metrics.
    pub max_commits: Option<usize>,

    /// Max files per commit for git metrics.
    pub max_commit_files: Option<usize>,

    /// Import graph granularity: "module" or "file".
    pub granularity: Option<String>,
}

/// Context command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    /// Token budget with optional k/m suffix.
    pub budget: Option<String>,

    /// Packing strategy: "greedy" or "spread".
    pub strategy: Option<String>,

    /// Ranking metric: "code", "tokens", "churn", "hotspot".
    pub rank_by: Option<String>,

    /// Output mode: "list", "bundle", "json".
    pub output: Option<String>,

    /// Strip blank lines from bundle output.
    pub compress: Option<bool>,
}

/// Badge command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BadgeConfig {
    /// Default metric for badges.
    pub metric: Option<String>,
}

/// Gate command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GateConfig {
    /// Path to policy file.
    pub policy: Option<String>,

    /// Path to baseline file for ratchet comparison.
    pub baseline: Option<String>,

    /// Analysis preset for compute-then-gate mode.
    pub preset: Option<String>,

    /// Fail fast on first error.
    pub fail_fast: Option<bool>,

    /// Inline policy rules.
    pub rules: Option<Vec<GateRule>>,

    /// Inline ratchet rules for baseline comparison.
    pub ratchet: Option<Vec<RatchetRuleConfig>>,

    /// Allow missing baseline values (treat as pass).
    pub allow_missing_baseline: Option<bool>,

    /// Allow missing current values (treat as pass).
    pub allow_missing_current: Option<bool>,
}

/// A single ratchet rule for baseline comparison (TOML configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetRuleConfig {
    /// JSON Pointer to the metric (e.g., "/complexity/avg_cyclomatic").
    pub pointer: String,

    /// Maximum allowed percentage increase from baseline.
    #[serde(default)]
    pub max_increase_pct: Option<f64>,

    /// Maximum allowed absolute value (hard ceiling).
    #[serde(default)]
    pub max_value: Option<f64>,

    /// Rule severity level: "error" (default) or "warn".
    #[serde(default)]
    pub level: Option<String>,

    /// Human-readable description of the rule.
    #[serde(default)]
    pub description: Option<String>,
}

/// A single gate policy rule (for inline TOML configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRule {
    /// Human-readable name for the rule.
    pub name: String,

    /// JSON Pointer to the value to check (RFC 6901).
    pub pointer: String,

    /// Comparison operator.
    pub op: String,

    /// Single value for comparison.
    #[serde(default)]
    pub value: Option<serde_json::Value>,

    /// Multiple values for "in" operator.
    #[serde(default)]
    pub values: Option<Vec<serde_json::Value>>,

    /// Negate the result.
    #[serde(default)]
    pub negate: bool,

    /// Rule severity level: "error" or "warn".
    #[serde(default)]
    pub level: Option<String>,

    /// Custom failure message.
    #[serde(default)]
    pub message: Option<String>,
}

/// A named view profile that can override settings for specific use cases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewProfile {
    // Shared settings
    /// Output format.
    pub format: Option<String>,

    /// Show only top N rows.
    pub top: Option<usize>,

    // Lang settings
    /// Include file counts in lang output.
    pub files: Option<bool>,

    // Module / Export settings
    /// Module roots for grouping.
    pub module_roots: Option<Vec<String>>,

    /// Module depth for grouping.
    pub module_depth: Option<usize>,

    /// Minimum lines of code.
    pub min_code: Option<usize>,

    /// Maximum rows in output.
    pub max_rows: Option<usize>,

    /// Redaction mode.
    pub redact: Option<String>,

    /// Include metadata record.
    pub meta: Option<bool>,

    /// Children handling mode.
    pub children: Option<String>,

    // Analyze settings
    /// Analysis preset.
    pub preset: Option<String>,

    /// Context window size.
    pub window: Option<usize>,

    // Context settings
    /// Token budget.
    pub budget: Option<String>,

    /// Packing strategy.
    pub strategy: Option<String>,

    /// Ranking metric.
    pub rank_by: Option<String>,

    /// Output mode for context.
    pub output: Option<String>,

    /// Strip blank lines.
    pub compress: Option<bool>,

    // Badge settings
    /// Badge metric.
    pub metric: Option<String>,
}

impl TomlConfig {
    /// Load configuration from a TOML string.
    pub fn parse(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Load configuration from a file path.
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

use std::path::Path;

/// Result type alias for TOML parsing errors.
pub type TomlResult<T> = Result<T, toml::de::Error>;

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
