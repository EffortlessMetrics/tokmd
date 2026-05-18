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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_totals() -> Totals {
        Totals {
            code: 100,
            lines: 200,
            files: 10,
            bytes: 5_000,
            tokens: 250,
            avg_lines: 20,
        }
    }

    fn sample_lang_row() -> LangRow {
        LangRow {
            lang: "Rust".into(),
            code: 100,
            lines: 150,
            files: 5,
            bytes: 3_000,
            tokens: 200,
            avg_lines: 30,
        }
    }

    fn sample_module_row() -> ModuleRow {
        ModuleRow {
            module: "src".into(),
            code: 80,
            lines: 120,
            files: 4,
            bytes: 2_500,
            tokens: 160,
            avg_lines: 30,
        }
    }

    fn sample_file_row() -> FileRow {
        FileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 50,
            comments: 10,
            blanks: 5,
            lines: 65,
            bytes: 2_000,
            tokens: 100,
        }
    }

    fn sample_scan_args() -> ScanArgs {
        ScanArgs {
            paths: vec!["src".into(), "tests".into()],
            excluded: vec!["target".into()],
            excluded_redacted: false,
            config: ConfigMode::Auto,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        }
    }

    // ── Totals ───────────────────────────────────────────────────────
    #[test]
    fn totals_serde_roundtrip() {
        let t = sample_totals();
        let json = serde_json::to_string(&t).unwrap();
        let back: Totals = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn totals_field_names_stable() {
        let value = serde_json::to_value(sample_totals()).unwrap();
        for key in ["code", "lines", "files", "bytes", "tokens", "avg_lines"] {
            assert!(value.get(key).is_some(), "missing key `{key}` in Totals");
        }
    }

    // ── LangRow / LangReport ─────────────────────────────────────────
    #[test]
    fn lang_row_serde_roundtrip() {
        let r = sample_lang_row();
        let json = serde_json::to_string(&r).unwrap();
        let back: LangRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn lang_row_field_names_stable() {
        let value = serde_json::to_value(sample_lang_row()).unwrap();
        for key in [
            "lang",
            "code",
            "lines",
            "files",
            "bytes",
            "tokens",
            "avg_lines",
        ] {
            assert!(value.get(key).is_some(), "missing key `{key}` in LangRow");
        }
    }

    #[test]
    fn lang_report_serde_roundtrip() {
        let report = LangReport {
            rows: vec![sample_lang_row()],
            total: sample_totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 10,
        };
        let json = serde_json::to_string(&report).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in ["rows", "total", "with_files", "children", "top"] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in LangReport"
            );
        }
        let back: LangReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rows.len(), 1);
        assert_eq!(back.rows[0], report.rows[0]);
        assert_eq!(back.total, report.total);
        assert_eq!(back.top, 10);
    }

    // ── ModuleRow / ModuleReport ─────────────────────────────────────
    #[test]
    fn module_row_serde_roundtrip() {
        let r = sample_module_row();
        let json = serde_json::to_string(&r).unwrap();
        let back: ModuleRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn module_row_field_names_stable() {
        let value = serde_json::to_value(sample_module_row()).unwrap();
        for key in [
            "module",
            "code",
            "lines",
            "files",
            "bytes",
            "tokens",
            "avg_lines",
        ] {
            assert!(value.get(key).is_some(), "missing key `{key}` in ModuleRow");
        }
    }

    #[test]
    fn module_report_serde_roundtrip() {
        let report = ModuleReport {
            rows: vec![sample_module_row()],
            total: sample_totals(),
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::ParentsOnly,
            top: 5,
        };
        let json = serde_json::to_string(&report).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "rows",
            "total",
            "module_roots",
            "module_depth",
            "children",
            "top",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in ModuleReport"
            );
        }
        let back: ModuleReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rows.len(), 1);
        assert_eq!(back.module_depth, 2);
        assert_eq!(back.top, 5);
    }

    // ── FileRow / ExportData ─────────────────────────────────────────
    #[test]
    fn file_row_serde_roundtrip_parent() {
        let r = sample_file_row();
        let json = serde_json::to_string(&r).unwrap();
        let back: FileRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn file_row_serde_roundtrip_child() {
        let r = FileRow {
            kind: FileKind::Child,
            ..sample_file_row()
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: FileRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.kind, FileKind::Child);
    }

    #[test]
    fn file_row_field_names_stable() {
        let value = serde_json::to_value(sample_file_row()).unwrap();
        for key in [
            "path", "module", "lang", "kind", "code", "comments", "blanks", "lines", "bytes",
            "tokens",
        ] {
            assert!(value.get(key).is_some(), "missing key `{key}` in FileRow");
        }
    }

    #[test]
    fn export_data_serde_roundtrip_preserves_row_order() {
        let mut rows = Vec::new();
        for path in ["a.rs", "b.rs", "c.rs", "d.rs"] {
            let mut row = sample_file_row();
            row.path = path.to_string();
            rows.push(row);
        }
        let data = ExportData {
            rows: rows.clone(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let json = serde_json::to_string(&data).unwrap();
        let back: ExportData = serde_json::from_str(&json).unwrap();
        let paths: Vec<_> = back.rows.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, vec!["a.rs", "b.rs", "c.rs", "d.rs"]);
        assert_eq!(back.children, ChildIncludeMode::Separate);
        assert_eq!(back.module_depth, 1);
    }

    // ── Enums ────────────────────────────────────────────────────────
    #[test]
    fn file_kind_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&FileKind::Parent).unwrap(),
            "\"parent\""
        );
        assert_eq!(
            serde_json::to_string(&FileKind::Child).unwrap(),
            "\"child\""
        );
    }

    #[test]
    fn scan_status_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&ScanStatus::Complete).unwrap(),
            "\"complete\""
        );
        assert_eq!(
            serde_json::to_string(&ScanStatus::Partial).unwrap(),
            "\"partial\""
        );
        for variant in [ScanStatus::Complete, ScanStatus::Partial] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ScanStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn child_include_mode_uses_kebab_case_for_parents_only() {
        assert_eq!(
            serde_json::to_string(&ChildIncludeMode::ParentsOnly).unwrap(),
            "\"parents-only\""
        );
        for variant in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn analysis_format_kebab_case_check() {
        assert_eq!(
            serde_json::to_string(&AnalysisFormat::Jsonld).unwrap(),
            "\"jsonld\""
        );
        assert_eq!(
            serde_json::to_string(&AnalysisFormat::Mermaid).unwrap(),
            "\"mermaid\""
        );
    }

    #[test]
    fn commit_intent_kind_all_variants_roundtrip() {
        for variant in [
            CommitIntentKind::Feat,
            CommitIntentKind::Fix,
            CommitIntentKind::Refactor,
            CommitIntentKind::Docs,
            CommitIntentKind::Test,
            CommitIntentKind::Chore,
            CommitIntentKind::Ci,
            CommitIntentKind::Build,
            CommitIntentKind::Perf,
            CommitIntentKind::Style,
            CommitIntentKind::Revert,
            CommitIntentKind::Other,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn config_mode_default_is_auto() {
        assert_eq!(ConfigMode::default(), ConfigMode::Auto);
        assert_eq!(
            serde_json::to_string(&ConfigMode::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&ConfigMode::None).unwrap(),
            "\"none\""
        );
    }

    // ── ToolInfo ─────────────────────────────────────────────────────
    #[test]
    fn tool_info_default_serde() {
        let tool = ToolInfo::default();
        let json = serde_json::to_string(&tool).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("name").is_some());
        assert!(value.get("version").is_some());
        assert_eq!(value["name"], "");
        assert_eq!(value["version"], "");
    }

    #[test]
    fn tool_info_current_has_tokmd_name() {
        let tool = ToolInfo::current();
        assert_eq!(tool.name, "tokmd");
        assert!(!tool.version.is_empty());
        let json = serde_json::to_string(&tool).unwrap();
        let back: ToolInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, tool.name);
        assert_eq!(back.version, tool.version);
    }

    // ── ScanArgs ─────────────────────────────────────────────────────
    #[test]
    fn scan_args_roundtrip_preserves_paths_order() {
        let args = sample_scan_args();
        let json = serde_json::to_string(&args).unwrap();
        let back: ScanArgs = serde_json::from_str(&json).unwrap();
        assert_eq!(back.paths, vec!["src".to_string(), "tests".to_string()]);
        assert_eq!(back.excluded, vec!["target".to_string()]);
        assert_eq!(back.config, ConfigMode::Auto);
    }

    #[test]
    fn scan_args_excluded_redacted_omitted_when_false() {
        let args = sample_scan_args();
        let value = serde_json::to_value(&args).unwrap();
        assert!(value.get("excluded_redacted").is_none());
    }

    #[test]
    fn scan_args_excluded_redacted_present_when_true() {
        let args = ScanArgs {
            excluded_redacted: true,
            ..sample_scan_args()
        };
        let value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["excluded_redacted"], true);
    }

    // ── Receipts ─────────────────────────────────────────────────────
    #[test]
    fn lang_receipt_flattens_report_fields() {
        let receipt = LangReceipt {
            schema_version: crate::SCHEMA_VERSION,
            generated_at_ms: 1_700_000_000_000,
            tool: ToolInfo::current(),
            mode: "lang".into(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: sample_scan_args(),
            args: LangArgsMeta {
                format: "md".into(),
                top: 10,
                with_files: true,
                children: ChildrenMode::Separate,
            },
            report: LangReport {
                rows: vec![sample_lang_row()],
                total: sample_totals(),
                with_files: true,
                children: ChildrenMode::Separate,
                top: 10,
            },
        };
        let value = serde_json::to_value(&receipt).unwrap();
        // Envelope fields
        for key in [
            "schema_version",
            "generated_at_ms",
            "tool",
            "mode",
            "status",
            "warnings",
            "scan",
            "args",
        ] {
            assert!(value.get(key).is_some(), "missing envelope key `{key}`");
        }
        // Flattened report fields
        for key in ["rows", "total", "with_files", "children", "top"] {
            assert!(
                value.get(key).is_some(),
                "missing flattened report key `{key}`"
            );
        }
        // Roundtrip
        let json = serde_json::to_string(&receipt).unwrap();
        let back: LangReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mode, "lang");
        assert_eq!(back.report.rows.len(), 1);
    }

    #[test]
    fn module_receipt_flattens_report_fields() {
        let receipt = ModuleReceipt {
            schema_version: crate::SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: ToolInfo::default(),
            mode: "module".into(),
            status: ScanStatus::Partial,
            warnings: vec!["something".into()],
            scan: sample_scan_args(),
            args: ModuleArgsMeta {
                format: "json".into(),
                module_roots: vec!["crates".into()],
                module_depth: 3,
                children: ChildIncludeMode::Separate,
                top: 20,
            },
            report: ModuleReport {
                rows: vec![sample_module_row()],
                total: sample_totals(),
                module_roots: vec!["crates".into()],
                module_depth: 3,
                children: ChildIncludeMode::Separate,
                top: 20,
            },
        };
        let json = serde_json::to_string(&receipt).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["status"], "partial");
        // Flattened ModuleReport fields
        for key in [
            "rows",
            "total",
            "module_roots",
            "module_depth",
            "children",
            "top",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing flattened key `{key}` in ModuleReceipt JSON"
            );
        }
        let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mode, "module");
        assert_eq!(back.report.module_depth, 3);
        assert_eq!(back.warnings, vec!["something".to_string()]);
    }

    #[test]
    fn export_receipt_flattens_data_fields() {
        let receipt = ExportReceipt {
            schema_version: crate::SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: ToolInfo::default(),
            mode: "export".into(),
            status: ScanStatus::Complete,
            warnings: vec![],
            scan: sample_scan_args(),
            args: ExportArgsMeta {
                format: ExportFormat::Json,
                module_roots: vec![],
                module_depth: 1,
                children: ChildIncludeMode::Separate,
                min_code: 0,
                max_rows: 100,
                redact: RedactMode::None,
                strip_prefix: None,
                strip_prefix_redacted: false,
            },
            data: ExportData {
                rows: vec![sample_file_row()],
                module_roots: vec![],
                module_depth: 1,
                children: ChildIncludeMode::Separate,
            },
        };
        let json = serde_json::to_string(&receipt).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        // Flattened ExportData fields
        for key in ["rows", "module_roots", "module_depth", "children"] {
            assert!(
                value.get(key).is_some(),
                "missing flattened key `{key}` in ExportReceipt JSON"
            );
        }
        let back: ExportReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mode, "export");
        assert_eq!(back.data.rows.len(), 1);
    }

    #[test]
    fn export_args_meta_strip_prefix_omitted_when_false() {
        let meta = ExportArgsMeta {
            format: ExportFormat::Csv,
            module_roots: vec![],
            module_depth: 0,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        };
        let value = serde_json::to_value(&meta).unwrap();
        assert!(value.get("strip_prefix_redacted").is_none());
    }

    #[test]
    fn export_args_meta_strip_prefix_present_when_true() {
        let meta = ExportArgsMeta {
            format: ExportFormat::Csv,
            module_roots: vec![],
            module_depth: 0,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: Some("abc".into()),
            strip_prefix_redacted: true,
        };
        let value = serde_json::to_value(&meta).unwrap();
        assert_eq!(value["strip_prefix_redacted"], true);
        assert_eq!(value["strip_prefix"], "abc");
    }

    #[test]
    fn run_receipt_serde_roundtrip() {
        let receipt = RunReceipt {
            schema_version: crate::SCHEMA_VERSION,
            generated_at_ms: 1_700_000_000_000,
            lang_file: "lang.json".into(),
            module_file: "module.json".into(),
            export_file: "export.json".into(),
        };
        let json = serde_json::to_string(&receipt).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "schema_version",
            "generated_at_ms",
            "lang_file",
            "module_file",
            "export_file",
        ] {
            assert!(
                value.get(key).is_some(),
                "missing key `{key}` in RunReceipt JSON"
            );
        }
        let back: RunReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(back.lang_file, "lang.json");
        assert_eq!(back.export_file, "export.json");
    }
}
