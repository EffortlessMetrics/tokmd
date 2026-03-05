//! Deep contract stability and enum coverage tests for `tokmd-types`.
//!
//! Covers: all receipt types roundtrip, exhaustive enum serde, schema version
//! constants, row field serialization, totals edge cases, diff structures,
//! property-based tests, and edge cases.

use proptest::prelude::*;
use serde_json::{Value, json};
use tokmd_types::cockpit::*;
use tokmd_types::*;

// =============================================================================
// Helpers
// =============================================================================

fn make_scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".to_string()],
        excluded: vec!["target".to_string()],
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

fn make_totals(code: usize) -> Totals {
    Totals {
        code,
        lines: code + code / 2,
        files: 5,
        bytes: code * 40,
        tokens: code * 10,
        avg_lines: if code > 0 { (code + code / 2) / 5 } else { 0 },
    }
}

fn make_lang_row(name: &str, code: usize) -> LangRow {
    LangRow {
        lang: name.to_string(),
        code,
        lines: code + 50,
        files: 3,
        bytes: code * 40,
        tokens: code * 10,
        avg_lines: (code + 50) / 3,
    }
}

// =============================================================================
// 1. Schema version constants are u32 and > 0
// =============================================================================

#[test]
fn schema_version_constants_are_positive_u32() {
    let versions: Vec<u32> = vec![
        SCHEMA_VERSION,
        HANDOFF_SCHEMA_VERSION,
        CONTEXT_BUNDLE_SCHEMA_VERSION,
        CONTEXT_SCHEMA_VERSION,
        COCKPIT_SCHEMA_VERSION,
    ];
    for v in versions {
        assert!(v > 0, "Schema version must be > 0, got {v}");
    }
}

#[test]
fn schema_version_exact_values() {
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
}

// =============================================================================
// 2. LangReceipt roundtrip
// =============================================================================

#[test]
fn lang_receipt_serde_roundtrip() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 12345,
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec!["test warning".to_string()],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 10,
            with_files: true,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![make_lang_row("Rust", 1000)],
            total: make_totals(1000),
            with_files: true,
            children: ChildrenMode::Collapse,
            top: 10,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.mode, "lang");
    assert_eq!(back.report.rows.len(), 1);
    assert_eq!(back.report.rows[0].lang, "Rust");
    assert_eq!(back.report.total.code, 1000);
    assert_eq!(back.warnings.len(), 1);
}

// =============================================================================
// 3. ModuleReceipt roundtrip
// =============================================================================

#[test]
fn module_receipt_serde_roundtrip() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 99,
        tool: ToolInfo::current(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 5,
        },
        report: ModuleReport {
            rows: vec![ModuleRow {
                module: "crates/core".to_string(),
                code: 500,
                lines: 700,
                files: 3,
                bytes: 25000,
                tokens: 6250,
                avg_lines: 233,
            }],
            total: make_totals(500),
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 5,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, "module");
    assert_eq!(back.report.rows[0].module, "crates/core");
}

// =============================================================================
// 4. ExportReceipt roundtrip
// =============================================================================

#[test]
fn export_receipt_serde_roundtrip() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 42,
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Jsonl,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::ParentsOnly,
            min_code: 0,
            max_rows: 10000,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 20,
                blanks: 10,
                lines: 130,
                bytes: 5000,
                tokens: 1250,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::ParentsOnly,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, "export");
    assert_eq!(back.data.rows[0].kind, FileKind::Parent);
}

// =============================================================================
// 5. DiffReceipt roundtrip
// =============================================================================

