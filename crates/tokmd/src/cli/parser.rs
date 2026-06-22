//! # tokmd::cli
//!
//! **Tier 5 (CLI parsing and configuration)**
//!
//! This module defines CLI arguments and configuration file structures.
//!
//! ## What belongs here
//! * Clap `Parser`, `Args`, `Subcommand` structs
//! * Configuration file struct definitions (Serde)
//! * Default values and enums
//!
//! ## What does NOT belong here
//! * Business logic
//! * I/O operations (except config file parsing)
//! * Higher-tier crate dependencies
//!
pub use crate::tool_schema::ToolSchemaFormat;
use clap::Parser;

mod analysis;
mod badge;
mod check_ignore;
mod cockpit;
mod commands;
mod completions;
mod context;
mod diff;
mod evidence_packet;
mod export;
mod gate;
mod global;
mod init;
mod lang;
mod module;
mod packet;
mod run;
mod sensor;
#[cfg(feature = "ast")]
mod syntax;
mod tools;
mod validate;
mod value_enums;

pub use analysis::{
    AnalysisPreset, CliAnalyzeArgs, EffortLayer, EffortModelKind, ImportGranularity, NearDupScope,
};
pub use badge::{BadgeArgs, BadgeMetric};
pub use check_ignore::CliCheckIgnoreArgs;
pub use cockpit::{BaselineArgs, CockpitArgs, CockpitFormat, DiffRangeMode};
pub use commands::Commands;
pub use completions::{CompletionsArgs, Shell};
pub use context::{
    CliContextArgs, ContextOutput, ContextStrategy, HandoffArgs, HandoffPreset, ValueMetric,
};
pub use diff::{ColorMode, DiffArgs, DiffFormat};
pub use evidence_packet::EvidencePacketArgs;
pub use export::CliExportArgs;
pub use gate::{CliGateArgs, GateFormat};
pub use global::GlobalArgs;
pub use init::{InitArgs, InitProfile};
pub use lang::CliLangArgs;
pub use module::CliModuleArgs;
pub use packet::{
    DEFAULT_PACKET_CONTEXT_BUDGET, DEFAULT_PACKET_DIR, PacketArgs, PacketCommand,
    PacketGenerateArgs,
};
pub use run::RunArgs;
pub use sensor::{SensorArgs, SensorFormat};
#[cfg(feature = "ast")]
pub use syntax::SyntaxArgs;
pub use tools::ToolsArgs;
pub use value_enums::{
    AnalysisFormat, ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode,
    TableFormat,
};

/// tokmd — code awareness for AI contexts
///
/// A small, chat-friendly wrapper around tokei for extracting, summarizing, and shaping code telemetry.
/// Run `tokmd` in any directory to get a high-level summary of the code.
/// Use `tokmd [COMMAND] --help` for detailed help.
///
/// Default mode (no subcommand) prints a language summary.
///
/// # Example
///
/// ```rust
/// use clap::Parser;
/// use tokmd::cli::{Cli, Commands};
///
/// // We use `try_parse_from` to avoid `exit()` in testing when args are invalid or `-h`/`--help` is passed.
/// let args = Cli::try_parse_from(["tokmd", "lang", "--top", "5"]).expect("valid arguments");
/// assert!(matches!(args.command, Some(Commands::Lang(_))));
/// ```
#[derive(Parser, Debug)]
#[command(name = "tokmd", version, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Default options for the implicit `lang` mode (when no subcommand is provided).
    #[command(flatten)]
    pub lang: CliLangArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Configuration profile to use (e.g., "llm_safe", "ci").
    #[arg(long, visible_alias = "view", global = true)]
    pub profile: Option<String>,
}

// =============================================================================
// TOML Configuration File Structures (re-exported from tokmd-settings)
// =============================================================================

pub use tokmd_settings::{
    AnalyzeConfig, BadgeConfig, ContextConfig, ExportConfig, GateConfig, GateRule, ModuleConfig,
    Profile, RatchetRuleConfig, ScanConfig, TomlConfig, TomlResult, UserConfig, ViewProfile,
};

#[cfg(test)]
mod tests {
    use super::*;

    // ── Default impls ─────────────────────────────────────────────────
    #[test]
    fn user_config_default_is_empty() {
        let c = UserConfig::default();
        assert!(c.profiles.is_empty());
        assert!(c.repos.is_empty());
    }

    #[test]
    fn profile_default_all_none() {
        let p = Profile::default();
        assert!(p.format.is_none());
        assert!(p.top.is_none());
        assert!(p.files.is_none());
        assert!(p.module_roots.is_none());
        assert!(p.module_depth.is_none());
        assert!(p.min_code.is_none());
        assert!(p.max_rows.is_none());
        assert!(p.redact.is_none());
        assert!(p.meta.is_none());
        assert!(p.children.is_none());
    }

    #[test]
    fn cli_lang_args_default() {
        let a = CliLangArgs::default();
        assert!(a.paths.is_none());
        assert!(a.format.is_none());
        assert!(a.top.is_none());
        assert!(!a.files);
        assert!(a.children.is_none());
    }

    // ── Enum serde roundtrips ─────────────────────────────────────────
    #[test]
    fn context_strategy_default_is_greedy() {
        assert_eq!(ContextStrategy::default(), ContextStrategy::Greedy);
    }

    #[test]
    fn value_metric_default_is_code() {
        assert_eq!(ValueMetric::default(), ValueMetric::Code);
    }

    #[test]
    fn context_output_default_is_list() {
        assert_eq!(ContextOutput::default(), ContextOutput::List);
    }

    #[test]
    fn cockpit_format_default_is_json() {
        assert_eq!(CockpitFormat::default(), CockpitFormat::Json);
    }

    #[test]
    fn handoff_preset_default_is_risk() {
        assert_eq!(HandoffPreset::default(), HandoffPreset::Risk);
    }

    #[test]
    fn diff_range_mode_default_is_two_dot() {
        assert_eq!(DiffRangeMode::default(), DiffRangeMode::TwoDot);
    }

    // ── Serde naming ──────────────────────────────────────────────────
    #[test]
    fn context_strategy_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&ContextStrategy::Greedy).unwrap(),
            "\"greedy\""
        );
        assert_eq!(
            serde_json::to_string(&ContextStrategy::Spread).unwrap(),
            "\"spread\""
        );
    }

    #[test]
    fn value_metric_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&ValueMetric::Hotspot).unwrap(),
            "\"hotspot\""
        );
    }

    // ── UserConfig serde roundtrip ────────────────────────────────────
    #[test]
    fn user_config_serde_roundtrip() {
        let mut c = UserConfig::default();
        c.profiles.insert(
            "llm_safe".into(),
            Profile {
                format: Some("json".into()),
                top: Some(10),
                redact: Some(tokmd_types::RedactMode::All),
                ..Profile::default()
            },
        );
        c.repos.insert("owner/repo".into(), "llm_safe".into());

        let json = serde_json::to_string(&c).unwrap();
        let back: UserConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.profiles.len(), 1);
        assert_eq!(back.repos.len(), 1);
        assert_eq!(back.profiles["llm_safe"].top, Some(10));
    }
}
