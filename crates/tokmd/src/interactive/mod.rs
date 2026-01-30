//! Interactive CLI utilities.
//!
//! This module provides interactive prompts and wizards for the CLI.
//! Requires the `ui` feature to be enabled for full functionality.

#[cfg(feature = "ui")]
pub mod tty;

#[cfg(feature = "ui")]
pub mod wizard;
