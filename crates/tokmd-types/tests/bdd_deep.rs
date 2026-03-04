//! Deep BDD-style scenario tests for tokmd-types.
//!
//! These tests exercise construction, serialization roundtrips, trait impls,
//! ordering, edge cases, and invariants for every core type in the crate.
//! Test names follow `given_xxx_when_yyy_then_zzz` convention.

use tokmd_types::{
    AnalysisFormat, ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION,
    CONTEXT_SCHEMA_VERSION, CapabilityState, CapabilityStatus, ChildIncludeMode, ChildrenMode,
    CommitIntentKind, ConfigMode, ContextBundleManifest, ContextExcludedPath, ContextFileRow,
    ContextLogRecord, ContextReceipt, DiffReceipt, DiffRow, DiffTotals, ExportArgsMeta, ExportData,
    ExportFormat, ExportReceipt, FileClassification, FileKind, FileRow, HANDOFF_SCHEMA_VERSION,
    HandoffComplexity, HandoffDerived, HandoffExcludedPath, HandoffHotspot, HandoffIntelligence,
    HandoffManifest, InclusionPolicy, LangArgsMeta, LangReceipt, LangReport, LangRow,
    ModuleArgsMeta, ModuleReceipt, ModuleReport, ModuleRow, PolicyExcludedFile, RedactMode,
    RunReceipt, SCHEMA_VERSION, ScanArgs, ScanStatus, SmartExcludedFile, TableFormat, TokenAudit,
    TokenEstimationMeta, ToolInfo, Totals,
    cockpit::{
        COCKPIT_SCHEMA_VERSION, ChangeSurface, CockpitReceipt, CodeHealth, CommitMatch,
        ComplexityIndicator, Composition, Contracts, Evidence, EvidenceSource, GateMeta,
        GateStatus, HealthWarning, MutationGate, ReviewItem, Risk, RiskLevel, ScopeCoverage,
        TrendComparison, TrendDirection, TrendIndicator, TrendMetric, WarningType,
    },
};

// =============================================================================
// Helpers
// =============================================================================

fn make_scan_args() -> ScanArgs {
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

fn make_totals() -> Totals {
    Totals {
        code: 100,
        lines: 200,
        files: 10,
        bytes: 5000,
        tokens: 250,
        avg_lines: 20,
    }
}

fn make_lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines: code * 2,
        files: 1,
        bytes: code * 50,
        tokens: code * 5,
        avg_lines: code * 2,
    }
}

fn make_module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.to_string(),
        code,
        lines: code * 2,
        files: 1,
        bytes: code * 50,
        tokens: code * 5,
        avg_lines: code * 2,
    }
}

fn make_file_row() -> FileRow {
    FileRow {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 20,
        blanks: 10,
        lines: 130,
        bytes: 4000,
        tokens: 500,
    }
}

fn make_diff_row(lang: &str, old_code: usize, new_code: usize) -> DiffRow {
    DiffRow {
        lang: lang.to_string(),
        old_code,
        new_code,
        delta_code: new_code as i64 - old_code as i64,
        old_lines: old_code * 2,
        new_lines: new_code * 2,
        delta_lines: (new_code as i64 - old_code as i64) * 2,
        old_files: 5,
        new_files: 5,
        delta_files: 0,
        old_bytes: old_code * 50,
        new_bytes: new_code * 50,
        delta_bytes: (new_code as i64 - old_code as i64) * 50,
        old_tokens: old_code * 5,
        new_tokens: new_code * 5,
        delta_tokens: (new_code as i64 - old_code as i64) * 5,
    }
}

fn make_evidence() -> Evidence {
    Evidence {
        overall_status: GateStatus::Pass,
        mutation: MutationGate {
            meta: GateMeta {
                status: GateStatus::Pass,
                source: EvidenceSource::RanLocal,
                commit_match: CommitMatch::Exact,
                scope: ScopeCoverage {
                    relevant: vec![],
                    tested: vec![],
                    ratio: 1.0,
                    lines_relevant: None,
                    lines_tested: None,
                },
                evidence_commit: None,
                evidence_generated_at_ms: None,
            },
            survivors: vec![],
            killed: 10,
            timeout: 0,
            unviable: 0,
        },
        diff_coverage: None,
        contracts: None,
        supply_chain: None,
        determinism: None,
        complexity: None,
    }
}

// =============================================================================
// 1. LangRow
// =============================================================================

#[test]
fn given_lang_row_when_created_then_all_fields_set() {
    let row = make_lang_row("Rust", 500);
    assert_eq!(row.lang, "Rust");
    assert_eq!(row.code, 500);
    assert_eq!(row.lines, 1000);
    assert_eq!(row.files, 1);
    assert_eq!(row.bytes, 25000);
    assert_eq!(row.tokens, 2500);
    assert_eq!(row.avg_lines, 1000);
}

