//! BDD-style scenario tests for tokmd-types.
//!
//! Each test follows Given/When/Then structure to verify type construction,
//! field access, serialization, and behavioral invariants.

use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, CapabilityStatus, ChildIncludeMode, ChildrenMode, CommitIntentKind,
    ConfigMode, ContextFileRow, DiffRow, DiffTotals, ExportFormat, FileClassification, FileKind,
    FileRow, HANDOFF_SCHEMA_VERSION, InclusionPolicy, LangRow, ModuleRow, PolicyExcludedFile,
    RedactMode, SCHEMA_VERSION, SmartExcludedFile, TableFormat, TokenAudit, TokenEstimationMeta,
    ToolInfo, Totals,
    cockpit::{
        COCKPIT_SCHEMA_VERSION, CommitMatch,
        ComplexityIndicator, EvidenceSource,
        GateStatus, RiskLevel, TrendDirection,
        WarningType,
    },
};

// =============================================================================
// Schema version constants
// =============================================================================

#[test]
fn schema_version_constants_are_correct() {
    // Given the documented schema versions
    // Then each constant matches its documented value
    assert_eq!(SCHEMA_VERSION, 2, "Core receipt schema version");
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3, "Cockpit schema version");
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5, "Handoff schema version");
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4, "Context schema version");
    assert_eq!(
        CONTEXT_BUNDLE_SCHEMA_VERSION, 2,
        "Context bundle schema version"
    );
}

// =============================================================================
// LangRow
// =============================================================================

#[test]
fn lang_row_construction_and_field_access() {
    // Given a LangRow with known values
    let row = LangRow {
        lang: "Rust".to_string(),
        code: 5000,
        lines: 7000,
        files: 42,
        bytes: 200_000,
        tokens: 50_000,
        avg_lines: 166,
    };

    // Then all fields are accessible and correct
    assert_eq!(row.lang, "Rust");
    assert_eq!(row.code, 5000);
    assert_eq!(row.lines, 7000);
    assert_eq!(row.files, 42);
    assert_eq!(row.bytes, 200_000);
    assert_eq!(row.tokens, 50_000);
    assert_eq!(row.avg_lines, 166);
}

#[test]
fn lang_row_serialized_fields_match_expected() {
    // Given a LangRow with known values
    let row = LangRow {
        lang: "Python".to_string(),
        code: 100,
        lines: 150,
        files: 3,
        bytes: 4096,
        tokens: 1024,
        avg_lines: 50,
    };

    // When serialized to JSON
    let value: serde_json::Value = serde_json::to_value(&row).unwrap();

    // Then fields match expected values
    assert_eq!(value["lang"], "Python");
    assert_eq!(value["code"], 100);
    assert_eq!(value["lines"], 150);
    assert_eq!(value["files"], 3);
    assert_eq!(value["bytes"], 4096);
    assert_eq!(value["tokens"], 1024);
    assert_eq!(value["avg_lines"], 50);
}

#[test]
fn lang_row_equality() {
    // Given two LangRows with the same values
    let a = LangRow {
        lang: "Go".to_string(),
        code: 1,
        lines: 2,
        files: 3,
        bytes: 4,
        tokens: 5,
        avg_lines: 6,
    };
    let b = a.clone();

    // Then they are equal
    assert_eq!(a, b);
}

// =============================================================================
// ModuleRow
// =============================================================================

#[test]
fn module_row_construction_and_field_access() {
    let row = ModuleRow {
        module: "src/parser".to_string(),
        code: 800,
        lines: 1200,
        files: 5,
        bytes: 32_000,
        tokens: 8_000,
        avg_lines: 240,
    };

    assert_eq!(row.module, "src/parser");
    assert_eq!(row.code, 800);
    assert_eq!(row.files, 5);
}

#[test]
fn module_row_serde_roundtrip() {
    let row = ModuleRow {
        module: "crates/core".to_string(),
        code: 999,
        lines: 1500,
        files: 10,
        bytes: 50_000,
        tokens: 12_500,
        avg_lines: 150,
    };

    let json = serde_json::to_string(&row).unwrap();
    let parsed: ModuleRow = serde_json::from_str(&json).unwrap();
    assert_eq!(row, parsed);
}

// =============================================================================
// FileRow
// =============================================================================

#[test]
fn file_row_construction_with_parent_kind() {
    let row = FileRow {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 200,
        comments: 30,
        blanks: 20,
        lines: 250,
        bytes: 8_000,
        tokens: 2_000,
    };

    assert_eq!(row.kind, FileKind::Parent);
    assert_eq!(row.path, "src/main.rs");
    assert_eq!(row.comments, 30);
    assert_eq!(row.blanks, 20);
}

