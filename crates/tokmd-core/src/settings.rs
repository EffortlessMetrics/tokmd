//! Pure settings types for binding-friendly API.
//!
//! These types mirror CLI arguments but without Clap dependencies,
//! making them suitable for FFI boundaries and library consumers.

use serde::{Deserialize, Serialize};

// Re-export types from tokmd_types for convenience
pub use tokmd_types::{ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode};

/// Global scan settings shared by all operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanSettings {
    /// Paths to scan (defaults to `["."]`).
    #[serde(default)]
    pub paths: Vec<String>,

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
