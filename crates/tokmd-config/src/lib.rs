//! # tokmd-config (compatibility facade)
//!
//! Historically `tokmd-config` hosted CLI parser definitions.
//! The parser and config schema now live in `tokmd` with pure settings in
//! `tokmd-settings`; this crate remains as a compatibility shim.

pub use tokmd::cli::*;