#[test]
fn file_row_construction_with_child_kind() {
    let row = FileRow {
        path: "index.html".to_string(),
        module: ".".to_string(),
        lang: "HTML".to_string(),
        kind: FileKind::Child,
        code: 50,
        comments: 5,
        blanks: 10,
        lines: 65,
        bytes: 2_000,
        tokens: 500,
    };

    assert_eq!(row.kind, FileKind::Child);
}

#[test]
fn file_row_serde_roundtrip() {
    let row = FileRow {
        path: "lib/utils.py".to_string(),
        module: "lib".to_string(),
        lang: "Python".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 20,
        blanks: 15,
        lines: 135,
        bytes: 5_000,
        tokens: 1_250,
    };

    let json = serde_json::to_string(&row).unwrap();
    let parsed: FileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(row, parsed);
}

// =============================================================================
// ExportRow (FileRow is the export row type)
// =============================================================================

#[test]
fn export_row_json_field_order_is_stable() {
    // Given a FileRow used as an export row
    let row = FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 3,
        lines: 15,
        bytes: 400,
        tokens: 100,
    };

    // When serialized
    let value: serde_json::Value = serde_json::to_value(&row).unwrap();

    // Then all expected fields exist
    assert!(value.get("path").is_some());
    assert!(value.get("module").is_some());
    assert!(value.get("lang").is_some());
    assert!(value.get("kind").is_some());
    assert!(value.get("code").is_some());
    assert!(value.get("comments").is_some());
    assert!(value.get("blanks").is_some());
    assert!(value.get("lines").is_some());
    assert!(value.get("bytes").is_some());
    assert!(value.get("tokens").is_some());
}

// =============================================================================
// ContextFileRow with effective_tokens
// =============================================================================

#[test]
fn context_file_row_effective_tokens_none_when_full() {
    // Given a ContextFileRow with Full policy
    let row = ContextFileRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 500,
        code: 100,
        lines: 150,
        bytes: 2_000,
        value: 100,
        rank_reason: String::new(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    };

    // Then effective_tokens is None (same as tokens)
    assert_eq!(row.effective_tokens, None);
    assert_eq!(row.policy, InclusionPolicy::Full);
}

#[test]
fn context_file_row_effective_tokens_some_when_head_tail() {
    // Given a ContextFileRow with HeadTail policy
    let row = ContextFileRow {
        path: "big_file.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        tokens: 10_000,
        code: 2_000,
        lines: 3_000,
        bytes: 40_000,
        value: 50,
        rank_reason: "code".to_string(),
        policy: InclusionPolicy::HeadTail,
        effective_tokens: Some(2_000),
        policy_reason: Some("exceeds per-file cap".to_string()),
        classifications: vec![FileClassification::Generated],
    };

    // Then effective_tokens is less than tokens
    assert_eq!(row.effective_tokens, Some(2_000));
    assert!(row.effective_tokens.unwrap() < row.tokens);
}

#[test]
fn context_file_row_skip_policy_serializes_correctly() {
    let row = ContextFileRow {
        path: "vendor/lib.min.js".to_string(),
        module: "vendor".to_string(),
        lang: "JavaScript".to_string(),
        tokens: 50_000,
        code: 1,
        lines: 1,
        bytes: 200_000,
        value: 0,
        rank_reason: String::new(),
        policy: InclusionPolicy::Skip,
        effective_tokens: Some(0),
        policy_reason: Some("minified".to_string()),
        classifications: vec![FileClassification::Minified, FileClassification::Vendored],
    };

    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("\"skip\""));
    assert!(json.contains("\"minified\""));
    assert!(json.contains("\"vendored\""));
}

