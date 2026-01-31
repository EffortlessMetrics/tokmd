//! # tokmd
//!
//! **Tier 5 (CLI Binary)**
//!
//! This is the entry point for the `tokmd` command-line application.
//! It orchestrates all other crates to perform the requested actions.
//!
//! ## What belongs here
//! * Command line argument parsing
//! * Configuration loading and profile resolution
//! * Command dispatch to appropriate handlers
//! * Error handling and exit codes
//!
//! ## What does NOT belong here
//! * Business logic (belongs in lower-tier crates)
//! * Duplicated functionality from other crates
//! * Complex computation (delegate to appropriate crates)
//!
//! This crate should contain minimal business logic.

mod analysis_utils;
mod badge;
mod commands;
mod config;
mod context_pack;
mod export_bundle;
mod git_scoring;
#[cfg(feature = "ui")]
mod interactive;
mod progress;
mod tools_schema;

use anyhow::Result;
use clap::Parser;
use tokmd_config as cli;

pub use config::{
    ConfigContext, ResolvedConfig, resolve_config, resolve_export, resolve_export_with_config,
    resolve_lang, resolve_lang_with_config, resolve_module, resolve_module_with_config,
    resolve_profile,
};

pub fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    let config_ctx = config::load_config();
    let profile_name = config::get_profile_name(cli.profile.as_ref());
    let resolved = config::resolve_config(&config_ctx, profile_name.as_deref());
    commands::dispatch(cli, &resolved)
}
