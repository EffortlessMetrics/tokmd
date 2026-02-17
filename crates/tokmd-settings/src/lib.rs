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
//! * Clap parsing (use tokmd-config)
//! * I/O operations
//! * Business logic

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

// Re-export types from tokmd_types for convenience
pub use tokmd_types::{ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode};

/// Scan options shared by all commands that invoke the scanner.
///
/// This mirrors the scan-relevant fields of `GlobalArgs` without any
/// UI-specific fields (`verbose`, `no_progress`). Lower-tier crates
/// (scan, format, model) depend on this instead of `tokmd-config`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanOptions {
    /// Glob patterns to exclude.
    #[serde(default)]
    pub excluded: Vec<String>,

    /// Whether to load `tokei.toml` / `.tokeirc`.
    #[serde(default)]
    pub config: ConfigMode,

    /// Count hidden files and directories.
    #[serde(default)]
    pub hidden: bool,

    /// Don't respect ignore files (.gitignore, .ignore, etc.).
    #[serde(default)]
    pub no_ignore: bool,

    /// Don't respect ignore files in parent directories.
    #[serde(default)]
    pub no_ignore_parent: bool,

    /// Don't respect .ignore and .tokeignore files.
    #[serde(default)]
    pub no_ignore_dot: bool,

    /// Don't respect VCS ignore files (.gitignore, .hgignore, etc.).
    #[serde(default)]
    pub no_ignore_vcs: bool,

    /// Treat doc strings as comments.
    #[serde(default)]
    pub treat_doc_strings_as_comments: bool,
}

/// Global scan settings shared by all operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanSettings {
    /// Paths to scan (defaults to `["."]`).
    #[serde(default)]
    pub paths: Vec<String>,

    /// Scan options (excludes, ignore flags, etc.).
    #[serde(flatten)]
    pub options: ScanOptions,
}

impl ScanSettings {
    /// Create settings for scanning the current directory with defaults.
    pub fn current_dir() -> Self {
        Self {
            paths: vec![".".to_string()],
            ..Default::default()
        }
    }

    /// Create settings for scanning specific paths.
    pub fn for_paths(paths: Vec<String>) -> Self {
        Self {
            paths,
            ..Default::default()
        }
    }
}

/// Settings for language summary (`tokmd lang`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSettings {
    /// Show only the top N rows (0 = all).
    #[serde(default)]
    pub top: usize,

    /// Include file counts and average lines per file.
    #[serde(default)]
    pub files: bool,

    /// How to handle embedded languages.
    #[serde(default = "default_children_mode")]
    pub children: ChildrenMode,

    /// Redaction mode for output.
    #[serde(default)]
    pub redact: Option<RedactMode>,
}

impl Default for LangSettings {
    fn default() -> Self {
        Self {
            top: 0,
            files: false,
            children: ChildrenMode::Collapse,
            redact: None,
        }
    }
}

fn default_children_mode() -> ChildrenMode {
    ChildrenMode::Collapse
}

/// Settings for module summary (`tokmd module`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSettings {
    /// Show only the top N modules (0 = all).
    #[serde(default)]
    pub top: usize,

    /// Top-level directories as "module roots".
    #[serde(default = "default_module_roots")]
    pub module_roots: Vec<String>,

    /// Path segments to include for module roots.
    #[serde(default = "default_module_depth")]
    pub module_depth: usize,

    /// How to handle embedded languages.
    #[serde(default = "default_child_include_mode")]
    pub children: ChildIncludeMode,

    /// Redaction mode for output.
    #[serde(default)]
    pub redact: Option<RedactMode>,
}

fn default_module_roots() -> Vec<String> {
    vec!["crates".to_string(), "packages".to_string()]
}

fn default_module_depth() -> usize {
    2
}

fn default_child_include_mode() -> ChildIncludeMode {
    ChildIncludeMode::Separate
}

impl Default for ModuleSettings {
    fn default() -> Self {
        Self {
            top: 0,
            module_roots: default_module_roots(),
            module_depth: default_module_depth(),
            children: default_child_include_mode(),
            redact: None,
        }
    }
}

