//! # tokmd
//!
//! **CLI Binary**
//!
//! This is the entry point for the `tokmd` command-line application.
//! It orchestrates the other crates to perform the requested actions.
//!
//! ## Responsibilities
//! * Parse command line arguments
//! * Load configuration
//! * Dispatch commands to appropriate handlers
//! * Handle errors and exit codes
//!
//! This crate should contain minimal business logic.

mod analysis_utils;
mod badge;
mod commands;
mod config;
mod export_bundle;

use anyhow::Result;
use clap::Parser;
use tokmd_config as cli;

pub use config::{resolve_export, resolve_lang, resolve_module, resolve_profile};

pub fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    let user_config = config::load_config();
    let profile = config::resolve_profile(&user_config, cli.profile.as_ref());
    commands::dispatch(cli, profile)
}
