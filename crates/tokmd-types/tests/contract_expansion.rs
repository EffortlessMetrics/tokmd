//! Expanded contract tests for tokmd-types.
//!
//! Focuses on receipt construction, optional-field contracts,
//! ScanArgs roundtrip, and handoff/context type coverage.

use proptest::prelude::*;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    ConfigMode, ContextBundleManifest, ContextExcludedPath, ContextFileRow, ContextReceipt,
    ExportArgsMeta, ExportData, ExportFormat, ExportReceipt, FileClassification, FileKind, FileRow,
    HANDOFF_SCHEMA_VERSION, HandoffComplexity, HandoffDerived, HandoffExcludedPath, HandoffHotspot,
    HandoffIntelligence, HandoffManifest, InclusionPolicy, LangArgsMeta, LangReceipt, LangReport,
    LangRow, ModuleArgsMeta, ModuleReceipt, ModuleReport, ModuleRow, RedactMode, RunReceipt,
    SCHEMA_VERSION, ScanArgs, ScanStatus, SmartExcludedFile, ToolInfo, Totals,
};

// =============================================================================
// Helpers
// =============================================================================

fn sample_tool_info() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.0.0-test".to_string(),
    }
}

fn sample_scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec![],
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

fn sample_totals() -> Totals {
    Totals {
        code: 1000,
        lines: 1500,
        files: 10,
        bytes: 50_000,
        tokens: 12_500,
        avg_lines: 150,
    }
}

fn sample_lang_row() -> LangRow {
    LangRow {
        lang: "Rust".to_string(),
        code: 1000,
        lines: 1500,
        files: 10,
        bytes: 50_000,
        tokens: 12_500,
        avg_lines: 150,
    }
}

// =============================================================================
// Scenario: LangReceipt construction and roundtrip
// =============================================================================

#[test]
fn given_lang_receipt_when_constructed_then_schema_version_matches_constant() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![sample_lang_row()],
            total: sample_totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };

    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.report.rows.len(), 1);
}

#[test]
fn given_lang_receipt_when_roundtripped_then_all_fields_preserved() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec!["test warning".to_string()],
        scan: sample_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 5,
            with_files: true,
            children: ChildrenMode::Separate,
        },
        report: LangReport {
            rows: vec![sample_lang_row()],
            total: sample_totals(),
            with_files: true,
            children: ChildrenMode::Separate,
            top: 5,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();

    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.generated_at_ms, 1700000000000);
    assert_eq!(back.warnings, vec!["test warning"]);
    assert_eq!(back.args.top, 5);
    assert!(back.args.with_files);
    assert_eq!(back.report.rows[0].lang, "Rust");
    assert_eq!(back.report.total, sample_totals());
}

// =============================================================================
// Scenario: ModuleReceipt construction and roundtrip
// =============================================================================

#[test]
fn given_module_receipt_when_roundtripped_then_module_rows_preserved() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
        report: ModuleReport {
            rows: vec![
                ModuleRow {
                    module: "crates/core".to_string(),
                    code: 500,
                    lines: 700,
                    files: 5,
                    bytes: 20_000,
                    tokens: 5_000,
                    avg_lines: 140,
                },
                ModuleRow {
                    module: "crates/types".to_string(),
                    code: 300,
                    lines: 400,
                    files: 3,
                    bytes: 12_000,
                    tokens: 3_000,
                    avg_lines: 133,
                },
            ],
            total: sample_totals(),
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();

    assert_eq!(back.report.rows.len(), 2);
    assert_eq!(back.report.rows[0].module, "crates/core");
    assert_eq!(back.report.rows[1].module, "crates/types");
    assert_eq!(back.args.module_depth, 2);
    assert_eq!(back.args.module_roots, vec!["crates"]);
}

// =============================================================================
// Scenario: ExportReceipt construction with file rows
// =============================================================================

#[test]
fn given_export_receipt_with_file_rows_when_roundtripped_then_preserved() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: sample_scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Jsonl,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::ParentsOnly,
            min_code: 10,
            max_rows: 5000,
            redact: RedactMode::Paths,
            strip_prefix: Some("/home/user".to_string()),
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 200,
                comments: 30,
                blanks: 20,
                lines: 250,
                bytes: 8_000,
                tokens: 2_000,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::ParentsOnly,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();

    assert_eq!(back.data.rows.len(), 1);
    assert_eq!(back.data.rows[0].path, "src/lib.rs");
    assert_eq!(back.data.rows[0].kind, FileKind::Parent);
    assert_eq!(back.args.redact, RedactMode::Paths);
    assert_eq!(back.args.min_code, 10);
    assert_eq!(back.args.strip_prefix.as_deref(), Some("/home/user"));
}

