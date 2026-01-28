//! # tokmd-config
//!
//! **Tier 3 (Configuration)**
//!
//! This crate defines the CLI arguments and configuration file structures.
//! Currently it couples strict configuration schemas with Clap CLI parsing.
//!
//! ## What belongs here
//! * Clap `Parser`, `Args`, `Subcommand` structs
//! * Configuration file struct definitions (Serde)
//! * Default values
//!
//! ## Future Direction
//! * Split into `tokmd-settings` (pure config) and `tokmd-cli` (Clap parsing)

use std::collections::HashMap;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

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
    #[arg(long, global = true)]
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
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Language summary (default).
    Lang(CliLangArgs),

    /// Module summary (group by path prefixes like crates/<name> or packages/<name>).
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub profiles: HashMap<String, Profile>,
    pub repos: HashMap<String, String>, // "owner/repo" -> "profile_name"
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

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<TableFormat>,

    /// Show only the top N rows (by code lines), plus an "Other" row if needed.
    /// Use 0 to show all rows.
    #[arg(long)]
    pub top: Option<usize>,

    /// Include file counts and average lines per file.
    #[arg(long)]
    pub files: bool,

    /// How to handle embedded languages (tokei "children" / blobs).
    #[arg(long, value_enum)]
    pub children: Option<ChildrenMode>,
}

#[derive(Args, Debug, Clone)]
pub struct CliModuleArgs {
    /// Paths to scan (directories, files, or globs). Defaults to "."
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<TableFormat>,

    /// Show only the top N modules (by code lines), plus an "Other" row if needed.
    /// Use 0 to show all rows.
    #[arg(long)]
    pub top: Option<usize>,

    /// Treat these top-level directories as "module roots".
    ///
    /// If a file path starts with one of these roots, the module key will include
    /// `module_depth` segments. Otherwise, the module key is the top-level directory.
    ///
    /// Example (defaults shown):
    ///   --module-roots crates,packages
    #[arg(long, value_delimiter = ',')]
    pub module_roots: Option<Vec<String>>,

    /// How many path segments to include for module roots.
    ///
    /// Example:
    ///   crates/foo/src/lib.rs  (depth=2) => crates/foo
    ///   crates/foo/src/lib.rs  (depth=1) => crates
    #[arg(long)]
    pub module_depth: Option<usize>,

    /// Whether to include embedded languages (tokei "children" / blobs) in module totals.
    #[arg(long, value_enum)]
    pub children: Option<ChildIncludeMode>,
}

#[derive(Args, Debug, Clone)]
pub struct CliExportArgs {
    /// Paths to scan (directories, files, or globs). Defaults to "."
    #[arg(value_name = "PATH")]
    pub paths: Option<Vec<PathBuf>>,

    /// Output format.
    #[arg(long, value_enum)]
    pub format: Option<ExportFormat>,

    /// Write output to this file instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,

    /// Module roots (see `tokmd module`).
    #[arg(long, value_delimiter = ',')]
    pub module_roots: Option<Vec<String>>,

    /// Module depth (see `tokmd module`).
    #[arg(long)]
    pub module_depth: Option<usize>,

    /// Whether to include embedded languages (tokei "children" / blobs).
    #[arg(long, value_enum)]
    pub children: Option<ChildIncludeMode>,

    /// Drop rows with fewer than N code lines.
    #[arg(long)]
    pub min_code: Option<usize>,

    /// Stop after emitting N rows (0 = unlimited).
    #[arg(long)]
    pub max_rows: Option<usize>,

    /// Include a meta record (JSON / JSONL only). Enabled by default.
    #[arg(long, action = clap::ArgAction::Set)]
    pub meta: Option<bool>,

    /// Redact paths (and optionally module names) for safer copy/paste into LLMs.
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

    /// Analysis preset to run.
    #[arg(long, value_enum)]
    pub preset: Option<AnalysisPreset>,

    /// Output format.
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

    /// Import graph granularity.
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
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableFormat {
    /// Markdown table (great for pasting into ChatGPT).
    Md,
    /// Tab-separated values (good for piping to other tools).
    Tsv,
    /// JSON (compact).
    Json,
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
pub enum ExportFormat {
    /// CSV with a header row.
    Csv,
    /// One JSON object per line.
    Jsonl,
    /// A single JSON array.
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigMode {
    /// Read `tokei.toml` / `.tokeirc` if present.
    #[default]
    Auto,
    /// Ignore config files.
    None,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildrenMode {
    /// Merge embedded content into the parent language totals.
    Collapse,
    /// Show embedded languages as separate "(embedded)" rows.
    Separate,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildIncludeMode {
    /// Include embedded languages as separate contributions.
    Separate,
    /// Ignore embedded languages.
    ParentsOnly,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RedactMode {
    /// Do not redact.
    None,
    /// Redact file paths.
    Paths,
    /// Redact file paths and module names.
    All,
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