#[test]
fn diff_receipt_serde_roundtrip() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 7,
        tool: ToolInfo::current(),
        mode: "diff".to_string(),
        from_source: "v1.0".to_string(),
        to_source: "v2.0".to_string(),
        diff_rows: vec![DiffRow {
            lang: "Rust".to_string(),
            old_code: 100,
            new_code: 200,
            delta_code: 100,
            old_lines: 150,
            new_lines: 300,
            delta_lines: 150,
            old_files: 5,
            new_files: 8,
            delta_files: 3,
            old_bytes: 5000,
            new_bytes: 10000,
            delta_bytes: 5000,
            old_tokens: 1250,
            new_tokens: 2500,
            delta_tokens: 1250,
        }],
        totals: DiffTotals {
            old_code: 100,
            new_code: 200,
            delta_code: 100,
            old_lines: 150,
            new_lines: 300,
            delta_lines: 150,
            old_files: 5,
            new_files: 8,
            delta_files: 3,
            old_bytes: 5000,
            new_bytes: 10000,
            delta_bytes: 5000,
            old_tokens: 1250,
            new_tokens: 2500,
            delta_tokens: 1250,
        },
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: DiffReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from_source, "v1.0");
    assert_eq!(back.totals.delta_code, 100);
}

// =============================================================================
// 6. RunReceipt roundtrip
// =============================================================================

#[test]
fn run_receipt_serde_roundtrip() {
    let receipt = RunReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 999,
        lang_file: "lang.json".to_string(),
        module_file: "module.json".to_string(),
        export_file: "export.jsonl".to_string(),
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: RunReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lang_file, "lang.json");
    assert_eq!(back.module_file, "module.json");
    assert_eq!(back.export_file, "export.jsonl");
}

// =============================================================================
// 7. ContextReceipt roundtrip
// =============================================================================

#[test]
fn context_receipt_serde_roundtrip() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 100,
        tool: ToolInfo::current(),
        mode: "context".to_string(),
        budget_tokens: 128000,
        used_tokens: 64000,
        utilization_pct: 50.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 10,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.budget_tokens, 128000);
    assert_eq!(back.used_tokens, 64000);
}

// =============================================================================
// 8. HandoffManifest roundtrip
// =============================================================================

#[test]
fn handoff_manifest_serde_roundtrip() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 200,
        tool: ToolInfo::current(),
        mode: "handoff".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "handoff_out".to_string(),
        budget_tokens: 50000,
        used_tokens: 25000,
        utilization_pct: 50.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 100,
        bundled_files: 50,
        intelligence_preset: "basic".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(back.total_files, 100);
    assert_eq!(back.bundled_files, 50);
}

// =============================================================================
// 9. CockpitReceipt roundtrip
// =============================================================================

#[test]
fn cockpit_receipt_serde_roundtrip() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 300,
        base_ref: "main".to_string(),
        head_ref: "feature".to_string(),
        change_surface: ChangeSurface {
            commits: 5,
            files_changed: 10,
            insertions: 200,
            deletions: 50,
            net_lines: 150,
            churn_velocity: 50.0,
            change_concentration: 0.8,
        },
        composition: Composition {
            code_pct: 70.0,
            test_pct: 20.0,
            docs_pct: 5.0,
            config_pct: 5.0,
            test_ratio: 0.29,
        },
        code_health: CodeHealth {
            score: 85,
            grade: "B".to_string(),
            large_files_touched: 1,
            avg_file_size: 200,
            complexity_indicator: ComplexityIndicator::Medium,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec!["src/lib.rs".to_string()],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 20,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: Evidence {
            overall_status: GateStatus::Pass,
            mutation: MutationGate {
                meta: GateMeta {
                    status: GateStatus::Skipped,
                    source: EvidenceSource::RanLocal,
                    commit_match: CommitMatch::Exact,
                    scope: ScopeCoverage {
                        relevant: vec![],
                        tested: vec![],
                        ratio: 0.0,
                        lines_relevant: None,
                        lines_tested: None,
                    },
                    evidence_commit: None,
                    evidence_generated_at_ms: None,
                },
                survivors: vec![],
                killed: 0,
                timeout: 0,
                unviable: 0,
            },
            diff_coverage: None,
            contracts: None,
            supply_chain: None,
            determinism: None,
            complexity: None,
        },
        review_plan: vec![],
        trend: None,
    };

    let json = serde_json::to_string(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(back.base_ref, "main");
    assert_eq!(back.change_surface.commits, 5);
}