#[test]
fn given_lang_row_when_serialized_then_json_roundtrips() {
    let row = make_lang_row("Python", 300);
    let json = serde_json::to_string(&row).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn given_lang_row_when_cloned_then_equal_to_original() {
    let row = make_lang_row("Go", 200);
    let cloned = row.clone();
    assert_eq!(cloned, row);
}

#[test]
fn given_lang_row_when_debug_printed_then_contains_type_name() {
    let row = make_lang_row("Rust", 100);
    let dbg = format!("{:?}", row);
    assert!(dbg.contains("LangRow"));
    assert!(dbg.contains("Rust"));
}

#[test]
fn given_two_lang_rows_when_different_lang_then_not_equal() {
    let a = make_lang_row("Rust", 100);
    let b = make_lang_row("Python", 100);
    assert_ne!(a, b);
}

#[test]
fn given_two_lang_rows_when_different_code_then_not_equal() {
    let a = make_lang_row("Rust", 100);
    let b = make_lang_row("Rust", 200);
    assert_ne!(a, b);
}

#[test]
fn given_lang_row_when_json_has_all_fields_then_deserializes() {
    let json = r#"{"lang":"C++","code":999,"lines":1500,"files":3,"bytes":40000,"tokens":9990,"avg_lines":500}"#;
    let row: LangRow = serde_json::from_str(json).unwrap();
    assert_eq!(row.lang, "C++");
    assert_eq!(row.code, 999);
}

// =============================================================================
// 2. ModuleRow
// =============================================================================

#[test]
fn given_module_row_when_created_then_all_fields_set() {
    let row = make_module_row("src/lib", 400);
    assert_eq!(row.module, "src/lib");
    assert_eq!(row.code, 400);
}

#[test]
fn given_module_row_when_serialized_then_json_roundtrips() {
    let row = make_module_row("crates/core", 250);
    let json = serde_json::to_string(&row).unwrap();
    let back: ModuleRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn given_module_row_with_forward_slashes_when_serialized_then_preserved() {
    let row = make_module_row("src/utils/helpers", 100);
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("src/utils/helpers"));
}

#[test]
fn given_module_row_when_cloned_then_equal() {
    let row = make_module_row("tests", 50);
    assert_eq!(row.clone(), row);
}

#[test]
fn given_module_row_when_debug_printed_then_contains_module_name() {
    let row = make_module_row("src/api", 100);
    let dbg = format!("{:?}", row);
    assert!(dbg.contains("ModuleRow"));
    assert!(dbg.contains("src/api"));
}

// =============================================================================
// 3. FileRow / ExportRow
// =============================================================================

#[test]
fn given_file_row_when_created_then_all_fields_populated() {
    let row = make_file_row();
    assert_eq!(row.path, "src/main.rs");
    assert_eq!(row.module, "src");
    assert_eq!(row.lang, "Rust");
    assert_eq!(row.kind, FileKind::Parent);
    assert_eq!(row.code, 100);
    assert_eq!(row.comments, 20);
    assert_eq!(row.blanks, 10);
    assert_eq!(row.lines, 130);
    assert_eq!(row.bytes, 4000);
    assert_eq!(row.tokens, 500);
}

#[test]
fn given_file_row_when_serialized_then_json_roundtrips() {
    let row = make_file_row();
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn given_file_row_with_child_kind_when_serialized_then_kind_preserved() {
    let mut row = make_file_row();
    row.kind = FileKind::Child;
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.kind, FileKind::Child);
}

#[test]
fn given_file_row_when_cloned_and_modified_then_original_unchanged() {
    let row = make_file_row();
    let mut cloned = row.clone();
    cloned.code = 999;
    assert_eq!(row.code, 100);
    assert_eq!(cloned.code, 999);
}

// =============================================================================
// 4. LangReceipt / ModuleReceipt / ExportReceipt
// =============================================================================

#[test]
fn given_lang_receipt_when_constructed_then_schema_version_present() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 10,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![make_lang_row("Rust", 500)],
            total: make_totals(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 10,
        },
    };
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.report.rows.len(), 1);
}

#[test]
fn given_lang_receipt_when_serialized_then_json_roundtrips() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec!["test warning".to_string()],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 5,
            with_files: true,
            children: ChildrenMode::Separate,
        },
        report: LangReport {
            rows: vec![],
            total: make_totals(),
            with_files: true,
            children: ChildrenMode::Separate,
            top: 5,
        },
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.warnings, vec!["test warning"]);
}

#[test]
fn given_module_receipt_when_constructed_then_schema_version_present() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ModuleArgsMeta {
            format: "json".to_string(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 10,
        },
        report: ModuleReport {
            rows: vec![make_module_row("src", 200)],
            total: make_totals(),
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            top: 10,
        },
    };
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.mode, "module");
}

