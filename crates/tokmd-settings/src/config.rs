//! TOML configuration file contracts.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

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
    pub view: BTreeMap<String, ViewProfile>,
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

    /// Effort model for estimate calculations.
    pub effort_model: Option<String>,

    /// Effort report layer.
    pub effort_layer: Option<String>,

    /// Base reference for effort delta computation.
    pub effort_base_ref: Option<String>,

    /// Head reference for effort delta computation.
    pub effort_head_ref: Option<String>,

    /// Enable Monte Carlo uncertainty for effort estimation.
    pub effort_monte_carlo: Option<bool>,

    /// Monte Carlo iterations for effort estimation.
    pub effort_mc_iterations: Option<usize>,

    /// Monte Carlo seed for effort estimation.
    pub effort_mc_seed: Option<u64>,
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
    ///
    /// # Example
    ///
    /// ```
    /// use tokmd_settings::TomlConfig;
    ///
    /// let toml_str = r#"
    /// [scan]
    /// paths = ["src", "tests"]
    /// hidden = true
    ///
    /// [export]
    /// format = "jsonl"
    /// "#;
    ///
    /// let config = TomlConfig::parse(toml_str).unwrap();
    /// assert_eq!(config.scan.paths.unwrap(), vec!["src", "tests"]);
    /// assert_eq!(config.scan.hidden, Some(true));
    /// assert_eq!(config.export.format.as_deref(), Some("jsonl"));
    /// ```
    pub fn parse(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Load configuration from a file path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use tokmd_settings::TomlConfig;
    ///
    /// let config = TomlConfig::from_file(Path::new("tokmd.toml")).unwrap();
    /// ```
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
