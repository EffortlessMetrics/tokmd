//! Exhaustive serde and invariant tests for tokmd-types (w72).

use tokmd_types::cockpit::*;
use tokmd_types::*;

// =============================================================================
// Schema version constants
// =============================================================================

#[test]
fn schema_version_positive() {
    // Use runtime values to avoid clippy::assertions_on_constants
    let sv: u32 = SCHEMA_VERSION;
    let hv: u32 = HANDOFF_SCHEMA_VERSION;
    let csv: u32 = CONTEXT_SCHEMA_VERSION;
    let cbv: u32 = CONTEXT_BUNDLE_SCHEMA_VERSION;
    let ckv: u32 = COCKPIT_SCHEMA_VERSION;
    assert!(sv > 0);
    assert!(hv > 0);
    assert!(csv > 0);
    assert!(cbv > 0);
    assert!(ckv > 0);
}

// =============================================================================
// Struct construction + JSON roundtrip
// =============================================================================

#[test]
fn totals_construct_and_serialize() {
    let t = Totals {
        code: 10,
        lines: 20,
        files: 3,
        bytes: 500,
        tokens: 50,
        avg_lines: 6,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Totals = serde_json::from_str(&json).unwrap();
    assert_eq!(back, t);
}

#[test]
fn lang_row_construct_and_serialize() {
    let r = LangRow {
        lang: "Python".into(),
        code: 42,
        lines: 60,
        files: 2,
        bytes: 1200,
        tokens: 100,
        avg_lines: 30,
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lang, "Python");
}

#[test]
fn module_row_construct_and_serialize() {
    let r = ModuleRow {
        module: "crates/core".into(),
        code: 500,
        lines: 700,
        files: 5,
        bytes: 20000,
        tokens: 5000,
        avg_lines: 140,
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: ModuleRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.module, "crates/core");
}

#[test]
fn file_row_construct_and_serialize() {
    let r = FileRow {
        path: "src/main.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 50,
        comments: 10,
        blanks: 5,
        lines: 65,
        bytes: 2000,
        tokens: 100,
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, r);
}

#[test]
fn diff_row_construct_and_serialize() {
    let r = DiffRow {
        lang: "Go".into(),
        old_code: 100,
        new_code: 110,
        delta_code: 10,
        old_lines: 200,
        new_lines: 210,
        delta_lines: 10,
        old_files: 5,
        new_files: 6,
        delta_files: 1,
        old_bytes: 4000,
        new_bytes: 4400,
        delta_bytes: 400,
        old_tokens: 1000,
        new_tokens: 1100,
        delta_tokens: 100,
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: DiffRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back, r);
}

#[test]
fn diff_totals_default_all_zero() {
    let dt = DiffTotals::default();
    assert_eq!(dt.old_code, 0);
    assert_eq!(dt.delta_code, 0);
    assert_eq!(dt.delta_tokens, 0);
}

#[test]
fn tool_info_construct_and_serialize() {
    let ti = ToolInfo {
        name: "test".into(),
        version: "0.1.0".into(),
    };
    let json = serde_json::to_string(&ti).unwrap();
    let back: ToolInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "test");
}

#[test]
fn tool_info_current_has_name_and_version() {
    let ti = ToolInfo::current();
    assert_eq!(ti.name, "tokmd");
    assert!(!ti.version.is_empty());
}

#[test]
fn scan_args_construct_and_serialize() {
    let sa = ScanArgs {
        paths: vec![".".into()],
        excluded: vec![],
        excluded_redacted: false,
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    };
    let json = serde_json::to_string(&sa).unwrap();
    let back: ScanArgs = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, vec!["."]);
}

#[test]
fn context_receipt_construct_and_serialize() {
    let cr = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1000,
        tool: ToolInfo::current(),
        mode: "context".into(),
        budget_tokens: 8000,
        used_tokens: 5000,
        utilization_pct: 62.5,
        strategy: "greedy".into(),
        rank_by: "code".into(),
        file_count: 3,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: None,
        bundle_audit: None,
    };
    let json = serde_json::to_string(&cr).unwrap();
    let back: ContextReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, CONTEXT_SCHEMA_VERSION);
    assert_eq!(back.mode, "context");
}