// =============================================================================
// 10. Exhaustive ScanStatus serde
// =============================================================================

#[test]
fn scan_status_exhaustive_serde() {
    let variants = [ScanStatus::Complete, ScanStatus::Partial];
    let expected = ["\"complete\"", "\"partial\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: ScanStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 11. Exhaustive ChildrenMode serde
// =============================================================================

#[test]
fn children_mode_exhaustive_serde() {
    let variants = [ChildrenMode::Collapse, ChildrenMode::Separate];
    let expected = ["\"collapse\"", "\"separate\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 12. Exhaustive ChildIncludeMode serde
// =============================================================================

#[test]
fn child_include_mode_exhaustive_serde() {
    let variants = [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly];
    let expected = ["\"separate\"", "\"parents-only\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 13. Exhaustive FileKind serde
// =============================================================================

#[test]
fn file_kind_exhaustive_serde() {
    let variants = [FileKind::Parent, FileKind::Child];
    let expected = ["\"parent\"", "\"child\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: FileKind = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 14. Exhaustive ExportFormat serde
// =============================================================================

#[test]
fn export_format_exhaustive_serde() {
    let variants = [
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ];
    let expected = ["\"csv\"", "\"jsonl\"", "\"json\"", "\"cyclonedx\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 15. Exhaustive TableFormat serde
// =============================================================================

#[test]
fn table_format_exhaustive_serde() {
    let variants = [TableFormat::Md, TableFormat::Tsv, TableFormat::Json];
    let expected = ["\"md\"", "\"tsv\"", "\"json\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: TableFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 16. Exhaustive RedactMode serde
// =============================================================================

#[test]
fn redact_mode_exhaustive_serde() {
    let variants = [RedactMode::None, RedactMode::Paths, RedactMode::All];
    let expected = ["\"none\"", "\"paths\"", "\"all\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 17. Exhaustive ConfigMode serde
// =============================================================================

#[test]
fn config_mode_exhaustive_serde() {
    let variants = [ConfigMode::Auto, ConfigMode::None];
    let expected = ["\"auto\"", "\"none\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: ConfigMode = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 18. Exhaustive AnalysisFormat serde
// =============================================================================

#[test]
fn analysis_format_exhaustive_serde() {
    let variants = [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Jsonld,
        AnalysisFormat::Xml,
        AnalysisFormat::Svg,
        AnalysisFormat::Mermaid,
        AnalysisFormat::Obj,
        AnalysisFormat::Midi,
        AnalysisFormat::Tree,
        AnalysisFormat::Html,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: AnalysisFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 19. Exhaustive InclusionPolicy serde
// =============================================================================

#[test]
fn inclusion_policy_exhaustive_serde() {
    let variants = [
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ];
    let expected = ["\"full\"", "\"head_tail\"", "\"summary\"", "\"skip\""];
    for (v, exp) in variants.iter().zip(expected.iter()) {
        let json = serde_json::to_string(v).unwrap();
        assert_eq!(&json, exp);
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 20. Exhaustive FileClassification serde
// =============================================================================

#[test]
fn file_classification_exhaustive_serde() {
    let variants = [
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 21. Exhaustive CommitIntentKind serde
// =============================================================================

#[test]
fn commit_intent_kind_exhaustive_serde() {
    let variants = [
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
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 22. Exhaustive cockpit enums serde
// =============================================================================

#[test]
fn gate_status_exhaustive_serde() {
    let variants = [
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

#[test]
fn evidence_source_exhaustive_serde() {
    let variants = [
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

#[test]
fn commit_match_exhaustive_serde() {
    let variants = [
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: CommitMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

#[test]
fn complexity_indicator_exhaustive_serde() {
    let variants = [
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: ComplexityIndicator = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

#[test]
fn risk_level_exhaustive_serde() {
    let variants = [
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

#[test]
fn warning_type_exhaustive_serde() {
    let variants = [
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: WarningType = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

#[test]
fn capability_state_exhaustive_serde() {
    let variants = [
        CapabilityState::Available,
        CapabilityState::Skipped,
        CapabilityState::Unavailable,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}

// =============================================================================
// 23. LangRow all fields serialize correctly
// =============================================================================

#[test]
fn lang_row_all_fields_serialize() {
    let row = LangRow {
        lang: "Python".to_string(),
        code: 500,
        lines: 700,
        files: 12,
        bytes: 20000,
        tokens: 5000,
        avg_lines: 58,
    };
    let v: Value = serde_json::to_value(&row).unwrap();
    assert_eq!(v["lang"], "Python");
    assert_eq!(v["code"], 500);
    assert_eq!(v["lines"], 700);
    assert_eq!(v["files"], 12);
    assert_eq!(v["bytes"], 20000);
    assert_eq!(v["tokens"], 5000);
    assert_eq!(v["avg_lines"], 58);
}

// =============================================================================
// 24. ModuleRow all fields serialize correctly
// =============================================================================

#[test]
fn module_row_all_fields_serialize() {
    let row = ModuleRow {
        module: "crates/core".to_string(),
        code: 300,
        lines: 400,
        files: 5,
        bytes: 15000,
        tokens: 3750,
        avg_lines: 80,
    };
    let v: Value = serde_json::to_value(&row).unwrap();
    assert_eq!(v["module"], "crates/core");
    assert_eq!(v["code"], 300);
    assert_eq!(v["files"], 5);
}

// =============================================================================
// 25. FileRow all fields serialize correctly
// =============================================================================

#[test]
fn file_row_all_fields_serialize() {
    let row = FileRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 200,
        comments: 30,
        blanks: 20,
        lines: 250,
        bytes: 8000,
        tokens: 2000,
    };
    let v: Value = serde_json::to_value(&row).unwrap();
    assert_eq!(v["path"], "src/lib.rs");
    assert_eq!(v["module"], "src");
    assert_eq!(v["lang"], "Rust");
    assert_eq!(v["kind"], "parent");
    assert_eq!(v["code"], 200);
    assert_eq!(v["comments"], 30);
    assert_eq!(v["blanks"], 20);
    assert_eq!(v["lines"], 250);
    assert_eq!(v["bytes"], 8000);
    assert_eq!(v["tokens"], 2000);
}

// =============================================================================
// 26. Totals: zero totals
// =============================================================================

#[test]
fn totals_zero_roundtrip() {
    let t = Totals {
        code: 0,
        lines: 0,
        files: 0,
        bytes: 0,
        tokens: 0,
        avg_lines: 0,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Totals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

// =============================================================================
// 27. Totals: realistic totals
// =============================================================================

#[test]
fn totals_realistic_roundtrip() {
    let t = Totals {
        code: 50000,
        lines: 75000,
        files: 200,
        bytes: 2_000_000,
        tokens: 500_000,
        avg_lines: 375,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Totals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

// =============================================================================
// 28. DiffTotals default is all zeroed
// =============================================================================

#[test]
fn diff_totals_default_all_zero() {
    let dt = DiffTotals::default();
    assert_eq!(dt.old_code, 0);
    assert_eq!(dt.new_code, 0);
    assert_eq!(dt.delta_code, 0);
    assert_eq!(dt.old_lines, 0);
    assert_eq!(dt.new_lines, 0);
    assert_eq!(dt.delta_lines, 0);
    assert_eq!(dt.old_files, 0);
    assert_eq!(dt.new_files, 0);
    assert_eq!(dt.delta_files, 0);
    assert_eq!(dt.old_bytes, 0);
    assert_eq!(dt.new_bytes, 0);
    assert_eq!(dt.delta_bytes, 0);
    assert_eq!(dt.old_tokens, 0);
    assert_eq!(dt.new_tokens, 0);
    assert_eq!(dt.delta_tokens, 0);
}

// =============================================================================
// 29. DiffTotals serde roundtrip with negative deltas
// =============================================================================

#[test]
fn diff_totals_negative_deltas_roundtrip() {
    let dt = DiffTotals {
        old_code: 1000,
        new_code: 500,
        delta_code: -500,
        old_lines: 1500,
        new_lines: 700,
        delta_lines: -800,
        old_files: 10,
        new_files: 5,
        delta_files: -5,
        old_bytes: 40000,
        new_bytes: 20000,
        delta_bytes: -20000,
        old_tokens: 10000,
        new_tokens: 5000,
        delta_tokens: -5000,
    };
    let json = serde_json::to_string(&dt).unwrap();
    let back: DiffTotals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, dt);
    assert_eq!(back.delta_code, -500);
}

// =============================================================================
// 30. DiffRow with negative deltas
// =============================================================================

#[test]
fn diff_row_negative_deltas() {
    let row = DiffRow {
        lang: "Go".to_string(),
        old_code: 500,
        new_code: 300,
        delta_code: -200,
        old_lines: 700,
        new_lines: 400,
        delta_lines: -300,
        old_files: 8,
        new_files: 5,
        delta_files: -3,
        old_bytes: 20000,
        new_bytes: 12000,
        delta_bytes: -8000,
        old_tokens: 5000,
        new_tokens: 3000,
        delta_tokens: -2000,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: DiffRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

// =============================================================================
// 31. Empty receipts (no rows)
// =============================================================================

#[test]
fn empty_lang_receipt() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo::default(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: make_totals(0),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert!(back.report.rows.is_empty());
    assert_eq!(back.report.total.code, 0);
}

#[test]
fn empty_export_receipt() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo::default(),
        mode: "export".to_string(),
        status: ScanStatus::Partial,
        warnings: vec![],
        scan: make_scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Csv,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 0,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ExportReceipt = serde_json::from_str(&json).unwrap();
    assert!(back.data.rows.is_empty());
}

// =============================================================================
// 32. Very large line counts
// =============================================================================

#[test]
fn very_large_line_counts_roundtrip() {
    let t = Totals {
        code: usize::MAX / 2,
        lines: usize::MAX / 2,
        files: 1_000_000,
        bytes: usize::MAX / 2,
        tokens: usize::MAX / 2,
        avg_lines: usize::MAX / 2,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Totals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

// =============================================================================
// 33. ToolInfo current has correct name
// =============================================================================

#[test]
fn tool_info_current_name_is_tokmd() {
    let ti = ToolInfo::current();
    assert_eq!(ti.name, "tokmd");
    assert!(!ti.version.is_empty());
}

// =============================================================================
// 34. Default impls
// =============================================================================

#[test]
fn config_mode_default_is_auto() {
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn inclusion_policy_default_is_full() {
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

// =============================================================================
// 35. ScanArgs excluded_redacted skip_serializing_if
// =============================================================================

#[test]
fn scan_args_excluded_redacted_skipped_when_false() {
    let args = ScanArgs {
        excluded_redacted: false,
        ..make_scan_args()
    };
    let json = serde_json::to_string(&args).unwrap();
    assert!(!json.contains("excluded_redacted"));
}

#[test]
fn scan_args_excluded_redacted_present_when_true() {
    let mut args = make_scan_args();
    args.excluded_redacted = true;
    let json = serde_json::to_string(&args).unwrap();
    assert!(json.contains("excluded_redacted"));
}

// =============================================================================
// 36. TokenEstimationMeta invariant
// =============================================================================

#[test]
fn token_estimation_invariant_min_le_est_le_max() {
    for bytes in [0, 1, 100, 4000, 999_999] {
        let est = TokenEstimationMeta::from_bytes(bytes, 4.0);
        assert!(
            est.tokens_min <= est.tokens_est,
            "min <= est failed for {bytes}"
        );
        assert!(
            est.tokens_est <= est.tokens_max,
            "est <= max failed for {bytes}"
        );
    }
}

// =============================================================================
// 37. TokenEstimationMeta serde alias roundtrip
// =============================================================================

#[test]
fn token_estimation_alias_tokens_high_to_tokens_min() {
    let json = json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_high": 800,
        "tokens_est": 1000,
        "tokens_low": 1334,
        "source_bytes": 4000
    });
    let est: TokenEstimationMeta = serde_json::from_value(json).unwrap();
    assert_eq!(est.tokens_min, 800);
    assert_eq!(est.tokens_max, 1334);
}

// =============================================================================
// 38. LangReport flattened in LangReceipt JSON
// =============================================================================

#[test]
fn lang_receipt_report_is_flattened() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1,
        tool: ToolInfo::default(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: make_totals(0),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };
    let v: Value = serde_json::to_value(receipt).unwrap();
    assert!(v.get("rows").is_some(), "rows flattened to top level");
    assert!(v.get("total").is_some(), "total flattened to top level");
    assert!(v.get("report").is_none(), "report key should not exist");
}

// =============================================================================
// Property tests
// =============================================================================

proptest! {
    #[test]
    fn proptest_totals_roundtrip(
        code in 0..1_000_000usize,
        lines in 0..2_000_000usize,
        files in 0..100_000usize,
        bytes in 0..100_000_000usize,
        tokens in 0..50_000_000usize,
        avg_lines in 0..10_000usize,
    ) {
        let t = Totals { code, lines, files, bytes, tokens, avg_lines };
        let json = serde_json::to_string(&t).unwrap();
        let back: Totals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, t);
    }

    #[test]
    fn proptest_lang_row_roundtrip(
        name in "[A-Za-z][A-Za-z0-9 ]{0,30}",
        code in 0..1_000_000usize,
        lines in 0..2_000_000usize,
        files in 1..1_000usize,
        bytes in 0..100_000_000usize,
        tokens in 0..50_000_000usize,
    ) {
        let avg = lines.checked_div(files).unwrap_or(0);
        let row = LangRow {
            lang: name.clone(),
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines: avg,
        };
        let json = serde_json::to_string(&row).unwrap();
        let back: LangRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.lang, name);
        prop_assert_eq!(back.code, code);
    }

    #[test]
    fn proptest_diff_totals_roundtrip(
        old_code in 0..1_000_000usize,
        new_code in 0..1_000_000usize,
    ) {
        let delta = new_code as i64 - old_code as i64;
        let dt = DiffTotals {
            old_code,
            new_code,
            delta_code: delta,
            old_lines: old_code,
            new_lines: new_code,
            delta_lines: delta,
            old_files: 5,
            new_files: 8,
            delta_files: 3,
            old_bytes: old_code * 40,
            new_bytes: new_code * 40,
            delta_bytes: delta * 40,
            old_tokens: old_code * 10,
            new_tokens: new_code * 10,
            delta_tokens: delta * 10,
        };
        let json = serde_json::to_string(&dt).unwrap();
        let back: DiffTotals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, dt);
    }

    #[test]
    fn proptest_file_row_roundtrip(
        path in "[a-z/]{1,50}",
        code in 0..100_000usize,
        comments in 0..10_000usize,
        blanks in 0..10_000usize,
    ) {
        let row = FileRow {
            path: path.clone(),
            module: "mod".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code,
            comments,
            blanks,
            lines: code + comments + blanks,
            bytes: (code + comments + blanks) * 40,
            tokens: code * 10,
        };
        let json = serde_json::to_string(&row).unwrap();
        let back: FileRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.path, path);
        prop_assert_eq!(back.code, code);
    }
}
