//! Interactive CLI utilities.
//!
//! This module provides interactive prompts and wizards for the CLI.

pub mod tty;
pub mod wizard;

pub use tty::should_be_interactive;
