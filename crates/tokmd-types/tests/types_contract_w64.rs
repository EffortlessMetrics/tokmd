//! W64 contract tests for `tokmd-types`.
//!
//! ~60 tests covering struct construction, schema constants, serde round-trips,
//! deterministic JSON output, enum variants, receipt envelopes, property-based
//! arithmetic, BDD-style scenarios, and edge cases.

use std::collections::BTreeMap;

use tokmd_types::cockpit::*;
use tokmd_types::*;

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Schema version constants
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn schema_version_at_least_minimum() {
    const { assert!(SCHEMA_VERSION >= 1) };
    const { assert!(HANDOFF_SCHEMA_VERSION >= 1) };
    const { assert!(CONTEXT_SCHEMA_VERSION >= 1) };
    const { assert!(CONTEXT_BUNDLE_SCHEMA_VERSION >= 1) };
    const { assert!(COCKPIT_SCHEMA_VERSION >= 1) };
}

#[test]
fn schema_version_exact_values() {
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Struct construction
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn totals_construction() {
    let t = Totals {
        code: 100,
        lines: 150,
        files: 5,
        bytes: 4000,
        tokens: 250,
        avg_lines: 30,
    };
    assert_eq!(t.code, 100);
    assert_eq!(t.lines, 150);
    assert_eq!(t.files, 5);
    assert_eq!(t.bytes, 4000);
    assert_eq!(t.tokens, 250);
    assert_eq!(t.avg_lines, 30);
}

#[test]
fn lang_row_construction() {
    let row = LangRow {
        lang: "Rust".to_string(),
        code: 5000,
        lines: 6500,
        files: 42,
        bytes: 180_000,
        tokens: 45_000,
        avg_lines: 154,
    };
    assert_eq!(row.lang, "Rust");
    assert_eq!(row.files, 42);
}

#[test]
fn module_row_construction() {
    let row = ModuleRow {
        module: "crates/tokmd-types".to_string(),
        code: 800,
        lines: 1100,
        files: 3,
        bytes: 32_000,
        tokens: 8_000,
        avg_lines: 366,
    };
    assert_eq!(row.module, "crates/tokmd-types");
    assert_eq!(row.code, 800);
}

#[test]
fn file_row_construction() {
    let row = FileRow {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 120,
        comments: 30,
        blanks: 20,
        lines: 170,
        bytes: 4_800,
        tokens: 1_200,
    };
    assert_eq!(row.path, "src/main.rs");
    assert_eq!(row.kind, FileKind::Parent);
    assert_eq!(row.lines, row.code + row.comments + row.blanks);
}

#[test]
fn scan_status_construction() {
    let complete = ScanStatus::Complete;
    let partial = ScanStatus::Partial;
    let j1 = serde_json::to_string(&complete).unwrap();
    let j2 = serde_json::to_string(&partial).unwrap();
    assert_eq!(j1, "\"complete\"");
    assert_eq!(j2, "\"partial\"");
}

#[test]
fn tool_info_default_construction() {
    let ti = ToolInfo::default();
    assert!(ti.name.is_empty());
    assert!(ti.version.is_empty());
}

#[test]
fn tool_info_current_construction() {
    let ti = ToolInfo::current();
    assert_eq!(ti.name, "tokmd");
    assert!(!ti.version.is_empty());
}

#[test]
fn diff_row_construction() {
    let row = DiffRow {
        lang: "Python".to_string(),
        old_code: 500,
        new_code: 600,
        delta_code: 100,
        old_lines: 700,
        new_lines: 850,
        delta_lines: 150,
        old_files: 8,
        new_files: 10,
        delta_files: 2,
        old_bytes: 20000,
        new_bytes: 25000,
        delta_bytes: 5000,
        old_tokens: 5000,
        new_tokens: 6000,
        delta_tokens: 1000,
    };
    assert_eq!(
        row.delta_code,
        (row.new_code as i64) - (row.old_code as i64)
    );
    assert_eq!(
        row.delta_files,
        (row.new_files as i64) - (row.old_files as i64)
    );
}

#[test]
fn diff_totals_default_is_all_zero() {
    let dt = DiffTotals::default();
    assert_eq!(dt.old_code, 0);
    assert_eq!(dt.new_code, 0);
    assert_eq!(dt.delta_code, 0);
    assert_eq!(dt.delta_tokens, 0);
    assert_eq!(dt.delta_files, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Enum variant exhaustiveness
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn file_kind_variants() {
    let variants = [FileKind::Parent, FileKind::Child];
    assert_eq!(variants.len(), 2);
    assert_ne!(variants[0], variants[1]);
}

#[test]
fn children_mode_variants() {
    let variants = [ChildrenMode::Collapse, ChildrenMode::Separate];
    assert_eq!(variants.len(), 2);
    assert_ne!(variants[0], variants[1]);
}

#[test]
fn child_include_mode_variants() {
    let variants = [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly];
    assert_eq!(variants.len(), 2);
    assert_ne!(variants[0], variants[1]);
}

#[test]
fn config_mode_default_is_auto() {
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn redact_mode_all_variants() {
    let variants = [RedactMode::None, RedactMode::Paths, RedactMode::All];
    assert_eq!(variants.len(), 3);
    // All distinct
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn export_format_variants() {
    let variants = [
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ];
    assert_eq!(variants.len(), 4);
}

#[test]
fn table_format_variants() {
    let variants = [TableFormat::Md, TableFormat::Tsv, TableFormat::Json];
    assert_eq!(variants.len(), 3);
}

#[test]
fn inclusion_policy_default_is_full() {
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

#[test]
fn inclusion_policy_variants() {
    let variants = [
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ];
    assert_eq!(variants.len(), 4);
}

#[test]
fn file_classification_variants() {
    let variants = [
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ];
    assert_eq!(variants.len(), 7);
}

#[test]
fn capability_state_variants() {
    let variants = [
        CapabilityState::Available,
        CapabilityState::Skipped,
        CapabilityState::Unavailable,
    ];
    assert_eq!(variants.len(), 3);
}

#[test]
fn analysis_format_all_variants() {
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
    assert_eq!(variants.len(), 10);
}

#[test]
fn commit_intent_kind_all_variants() {
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
    assert_eq!(variants.len(), 12);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Serde round-trips for structs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn totals_serde_roundtrip() {
    let t = Totals {
        code: 1234,
        lines: 5678,
        files: 99,
        bytes: 100_000,
        tokens: 25_000,
        avg_lines: 57,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Totals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

#[test]
fn lang_row_serde_roundtrip() {
    let row = LangRow {
        lang: "TypeScript".to_string(),
        code: 3000,
        lines: 4000,
        files: 20,
        bytes: 90_000,
        tokens: 30_000,
        avg_lines: 200,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn module_row_serde_roundtrip() {
    let row = ModuleRow {
        module: "src/utils".to_string(),
        code: 400,
        lines: 600,
        files: 8,
        bytes: 16_000,
        tokens: 4_000,
        avg_lines: 75,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: ModuleRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn file_row_serde_roundtrip() {
    let row = FileRow {
        path: "lib/parser.rs".to_string(),
        module: "lib".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 300,
        comments: 50,
        blanks: 30,
        lines: 380,
        bytes: 12_000,
        tokens: 3_000,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn diff_row_serde_roundtrip() {
    let row = DiffRow {
        lang: "Go".to_string(),
        old_code: 200,
        new_code: 250,
        delta_code: 50,
        old_lines: 300,
        new_lines: 380,
        delta_lines: 80,
        old_files: 5,
        new_files: 6,
        delta_files: 1,
        old_bytes: 8000,
        new_bytes: 10000,
        delta_bytes: 2000,
        old_tokens: 2000,
        new_tokens: 2500,
        delta_tokens: 500,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: DiffRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn diff_totals_serde_roundtrip() {
    let t = DiffTotals {
        old_code: 1000,
        new_code: 1200,
        delta_code: 200,
        ..DiffTotals::default()
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: DiffTotals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Deterministic JSON output (sorted keys)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn totals_json_deterministic() {
    let t = Totals {
        code: 10,
        lines: 20,
        files: 2,
        bytes: 100,
        tokens: 25,
        avg_lines: 10,
    };
    let json1 = serde_json::to_string(&t).unwrap();
    let json2 = serde_json::to_string(&t).unwrap();
    assert_eq!(json1, json2, "JSON output must be deterministic");
}

#[test]
fn lang_row_json_field_order_stable() {
    let row = LangRow {
        lang: "C".to_string(),
        code: 1,
        lines: 2,
        files: 1,
        bytes: 10,
        tokens: 3,
        avg_lines: 2,
    };
    let json = serde_json::to_string(&row).unwrap();
    // "lang" should appear before "code" in serde output (field declaration order)
    let lang_pos = json.find("\"lang\"").unwrap();
    let code_pos = json.find("\"code\"").unwrap();
    assert!(lang_pos < code_pos);
}

#[test]
fn btreemap_ensures_sorted_keys() {
    let mut map = BTreeMap::new();
    map.insert("z_lang".to_string(), 1usize);
    map.insert("a_lang".to_string(), 2usize);
    map.insert("m_lang".to_string(), 3usize);
    let json = serde_json::to_string(&map).unwrap();
    let a_pos = json.find("\"a_lang\"").unwrap();
    let m_pos = json.find("\"m_lang\"").unwrap();
    let z_pos = json.find("\"z_lang\"").unwrap();
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Serde round-trips for enums
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn all_enum_serde_roundtrips() {
    // FileKind
    for v in [FileKind::Parent, FileKind::Child] {
        let j = serde_json::to_string(&v).unwrap();
        let b: FileKind = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
    // ChildrenMode
    for v in [ChildrenMode::Collapse, ChildrenMode::Separate] {
        let j = serde_json::to_string(&v).unwrap();
        let b: ChildrenMode = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
    // ConfigMode
    for v in [ConfigMode::Auto, ConfigMode::None] {
        let j = serde_json::to_string(&v).unwrap();
        let b: ConfigMode = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
}

#[test]
fn kebab_case_serialization() {
    assert_eq!(serde_json::to_string(&TableFormat::Md).unwrap(), "\"md\"");
    assert_eq!(
        serde_json::to_string(&ExportFormat::Cyclonedx).unwrap(),
        "\"cyclonedx\""
    );
    assert_eq!(
        serde_json::to_string(&ChildIncludeMode::ParentsOnly).unwrap(),
        "\"parents-only\""
    );
    assert_eq!(
        serde_json::to_string(&RedactMode::None).unwrap(),
        "\"none\""
    );
}

#[test]
fn snake_case_serialization() {
    assert_eq!(
        serde_json::to_string(&FileKind::Parent).unwrap(),
        "\"parent\""
    );
    assert_eq!(
        serde_json::to_string(&FileKind::Child).unwrap(),
        "\"child\""
    );
    assert_eq!(
        serde_json::to_string(&ScanStatus::Complete).unwrap(),
        "\"complete\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::DataBlob).unwrap(),
        "\"data_blob\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::HeadTail).unwrap(),
        "\"head_tail\""
    );
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Feat).unwrap(),
        "\"feat\""
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. TokenEstimationMeta
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn token_estimation_invariant() {
    let est = TokenEstimationMeta::from_bytes(10_000, 4.0);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

#[test]
fn token_estimation_zero_bytes() {
    let est = TokenEstimationMeta::from_bytes(0, 4.0);
    assert_eq!(est.tokens_min, 0);
    assert_eq!(est.tokens_est, 0);
    assert_eq!(est.tokens_max, 0);
    assert_eq!(est.source_bytes, 0);
}

#[test]
fn token_estimation_custom_bounds() {
    let est = TokenEstimationMeta::from_bytes_with_bounds(2000, 4.0, 2.0, 10.0);
    assert_eq!(est.tokens_est, 500); // 2000/4.0
    assert_eq!(est.tokens_min, 200); // 2000/10.0
    assert_eq!(est.tokens_max, 1000); // 2000/2.0
}

#[test]
fn token_estimation_serde_roundtrip() {
    let est = TokenEstimationMeta::from_bytes(8000, 4.0);
    let json = serde_json::to_string(&est).unwrap();
    let back: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.source_bytes, 8000);
    assert_eq!(back.tokens_est, est.tokens_est);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. TokenAudit
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn token_audit_basic() {
    let audit = TokenAudit::from_output(1000, 800);
    assert_eq!(audit.output_bytes, 1000);
    assert_eq!(audit.overhead_bytes, 200);
    assert!((audit.overhead_pct - 0.2).abs() < f64::EPSILON);
}

#[test]
fn token_audit_zero_output() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.overhead_pct, 0.0);
    assert_eq!(audit.tokens_est, 0);
}

#[test]
fn token_audit_content_exceeds_output_saturates() {
    let audit = TokenAudit::from_output(100, 500);
    assert_eq!(audit.overhead_bytes, 0);
    assert_eq!(audit.overhead_pct, 0.0);
}

#[test]
fn token_audit_serde_roundtrip() {
    let audit = TokenAudit::from_output(5000, 4000);
    let json = serde_json::to_string(&audit).unwrap();
    let back: TokenAudit = serde_json::from_str(&json).unwrap();
    assert_eq!(back.output_bytes, audit.output_bytes);
    assert_eq!(back.overhead_bytes, audit.overhead_bytes);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. Receipt envelope metadata
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn lang_receipt_envelope_fields() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1234567890,
        tool: ToolInfo::current(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: ScanArgs {
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
        },
        args: LangArgsMeta {
            format: "md".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.tool.name, "tokmd");
}

#[test]
fn run_receipt_construction() {
    let receipt = RunReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 999,
        lang_file: "lang.json".to_string(),
        module_file: "module.json".to_string(),
        export_file: "export.jsonl".to_string(),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let back: RunReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, SCHEMA_VERSION);
    assert_eq!(back.lang_file, "lang.json");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. Cockpit types
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn gate_status_all_variants_serde() {
    for v in [
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ] {
        let j = serde_json::to_string(&v).unwrap();
        let b: GateStatus = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
}

#[test]
fn risk_level_display() {
    assert_eq!(RiskLevel::Low.to_string(), "low");
    assert_eq!(RiskLevel::Medium.to_string(), "medium");
    assert_eq!(RiskLevel::High.to_string(), "high");
    assert_eq!(RiskLevel::Critical.to_string(), "critical");
}

#[test]
fn complexity_indicator_serde() {
    for v in [
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ] {
        let j = serde_json::to_string(&v).unwrap();
        let b: ComplexityIndicator = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
}

#[test]
fn trend_direction_serde() {
    for v in [
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Degrading,
    ] {
        let j = serde_json::to_string(&v).unwrap();
        let b: TrendDirection = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
}

#[test]
fn evidence_source_serde() {
    for v in [
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ] {
        let j = serde_json::to_string(&v).unwrap();
        let b: EvidenceSource = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
}

#[test]
fn commit_match_serde() {
    for v in [
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ] {
        let j = serde_json::to_string(&v).unwrap();
        let b: CommitMatch = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
}

#[test]
fn warning_type_serde() {
    for v in [
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ] {
        let j = serde_json::to_string(&v).unwrap();
        let b: WarningType = serde_json::from_str(&j).unwrap();
        assert_eq!(b, v);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. Property: Totals arithmetic (sum of parts = total)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn property_totals_sum_of_lang_rows() {
    let rows = [
        LangRow {
            lang: "Rust".to_string(),
            code: 1000,
            lines: 1500,
            files: 10,
            bytes: 40000,
            tokens: 10000,
            avg_lines: 150,
        },
        LangRow {
            lang: "Python".to_string(),
            code: 500,
            lines: 700,
            files: 5,
            bytes: 15000,
            tokens: 5000,
            avg_lines: 140,
        },
        LangRow {
            lang: "Go".to_string(),
            code: 300,
            lines: 400,
            files: 3,
            bytes: 10000,
            tokens: 3000,
            avg_lines: 133,
        },
    ];

    let total_code: usize = rows.iter().map(|r| r.code).sum();
    let total_lines: usize = rows.iter().map(|r| r.lines).sum();
    let total_files: usize = rows.iter().map(|r| r.files).sum();
    let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
    let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();
    let avg = total_lines.checked_div(total_files).unwrap_or(0);

    let totals = Totals {
        code: total_code,
        lines: total_lines,
        files: total_files,
        bytes: total_bytes,
        tokens: total_tokens,
        avg_lines: avg,
    };

    assert_eq!(totals.code, 1800);
    assert_eq!(totals.lines, 2600);
    assert_eq!(totals.files, 18);
    assert_eq!(totals.bytes, 65000);
    assert_eq!(totals.tokens, 18000);
    assert_eq!(totals.avg_lines, 144); // 2600 / 18
}

#[test]
fn property_diff_row_delta_consistency() {
    let row = DiffRow {
        lang: "Rust".to_string(),
        old_code: 1000,
        new_code: 1200,
        delta_code: 200,
        old_lines: 1500,
        new_lines: 1800,
        delta_lines: 300,
        old_files: 10,
        new_files: 12,
        delta_files: 2,
        old_bytes: 40000,
        new_bytes: 48000,
        delta_bytes: 8000,
        old_tokens: 10000,
        new_tokens: 12000,
        delta_tokens: 2000,
    };
    assert_eq!(
        row.delta_code,
        (row.new_code as i64) - (row.old_code as i64)
    );
    assert_eq!(
        row.delta_lines,
        (row.new_lines as i64) - (row.old_lines as i64)
    );
    assert_eq!(
        row.delta_files,
        (row.new_files as i64) - (row.old_files as i64)
    );
    assert_eq!(
        row.delta_bytes,
        (row.new_bytes as i64) - (row.old_bytes as i64)
    );
    assert_eq!(
        row.delta_tokens,
        (row.new_tokens as i64) - (row.old_tokens as i64)
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 12. BDD: Given rows / When computing totals / Then sums match
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bdd_given_file_rows_when_summing_then_totals_match() {
    // Given: a set of file rows for a module
    let rows = [
        FileRow {
            path: "src/a.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 5000,
            tokens: 1000,
        },
        FileRow {
            path: "src/b.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 200,
            comments: 40,
            blanks: 20,
            lines: 260,
            bytes: 8000,
            tokens: 2000,
        },
    ];

    // When: we compute aggregates
    let total_code: usize = rows.iter().map(|r| r.code).sum();
    let total_lines: usize = rows.iter().map(|r| r.lines).sum();
    let total_files = rows.len();

    // Then: sums are correct
    assert_eq!(total_code, 300);
    assert_eq!(total_lines, 390);
    assert_eq!(total_files, 2);
}

#[test]
fn bdd_given_empty_rows_when_summing_then_zero() {
    let rows: Vec<LangRow> = vec![];
    let total_code: usize = rows.iter().map(|r| r.code).sum();
    let total_files: usize = rows.iter().map(|r| r.files).sum();
    assert_eq!(total_code, 0);
    assert_eq!(total_files, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 13. Edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn edge_zero_valued_totals() {
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

#[test]
fn edge_max_usize_values() {
    let t = Totals {
        code: usize::MAX,
        lines: usize::MAX,
        files: usize::MAX,
        bytes: usize::MAX,
        tokens: usize::MAX,
        avg_lines: usize::MAX,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Totals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

#[test]
fn edge_empty_string_lang() {
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
    assert_eq!(back.lang, "");
}

#[test]
fn edge_unicode_in_path() {
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
        tokens: 100,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/日本語/ファイル.rs");
}

#[test]
fn edge_negative_delta_in_diff_row() {
    let row = DiffRow {
        lang: "Rust".to_string(),
        old_code: 500,
        new_code: 300,
        delta_code: -200,
        old_lines: 800,
        new_lines: 500,
        delta_lines: -300,
        old_files: 10,
        new_files: 8,
        delta_files: -2,
        old_bytes: 20000,
        new_bytes: 12000,
        delta_bytes: -8000,
        old_tokens: 5000,
        new_tokens: 3000,
        delta_tokens: -2000,
    };
    assert!(row.delta_code < 0);
    assert!(row.delta_lines < 0);
    let json = serde_json::to_string(&row).unwrap();
    let back: DiffRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, row);
}

#[test]
fn edge_empty_warnings_vec() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo::default(),
        mode: "lang".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: ScanArgs {
            paths: vec![],
            excluded: vec![],
            excluded_redacted: false,
            config: ConfigMode::Auto,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        },
        args: LangArgsMeta {
            format: "json".to_string(),
            top: 0,
            with_files: false,
            children: ChildrenMode::Collapse,
        },
        report: LangReport {
            rows: vec![],
            total: Totals {
                code: 0,
                lines: 0,
                files: 0,
                bytes: 0,
                tokens: 0,
                avg_lines: 0,
            },
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        },
    };
    assert!(receipt.warnings.is_empty());
    let json = serde_json::to_string(&receipt).unwrap();
    assert!(json.contains("\"warnings\":[]"));
}

#[test]
fn edge_large_token_estimation() {
    let est = TokenEstimationMeta::from_bytes(1_000_000_000, 4.0);
    assert_eq!(est.tokens_est, 250_000_000);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 14. Handoff and context types
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn context_excluded_path_serde() {
    let excluded = ContextExcludedPath {
        path: "vendor/lib.js".to_string(),
        reason: "vendored".to_string(),
    };
    let json = serde_json::to_string(&excluded).unwrap();
    let back: ContextExcludedPath = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "vendor/lib.js");
    assert_eq!(back.reason, "vendored");
}

#[test]
fn smart_excluded_file_serde() {
    let f = SmartExcludedFile {
        path: "package-lock.json".to_string(),
        reason: "lockfile".to_string(),
        tokens: 50000,
    };
    let json = serde_json::to_string(&f).unwrap();
    let back: SmartExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tokens, 50000);
}

#[test]
fn artifact_entry_with_hash() {
    let entry = ArtifactEntry {
        name: "bundle.txt".to_string(),
        path: "output/bundle.txt".to_string(),
        description: "Code bundle".to_string(),
        bytes: 12345,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: "abc123".to_string(),
        }),
    };
    let json = serde_json::to_string(&entry).unwrap();
    let back: ArtifactEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.hash.as_ref().unwrap().algo, "blake3");
}

#[test]
fn artifact_entry_without_hash() {
    let entry = ArtifactEntry {
        name: "receipt.json".to_string(),
        path: "output/receipt.json".to_string(),
        description: "Receipt".to_string(),
        bytes: 500,
        hash: None,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains("\"hash\""));
}

#[test]
fn policy_excluded_file_serde() {
    let f = PolicyExcludedFile {
        path: "generated/parser.rs".to_string(),
        original_tokens: 10000,
        policy: InclusionPolicy::Skip,
        reason: "generated code".to_string(),
        classifications: vec![FileClassification::Generated],
    };
    let json = serde_json::to_string(&f).unwrap();
    let back: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(back.policy, InclusionPolicy::Skip);
    assert_eq!(back.classifications.len(), 1);
}

#[test]
fn capability_status_serde() {
    let cs = CapabilityStatus {
        name: "git".to_string(),
        status: CapabilityState::Available,
        reason: None,
    };
    let json = serde_json::to_string(&cs).unwrap();
    let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "git");
    assert!(!json.contains("\"reason\""));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 15. Cockpit receipt round-trip
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_receipt_minimal_roundtrip() {
    let receipt = CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms: 1000,
        base_ref: "main".to_string(),
        head_ref: "HEAD".to_string(),
        change_surface: ChangeSurface {
            commits: 1,
            files_changed: 2,
            insertions: 10,
            deletions: 5,
            net_lines: 5,
            churn_velocity: 15.0,
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
            large_files_touched: 0,
            avg_file_size: 100,
            complexity_indicator: ComplexityIndicator::Low,
            warnings: vec![],
        },
        risk: Risk {
            hotspots_touched: vec![],
            bus_factor_warnings: vec![],
            level: RiskLevel::Low,
            score: 10,
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
    assert_eq!(back.mode, "cockpit");
}

#[test]
fn review_item_with_optional_fields() {
    let item = ReviewItem {
        path: "src/main.rs".to_string(),
        reason: "high churn".to_string(),
        priority: 1,
        complexity: Some(4),
        lines_changed: Some(200),
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: ReviewItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.complexity, Some(4));
    assert_eq!(back.lines_changed, Some(200));
}

#[test]
fn review_item_without_optional_fields() {
    let item = ReviewItem {
        path: "lib.rs".to_string(),
        reason: "new file".to_string(),
        priority: 3,
        complexity: None,
        lines_changed: None,
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(!json.contains("\"complexity\""));
    assert!(!json.contains("\"lines_changed\""));
}

#[test]
fn trend_comparison_default() {
    let trend = TrendComparison::default();
    assert!(!trend.baseline_available);
    assert!(trend.baseline_path.is_none());
    assert!(trend.health.is_none());
    assert!(trend.risk.is_none());
    assert!(trend.complexity.is_none());
}
