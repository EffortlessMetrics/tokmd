//! # tokmd::cli
//!
//! **Tier 5 (CLI parsing and configuration)**
//!
//! This module defines CLI arguments and configuration file structures.
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
use std::path::PathBuf;

pub use crate::tool_schema::ToolSchemaFormat;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

mod analysis;
mod badge;
mod check_ignore;
mod cockpit;
mod completions;
mod context;
mod diff;
mod gate;
mod init;
mod lang;
mod run;
mod sensor;
mod tools;

pub use analysis::{
    AnalysisPreset, CliAnalyzeArgs, EffortLayer, EffortModelKind, ImportGranularity, NearDupScope,
};
pub use badge::{BadgeArgs, BadgeMetric};
pub use check_ignore::CliCheckIgnoreArgs;
pub use cockpit::{BaselineArgs, CockpitArgs, CockpitFormat, DiffRangeMode};
pub use completions::{CompletionsArgs, Shell};
pub use context::{
    CliContextArgs, ContextOutput, ContextStrategy, HandoffArgs, HandoffPreset, ValueMetric,
};
pub use diff::{ColorMode, DiffArgs, DiffFormat};
pub use gate::{CliGateArgs, GateFormat};
pub use init::{InitArgs, InitProfile};
pub use lang::CliLangArgs;
pub use run::RunArgs;
pub use sensor::{SensorArgs, SensorFormat};
pub use tools::ToolsArgs;

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

impl From<TableFormat> for tokmd_types::TableFormat {
    fn from(value: TableFormat) -> Self {
        match value {
            TableFormat::Md => Self::Md,
            TableFormat::Tsv => Self::Tsv,
            TableFormat::Json => Self::Json,
        }
    }
}

impl From<tokmd_types::TableFormat> for TableFormat {
    fn from(value: tokmd_types::TableFormat) -> Self {
        match value {
            tokmd_types::TableFormat::Md => Self::Md,
            tokmd_types::TableFormat::Tsv => Self::Tsv,
            tokmd_types::TableFormat::Json => Self::Json,
        }
    }
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
    /// CycloneDX 1.6 JSON SBOM format.
    Cyclonedx,
}

impl From<ExportFormat> for tokmd_types::ExportFormat {
    fn from(value: ExportFormat) -> Self {
        match value {
            ExportFormat::Csv => Self::Csv,
            ExportFormat::Jsonl => Self::Jsonl,
            ExportFormat::Json => Self::Json,
            ExportFormat::Cyclonedx => Self::Cyclonedx,
        }
    }
}