// =============================================================================
// Scenario: RunReceipt roundtrip
// =============================================================================

#[test]
fn given_run_receipt_when_roundtripped_then_file_names_preserved() {
    let receipt = RunReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        lang_file: "out/lang.json".to_string(),
        module_file: "out/module.json".to_string(),
        export_file: "out/export.jsonl".to_string(),
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: RunReceipt = serde_json::from_str(&json).unwrap();

    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.lang_file, "out/lang.json");
    assert_eq!(back.module_file, "out/module.json");
    assert_eq!(back.export_file, "out/export.jsonl");
}

// =============================================================================
// Scenario: ScanArgs roundtrip preserves all boolean flags
// =============================================================================

#[test]
fn given_scan_args_with_all_flags_true_when_roundtripped_then_preserved() {
    let args = ScanArgs {
        paths: vec!["src".to_string(), "tests".to_string()],
        excluded: vec!["target".to_string(), "*.lock".to_string()],
        excluded_redacted: true,
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };

    let json = serde_json::to_string(&args).unwrap();
    let back: ScanArgs = serde_json::from_str(&json).unwrap();

    assert_eq!(back.paths, vec!["src", "tests"]);
    assert_eq!(back.excluded, vec!["target", "*.lock"]);
    assert!(back.excluded_redacted);
    assert_eq!(back.config, ConfigMode::None);
    assert!(back.hidden);
    assert!(back.no_ignore);
    assert!(back.no_ignore_parent);
    assert!(back.no_ignore_dot);
    assert!(back.no_ignore_vcs);
    assert!(back.treat_doc_strings_as_comments);
}

#[test]
fn given_scan_args_with_defaults_when_serialized_then_excluded_redacted_omitted() {
    let args = sample_scan_args();
    let json = serde_json::to_string(&args).unwrap();
    // excluded_redacted defaults to false and uses skip_serializing_if = Not::not
    assert!(
        !json.contains("\"excluded_redacted\""),
        "false excluded_redacted should be omitted"
    );
}

// =============================================================================
// Scenario: HandoffManifest construction and optional fields
// =============================================================================

#[test]
fn given_handoff_manifest_when_optional_fields_none_then_omitted_from_json() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "handoff".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "out".to_string(),
        budget_tokens: 100_000,
        used_tokens: 80_000,
        utilization_pct: 0.8,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 100,
        bundled_files: 80,
        intelligence_preset: "health".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();

    assert!(!json.contains("\"rank_by_effective\""));
    assert!(!json.contains("\"fallback_reason\""));
    assert!(!json.contains("\"token_estimation\""));
    assert!(!json.contains("\"code_audit\""));
    assert!(!json.contains("\"excluded_by_policy\""));

    // Roundtrip
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(back.budget_tokens, 100_000);
    assert!(back.rank_by_effective.is_none());
    assert!(back.fallback_reason.is_none());
    assert!(back.token_estimation.is_none());
    assert!(back.code_audit.is_none());
}

// =============================================================================
// Scenario: ContextReceipt optional fields contract
// =============================================================================

#[test]
fn given_context_receipt_when_optional_fields_none_then_omitted_from_json() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "context".to_string(),
        budget_tokens: 50_000,
        used_tokens: 40_000,
        utilization_pct: 0.8,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 20,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    assert!(!json.contains("\"rank_by_effective\""));
    assert!(!json.contains("\"fallback_reason\""));
    assert!(!json.contains("\"token_estimation\""));
    assert!(!json.contains("\"bundle_audit\""));
    assert!(!json.contains("\"excluded_by_policy\""));

    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert!(back.rank_by_effective.is_none());
}

// =============================================================================
// Scenario: ContextBundleManifest roundtrip
// =============================================================================

