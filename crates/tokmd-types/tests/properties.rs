//! Property-based tests for tokmd-types serialization.
//!
//! These tests verify that core data types round-trip correctly through JSON.

use proptest::prelude::*;
use tokmd_types::{
    AnalysisFormat, ArtifactEntry, ArtifactHash, CONTEXT_SCHEMA_VERSION, ChildIncludeMode,
    ChildrenMode, CommitIntentKind, ConfigMode, ContextExcludedPath, ContextReceipt, DiffRow,
    DiffTotals, ExportFormat, FileClassification, FileKind, FileRow, HandoffDerived,
    HandoffHotspot, InclusionPolicy, LangRow, ModuleRow, PolicyExcludedFile, RedactMode,
    ScanStatus, SmartExcludedFile, TableFormat, TokenAudit, TokenEstimationMeta, ToolInfo, Totals,
    cockpit::{
        CommitMatch, ComplexityIndicator, EvidenceSource, GateStatus, RiskLevel, TrendDirection,
        WarningType,
    },
};

// Arbitrary implementations for generating test data

fn arb_totals() -> impl Strategy<Value = Totals> {
    (
        0usize..100000,
        0usize..200000,
        0usize..10000,
        0usize..10000000,
        0usize..1000000,
        0usize..1000,
    )
        .prop_map(|(code, lines, files, bytes, tokens, avg_lines)| Totals {
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines,
        })
}

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[a-zA-Z][a-zA-Z0-9 ]*",
        0usize..100000,
        0usize..200000,
        0usize..10000,
        0usize..10000000,
        0usize..1000000,
        0usize..1000,
    )
        .prop_map(
            |(lang, code, lines, files, bytes, tokens, avg_lines)| LangRow {
                lang,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        "[a-zA-Z0-9_/]+",
        0usize..100000,
        0usize..200000,
        0usize..10000,
        0usize..10000000,
        0usize..1000000,
        0usize..1000,
    )
        .prop_map(
            |(module, code, lines, files, bytes, tokens, avg_lines)| ModuleRow {
                module,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            },
        )
}

fn arb_file_kind() -> impl Strategy<Value = FileKind> {
    prop_oneof![Just(FileKind::Parent), Just(FileKind::Child),]
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-zA-Z0-9_/]+\\.[a-z]+",
        "[a-zA-Z0-9_/]+",
        "[a-zA-Z]+",
        arb_file_kind(),
        0usize..100000,
        0usize..50000,
        0usize..50000,
        0usize..200000,
        0usize..10000000,
        0usize..1000000,
    )
        .prop_map(
            |(path, module, lang, kind, code, comments, blanks, lines, bytes, tokens)| FileRow {
                path,
                module,
                lang,
                kind,
                code,
                comments,
                blanks,
                lines,
                bytes,
                tokens,
            },
        )
}

