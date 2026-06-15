//! BDD-style scenario tests for tokmd-analysis orchestration.
//!
//! Each test reads as a Given/When/Then scenario covering the orchestration
//! of derived metrics, presets, and limits across multiple files and modules.

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ============================================================================
// Helpers
// ============================================================================

fn file_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.into(),
        module: module.into(),
        lang: lang.into(),
        kind: FileKind::Parent,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 40,
        tokens: code * 3,
    }
}

fn make_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: "separate".to_string(),
    }
}

fn make_ctx(export: ExportData) -> AnalysisContext {
    AnalysisContext {
        export,
        root: PathBuf::from("."),
        source: make_source(),
    }
}

fn make_req(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: format!("{:?}", preset).to_lowercase(),
            format: "json".to_string(),
            window_tokens: None,
            git: Some(false),
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        #[cfg(feature = "effort")]
        effort: None,
        window_tokens: None,
        git: Some(false),
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.8,
        near_dup_max_files: 2000,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    }
}

// ============================================================================
// Scenarios
// ============================================================================

#[test]
fn scenario_multi_language_polyglot_and_distribution() {
    // Given
    let export = ExportData {
        rows: vec![
            file_row("backend/src/main.rs", "backend", "Rust", 500),
            file_row("backend/src/lib.rs", "backend", "Rust", 800),
            file_row("frontend/src/index.ts", "frontend", "TypeScript", 400),
            file_row("frontend/src/app.ts", "frontend", "TypeScript", 600),
            file_row("scripts/deploy.sh", "scripts", "Shell", 50),
        ],
        module_roots: vec!["backend".to_string(), "frontend".to_string(), "scripts".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    // When
    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt))
        .expect("Analysis should succeed");

    // Then
    let derived = receipt.derived.expect("Derived metrics should be present");
    assert_eq!(derived.totals.files, 5);

    // Polyglot should identify dominant language
    assert!(derived.polyglot.entropy >= 0.0);
    assert_eq!(derived.polyglot.dominant_lang, "Rust"); // 1300 vs 1000 vs 50

    // Distribution metrics should capture all files
    assert_eq!(derived.distribution.count, 5);
    assert!(derived.distribution.gini > 0.0);
    assert!(derived.distribution.median > 0.0);
}

#[test]
fn scenario_analysis_limits_guardrails() {
    // Given
    let export = ExportData {
        rows: vec![
            file_row("src/a.rs", "src", "Rust", 1000),
            file_row("src/b.rs", "src", "Rust", 1000),
            file_row("src/c.rs", "src", "Rust", 1000),
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    // When: Applying strict limits
    let mut req = make_req(AnalysisPreset::Receipt);
    req.limits.max_files = Some(2); // Artificially low limit

    let receipt = analyze(make_ctx(export), req).expect("Analysis should succeed");

    // Then
    let derived = receipt.derived.unwrap();
    // Derived totals always reflect full input data, but limits control file I/O for enrichers
    assert_eq!(derived.totals.files, 3);
}

#[test]
fn scenario_context_window_fitting() {
    // Given
    let export = ExportData {
        rows: vec![file_row("src/main.rs", "src", "Rust", 500)],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    // When: Requesting context window bounds that easily fit
    let mut req = make_req(AnalysisPreset::Receipt);
    req.window_tokens = Some(100_000);

    let receipt = analyze(make_ctx(export.clone()), req).expect("Analysis should succeed");

    // Then
    let derived = receipt.derived.unwrap();
    let ctx = derived.context_window.expect("Context window should be populated");
    assert!(ctx.fits, "Total tokens should fit within window");
    assert_eq!(ctx.window_tokens, 100_000);

    // When: Requesting a tiny context window that does not fit
    let mut req = make_req(AnalysisPreset::Receipt);
    req.window_tokens = Some(10); // Very small window

    let receipt = analyze(make_ctx(export), req).expect("Analysis should succeed");

    // Then
    let derived = receipt.derived.unwrap();
    let ctx = derived.context_window.unwrap();
    assert!(!ctx.fits, "Tokens should exceed window");
}

#[test]
fn scenario_missing_enrichers_for_disabled_features() {
    // Given
    let export = ExportData {
        rows: vec![file_row("src/main.rs", "src", "Rust", 500)],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    // When: Requesting full Deep analysis which requires optional features
    let req = make_req(AnalysisPreset::Deep);

    let receipt = analyze(make_ctx(export), req).expect("Analysis should succeed");

    // Then: If features are disabled, we get a partial status and warnings
    if !cfg!(all(feature = "content", feature = "walk")) {
        assert_eq!(receipt.status, ScanStatus::Partial);
        assert!(!receipt.warnings.is_empty(), "Warnings should be generated for missing features");

        // Derived metrics are always generated
        assert!(receipt.derived.is_some());
    } else {
        // If all features enabled, it might be complete or partial depending on git
        assert!(receipt.derived.is_some());
    }
}
