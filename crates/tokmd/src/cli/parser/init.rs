//! Init command parser types.
//!
//! This module owns the clap contract for `tokmd init` while the parent parser
//! module keeps the top-level command dispatch shape.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone)]
#[command(
    after_help = "Examples:\n  tokmd init --template rust\n  tokmd init --non-interactive --template node --force"
)]
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
pub enum InitProfile {
    Default,
    Rust,
    Node,
    Mono,
    Python,
    Go,
    Cpp,
}