proptest! {
    // ========================
    // FileKind Round-trip
    // ========================

    #[test]
    fn file_kind_roundtrip(kind in arb_file_kind()) {
        let json = serde_json::to_string(&kind).expect("serialize");
        let parsed: FileKind = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(kind, parsed);
    }

    #[test]
    fn file_kind_snake_case(_dummy in 0..1u8) {
        // FileKind uses snake_case serialization
        let parent_json = serde_json::to_string(&FileKind::Parent).expect("serialize");
        let child_json = serde_json::to_string(&FileKind::Child).expect("serialize");

        prop_assert_eq!(parent_json, "\"parent\"");
        prop_assert_eq!(child_json, "\"child\"");
    }

    // ========================
    // Totals Round-trip
    // ========================

    #[test]
    fn totals_roundtrip(totals in arb_totals()) {
        let json = serde_json::to_string(&totals).expect("serialize");
        let parsed: Totals = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(totals, parsed);
    }

    #[test]
    fn totals_json_has_all_fields(totals in arb_totals()) {
        let json = serde_json::to_string(&totals).expect("serialize");
        prop_assert!(json.contains("\"code\""));
        prop_assert!(json.contains("\"lines\""));
        prop_assert!(json.contains("\"files\""));
        prop_assert!(json.contains("\"bytes\""));
        prop_assert!(json.contains("\"tokens\""));
        prop_assert!(json.contains("\"avg_lines\""));
    }

    // ========================
    // LangRow Round-trip
    // ========================

    #[test]
    fn lang_row_roundtrip(row in arb_lang_row()) {
        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: LangRow = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(row, parsed);
    }

    #[test]
    fn lang_row_json_has_all_fields(row in arb_lang_row()) {
        let json = serde_json::to_string(&row).expect("serialize");
        prop_assert!(json.contains("\"lang\""), "Missing lang field");
        prop_assert!(json.contains("\"code\""), "Missing code field");
        prop_assert!(json.contains("\"lines\""), "Missing lines field");
        prop_assert!(json.contains("\"files\""), "Missing files field");
        prop_assert!(json.contains("\"bytes\""), "Missing bytes field");
        prop_assert!(json.contains("\"tokens\""), "Missing tokens field");
        prop_assert!(json.contains("\"avg_lines\""), "Missing avg_lines field");
    }

    // ========================
    // ModuleRow Round-trip
    // ========================

    #[test]
    fn module_row_roundtrip(row in arb_module_row()) {
        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: ModuleRow = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(row, parsed);
    }

    #[test]
    fn module_row_json_has_all_fields(row in arb_module_row()) {
        let json = serde_json::to_string(&row).expect("serialize");
        prop_assert!(json.contains("\"module\""), "Missing module field");
        prop_assert!(json.contains("\"code\""), "Missing code field");
        prop_assert!(json.contains("\"lines\""), "Missing lines field");
        prop_assert!(json.contains("\"files\""), "Missing files field");
        prop_assert!(json.contains("\"bytes\""), "Missing bytes field");
        prop_assert!(json.contains("\"tokens\""), "Missing tokens field");
        prop_assert!(json.contains("\"avg_lines\""), "Missing avg_lines field");
    }

    // ========================
    // FileRow Round-trip
    // ========================

    #[test]
    fn file_row_roundtrip(row in arb_file_row()) {
        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: FileRow = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(row, parsed);
    }

    #[test]
    fn file_row_json_has_all_fields(row in arb_file_row()) {
        let json = serde_json::to_string(&row).expect("serialize");
        prop_assert!(json.contains("\"path\""), "Missing path field");
        prop_assert!(json.contains("\"module\""), "Missing module field");
        prop_assert!(json.contains("\"lang\""), "Missing lang field");
        prop_assert!(json.contains("\"kind\""), "Missing kind field");
        prop_assert!(json.contains("\"code\""), "Missing code field");
        prop_assert!(json.contains("\"comments\""), "Missing comments field");
        prop_assert!(json.contains("\"blanks\""), "Missing blanks field");
        prop_assert!(json.contains("\"lines\""), "Missing lines field");
        prop_assert!(json.contains("\"bytes\""), "Missing bytes field");
        prop_assert!(json.contains("\"tokens\""), "Missing tokens field");
    }

    // ========================
    // Vector Round-trips
    // ========================

    #[test]
    fn lang_rows_vector_roundtrip(rows in prop::collection::vec(arb_lang_row(), 0..10)) {
        let json = serde_json::to_string(&rows).expect("serialize");
        let parsed: Vec<LangRow> = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(rows, parsed);
    }

    #[test]
    fn module_rows_vector_roundtrip(rows in prop::collection::vec(arb_module_row(), 0..10)) {
        let json = serde_json::to_string(&rows).expect("serialize");
        let parsed: Vec<ModuleRow> = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(rows, parsed);
    }

    #[test]
    fn file_rows_vector_roundtrip(rows in prop::collection::vec(arb_file_row(), 0..10)) {
        let json = serde_json::to_string(&rows).expect("serialize");
        let parsed: Vec<FileRow> = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(rows, parsed);
    }

    // ========================
    // Field Value Constraints
    // ========================

    #[test]
    fn totals_fields_are_usize_compatible(totals in arb_totals()) {
        // Verify serialization produces valid JSON numbers
        let json = serde_json::to_string(&totals).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse as value");

        prop_assert!(value["code"].is_u64());
        prop_assert!(value["lines"].is_u64());
        prop_assert!(value["files"].is_u64());
        prop_assert!(value["bytes"].is_u64());
        prop_assert!(value["tokens"].is_u64());
        prop_assert!(value["avg_lines"].is_u64());
    }

    // ========================
    // Edge Cases
    // ========================

    #[test]
    fn totals_zero_values_roundtrip(_dummy in 0..1u8) {
        let zero = Totals {
            code: 0,
            lines: 0,
            files: 0,
            bytes: 0,
            tokens: 0,
            avg_lines: 0,
        };
        let json = serde_json::to_string(&zero).expect("serialize");
        let parsed: Totals = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(zero, parsed);
    }

    #[test]
    fn totals_max_values_roundtrip(_dummy in 0..1u8) {
        // Test with large but realistic values
        let large = Totals {
            code: 10_000_000,
            lines: 20_000_000,
            files: 100_000,
            bytes: 1_000_000_000,
            tokens: 100_000_000,
            avg_lines: 200,
        };
        let json = serde_json::to_string(&large).expect("serialize");
        let parsed: Totals = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(large, parsed);
    }

    #[test]
    fn lang_row_with_special_chars(
        code in 0usize..1000,
        lines in 0usize..2000,
        files in 0usize..100,
        bytes in 0usize..100000,
        tokens in 0usize..10000,
        avg_lines in 0usize..100
    ) {
        // Test language names that might need escaping
        let row = LangRow {
            lang: "C++ (Modern)".to_string(),
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines,
        };
        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: LangRow = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(row, parsed);
    }

    // ========================
    // DiffRow Round-trip
    // ========================

    #[test]
    fn diff_row_roundtrip(
        lang in "[a-zA-Z]+",
        old_code in 0usize..100000,
        new_code in 0usize..100000,
        old_lines in 0usize..200000,
        new_lines in 0usize..200000,
        old_files in 0usize..10000,
        new_files in 0usize..10000,
        old_bytes in 0usize..10000000,
        new_bytes in 0usize..10000000,
        old_tokens in 0usize..1000000,
        new_tokens in 0usize..1000000,
    ) {
        let row = DiffRow {
            lang,
            old_code,
            new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines,
            new_lines,
            delta_lines: new_lines as i64 - old_lines as i64,
            old_files,
            new_files,
            delta_files: new_files as i64 - old_files as i64,
            old_bytes,
            new_bytes,
            delta_bytes: new_bytes as i64 - old_bytes as i64,
            old_tokens,
            new_tokens,
            delta_tokens: new_tokens as i64 - old_tokens as i64,
        };
        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: DiffRow = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(row, parsed);
    }

    // ========================
    // DiffTotals Round-trip
    // ========================

    #[test]
    fn diff_totals_roundtrip(
        old_code in 0usize..100000,
        new_code in 0usize..100000,
        old_lines in 0usize..200000,
        new_lines in 0usize..200000,
        old_files in 0usize..10000,
        new_files in 0usize..10000,
        old_bytes in 0usize..10000000,
        new_bytes in 0usize..10000000,
        old_tokens in 0usize..1000000,
        new_tokens in 0usize..1000000,
    ) {
        let totals = DiffTotals {
            old_code,
            new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines,
            new_lines,
            delta_lines: new_lines as i64 - old_lines as i64,
            old_files,
            new_files,
            delta_files: new_files as i64 - old_files as i64,
            old_bytes,
            new_bytes,
            delta_bytes: new_bytes as i64 - old_bytes as i64,
            old_tokens,
            new_tokens,
            delta_tokens: new_tokens as i64 - old_tokens as i64,
        };
        let json = serde_json::to_string(&totals).expect("serialize");
        let parsed: DiffTotals = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(totals, parsed);
    }

    // ========================
    // TokenEstimationMeta invariant: min <= est <= max
    // ========================

    #[test]
    fn token_estimation_invariant(bytes in 0usize..10_000_000) {
        let meta = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert!(meta.tokens_min <= meta.tokens_est,
            "tokens_min ({}) > tokens_est ({}) for bytes={}",
            meta.tokens_min, meta.tokens_est, bytes);
        prop_assert!(meta.tokens_est <= meta.tokens_max,
            "tokens_est ({}) > tokens_max ({}) for bytes={}",
            meta.tokens_est, meta.tokens_max, bytes);
        prop_assert_eq!(meta.source_bytes, bytes);
    }

    #[test]
    fn token_estimation_custom_bounds_invariant(
        bytes in 0usize..10_000_000,
        bpt_est in 2.0f64..10.0,
    ) {
        let bpt_low = (bpt_est * 0.5).max(0.1);
        let bpt_high = bpt_est * 2.0;
        let meta = TokenEstimationMeta::from_bytes_with_bounds(bytes, bpt_est, bpt_low, bpt_high);
        prop_assert!(meta.tokens_min <= meta.tokens_est,
            "tokens_min ({}) > tokens_est ({}) for bytes={}, bpt_est={}",
            meta.tokens_min, meta.tokens_est, bytes, bpt_est);
        prop_assert!(meta.tokens_est <= meta.tokens_max,
            "tokens_est ({}) > tokens_max ({}) for bytes={}, bpt_est={}",
            meta.tokens_est, meta.tokens_max, bytes, bpt_est);
    }

    // ========================
    // TokenAudit invariants
    // ========================

    #[test]
    fn token_audit_invariant(
        output_bytes in 0u64..10_000_000,
        content_bytes in 0u64..10_000_000,
    ) {
        let audit = TokenAudit::from_output(output_bytes, content_bytes);
        prop_assert_eq!(audit.output_bytes, output_bytes);
        // overhead is always <= output
        prop_assert!(audit.overhead_bytes <= output_bytes);
        // overhead_pct in [0.0, 1.0]
        prop_assert!(audit.overhead_pct >= 0.0);
        prop_assert!(audit.overhead_pct <= 1.0);
        // token ordering: min <= est <= max
        prop_assert!(audit.tokens_min <= audit.tokens_est);
        prop_assert!(audit.tokens_est <= audit.tokens_max);
    }

    // ========================
    // Enum round-trips
    // ========================

    #[test]
    fn children_mode_roundtrip(idx in 0usize..2) {
        let mode = [ChildrenMode::Collapse, ChildrenMode::Separate][idx];
        let json = serde_json::to_string(&mode).expect("serialize");
        let parsed: ChildrenMode = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn child_include_mode_roundtrip(idx in 0usize..2) {
        let mode = [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly][idx];
        let json = serde_json::to_string(&mode).expect("serialize");
        let parsed: ChildIncludeMode = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn table_format_roundtrip(idx in 0usize..3) {
        let fmt = [TableFormat::Md, TableFormat::Tsv, TableFormat::Json][idx];
        let json = serde_json::to_string(&fmt).expect("serialize");
        let parsed: TableFormat = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(fmt, parsed);
    }

    #[test]
    fn export_format_roundtrip(idx in 0usize..4) {
        let fmt = [ExportFormat::Csv, ExportFormat::Jsonl, ExportFormat::Json, ExportFormat::Cyclonedx][idx];
        let json = serde_json::to_string(&fmt).expect("serialize");
        let parsed: ExportFormat = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(fmt, parsed);
    }

    #[test]
    fn config_mode_roundtrip(idx in 0usize..2) {
        let mode = [ConfigMode::Auto, ConfigMode::None][idx];
        let json = serde_json::to_string(&mode).expect("serialize");
        let parsed: ConfigMode = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn redact_mode_roundtrip(idx in 0usize..3) {
        let mode = [RedactMode::None, RedactMode::Paths, RedactMode::All][idx];
        let json = serde_json::to_string(&mode).expect("serialize");
        let parsed: RedactMode = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn inclusion_policy_roundtrip(idx in 0usize..4) {
        let policy = [InclusionPolicy::Full, InclusionPolicy::HeadTail,
                      InclusionPolicy::Summary, InclusionPolicy::Skip][idx];
        let json = serde_json::to_string(&policy).expect("serialize");
        let parsed: InclusionPolicy = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(policy, parsed);
    }

    #[test]
    fn file_classification_roundtrip(idx in 0usize..7) {
        let cls = [
            FileClassification::Generated, FileClassification::Fixture,
            FileClassification::Vendored, FileClassification::Lockfile,
            FileClassification::Minified, FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ][idx];
        let json = serde_json::to_string(&cls).expect("serialize");
        let parsed: FileClassification = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(cls, parsed);
    }

    #[test]
    fn commit_intent_kind_roundtrip(idx in 0usize..12) {
        let kind = [
            CommitIntentKind::Feat, CommitIntentKind::Fix, CommitIntentKind::Refactor,
            CommitIntentKind::Docs, CommitIntentKind::Test, CommitIntentKind::Chore,
            CommitIntentKind::Ci, CommitIntentKind::Build, CommitIntentKind::Perf,
            CommitIntentKind::Style, CommitIntentKind::Revert, CommitIntentKind::Other,
        ][idx];
        let json = serde_json::to_string(&kind).expect("serialize");
        let parsed: CommitIntentKind = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(kind, parsed);
    }

    // ========================
    // Cockpit enum round-trips
    // ========================

    #[test]
    fn gate_status_roundtrip(idx in 0usize..5) {
        let s = [GateStatus::Pass, GateStatus::Warn, GateStatus::Fail,
                 GateStatus::Skipped, GateStatus::Pending][idx];
        let json = serde_json::to_string(&s).expect("serialize");
        let parsed: GateStatus = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(s, parsed);
    }

    #[test]
    fn risk_level_roundtrip(idx in 0usize..4) {
        let r = [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Critical][idx];
        let json = serde_json::to_string(&r).expect("serialize");
        let parsed: RiskLevel = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(r, parsed);
    }

    #[test]
    fn complexity_indicator_roundtrip(idx in 0usize..4) {
        let c = [ComplexityIndicator::Low, ComplexityIndicator::Medium,
                 ComplexityIndicator::High, ComplexityIndicator::Critical][idx];
        let json = serde_json::to_string(&c).expect("serialize");
        let parsed: ComplexityIndicator = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(c, parsed);
    }

    #[test]
    fn warning_type_roundtrip(idx in 0usize..5) {
        let w = [WarningType::LargeFile, WarningType::HighChurn,
                 WarningType::LowTestCoverage, WarningType::ComplexChange,
                 WarningType::BusFactor][idx];
        let json = serde_json::to_string(&w).expect("serialize");
        let parsed: WarningType = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(w, parsed);
    }

    #[test]
    fn trend_direction_roundtrip(idx in 0usize..3) {
        let d = [TrendDirection::Improving, TrendDirection::Stable,
                 TrendDirection::Degrading][idx];
        let json = serde_json::to_string(&d).expect("serialize");
        let parsed: TrendDirection = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(d, parsed);
    }

    #[test]
    fn evidence_source_roundtrip(idx in 0usize..3) {
        let s = [EvidenceSource::CiArtifact, EvidenceSource::Cached,
                 EvidenceSource::RanLocal][idx];
        let json = serde_json::to_string(&s).expect("serialize");
        let parsed: EvidenceSource = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(s, parsed);
    }

    #[test]
    fn commit_match_roundtrip(idx in 0usize..4) {
        let m = [CommitMatch::Exact, CommitMatch::Partial,
                 CommitMatch::Stale, CommitMatch::Unknown][idx];
        let json = serde_json::to_string(&m).expect("serialize");
        let parsed: CommitMatch = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(m, parsed);
    }

    // ========================
    // AnalysisFormat Round-trip
    // ========================

    #[test]
    fn analysis_format_roundtrip(idx in 0usize..10) {
        let fmt = [
            AnalysisFormat::Md, AnalysisFormat::Json, AnalysisFormat::Jsonld,
            AnalysisFormat::Xml, AnalysisFormat::Svg, AnalysisFormat::Mermaid,
            AnalysisFormat::Obj, AnalysisFormat::Midi, AnalysisFormat::Tree,
            AnalysisFormat::Html,
        ][idx];
        let json = serde_json::to_string(&fmt).expect("serialize");
        let parsed: AnalysisFormat = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(fmt, parsed);
    }

    // ========================
    // ScanStatus Round-trip
    // ========================

    #[test]
    fn scan_status_roundtrip(idx in 0usize..2) {
        let status = [ScanStatus::Complete, ScanStatus::Partial][idx].clone();
        let json = serde_json::to_string(&status).expect("serialize");
        let parsed: ScanStatus = serde_json::from_str(&json).expect("deserialize");
        let json2 = serde_json::to_string(&parsed).expect("serialize again");
        prop_assert_eq!(json, json2);
    }

    // ========================
    // ToolInfo Round-trip
    // ========================

    #[test]
    fn tool_info_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9_-]*",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let info = ToolInfo { name: name.clone(), version: version.clone() };
        let json = serde_json::to_string(&info).expect("serialize");
        let parsed: ToolInfo = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&info.name, &parsed.name);
        prop_assert_eq!(&info.version, &parsed.version);
    }

    // ========================
    // CapabilityState Round-trip (types version)
    // ========================

    #[test]
    fn types_capability_state_roundtrip(idx in 0usize..3) {
        let state = [
            tokmd_types::CapabilityState::Available,
            tokmd_types::CapabilityState::Skipped,
            tokmd_types::CapabilityState::Unavailable,
        ][idx];
        let json = serde_json::to_string(&state).expect("serialize");
        let parsed: tokmd_types::CapabilityState =
            serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(state, parsed);
    }

    // ========================
    // ArtifactEntry Round-trip
    // ========================

    #[test]
    fn artifact_entry_roundtrip(
        name in "[a-zA-Z0-9_-]{1,20}",
        path in "[a-zA-Z0-9_/.-]{1,40}",
        description in "[A-Za-z0-9 ]{1,60}",
        bytes in 0u64..10_000_000,
    ) {
        let entry = ArtifactEntry {
            name, path, description, bytes,
            hash: Some(ArtifactHash {
                algo: "blake3".to_string(),
                hash: "abc123def456".to_string(),
            }),
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let parsed: ArtifactEntry = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&entry.name, &parsed.name);
        prop_assert_eq!(&entry.path, &parsed.path);
        prop_assert_eq!(&entry.description, &parsed.description);
        prop_assert_eq!(entry.bytes, parsed.bytes);
        prop_assert!(parsed.hash.is_some());
    }

    // ========================
    // SmartExcludedFile Round-trip
    // ========================

    #[test]
    fn smart_excluded_file_roundtrip(
        path in "[a-zA-Z0-9_/.-]{1,40}",
        reason in "[A-Za-z0-9 ]{1,40}",
        tokens in 0usize..1_000_000,
    ) {
        let file = SmartExcludedFile { path, reason, tokens };
        let json = serde_json::to_string(&file).expect("serialize");
        let parsed: SmartExcludedFile = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&file.path, &parsed.path);
        prop_assert_eq!(&file.reason, &parsed.reason);
        prop_assert_eq!(file.tokens, parsed.tokens);
    }

    // ========================
    // PolicyExcludedFile Round-trip
    // ========================

    #[test]
    fn policy_excluded_file_roundtrip(
        path in "[a-zA-Z0-9_/.-]{1,40}",
        original_tokens in 0usize..1_000_000,
        reason in "[A-Za-z0-9 ]{1,40}",
    ) {
        let file = PolicyExcludedFile {
            path, original_tokens,
            policy: InclusionPolicy::Skip,
            reason,
            classifications: vec![FileClassification::Minified],
        };
        let json = serde_json::to_string(&file).expect("serialize");
        let parsed: PolicyExcludedFile = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&file.path, &parsed.path);
        prop_assert_eq!(file.original_tokens, parsed.original_tokens);
        prop_assert_eq!(file.classifications.len(), parsed.classifications.len());
    }

    // ========================
    // HandoffHotspot Round-trip
    // ========================

    #[test]
    fn handoff_hotspot_roundtrip(
        path in "[a-zA-Z0-9_/.-]{1,40}",
        commits in 0usize..10000,
        lines in 0usize..100000,
        score in 0usize..100000,
    ) {
        let hotspot = HandoffHotspot { path, commits, lines, score };
        let json = serde_json::to_string(&hotspot).expect("serialize");
        let parsed: HandoffHotspot = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&hotspot.path, &parsed.path);
        prop_assert_eq!(hotspot.commits, parsed.commits);
        prop_assert_eq!(hotspot.lines, parsed.lines);
        prop_assert_eq!(hotspot.score, parsed.score);
    }

    // ========================
    // HandoffDerived Round-trip
    // ========================

    #[test]
    fn handoff_derived_roundtrip(
        total_files in 0usize..100000,
        total_code in 0usize..10_000_000,
        total_lines in 0usize..20_000_000,
        total_tokens in 0usize..100_000_000,
        lang_count in 0usize..100,
        dominant_pct in 0.0f64..100.0,
    ) {
        let derived = HandoffDerived {
            total_files, total_code, total_lines, total_tokens, lang_count,
            dominant_lang: "Rust".to_string(),
            dominant_pct,
        };
        let json = serde_json::to_string(&derived).expect("serialize");
        let parsed: HandoffDerived = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(derived.total_files, parsed.total_files);
        prop_assert_eq!(derived.total_code, parsed.total_code);
        prop_assert_eq!(derived.total_lines, parsed.total_lines);
        prop_assert_eq!(derived.total_tokens, parsed.total_tokens);
        prop_assert_eq!(derived.lang_count, parsed.lang_count);
        prop_assert_eq!(&derived.dominant_lang, &parsed.dominant_lang);
    }

    // ========================
    // ContextExcludedPath Round-trip
    // ========================

    #[test]
    fn context_excluded_path_roundtrip(
        path in "[a-zA-Z0-9_/.-]{1,40}",
        reason in "[A-Za-z0-9 ]{1,40}",
    ) {
        let excl = ContextExcludedPath { path, reason };
        let json = serde_json::to_string(&excl).expect("serialize");
        let parsed: ContextExcludedPath = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&excl.path, &parsed.path);
        prop_assert_eq!(&excl.reason, &parsed.reason);
    }

    // ========================
    // Serialization determinism
    // ========================

    #[test]
    fn totals_serialization_deterministic(totals in arb_totals()) {
        let json1 = serde_json::to_string(&totals).expect("serialize");
        let json2 = serde_json::to_string(&totals).expect("serialize again");
        prop_assert_eq!(json1, json2);
    }

    #[test]
    fn token_estimation_from_bytes_deterministic(bytes in 0usize..10_000_000) {
        let m1 = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        let m2 = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert_eq!(m1.tokens_min, m2.tokens_min);
        prop_assert_eq!(m1.tokens_est, m2.tokens_est);
        prop_assert_eq!(m1.tokens_max, m2.tokens_max);
        prop_assert_eq!(m1.source_bytes, m2.source_bytes);
    }

    // ========================
    // DiffRow delta consistency
    // ========================

    #[test]
    fn diff_row_delta_matches_old_new(
        old_code in 0usize..100000,
        new_code in 0usize..100000,
    ) {
        let expected_delta = new_code as i64 - old_code as i64;
        let row = DiffRow {
            lang: "Rust".to_string(),
            old_code, new_code,
            delta_code: expected_delta,
            old_lines: 0, new_lines: 0, delta_lines: 0,
            old_files: 0, new_files: 0, delta_files: 0,
            old_bytes: 0, new_bytes: 0, delta_bytes: 0,
            old_tokens: 0, new_tokens: 0, delta_tokens: 0,
        };
        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: DiffRow = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(parsed.delta_code, parsed.new_code as i64 - parsed.old_code as i64);
    }
}

