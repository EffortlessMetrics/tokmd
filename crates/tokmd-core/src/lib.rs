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
//! ```rust
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
//! assert!(receipt.report.rows.len() > 0);
//! ```
//!
//! ## JSON API (for bindings)
//!
//! ```rust
//! use tokmd_core::ffi::run_json;
//!
//! let result = run_json("lang", r#"{"paths": ["."], "top": 10}"#);
//! let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
//! assert_eq!(parsed["ok"], true);
//! ```

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
#[cfg(all(test, feature = "analysis"))]
use tokmd_analysis as analysis;

// Public modules
pub mod context_git;
pub mod context_policy;
pub mod error;
pub mod ffi;
pub mod settings;
mod workflows;
pub use tokmd_scan::InMemoryFile;
pub use tokmd_types as types;
#[cfg(feature = "cockpit")]
pub use workflows::cockpit_workflow;
#[cfg(all(test, feature = "cockpit"))]
use workflows::parse_cockpit_range_mode;
#[cfg(feature = "analysis")]
pub use workflows::{
    analyze_workflow, analyze_workflow_from_inputs, supports_rootless_in_memory_analyze_preset,
};
pub use workflows::{
    diff_workflow, export_workflow, export_workflow_from_inputs, lang_workflow,
    lang_workflow_from_inputs, module_workflow, module_workflow_from_inputs,
};
#[cfg(all(test, feature = "analysis"))]
use workflows::{parse_analysis_preset, parse_effort_request};

use settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_format::scan_args;
use tokmd_settings::ScanOptions;
use tokmd_types::{
    ChildIncludeMode, ExportArgsMeta, ExportData, ExportReceipt, FileRow, LangArgsMeta,
    LangReceipt, LangReport, ModuleArgsMeta, ModuleReceipt, ModuleReport, RedactMode,
    SCHEMA_VERSION, ScanStatus, ToolInfo,
};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn now_ms() -> u128 {
    // Keep wasm receipts from reusing zero as a fake wall-clock sentinel.
    js_sys::Date::now().max(1.0) as u128
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

// =============================================================================
// Settings-based workflows (new API for bindings)
// =============================================================================

// =============================================================================
// Analysis formatting facade (requires `analysis` feature)
// =============================================================================

/// Analysis formatting re-exports for Tier 5 products.
///
/// This module provides Tier 4 facade access to Tier 3 analysis formatting,
/// maintaining tier boundary compliance for tokmd CLI and other products.
///
/// ## Example
///
/// ```rust
/// use tokmd_core::analysis_facade::{render, RenderedOutput};
/// use tokmd_types::AnalysisFormat;
/// use tokmd_analysis_types::AnalysisReceipt;
///
/// fn format_analysis(receipt: &AnalysisReceipt, format: AnalysisFormat) -> anyhow::Result<String> {
///     match render(receipt, format)? {
///         RenderedOutput::Text(text) => Ok(text),
///         RenderedOutput::Binary(_) => Err(anyhow::anyhow!("Binary output not supported")),
///     }
/// }
/// ```
#[cfg(feature = "analysis")]
pub mod analysis_facade {
    /// Render an analysis receipt to the specified format.
    ///
    /// # Arguments
    /// * `receipt` — The analysis receipt to render (from `tokmd_analysis_types`)
    /// * `format` — Target output format (from `tokmd_types::AnalysisFormat`)
    ///
    /// # Returns
    /// `RenderedOutput` enum containing either text or binary data
    ///
    /// # Errors
    /// Returns error if:
    /// - JSON/XML serialization fails
    /// - `fun` feature is disabled but OBJ/MIDI format requested
    pub use tokmd_format::analysis::render;

    /// Output container for rendered analysis.
    ///
    /// ## Variants
    /// - `Text(String)` — Textual formats: Markdown, JSON, XML, SVG, Mermaid, Tree, HTML
    /// - `Binary(Vec<u8>)` — Binary formats: MIDI (requires `fun` feature)
    pub use tokmd_format::analysis::RenderedOutput;
}

// =============================================================================
// Helper functions
// =============================================================================

/// Convert ScanSettings to ScanOptions for lower-tier crates.
fn settings_to_scan_options(scan: &ScanSettings) -> ScanOptions {
    scan.options.clone()
}

fn scan_paths_or_current_dir(scan: &ScanSettings) -> Vec<PathBuf> {
    if scan.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        scan.paths.iter().map(PathBuf::from).collect()
    }
}

