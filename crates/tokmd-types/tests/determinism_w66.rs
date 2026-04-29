//! Determinism hardening tests for tokmd-types.
//!
//! The #1 invariant: same input must yield byte-identical output.

use proptest::prelude::*;
use tokmd_types::*;

// -- Helpers --

fn sample_lang_row(lang: &str, code: usize) -> LangRow {
    LangRow {
        lang: lang.to_string(),
        code,
        lines: code + 50,
        files: 3,
        bytes: code * 4,
        tokens: code,
        avg_lines: (code + 50) / 3,
    }
}

fn sample_module_row(module: &str, code: usize) -> ModuleRow {
    ModuleRow {
        module: module.to_string(),
        code,
        lines: code + 40,
        files: 2,
        bytes: code * 4,
        tokens: code,
        avg_lines: (code + 40) / 2,
    }
}

fn sample_file_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 10,
        blanks: 5,
        lines: code + 15,
        bytes: code * 4,
        tokens: code,
    }
}

fn sample_totals(code: usize) -> Totals {
    Totals {
        code,
        lines: code + 100,
        files: 5,
        bytes: code * 4,
        tokens: code,
        avg_lines: (code + 100) / 5,
    }
}

fn sample_diff_row(lang: &str, old_code: usize, new_code: usize) -> DiffRow {
    DiffRow {
        lang: lang.to_string(),
        old_code,
        new_code,
        delta_code: new_code as i64 - old_code as i64,
        old_lines: old_code + 50,
        new_lines: new_code + 50,
        delta_lines: new_code as i64 - old_code as i64,
        old_files: 3,
        new_files: 4,
        delta_files: 1,
        old_bytes: old_code * 4,
        new_bytes: new_code * 4,
        delta_bytes: (new_code as i64 - old_code as i64) * 4,
        old_tokens: old_code,
        new_tokens: new_code,
        delta_tokens: new_code as i64 - old_code as i64,
    }
}

// -- 1. LangRow serialization round-trip --

#[test]
fn lang_row_serialize_roundtrip_is_stable() {
    let row = sample_lang_row("Rust", 500);
    let json1 = serde_json::to_string(&row).unwrap();
    let deserialized: LangRow = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(
        json1, json2,
        "serialize-deserialize-serialize must be identical"
    );
}

#[test]
fn lang_row_repeated_serialization_is_byte_stable() {
    let row = sample_lang_row("Python", 1000);
    let outputs: Vec<String> = (0..100)
        .map(|_| serde_json::to_string(&row).unwrap())
        .collect();
    assert!(outputs.windows(2).all(|w| w[0] == w[1]));
}

// -- 2. ModuleRow serialization round-trip --

#[test]
fn module_row_serialize_roundtrip_is_stable() {
    let row = sample_module_row("crates/tokmd", 800);
    let json1 = serde_json::to_string(&row).unwrap();
    let deserialized: ModuleRow = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn module_row_repeated_serialization_is_byte_stable() {
    let row = sample_module_row("src", 200);
    let outputs: Vec<String> = (0..100)
        .map(|_| serde_json::to_string(&row).unwrap())
        .collect();
    assert!(outputs.windows(2).all(|w| w[0] == w[1]));
}

// -- 3. FileRow serialization round-trip --

#[test]
fn file_row_serialize_roundtrip_is_stable() {
    let row = sample_file_row("src/main.rs", "Rust", 120);
    let json1 = serde_json::to_string(&row).unwrap();
    let deserialized: FileRow = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn file_row_child_kind_roundtrip_is_stable() {
    let row = FileRow {
        kind: FileKind::Child,
        ..sample_file_row("index.html", "HTML", 50)
    };
    let json1 = serde_json::to_string(&row).unwrap();
    let deserialized: FileRow = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json1, json2);
}

// -- 4. Totals serialization round-trip --

#[test]
fn totals_serialize_roundtrip_is_stable() {
    let t = sample_totals(5000);
    let json1 = serde_json::to_string(&t).unwrap();
    let deserialized: Totals = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json1, json2);
}

// -- 5. BTreeMap ordering in LangReport --

#[test]
fn lang_report_rows_serialization_order_is_stable() {
    let report = LangReport {
        rows: vec![
            sample_lang_row("Rust", 500),
            sample_lang_row("Python", 300),
            sample_lang_row("Go", 100),
        ],
        total: sample_totals(900),
        with_files: true,
        children: ChildrenMode::Collapse,
        top: 0,
    };
    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();
    assert_eq!(json1, json2);
}