#[test]
fn given_module_receipt_when_serialized_then_json_roundtrips() {
    let receipt = ModuleReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 42,
        tool: ToolInfo::default(),
        mode: "module".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ModuleArgsMeta {
            format: "md".to_string(),
            module_roots: vec!["src".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::ParentsOnly,
            top: 0,
        },
        report: ModuleReport {
            rows: vec![],
            total: make_totals(),
            module_roots: vec!["src".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::ParentsOnly,
            top: 0,
        },
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
}

#[test]
fn given_export_receipt_when_constructed_then_schema_version_present() {
    let receipt = ExportReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "export".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: ExportArgsMeta {
            format: ExportFormat::Json,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
            min_code: 0,
            max_rows: 1000,
            redact: RedactMode::None,
            strip_prefix: None,
            strip_prefix_redacted: false,
        },
        data: ExportData {
            rows: vec![make_file_row()],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
    };
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.mode, "export");
    assert_eq!(receipt.data.rows.len(), 1);
}

#[test]
fn given_export_receipt_when_serialized_then_json_roundtrips() {
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
            min_code: 10,
            max_rows: 500,
            redact: RedactMode::Paths,
            strip_prefix: Some("/home/user".to_string()),
            strip_prefix_redacted: true,
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
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert!(back.args.strip_prefix_redacted);
}

// =============================================================================
// 5. DiffRow
// =============================================================================

#[test]
fn given_diff_row_when_positive_delta_then_delta_is_positive() {
    let row = make_diff_row("Rust", 100, 200);
    assert_eq!(row.delta_code, 100);
    assert!(row.delta_code > 0);
    assert!(row.delta_lines > 0);
    assert!(row.delta_bytes > 0);
    assert!(row.delta_tokens > 0);
}

#[test]
fn given_diff_row_when_negative_delta_then_delta_is_negative() {
    let row = make_diff_row("Rust", 200, 100);
    assert_eq!(row.delta_code, -100);
    assert!(row.delta_code < 0);
    assert!(row.delta_lines < 0);
}

#[test]
fn given_diff_row_when_zero_delta_then_all_deltas_zero() {
    let row = make_diff_row("Rust", 100, 100);
    assert_eq!(row.delta_code, 0);
    assert_eq!(row.delta_lines, 0);
    assert_eq!(row.delta_bytes, 0);
    assert_eq!(row.delta_tokens, 0);
    assert_eq!(row.delta_files, 0);
}

#[test]
fn given_diff_row_when_serialized_then_json_roundtrips() {
    let row = make_diff_row("Python", 50, 75);
    let json = serde_json::to_string(&row).unwrap();
    let back: DiffRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn given_diff_totals_when_default_then_all_zeros() {
    let t = DiffTotals::default();
    assert_eq!(t.old_code, 0);
    assert_eq!(t.new_code, 0);
    assert_eq!(t.delta_code, 0);
    assert_eq!(t.old_lines, 0);
    assert_eq!(t.new_lines, 0);
    assert_eq!(t.delta_lines, 0);
    assert_eq!(t.old_files, 0);
    assert_eq!(t.new_files, 0);
    assert_eq!(t.delta_files, 0);
    assert_eq!(t.old_bytes, 0);
    assert_eq!(t.new_bytes, 0);
    assert_eq!(t.delta_bytes, 0);
    assert_eq!(t.old_tokens, 0);
    assert_eq!(t.new_tokens, 0);
    assert_eq!(t.delta_tokens, 0);
}

#[test]
fn given_diff_receipt_when_constructed_then_contains_metadata() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "diff".to_string(),
        from_source: "v1.0".to_string(),
        to_source: "v2.0".to_string(),
        diff_rows: vec![make_diff_row("Rust", 100, 150)],
        totals: DiffTotals::default(),
    };
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.from_source, "v1.0");
    assert_eq!(receipt.to_source, "v2.0");
    assert_eq!(receipt.diff_rows.len(), 1);
}

#[test]
fn given_diff_receipt_when_serialized_then_json_roundtrips() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 42,
        tool: ToolInfo::default(),
        mode: "diff".to_string(),
        from_source: "a.json".to_string(),
        to_source: "b.json".to_string(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: DiffReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.from_source, "a.json");
}

// =============================================================================
// 6. ChildrenMode
// =============================================================================

#[test]
fn given_children_mode_collapse_when_serialized_then_kebab_case() {
    let json = serde_json::to_string(&ChildrenMode::Collapse).unwrap();
    assert_eq!(json, "\"collapse\"");
}

#[test]
fn given_children_mode_separate_when_serialized_then_kebab_case() {
    let json = serde_json::to_string(&ChildrenMode::Separate).unwrap();
    assert_eq!(json, "\"separate\"");
}

#[test]
fn given_children_mode_when_roundtripped_then_all_variants_preserved() {
    for variant in [ChildrenMode::Collapse, ChildrenMode::Separate] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_child_include_mode_when_roundtripped_then_all_variants_preserved() {
    for variant in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_child_include_parents_only_when_serialized_then_kebab_case() {
    let json = serde_json::to_string(&ChildIncludeMode::ParentsOnly).unwrap();
    assert_eq!(json, "\"parents-only\"");
}

// =============================================================================
// 7. OutputFormat (TableFormat / ExportFormat / AnalysisFormat)
// =============================================================================

#[test]
fn given_table_format_when_all_variants_serialized_then_kebab_case() {
    assert_eq!(serde_json::to_string(&TableFormat::Md).unwrap(), "\"md\"");
    assert_eq!(serde_json::to_string(&TableFormat::Tsv).unwrap(), "\"tsv\"");
    assert_eq!(
        serde_json::to_string(&TableFormat::Json).unwrap(),
        "\"json\""
    );
}

#[test]
fn given_export_format_when_all_variants_serialized_then_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ExportFormat::Csv).unwrap(),
        "\"csv\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Jsonl).unwrap(),
        "\"jsonl\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Json).unwrap(),
        "\"json\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Cyclonedx).unwrap(),
        "\"cyclonedx\""
    );
}

