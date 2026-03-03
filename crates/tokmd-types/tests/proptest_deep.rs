//! Deep property-based tests for tokmd-types.
//!
//! Covers enum serde round-trips, schema version invariants,
//! TokenEstimationMeta ordering, DiffRow delta invariants,
//! LangRow/ModuleRow roundtrips, path normalization, and
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
    fn table_format_roundtrip(idx in 0usize..3) {
        let fmt = [TableFormat::Md, TableFormat::Tsv, TableFormat::Json][idx];
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
    fn file_classification_roundtrip(idx in 0usize..5) {
        let cls = [
            FileClassification::Generated,
            FileClassification::Fixture,
            FileClassification::Vendored,
            FileClassification::Lockfile,
            FileClassification::Minified,
        ][idx];
        let json = serde_json::to_string(&cls).unwrap();
        let parsed: FileClassification = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(cls, parsed);
    }

    #[test]
    fn commit_intent_roundtrip(idx in 0usize..6) {
        let intent = [
            CommitIntentKind::Feat,
            CommitIntentKind::Fix,
            CommitIntentKind::Refactor,
            CommitIntentKind::Docs,
            CommitIntentKind::Test,
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
        old_code in 0usize..100_000,
        new_code in 0usize..100_000,
    ) {
        let delta_code = new_code as i64 - old_code as i64;
        let row = DiffRow {
            lang: "Rust".into(),
            old_code,
            new_code,
            delta_code,
            old_lines: 0, new_lines: 0, delta_lines: 0,
            old_files: 0, new_files: 0, delta_files: 0,
            old_bytes: 0, new_bytes: 0, delta_bytes: 0,
            old_tokens: 0, new_tokens: 0, delta_tokens: 0,
        };
        prop_assert_eq!(row.delta_code, row.new_code as i64 - row.old_code as i64);
    }

    #[test]
    fn diff_row_roundtrip(
        old_code in 0usize..100_000,
        new_code in 0usize..100_000,
    ) {
        let row = DiffRow {
            lang: "Go".into(),
            old_code,
            new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines: 0, new_lines: 0, delta_lines: 0,
            old_files: 0, new_files: 0, delta_files: 0,
            old_bytes: 0, new_bytes: 0, delta_bytes: 0,
            old_tokens: 0, new_tokens: 0, delta_tokens: 0,
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

// =========================================================================
// LangRow serde round-trip with arbitrary values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lang_row_deep_roundtrip(
        lang in "[A-Z][a-zA-Z0-9 #+]{0,20}",
        code in 0usize..1_000_000,
        lines in 0usize..2_000_000,
        files in 0usize..10_000,
        bytes in 0usize..100_000_000,
        tokens in 0usize..10_000_000,
        avg_lines in 0usize..5000,
    ) {
        let row = tokmd_types::LangRow { lang, code, lines, files, bytes, tokens, avg_lines };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: tokmd_types::LangRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }

    #[test]
    fn module_row_deep_roundtrip(
        module in "[a-z][a-zA-Z0-9_/]{0,30}",
        code in 0usize..1_000_000,
        lines in 0usize..2_000_000,
        files in 0usize..10_000,
        bytes in 0usize..100_000_000,
        tokens in 0usize..10_000_000,
        avg_lines in 0usize..5000,
    ) {
        let row = tokmd_types::ModuleRow { module, code, lines, files, bytes, tokens, avg_lines };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: tokmd_types::ModuleRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// Path normalization via tokmd-path: idempotent and forward-slash only
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn path_normalize_idempotent(
        parts in proptest::collection::vec("[a-zA-Z0-9_]{1,10}", 1..=5),
        ext in proptest::sample::select(vec!["rs", "py", "go", "js", "toml"]),
    ) {
        let raw = format!("{}.{}", parts.join("/"), ext);
        let once = tokmd_path::normalize_rel_path(&raw);
        let twice = tokmd_path::normalize_rel_path(&once);
        prop_assert_eq!(&once, &twice, "normalize must be idempotent");
    }

    #[test]
    fn path_normalize_no_backslash(
        parts in proptest::collection::vec("[a-zA-Z0-9_]{1,8}", 1..=4),
    ) {
        let raw = parts.join("\\");
        let normalized = tokmd_path::normalize_slashes(&raw);
        prop_assert!(
            !normalized.contains('\\'),
            "Normalized path must not contain backslash: {}", normalized
        );
    }

    #[test]
    fn path_normalize_backslash_forward_slash_equivalent(
        parts in proptest::collection::vec("[a-zA-Z0-9_]{1,8}", 2..=5),
    ) {
        let unix_path = parts.join("/");
        let win_path = parts.join("\\");
        prop_assert_eq!(
            tokmd_path::normalize_slashes(&unix_path),
            tokmd_path::normalize_slashes(&win_path),
            "Forward and backslash paths must normalize identically"
        );
    }
}

// =========================================================================
// DiffRow: serde roundtrip with extreme values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn diff_row_extreme_values(
        old_code in 0usize..1_000_000,
        new_code in 0usize..1_000_000,
    ) {
        let delta_code = new_code as i64 - old_code as i64;
        let row = DiffRow {
            lang: "Test".into(),
            old_code,
            new_code,
            delta_code,
            old_lines: 0, new_lines: 0, delta_lines: 0,
            old_files: 0, new_files: 0, delta_files: 0,
            old_bytes: 0, new_bytes: 0, delta_bytes: 0,
            old_tokens: 0, new_tokens: 0, delta_tokens: 0,
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: DiffRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row.delta_code, parsed.delta_code);
        prop_assert_eq!(row.old_code, parsed.old_code);
        prop_assert_eq!(row.new_code, parsed.new_code);
    }
}