// ========================
// ToolInfo Tests (outside proptest! macro for simpler testing)
// ========================

#[test]
fn tool_info_current_returns_correct_name() {
    let info = tokmd_types::ToolInfo::current();
    assert_eq!(
        info.name, "tokmd",
        "ToolInfo::current() should return 'tokmd' as the tool name"
    );
}

#[test]
fn tool_info_current_returns_non_empty_version() {
    let info = tokmd_types::ToolInfo::current();
    assert!(
        !info.version.is_empty(),
        "ToolInfo::current() should return a non-empty version string"
    );
}

#[test]
fn tool_info_current_differs_from_default() {
    let current = tokmd_types::ToolInfo::current();
    let default = tokmd_types::ToolInfo::default();

    // current() should not return the same as default()
    assert_ne!(
        current.name, default.name,
        "ToolInfo::current() should not return empty name like default"
    );
    assert_ne!(
        current.version, default.version,
        "ToolInfo::current() should not return empty version like default"
    );
}

// ========================
// TokenEstimationMeta serde alias tests
// ========================

#[test]
fn token_estimation_meta_old_field_aliases() {
    // Old JSON used tokens_high / tokens_low; aliases must map them to tokens_min / tokens_max.
    let json = serde_json::json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_high": 200,
        "tokens_est": 250,
        "tokens_low": 334,
        "source_bytes": 1000
    });

    let parsed: TokenEstimationMeta =
        serde_json::from_value(json).expect("deserialize with old field names");

    assert_eq!(
        parsed.tokens_min, 200,
        "tokens_high should alias to tokens_min"
    );
    assert_eq!(parsed.tokens_est, 250);
    assert_eq!(
        parsed.tokens_max, 334,
        "tokens_low should alias to tokens_max"
    );
    assert_eq!(parsed.source_bytes, 1000);
    assert!((parsed.bytes_per_token_est - 4.0).abs() < f64::EPSILON);
    assert!((parsed.bytes_per_token_low - 3.0).abs() < f64::EPSILON);
    assert!((parsed.bytes_per_token_high - 5.0).abs() < f64::EPSILON);
}