#[test]
fn given_analysis_format_when_all_variants_roundtripped_then_preserved() {
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
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: AnalysisFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// =============================================================================
// 8. Schema version constants
// =============================================================================

#[test]
fn given_schema_versions_when_checked_then_all_positive_nonzero() {
    const { assert!(SCHEMA_VERSION > 0); }
    const { assert!(COCKPIT_SCHEMA_VERSION > 0); }
    const { assert!(HANDOFF_SCHEMA_VERSION > 0); }
    const { assert!(CONTEXT_SCHEMA_VERSION > 0); }
    const { assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0); }
}

#[test]
fn given_schema_versions_when_checked_then_match_documented_values() {
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

// =============================================================================
// 9. RunReceipt
// =============================================================================

#[test]
fn given_run_receipt_when_constructed_then_all_fields_set() {
    let receipt = RunReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        lang_file: "lang.json".to_string(),
        module_file: "module.json".to_string(),
        export_file: "export.jsonl".to_string(),
    };
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.lang_file, "lang.json");
    assert_eq!(receipt.module_file, "module.json");
    assert_eq!(receipt.export_file, "export.jsonl");
}

#[test]
fn given_run_receipt_when_serialized_then_json_roundtrips() {
    let receipt = RunReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 42,
        lang_file: "a.json".to_string(),
        module_file: "b.json".to_string(),
        export_file: "c.jsonl".to_string(),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: RunReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.lang_file, "a.json");
}

// =============================================================================
// 10. CockpitReceipt
// =============================================================================

#[test]
fn given_cockpit_receipt_when_constructed_then_schema_version_matches() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 1_700_000_000,
        base_ref: "main".to_string(),
        head_ref: "feature/x".to_string(),
        change_surface: ChangeSurface {
            commits: 5,
            files_changed: 10,
            insertions: 200,
            deletions: 50,
            net_lines: 150,
            churn_velocity: 50.0,
            change_concentration: 0.6,
        },
        composition: Composition {
            code_pct: 80.0,
            test_pct: 15.0,
            docs_pct: 3.0,
            config_pct: 2.0,
            test_ratio: 0.19,
        },
        code_health: CodeHealth {
            score: 90,
            grade: "A".to_string(),
            large_files_touched: 0,
            avg_file_size: 150,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 5,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: make_evidence(),
        review_plan: vec![],
        trend: None,
    };
    assert_eq!(receipt.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(receipt.mode, "cockpit");
    assert_eq!(receipt.change_surface.commits, 5);
    assert_eq!(receipt.code_health.grade, "A");
}

#[test]
fn given_cockpit_receipt_when_serialized_then_json_roundtrips() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 0,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
        change_surface: ChangeSurface {
            commits: 0,
            files_changed: 0,
            insertions: 0,
            deletions: 0,
            net_lines: 0,
            churn_velocity: 0.0,
            change_concentration: 0.0,
        },
        composition: Composition {
            code_pct: 0.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        },
        code_health: CodeHealth {
            score: 0,
            grade: "F".to_string(),
            large_files_touched: 0,
            avg_file_size: 0,
            complexity_indicator: ComplexityIndicator::Critical,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Critical,
            score: 100,
        },
        contracts: Contracts {
            api_changed: true,
            cli_changed: true,
            schema_changed: true,
            breaking_indicators: 3,
        },
        evidence: make_evidence(),
        review_plan: vec![ReviewItem {
            path: "src/main.rs".to_string(),
            reason: "high churn".to_string(),
            priority: 1,
            complexity: Some(4),
            lines_changed: Some(100),
        }],
        trend: None,
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, COCKPIT_SCHEMA_VERSION);
    assert_eq!(back.review_plan.len(), 1);
    assert_eq!(back.risk.level, RiskLevel::Critical);
}

