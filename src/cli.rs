use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

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
    pub lang: LangArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    /// Paths to scan (directories, files, or globs). Defaults to "."
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<PathBuf>,

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
    #[arg(long)]
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
    Lang(LangArgs),

    /// Module summary (group by path prefixes like crates/<name> or packages/<name>).
    Module(ModuleArgs),

    /// Export a file-level dataset (CSV / JSONL / JSON).
    Export(ExportArgs),

    /// Write a `.tokeignore` template to the target directory.
    Init(InitArgs),
}

#[derive(Args, Debug, Clone)]
pub struct LangArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = TableFormat::Md)]
    pub format: TableFormat,

    /// Show only the top N rows (by code lines), plus an "Other" row if needed.
    /// Use 0 to show all rows.
    #[arg(long, default_value_t = 0)]
    pub top: usize,

    /// Include file counts and average lines per file.
    #[arg(long)]
    pub files: bool,

    /// How to handle embedded languages (tokei "children" / blobs).
    ///
    /// - collapse: merge embedded content into the parent language row.
    /// - separate: show "(embedded)" rows for child languages.
    #[arg(long, value_enum, default_value_t = ChildrenMode::Collapse)]
    pub children: ChildrenMode,
}

#[derive(Args, Debug, Clone)]
pub struct ModuleArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = TableFormat::Md)]
    pub format: TableFormat,

    /// Show only the top N modules (by code lines), plus an "Other" row if needed.
    /// Use 0 to show all rows.
    #[arg(long, default_value_t = 0)]
    pub top: usize,

    /// Treat these top-level directories as "module roots".
    ///
    /// If a file path starts with one of these roots, the module key will include
    /// `module_depth` segments. Otherwise, the module key is the top-level directory.
    ///
    /// Example (defaults shown):
    ///   --module-roots crates,packages
    #[arg(long, value_delimiter = ',', default_value = "crates,packages")]
    pub module_roots: Vec<String>,

    /// How many path segments to include for module roots.
    ///
    /// Example:
    ///   crates/foo/src/lib.rs  (depth=2) => crates/foo
    ///   crates/foo/src/lib.rs  (depth=1) => crates
    #[arg(long, default_value_t = 2)]
    pub module_depth: usize,

    /// Whether to include embedded languages (tokei "children" / blobs) in module totals.
    ///
    /// - separate: include embedded reports alongside parent reports.
    /// - parents-only: ignore embedded reports.
    #[arg(long, value_enum, default_value_t = ChildIncludeMode::Separate)]
    pub children: ChildIncludeMode,
}

#[derive(Args, Debug, Clone)]
pub struct ExportArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = ExportFormat::Jsonl)]
    pub format: ExportFormat,

    /// Write output to this file instead of stdout.
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,

    /// Module roots (see `tokmd module`).
    #[arg(long, value_delimiter = ',', default_value = "crates,packages")]
    pub module_roots: Vec<String>,

    /// Module depth (see `tokmd module`).
    #[arg(long, default_value_t = 2)]
    pub module_depth: usize,

    /// Whether to include embedded languages (tokei "children" / blobs).
    #[arg(long, value_enum, default_value_t = ChildIncludeMode::Separate)]
    pub children: ChildIncludeMode,

    /// Drop rows with fewer than N code lines.
    #[arg(long, default_value_t = 0)]
    pub min_code: usize,

    /// Stop after emitting N rows (0 = unlimited).
    #[arg(long, default_value_t = 0)]
    pub max_rows: usize,

    /// Include a meta record (JSON / JSONL only). Enabled by default.
    #[arg(long, default_value_t = true)]
    pub meta: bool,

    /// Redact paths (and optionally module names) for safer copy/paste into LLMs.
    #[arg(long, value_enum, default_value_t = RedactMode::None)]
    pub redact: RedactMode,

    /// Strip this prefix from paths before output (helps when paths are absolute).
    #[arg(long, value_name = "PATH")]
    pub strip_prefix: Option<PathBuf>,
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
    pub profile: InitProfile,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableFormat {
    /// Markdown table (great for pasting into ChatGPT).
    Md,
    /// Tab-separated values (good for piping to other tools).
    Tsv,
    /// JSON (compact).
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    /// CSV with a header row.
    Csv,
    /// One JSON object per line.
    Jsonl,
    /// A single JSON array.
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigMode {
    /// Read `tokei.toml` / `.tokeirc` if present.
    Auto,
    /// Ignore config files.
    None,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildrenMode {
    /// Merge embedded content into the parent language totals.
    Collapse,
    /// Show embedded languages as separate "(embedded)" rows.
    Separate,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildIncludeMode {
    /// Include embedded languages as separate contributions.
    Separate,
    /// Ignore embedded languages.
    ParentsOnly,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RedactMode {
    /// Do not redact.
    None,
    /// Redact file paths.
    Paths,
    /// Redact file paths and module names.
    All,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InitProfile {
    Default,
    Rust,
    Node,
    Mono,
}
