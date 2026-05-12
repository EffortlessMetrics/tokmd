//! # tokmd-settings
//!
//! **Tier 0 (Pure Settings)**
//!
//! Clap-free settings types for the scan and format layers.
//! These types mirror CLI arguments without Clap dependencies,
//! making them suitable for FFI boundaries and library consumers.
//!
//! ## What belongs here
//! * Pure data types with Serde derive
//! * Scan, language, module, export, analyze, diff settings
//! * Default values and conversions
//!
//! ## What does NOT belong here
//! * Clap parsing (use `tokmd::cli`)
//! * I/O operations
//! * Business logic

mod commands;
mod config;
mod profile;
mod scan;

pub use commands::{
    AnalyzeSettings, CockpitSettings, DiffSettings, ExportSettings, LangSettings, ModuleSettings,
};
pub use config::{
    AnalyzeConfig, BadgeConfig, ContextConfig, ExportConfig, GateConfig, GateRule, ModuleConfig,
    RatchetRuleConfig, ScanConfig, TomlConfig, ViewProfile,
};
pub use profile::{Profile, UserConfig};
pub use scan::{ScanOptions, ScanSettings};

// Re-export types from tokmd_types for convenience.
pub use tokmd_types::{ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode};

/// Result type alias for TOML parsing errors.
pub type TomlResult<T> = Result<T, toml::de::Error>;

#[cfg(test)]
mod tests;