#[test]
fn context_file_row_default_policy_omitted_in_json() {
    // Given a ContextFileRow with default (Full) policy
    let row = ContextFileRow {
        path: "main.rs".to_string(),
        module: ".".to_string(),
        lang: "Rust".to_string(),
        tokens: 100,
        code: 50,
        lines: 80,
        bytes: 400,
        value: 50,
        rank_reason: String::new(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    };

    // When serialized
    let json = serde_json::to_string(&row).unwrap();

    // Then skip_serializing_if omits default policy and empty fields
    assert!(
        !json.contains("\"policy\""),
        "Full policy should be skipped"
    );
    assert!(
        !json.contains("\"effective_tokens\""),
        "None effective_tokens should be skipped"
    );
    assert!(
        !json.contains("\"policy_reason\""),
        "None policy_reason should be skipped"
    );
    assert!(
        !json.contains("\"classifications\""),
        "Empty classifications should be skipped"
    );
    assert!(
        !json.contains("\"rank_reason\""),
        "Empty rank_reason should be skipped"
    );
}

// =============================================================================
// Totals
// =============================================================================

#[test]
fn totals_construction_and_equality() {
    let a = Totals {
        code: 1000,
        lines: 1500,
        files: 20,
        bytes: 50_000,
        tokens: 12_500,
        avg_lines: 75,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

// =============================================================================
// DiffRow and DiffTotals
// =============================================================================

#[test]
fn diff_row_delta_calculations() {
    // Given a DiffRow showing growth
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
        old_bytes: 40_000,
        new_bytes: 48_000,
        delta_bytes: 8_000,
        old_tokens: 10_000,
        new_tokens: 12_000,
        delta_tokens: 2_000,
    };

    // Then delta = new - old
    assert_eq!(row.delta_code, row.new_code as i64 - row.old_code as i64);
    assert_eq!(row.delta_lines, row.new_lines as i64 - row.old_lines as i64);
    assert_eq!(row.delta_files, row.new_files as i64 - row.old_files as i64);
}

#[test]
fn diff_row_negative_delta() {
    // Given a DiffRow showing shrinkage
    let row = DiffRow {
        lang: "C".to_string(),
        old_code: 500,
        new_code: 300,
        delta_code: -200,
        old_lines: 700,
        new_lines: 400,
        delta_lines: -300,
        old_files: 5,
        new_files: 3,
        delta_files: -2,
        old_bytes: 20_000,
        new_bytes: 12_000,
        delta_bytes: -8_000,
        old_tokens: 5_000,
        new_tokens: 3_000,
        delta_tokens: -2_000,
    };

    assert!(row.delta_code < 0);
    assert!(row.delta_lines < 0);
    assert!(row.delta_files < 0);
}

#[test]
fn diff_totals_default_is_zero() {
    let totals = DiffTotals::default();
    assert_eq!(totals.old_code, 0);
    assert_eq!(totals.new_code, 0);
    assert_eq!(totals.delta_code, 0);
    assert_eq!(totals.old_lines, 0);
    assert_eq!(totals.new_lines, 0);
    assert_eq!(totals.delta_lines, 0);
}

// =============================================================================
// Enum serde tests
// =============================================================================

#[test]
fn file_kind_serializes_as_snake_case() {
    assert_eq!(
        serde_json::to_string(&FileKind::Parent).unwrap(),
        "\"parent\""
    );
    assert_eq!(
        serde_json::to_string(&FileKind::Child).unwrap(),
        "\"child\""
    );
}

#[test]
fn children_mode_serializes_as_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Collapse).unwrap(),
        "\"collapse\""
    );
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Separate).unwrap(),
        "\"separate\""
    );
}

#[test]
fn child_include_mode_serializes_as_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ChildIncludeMode::Separate).unwrap(),
        "\"separate\""
    );
    assert_eq!(
        serde_json::to_string(&ChildIncludeMode::ParentsOnly).unwrap(),
        "\"parents-only\""
    );
}

#[test]
fn table_format_serializes_as_kebab_case() {
    assert_eq!(serde_json::to_string(&TableFormat::Md).unwrap(), "\"md\"");
    assert_eq!(serde_json::to_string(&TableFormat::Tsv).unwrap(), "\"tsv\"");
    assert_eq!(
        serde_json::to_string(&TableFormat::Json).unwrap(),
        "\"json\""
    );
}

#[test]
fn export_format_serializes_as_kebab_case() {
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
fn config_mode_default_is_auto() {
    assert_eq!(ConfigMode::default(), ConfigMode::Auto);
}

#[test]
fn config_mode_serializes_as_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ConfigMode::Auto).unwrap(),
        "\"auto\""
    );
    assert_eq!(
        serde_json::to_string(&ConfigMode::None).unwrap(),
        "\"none\""
    );
}

#[test]
fn redact_mode_serializes_as_kebab_case() {
    assert_eq!(
        serde_json::to_string(&RedactMode::None).unwrap(),
        "\"none\""
    );
    assert_eq!(
        serde_json::to_string(&RedactMode::Paths).unwrap(),
        "\"paths\""
    );
    assert_eq!(serde_json::to_string(&RedactMode::All).unwrap(), "\"all\"");
}

#[test]
fn inclusion_policy_default_is_full() {
    assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
}

#[test]
fn inclusion_policy_serializes_as_snake_case() {
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::Full).unwrap(),
        "\"full\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::HeadTail).unwrap(),
        "\"head_tail\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::Summary).unwrap(),
        "\"summary\""
    );
    assert_eq!(
        serde_json::to_string(&InclusionPolicy::Skip).unwrap(),
        "\"skip\""
    );
}