// -- 6. BTreeMap ordering in ModuleReport --

#[test]
fn module_report_serialization_order_is_stable() {
    let report = ModuleReport {
        rows: vec![
            sample_module_row("crates/tokmd", 800),
            sample_module_row("src", 200),
            sample_module_row("tests", 50),
        ],
        total: sample_totals(1050),
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
        top: 0,
    };
    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();
    assert_eq!(json1, json2);
}

// -- 7. ExportData serialization order --

#[test]
fn export_data_serialization_order_is_stable() {
    let data = ExportData {
        rows: vec![
            sample_file_row("src/main.rs", "Rust", 120),
            sample_file_row("src/lib.rs", "Rust", 80),
            sample_file_row("tests/test.py", "Python", 40),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let json1 = serde_json::to_string(&data).unwrap();
    let json2 = serde_json::to_string(&data).unwrap();
    assert_eq!(json1, json2);
}

// -- 8. DiffRow serialization stability --

#[test]
fn diff_row_serialize_roundtrip_is_stable() {
    let row = sample_diff_row("Rust", 100, 200);
    let json1 = serde_json::to_string(&row).unwrap();
    let deserialized: DiffRow = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json1, json2);
}

// -- 9. DiffTotals default serialization stability --

#[test]
fn diff_totals_default_serialize_roundtrip_is_stable() {
    let t = DiffTotals::default();
    let json1 = serde_json::to_string(&t).unwrap();
    let deserialized: DiffTotals = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json1, json2);
}

// -- 10. DiffReceipt serialization stability --

#[test]
fn diff_receipt_serialization_is_deterministic() {
    let receipt = DiffReceipt {
        schema_version: SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "1.0.0".into(),
        },
        mode: "diff".into(),
        from_source: "v1.0".into(),
        to_source: "v2.0".into(),
        diff_rows: vec![
            sample_diff_row("Go", 50, 80),
            sample_diff_row("Rust", 100, 200),
        ],
        totals: DiffTotals::default(),
    };
    let json1 = serde_json::to_string(&receipt).unwrap();
    let json2 = serde_json::to_string(&receipt).unwrap();
    assert_eq!(json1, json2);
}

// -- 11. Enum serialization stability --

#[test]
fn enum_serialization_is_stable() {
    let json_collapse = serde_json::to_string(&ChildrenMode::Collapse).unwrap();
    let json_separate = serde_json::to_string(&ChildrenMode::Separate).unwrap();
    assert_eq!(json_collapse, "\"collapse\"");
    assert_eq!(json_separate, "\"separate\"");
    let rt: ChildrenMode = serde_json::from_str(&json_collapse).unwrap();
    assert_eq!(serde_json::to_string(&rt).unwrap(), json_collapse);
}

#[test]
fn child_include_mode_serialization_is_stable() {
    let json_sep = serde_json::to_string(&ChildIncludeMode::Separate).unwrap();
    let json_po = serde_json::to_string(&ChildIncludeMode::ParentsOnly).unwrap();
    assert_eq!(json_sep, "\"separate\"");
    assert_eq!(json_po, "\"parents-only\"");
}

#[test]
fn file_kind_serialization_is_stable() {
    let json_parent = serde_json::to_string(&FileKind::Parent).unwrap();
    let json_child = serde_json::to_string(&FileKind::Child).unwrap();
    assert_eq!(json_parent, "\"parent\"");
    assert_eq!(json_child, "\"child\"");
}

#[test]
fn scan_status_serialization_is_stable() {
    assert_eq!(
        serde_json::to_string(&ScanStatus::Complete).unwrap(),
        "\"complete\""
    );
    assert_eq!(
        serde_json::to_string(&ScanStatus::Partial).unwrap(),
        "\"partial\""
    );
}

// -- 12. Schema version constant stability --

#[test]
fn schema_version_constants_are_pinned() {
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
}

// -- 13. TokenEstimationMeta determinism --