/// Settings for file-level export (`tokmd export`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    /// Output format.
    #[serde(default = "default_export_format")]
    pub format: ExportFormat,

    /// Module roots (see `ModuleSettings`).
    #[serde(default = "default_module_roots")]
    pub module_roots: Vec<String>,

    /// Module depth (see `ModuleSettings`).
    #[serde(default = "default_module_depth")]
    pub module_depth: usize,

    /// How to handle embedded languages.
    #[serde(default = "default_child_include_mode")]
    pub children: ChildIncludeMode,

    /// Drop rows with fewer than N code lines.
    #[serde(default)]
    pub min_code: usize,

    /// Stop after emitting N rows (0 = unlimited).
    #[serde(default)]
    pub max_rows: usize,

    /// Redaction mode.
    #[serde(default = "default_redact_mode")]
    pub redact: RedactMode,

    /// Include a meta record.
    #[serde(default = "default_meta")]
    pub meta: bool,

    /// Strip this prefix from paths.
    #[serde(default)]
    pub strip_prefix: Option<String>,
}

fn default_redact_mode() -> RedactMode {
    RedactMode::None
}

fn default_export_format() -> ExportFormat {
    ExportFormat::Jsonl
}

fn default_meta() -> bool {
    true
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: default_export_format(),
            module_roots: default_module_roots(),
            module_depth: default_module_depth(),
            children: default_child_include_mode(),
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            meta: true,
            strip_prefix: None,
        }
    }
}

/// Settings for analysis (`tokmd analyze`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeSettings {
    /// Analysis preset to run.
    #[serde(default = "default_preset")]
    pub preset: String,

    /// Context window size (tokens) for utilization bars.
    #[serde(default)]
    pub window: Option<usize>,

    /// Force-enable git-based metrics.
    #[serde(default)]
    pub git: Option<bool>,

    /// Limit files walked for asset/deps/content scans.
    #[serde(default)]
    pub max_files: Option<usize>,

    /// Limit total bytes read during content scans.
    #[serde(default)]
    pub max_bytes: Option<u64>,

    /// Limit bytes per file during content scans.
    #[serde(default)]
    pub max_file_bytes: Option<u64>,

    /// Limit commits scanned for git metrics.
    #[serde(default)]
    pub max_commits: Option<usize>,

    /// Limit files per commit for git metrics.
    #[serde(default)]
    pub max_commit_files: Option<usize>,

    /// Import graph granularity.
    #[serde(default = "default_granularity")]
    pub granularity: String,
}

fn default_preset() -> String {
    "receipt".to_string()
}

fn default_granularity() -> String {
    "module".to_string()
}

impl Default for AnalyzeSettings {
    fn default() -> Self {
        Self {
            preset: default_preset(),
            window: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            granularity: default_granularity(),
        }
    }
}

/// Settings for diff comparison (`tokmd diff`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSettings {
    /// Base reference to compare from.
    pub from: String,

    /// Target reference to compare to.
    pub to: String,
}

// =============================================================================
// TOML Configuration File Structures
// =============================================================================

/// Root TOML configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TomlConfig {
    /// Scan settings (applies to all commands).
    pub scan: ScanConfig,

    /// Module command settings.
    pub module: ModuleConfig,

    /// Export command settings.
    pub export: ExportConfig,

    /// Analyze command settings.
    pub analyze: AnalyzeConfig,

    /// Context command settings.
    pub context: ContextConfig,

    /// Badge command settings.
    pub badge: BadgeConfig,

    /// Gate command settings.
    pub gate: GateConfig,

    /// Named view profiles (e.g., [view.llm], [view.ci]).
    #[serde(default)]
    pub view: HashMap<String, ViewProfile>,
}

/// Scan settings shared by all commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanConfig {
    /// Paths to scan (default: ["."])
    pub paths: Option<Vec<String>>,

    /// Glob patterns to exclude.
    pub exclude: Option<Vec<String>>,

    /// Include hidden files and directories.
    pub hidden: Option<bool>,

    /// Config file strategy for tokei: "auto" or "none".
    pub config: Option<String>,

    /// Disable all ignore files.
    pub no_ignore: Option<bool>,

    /// Disable parent directory ignore file traversal.
    pub no_ignore_parent: Option<bool>,

    /// Disable .ignore/.tokeignore files.
    pub no_ignore_dot: Option<bool>,

    /// Disable .gitignore files.
    pub no_ignore_vcs: Option<bool>,

    /// Treat doc comments as comments instead of code.
    pub doc_comments: Option<bool>,
}

/// Module command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ModuleConfig {
    /// Root directories for module grouping.
    pub roots: Option<Vec<String>>,

    /// Depth for module grouping.
    pub depth: Option<usize>,

    /// Children handling: "collapse" or "separate".
    pub children: Option<String>,
}

