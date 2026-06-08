//! Module summary command parser types.
//!
//! This module owns the clap contract for `tokmd module` while the parent parser
//! module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::Args;

use super::{ChildIncludeMode, TableFormat};

/// Command line arguments for `tokmd module`.
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use tokmd::cli::{Cli, Commands, CliModuleArgs, TableFormat};
///
/// let args = Cli::try_parse_from(["tokmd", "module", "--format", "md", "--top", "5"]).unwrap();
/// if let Some(Commands::Module(module_args)) = args.command {
///     assert_eq!(module_args.top, Some(5));
///     assert!(matches!(module_args.format, Some(TableFormat::Md)));
/// }
/// ```
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
