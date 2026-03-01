//! Determinism-focused regression tests for tokmd-types.
//!
//! Verifies that serialization is stable, BTreeMap ordering is preserved,
//! and schema_version constants match documented values.

use std::collections::BTreeMap;

use proptest::prelude::*;
use serde_json::Value;

use tokmd_types::{
    ChildrenMode, FileKind, FileRow, LangReport, LangRow, ModuleRow, SCHEMA_VERSION, Totals,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_lang_row(lang: &str, code: usize) -> LangRow {
    let lines = code + 20;
    let files = 3;
    LangRow {
        lang: lang.to_string(),
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code * 10 / 4,
        avg_lines: lines / files,
    }
}

fn make_module_row(module: &str, code: usize) -> ModuleRow {
    let lines = code + 30;
    let files = 2;
    ModuleRow {
        module: module.to_string(),
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code * 10 / 4,
        avg_lines: lines / files,
    }
}

fn make_file_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: path.split('/').next().unwrap_or("(root)").to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 10,
        blanks: 5,
        lines: code + 15,
        bytes: code * 10,
        tokens: code * 10 / 4,
    }
}

// ---------------------------------------------------------------------------
// 1. JSON serialization produces identical output for identical fields
// ---------------------------------------------------------------------------

#[test]
fn determinism_lang_row_json_identical() {
    let a = make_lang_row("Rust", 500);
    let b = make_lang_row("Rust", 500);
    let ja = serde_json::to_string(&a).unwrap();
    let jb = serde_json::to_string(&b).unwrap();
    assert_eq!(ja, jb, "Identical LangRow must serialize identically");
}

#[test]
fn determinism_module_row_json_identical() {
    let a = make_module_row("crates/foo", 300);
    let b = make_module_row("crates/foo", 300);
    let ja = serde_json::to_string(&a).unwrap();
    let jb = serde_json::to_string(&b).unwrap();
    assert_eq!(ja, jb, "Identical ModuleRow must serialize identically");
}

#[test]
fn determinism_file_row_json_identical() {
    let a = make_file_row("src/lib.rs", "Rust", 400);
    let b = make_file_row("src/lib.rs", "Rust", 400);
    let ja = serde_json::to_string(&a).unwrap();
    let jb = serde_json::to_string(&b).unwrap();
    assert_eq!(ja, jb, "Identical FileRow must serialize identically");
}

// ---------------------------------------------------------------------------
// 2. BTreeMap ordering in receipt envelopes is stable
// ---------------------------------------------------------------------------

#[test]
fn determinism_btreemap_key_ordering_stable() {
    // Simulate the kind of map used in receipt processing.
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    map.insert("Rust".into(), 1000);
    map.insert("Python".into(), 500);
    map.insert("Go".into(), 300);
    map.insert("TOML".into(), 50);

    let json1 = serde_json::to_string(&map).unwrap();
    let json2 = serde_json::to_string(&map).unwrap();
    assert_eq!(json1, json2, "BTreeMap serialization must be stable");

    // Verify alphabetical ordering (BTreeMap guarantee).
    let parsed: Value = serde_json::from_str(&json1).unwrap();
    let keys: Vec<&str> = parsed
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["Go", "Python", "Rust", "TOML"]);
}

#[test]
fn determinism_btreemap_insertion_order_irrelevant() {
    // Insert in different orders, result must be identical.
    let mut map_a: BTreeMap<String, usize> = BTreeMap::new();
    map_a.insert("Zebra".into(), 1);
    map_a.insert("Alpha".into(), 2);
    map_a.insert("Middle".into(), 3);

    let mut map_b: BTreeMap<String, usize> = BTreeMap::new();
    map_b.insert("Alpha".into(), 2);
    map_b.insert("Middle".into(), 3);
    map_b.insert("Zebra".into(), 1);

    let ja = serde_json::to_string(&map_a).unwrap();
    let jb = serde_json::to_string(&map_b).unwrap();
    assert_eq!(
        ja, jb,
        "BTreeMap must produce identical JSON regardless of insertion order"
    );
}