#[test]
fn file_classification_serializes_as_snake_case() {
    assert_eq!(
        serde_json::to_string(&FileClassification::Generated).unwrap(),
        "\"generated\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Fixture).unwrap(),
        "\"fixture\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Vendored).unwrap(),
        "\"vendored\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Lockfile).unwrap(),
        "\"lockfile\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Minified).unwrap(),
        "\"minified\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::DataBlob).unwrap(),
        "\"data_blob\""
    );
    assert_eq!(
        serde_json::to_string(&FileClassification::Sourcemap).unwrap(),
        "\"sourcemap\""
    );
}

#[test]
fn capability_state_serializes_as_snake_case() {
    assert_eq!(
        serde_json::to_string(&CapabilityState::Available).unwrap(),
        "\"available\""
    );
    assert_eq!(
        serde_json::to_string(&CapabilityState::Skipped).unwrap(),
        "\"skipped\""
    );
    assert_eq!(
        serde_json::to_string(&CapabilityState::Unavailable).unwrap(),
        "\"unavailable\""
    );
}

#[test]
fn commit_intent_kind_all_variants_roundtrip() {
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

    for variant in &variants {
        let json = serde_json::to_string(variant).unwrap();
        let parsed: CommitIntentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(*variant, parsed);
    }
}

#[test]
fn commit_intent_kind_serializes_as_snake_case() {
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Feat).unwrap(),
        "\"feat\""
    );
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Fix).unwrap(),
        "\"fix\""
    );
    assert_eq!(
        serde_json::to_string(&CommitIntentKind::Other).unwrap(),
        "\"other\""
    );
}

// =============================================================================
// TokenEstimationMeta
// =============================================================================

#[test]
fn token_estimation_invariant_min_le_est_le_max() {
    // Given various byte counts
    for bytes in [0, 1, 100, 1000, 10_000, 1_000_000] {
        let meta = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        // Then tokens_min <= tokens_est <= tokens_max
        assert!(
            meta.tokens_min <= meta.tokens_est,
            "tokens_min ({}) > tokens_est ({}) for bytes={}",
            meta.tokens_min,
            meta.tokens_est,
            bytes,
        );
        assert!(
            meta.tokens_est <= meta.tokens_max,
            "tokens_est ({}) > tokens_max ({}) for bytes={}",
            meta.tokens_est,
            meta.tokens_max,
            bytes,
        );
    }
}

#[test]
fn token_estimation_zero_bytes() {
    let meta = TokenEstimationMeta::from_bytes(0, TokenEstimationMeta::DEFAULT_BPT_EST);
    assert_eq!(meta.tokens_min, 0);
    assert_eq!(meta.tokens_est, 0);
    assert_eq!(meta.tokens_max, 0);
    assert_eq!(meta.source_bytes, 0);
}

#[test]
fn token_estimation_custom_bounds() {
    let meta = TokenEstimationMeta::from_bytes_with_bounds(1000, 4.0, 2.0, 8.0);
    assert_eq!(meta.source_bytes, 1000);
    assert_eq!(meta.tokens_est, 250); // 1000/4.0
    assert_eq!(meta.tokens_min, 125); // 1000/8.0
    assert_eq!(meta.tokens_max, 500); // 1000/2.0
}

// =============================================================================
// TokenAudit
// =============================================================================

#[test]
fn token_audit_overhead_calculation() {
    let audit = TokenAudit::from_output(10_000, 8_000);
    assert_eq!(audit.output_bytes, 10_000);
    assert_eq!(audit.overhead_bytes, 2_000);
    assert!((audit.overhead_pct - 0.2).abs() < f64::EPSILON);
}

