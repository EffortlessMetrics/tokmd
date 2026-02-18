//! # tokmd-core
//!
//! **Tier 4 (Library Facade)**
//!
//! This crate is the **primary library interface** for `tokmd`.
//! It coordinates scanning, aggregation, and modeling to produce code inventory receipts.
//!
//! If you are embedding `tokmd` into another Rust application, depend on this crate
//! and `tokmd-types`. Avoid depending on `tokmd-scan` or `tokmd-model` directly unless necessary.
//!
//! ## What belongs here
//! * High-level workflow coordination
//! * Simplified API for library consumers
//! * Re-exports for convenience
//! * FFI-friendly JSON entrypoint
//!
//! ## What does NOT belong here
//! * CLI argument parsing (use tokmd crate)
//! * Low-level scanning logic (use tokmd-scan)
//! * Aggregation details (use tokmd-model)
//!
//! ## Example
//!
//! ```rust,no_run
//! use tokmd_core::{lang_workflow, settings::{ScanSettings, LangSettings}};
//!
//! // Configure scan
//! let scan = ScanSettings::current_dir();
//! let lang = LangSettings {
//!     top: 10,
//!     files: true,
//!     ..Default::default()
//! };
//!
//! // Run pipeline
//! let receipt = lang_workflow(&scan, &lang).expect("Scan failed");
//! println!("Scanned {} languages", receipt.report.rows.len());
//! ```
//!
//! ## JSON API (for bindings)
//!
//! ```rust,no_run
//! use tokmd_core::ffi::run_json;
//!
//! let result = run_json("lang", r#"{"paths": ["."], "top": 10}"#);
//! println!("{}", result);
//! ```

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
#[cfg(feature = "analysis")]
use tokmd_analysis as analysis;
#[cfg(feature = "analysis")]
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};

// Public modules
pub mod error;
pub mod ffi;
pub mod settings;

// Re-export types for convenience
pub use tokmd_config as config;
pub use tokmd_types as types;

use settings::{DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_config::GlobalArgs;
use tokmd_format::scan_args;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    DiffReceipt, ExportArgsMeta, ExportData, ExportReceipt, LangArgs, LangArgsMeta, LangReceipt,
    LangReport, ModuleArgsMeta, ModuleReceipt, RedactMode, SCHEMA_VERSION, ScanStatus, ToolInfo,
};

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

// =============================================================================
// Settings-based workflows (new API for bindings)
// =============================================================================

/// Runs the language summary workflow with pure settings types.
///
/// This is the binding-friendly API that doesn't require Clap types.
///
/// # Arguments
///
/// * `scan` - Scan settings (paths, exclusions, etc.)
/// * `lang` - Language-specific settings (top N, files, etc.)
///
/// # Returns
///
/// A `LangReceipt` containing the language summary.
pub fn lang_workflow(scan: &ScanSettings, lang: &LangSettings) -> Result<LangReceipt> {
    let scan_opts = settings_to_scan_options(scan);
    let paths: Vec<PathBuf> = scan.paths.iter().map(PathBuf::from).collect();

    // Scan
    let languages = tokmd_scan::scan(&paths, &scan_opts)?;

    // Model
    let report = tokmd_model::create_lang_report(&languages, lang.top, lang.files, lang.children);

    // Build receipt
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(&paths, &scan_opts, lang.redact),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: lang.top,
            with_files: lang.files,
            children: lang.children,
        },
        report,
    };

    Ok(receipt)
}

/// Runs the module summary workflow with pure settings types.
///
/// # Arguments
///
/// * `scan` - Scan settings (paths, exclusions, etc.)
/// * `module` - Module-specific settings (roots, depth, etc.)
///
/// # Returns
///
/// A `ModuleReceipt` containing the module breakdown.
pub fn module_workflow(scan: &ScanSettings, module: &ModuleSettings) -> Result<ModuleReceipt> {
    let scan_opts = settings_to_scan_options(scan);
    let paths: Vec<PathBuf> = scan.paths.iter().map(PathBuf::from).collect();

    // Scan
    let languages = tokmd_scan::scan(&paths, &scan_opts)?;

    // Model
    let report = tokmd_model::create_module_report(
        &languages,
        &module.module_roots,
        module.module_depth,
        module.children,
        module.top,
    );

    // Build receipt
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(&paths, &scan_opts, module.redact),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            top: module.top,
            module_roots: module.module_roots.clone(),
            module_depth: module.module_depth,
            children: module.children,
        },
        report,
    };

    Ok(receipt)
}

