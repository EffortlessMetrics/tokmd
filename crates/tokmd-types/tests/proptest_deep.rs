//! Deep property-based tests for tokmd-types.
//!
//! Covers enum serde round-trips, schema version invariants,
//! TokenEstimationMeta ordering, DiffRow delta invariants, and
//! serialization casing conventions.

use proptest::prelude::*;
use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    CommitIntentKind, ConfigMode, DiffRow, DiffTotals, ExportFormat, FileClassification, FileKind,
    HANDOFF_SCHEMA_VERSION, InclusionPolicy, RedactMode, SCHEMA_VERSION, TableFormat,
    TokenEstimationMeta, Totals, cockpit::COCKPIT_SCHEMA_VERSION,
};

// =========================================================================
// Enum serde round-trips — every variant survives JSON serialization
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn children_mode_roundtrip(idx in 0usize..2) {
        let mode = [ChildrenMode::Collapse, ChildrenMode::Separate][idx];
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: ChildrenMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn child_include_mode_roundtrip(idx in 0usize..2) {
        let mode = [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly][idx];
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: ChildIncludeMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn table_format_roundtrip(idx in 0usize..2) {
        let fmt = [TableFormat::Md, TableFormat::Tsv][idx];
        let json = serde_json::to_string(&fmt).unwrap();
        let parsed: TableFormat = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(fmt, parsed);
    }

    #[test]
    fn export_format_roundtrip(idx in 0usize..4) {
        let fmt = [
            ExportFormat::Csv,
            ExportFormat::Jsonl,
            ExportFormat::Json,
            ExportFormat::Cyclonedx,
        ][idx];
        let json = serde_json::to_string(&fmt).unwrap();
        let parsed: ExportFormat = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(fmt, parsed);
    }

    #[test]
    fn file_kind_roundtrip(idx in 0usize..2) {
        let kind = [FileKind::Parent, FileKind::Child][idx];
        let json = serde_json::to_string(&kind).unwrap();
        let parsed: FileKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(kind, parsed);
    }

    #[test]
    fn redact_mode_roundtrip(idx in 0usize..3) {
        let mode = [RedactMode::None, RedactMode::Paths, RedactMode::All][idx];
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: RedactMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn config_mode_roundtrip(idx in 0usize..2) {
        let mode = [ConfigMode::Auto, ConfigMode::None][idx];
        let json = serde_json::to_string(&mode).unwrap();
        let parsed: ConfigMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(mode, parsed);
    }

    #[test]
    fn inclusion_policy_roundtrip(idx in 0usize..4) {
        let policy = [
            InclusionPolicy::Full,
            InclusionPolicy::HeadTail,
            InclusionPolicy::Summary,
            InclusionPolicy::Skip,
        ][idx];
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: InclusionPolicy = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(policy, parsed);
    }

    #[test]
    fn file_classification_roundtrip(idx in 0usize..7) {
        let cls = [
            FileClassification::Generated,
            FileClassification::Fixture,
            FileClassification::Vendored,
            FileClassification::Lockfile,
            FileClassification::Minified,
            FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ][idx];
        let json = serde_json::to_string(&cls).unwrap();
        let parsed: FileClassification = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(cls, parsed);
    }

    #[test]
    fn commit_intent_roundtrip(idx in 0usize..5) {
        let intent = [
            CommitIntentKind::Feat,
            CommitIntentKind::Fix,
            CommitIntentKind::Refactor,
            CommitIntentKind::Docs,
            CommitIntentKind::Chore,
        ][idx];
        let json = serde_json::to_string(&intent).unwrap();
        let parsed: CommitIntentKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(intent, parsed);
    }
}

// =========================================================================
// Schema versions are always positive
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn schema_versions_are_positive(_dummy in 0..1u8) {
        prop_assert!(SCHEMA_VERSION > 0);
        prop_assert!(COCKPIT_SCHEMA_VERSION > 0);
        prop_assert!(HANDOFF_SCHEMA_VERSION > 0);
        prop_assert!(CONTEXT_SCHEMA_VERSION > 0);
        prop_assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0);
    }
}

// =========================================================================
// Totals serde round-trip with arbitrary values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn totals_roundtrip_preserves_all_fields(
        code in 0usize..1_000_000,
        lines in 0usize..2_000_000,
        files in 0usize..10_000,
        bytes in 0usize..100_000_000,
        tokens in 0usize..10_000_000,
        avg_lines in 0usize..5000,
    ) {
        let totals = Totals { code, lines, files, bytes, tokens, avg_lines };
        let json = serde_json::to_string(&totals).unwrap();
        let parsed: Totals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(totals, parsed);
    }
}

