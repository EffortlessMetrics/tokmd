//! Feature gate correctness and "no green by omission" boundary tests.
//!
//! Verifies that the analysis pipeline never silently succeeds when optional
//! features are unavailable — every missing capability must be reported as
//! unavailable/skipped through warnings, and no enricher may produce empty
//! results while pretending it ran.

use std::path::PathBuf;
use tokmd_analysis::{AnalysisContext, AnalysisRequest, ImportGranularity, analyze};
use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisSource, NearDupScope,
};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// -- helpers --

fn sample_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 2,
        blanks: 1,
        lines: code + 3,
        bytes: code * 30,
        tokens: code * 5,
    }
}

fn sample_export() -> ExportData {
    ExportData {
        rows: vec![
            sample_row("src/main.rs", "src", "Rust", 100),
            sample_row("src/lib.rs", "src", "Rust", 200),
            sample_row("src/util.rs", "src", "Rust", 50),
            sample_row("tests/smoke.rs", "tests", "Rust", 30),
        ],
        module_roots: vec!["src".to_string(), "tests".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn make_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: Some(2),
        export_generated_at_ms: Some(0),
        base_signature: None,
        module_roots: vec!["src".to_string()],
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

fn make_req(preset: PresetKind) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: preset.as_str().to_string(),
            format: "json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "module".to_string(),
        },
        limits: Default::default(),
        window_tokens: None,
        git: None,
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.8,
        near_dup_max_files: 500,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: vec![],
    }
}

// -- 1. git feature gate tests --

/// When git feature is compiled in, risk preset should not have git=None
/// pretending it ran. When git is absent, warnings must be emitted.
#[test]
fn risk_preset_git_gate_no_silent_success() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Risk)).unwrap();
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git, "risk plan must request git enricher");

    #[cfg(not(feature = "git"))]
    {
        assert!(
            receipt.git.is_none(),
            "git must be None without git feature"
        );
        assert!(
            receipt
                .warnings
                .iter()
                .any(|w| w.contains(DisabledFeature::GitMetrics.warning())),
            "must warn about disabled git: {:?}",
            receipt.warnings
        );
    }

    #[cfg(feature = "git")]
    {
        let has_git = receipt.git.is_some();
        let has_git_warning = receipt.warnings.iter().any(|w| w.contains("git"));
        assert!(
            has_git || has_git_warning,
            "git feature compiled in but no git report and no git warning"
        );
    }
}

/// Git preset must request git and churn.
#[test]
fn git_preset_requests_git_and_churn() {
    let plan = preset_plan_for(PresetKind::Git);
    assert!(plan.git, "git preset must request git enricher");
    #[cfg(feature = "git")]
    assert!(plan.churn, "git preset must request churn with git feature");
}

/// Identity preset requests git + fingerprint.
#[test]
fn identity_preset_git_dependent_fields() {
    let plan = preset_plan_for(PresetKind::Identity);
    assert!(plan.git, "identity must request git");
    #[cfg(feature = "git")]
    assert!(
        plan.fingerprint,
        "identity must request fingerprint with git"
    );
}

/// Explicit git=false suppresses git even when plan requests it.
#[test]
fn git_false_override_suppresses_git_in_deep_preset() {
    let mut req = make_req(PresetKind::Deep);
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(
        receipt.git.is_none(),
        "explicit git=false must suppress git report even for deep preset"
    );
}

// -- 2. content feature gate tests --

/// Health preset requests content-dependent enrichers (todo, complexity).
#[test]
fn health_preset_content_gate_no_silent_success() {
    let _receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Health)).unwrap();
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo, "health plan must request TODO scan");
    assert!(plan.complexity, "health plan must request complexity");

    #[cfg(not(feature = "content"))]
    {
        assert!(
            receipt
                .warnings
                .iter()
                .any(|w| w.contains(DisabledFeature::TodoScan.warning())),
            "must warn about disabled TODO scan: {:?}",
            receipt.warnings
        );
        assert!(
            receipt
                .warnings
                .iter()
                .any(|w| w.contains(DisabledFeature::ComplexityAnalysis.warning())),
            "must warn about disabled complexity: {:?}",
            receipt.warnings
        );
    }
}

/// Architecture preset depends on content for imports and api_surface.
#[test]
fn architecture_preset_content_gate() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports, "architecture must request imports");
    assert!(plan.api_surface, "architecture must request api_surface");

    #[cfg(not(feature = "content"))]
    {
        let receipt = analyze(
            make_ctx(sample_export()),
            make_req(PresetKind::Architecture),
        )
        .unwrap();
        assert!(
            receipt
                .warnings
                .iter()
                .any(|w| w.contains(DisabledFeature::ImportScan.warning())),
            "must warn about disabled import scan"
        );
    }
}