/// Runs the export workflow with pure settings types.
///
/// # Arguments
///
/// * `scan` - Scan settings (paths, exclusions, etc.)
/// * `export` - Export-specific settings (format, min_code, etc.)
///
/// # Returns
///
/// An `ExportReceipt` containing file-level data.
pub fn export_workflow(scan: &ScanSettings, export: &ExportSettings) -> Result<ExportReceipt> {
    let scan_opts = settings_to_scan_options(scan);
    let paths: Vec<PathBuf> = scan.paths.iter().map(PathBuf::from).collect();
    let strip_prefix = export.strip_prefix.as_deref();

    // Scan
    let languages = tokmd_scan::scan(&paths, &scan_opts)?;

    // Model
    let data = tokmd_model::create_export_data(
        &languages,
        &export.module_roots,
        export.module_depth,
        export.children,
        strip_prefix.map(std::path::Path::new),
        export.min_code,
        export.max_rows,
    );

    // Apply redaction if needed
    let should_redact = export.redact == RedactMode::Paths || export.redact == RedactMode::All;
    let strip_prefix_redacted = should_redact && export.strip_prefix.is_some();

    // Build receipt
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(&paths, &scan_opts, Some(export.redact)),
        args: ExportArgsMeta {
            format: export.format,
            module_roots: export.module_roots.clone(),
            module_depth: export.module_depth,
            children: export.children,
            min_code: export.min_code,
            max_rows: export.max_rows,
            redact: export.redact,
            strip_prefix: if should_redact {
                export
                    .strip_prefix
                    .as_ref()
                    .map(|p| tokmd_format::redact_path(p))
            } else {
                export.strip_prefix.clone()
            },
            strip_prefix_redacted,
        },
        data: redact_export_data(data, export.redact),
    };

    Ok(receipt)
}

/// Runs the diff workflow comparing two receipts or paths.
///
/// # Arguments
///
/// * `settings` - Diff settings (from, to references)
///
/// # Returns
///
/// A `DiffReceipt` showing changes between the two states.
pub fn diff_workflow(settings: &DiffSettings) -> Result<DiffReceipt> {
    // Load or scan the "from" state
    let from_report = load_lang_report(&settings.from)?;

    // Load or scan the "to" state
    let to_report = load_lang_report(&settings.to)?;

    // Compute diff
    let rows = tokmd_format::compute_diff_rows(&from_report, &to_report);
    let totals = tokmd_format::compute_diff_totals(&rows);

    Ok(tokmd_format::create_diff_receipt(
        &settings.from,
        &settings.to,
        rows,
        totals,
    ))
}

/// Analyze workflow (requires `analysis` feature).
///
/// Runs export + analysis workflows and returns an `AnalysisReceipt`.
#[cfg(feature = "analysis")]
pub fn analyze_workflow(
    scan: &ScanSettings,
    analyze: &settings::AnalyzeSettings,
) -> Result<tokmd_analysis_types::AnalysisReceipt> {
    let export_receipt = export_workflow(scan, &ExportSettings::default())?;
    let (preset, preset_meta) = parse_analysis_preset(&analyze.preset)?;
    let (granularity, granularity_meta) = parse_import_granularity(&analyze.granularity)?;

    let source = AnalysisSource {
        inputs: scan.paths.clone(),
        export_path: None,
        base_receipt_path: None,
        export_schema_version: Some(export_receipt.schema_version),
        export_generated_at_ms: Some(export_receipt.generated_at_ms),
        base_signature: None,
        module_roots: export_receipt.data.module_roots.clone(),
        module_depth: export_receipt.data.module_depth,
        children: child_include_mode_to_string(export_receipt.data.children),
    };

    let request = analysis::AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: preset_meta,
            format: "json".to_string(),
            window_tokens: analyze.window,
            git: analyze.git,
            max_files: analyze.max_files,
            max_bytes: analyze.max_bytes,
            max_file_bytes: analyze.max_file_bytes,
            max_commits: analyze.max_commits,
            max_commit_files: analyze.max_commit_files,
            import_granularity: granularity_meta,
        },
        limits: analysis::AnalysisLimits {
            max_files: analyze.max_files,
            max_bytes: analyze.max_bytes,
            max_file_bytes: analyze.max_file_bytes,
            max_commits: analyze.max_commits,
            max_commit_files: analyze.max_commit_files,
        },
        window_tokens: analyze.window,
        git: analyze.git,
        import_granularity: granularity,
        detail_functions: false,
    };

    let root = derive_analysis_root(scan)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let ctx = analysis::AnalysisContext {
        export: export_receipt.data,
        root,
        source,
    };

    analysis::analyze(ctx, request)
}

