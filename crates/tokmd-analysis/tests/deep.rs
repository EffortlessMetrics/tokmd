//! Deep orchestration tests for the analysis crate.
//!
//! Covers areas not addressed by existing test files:
//! - Near-duplicate request path (req.near_dup = true/false)
//! - Warning message content matches DisabledFeature catalog
//! - Concurrent (multi-threaded) determinism
//! - Integrity hash uniqueness across different inputs
//! - ToolInfo presence in receipt
//! - Receipt size monotonicity (Deep > Receipt)
//! - AnalysisLimits propagation
//! - Deep nesting / many-module directory shapes
//! - Context window boundary conditions
//! - Schema version sanity invariants
//! - Serialization round-trip of AnalysisArgsMeta
//! - Preset plan field-level truth-table spot checks

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
            preset: format!("{:?}", preset).to_lowercase(),
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

fn run_preset(export: ExportData, preset: AnalysisPreset) -> tokmd_analysis_types::AnalysisReceipt {
    let mut req = make_req(preset);
    req.git = Some(false);
    analyze(make_ctx(export), req).expect("analyze should not fail")
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Near-duplicate request path
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn near_dup_disabled_produces_no_dup_section_on_receipt() {
    let mut req = make_req(AnalysisPreset::Receipt);
    req.near_dup = false;
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(
        receipt.dup.is_none(),
        "Receipt preset with near_dup=false should have no dup section"
    );
}

#[test]
fn near_dup_enabled_does_not_panic_on_receipt_preset() {
    // near_dup=true on Receipt preset: content feature may or may not be
    // enabled, but it should never panic.
    let mut req = make_req(AnalysisPreset::Receipt);
    req.near_dup = true;
    req.near_dup_threshold = 0.90;
    req.near_dup_max_files = 100;
    req.near_dup_scope = NearDupScope::Global;
    req.near_dup_max_pairs = Some(50);
    req.near_dup_exclude = vec!["*.lock".to_string()];
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    // Without content feature, a warning is emitted; with it, dup section is populated.
    assert!(receipt.derived.is_some());
}

#[test]
fn near_dup_enabled_on_all_presets_never_panics() {
    for preset in PresetKind::all() {
        let mut req = make_req(*preset);
        req.git = Some(false);
        req.near_dup = true;
        req.near_dup_threshold = 0.75;
        req.near_dup_max_files = 500;
        let _ = analyze(make_ctx(sample_export()), req)
            .unwrap_or_else(|e| panic!("near_dup on {:?} panicked: {}", preset, e));
    }
}

#[test]
fn near_dup_scope_variants_do_not_panic() {
    for scope in [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global,
    ] {
        let mut req = make_req(AnalysisPreset::Receipt);
        req.near_dup = true;
        req.near_dup_scope = scope;
        let _ = analyze(make_ctx(sample_export()), req).unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Warning message content matches DisabledFeature catalog
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn warnings_from_disabled_features_match_catalog() {
    use tokmd_analysis_grid::DisabledFeature;

    // Deep preset triggers every enricher → without all features, every
    // disabled feature should appear as a warning.
    let receipt = run_preset(sample_export(), AnalysisPreset::Deep);

    let known_warnings: Vec<&str> = vec![
        DisabledFeature::FileInventory.warning(),
        DisabledFeature::TodoScan.warning(),
        DisabledFeature::DuplicationScan.warning(),
        DisabledFeature::ImportScan.warning(),
        DisabledFeature::GitMetrics.warning(),
        DisabledFeature::EntropyProfiling.warning(),
        DisabledFeature::LicenseRadar.warning(),
        DisabledFeature::ComplexityAnalysis.warning(),
        DisabledFeature::ApiSurfaceAnalysis.warning(),
    ];

    // Every warning in the receipt should either be a known catalog message
    // or a runtime error message (starts with a known prefix).
    for warning in &receipt.warnings {
        let is_catalog = known_warnings.iter().any(|k| warning == k);
        let is_runtime = warning.contains("failed:")
            || warning.contains("not a git repo")
            || warning.contains("feature disabled")
            || warning.contains("feature is disabled");
        assert!(
            is_catalog || is_runtime,
            "Unexpected warning not in catalog: {:?}",
            warning
        );
    }
}

#[test]
fn receipt_preset_emits_no_disabled_feature_warnings() {
    // Receipt preset asks for nothing optional → no feature-gated warnings.
    let receipt = run_preset(sample_export(), AnalysisPreset::Receipt);
    assert!(
        receipt.warnings.is_empty(),
        "Receipt should emit zero warnings, got: {:?}",
        receipt.warnings
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Concurrent (multi-threaded) determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn concurrent_analyze_calls_produce_identical_results() {
    use std::thread;

    let export = sample_export();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let ex = export.clone();
            thread::spawn(move || {
                let receipt = run_preset(ex, AnalysisPreset::Receipt);
                let mut val = serde_json::to_value(receipt).unwrap();
                // Strip volatile fields
                if let Some(obj) = val.as_object_mut() {
                    obj.remove("generated_at_ms");
                    obj.remove("tool");
                }
                val
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Thread 0 and thread {} produced different results",
            i
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Integrity hash uniqueness across different inputs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn different_exports_produce_different_integrity_hashes() {
    let export_a = ExportData {
        rows: vec![row("a.rs", "src", "Rust", 100)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let export_b = ExportData {
        rows: vec![row("b.rs", "src", "Rust", 100)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let r_a = run_preset(export_a, AnalysisPreset::Receipt);
    let r_b = run_preset(export_b, AnalysisPreset::Receipt);

    let h_a = &r_a.derived.unwrap().integrity.hash;
    let h_b = &r_b.derived.unwrap().integrity.hash;
    assert_ne!(
        h_a, h_b,
        "different file paths should produce different hashes"
    );
}

#[test]
fn different_code_counts_produce_different_integrity_hashes() {
    let export_a = ExportData {
        rows: vec![row("a.rs", "src", "Rust", 100)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let export_b = ExportData {
        rows: vec![row("a.rs", "src", "Rust", 101)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let h_a = &run_preset(export_a, AnalysisPreset::Receipt)
        .derived
        .unwrap()
        .integrity
        .hash;
    let h_b = &run_preset(export_b, AnalysisPreset::Receipt)
        .derived
        .unwrap()
        .integrity
        .hash;
    assert_ne!(
        h_a, h_b,
        "different code counts should yield different hashes"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. ToolInfo presence in receipt
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tool_info_is_populated() {
    let receipt = run_preset(sample_export(), AnalysisPreset::Receipt);
    assert!(
        !receipt.tool.name.is_empty(),
        "tool name should be populated"
    );
    assert!(
        !receipt.tool.version.is_empty(),
        "tool version should be populated"
    );
}

#[test]
fn tool_info_is_consistent_across_presets() {
    let r1 = run_preset(sample_export(), AnalysisPreset::Receipt);
    let r2 = run_preset(sample_export(), AnalysisPreset::Deep);
    assert_eq!(r1.tool.name, r2.tool.name);
    assert_eq!(r1.tool.version, r2.tool.version);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Receipt size monotonicity: Deep JSON ≥ Receipt JSON
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deep_receipt_json_is_at_least_as_large_as_receipt() {
    let export = sample_export();
    let receipt_json =
        serde_json::to_string(&run_preset(export.clone(), AnalysisPreset::Receipt)).unwrap();
    let deep_json = serde_json::to_string(&run_preset(export, AnalysisPreset::Deep)).unwrap();
    assert!(
        deep_json.len() >= receipt_json.len(),
        "Deep JSON ({} bytes) should be >= Receipt JSON ({} bytes)",
        deep_json.len(),
        receipt_json.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. AnalysisLimits propagation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn custom_limits_do_not_crash() {
    let mut req = make_req(AnalysisPreset::Deep);
    req.git = Some(false);
    req.limits = AnalysisLimits {
        max_files: Some(5),
        max_bytes: Some(1024),
        max_file_bytes: Some(256),
        max_commits: Some(10),
        max_commit_files: Some(5),
    };
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.derived.is_some());
}

#[test]
fn zero_max_files_limit_does_not_panic() {
    let mut req = make_req(AnalysisPreset::Deep);
    req.git = Some(false);
    req.limits = AnalysisLimits {
        max_files: Some(0),
        max_bytes: Some(0),
        max_file_bytes: Some(0),
        max_commits: Some(0),
        max_commit_files: Some(0),
    };
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.derived.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Deep nesting / many-module directory shapes
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deeply_nested_modules_produce_valid_receipt() {
    let export = ExportData {
        rows: vec![
            row("a/b/c/d/e/f.rs", "a/b/c/d/e", "Rust", 10),
            row("a/b/c/d/g.rs", "a/b/c/d", "Rust", 20),
            row("a/b/c/h.rs", "a/b/c", "Rust", 30),
            row("x/y/z.py", "x/y", "Python", 40),
        ],
        module_roots: vec!["a".to_string()],
        module_depth: 5,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 4);
    assert_eq!(derived.totals.code, 100);
    assert_eq!(derived.polyglot.lang_count, 2);
}

#[test]
fn many_modules_produce_valid_receipt() {
    let rows: Vec<FileRow> = (0..50)
        .map(|i| {
            row(
                &format!("mod_{}/main.rs", i),
                &format!("mod_{}", i),
                "Rust",
                10 + i,
            )
        })
        .collect();
    let expected_code: usize = (0..50).map(|i| 10 + i).sum();
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 50);
    assert_eq!(derived.totals.code, expected_code);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Context window boundary conditions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn context_window_fits_when_tokens_within_budget() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "small.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 100,
            tokens: 20,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut req = make_req(AnalysisPreset::Receipt);
    req.window_tokens = Some(100_000);

    let receipt = analyze(make_ctx(export), req).unwrap();
    let cw = receipt.derived.unwrap().context_window.unwrap();
    assert!(cw.fits, "20 tokens should fit in 100k window");
    assert!(cw.pct <= 1.0);
}

#[test]
fn context_window_exactly_at_boundary() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "exact.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 500,
            comments: 0,
            blanks: 0,
            lines: 500,
            bytes: 5000,
            tokens: 1000,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut req = make_req(AnalysisPreset::Receipt);
    req.window_tokens = Some(1000); // exactly equals tokens

    let receipt = analyze(make_ctx(export), req).unwrap();
    let cw = receipt.derived.unwrap().context_window.unwrap();
    // 1000 tokens with 1000 window → pct = 1.0, fits = true
    assert!(cw.fits, "exact boundary should fit");
    assert!((cw.pct - 1.0).abs() < f64::EPSILON);
}

#[test]
fn context_window_with_zero_tokens_fits_any_window() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut req = make_req(AnalysisPreset::Receipt);
    req.window_tokens = Some(128);

    let receipt = analyze(make_ctx(export), req).unwrap();
    let cw = receipt.derived.unwrap().context_window.unwrap();
    assert!(cw.fits, "zero tokens should fit any window");
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Schema version sanity invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn schema_version_is_positive() {
    const { assert!(ANALYSIS_SCHEMA_VERSION > 0); }
}

#[test]
fn schema_version_matches_across_all_presets() {
    for preset in PresetKind::all() {
        let receipt = run_preset(sample_export(), *preset);
        assert_eq!(
            receipt.schema_version, ANALYSIS_SCHEMA_VERSION,
            "preset {:?} emitted wrong schema version",
            preset
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Serialization round-trip of AnalysisArgsMeta
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn args_meta_json_roundtrip() {
    let args = AnalysisArgsMeta {
        preset: "deep".to_string(),
        format: "json".to_string(),
        window_tokens: Some(128_000),
        git: Some(true),
        max_files: Some(10_000),
        max_bytes: Some(50_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(500),
        max_commit_files: Some(200),
        import_granularity: "file".to_string(),
    };

    let json = serde_json::to_string(&args).unwrap();
    let round: AnalysisArgsMeta = serde_json::from_str(&json).unwrap();

    assert_eq!(round.preset, args.preset);
    assert_eq!(round.format, args.format);
    assert_eq!(round.window_tokens, args.window_tokens);
    assert_eq!(round.git, args.git);
    assert_eq!(round.max_files, args.max_files);
    assert_eq!(round.max_bytes, args.max_bytes);
    assert_eq!(round.max_file_bytes, args.max_file_bytes);
    assert_eq!(round.max_commits, args.max_commits);
    assert_eq!(round.max_commit_files, args.max_commit_files);
    assert_eq!(round.import_granularity, args.import_granularity);
}

#[test]
fn source_json_roundtrip() {
    let source = AnalysisSource {
        inputs: vec!["/repo".to_string(), "/other".to_string()],
        export_path: Some("/tmp/export.json".to_string()),
        base_receipt_path: Some("/tmp/base.json".to_string()),
        export_schema_version: Some(2),
        export_generated_at_ms: Some(1700000000000),
        base_signature: Some("abc123".to_string()),
        module_roots: vec!["crates".to_string(), "packages".to_string()],
        module_depth: 3,
        children: "collapse".to_string(),
    };

    let json = serde_json::to_string(&source).unwrap();
    let round: AnalysisSource = serde_json::from_str(&json).unwrap();

    assert_eq!(round.inputs, source.inputs);
    assert_eq!(round.export_path, source.export_path);
    assert_eq!(round.base_signature, source.base_signature);
    assert_eq!(round.module_roots, source.module_roots);
    assert_eq!(round.module_depth, source.module_depth);
    assert_eq!(round.children, source.children);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Preset plan field-level truth-table spot checks
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_plan_has_all_flags_false() {
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
fn health_plan_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo, "health should enable todo");
    assert!(plan.complexity, "health should enable complexity");
    assert!(!plan.git, "health should not enable git");
    assert!(!plan.assets, "health should not enable assets");
    assert!(!plan.imports, "health should not enable imports");
    assert!(!plan.fun, "health should not enable fun");
}

#[test]
fn risk_plan_enables_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git, "risk should enable git");
    assert!(plan.complexity, "risk should enable complexity");
    assert!(!plan.assets, "risk should not enable assets");
    assert!(!plan.deps, "risk should not enable deps");
    assert!(!plan.fun, "risk should not enable fun");
}

#[test]
fn supply_plan_enables_assets_and_deps_only() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets, "supply should enable assets");
    assert!(plan.deps, "supply should enable deps");
    assert!(!plan.git, "supply should not enable git");
    assert!(!plan.todo, "supply should not enable todo");
    assert!(!plan.imports, "supply should not enable imports");
    assert!(!plan.fun, "supply should not enable fun");
}

#[test]
fn architecture_plan_enables_imports_and_api_surface() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports, "architecture should enable imports");
    assert!(plan.api_surface, "architecture should enable api_surface");
    assert!(!plan.git, "architecture should not enable git");
    assert!(!plan.assets, "architecture should not enable assets");
}

#[test]
fn security_plan_enables_entropy_and_license() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy, "security should enable entropy");
    assert!(plan.license, "security should enable license");
    assert!(!plan.git, "security should not enable git");
    assert!(!plan.imports, "security should not enable imports");
}

#[test]
fn identity_plan_enables_archetype_and_git() {
    let plan = preset_plan_for(PresetKind::Identity);
    assert!(plan.archetype, "identity should enable archetype");
    assert!(plan.git, "identity should enable git");
    assert!(!plan.imports, "identity should not enable imports");
    assert!(!plan.fun, "identity should not enable fun");
}

#[test]
fn git_plan_enables_git_only() {
    let plan = preset_plan_for(PresetKind::Git);
    assert!(plan.git, "git preset should enable git");
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.imports);
    assert!(!plan.fun);
    assert!(!plan.entropy);
    assert!(!plan.license);
}

#[test]
fn fun_plan_enables_fun_only() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun, "fun preset should enable fun");
    assert!(!plan.git);
    assert!(!plan.assets);
    assert!(!plan.todo);
    assert!(!plan.imports);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
}

#[test]
fn deep_plan_enables_everything_except_fun() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(plan.todo);
    assert!(plan.dup);
    assert!(plan.imports);
    assert!(plan.git);
    assert!(plan.archetype);
    assert!(plan.topics);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
    assert!(!plan.fun, "deep should NOT enable fun");
}

#[test]
fn receipt_plan_needs_no_files() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(!plan.needs_files(), "receipt plan should not need files");
}

#[test]
fn deep_plan_needs_files() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.needs_files(), "deep plan should need files");
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Feature availability reflected in warnings
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_unavailable_reported_in_warnings() {
    let tmp = tempfile::tempdir().unwrap();
    let mut req = make_req(AnalysisPreset::Git);
    req.git = Some(true);

    let ctx = AnalysisContext {
        export: sample_export(),
        root: tmp.path().to_path_buf(),
        source: make_source(),
    };
    let receipt = analyze(ctx, req).unwrap();

    assert!(
        matches!(receipt.status, ScanStatus::Partial),
        "git preset on non-repo should be Partial"
    );
    let has_git_warning = receipt.warnings.iter().any(|w| w.contains("git"));
    assert!(
        has_git_warning,
        "should mention git in warnings: {:?}",
        receipt.warnings
    );
}

#[test]
fn deep_preset_on_temp_dir_produces_partial_status() {
    let tmp = tempfile::tempdir().unwrap();
    let mut req = make_req(AnalysisPreset::Deep);
    req.git = Some(true);

    let ctx = AnalysisContext {
        export: sample_export(),
        root: tmp.path().to_path_buf(),
        source: make_source(),
    };
    let receipt = analyze(ctx, req).unwrap();

    assert!(
        matches!(receipt.status, ScanStatus::Partial),
        "Deep preset on temp dir should be Partial"
    );
    assert!(
        !receipt.warnings.is_empty(),
        "Deep preset on temp dir should produce warnings"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Derived metric value spot checks
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn reading_time_is_proportional_to_code_lines() {
    let export = ExportData {
        rows: vec![row("a.rs", "src", "Rust", 200)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let rt = receipt.derived.unwrap().reading_time;
    // 200 lines / 20 lines per minute = 10.0 minutes
    assert!(
        (rt.minutes - 10.0).abs() < 0.01,
        "expected 10.0 minutes, got {}",
        rt.minutes
    );
}

#[test]
fn cocomo_kloc_matches_code_div_1000() {
    let export = ExportData {
        rows: vec![row("a.rs", "src", "Rust", 5000)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let cocomo = receipt.derived.unwrap().cocomo.unwrap();
    assert!(
        (cocomo.kloc - 5.0).abs() < 0.001,
        "expected 5.0 KLOC, got {}",
        cocomo.kloc
    );
}

#[test]
fn doc_density_ratio_is_comments_over_code_plus_comments() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "a.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 800,
            comments: 200,
            blanks: 0,
            lines: 1000,
            bytes: 10000,
            tokens: 1600,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let ratio = receipt.derived.unwrap().doc_density.total.ratio;
    // 200 / (800 + 200) = 0.2
    assert!(
        (ratio - 0.2).abs() < 0.01,
        "expected ~0.2 doc density, got {}",
        ratio
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Polyglot edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn single_language_has_zero_entropy() {
    let export = ExportData {
        rows: vec![
            row("a.rs", "src", "Rust", 100),
            row("b.rs", "src", "Rust", 200),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let poly = &receipt.derived.unwrap().polyglot;
    assert_eq!(poly.lang_count, 1);
    assert!(
        (poly.entropy - 0.0).abs() < f64::EPSILON,
        "single lang entropy should be 0"
    );
    assert_eq!(poly.dominant_lang, "Rust");
    assert!((poly.dominant_pct - 1.0).abs() < 0.001);
}

#[test]
fn two_equal_languages_have_entropy_one() {
    let export = ExportData {
        rows: vec![
            row("a.rs", "src", "Rust", 100),
            row("b.py", "src", "Python", 100),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let poly = &receipt.derived.unwrap().polyglot;
    assert_eq!(poly.lang_count, 2);
    assert!(
        (poly.entropy - 1.0).abs() < 0.01,
        "two equal langs should have entropy ~1.0, got {}",
        poly.entropy
    );
    assert!((poly.dominant_pct - 0.5).abs() < 0.01);
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. detail_functions flag doesn't panic
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn detail_functions_flag_does_not_panic() {
    let mut req = make_req(AnalysisPreset::Health);
    req.git = Some(false);
    req.detail_functions = true;
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.derived.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. All format strings tolerated
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn various_format_strings_do_not_panic() {
    for fmt in ["json", "md", "tree", "tsv", "json,tree", ""] {
        let mut req = make_req(AnalysisPreset::Receipt);
        req.args.format = fmt.to_string();
        let receipt = analyze(make_ctx(sample_export()), req).unwrap();
        assert!(receipt.derived.is_some());
        if fmt.contains("tree") {
            assert!(receipt.derived.as_ref().unwrap().tree.is_some());
        } else {
            assert!(receipt.derived.as_ref().unwrap().tree.is_none());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Multi-language distribution stats
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn distribution_stats_are_consistent() {
    let export = ExportData {
        rows: vec![
            row("a.rs", "src", "Rust", 100),
            row("b.rs", "src", "Rust", 200),
            row("c.rs", "src", "Rust", 300),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let dist = &receipt.derived.unwrap().distribution;

    assert_eq!(dist.count, 3);
    // Distribution is computed from lines (code + comments + blanks).
    // row() helper: comments = code/5, blanks = code/10 → lines = code * 1.3
    assert!(dist.min > 0);
    assert!(dist.max > 0);
    assert!(dist.mean > 0.0);
    assert!(dist.median > 0.0);
    assert!(dist.min <= dist.max);
    assert!((0.0..=1.0).contains(&dist.gini));
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. Top offenders capped at 10
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn top_offenders_capped_at_ten() {
    let rows: Vec<FileRow> = (0..25)
        .map(|i| row(&format!("src/f{}.rs", i), "src", "Rust", (i + 1) * 100))
        .collect();
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = run_preset(export, AnalysisPreset::Receipt);
    let top = &receipt.derived.unwrap().top;

    assert!(top.largest_lines.len() <= 10);
    assert!(top.largest_tokens.len() <= 10);
    assert!(top.largest_bytes.len() <= 10);
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. AnalysisReceipt full JSON round-trip with all presets
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_presets_json_roundtrip_is_lossless() {
    let export = sample_export();
    for preset in PresetKind::all() {
        let receipt = run_preset(export.clone(), *preset);
        let json = serde_json::to_string_pretty(&receipt).unwrap();
        let deserialized: tokmd_analysis_types::AnalysisReceipt = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("preset {:?} failed to deserialize: {}", preset, e));
        // Compare non-volatile fields
        let mut v1 = serde_json::to_value(receipt).unwrap();
        let mut v2 = serde_json::to_value(deserialized).unwrap();
        for v in [&mut v1, &mut v2] {
            if let Some(obj) = v.as_object_mut() {
                obj.remove("generated_at_ms");
                obj.remove("tool");
            }
        }
        assert_eq!(v1, v2, "preset {:?} JSON round-trip is lossy", preset);
    }
}