#[test]
fn given_cockpit_receipt_with_trend_when_serialized_then_trend_present() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 0,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
            files_changed: 1,
            insertions: 1,
            deletions: 0,
            net_lines: 1,
            churn_velocity: 1.0,
            change_concentration: 1.0,
        },
        composition: Composition {
            code_pct: 100.0,
            test_pct: 0.0,
            docs_pct: 0.0,
            config_pct: 0.0,
            test_ratio: 0.0,
        },
        code_health: CodeHealth {
            score: 50,
            grade: "C".to_string(),
            large_files_touched: 0,
            avg_file_size: 50,
            complexity_indicator: ComplexityIndicator::Medium,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Medium,
            score: 50,
        },
        contracts: Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        },
        evidence: make_evidence(),
        review_plan: vec![],
        trend: Some(TrendComparison {
            baseline_available: true,
            baseline_path: Some("baseline.json".to_string()),
            baseline_generated_at_ms: Some(1_600_000_000),
            health: Some(TrendMetric {
                current: 50.0,
                previous: 45.0,
                delta: 5.0,
                delta_pct: 11.1,
                direction: TrendDirection::Improving,
            }),
            risk: Some(TrendMetric {
                current: 50.0,
                previous: 60.0,
                delta: -10.0,
                delta_pct: -16.7,
                direction: TrendDirection::Improving,
            }),
            complexity: Some(TrendIndicator {
                direction: TrendDirection::Stable,
                summary: "No significant change".to_string(),
                files_increased: 1,
                files_decreased: 1,
                avg_cyclomatic_delta: Some(0.0),
                avg_cognitive_delta: None,
            }),
        }),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: CockpitReceipt = serde_json::from_str(&json).unwrap();
    assert!(back.trend.is_some());
    let trend = back.trend.unwrap();
    assert!(trend.baseline_available);
    assert_eq!(trend.health.unwrap().direction, TrendDirection::Improving);
}

// =============================================================================
// 11. HandoffManifest
// =============================================================================

#[test]
fn given_handoff_manifest_when_constructed_then_schema_version_matches() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "handoff".to_string(),
        inputs: vec![".".to_string()],
        output_dir: "out/".to_string(),
        budget_tokens: 100_000,
        used_tokens: 80_000,
        utilization_pct: 80.0,
        strategy: "greedy".to_string(),
        rank_by: "tokens".to_string(),
        capabilities: vec![],
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![],
        excluded_patterns: vec![],
        smart_excluded_files: vec![],
        total_files: 50,
        bundled_files: 40,
        intelligence_preset: "health".to_string(),
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        code_audit: None,
    };
    assert_eq!(manifest.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(manifest.mode, "handoff");
    assert_eq!(manifest.budget_tokens, 100_000);
}

#[test]
fn given_handoff_manifest_when_serialized_then_json_roundtrips() {
    let manifest = HandoffManifest {
        schema_version: HANDOFF_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo::default(),
        mode: "handoff".to_string(),
        inputs: vec![],
        output_dir: ".".to_string(),
        budget_tokens: 0,
        used_tokens: 0,
        utilization_pct: 0.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        capabilities: vec![CapabilityStatus {
            name: "git".to_string(),
            status: CapabilityState::Available,
            reason: None,
        }],
        artifacts: vec![ArtifactEntry {
            name: "bundle.txt".to_string(),
            path: "out/bundle.txt".to_string(),
            description: "Code bundle".to_string(),
            bytes: 1000,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: "abc123".to_string(),
            }),
        }],
        included_files: vec![],
        excluded_paths: vec![HandoffExcludedPath {
            path: "target/".to_string(),
            reason: "build output".to_string(),
        }],
        excluded_patterns: vec!["*.o".to_string()],
        smart_excluded_files: vec![SmartExcludedFile {
            path: "Cargo.lock".to_string(),
            reason: "lockfile".to_string(),
            tokens: 5000,
        }],
        total_files: 100,
        bundled_files: 50,
        intelligence_preset: "deep".to_string(),
        rank_by_effective: Some("code".to_string()),
        fallback_reason: Some("git unavailable".to_string()),
        excluded_by_policy: vec![PolicyExcludedFile {
            path: "generated.rs".to_string(),
            original_tokens: 10000,
            policy: InclusionPolicy::Skip,
            reason: "generated file".to_string(),
            classifications: vec![FileClassification::Generated],
        }],
        token_estimation: Some(TokenEstimationMeta::from_bytes(10000, 4.0)),
        code_audit: Some(TokenAudit::from_output(8000, 7000)),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    let back: HandoffManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, HANDOFF_SCHEMA_VERSION);
    assert_eq!(back.capabilities.len(), 1);
    assert_eq!(back.artifacts.len(), 1);
    assert!(back.token_estimation.is_some());
    assert!(back.code_audit.is_some());
}

// =============================================================================
// 12. ContextReceipt
// =============================================================================

#[test]
fn given_context_receipt_when_constructed_then_schema_version_matches() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "context".to_string(),
        budget_tokens: 128_000,
        used_tokens: 100_000,
        utilization_pct: 78.1,
        strategy: "greedy".to_string(),
        rank_by: "tokens".to_string(),
        file_count: 25,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };
    assert_eq!(receipt.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(receipt.mode, "context");
    assert_eq!(receipt.budget_tokens, 128_000);
}

#[test]
fn given_context_receipt_with_files_when_serialized_then_json_roundtrips() {
    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 42,
        tool: ToolInfo::default(),
        mode: "context".to_string(),
        budget_tokens: 50_000,
        used_tokens: 40_000,
        utilization_pct: 80.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 1,
        files: vec![ContextFileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            tokens: 5000,
            code: 200,
            lines: 300,
            bytes: 20000,
            value: 5000,
            rank_reason: "high code density".to_string(),
            policy: InclusionPolicy::Full,
            effective_tokens: None,
            policy_reason: None,
            classifications: vec![],
        }],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: Some(TokenEstimationMeta::from_bytes(20000, 4.0)),
        bundle_audit: None,
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.files.len(), 1);
    assert_eq!(back.files[0].path, "src/lib.rs");
}

