//! Shared scan settings independent of clap parsing.

use serde::{Deserialize, Serialize};
use tokmd_types::ConfigMode;

/// Scan options shared by all commands that invoke the scanner.
///
/// This mirrors the scan-relevant fields of CLI global args without any
/// UI-specific fields (`verbose`, `no_progress`). Lower-tier crates
/// (scan, format, model) depend on this instead of the CLI parser.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanOptions {
    /// Glob patterns to exclude.
    #[serde(default)]
    pub excluded: Vec<String>,

    /// Whether to load scan config files (`tokei.toml` / `.tokeirc`).
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