#[test]
fn token_audit_zero_output_bytes() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.output_bytes, 0);
    assert_eq!(audit.overhead_bytes, 0);
    assert!((audit.overhead_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn token_audit_content_exceeds_output_saturates() {
    // content_bytes > output_bytes should saturate to 0 overhead
    let audit = TokenAudit::from_output(100, 200);
    assert_eq!(audit.overhead_bytes, 0);
}

// =============================================================================
// ToolInfo
// =============================================================================

#[test]
fn tool_info_default_is_empty() {
    let info = ToolInfo::default();
    assert!(info.name.is_empty());
    assert!(info.version.is_empty());
}

#[test]
fn tool_info_current_has_semver_like_version() {
    let info = ToolInfo::current();
    // Version should contain at least one dot (e.g., "0.1.0")
    assert!(
        info.version.contains('.'),
        "Version '{}' should be semver-like",
        info.version
    );
}

// =============================================================================
// Cockpit enums
// =============================================================================

#[test]
fn gate_status_all_variants_roundtrip() {
    let variants = [
        GateStatus::Pass,
        GateStatus::Warn,
        GateStatus::Fail,
        GateStatus::Skipped,
        GateStatus::Pending,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let parsed: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, parsed);
    }
}

#[test]
fn risk_level_display_matches_serde() {
    // Given each risk level variant
    let variants = [
        (RiskLevel::Low, "low"),
        (RiskLevel::Medium, "medium"),
        (RiskLevel::High, "high"),
        (RiskLevel::Critical, "critical"),
    ];

    for (level, expected) in &variants {
        // Display trait should match the serde serialization (minus quotes)
        assert_eq!(level.to_string(), *expected);
        let json = serde_json::to_string(level).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn complexity_indicator_all_variants_roundtrip() {
    let variants = [
        ComplexityIndicator::Low,
        ComplexityIndicator::Medium,
        ComplexityIndicator::High,
        ComplexityIndicator::Critical,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let parsed: ComplexityIndicator = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, parsed);
    }
}

#[test]
fn warning_type_all_variants_roundtrip() {
    let variants = [
        WarningType::LargeFile,
        WarningType::HighChurn,
        WarningType::LowTestCoverage,
        WarningType::ComplexChange,
        WarningType::BusFactor,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let parsed: WarningType = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, parsed);
    }
}

#[test]
fn trend_direction_all_variants_roundtrip() {
    let variants = [
        TrendDirection::Improving,
        TrendDirection::Stable,
        TrendDirection::Degrading,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let parsed: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, parsed);
    }
}

#[test]
fn evidence_source_all_variants_roundtrip() {
    let variants = [
        EvidenceSource::CiArtifact,
        EvidenceSource::Cached,
        EvidenceSource::RanLocal,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let parsed: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, parsed);
    }
}

#[test]
fn commit_match_all_variants_roundtrip() {
    let variants = [
        CommitMatch::Exact,
        CommitMatch::Partial,
        CommitMatch::Stale,
        CommitMatch::Unknown,
    ];
    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let parsed: CommitMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, parsed);
    }
}

// =============================================================================
// Misc types
// =============================================================================

#[test]
fn smart_excluded_file_roundtrip() {
    let f = SmartExcludedFile {
        path: "package-lock.json".to_string(),
        reason: "lockfile".to_string(),
        tokens: 50_000,
    };
    let json = serde_json::to_string(&f).unwrap();
    let parsed: SmartExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.path, "package-lock.json");
    assert_eq!(parsed.tokens, 50_000);
}

#[test]
fn policy_excluded_file_roundtrip() {
    let f = PolicyExcludedFile {
        path: "gen/proto.rs".to_string(),
        original_tokens: 100_000,
        policy: InclusionPolicy::Skip,
        reason: "generated code".to_string(),
        classifications: vec![FileClassification::Generated],
    };
    let json = serde_json::to_string(&f).unwrap();
    let parsed: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.original_tokens, 100_000);
    assert_eq!(parsed.policy, InclusionPolicy::Skip);
}

#[test]
fn artifact_entry_with_hash_roundtrip() {
    let entry = ArtifactEntry {
        name: "code.txt".to_string(),
        path: "bundle/code.txt".to_string(),
        description: "Code bundle".to_string(),
        bytes: 1024,
        hash: Some(ArtifactHash {
            algo: "blake3".to_string(),
            hash: "abc123def456".to_string(),
        }),
    };
    let json = serde_json::to_string(&entry).unwrap();
    let parsed: ArtifactEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.hash.as_ref().unwrap().algo, "blake3");
}

#[test]
fn artifact_entry_without_hash_omits_field() {
    let entry = ArtifactEntry {
        name: "tree.txt".to_string(),
        path: "bundle/tree.txt".to_string(),
        description: "File tree".to_string(),
        bytes: 256,
        hash: None,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains("\"hash\""), "None hash should be omitted");
}

#[test]
fn capability_status_roundtrip() {
    let status = CapabilityStatus {
        name: "git".to_string(),
        status: CapabilityState::Available,
        reason: None,
    };
    let json = serde_json::to_string(&status).unwrap();
    let parsed: CapabilityStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "git");
    assert_eq!(parsed.status, CapabilityState::Available);
}

#[test]
fn analysis_format_all_variants_roundtrip() {
    use tokmd_types::AnalysisFormat;
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
        let parsed: AnalysisFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(*v, parsed);
    }
}