// =========================================================================
// DiffRow: delta_code always equals new_code - old_code (as i64)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn diff_row_delta_equals_new_minus_old(
        old in 0usize..100_000,
        new in 0usize..100_000,
    ) {
        let row = DiffRow {
            lang: "Rust".into(),
            old_code: old,
            new_code: new,
            delta_code: new as i64 - old as i64,
            old_lines: old,
            new_lines: new,
            delta_lines: new as i64 - old as i64,
            old_files: 1,
            new_files: 1,
            delta_files: 0,
            old_bytes: old * 10,
            new_bytes: new * 10,
            delta_bytes: (new as i64 - old as i64) * 10,
            old_tokens: old / 4,
            new_tokens: new / 4,
            delta_tokens: new as i64 / 4 - old as i64 / 4,
        };
        prop_assert_eq!(row.delta_code, row.new_code as i64 - row.old_code as i64);
    }

    #[test]
    fn diff_row_roundtrip(
        old in 0usize..100_000,
        new in 0usize..100_000,
    ) {
        let row = DiffRow {
            lang: "Go".into(),
            old_code: old,
            new_code: new,
            delta_code: new as i64 - old as i64,
            old_lines: old,
            new_lines: new,
            delta_lines: new as i64 - old as i64,
            old_files: 1,
            new_files: 1,
            delta_files: 0,
            old_bytes: old * 10,
            new_bytes: new * 10,
            delta_bytes: (new as i64 - old as i64) * 10,
            old_tokens: old / 4,
            new_tokens: new / 4,
            delta_tokens: new as i64 / 4 - old as i64 / 4,
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: DiffRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// DiffTotals: default is zeroed
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn diff_totals_default_is_zero(_dummy in 0..1u8) {
        let t = DiffTotals::default();
        prop_assert_eq!(t.old_code, 0);
        prop_assert_eq!(t.new_code, 0);
        prop_assert_eq!(t.delta_code, 0);
    }
}

// =========================================================================
// TokenEstimationMeta: ordering invariant (min <= est <= max)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_estimation_ordering_invariant(
        source_bytes in 1usize..10_000_000,
    ) {
        let meta = TokenEstimationMeta::from_bytes(source_bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert!(
            meta.tokens_min <= meta.tokens_est,
            "min ({}) must be <= est ({})",
            meta.tokens_min, meta.tokens_est
        );
        prop_assert!(
            meta.tokens_est <= meta.tokens_max,
            "est ({}) must be <= max ({})",
            meta.tokens_est, meta.tokens_max
        );
    }

    #[test]
    fn token_estimation_source_bytes_preserved(
        source_bytes in 0usize..10_000_000,
    ) {
        let meta = TokenEstimationMeta::from_bytes(source_bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert_eq!(meta.source_bytes, source_bytes);
    }

    #[test]
    fn token_estimation_zero_bytes_zero_tokens(_dummy in 0..1u8) {
        let meta = TokenEstimationMeta::from_bytes(0, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert_eq!(meta.tokens_min, 0);
        prop_assert_eq!(meta.tokens_est, 0);
        prop_assert_eq!(meta.tokens_max, 0);
    }

    #[test]
    fn token_estimation_custom_bounds_ordering(
        source_bytes in 1usize..10_000_000,
    ) {

        let meta = TokenEstimationMeta::from_bytes(source_bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert!(meta.tokens_min <= meta.tokens_max);
    }
}

// =========================================================================
// Serialization casing conventions
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn enum_serialization_is_lowercase(_dummy in 0..1u8) {
        // spot-check that serde(rename_all) is applied
        let json = serde_json::to_string(&ChildrenMode::Collapse).unwrap();
        prop_assert_eq!(json, "\"collapse\"");
        let json = serde_json::to_string(&TableFormat::Md).unwrap();
        prop_assert_eq!(json, "\"md\"");
        let json = serde_json::to_string(&RedactMode::None).unwrap();
        prop_assert_eq!(json, "\"none\"");
    }

    #[test]
    fn file_classification_serialization_is_snake_case(_dummy in 0..1u8) {
        let json = serde_json::to_string(&FileClassification::Generated).unwrap();
        let s = json.trim_matches('"');
        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "FileClassification should be lowercase: {}", s
        );
    }

    #[test]
    fn commit_intent_serialization_is_snake_case(_dummy in 0..1u8) {
        let json = serde_json::to_string(&CommitIntentKind::Feat).unwrap();
        let s = json.trim_matches('"');
        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "CommitIntentKind should be lowercase: {}", s
        );
    }
}