// =============================================================================
// Cockpit workflow (requires `cockpit` feature)
// =============================================================================

/// Cockpit workflow: compute PR metrics and evidence gates.
///
/// Runs the cockpit analysis pipeline using pure settings types.
///
/// # Arguments
///
/// * `settings` - Cockpit settings (base/head refs, range mode, baseline)
///
/// # Returns
///
/// A `CockpitReceipt` containing PR metrics, evidence gates, and review plan.
#[cfg(feature = "cockpit")]
pub fn cockpit_workflow(
    settings: &settings::CockpitSettings,
) -> Result<tokmd_types::cockpit::CockpitReceipt> {
    use tokmd_types::cockpit::CockpitReceipt;

    if !tokmd_git::git_available() {
        anyhow::bail!("git is not available on PATH");
    }

    let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
    let repo_root = tokmd_git::repo_root(&cwd)
        .ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

    let range_mode = match settings.range_mode.as_str() {
        "three-dot" | "3dot" => tokmd_git::GitRangeMode::ThreeDot,
        _ => tokmd_git::GitRangeMode::TwoDot,
    };

    let resolved_base =
        tokmd_git::resolve_base_ref(&repo_root, &settings.base).ok_or_else(|| {
            anyhow::anyhow!(
                "base ref '{}' not found and no fallback resolved",
                settings.base
            )
        })?;

    let baseline_path = settings.baseline.as_deref();

    let mut receipt: CockpitReceipt = tokmd_cockpit::compute_cockpit(
        &repo_root,
        &resolved_base,
        &settings.head,
        range_mode,
        baseline_path.map(std::path::Path::new),
    )?;

    // Load baseline and compute trend if provided
    if let Some(baseline_path) = baseline_path {
        receipt.trend = Some(tokmd_cockpit::load_and_compute_trend(
            std::path::Path::new(baseline_path),
            &receipt,
        )?);
    }

    Ok(receipt)
}

#[cfg(feature = "cockpit")]
use anyhow::Context as _;

// =============================================================================
// Legacy API (for backwards compatibility with CLI)
// =============================================================================

/// Runs the complete scan workflow: Scan -> Model -> Receipt.
///
/// This is the high-level entry point for generating a language inventory.
///
/// # Arguments
///
/// * `global` - Global scan configuration (excluded patterns, ignore settings, etc.)
/// * `lang` - Language-specific arguments (format, top N, etc.)
/// * `redact` - Optional redaction mode for safer output (e.g., when sharing with LLMs)
///
/// # Redaction Modes
///
/// * `None` or `Some(RedactMode::None)` - No redaction, paths shown as-is
/// * `Some(RedactMode::Paths)` - Redact file paths (replaced with hashed values preserving extension)
/// * `Some(RedactMode::All)` - Redact paths and excluded patterns
pub fn scan_workflow(
    global: &GlobalArgs,
    lang: &LangArgs,
    redact: Option<RedactMode>,
) -> Result<LangReceipt> {
    // 1. Scan
    let scan_opts = ScanOptions::from(global);
    let languages = tokmd_scan::scan(&lang.paths, &scan_opts)?;

    // 2. Model (Aggregation & Filtering)
    // create_lang_report handles filtering (top N) and children mode
    let report = tokmd_model::create_lang_report(&languages, lang.top, lang.files, lang.children);

    // 3. Receipt Construction
    // We construct the receipt manually as it's just a data carrier.
    let scan_args = scan_args(&lang.paths, &scan_opts, redact);

    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![], // Tokei scan might have warnings but scan() doesn't return them currently
        scan: scan_args,
        args: LangArgsMeta {
            format: format!("{:?}", lang.format), // Enums might need Display impl or conversion
            top: lang.top,
            with_files: lang.files,
            children: lang.children,
        },
        report,
    };

    Ok(receipt)
}

// =============================================================================
// Helper functions
// =============================================================================

/// Convert ScanSettings to ScanOptions for lower-tier crates.
fn settings_to_scan_options(scan: &ScanSettings) -> ScanOptions {
    scan.options.clone()
}