#[test]
fn token_estimation_is_deterministic_for_same_input() {
    let est1 = TokenEstimationMeta::from_bytes(40000, 4.0);
    let est2 = TokenEstimationMeta::from_bytes(40000, 4.0);
    let json1 = serde_json::to_string(&est1).unwrap();
    let json2 = serde_json::to_string(&est2).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn token_estimation_invariant_min_le_est_le_max() {
    let est = TokenEstimationMeta::from_bytes(12345, 4.0);
    assert!(est.tokens_min <= est.tokens_est);
    assert!(est.tokens_est <= est.tokens_max);
}

// -- 14. TokenAudit determinism --

#[test]
fn token_audit_is_deterministic_for_same_input() {
    let a1 = TokenAudit::from_output(5000, 4500);
    let a2 = TokenAudit::from_output(5000, 4500);
    let json1 = serde_json::to_string(&a1).unwrap();
    let json2 = serde_json::to_string(&a2).unwrap();
    assert_eq!(json1, json2);
}

// -- Property tests --

proptest! {
    #[test]
    fn prop_lang_row_roundtrip(
        code in 0usize..100_000,
        lines in 0usize..200_000,
        files in 0usize..1000,
    ) {
        let row = LangRow {
            lang: "TestLang".into(),
            code,
            lines,
            files,
            bytes: code * 4,
            tokens: code,
            avg_lines: if files > 0 { lines / files } else { 0 },
        };
        let json1 = serde_json::to_string(&row).unwrap();
        let rt: LangRow = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&rt).unwrap();
        prop_assert_eq!(json1, json2);
    }

    #[test]
    fn prop_lang_row_sort_deterministic(
        a_code in 0usize..10_000,
        b_code in 0usize..10_000,
        c_code in 0usize..10_000,
    ) {
        let mut rows_fwd = vec![
            sample_lang_row("Alpha", a_code),
            sample_lang_row("Beta", b_code),
            sample_lang_row("Gamma", c_code),
        ];
        let mut rows_rev = rows_fwd.clone();
        rows_rev.reverse();
        let sort_fn = |a: &LangRow, b: &LangRow| {
            b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang))
        };
        rows_fwd.sort_by(sort_fn);
        rows_rev.sort_by(sort_fn);
        let json_fwd = serde_json::to_string(&rows_fwd).unwrap();
        let json_rev = serde_json::to_string(&rows_rev).unwrap();
        prop_assert_eq!(json_fwd, json_rev);
    }

    #[test]
    fn prop_module_row_sort_deterministic(
        a_code in 0usize..10_000,
        b_code in 0usize..10_000,
    ) {
        let mut rows1 = vec![
            sample_module_row("alpha", a_code),
            sample_module_row("beta", b_code),
        ];
        let mut rows2 = vec![
            sample_module_row("beta", b_code),
            sample_module_row("alpha", a_code),
        ];
        let sort_fn = |a: &ModuleRow, b: &ModuleRow| {
            b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module))
        };
        rows1.sort_by(sort_fn);
        rows2.sort_by(sort_fn);
        let json1 = serde_json::to_string(&rows1).unwrap();
        let json2 = serde_json::to_string(&rows2).unwrap();
        prop_assert_eq!(json1, json2);
    }

    #[test]
    fn prop_file_row_sort_deterministic(
        a_code in 0usize..10_000,
        b_code in 0usize..10_000,
    ) {
        let mut rows1 = vec![
            sample_file_row("src/a.rs", "Rust", a_code),
            sample_file_row("src/b.rs", "Rust", b_code),
        ];
        let mut rows2 = rows1.clone();
        rows2.reverse();
        let sort_fn = |a: &FileRow, b: &FileRow| {
            b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path))
        };
        rows1.sort_by(sort_fn);
        rows2.sort_by(sort_fn);
        let json1 = serde_json::to_string(&rows1).unwrap();
        let json2 = serde_json::to_string(&rows2).unwrap();
        prop_assert_eq!(json1, json2);
    }

    #[test]
    fn prop_totals_roundtrip(
        code in 0usize..100_000,
        lines in 0usize..200_000,
        files in 1usize..1000,
    ) {
        let t = Totals {
            code,
            lines,
            files,
            bytes: code * 4,
            tokens: code,
            avg_lines: lines / files,
        };
        let json1 = serde_json::to_string(&t).unwrap();
        let rt: Totals = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&rt).unwrap();
        prop_assert_eq!(json1, json2);
    }

    #[test]
    fn prop_diff_row_roundtrip(
        old_code in 0usize..50_000,
        new_code in 0usize..50_000,
    ) {
        let row = sample_diff_row("PropLang", old_code, new_code);
        let json1 = serde_json::to_string(&row).unwrap();
        let rt: DiffRow = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&rt).unwrap();
        prop_assert_eq!(json1, json2);
    }
}
