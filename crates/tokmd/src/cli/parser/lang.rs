//! Language summary command parser types.
//!
//! This module owns the clap contract for implicit/default `tokmd lang` options
//! while the parent parser module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::Args;

use super::{ChildrenMode, TableFormat};

/// Command line arguments for `tokmd lang`.
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use tokmd::cli::{Cli, Commands, CliLangArgs};
///
/// let args = Cli::try_parse_from(["tokmd", "lang", "--format", "md", "--top", "5"]).unwrap();
/// if let Some(Commands::Lang(lang_args)) = args.command {
///     assert_eq!(lang_args.top, Some(5));
/// }
/// ```
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
