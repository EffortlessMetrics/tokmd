//! Property-based tests for tokmd-types serialization.
//!
//! These tests verify that core data types round-trip correctly through JSON.

use proptest::prelude::*;
use tokmd_types::{FileKind, FileRow, LangRow, ModuleRow, TokenAudit, TokenEstimationMeta, Totals};

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