#[test]
fn given_context_bundle_manifest_when_roundtripped_then_preserved() {
    let manifest = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool_info(),
        mode: "context".to_string(),
        budget_tokens: 50_000,
        used_tokens: 45_000,
        utilization_pct: 0.9,
        strategy: "greedy".to_string(),
        rank_by: "tokens".to_string(),
        file_count: 15,
        bundle_bytes: 120_000,
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![ContextExcludedPath {
            path: "target".to_string(),
            reason: "build output".to_string(),
        }],
        excluded_patterns: vec!["*.lock".to_string()],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let back: ContextBundleManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(back.schema_version, CONTEXT_BUNDLE_SCHEMA_VERSION);
    assert_eq!(back.bundle_bytes, 120_000);
    assert_eq!(back.excluded_paths.len(), 1);
    assert_eq!(back.excluded_paths[0].path, "target");
    assert_eq!(back.excluded_patterns, vec!["*.lock"]);
}

// =============================================================================
// Scenario: HandoffIntelligence optional fields
// =============================================================================

#[test]
fn given_handoff_intelligence_with_all_none_when_roundtripped_then_nones_preserved() {
    let intel = HandoffIntelligence {
        tree: None,
        tree_depth: None,
        hotspots: None,
        complexity: None,
        derived: None,
        warnings: vec![],
    };

    let json = serde_json::to_string(&intel).unwrap();
    let back: HandoffIntelligence = serde_json::from_str(&json).unwrap();

    assert!(back.tree.is_none());
    assert!(back.tree_depth.is_none());
    assert!(back.hotspots.is_none());
    assert!(back.complexity.is_none());
    assert!(back.derived.is_none());
    assert!(back.warnings.is_empty());
}

#[test]
fn given_handoff_intelligence_with_all_fields_when_roundtripped_then_preserved() {
    let intel = HandoffIntelligence {
        tree: Some("├── src/\n│   └── lib.rs".to_string()),
        tree_depth: Some(3),
        hotspots: Some(vec![HandoffHotspot {
            path: "src/core.rs".to_string(),
            commits: 42,
            lines: 500,
            score: 21_000,
        }]),
        complexity: Some(HandoffComplexity {
            total_functions: 200,
            avg_function_length: 15.5,
            max_function_length: 120,
            avg_cyclomatic: 3.2,
            max_cyclomatic: 25,
            high_risk_files: 5,
        }),
        derived: Some(HandoffDerived {
            total_files: 50,
            total_code: 10_000,
            total_lines: 15_000,
            total_tokens: 25_000,
            lang_count: 3,
            dominant_lang: "Rust".to_string(),
            dominant_pct: 0.72,
        }),
        warnings: vec!["Git history unavailable".to_string()],
    };

    let json = serde_json::to_string(&intel).unwrap();
    let back: HandoffIntelligence = serde_json::from_str(&json).unwrap();

    assert_eq!(back.tree_depth, Some(3));
    assert_eq!(back.hotspots.as_ref().unwrap().len(), 1);
    assert_eq!(back.hotspots.as_ref().unwrap()[0].commits, 42);
    let comp = back.complexity.as_ref().unwrap();
    assert_eq!(comp.total_functions, 200);
    assert_eq!(comp.max_cyclomatic, 25);
    let der = back.derived.as_ref().unwrap();
    assert_eq!(der.dominant_lang, "Rust");
    assert!((der.dominant_pct - 0.72).abs() < 1e-10);
}

// =============================================================================
// Scenario: Schema version constants are separate per family
// =============================================================================

#[test]
fn given_schema_versions_then_each_family_has_independent_version() {
    // Core receipts
    assert_eq!(SCHEMA_VERSION, 2);
    // Handoff
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    // Context
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    // Context bundle
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);

    // They must not all be equal (families are independent)
    assert_ne!(SCHEMA_VERSION, HANDOFF_SCHEMA_VERSION);
    assert_ne!(SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION);
}

// =============================================================================
// Scenario: ContextFileRow with all classification variants
// =============================================================================

#[test]
fn given_context_file_row_with_multiple_classifications_when_roundtripped_then_preserved() {
    let row = ContextFileRow {
        path: "vendor/lib.min.js".to_string(),
        module: "vendor".to_string(),
        lang: "JavaScript".to_string(),
        tokens: 50_000,
        code: 1,
        lines: 1,
        bytes: 200_000,
        value: 0,
        rank_reason: "low-value".to_string(),
        policy: InclusionPolicy::Skip,
        effective_tokens: Some(0),
        policy_reason: Some("minified vendored code".to_string()),
        classifications: vec![FileClassification::Minified, FileClassification::Vendored],
    };

    let json = serde_json::to_string(&row).unwrap();
    let back: ContextFileRow = serde_json::from_str(&json).unwrap();

    assert_eq!(back.policy, InclusionPolicy::Skip);
    assert_eq!(back.effective_tokens, Some(0));
    assert_eq!(back.classifications.len(), 2);
    assert_eq!(back.classifications[0], FileClassification::Minified);
    assert_eq!(back.classifications[1], FileClassification::Vendored);
}

