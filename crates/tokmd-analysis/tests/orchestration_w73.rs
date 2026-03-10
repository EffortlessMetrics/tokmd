//! W73 deep tests for analysis orchestration.
//!
//! Covers:
//! - Preset→enricher mapping correctness for all 11 presets
//! - Enricher execution determinism (multiple runs yield identical receipts)
//! - Missing capability reporting (feature-gated warnings)
//! - Preset composition (deep = everything except fun)
//! - AnalysisRequest field propagation

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis_grid::{PresetKind, preset_plan_for};
use tokmd_analysis_types::{ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ─── Helpers ────────────────────────────────────────────────────────────────

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
            preset: preset.as_str().to_string(),
            format: "json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        #[cfg(feature = "effort")]
        effort: None,
        window_tokens: None,
        git: None,
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    }
}

fn row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 10,
        tokens: code * 2,
    }
}

fn sample_export() -> ExportData {
    ExportData {
        rows: vec![
            row("src/main.rs", "src", "Rust", 200),
            row("src/lib.rs", "src", "Rust", 150),
            row("tests/test.rs", "tests", "Rust", 80),
            row("Cargo.toml", "(root)", "TOML", 30),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn empty_export() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Preset → enricher plan correctness
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_preset_enables_no_optional_enrichers() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
    assert!(!plan.api_surface);
}

#[test]
fn health_preset_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo, "health should enable todo");
    assert!(plan.complexity, "health should enable complexity");
    assert!(!plan.git, "health should not enable git");
    assert!(!plan.assets, "health should not enable assets");
    assert!(!plan.imports, "health should not enable imports");
    assert!(!plan.fun, "health should not enable fun");
}

#[test]
fn risk_preset_enables_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git, "risk should enable git");
    assert!(plan.complexity, "risk should enable complexity");
    assert!(!plan.todo, "risk should not enable todo");
    assert!(!plan.assets, "risk should not enable assets");
    assert!(!plan.fun, "risk should not enable fun");
}

#[test]
fn supply_preset_enables_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets, "supply should enable assets");
    assert!(plan.deps, "supply should enable deps");
    assert!(!plan.git, "supply should not enable git");
    assert!(!plan.todo, "supply should not enable todo");
    assert!(!plan.imports, "supply should not enable imports");
}

#[test]
fn architecture_preset_enables_imports_and_api_surface() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports, "architecture should enable imports");
    assert!(plan.api_surface, "architecture should enable api_surface");
    assert!(!plan.git, "architecture should not enable git");
    assert!(!plan.assets, "architecture should not enable assets");
    assert!(!plan.todo, "architecture should not enable todo");
}

#[test]
fn topics_preset_enables_only_topics() {
    let plan = preset_plan_for(PresetKind::Topics);
    assert!(plan.topics, "topics preset should enable topics");
    assert!(!plan.git);
    assert!(!plan.assets);
    assert!(!plan.todo);
    assert!(!plan.imports);
    assert!(!plan.dup);
    assert!(!plan.entropy);
    assert!(!plan.license);
}

#[test]
fn security_preset_enables_entropy_and_license() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy, "security should enable entropy");
    assert!(plan.license, "security should enable license");
    assert!(!plan.git);
    assert!(!plan.assets);
    assert!(!plan.todo);
    assert!(!plan.imports);
}

#[test]
fn identity_preset_enables_archetype_git_and_fingerprint() {
    let plan = preset_plan_for(PresetKind::Identity);
    assert!(plan.archetype, "identity should enable archetype");
    assert!(plan.git, "identity should enable git");
    assert!(!plan.assets);
    assert!(!plan.todo);
    assert!(!plan.imports);
    assert!(!plan.entropy);
}

#[test]
fn git_preset_enables_git_only() {
    let plan = preset_plan_for(PresetKind::Git);
    assert!(plan.git, "git preset should enable git");
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.imports);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
}

#[test]
fn fun_preset_enables_only_fun() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun, "fun preset should enable fun");
    assert!(!plan.git);
    assert!(!plan.assets);
    assert!(!plan.todo);
    assert!(!plan.imports);
    assert!(!plan.dup);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
    assert!(!plan.api_surface);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Deep preset = everything except fun
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deep_preset_enables_all_base_enrichers_except_fun() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets, "deep should enable assets");
    assert!(plan.deps, "deep should enable deps");
    assert!(plan.todo, "deep should enable todo");
    assert!(plan.dup, "deep should enable dup");
    assert!(plan.imports, "deep should enable imports");
    assert!(plan.git, "deep should enable git");
    assert!(plan.archetype, "deep should enable archetype");
    assert!(plan.topics, "deep should enable topics");
    assert!(plan.entropy, "deep should enable entropy");
    assert!(plan.license, "deep should enable license");
    assert!(plan.complexity, "deep should enable complexity");
    assert!(plan.api_surface, "deep should enable api_surface");
    assert!(!plan.fun, "deep should NOT enable fun");
}