#[test]
fn token_estimation_meta_roundtrip() {
    let meta = TokenEstimationMeta::from_bytes(1000, TokenEstimationMeta::DEFAULT_BPT_EST);

    let json_str = serde_json::to_string(&meta).expect("serialize");

    // New field names must appear in serialized output.
    assert!(
        json_str.contains("\"tokens_min\""),
        "should serialize as tokens_min"
    );
    assert!(
        json_str.contains("\"tokens_max\""),
        "should serialize as tokens_max"
    );
    assert!(
        !json_str.contains("\"tokens_high\""),
        "old name tokens_high must not appear"
    );
    assert!(
        !json_str.contains("\"tokens_low\""),
        "old name tokens_low must not appear"
    );

    let parsed: TokenEstimationMeta =
        serde_json::from_str(&json_str).expect("deserialize roundtrip");

    assert_eq!(parsed.tokens_min, meta.tokens_min);
    assert_eq!(parsed.tokens_est, meta.tokens_est);
    assert_eq!(parsed.tokens_max, meta.tokens_max);
    assert_eq!(parsed.source_bytes, meta.source_bytes);
    assert!((parsed.bytes_per_token_est - meta.bytes_per_token_est).abs() < f64::EPSILON);
    assert!((parsed.bytes_per_token_low - meta.bytes_per_token_low).abs() < f64::EPSILON);
    assert!((parsed.bytes_per_token_high - meta.bytes_per_token_high).abs() < f64::EPSILON);
}