/// Deep preset requests all content-dependent enrichers.
#[test]
fn deep_preset_requests_all_content_enrichers() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.todo, "deep must request todo");
    assert!(plan.dup, "deep must request dup");
    assert!(plan.imports, "deep must request imports");
    assert!(plan.entropy, "deep must request entropy");
    assert!(plan.complexity, "deep must request complexity");
    assert!(plan.api_surface, "deep must request api_surface");
}

// -- 3. walk feature gate tests --

/// Supply preset depends on walk for assets.
#[test]
fn supply_preset_walk_gate_no_silent_success() {
    let _receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Supply)).unwrap();
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets, "supply plan must request assets");

    #[cfg(not(feature = "walk"))]
    {
        assert!(
            receipt
                .warnings
                .iter()
                .any(|w| w.contains(DisabledFeature::FileInventory.warning())),
            "must warn about disabled file inventory: {:?}",
            receipt.warnings
        );
    }
}

/// Security preset depends on walk+content for entropy and license.
#[test]
fn security_preset_walk_content_gates() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy, "security must request entropy");
    assert!(plan.license, "security must request license");
}

// -- 4. receipt preset works without optional features --

/// Receipt preset must produce derived metrics without any optional features.
#[test]
fn receipt_preset_works_without_optional_features() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(!plan.git, "receipt must not request git");
    assert!(!plan.todo, "receipt must not request todo");
    assert!(!plan.dup, "receipt must not request dup");
    assert!(!plan.imports, "receipt must not request imports");
    assert!(!plan.entropy, "receipt must not request entropy");
    assert!(!plan.assets, "receipt must not request assets");
    assert!(!plan.license, "receipt must not request license");
    assert!(!plan.complexity, "receipt must not request complexity");
    assert!(!plan.api_surface, "receipt must not request api_surface");
    assert!(!plan.fun, "receipt must not request fun");

    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert!(
        receipt.derived.is_some(),
        "receipt must always produce derived"
    );
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert!(
        matches!(receipt.status, ScanStatus::Complete),
        "receipt preset should be complete without optional features"
    );
}

/// Receipt preset with empty export still produces derived (no panic).
#[test]
fn receipt_preset_empty_export_still_produces_derived() {
    let empty = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = analyze(make_ctx(empty), make_req(PresetKind::Receipt)).unwrap();
    assert!(receipt.derived.is_some());
}

// -- 5. health preset with content but without git --

/// Health preset should not require git at all.
#[test]
fn health_preset_does_not_request_git() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(!plan.git, "health must not request git");

    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Health)).unwrap();
    assert!(receipt.git.is_none(), "health must not produce git report");
    assert!(receipt.derived.is_some(), "health must include derived");
}

// -- 6. risk preset graceful degradation without git --

/// Risk preset with git=false degrades to just complexity + derived.
#[test]
fn risk_preset_degrades_gracefully_without_git() {
    let mut req = make_req(PresetKind::Risk);
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.git.is_none(), "git must be absent when git=false");
    assert!(
        receipt.derived.is_some(),
        "derived must still be present on degradation"
    );
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// -- 7. capability reporting accuracy per preset --

/// Every preset in the grid has a corresponding plan via preset_plan_for.
#[test]
fn every_preset_has_a_plan() {
    for kind in PresetKind::all() {
        let plan = preset_plan_for(*kind);
        let _ = plan.needs_files();
    }
}

/// preset_plan_for_name accepts all canonical preset names.
#[test]
fn preset_plan_for_name_accepts_all_canonical_names() {
    let names = [
        "receipt",
        "health",
        "risk",
        "supply",
        "architecture",
        "topics",
        "security",
        "identity",
        "git",
        "deep",
        "fun",
    ];
    for name in names {
        assert!(
            preset_plan_for_name(name).is_some(),
            "preset_plan_for_name must accept '{name}'"
        );
    }
}

/// preset_plan_for_name rejects unknown names.
#[test]
fn preset_plan_for_name_rejects_unknown() {
    assert!(preset_plan_for_name("nonexistent").is_none());
    assert!(preset_plan_for_name("").is_none());
    assert!(preset_plan_for_name("RECEIPT").is_none());
}

/// needs_files is true for any preset that requests file-level enrichers.
#[test]
fn needs_files_consistent_with_plan_flags() {
    for row in &PRESET_GRID {
        let plan = row.plan;
        let any_file_enricher = plan.assets
            || plan.deps
            || plan.todo
            || plan.dup
            || plan.imports
            || plan.entropy
            || plan.license
            || plan.complexity
            || plan.api_surface;
        if any_file_enricher {
            assert!(
                plan.needs_files(),
                "{:?} has file-level enrichers but needs_files() is false",
                row.preset
            );
        }
    }
}