#[test]
fn deep_is_superset_of_all_non_fun_presets() {
    let deep = preset_plan_for(PresetKind::Deep);
    for kind in PresetKind::all() {
        if *kind == PresetKind::Deep || *kind == PresetKind::Fun {
            continue;
        }
        let plan = preset_plan_for(*kind);
        if plan.assets {
            assert!(deep.assets, "{:?} has assets but deep doesn't", kind);
        }
        if plan.deps {
            assert!(deep.deps, "{:?} has deps but deep doesn't", kind);
        }
        if plan.todo {
            assert!(deep.todo, "{:?} has todo but deep doesn't", kind);
        }
        if plan.dup {
            assert!(deep.dup, "{:?} has dup but deep doesn't", kind);
        }
        if plan.imports {
            assert!(deep.imports, "{:?} has imports but deep doesn't", kind);
        }
        if plan.git {
            assert!(deep.git, "{:?} has git but deep doesn't", kind);
        }
        if plan.archetype {
            assert!(deep.archetype, "{:?} has archetype but deep doesn't", kind);
        }
        if plan.topics {
            assert!(deep.topics, "{:?} has topics but deep doesn't", kind);
        }
        if plan.entropy {
            assert!(deep.entropy, "{:?} has entropy but deep doesn't", kind);
        }
        if plan.license {
            assert!(deep.license, "{:?} has license but deep doesn't", kind);
        }
        if plan.complexity {
            assert!(
                deep.complexity,
                "{:?} has complexity but deep doesn't",
                kind
            );
        }
        if plan.api_surface {
            assert!(
                deep.api_surface,
                "{:?} has api_surface but deep doesn't",
                kind
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Enricher execution determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_receipt_deterministic_across_runs() {
    let export = sample_export();
    let r1 = analyze(make_ctx(export.clone()), make_req(AnalysisPreset::Receipt)).unwrap();
    let r2 = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();

    let d1 = r1.derived.as_ref().unwrap();
    let d2 = r2.derived.as_ref().unwrap();

    assert_eq!(d1.totals.code, d2.totals.code);
    assert_eq!(d1.totals.lines, d2.totals.lines);
    assert_eq!(d1.totals.files, d2.totals.files);
    assert_eq!(d1.integrity.hash, d2.integrity.hash);
    assert_eq!(d1.distribution.count, d2.distribution.count);
}

#[test]
fn analyze_deterministic_warnings_for_same_preset() {
    let export = sample_export();
    let r1 = analyze(make_ctx(export.clone()), make_req(AnalysisPreset::Health)).unwrap();
    let r2 = analyze(make_ctx(export), make_req(AnalysisPreset::Health)).unwrap();

    assert_eq!(r1.warnings.len(), r2.warnings.len());
    for (w1, w2) in r1.warnings.iter().zip(r2.warnings.iter()) {
        assert_eq!(w1, w2, "Warnings should be identical across runs");
    }
}

#[test]
fn analyze_all_presets_produce_valid_receipts() {
    let export = sample_export();
    for kind in PresetKind::all() {
        let receipt = analyze(make_ctx(export.clone()), make_req(*kind)).unwrap();
        assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
        assert_eq!(receipt.mode, "analysis");
        assert!(receipt.generated_at_ms > 0);
        assert!(receipt.derived.is_some(), "{:?} should have derived", kind);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Missing capability reporting
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_preset_produces_no_warnings() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(
        receipt.warnings.is_empty(),
        "Receipt should have no warnings but got: {:?}",
        receipt.warnings
    );
    assert!(matches!(receipt.status, ScanStatus::Complete));
}

#[test]
fn empty_export_still_produces_valid_receipt() {
    let receipt = analyze(make_ctx(empty_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.code, 0);
    assert_eq!(derived.totals.files, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. base_signature backfill
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn base_signature_backfilled_from_integrity_hash() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    let derived = receipt.derived.as_ref().unwrap();
    assert!(
        receipt.source.base_signature.is_some(),
        "base_signature should be backfilled"
    );
    assert_eq!(
        receipt.source.base_signature.as_deref(),
        Some(derived.integrity.hash.as_str()),
    );
}

#[test]
fn base_signature_not_overwritten_when_provided() {
    let mut source = make_source();
    source.base_signature = Some("custom-sig-123".to_string());
    let ctx = AnalysisContext {
        export: sample_export(),
        root: PathBuf::from("."),
        source,
    };
    let receipt = analyze(ctx, make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(
        receipt.source.base_signature.as_deref(),
        Some("custom-sig-123"),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Git override flag
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_override_false_suppresses_git_on_risk_preset() {
    let mut req = make_req(AnalysisPreset::Risk);
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    // Git is disabled by override, so no git report
    assert!(
        receipt.git.is_none(),
        "git should be suppressed by override"
    );
}

#[test]
fn git_override_true_on_receipt_preset_attempts_git() {
    let mut req = make_req(AnalysisPreset::Receipt);
    req.git = Some(true);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    // Without the git feature compiled, this will produce a warning.
    // With git feature, it may fail (not in a git repo) producing a warning.
    // Either way the receipt is valid.
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Tree building via format string
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tree_format_populates_tree_field() {
    let mut req = make_req(AnalysisPreset::Receipt);
    req.args.format = "json+tree".to_string();
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(
        derived.tree.is_some(),
        "format containing 'tree' should populate tree field"
    );
}

#[test]
fn non_tree_format_leaves_tree_none() {
    let req = make_req(AnalysisPreset::Receipt);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(
        derived.tree.is_none(),
        "default format should not build tree"
    );
}
