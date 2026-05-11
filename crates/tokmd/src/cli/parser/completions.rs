//! Completions command parser types.
//!
//! This module owns the clap contract for `tokmd completions` while the parent
//! parser module keeps the top-level command dispatch shape.

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

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