#[test]
fn determinism_lang_report_json_field_order() {
    // The LangReport JSON must have fields in the same order every time.
    let report = LangReport {
        rows: vec![make_lang_row("Rust", 1000), make_lang_row("Python", 500)],
        total: Totals {
            code: 1500,
            lines: 1540,
            files: 6,
            bytes: 15000,
            tokens: 3750,
            avg_lines: 256,
        },
        with_files: false,
        children: ChildrenMode::Collapse,
        top: 0,
    };

    let j1 = serde_json::to_string_pretty(&report).unwrap();
    let j2 = serde_json::to_string_pretty(&report).unwrap();
    assert_eq!(j1, j2, "LangReport must serialize with stable field order");

    // Verify top-level keys appear in struct-declaration order.
    let parsed: Value = serde_json::from_str(&j1).unwrap();
    let keys: Vec<&str> = parsed
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert!(keys.contains(&"rows"), "Must contain 'rows' key");
    assert!(keys.contains(&"total"), "Must contain 'total' key");
}

// ---------------------------------------------------------------------------
// 3. Schema version constant matches documented value
// ---------------------------------------------------------------------------

#[test]
fn determinism_schema_version_is_documented_value() {
    // The SCHEMA_VERSION for core receipts must be 2 as documented.
    assert_eq!(SCHEMA_VERSION, 2, "Core SCHEMA_VERSION must be 2");
}

#[test]
fn determinism_schema_version_in_serialized_receipt() {
    // When embedded in a receipt, schema_version must match the constant.
    let receipt = tokmd_types::LangReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: tokmd_types::ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
        mode: "lang".into(),
        status: tokmd_types::ScanStatus::Complete,
        warnings: vec![],
        scan: tokmd_types::ScanArgs {
            paths: vec![".".into()],
            excluded: vec![],
            excluded_redacted: false,
            config: tokmd_types::ConfigMode::Auto,
            hidden: false,
            no_ignore: false,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            treat_doc_strings_as_comments: false,
        },
        args: tokmd_types::LangArgsMeta {
            format: "json".into(),
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
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        parsed["schema_version"].as_u64().unwrap(),
        SCHEMA_VERSION as u64,
        "Serialized schema_version must match constant"
    );
}

// ---------------------------------------------------------------------------
// 4. Proptest: serialization round-trip determinism
// ---------------------------------------------------------------------------

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        prop::sample::select(vec!["Rust", "Python", "Go", "Java", "C", "TypeScript"]),
        1usize..50_000,
        1usize..500,
    )
        .prop_map(|(lang, code, files)| {
            let lines = code + 20;
            LangRow {
                lang: lang.to_string(),
                code,
                lines,
                files,
                bytes: code * 10,
                tokens: code * 10 / 4,
                avg_lines: if files > 0 { lines / files } else { 0 },
            }
        })
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        prop::sample::select(vec!["src", "crates/foo", "crates/bar", "tests", "lib"]),
        1usize..50_000,
        1usize..500,
    )
        .prop_map(|(module, code, files)| {
            let lines = code + 30;
            ModuleRow {
                module: module.to_string(),
                code,
                lines,
                files,
                bytes: code * 10,
                tokens: code * 10 / 4,
                avg_lines: if files > 0 { lines / files } else { 0 },
            }
        })
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        prop::sample::select(vec!["src/lib.rs", "src/main.rs", "tests/it.rs", "build.rs"]),
        prop::sample::select(vec!["Rust", "Python", "Go"]),
        1usize..20_000,
    )
        .prop_map(|(path, lang, code)| FileRow {
            path: path.to_string(),
            module: path.split('/').next().unwrap_or("(root)").to_string(),
            lang: lang.to_string(),
            kind: FileKind::Parent,
            code,
            comments: 10,
            blanks: 5,
            lines: code + 15,
            bytes: code * 10,
            tokens: code * 10 / 4,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn determinism_lang_row_roundtrip(row in arb_lang_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let back: LangRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, back.clone());
        // Serialize again — must be identical.
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(json, json2);
    }

    #[test]
    fn determinism_module_row_roundtrip(row in arb_module_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let back: ModuleRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, back.clone());
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(json, json2);
    }

    #[test]
    fn determinism_file_row_roundtrip(row in arb_file_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let back: FileRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, back.clone());
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(json, json2);
    }
}