/// Export command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportConfig {
    /// Minimum lines of code to include.
    pub min_code: Option<usize>,

    /// Maximum rows in output.
    pub max_rows: Option<usize>,

    /// Redaction mode: "none", "paths", or "all".
    pub redact: Option<String>,

    /// Output format: "jsonl", "csv", "json", "cyclonedx".
    pub format: Option<String>,

    /// Children handling: "collapse" or "separate".
    pub children: Option<String>,
}

/// Analyze command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalyzeConfig {
    /// Analysis preset.
    pub preset: Option<String>,

    /// Context window size for utilization analysis.
    pub window: Option<usize>,

    /// Output format.
    pub format: Option<String>,

    /// Force git metrics on/off.
    pub git: Option<bool>,

    /// Max files for asset/deps/content scans.
    pub max_files: Option<usize>,

    /// Max total bytes for content scans.
    pub max_bytes: Option<u64>,

    /// Max bytes per file for content scans.
    pub max_file_bytes: Option<u64>,

    /// Max commits for git metrics.
    pub max_commits: Option<usize>,

    /// Max files per commit for git metrics.
    pub max_commit_files: Option<usize>,

    /// Import graph granularity: "module" or "file".
    pub granularity: Option<String>,
}

/// Context command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    /// Token budget with optional k/m suffix.
    pub budget: Option<String>,

    /// Packing strategy: "greedy" or "spread".
    pub strategy: Option<String>,

    /// Ranking metric: "code", "tokens", "churn", "hotspot".
    pub rank_by: Option<String>,

    /// Output mode: "list", "bundle", "json".
    pub output: Option<String>,

    /// Strip blank lines from bundle output.
    pub compress: Option<bool>,
}

/// Badge command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BadgeConfig {
    /// Default metric for badges.
    pub metric: Option<String>,
}

/// Gate command settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GateConfig {
    /// Path to policy file.
    pub policy: Option<String>,

    /// Path to baseline file for ratchet comparison.
    pub baseline: Option<String>,

    /// Analysis preset for compute-then-gate mode.
    pub preset: Option<String>,

    /// Fail fast on first error.
    pub fail_fast: Option<bool>,

    /// Inline policy rules.
    pub rules: Option<Vec<GateRule>>,

    /// Inline ratchet rules for baseline comparison.
    pub ratchet: Option<Vec<RatchetRuleConfig>>,

    /// Allow missing baseline values (treat as pass).
    pub allow_missing_baseline: Option<bool>,

    /// Allow missing current values (treat as pass).
    pub allow_missing_current: Option<bool>,
}

/// A single ratchet rule for baseline comparison (TOML configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetRuleConfig {
    /// JSON Pointer to the metric (e.g., "/complexity/avg_cyclomatic").
    pub pointer: String,

    /// Maximum allowed percentage increase from baseline.
    #[serde(default)]
    pub max_increase_pct: Option<f64>,

    /// Maximum allowed absolute value (hard ceiling).
    #[serde(default)]
    pub max_value: Option<f64>,

    /// Rule severity level: "error" (default) or "warn".
    #[serde(default)]
    pub level: Option<String>,

    /// Human-readable description of the rule.
    #[serde(default)]
    pub description: Option<String>,
}

/// A single gate policy rule (for inline TOML configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateRule {
    /// Human-readable name for the rule.
    pub name: String,

    /// JSON Pointer to the value to check (RFC 6901).
    pub pointer: String,

    /// Comparison operator.
    pub op: String,

    /// Single value for comparison.
    #[serde(default)]
    pub value: Option<serde_json::Value>,

    /// Multiple values for "in" operator.
    #[serde(default)]
    pub values: Option<Vec<serde_json::Value>>,

    /// Negate the result.
    #[serde(default)]
    pub negate: bool,

    /// Rule severity level: "error" or "warn".
    #[serde(default)]
    pub level: Option<String>,

    /// Custom failure message.
    #[serde(default)]
    pub message: Option<String>,
}

/// A named view profile that can override settings for specific use cases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewProfile {
    // Shared settings
    /// Output format.
    pub format: Option<String>,

    /// Show only top N rows.
    pub top: Option<usize>,

    // Lang settings
    /// Include file counts in lang output.
    pub files: Option<bool>,

    // Module / Export settings
    /// Module roots for grouping.
    pub module_roots: Option<Vec<String>>,

    /// Module depth for grouping.
    pub module_depth: Option<usize>,

    /// Minimum lines of code.
    pub min_code: Option<usize>,

    /// Maximum rows in output.
    pub max_rows: Option<usize>,

    /// Redaction mode.
    pub redact: Option<String>,

    /// Include metadata record.
    pub meta: Option<bool>,

    /// Children handling mode.
    pub children: Option<String>,

    // Analyze settings
    /// Analysis preset.
    pub preset: Option<String>,

    /// Context window size.
    pub window: Option<usize>,

    // Context settings
    /// Token budget.
    pub budget: Option<String>,

    /// Packing strategy.
    pub strategy: Option<String>,

    /// Ranking metric.
    pub rank_by: Option<String>,

    /// Output mode for context.
    pub output: Option<String>,

    /// Strip blank lines.
    pub compress: Option<bool>,

    // Badge settings
    /// Badge metric.
    pub metric: Option<String>,
}

