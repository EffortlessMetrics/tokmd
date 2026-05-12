//! Core inventory receipt DTOs.
//!
//! This module owns the serde-stable structures emitted by `tokmd lang`,
//! `tokmd module`, `tokmd export`, and `tokmd run`. Public consumers should
//! continue using the root-level re-exports from `tokmd_types`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A small totals struct shared by summary outputs.
///
/// # Examples
///
/// ```
/// use tokmd_types::Totals;
///
/// let totals = Totals {
///     code: 1000,
///     lines: 1500,
///     files: 10,
///     bytes: 40000,
///     tokens: 10000,
///     avg_lines: 150,
/// };
/// assert_eq!(totals.code, 1000);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Totals {
    pub code: usize,
    pub lines: usize,
    pub files: usize,
    pub bytes: usize,
    pub tokens: usize,
    pub avg_lines: usize,
}

/// A single language row in the lang summary.
///
/// # Examples
///
/// ```
/// use tokmd_types::LangRow;
///
/// let row = LangRow {
///     lang: "Rust".to_string(),
///     code: 5000,
///     lines: 6500,
///     files: 42,
///     bytes: 180_000,
///     tokens: 45_000,
///     avg_lines: 154,
/// };
/// assert_eq!(row.lang, "Rust");
/// assert_eq!(row.files, 42);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LangRow {
    pub lang: String,
    pub code: usize,
    pub lines: usize,
    pub files: usize,
    pub bytes: usize,
    pub tokens: usize,
    pub avg_lines: usize,
}

/// A report detailing language statistics.
///
/// # Examples
///
/// ```
/// use tokmd_types::{LangReport, LangRow, Totals, ChildrenMode};
///
/// let report = LangReport {
///     rows: vec![
///         LangRow {
///             lang: "Rust".to_string(),
///             code: 5000,
///             lines: 6500,
///             files: 42,
///             bytes: 180_000,
///             tokens: 45_000,
///             avg_lines: 154,
///         }
///     ],
///     total: Totals {
///         code: 5000,
///         lines: 6500,
///         files: 42,
///         bytes: 180_000,
///         tokens: 45_000,
///         avg_lines: 154,
///     },
///     with_files: false,
///     children: ChildrenMode::Collapse,
///     top: 10,
/// };
/// assert_eq!(report.rows.len(), 1);
/// assert_eq!(report.total.files, 42);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangReport {
    pub rows: Vec<LangRow>,
    pub total: Totals,
    pub with_files: bool,
    pub children: ChildrenMode,
    pub top: usize,
}

/// A single module row in the module breakdown.
///
/// # Examples
///
/// ```
/// use tokmd_types::ModuleRow;
///
/// let row = ModuleRow {
///     module: "crates/tokmd-types".to_string(),
///     code: 800,
///     lines: 1100,
///     files: 3,
///     bytes: 32_000,
///     tokens: 8_000,
///     avg_lines: 366,
/// };
/// assert_eq!(row.module, "crates/tokmd-types");
/// assert_eq!(row.code, 800);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleRow {
    pub module: String,
    pub code: usize,
    pub lines: usize,
    pub files: usize,
    pub bytes: usize,
    pub tokens: usize,
    pub avg_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleReport {
    pub rows: Vec<ModuleRow>,
    pub total: Totals,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
    pub top: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Parent,
    Child,
}

/// A single file row in the export inventory.
///
/// # Examples
///
/// ```
/// use tokmd_types::{FileRow, FileKind};
///
/// let row = FileRow {
///     path: "src/main.rs".to_string(),
///     module: "src".to_string(),
///     lang: "Rust".to_string(),
///     kind: FileKind::Parent,
///     code: 120,
///     comments: 30,
///     blanks: 20,
///     lines: 170,
///     bytes: 4_800,
///     tokens: 1_200,
/// };
/// assert_eq!(row.path, "src/main.rs");
/// assert_eq!(row.kind, FileKind::Parent);
/// assert_eq!(row.lines, row.code + row.comments + row.blanks);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileRow {
    pub path: String,
    pub module: String,
    pub lang: String,
    pub kind: FileKind,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub lines: usize,
    pub bytes: usize,
    pub tokens: usize,
}

