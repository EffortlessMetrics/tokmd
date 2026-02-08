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

#[cfg(test)]
mod tests {
    use super::*;

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
}