fn deterministic_in_memory_scan_options(scan_opts: &ScanOptions) -> ScanOptions {
    let mut effective = scan_opts.clone();
    // Explicit in-memory inputs are authoritative; they should not depend on
    // host cwd config discovery or be filtered back out by hidden/exclude rules.
    effective.config = tokmd_types::ConfigMode::None;
    effective.hidden = true;
    effective.excluded.clear();
    effective
}

fn collect_pure_in_memory_rows(
    inputs: &[InMemoryFile],
    scan_opts: &ScanOptions,
    module_roots: &[String],
    module_depth: usize,
    children: ChildIncludeMode,
) -> Result<(Vec<PathBuf>, Vec<FileRow>)> {
    let paths = tokmd_scan::normalize_in_memory_paths(inputs)?;
    let config = tokmd_scan::config_from_scan_options(scan_opts);
    let row_inputs: Vec<tokmd_model::InMemoryRowInput<'_>> = paths
        .iter()
        .zip(inputs)
        .map(|(path, input)| {
            tokmd_model::InMemoryRowInput::new(path.as_path(), input.bytes.as_slice())
        })
        .collect();
    let rows = tokmd_model::collect_in_memory_file_rows(
        &row_inputs,
        module_roots,
        module_depth,
        children,
        &config,
    );
    Ok((paths, rows))
}

fn strip_virtual_export_prefix(
    rows: Vec<FileRow>,
    strip_prefix: &str,
    module_roots: &[String],
    module_depth: usize,
) -> Vec<FileRow> {
    rows.into_iter()
        .map(|mut row| {
            let normalized =
                tokmd_model::normalize_path(Path::new(&row.path), Some(Path::new(strip_prefix)));
            row.path = normalized.clone();
            row.module = tokmd_model::module_key(&normalized, module_roots, module_depth);
            row
        })
        .collect()
}

fn build_lang_receipt(
    paths: &[PathBuf],
    scan_opts: &ScanOptions,
    lang: &LangSettings,
    report: LangReport,
) -> LangReceipt {
    LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(paths, scan_opts, lang.redact),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: lang.top,
            with_files: lang.files,
            children: lang.children,
        },
        report,
    }
}

fn build_module_receipt(
    paths: &[PathBuf],
    scan_opts: &ScanOptions,
    module: &ModuleSettings,
    report: ModuleReport,
) -> ModuleReceipt {
    ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(paths, scan_opts, module.redact),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            top: module.top,
            module_roots: module.module_roots.clone(),
            module_depth: module.module_depth,
            children: module.children,
        },
        report,
    }
}