// -- 8. receipt metadata accuracy --

/// Analysis receipt always includes schema_version, tool name, and mode.
#[test]
fn receipt_metadata_always_present() {
    for preset in &[PresetKind::Receipt, PresetKind::Health, PresetKind::Deep] {
        let receipt = analyze(make_ctx(sample_export()), make_req(*preset)).unwrap();
        assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
        assert_eq!(receipt.tool.name, "tokmd");
        assert_eq!(receipt.mode, "analysis");
    }
}

/// Warnings array is always present (even if empty) in every receipt.
#[test]
fn warnings_array_always_present() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    let _ = receipt.warnings.len();
}

/// Status must be one of Complete or Partial.
#[test]
fn status_is_complete_or_partial_for_all_presets() {
    for preset in &[PresetKind::Receipt, PresetKind::Health, PresetKind::Supply] {
        let receipt = analyze(make_ctx(sample_export()), make_req(*preset)).unwrap();
        match receipt.status {
            ScanStatus::Complete | ScanStatus::Partial => {}
        }
    }
}

// -- disabled feature warning contract tests --

/// Every DisabledFeature warning must contain "disabled" or "skipping".
#[test]
fn disabled_feature_warnings_contain_required_keywords() {
    let all_features = [
        DisabledFeature::FileInventory,
        DisabledFeature::TodoScan,
        DisabledFeature::DuplicationScan,
        DisabledFeature::NearDuplicateScan,
        DisabledFeature::ImportScan,
        DisabledFeature::GitMetrics,
        DisabledFeature::EntropyProfiling,
        DisabledFeature::LicenseRadar,
        DisabledFeature::ComplexityAnalysis,
        DisabledFeature::ApiSurfaceAnalysis,
        DisabledFeature::Archetype,
        DisabledFeature::Topics,
        DisabledFeature::Fun,
    ];
    for f in all_features {
        let w = f.warning();
        assert!(
            w.contains("disabled") || w.contains("skipping"),
            "{f:?} warning must mention 'disabled' or 'skipping': {w}"
        );
    }
}

/// Git-related DisabledFeature mentions "git".
#[test]
fn git_disabled_feature_mentions_git() {
    let w = DisabledFeature::GitMetrics.warning();
    assert!(
        w.contains("git"),
        "GitMetrics warning must mention git: {w}"
    );
}

/// Walk-related DisabledFeature mentions "walk".
#[test]
fn walk_disabled_feature_mentions_walk() {
    let w = DisabledFeature::FileInventory.warning();
    assert!(
        w.contains("walk"),
        "FileInventory warning must mention walk: {w}"
    );
}

/// Content-related DisabledFeatures mention "content".
#[test]
fn content_disabled_features_mention_content() {
    let content_features = [
        DisabledFeature::TodoScan,
        DisabledFeature::DuplicationScan,
        DisabledFeature::NearDuplicateScan,
        DisabledFeature::ImportScan,
    ];
    for f in content_features {
        let w = f.warning();
        assert!(
            w.contains("content"),
            "{f:?} warning must mention content: {w}"
        );
    }
}

/// Grid has exactly 11 presets matching PRESET_KINDS.
#[test]
fn grid_covers_all_preset_kinds() {
    assert_eq!(PRESET_GRID.len(), PRESET_KINDS.len());
    for kind in &PRESET_KINDS {
        assert!(
            PRESET_GRID.iter().any(|row| row.preset == *kind),
            "PRESET_GRID must cover {kind:?}"
        );
    }
}

/// Deep preset enables strictly more enrichers than receipt.
#[test]
fn deep_is_superset_of_receipt() {
    let receipt = preset_plan_for(PresetKind::Receipt);
    let deep = preset_plan_for(PresetKind::Deep);

    if receipt.assets {
        assert!(deep.assets);
    }
    if receipt.todo {
        assert!(deep.todo);
    }
    if receipt.git {
        assert!(deep.git);
    }

    let deep_count = [
        deep.assets,
        deep.deps,
        deep.todo,
        deep.dup,
        deep.imports,
        deep.git,
        deep.entropy,
        deep.license,
        deep.complexity,
        deep.api_surface,
    ]
    .iter()
    .filter(|&&b| b)
    .count();
    let receipt_count = [
        receipt.assets,
        receipt.deps,
        receipt.todo,
        receipt.dup,
        receipt.imports,
        receipt.git,
        receipt.entropy,
        receipt.license,
        receipt.complexity,
        receipt.api_surface,
    ]
    .iter()
    .filter(|&&b| b)
    .count();
    assert!(
        deep_count > receipt_count,
        "deep ({deep_count}) must enable more enrichers than receipt ({receipt_count})"
    );
}