#[test]
fn given_context_bundle_manifest_when_constructed_then_schema_version_matches() {
    let manifest = ContextBundleManifest {
        schema_version: CONTEXT_BUNDLE_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo::default(),
        mode: "context-bundle".to_string(),
        budget_tokens: 100_000,
        used_tokens: 80_000,
        utilization_pct: 80.0,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 10,
        bundle_bytes: 50_000,
        artifacts: vec![],
        included_files: vec![],
        excluded_paths: vec![ContextExcludedPath {
            path: "target/".to_string(),
            reason: "build output".to_string(),
        }],
        excluded_patterns: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };
    assert_eq!(manifest.schema_version, CONTEXT_BUNDLE_SCHEMA_VERSION);
}

#[test]
fn given_context_log_record_when_serialized_then_json_roundtrips() {
    let record = ContextLogRecord {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        budget_tokens: 50_000,
        used_tokens: 40_000,
        utilization_pct: 80.0,
        strategy: "greedy".to_string(),
        rank_by: "tokens".to_string(),
        file_count: 10,
        total_bytes: 200_000,
        output_destination: "stdout".to_string(),
    };
    let json = serde_json::to_string(&record).unwrap();
    let back: ContextLogRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.file_count, 10);
}

// =============================================================================
// 13. TokenEstimation
// =============================================================================

#[test]
fn given_token_estimation_when_from_bytes_then_invariant_min_le_est_le_max() {
    for bytes in [0, 1, 100, 1000, 10_000, 100_000, 1_000_000] {
        let est = TokenEstimationMeta::from_bytes(bytes, 4.0);
        assert!(
            est.tokens_min <= est.tokens_est,
            "min ({}) <= est ({}) for bytes={}",
            est.tokens_min,
            est.tokens_est,
            bytes
        );
        assert!(
            est.tokens_est <= est.tokens_max,
            "est ({}) <= max ({}) for bytes={}",
            est.tokens_est,
            est.tokens_max,
            bytes
        );
    }
}

#[test]
fn given_token_estimation_when_zero_bytes_then_all_tokens_zero() {
    let est = TokenEstimationMeta::from_bytes(0, 4.0);
    assert_eq!(est.tokens_min, 0);
    assert_eq!(est.tokens_est, 0);
    assert_eq!(est.tokens_max, 0);
    assert_eq!(est.source_bytes, 0);
}

#[test]
fn given_token_estimation_when_exact_divisor_then_no_rounding() {
    let est = TokenEstimationMeta::from_bytes(4000, 4.0);
    assert_eq!(est.tokens_est, 1000);
    assert_eq!(est.source_bytes, 4000);
}

#[test]
fn given_token_estimation_when_custom_bounds_then_applied() {
    let est = TokenEstimationMeta::from_bytes_with_bounds(1000, 4.0, 2.0, 8.0);
    assert_eq!(est.tokens_est, 250); // 1000 / 4.0
    assert_eq!(est.tokens_min, 125); // 1000 / 8.0
    assert_eq!(est.tokens_max, 500); // 1000 / 2.0
    assert_eq!(est.bytes_per_token_est, 4.0);
    assert_eq!(est.bytes_per_token_low, 2.0);
    assert_eq!(est.bytes_per_token_high, 8.0);
}

#[test]
fn given_token_estimation_when_serialized_then_json_roundtrips() {
    let est = TokenEstimationMeta::from_bytes(5000, 4.0);
    let json = serde_json::to_string(&est).unwrap();
    let back: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.source_bytes, 5000);
    assert_eq!(back.tokens_est, est.tokens_est);
}

#[test]
fn given_token_estimation_defaults_then_constants_correct() {
    assert_eq!(TokenEstimationMeta::DEFAULT_BPT_EST, 4.0);
    assert_eq!(TokenEstimationMeta::DEFAULT_BPT_LOW, 3.0);
    assert_eq!(TokenEstimationMeta::DEFAULT_BPT_HIGH, 5.0);
}

#[test]
fn given_token_audit_when_from_output_then_overhead_calculated() {
    let audit = TokenAudit::from_output(10_000, 8_000);
    assert_eq!(audit.output_bytes, 10_000);
    assert_eq!(audit.overhead_bytes, 2_000);
    assert!((audit.overhead_pct - 0.2).abs() < f64::EPSILON);
}

#[test]
fn given_token_audit_when_zero_output_then_no_division_error() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.overhead_pct, 0.0);
    assert_eq!(audit.tokens_est, 0);
}

#[test]
fn given_token_audit_when_content_exceeds_output_then_saturates() {
    let audit = TokenAudit::from_output(100, 500);
    assert_eq!(audit.overhead_bytes, 0);
    assert_eq!(audit.overhead_pct, 0.0);
}