fn build_export_receipt(
    paths: &[PathBuf],
    scan_opts: &ScanOptions,
    export: &ExportSettings,
    data: ExportData,
) -> ExportReceipt {
    let should_redact = export.redact == RedactMode::Paths || export.redact == RedactMode::All;
    let strip_prefix_redacted = should_redact && export.strip_prefix.is_some();

    ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: scan_args(paths, scan_opts, Some(export.redact)),
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

/// Re-export analysis schema version for bindings.
#[cfg(feature = "analysis")]
pub const CORE_ANALYSIS_SCHEMA_VERSION: u32 = tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION;

/// Get the current tokmd version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "analysis")]
    use crate::settings::AnalyzeSettings;
    #[cfg(feature = "analysis")]
    use std::fs;
    #[cfg(feature = "analysis")]
    use std::path::{Path, PathBuf};
    #[cfg(feature = "analysis")]
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(feature = "analysis")]
    #[derive(Debug)]
    struct TempDirGuard(PathBuf);

    #[cfg(feature = "analysis")]
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

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

    #[cfg(feature = "analysis")]
    #[test]
    fn effort_request_defaults_to_estimate_preset() {
        let analyze = AnalyzeSettings {
            preset: "estimate".to_string(),
            ..Default::default()
        };
        let req = parse_effort_request(&analyze, "estimate").expect("parse effort request");
        let req = req.expect("estimate should imply effort request");
        assert_eq!(
            req.model.as_str(),
            analysis::EffortModelKind::Cocomo81Basic.as_str()
        );
        assert_eq!(req.layer.as_str(), analysis::EffortLayer::Full.as_str());
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn effort_request_not_implied_for_non_estimate_without_flags() {
        let analyze = AnalyzeSettings {
            preset: "receipt".to_string(),
            ..Default::default()
        };
        let req = parse_effort_request(&analyze, "receipt").expect("parse effort request");
        assert!(req.is_none());
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn effort_request_rejects_unsupported_model() {
        let analyze = AnalyzeSettings {
            preset: "estimate".to_string(),
            effort_model: Some("cocomo2-early".to_string()),
            ..Default::default()
        };
        let err =
            parse_effort_request(&analyze, "estimate").expect_err("unsupported model should fail");
        assert!(err.to_string().contains("only 'cocomo81-basic'"));
    }

    #[cfg(feature = "analysis")]
    fn mk_temp_dir(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut root = std::env::temp_dir();
        root.push(format!("{prefix}-{timestamp}-{}", std::process::id()));
        root
    }

    #[cfg(feature = "analysis")]
    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn analyze_workflow_estimate_preset_populates_effort_and_size_basis_breakdown() {
        let root = mk_temp_dir("tokmd-core-estimate-preset");
        let _guard = TempDirGuard(root.clone());
        write_file(&root.join("src/main.rs"), "fn main() {}\n");
        write_file(
            &root.join("target/generated/bundle.min.js"),
            "console.log(1);\n",
        );
        write_file(
            &root.join("vendor/lib/external.rs"),
            "pub fn external() {}\n",
        );

        let scan = settings::ScanSettings::for_paths(vec![root.display().to_string()]);
        let analyze = AnalyzeSettings {
            preset: "estimate".to_string(),
            ..Default::default()
        };

        let receipt = analyze_workflow(&scan, &analyze).expect("estimate analyze failed");
        let effort = receipt
            .effort
            .as_ref()
            .expect("estimate preset should produce effort");

        assert!(effort.results.effort_pm_p50 > 0.0);
        assert_eq!(
            effort.size_basis.total_lines,
            effort.size_basis.authored_lines
                + effort.size_basis.generated_lines
                + effort.size_basis.vendored_lines
        );
        assert!(effort.size_basis.authored_lines > 0);
        assert!(
            effort.size_basis.generated_lines + effort.size_basis.vendored_lines > 0,
            "expected deterministic generated or vendored lines"
        );
    }
}

// =============================================================================
// Mutation-killing tests for private functions
// These target surviving mutants identified in conveyor verification run.
// =============================================================================

#[cfg(test)]
mod mutation_tests {
    use super::*;
    use tokmd_settings::ExportSettings;
    use tokmd_types::ExportData;
    use tokmd_types::RedactMode;

    // Helper to create minimal ExportData
    fn empty_export_data() -> ExportData {
        ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 3,
            children: tokmd_types::ChildIncludeMode::Separate,
        }
    }

    // Helper to create minimal ScanOptions
    fn minimal_scan_opts() -> ScanOptions {
        ScanOptions {
            excluded: vec![],
            config: tokmd_types::ConfigMode::Auto,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        }
    }

    // Helper to create ExportSettings with specific redact/strip_prefix
    fn export_settings(redact: RedactMode, strip_prefix: Option<String>) -> ExportSettings {
        ExportSettings {
            format: tokmd_settings::ExportFormat::Json,
            module_roots: vec![],
            module_depth: 3,
            children: tokmd_types::ChildIncludeMode::Separate,
            min_code: 1,
            max_rows: 1000,
            redact,
            meta: true,
            strip_prefix,
        }
    }

    // =============================================================================
    // parse_analysis_preset — Kill 9/12 untested match arms
    // =============================================================================

    #[test]
    #[cfg(feature = "analysis")]
    fn parse_analysis_preset_all_twelve_variants() {
        #[cfg(feature = "analysis")]
        use tokmd_analysis::AnalysisPreset;

        let variants = [
            ("receipt", AnalysisPreset::Receipt),
            ("estimate", AnalysisPreset::Estimate),
            ("health", AnalysisPreset::Health),
            ("risk", AnalysisPreset::Risk),
            ("supply", AnalysisPreset::Supply),
            ("architecture", AnalysisPreset::Architecture),
            ("topics", AnalysisPreset::Topics),
            ("security", AnalysisPreset::Security),
            ("identity", AnalysisPreset::Identity),
            ("git", AnalysisPreset::Git),
            ("deep", AnalysisPreset::Deep),
            ("fun", AnalysisPreset::Fun),
        ];

        for (input, expected) in &variants {
            // Test exact lowercase
            let (preset, normalized) = parse_analysis_preset(input).unwrap();
            assert_eq!(preset, *expected, "Exact match failed for: {}", input);
            assert_eq!(normalized, *input, "Normalization failed for: {}", input);

            // Test uppercase (normalization)
            let upper = input.to_uppercase();
            let (preset, normalized) = parse_analysis_preset(&upper).unwrap();
            assert_eq!(preset, *expected, "Uppercase match failed for: {}", upper);
            assert_eq!(
                normalized, *input,
                "Uppercase normalization failed for: {}",
                upper
            );

            // Test mixed case with whitespace (normalization)
            let mixed = format!("  {}  ", input);
            let (preset, normalized) = parse_analysis_preset(&mixed).unwrap();
            assert_eq!(preset, *expected, "Mixed case match failed for: {}", mixed);
            assert_eq!(
                normalized, *input,
                "Mixed case normalization failed for: {}",
                mixed
            );
        }
    }

    #[test]
    #[cfg(feature = "analysis")]
    fn parse_analysis_preset_invalid_variants_fail() {
        let invalid = [
            "unknown",
            "invalid",
            "",
            "receipts",         // typo
            "healthh",          // typo
            "ARCH",             // partial match
            "receipt_estimate", // combined
        ];

        for input in &invalid {
            assert!(
                parse_analysis_preset(input).is_err(),
                "Should fail for invalid input: {}",
                input
            );
        }
    }

    // =============================================================================
    // build_export_receipt — Kill && → || mutation on strip_prefix_redacted
    // =============================================================================

    #[test]
    fn build_export_receipt_redact_paths_with_strip_prefix() {
        let settings = export_settings(RedactMode::Paths, Some("/project".to_string()));
        let data = empty_export_data();
        let paths = vec![PathBuf::from("/project/src/main.rs")];

        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        // strip_prefix_redacted = should_redact && strip_prefix.is_some()
        // = true && true = true
        assert!(
            receipt.args.strip_prefix_redacted,
            "strip_prefix_redacted should be true when redact=Paths and strip_prefix=Some"
        );
    }

    #[test]
    fn build_export_receipt_redact_paths_without_strip_prefix() {
        let settings = export_settings(RedactMode::Paths, None);
        let data = empty_export_data();
        let paths = vec![PathBuf::from("/project/src/main.rs")];

        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        // strip_prefix_redacted = should_redact && strip_prefix.is_some()
        // = true && false = false
        // This kills the && → || mutation (|| would give true)
        assert!(
            !receipt.args.strip_prefix_redacted,
            "strip_prefix_redacted should be false when strip_prefix=None (kills &&→||)"
        );
    }

    #[test]
    fn build_export_receipt_no_redact_with_strip_prefix() {
        let settings = export_settings(RedactMode::None, Some("/project".to_string()));
        let data = empty_export_data();
        let paths = vec![PathBuf::from("/project/src/main.rs")];

        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        // strip_prefix_redacted = should_redact && strip_prefix.is_some()
        // = false && true = false
        assert!(
            !receipt.args.strip_prefix_redacted,
            "strip_prefix_redacted should be false when redact=None"
        );
    }

    #[test]
    fn build_export_receipt_redact_all_with_strip_prefix() {
        let settings = export_settings(RedactMode::All, Some("/project".to_string()));
        let data = empty_export_data();
        let paths = vec![PathBuf::from("/project/src/main.rs")];

        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        // strip_prefix_redacted = should_redact && strip_prefix.is_some()
        // = true && true = true (All also triggers should_redact)
        assert!(
            receipt.args.strip_prefix_redacted,
            "strip_prefix_redacted should be true when redact=All and strip_prefix=Some"
        );
    }

    #[test]
    fn build_export_receipt_redact_all_without_strip_prefix() {
        let settings = export_settings(RedactMode::All, None);
        let data = empty_export_data();
        let paths = vec![PathBuf::from("/project/src/main.rs")];

        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        // strip_prefix_redacted = should_redact && strip_prefix.is_some()
        // = true && false = false
        // This kills the && → || mutation
        assert!(
            !receipt.args.strip_prefix_redacted,
            "strip_prefix_redacted should be false when strip_prefix=None (kills &&→||)"
        );
    }

    #[test]
    fn build_export_receipt_strip_prefix_redaction_logic() {
        // Test the ternary logic: strip_prefix redaction in ExportArgsMeta
        // Kills mutations that change the if/else logic on strip_prefix

        // Case 1: redact=Paths → strip_prefix should be redacted
        let settings = export_settings(RedactMode::Paths, Some("/project".to_string()));
        let data = empty_export_data();
        let paths = vec![PathBuf::from("/project/src/main.rs")];
        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        // When redacted, strip_prefix should be transformed (not the original)
        assert!(receipt.args.strip_prefix.is_some());
        assert_ne!(
            receipt.args.strip_prefix,
            Some("/project".to_string()),
            "strip_prefix should be redacted/transformed when redact=Paths"
        );

        // Case 2: redact=None → strip_prefix should pass through unchanged
        let settings = export_settings(RedactMode::None, Some("/project".to_string()));
        let data = empty_export_data();
        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        assert_eq!(
            receipt.args.strip_prefix,
            Some("/project".to_string()),
            "strip_prefix should pass through unchanged when redact=None"
        );

        // Case 3: redact=All → strip_prefix should be redacted
        let settings = export_settings(RedactMode::All, Some("/project".to_string()));
        let data = empty_export_data();
        let receipt = build_export_receipt(&paths, &minimal_scan_opts(), &settings, data);

        assert!(receipt.args.strip_prefix.is_some());
        assert_ne!(
            receipt.args.strip_prefix,
            Some("/project".to_string()),
            "strip_prefix should be redacted when redact=All"
        );
    }

    #[test]
    #[cfg(feature = "analysis")]
    fn parse_analysis_preset_normalization_edge_cases() {
        // Kills mutations that remove .trim() or .to_ascii_lowercase()

        // Test trim removal
        let (preset, _) = parse_analysis_preset("  receipt  ").unwrap();
        assert_eq!(
            preset,
            tokmd_analysis::AnalysisPreset::Receipt,
            "Leading/trailing whitespace should be trimmed"
        );

        let (preset, _) = parse_analysis_preset("\tHEALTH\n").unwrap();
        assert_eq!(
            preset,
            tokmd_analysis::AnalysisPreset::Health,
            "Tabs and newlines should be trimmed, case normalized"
        );

        // Test to_ascii_lowercase removal
        let (preset, _) = parse_analysis_preset("ReCeIpT").unwrap();
        assert_eq!(
            preset,
            tokmd_analysis::AnalysisPreset::Receipt,
            "Mixed case should be normalized to lowercase"
        );

        let (preset, _) = parse_analysis_preset("ESTIMATE").unwrap();
        assert_eq!(
            preset,
            tokmd_analysis::AnalysisPreset::Estimate,
            "Uppercase should be normalized"
        );

        // Test combined trim + lowercase
        let (preset, normalized) = parse_analysis_preset("  DeEp  ").unwrap();
        assert_eq!(preset, tokmd_analysis::AnalysisPreset::Deep);
        assert_eq!(normalized, "deep", "Should be trimmed and lowercased");
    }

    // =============================================================================
    // cockpit_workflow — Kill boolean logic mutations (requires git + cockpit feature)
    // =============================================================================

    #[cfg(feature = "cockpit")]
    #[test]
    fn cockpit_workflow_range_mode_parsing() {
        assert!(matches!(
            parse_cockpit_range_mode("three-dot").expect("three-dot should parse"),
            tokmd_git::GitRangeMode::ThreeDot
        ));
        assert!(matches!(
            parse_cockpit_range_mode("3dot").expect("3dot should parse"),
            tokmd_git::GitRangeMode::ThreeDot
        ));
        assert!(matches!(
            parse_cockpit_range_mode("two-dot").expect("two-dot should parse"),
            tokmd_git::GitRangeMode::TwoDot
        ));
        assert!(matches!(
            parse_cockpit_range_mode("2dot").expect("2dot should parse"),
            tokmd_git::GitRangeMode::TwoDot
        ));
        assert!(matches!(
            parse_cockpit_range_mode("  THREE-DOT  ").expect("trimmed/case-insensitive parse"),
            tokmd_git::GitRangeMode::ThreeDot
        ));
    }

    #[cfg(feature = "cockpit")]
    #[test]
    fn cockpit_workflow_range_mode_invalid_rejected() {
        let err = parse_cockpit_range_mode("invalid").expect_err("invalid mode should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("range_mode"),
            "Error should reference range_mode field; got: {msg}"
        );
    }
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub mod readme_doctests {}
