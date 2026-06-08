//! File export command parser types.
//!
//! This module owns the clap contract for `tokmd export` while the parent parser
//! module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::Args;

use super::{ChildIncludeMode, ExportFormat, RedactMode};

/// Command line arguments for `tokmd export`.
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use tokmd::cli::{Cli, Commands, CliExportArgs, ExportFormat};
///
/// let args = Cli::try_parse_from(["tokmd", "export", "--format", "jsonl", "--min-code", "10"]).unwrap();
/// if let Some(Commands::Export(export_args)) = args.command {
///     assert_eq!(export_args.min_code, Some(10));
///     assert!(matches!(export_args.format, Some(ExportFormat::Jsonl)));
/// }
/// ```
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