// =============================================================================
// 14. Edge cases
// =============================================================================

#[test]
fn given_lang_row_when_empty_string_lang_then_ok() {
    let row = LangRow {
        lang: String::new(),
        code: 0,
        lines: 0,
        files: 0,
        bytes: 0,
        tokens: 0,
        avg_lines: 0,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
    assert!(back.lang.is_empty());
}

#[test]
fn given_module_row_when_empty_module_name_then_ok() {
    let row = ModuleRow {
        module: String::new(),
        code: 0,
        lines: 0,
        files: 0,
        bytes: 0,
        tokens: 0,
        avg_lines: 0,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: ModuleRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn given_lang_row_when_zero_values_then_serializes() {
    let row = LangRow {
        lang: "Empty".to_string(),
        code: 0,
        lines: 0,
        files: 0,
        bytes: 0,
        tokens: 0,
        avg_lines: 0,
    };
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("\"code\":0"));
}

#[test]
fn given_lang_row_when_max_usize_values_then_serializes() {
    let row = LangRow {
        lang: "Max".to_string(),
        code: usize::MAX,
        lines: usize::MAX,
        files: usize::MAX,
        bytes: usize::MAX,
        tokens: usize::MAX,
        avg_lines: usize::MAX,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.code, usize::MAX);
}

#[test]
fn given_diff_row_when_large_negative_delta_then_serializes() {
    let row = DiffRow {
        lang: "Rust".to_string(),
        old_code: usize::MAX,
        new_code: 0,
        delta_code: i64::MIN,
        old_lines: 0,
        new_lines: 0,
        delta_lines: i64::MIN,
        old_files: 0,
        new_files: 0,
        delta_files: i64::MIN,
        old_bytes: 0,
        new_bytes: 0,
        delta_bytes: i64::MIN,
        old_tokens: 0,
        new_tokens: 0,
        delta_tokens: i64::MIN,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: DiffRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.delta_code, i64::MIN);
}

#[test]
fn given_file_row_when_unicode_path_then_roundtrips() {
    let row = FileRow {
        path: "src/日本語/ファイル.rs".to_string(),
        module: "src/日本語".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 500,
        tokens: 50,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/日本語/ファイル.rs");
}

#[test]
fn given_totals_when_cloned_then_independent() {
    let t = make_totals();
    let mut cloned = t.clone();
    cloned.code = 999;
    assert_eq!(t.code, 100);
    assert_eq!(cloned.code, 999);
}

#[test]
fn given_tool_info_default_when_checked_then_empty_strings() {
    let ti = ToolInfo::default();
    assert!(ti.name.is_empty());
    assert!(ti.version.is_empty());
}

#[test]
fn given_tool_info_current_when_checked_then_name_is_tokmd() {
    let ti = ToolInfo::current();
    assert_eq!(ti.name, "tokmd");
    assert!(!ti.version.is_empty());
}

#[test]
fn given_config_mode_default_then_is_auto() {
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn given_inclusion_policy_default_then_is_full() {
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

// =============================================================================
// Additional cockpit enum edge cases
// =============================================================================

#[test]
fn given_gate_status_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_evidence_source_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_commit_match_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CommitMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_complexity_indicator_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: ComplexityIndicator = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_warning_type_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: WarningType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_risk_level_when_display_then_lowercase() {
    assert_eq!(RiskLevel::Low.to_string(), "low");
    assert_eq!(RiskLevel::Medium.to_string(), "medium");
    assert_eq!(RiskLevel::High.to_string(), "high");
    assert_eq!(RiskLevel::Critical.to_string(), "critical");
}

#[test]
fn given_trend_direction_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Degrading,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_file_classification_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_capability_state_when_all_variants_roundtripped_then_preserved() {
    for variant in [
        CapabilityState::Available,
        CapabilityState::Skipped,
        CapabilityState::Unavailable,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_commit_intent_kind_when_all_variants_roundtripped_then_preserved() {
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
fn given_scan_status_when_both_variants_roundtripped_then_preserved() {
    let json_complete = serde_json::to_string(&ScanStatus::Complete).unwrap();
    assert_eq!(json_complete, "\"complete\"");
    let json_partial = serde_json::to_string(&ScanStatus::Partial).unwrap();
    assert_eq!(json_partial, "\"partial\"");
}

#[test]
fn given_redact_mode_when_all_variants_roundtripped_then_preserved() {
    for variant in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn given_file_kind_when_ordering_then_parent_before_child() {
    assert!(FileKind::Parent < FileKind::Child);
}

// =============================================================================
// Handoff intelligence types
// =============================================================================

#[test]
fn given_handoff_intelligence_when_all_none_then_serializes() {
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
    assert!(back.hotspots.is_none());
}

#[test]
fn given_handoff_intelligence_when_fully_populated_then_roundtrips() {
    let intel = HandoffIntelligence {
        tree: Some("src/\n  lib.rs\n  main.rs".to_string()),
        tree_depth: Some(3),
        hotspots: Some(vec![HandoffHotspot {
            path: "src/lib.rs".to_string(),
            commits: 50,
            lines: 500,
            score: 25000,
        }]),
        complexity: Some(HandoffComplexity {
            total_functions: 100,
            avg_function_length: 20.5,
            max_function_length: 150,
            avg_cyclomatic: 3.2,
            max_cyclomatic: 15,
            high_risk_files: 2,
        }),
        derived: Some(HandoffDerived {
            total_files: 50,
            total_code: 10000,
            total_lines: 15000,
            total_tokens: 50000,
            lang_count: 3,
            dominant_lang: "Rust".to_string(),
            dominant_pct: 85.5,
        }),
        warnings: vec!["Some warning".to_string()],
    };
    let json = serde_json::to_string(&intel).unwrap();
    let back: HandoffIntelligence = serde_json::from_str(&json).unwrap();
    assert!(back.tree.is_some());
    assert_eq!(back.hotspots.unwrap().len(), 1);
    assert_eq!(back.complexity.unwrap().total_functions, 100);
    assert_eq!(back.derived.unwrap().dominant_lang, "Rust");
}

// =============================================================================
// Health warning construction
// =============================================================================

#[test]
fn given_health_warning_when_constructed_then_all_fields_set() {
    let warning = HealthWarning {
        path: "src/big_file.rs".to_string(),
        warning_type: WarningType::LargeFile,
        message: "File exceeds 500 lines".to_string(),
    };
    assert_eq!(warning.path, "src/big_file.rs");
    assert_eq!(warning.warning_type, WarningType::LargeFile);
}

// =============================================================================
// Trend types edge cases
// =============================================================================

#[test]
fn given_trend_comparison_default_when_checked_then_empty() {
    let trend = TrendComparison::default();
    assert!(!trend.baseline_available);
    assert!(trend.baseline_path.is_none());
    assert!(trend.health.is_none());
    assert!(trend.risk.is_none());
    assert!(trend.complexity.is_none());
}

#[test]
fn given_trend_metric_when_serialized_then_roundtrips() {
    let metric = TrendMetric {
        current: 85.0,
        previous: 80.0,
        delta: 5.0,
        delta_pct: 6.25,
        direction: TrendDirection::Improving,
    };
    let json = serde_json::to_string(&metric).unwrap();
    let back: TrendMetric = serde_json::from_str(&json).unwrap();
    assert_eq!(back.direction, TrendDirection::Improving);
    assert!((back.delta - 5.0).abs() < f64::EPSILON);
}

// =============================================================================
// ContextFileRow with policy variations
// =============================================================================

#[test]
fn given_context_file_row_with_skip_policy_when_serialized_then_policy_in_json() {
    let row = ContextFileRow {
        path: "gen/output.rs".to_string(),
        module: "gen".to_string(),
        lang: "Rust".to_string(),
        tokens: 10000,
        code: 500,
        lines: 800,
        bytes: 40000,
        value: 100,
        rank_reason: String::new(),
        policy: InclusionPolicy::Skip,
        effective_tokens: Some(0),
        policy_reason: Some("generated file".to_string()),
        classifications: vec![FileClassification::Generated],
    };
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("\"skip\""));
    let back: ContextFileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.policy, InclusionPolicy::Skip);
    assert_eq!(back.effective_tokens, Some(0));
}

#[test]
fn given_context_file_row_with_head_tail_policy_when_serialized_then_roundtrips() {
    let row = ContextFileRow {
        path: "src/big.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 50000,
        code: 2000,
        lines: 3000,
        bytes: 200000,
        value: 1000,
        rank_reason: "high value".to_string(),
        policy: InclusionPolicy::HeadTail,
        effective_tokens: Some(5000),
        policy_reason: Some("exceeds per-file cap".to_string()),
        classifications: vec![],
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: ContextFileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.policy, InclusionPolicy::HeadTail);
    assert_eq!(back.effective_tokens, Some(5000));
}

// =============================================================================
// ScanArgs edge case
// =============================================================================

#[test]
fn given_scan_args_with_all_flags_true_when_serialized_then_roundtrips() {
    let args = ScanArgs {
        paths: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        excluded: vec!["*.log".to_string()],
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
    assert_eq!(back.paths.len(), 3);
    assert!(back.excluded_redacted);
    assert!(back.hidden);
    assert!(back.no_ignore);
    assert!(back.treat_doc_strings_as_comments);
}

// =============================================================================
// Inclusion policy ordering
// =============================================================================

#[test]
fn given_inclusion_policies_when_ordered_then_full_is_smallest() {
    assert!(InclusionPolicy::Full < InclusionPolicy::HeadTail);
    assert!(InclusionPolicy::HeadTail < InclusionPolicy::Summary);
    assert!(InclusionPolicy::Summary < InclusionPolicy::Skip);
}

#[test]
fn given_file_classification_when_ordered_then_deterministic() {
    // Just verify ordering doesn't panic and is consistent
    let mut items = vec![
        FileClassification::Sourcemap,
        FileClassification::Generated,
        FileClassification::Lockfile,
    ];
    items.sort();
    assert_eq!(items[0], FileClassification::Generated);
}