// =============================================================================
// Scenario: SmartExcludedFile and HandoffExcludedPath roundtrip
// =============================================================================

#[test]
fn given_smart_excluded_file_when_roundtripped_then_fields_preserved() {
    let f = SmartExcludedFile {
        path: "yarn.lock".to_string(),
        reason: "lockfile detected".to_string(),
        tokens: 100_000,
    };

    let json = serde_json::to_string(&f).unwrap();
    let back: SmartExcludedFile = serde_json::from_str(&json).unwrap();

    assert_eq!(back.path, "yarn.lock");
    assert_eq!(back.reason, "lockfile detected");
    assert_eq!(back.tokens, 100_000);
}

#[test]
fn given_handoff_excluded_path_when_roundtripped_then_fields_preserved() {
    let p = HandoffExcludedPath {
        path: "target/debug".to_string(),
        reason: "build output directory".to_string(),
    };

    let json = serde_json::to_string(&p).unwrap();
    let back: HandoffExcludedPath = serde_json::from_str(&json).unwrap();

    assert_eq!(back.path, "target/debug");
    assert_eq!(back.reason, "build output directory");
}

// =============================================================================
// Property: ScanArgs roundtrip with arbitrary boolean combinations
// =============================================================================

proptest! {
    #[test]
    fn prop_scan_args_boolean_flags_roundtrip(
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        treat_doc_strings_as_comments in any::<bool>(),
    ) {
        let args = ScanArgs {
            paths: vec![".".to_string()],
            excluded: vec![],
            excluded_redacted: false,
            config: ConfigMode::Auto,
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments,
        };

        let json = serde_json::to_string(&args).unwrap();
        let back: ScanArgs = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(back.hidden, hidden);
        prop_assert_eq!(back.no_ignore, no_ignore);
        prop_assert_eq!(back.no_ignore_parent, no_ignore_parent);
        prop_assert_eq!(back.no_ignore_dot, no_ignore_dot);
        prop_assert_eq!(back.no_ignore_vcs, no_ignore_vcs);
        prop_assert_eq!(back.treat_doc_strings_as_comments, treat_doc_strings_as_comments);
    }

    #[test]
    fn prop_context_file_row_roundtrip(
        tokens in 0usize..1_000_000,
        code in 0usize..100_000,
        lines in 0usize..200_000,
        bytes in 0usize..10_000_000,
        value in 0usize..1000,
        policy_idx in 0usize..4,
    ) {
        let policies = [
            InclusionPolicy::Full,
            InclusionPolicy::HeadTail,
            InclusionPolicy::Summary,
            InclusionPolicy::Skip,
        ];
        let policy = policies[policy_idx];
        let effective_tokens = if policy == InclusionPolicy::Full { None } else { Some(tokens / 2) };

        let row = ContextFileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            tokens,
            code,
            lines,
            bytes,
            value,
            rank_reason: String::new(),
            policy,
            effective_tokens,
            policy_reason: None,
            classifications: vec![],
        };

        let json = serde_json::to_string(&row).unwrap();
        let back: ContextFileRow = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(back.tokens, tokens);
        prop_assert_eq!(back.code, code);
        prop_assert_eq!(back.value, value);
        prop_assert_eq!(back.policy, policy);
        prop_assert_eq!(back.effective_tokens, effective_tokens);
    }

    #[test]
    fn prop_handoff_hotspot_roundtrip(
        commits in 0usize..10_000,
        lines in 0usize..100_000,
        score in 0usize..10_000_000,
    ) {
        let hotspot = HandoffHotspot {
            path: "src/core.rs".to_string(),
            commits,
            lines,
            score,
        };

        let json = serde_json::to_string(&hotspot).unwrap();
        let back: HandoffHotspot = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(back.commits, commits);
        prop_assert_eq!(back.lines, lines);
        prop_assert_eq!(back.score, score);
    }
}