#[cfg(feature = "analysis")]
fn parse_analysis_preset(value: &str) -> Result<(analysis::AnalysisPreset, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    let preset = match normalized.as_str() {
        "receipt" => analysis::AnalysisPreset::Receipt,
        "health" => analysis::AnalysisPreset::Health,
        "risk" => analysis::AnalysisPreset::Risk,
        "supply" => analysis::AnalysisPreset::Supply,
        "architecture" => analysis::AnalysisPreset::Architecture,
        "topics" => analysis::AnalysisPreset::Topics,
        "security" => analysis::AnalysisPreset::Security,
        "identity" => analysis::AnalysisPreset::Identity,
        "git" => analysis::AnalysisPreset::Git,
        "deep" => analysis::AnalysisPreset::Deep,
        "fun" => analysis::AnalysisPreset::Fun,
        _ => {
            return Err(error::TokmdError::invalid_field(
                "preset",
                "'receipt', 'health', 'risk', 'supply', 'architecture', 'topics', 'security', 'identity', 'git', 'deep', or 'fun'",
            )
            .into());
        }
    };
    Ok((preset, normalized))
}

#[cfg(feature = "analysis")]
fn parse_import_granularity(value: &str) -> Result<(analysis::ImportGranularity, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    let granularity = match normalized.as_str() {
        "module" => analysis::ImportGranularity::Module,
        "file" => analysis::ImportGranularity::File,
        _ => {
            return Err(
                error::TokmdError::invalid_field("granularity", "'module' or 'file'").into(),
            );
        }
    };
    Ok((granularity, normalized))
}

#[cfg(feature = "analysis")]
fn child_include_mode_to_string(mode: tokmd_types::ChildIncludeMode) -> String {
    match mode {
        tokmd_types::ChildIncludeMode::Separate => "separate".to_string(),
        tokmd_types::ChildIncludeMode::ParentsOnly => "parents-only".to_string(),
    }
}

#[cfg(feature = "analysis")]
fn derive_analysis_root(scan: &ScanSettings) -> Option<PathBuf> {
    let first = scan.paths.first()?;
    if first.trim().is_empty() {
        return None;
    }

    let candidate = PathBuf::from(first);
    let absolute = if candidate.is_absolute() {
        candidate
    } else {
        std::env::current_dir().ok()?.join(candidate)
    };

    if absolute.is_dir() {
        Some(absolute)
    } else {
        absolute.parent().map(|p| p.to_path_buf())
    }
}

/// Load a LangReport from a file path or scan a directory.
fn load_lang_report(source: &str) -> Result<LangReport> {
    let path = std::path::Path::new(source);

    if path.exists() && path.is_file() {
        // Try to load as a receipt file
        let content = std::fs::read_to_string(path)?;
        if let Ok(receipt) = serde_json::from_str::<LangReceipt>(&content) {
            return Ok(receipt.report);
        }
        // Fall through to scanning if not a valid receipt
    }

    // Scan the path
    let scan = ScanSettings::for_paths(vec![source.to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang)?;
    Ok(receipt.report)
}

/// Apply redaction to export data.
fn redact_export_data(data: ExportData, mode: RedactMode) -> ExportData {
    if mode == RedactMode::None {
        return data;
    }

    let rows = data
        .rows
        .into_iter()
        .map(|mut row| {
            if mode == RedactMode::Paths || mode == RedactMode::All {
                row.path = tokmd_format::redact_path(&row.path);
            }
            if mode == RedactMode::All {
                row.module = tokmd_format::short_hash(&row.module);
            }
            row
        })
        .collect();

    ExportData {
        rows,
        module_roots: data.module_roots,
        module_depth: data.module_depth,
        children: data.children,
    }
}

// =============================================================================
// Re-exports for binding convenience
// =============================================================================

/// Re-export schema version for bindings.
pub const CORE_SCHEMA_VERSION: u32 = SCHEMA_VERSION;

/// Get the current tokmd version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_not_empty() {
        assert!(!version().is_empty());
    }

    #[test]
    fn settings_to_scan_options_preserves_values() {
        let scan = ScanSettings {
            paths: vec!["src".to_string()],
            options: ScanOptions {
                excluded: vec!["target".to_string()],
                hidden: true,
                no_ignore: true,
                ..Default::default()
            },
        };

        let opts = settings_to_scan_options(&scan);
        assert_eq!(opts.excluded, vec!["target"]);
        assert!(opts.hidden);
        assert!(opts.no_ignore);
    }

    #[test]
    fn scan_settings_current_dir() {
        let settings = ScanSettings::current_dir();
        assert_eq!(settings.paths, vec!["."]);
    }

    #[test]
    fn scan_settings_for_paths() {
        let settings = ScanSettings::for_paths(vec!["src".to_string(), "lib".to_string()]);
        assert_eq!(settings.paths, vec!["src", "lib"]);
    }
}