impl From<tokmd_types::ExportFormat> for ExportFormat {
    fn from(value: tokmd_types::ExportFormat) -> Self {
        match value {
            tokmd_types::ExportFormat::Csv => Self::Csv,
            tokmd_types::ExportFormat::Jsonl => Self::Jsonl,
            tokmd_types::ExportFormat::Json => Self::Json,
            tokmd_types::ExportFormat::Cyclonedx => Self::Cyclonedx,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigMode {
    /// Read scan config files (`tokei.toml` / `.tokeirc`) if present.
    #[default]
    Auto,
    /// Ignore config files.
    None,
}

impl From<ConfigMode> for tokmd_types::ConfigMode {
    fn from(value: ConfigMode) -> Self {
        match value {
            ConfigMode::Auto => Self::Auto,
            ConfigMode::None => Self::None,
        }
    }
}

impl From<tokmd_types::ConfigMode> for ConfigMode {
    fn from(value: tokmd_types::ConfigMode) -> Self {
        match value {
            tokmd_types::ConfigMode::Auto => Self::Auto,
            tokmd_types::ConfigMode::None => Self::None,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildrenMode {
    /// Merge embedded content into the parent language totals.
    Collapse,
    /// Show embedded languages as separate "(embedded)" rows.
    Separate,
}

impl From<ChildrenMode> for tokmd_types::ChildrenMode {
    fn from(value: ChildrenMode) -> Self {
        match value {
            ChildrenMode::Collapse => Self::Collapse,
            ChildrenMode::Separate => Self::Separate,
        }
    }
}

impl From<tokmd_types::ChildrenMode> for ChildrenMode {
    fn from(value: tokmd_types::ChildrenMode) -> Self {
        match value {
            tokmd_types::ChildrenMode::Collapse => Self::Collapse,
            tokmd_types::ChildrenMode::Separate => Self::Separate,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildIncludeMode {
    /// Include embedded languages as separate contributions.
    Separate,
    /// Ignore embedded languages.
    ParentsOnly,
}

impl From<ChildIncludeMode> for tokmd_types::ChildIncludeMode {
    fn from(value: ChildIncludeMode) -> Self {
        match value {
            ChildIncludeMode::Separate => Self::Separate,
            ChildIncludeMode::ParentsOnly => Self::ParentsOnly,
        }
    }
}

impl From<tokmd_types::ChildIncludeMode> for ChildIncludeMode {
    fn from(value: tokmd_types::ChildIncludeMode) -> Self {
        match value {
            tokmd_types::ChildIncludeMode::Separate => Self::Separate,
            tokmd_types::ChildIncludeMode::ParentsOnly => Self::ParentsOnly,
        }
    }
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

impl From<RedactMode> for tokmd_types::RedactMode {
    fn from(value: RedactMode) -> Self {
        match value {
            RedactMode::None => Self::None,
            RedactMode::Paths => Self::Paths,
            RedactMode::All => Self::All,
        }
    }
}

impl From<tokmd_types::RedactMode> for RedactMode {
    fn from(value: tokmd_types::RedactMode) -> Self {
        match value {
            tokmd_types::RedactMode::None => Self::None,
            tokmd_types::RedactMode::Paths => Self::Paths,
            tokmd_types::RedactMode::All => Self::All,
        }
    }
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

impl From<AnalysisFormat> for tokmd_types::AnalysisFormat {
    fn from(value: AnalysisFormat) -> Self {
        match value {
            AnalysisFormat::Md => Self::Md,
            AnalysisFormat::Json => Self::Json,
            AnalysisFormat::Jsonld => Self::Jsonld,
            AnalysisFormat::Xml => Self::Xml,
            AnalysisFormat::Svg => Self::Svg,
            AnalysisFormat::Mermaid => Self::Mermaid,
            AnalysisFormat::Obj => Self::Obj,
            AnalysisFormat::Midi => Self::Midi,
            AnalysisFormat::Tree => Self::Tree,
            AnalysisFormat::Html => Self::Html,
        }
    }
}

impl From<tokmd_types::AnalysisFormat> for AnalysisFormat {
    fn from(value: tokmd_types::AnalysisFormat) -> Self {
        match value {
            tokmd_types::AnalysisFormat::Md => Self::Md,
            tokmd_types::AnalysisFormat::Json => Self::Json,
            tokmd_types::AnalysisFormat::Jsonld => Self::Jsonld,
            tokmd_types::AnalysisFormat::Xml => Self::Xml,
            tokmd_types::AnalysisFormat::Svg => Self::Svg,
            tokmd_types::AnalysisFormat::Mermaid => Self::Mermaid,
            tokmd_types::AnalysisFormat::Obj => Self::Obj,
            tokmd_types::AnalysisFormat::Midi => Self::Midi,
            tokmd_types::AnalysisFormat::Tree => Self::Tree,
            tokmd_types::AnalysisFormat::Html => Self::Html,
        }
    }
}

/// tokmd — code awareness for AI contexts
///
/// A small, chat-friendly wrapper around tokei for extracting, summarizing, and shaping code telemetry.
/// Run `tokmd` in any directory to get a high-level summary of the code.
/// Use `tokmd [COMMAND] --help` for detailed help.
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

    /// Whether to load scan config files (`tokei.toml` / `.tokeirc`).
    #[arg(long, value_enum, value_name = "MODE", default_value_t = ConfigMode::Auto)]
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
    #[command(visible_alias = "analyse")]
    Analyze(CliAnalyzeArgs),

    /// Render a simple SVG badge for a metric.
    Badge(BadgeArgs),

    /// Write a `.tokeignore` template to the target directory.
    Init(InitArgs),

    /// Generate shell completions.
    #[command(visible_alias = "completion")]
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
    #[arg(long, visible_alias = "depth")]
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
    #[arg(long, visible_alias = "depth")]
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

// =============================================================================
// TOML Configuration File Structures (re-exported from tokmd-settings)
// =============================================================================

pub use tokmd_settings::{
    AnalyzeConfig, BadgeConfig, ContextConfig, ExportConfig, GateConfig, GateRule, ModuleConfig,
    Profile, RatchetRuleConfig, ScanConfig, TomlConfig, TomlResult, UserConfig, ViewProfile,
};

// ============================================================
// Conversions between CLI GlobalArgs and Tier-0 ScanOptions
// ============================================================

impl From<&GlobalArgs> for tokmd_settings::ScanOptions {
    fn from(g: &GlobalArgs) -> Self {
        Self {
            excluded: g.excluded.clone(),
            config: g.config.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Default impls ─────────────────────────────────────────────────
    #[test]
    fn user_config_default_is_empty() {
        let c = UserConfig::default();
        assert!(c.profiles.is_empty());
        assert!(c.repos.is_empty());
    }

    #[test]
    fn profile_default_all_none() {
        let p = Profile::default();
        assert!(p.format.is_none());
        assert!(p.top.is_none());
        assert!(p.files.is_none());
        assert!(p.module_roots.is_none());
        assert!(p.module_depth.is_none());
        assert!(p.min_code.is_none());
        assert!(p.max_rows.is_none());
        assert!(p.redact.is_none());
        assert!(p.meta.is_none());
        assert!(p.children.is_none());
    }

    #[test]
    fn global_args_default() {
        let g = GlobalArgs::default();
        assert!(g.excluded.is_empty());
        assert_eq!(g.config, ConfigMode::Auto);
        assert!(!g.hidden);
        assert!(!g.no_ignore);
        assert_eq!(g.verbose, 0);
    }

    #[test]
    fn cli_lang_args_default() {
        let a = CliLangArgs::default();
        assert!(a.paths.is_none());
        assert!(a.format.is_none());
        assert!(a.top.is_none());
        assert!(!a.files);
        assert!(a.children.is_none());
    }

    #[test]
    fn depth_visible_alias_sets_module_depth_for_module_like_commands() {
        let cli = Cli::try_parse_from(["tokmd", "module", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Module(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }

        let cli = Cli::try_parse_from(["tokmd", "export", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Export(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }

        let cli = Cli::try_parse_from(["tokmd", "context", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Context(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }

        let cli = Cli::try_parse_from(["tokmd", "handoff", "--depth", "3"]).unwrap();
        match cli.command.unwrap() {
            Commands::Handoff(args) => assert_eq!(args.module_depth, Some(3)),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    // ── Enum serde roundtrips ─────────────────────────────────────────
    #[test]
    fn context_strategy_default_is_greedy() {
        assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
    }

    #[test]
    fn value_metric_default_is_code() {
        assert_eq!(ValueMetric::default(), ValueMetric::Code);
    }

    #[test]
    fn context_output_default_is_list() {
        assert_eq!(ContextOutput::default(), ContextOutput::List);
    }

    #[test]
    fn cockpit_format_default_is_json() {
        assert_eq!(CockpitFormat::default(), CockpitFormat::Json);
    }

    #[test]
    fn handoff_preset_default_is_risk() {
        assert_eq!(HandoffPreset::default(), HandoffPreset::Risk);
    }

    #[test]
    fn diff_range_mode_default_is_two_dot() {
        assert_eq!(DiffRangeMode::default(), DiffRangeMode::TwoDot);
    }

    // ── Serde naming ──────────────────────────────────────────────────
    #[test]
    fn context_strategy_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&ContextStrategy::Greedy).unwrap(),
            "\"greedy\""
        );
        assert_eq!(
            serde_json::to_string(&ContextStrategy::Spread).unwrap(),
            "\"spread\""
        );
    }

    #[test]
    fn value_metric_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&ValueMetric::Hotspot).unwrap(),
            "\"hotspot\""
        );
    }

    // ── UserConfig serde roundtrip ────────────────────────────────────
    #[test]
    fn user_config_serde_roundtrip() {
        let mut c = UserConfig::default();
        c.profiles.insert(
            "llm_safe".into(),
            Profile {
                format: Some("json".into()),
                top: Some(10),
                redact: Some(tokmd_types::RedactMode::All),
                ..Profile::default()
            },
        );
        c.repos.insert("owner/repo".into(), "llm_safe".into());

        let json = serde_json::to_string(&c).unwrap();
        let back: UserConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.profiles.len(), 1);
        assert_eq!(back.repos.len(), 1);
        assert_eq!(back.profiles["llm_safe"].top, Some(10));
    }

    // ── GlobalArgs → ScanOptions conversion ───────────────────────────
    #[test]
    fn global_args_to_scan_options() {
        let g = GlobalArgs {
            excluded: vec!["target".into()],
            config: ConfigMode::None,
            hidden: true,
            no_ignore: true,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: true,
            verbose: 0,
            no_progress: false,
        };
        let opts: tokmd_settings::ScanOptions = (&g).into();
        assert_eq!(opts.excluded, vec!["target"]);
        assert_eq!(opts.config, tokmd_types::ConfigMode::None);
        assert!(opts.hidden);
        assert!(opts.no_ignore);
        assert!(opts.treat_doc_strings_as_comments);
    }

    #[test]
    fn global_args_owned_to_scan_options() {
        let g = GlobalArgs {
            excluded: vec!["vendor".into()],
            config: ConfigMode::Auto,
            hidden: false,
            ..GlobalArgs::default()
        };
        let opts: tokmd_settings::ScanOptions = g.into();
        assert_eq!(opts.excluded, vec!["vendor"]);
        assert!(!opts.hidden);
    }
}