// =============================================================================
// ContextFileRow: tokens vs effective_tokens
// =============================================================================

#[test]
fn context_file_row_effective_tokens_none_means_same_as_tokens() {
    let row = ContextFileRow {
        path: "src/lib.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        tokens: 500,
        code: 100,
        lines: 120,
        bytes: 2000,
        value: 500,
        rank_reason: String::new(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    };
    // When effective_tokens is None, the effective count equals tokens
    assert!(row.effective_tokens.is_none());
    let effective = row.effective_tokens.unwrap_or(row.tokens);
    assert_eq!(effective, 500);
}

#[test]
fn context_file_row_effective_tokens_reduced_by_policy() {
    let row = ContextFileRow {
        path: "vendor/lib.js".into(),
        module: "vendor".into(),
        lang: "JavaScript".into(),
        tokens: 10000,
        code: 3000,
        lines: 3500,
        bytes: 40000,
        value: 100,
        rank_reason: String::new(),
        policy: InclusionPolicy::HeadTail,
        effective_tokens: Some(200),
        policy_reason: Some("Per-file cap".into()),
        classifications: vec![FileClassification::Vendored],
    };
    let effective = row.effective_tokens.unwrap_or(row.tokens);
    assert!(effective < row.tokens);
    assert_eq!(effective, 200);
}

// =============================================================================
// Enum serde coverage — every variant
// =============================================================================

#[test]
fn file_kind_all_variants() {
    for variant in [FileKind::Parent, FileKind::Child] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: FileKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn scan_status_all_variants() {
    for json_str in [r#""complete""#, r#""partial""#] {
        let _v: ScanStatus = serde_json::from_str(json_str).unwrap();
    }
}

#[test]
fn commit_intent_kind_all_variants() {
    let all = [
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
    assert_eq!(all.len(), 12, "CommitIntentKind should have 12 variants");
    for v in all {
        let json = serde_json::to_string(&v).unwrap();
        let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn table_format_all_variants() {
    for v in [TableFormat::Md, TableFormat::Tsv, TableFormat::Json] {
        let json = serde_json::to_string(&v).unwrap();
        let back: TableFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn export_format_all_variants() {
    for v in [
        ExportFormat::Csv,
        ExportFormat::Jsonl,
        ExportFormat::Json,
        ExportFormat::Cyclonedx,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ExportFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn config_mode_all_variants() {
    for v in [ConfigMode::Auto, ConfigMode::None] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ConfigMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn children_mode_all_variants() {
    for v in [ChildrenMode::Collapse, ChildrenMode::Separate] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ChildrenMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn child_include_mode_all_variants() {
    for v in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn redact_mode_all_variants() {
    for v in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
        let json = serde_json::to_string(&v).unwrap();
        let back: RedactMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn analysis_format_all_variants() {
    let all = [
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
    assert_eq!(all.len(), 10, "AnalysisFormat should have 10 variants");
    for v in all {
        let json = serde_json::to_string(&v).unwrap();
        let back: AnalysisFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn file_classification_all_variants() {
    let all = [
        FileClassification::Generated,
        FileClassification::Fixture,
        FileClassification::Vendored,
        FileClassification::Lockfile,
        FileClassification::Minified,
        FileClassification::DataBlob,
        FileClassification::Sourcemap,
    ];
    assert_eq!(all.len(), 7, "FileClassification should have 7 variants");
    for v in all {
        let json = serde_json::to_string(&v).unwrap();
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn inclusion_policy_all_variants_and_default() {
    let all = [
        InclusionPolicy::Full,
        InclusionPolicy::HeadTail,
        InclusionPolicy::Summary,
        InclusionPolicy::Skip,
    ];
    assert_eq!(all.len(), 4, "InclusionPolicy should have 4 variants");
    for v in all {
        let json = serde_json::to_string(&v).unwrap();
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

#[test]
fn capability_state_all_variants() {
    for v in [
        CapabilityState::Available,
        CapabilityState::Skipped,
        CapabilityState::Unavailable,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

// =============================================================================
// Backward compatibility: unknown fields tolerated
// =============================================================================

#[test]
fn totals_ignores_unknown_fields() {
    let json =
        r#"{"code":1,"lines":2,"files":1,"bytes":10,"tokens":3,"avg_lines":2,"extra_field":42}"#;
    let t: Totals = serde_json::from_str(json).unwrap();
    assert_eq!(t.code, 1);
}

#[test]
fn lang_row_ignores_unknown_fields() {
    let json = r#"{"lang":"Rust","code":1,"lines":2,"files":1,"bytes":10,"tokens":3,"avg_lines":2,"future_field":true}"#;
    let r: LangRow = serde_json::from_str(json).unwrap();
    assert_eq!(r.lang, "Rust");
}

#[test]
fn tool_info_ignores_unknown_fields() {
    let json = r#"{"name":"x","version":"0.1","unknown":"ok"}"#;
    let ti: ToolInfo = serde_json::from_str(json).unwrap();
    assert_eq!(ti.name, "x");
}

// =============================================================================
// Receipt families: metadata / envelope / schema_version
// =============================================================================

fn make_scan_args() -> ScanArgs {
    ScanArgs {
        paths: vec![".".into()],
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

#[test]
fn lang_receipt_has_schema_and_tool() {
    let receipt = LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 123,
        tool: ToolInfo::current(),
        mode: "lang".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        scan: make_scan_args(),
        args: LangArgsMeta {
            format: "md".into(),
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
    let json = serde_json::to_string(&receipt).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], SCHEMA_VERSION);
    assert_eq!(v["mode"], "lang");
    assert!(v["tool"]["name"].is_string());
}

#[test]
fn diff_receipt_has_schema_and_tool() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 456,
        tool: ToolInfo::current(),
        mode: "diff".into(),
        from_source: "a.json".into(),
        to_source: "b.json".into(),
        diff_rows: vec![],
        totals: DiffTotals::default(),
    };
    let json = serde_json::to_string(&receipt).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], SCHEMA_VERSION);
    assert_eq!(v["mode"], "diff");
}

// =============================================================================
// Cockpit types
// =============================================================================

#[test]
fn cockpit_gate_status_all_variants() {
    let all = [
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ];
    assert_eq!(all.len(), 5);
    for v in all {
        let json = serde_json::to_string(&v).unwrap();
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn cockpit_evidence_source_all_variants() {
    for v in [
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn cockpit_commit_match_all_variants() {
    for v in [
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: CommitMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn cockpit_complexity_indicator_all_variants() {
    for v in [
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: ComplexityIndicator = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn cockpit_warning_type_all_variants() {
    for v in [
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: WarningType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn cockpit_risk_level_all_variants_and_display() {
    for (v, expected) in [
        (RiskLevel::Low, "low"),
        (RiskLevel::Medium, "medium"),
        (RiskLevel::High, "high"),
        (RiskLevel::Critical, "critical"),
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
        assert_eq!(v.to_string(), expected);
    }
}

#[test]
fn cockpit_trend_direction_all_variants() {
    for v in [
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Degrading,
    ] {
        let json = serde_json::to_string(&v).unwrap();
        let back: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

// =============================================================================
// TokenEstimationMeta invariants
// =============================================================================

#[test]
fn token_estimation_invariant_min_le_est_le_max() {
    for bytes in [0, 1, 100, 4000, 99999] {
        let est = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        assert!(
            est.tokens_min <= est.tokens_est,
            "min={} > est={} for bytes={}",
            est.tokens_min,
            est.tokens_est,
            bytes
        );
        assert!(
            est.tokens_est <= est.tokens_max,
            "est={} > max={} for bytes={}",
            est.tokens_est,
            est.tokens_max,
            bytes
        );
    }
}

#[test]
fn token_audit_overhead_invariant() {
    let audit = TokenAudit::from_output(1000, 800);
    assert_eq!(audit.overhead_bytes, 200);
    assert!(audit.overhead_pct >= 0.0 && audit.overhead_pct <= 1.0);
}
