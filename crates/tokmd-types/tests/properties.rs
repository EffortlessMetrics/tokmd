//! Property-based tests for tokmd-types serialization.
//!
//! These tests verify that core data types round-trip correctly through JSON.

use proptest::prelude::*;
use tokmd_types::{FileKind, FileRow, LangRow, ModuleRow, Totals};

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