// ========================
// TokenAudit serde alias tests
// ========================

#[test]
fn token_audit_old_field_aliases() {
    // Old JSON used tokens_high / tokens_low; aliases must map them to tokens_min / tokens_max.
    let json = serde_json::json!({
        "output_bytes": 5000,
        "tokens_high": 1000,
        "tokens_est": 1250,
        "tokens_low": 1667,
        "overhead_bytes": 200,
        "overhead_pct": 0.04
    });

    let parsed: TokenAudit =
        serde_json::from_value(json).expect("deserialize with old field names");

    assert_eq!(
        parsed.tokens_min, 1000,
        "tokens_high should alias to tokens_min"
    );
    assert_eq!(parsed.tokens_est, 1250);
    assert_eq!(
        parsed.tokens_max, 1667,
        "tokens_low should alias to tokens_max"
    );
    assert_eq!(parsed.output_bytes, 5000);
    assert_eq!(parsed.overhead_bytes, 200);
    assert!((parsed.overhead_pct - 0.04).abs() < f64::EPSILON);
}

#[test]
fn token_audit_roundtrip() {
    let audit = TokenAudit::from_output(5000, 4800);

    let json_str = serde_json::to_string(&audit).expect("serialize");

    // New field names must appear in serialized output.
    assert!(
        json_str.contains("\"tokens_min\""),
        "should serialize as tokens_min"
    );
    assert!(
        json_str.contains("\"tokens_max\""),
        "should serialize as tokens_max"
    );
    assert!(
        !json_str.contains("\"tokens_high\""),
        "old name tokens_high must not appear"
    );
    assert!(
        !json_str.contains("\"tokens_low\""),
        "old name tokens_low must not appear"
    );

    let parsed: TokenAudit = serde_json::from_str(&json_str).expect("deserialize roundtrip");

    assert_eq!(parsed.tokens_min, audit.tokens_min);
    assert_eq!(parsed.tokens_est, audit.tokens_est);
    assert_eq!(parsed.tokens_max, audit.tokens_max);
    assert_eq!(parsed.output_bytes, audit.output_bytes);
    assert_eq!(parsed.overhead_bytes, audit.overhead_bytes);
    assert!((parsed.overhead_pct - audit.overhead_pct).abs() < f64::EPSILON);
}

