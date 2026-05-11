//! Check-ignore command parser types.
//!
//! This module owns the clap contract for `tokmd check-ignore` while the parent
//! parser module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct CliCheckIgnoreArgs {
    /// File path(s) to check.
    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<PathBuf>,

    /// Show verbose output with rule sources.
    #[arg(long, short = 'v')]
    pub verbose: bool,
}