/// Detailed export data containing individual file statistics.
///
/// # Examples
///
/// ```
/// use tokmd_types::{ExportData, FileRow, FileKind, ChildIncludeMode};
///
/// let data = ExportData {
///     rows: vec![
///         FileRow {
///             path: "src/main.rs".to_string(),
///             module: "src".to_string(),
///             lang: "Rust".to_string(),
///             kind: FileKind::Parent,
///             code: 120,
///             comments: 30,
///             blanks: 20,
///             lines: 170,
///             bytes: 4_800,
///             tokens: 1_200,
///         }
///     ],
///     module_roots: vec![],
///     module_depth: 1,
///     children: ChildIncludeMode::Separate,
/// };
/// assert_eq!(data.rows.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub rows: Vec<FileRow>,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub lang_file: String,
    pub module_file: String,
    pub export_file: String,
    // We could store the scan args here too
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Complete,
    Partial,
}

/// Classification of a commit's intent, derived from subject line.
///
/// Lives in `tokmd-types` (Tier 0) so that both `tokmd-git` (Tier 2) and
/// `tokmd-analysis-types` (Tier 0) can reference it without creating
/// upward dependency edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitIntentKind {
    Feat,
    Fix,
    Refactor,
    Docs,
    Test,
    Chore,
    Ci,
    Build,
    Perf,
    Style,
    Revert,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
}

impl ToolInfo {
    pub fn current() -> Self {
        Self {
            name: "tokmd".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanArgs {
    pub paths: Vec<String>,
    pub excluded: Vec<String>,
    /// True if `excluded` patterns were redacted (replaced with hashes).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub excluded_redacted: bool,
    pub config: ConfigMode,
    pub hidden: bool,
    pub no_ignore: bool,
    pub no_ignore_parent: bool,
    pub no_ignore_dot: bool,
    pub no_ignore_vcs: bool,
    pub treat_doc_strings_as_comments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangArgsMeta {
    pub format: String,
    pub top: usize,
    pub with_files: bool,
    pub children: ChildrenMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String, // "lang"
    pub status: ScanStatus,
    pub warnings: Vec<String>,
    pub scan: ScanArgs,
    pub args: LangArgsMeta,
    #[serde(flatten)]
    pub report: LangReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleArgsMeta {
    pub format: String,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
    pub top: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String, // "module"
    pub status: ScanStatus,
    pub warnings: Vec<String>,
    pub scan: ScanArgs,
    pub args: ModuleArgsMeta,
    #[serde(flatten)]
    pub report: ModuleReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportArgsMeta {
    pub format: ExportFormat,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
    pub min_code: usize,
    pub max_rows: usize,
    pub redact: RedactMode,
    pub strip_prefix: Option<String>,
    /// True if `strip_prefix` was redacted (replaced with a hash).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub strip_prefix_redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportReceipt {
    pub schema_version: u32,
    pub generated_at_ms: u128,
    pub tool: ToolInfo,
    pub mode: String, // "export"
    pub status: ScanStatus,
    pub warnings: Vec<String>,
    pub scan: ScanArgs,
    pub args: ExportArgsMeta,
    #[serde(flatten)]
    pub data: ExportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangArgs {
    pub paths: Vec<PathBuf>,
    pub format: TableFormat,
    pub top: usize,
    pub files: bool,
    pub children: ChildrenMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleArgs {
    pub paths: Vec<PathBuf>,
    pub format: TableFormat,
    pub top: usize,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportArgs {
    pub paths: Vec<PathBuf>,
    pub format: ExportFormat,
    pub output: Option<PathBuf>,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
    pub min_code: usize,
    pub max_rows: usize,
    pub redact: RedactMode,
    pub meta: bool,
    pub strip_prefix: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TableFormat {
    /// Markdown table (great for pasting into ChatGPT).
    Md,
    /// Tab-separated values (good for piping to other tools).
    Tsv,
    /// JSON (compact).
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    /// CSV with a header row.
    Csv,
    /// One JSON object per line.
    Jsonl,
    /// A single JSON array.
    Json,
    /// CycloneDX 1.6 JSON SBOM format.
    Cyclonedx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigMode {
    /// Read scan config files (`tokei.toml` / `.tokeirc`) if present.
    #[default]
    Auto,
    /// Ignore config files.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildrenMode {
    /// Merge embedded content into the parent language totals.
    Collapse,
    /// Show embedded languages as separate "(embedded)" rows.
    Separate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChildIncludeMode {
    /// Include embedded languages as separate contributions.
    Separate,
    /// Ignore embedded languages.
    ParentsOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RedactMode {
    /// Do not redact.
    None,
    /// Redact file paths.
    Paths,
    /// Redact file paths and module names.
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisFormat {
    Md,
    Json,
    Jsonld,
    Xml,
    Svg,
    Mermaid,
    Obj,
    Midi,
    Tree,
    Html,
}