// ========================
// Full ContextReceipt E2E backward compatibility test
// ========================

#[test]
fn context_receipt_token_rename_backward_compat() {
    // Build a valid ContextReceipt with token_estimation and bundle_audit populated.
    let estimation = TokenEstimationMeta::from_bytes(10_000, TokenEstimationMeta::DEFAULT_BPT_EST);
    let audit = TokenAudit::from_output(12_000, 10_000);

    let receipt = ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo::current(),
        mode: "bundle".to_string(),
        budget_tokens: 128_000,
        used_tokens: 2_500,
        utilization_pct: 1.95,
        strategy: "greedy".to_string(),
        rank_by: "code".to_string(),
        file_count: 1,
        files: vec![],
        rank_by_effective: None,
        fallback_reason: None,
        excluded_by_policy: vec![],
        token_estimation: Some(estimation.clone()),
        bundle_audit: Some(audit.clone()),
    };

    // Serialize to JSON (uses new field names: tokens_min, tokens_max).
    let json_str = serde_json::to_string_pretty(&receipt).expect("serialize receipt");

    // Simulate old-format JSON by replacing new names with old aliases.
    let old_json = json_str
        .replace("\"tokens_min\"", "\"tokens_high\"")
        .replace("\"tokens_max\"", "\"tokens_low\"");

    // Verify old names are present and new names are gone.
    assert!(old_json.contains("\"tokens_high\""));
    assert!(old_json.contains("\"tokens_low\""));
    assert!(!old_json.contains("\"tokens_min\""));
    assert!(!old_json.contains("\"tokens_max\""));

    // Deserialize the old-format JSON back into ContextReceipt.
    let parsed: ContextReceipt =
        serde_json::from_str(&old_json).expect("deserialize with old field names");

    // Assert the token_estimation values round-tripped correctly.
    let parsed_est = parsed
        .token_estimation
        .expect("token_estimation should be present");
    assert_eq!(parsed_est.tokens_min, estimation.tokens_min);
    assert_eq!(parsed_est.tokens_est, estimation.tokens_est);
    assert_eq!(parsed_est.tokens_max, estimation.tokens_max);
    assert_eq!(parsed_est.source_bytes, estimation.source_bytes);

    // Assert the bundle_audit values round-tripped correctly.
    let parsed_audit = parsed.bundle_audit.expect("bundle_audit should be present");
    assert_eq!(parsed_audit.tokens_min, audit.tokens_min);
    assert_eq!(parsed_audit.tokens_est, audit.tokens_est);
    assert_eq!(parsed_audit.tokens_max, audit.tokens_max);
    assert_eq!(parsed_audit.output_bytes, audit.output_bytes);
}