impl TomlConfig {
    /// Load configuration from a TOML string.
    pub fn parse(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Load configuration from a file path.
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Result type alias for TOML parsing errors.
pub type TomlResult<T> = Result<T, toml::de::Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn scan_options_default() {
        let opts = ScanOptions::default();
        assert!(opts.excluded.is_empty());
        assert!(!opts.hidden);
        assert!(!opts.no_ignore);
    }

    #[test]
    fn scan_settings_current_dir() {
        let s = ScanSettings::current_dir();
        assert_eq!(s.paths, vec!["."]);
    }

    #[test]
    fn scan_settings_for_paths() {
        let s = ScanSettings::for_paths(vec!["src".into(), "lib".into()]);
        assert_eq!(s.paths.len(), 2);
    }

    #[test]
    fn scan_settings_flatten() {
        // Verify that ScanOptions fields are accessible through ScanSettings
        let s = ScanSettings {
            paths: vec![".".into()],
            options: ScanOptions {
                hidden: true,
                ..Default::default()
            },
        };
        assert!(s.options.hidden);
    }

    #[test]
    fn serde_roundtrip_scan_options() {
        let opts = ScanOptions {
            excluded: vec!["target".into()],
            config: ConfigMode::None,
            hidden: true,
            no_ignore: false,
            no_ignore_parent: true,
            no_ignore_dot: false,
            no_ignore_vcs: true,
            treat_doc_strings_as_comments: true,
        };
        let json = serde_json::to_string(&opts).unwrap();
        let back: ScanOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(back.excluded, opts.excluded);
        assert!(back.hidden);
        assert!(back.no_ignore_parent);
        assert!(back.no_ignore_vcs);
        assert!(back.treat_doc_strings_as_comments);
    }

    #[test]
    fn serde_roundtrip_scan_settings() {
        let s = ScanSettings {
            paths: vec!["src".into()],
            options: ScanOptions {
                excluded: vec!["*.bak".into()],
                ..Default::default()
            },
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ScanSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.paths, s.paths);
        assert_eq!(back.options.excluded, s.options.excluded);
    }

    #[test]
    fn serde_roundtrip_lang_settings() {
        let s = LangSettings {
            top: 10,
            files: true,
            children: ChildrenMode::Separate,
            redact: Some(RedactMode::Paths),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: LangSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.top, 10);
        assert!(back.files);
    }

    #[test]
    fn serde_roundtrip_export_settings() {
        let s = ExportSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: ExportSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.min_code, 0);
        assert!(back.meta);
    }

    #[test]
    fn serde_roundtrip_analyze_settings() {
        let s = AnalyzeSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.preset, "receipt");
        assert_eq!(back.granularity, "module");
    }

    #[test]
    fn serde_roundtrip_diff_settings() {
        let s = DiffSettings {
            from: "v1.0".into(),
            to: "v2.0".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: DiffSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.from, "v1.0");
        assert_eq!(back.to, "v2.0");
    }

    #[test]
    fn toml_parse_and_view_profiles() {
        let toml_str = r#"
[scan]
hidden = true

[view.llm]
format = "json"
top = 10
"#;
        let config = TomlConfig::parse(toml_str).expect("parse config");
        assert_eq!(config.scan.hidden, Some(true));
        let llm = config.view.get("llm").expect("llm profile");
        assert_eq!(llm.format.as_deref(), Some("json"));
        assert_eq!(llm.top, Some(10));
    }

    #[test]
    fn toml_from_file_roundtrip() {
        let toml_content = r#"
[module]
depth = 3
roots = ["src", "tests"]
"#;

        let mut temp_file = NamedTempFile::new().expect("temp file");
        temp_file
            .write_all(toml_content.as_bytes())
            .expect("write config");

        let config = TomlConfig::from_file(temp_file.path()).expect("load config");
        assert_eq!(config.module.depth, Some(3));
        assert_eq!(
            config.module.roots,
            Some(vec!["src".to_string(), "tests".to_string()])
        );
    }
}
