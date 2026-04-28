use proptest::prelude::*;
use tokmd_types::{DiffRow, DiffTotals};

proptest! {
    #[test]
    fn diff_totals_serialize_roundtrip_stable(
        old_code in 0usize..50_000,
        new_code in 0usize..50_000,
        delta_code in -50_000i64..50_000i64,
        old_lines in 0usize..100_000,
        new_lines in 0usize..100_000,
        delta_lines in -100_000i64..100_000i64,
        old_files in 1usize..5_000,
        new_files in 1usize..5_000,
        delta_files in -5_000i64..5_000i64,
        old_bytes in 0usize..5_000_000,
        new_bytes in 0usize..5_000_000,
        delta_bytes in -5_000_000i64..5_000_000i64,
        old_tokens in 0usize..500_000,
        new_tokens in 0usize..500_000,
        delta_tokens in -500_000i64..500_000i64
    ) {
        let totals = DiffTotals {
            old_code, new_code, delta_code,
            old_lines, new_lines, delta_lines,
            old_files, new_files, delta_files,
            old_bytes, new_bytes, delta_bytes,
            old_tokens, new_tokens, delta_tokens,
        };
        let json1 = serde_json::to_string(&totals).expect("serialize");
        let back: DiffTotals = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "JSON → deserialize → JSON must be stable for DiffTotals");
    }

    #[test]
    fn diff_row_serialize_roundtrip_stable(
        lang in "[A-Za-z][A-Za-z0-9 ]{0,15}",
        old_code in 0usize..50_000,
        new_code in 0usize..50_000,
        delta_code in -50_000i64..50_000i64,
        old_lines in 0usize..100_000,
        new_lines in 0usize..100_000,
        delta_lines in -100_000i64..100_000i64,
        old_files in 1usize..5_000,
        new_files in 1usize..5_000,
        delta_files in -5_000i64..5_000i64,
        old_bytes in 0usize..5_000_000,
        new_bytes in 0usize..5_000_000,
        delta_bytes in -5_000_000i64..5_000_000i64,
        old_tokens in 0usize..500_000,
        new_tokens in 0usize..500_000,
        delta_tokens in -500_000i64..500_000i64
    ) {
        let row = DiffRow {
            lang, old_code, new_code, delta_code,
            old_lines, new_lines, delta_lines,
            old_files, new_files, delta_files,
            old_bytes, new_bytes, delta_bytes,
            old_tokens, new_tokens, delta_tokens,
        };
        let json1 = serde_json::to_string(&row).expect("serialize");
        let back: DiffRow = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "JSON → deserialize → JSON must be stable for DiffRow");
    }
}

use tokmd_types::{FileKind, FileRow, LangRow, ModuleRow};

proptest! {
    #[test]
    fn file_row_serialize_roundtrip_stable(
        path in "[a-z][a-z0-9_/]{1,30}\\.[a-z]{1,4}",
        module in "[a-z][a-z0-9_/]{0,15}",
        lang in "[A-Za-z][A-Za-z0-9 ]{0,10}",
        kind in prop_oneof![Just(FileKind::Parent), Just(FileKind::Child)],
        code in 0usize..50_000,
        comments in 0usize..10_000,
        blanks in 0usize..10_000,
        lines in 0usize..100_000,
        bytes in 0usize..5_000_000,
        tokens in 0usize..500_000
    ) {
        let row = FileRow {
            path, module, lang, kind, code, comments, blanks, lines, bytes, tokens,
        };
        let json1 = serde_json::to_string(&row).expect("serialize");
        let back: FileRow = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "JSON → deserialize → JSON must be stable for FileRow");
    }

    #[test]
    fn lang_row_serialize_roundtrip_stable(
        lang in "[A-Za-z][A-Za-z0-9 ]{0,15}",
        code in 0usize..50_000,
        lines in 0usize..100_000,
        files in 1usize..5_000,
        bytes in 0usize..5_000_000,
        tokens in 0usize..500_000,
        avg_lines in 0usize..500
    ) {
        let row = LangRow {
            lang, code, lines, files, bytes, tokens, avg_lines,
        };
        let json1 = serde_json::to_string(&row).expect("serialize");
        let back: LangRow = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "JSON → deserialize → JSON must be stable for LangRow");
    }

    #[test]
    fn module_row_serialize_roundtrip_stable(
        module in "[a-z][a-z0-9_/]{0,20}",
        code in 0usize..50_000,
        lines in 0usize..100_000,
        files in 1usize..5_000,
        bytes in 0usize..5_000_000,
        tokens in 0usize..500_000,
        avg_lines in 0usize..500
    ) {
        let row = ModuleRow {
            module, code, lines, files, bytes, tokens, avg_lines,
        };
        let json1 = serde_json::to_string(&row).expect("serialize");
        let back: ModuleRow = serde_json::from_str(&json1).expect("deserialize");
        let json2 = serde_json::to_string(&back).expect("re-serialize");
        prop_assert_eq!(&json1, &json2, "JSON → deserialize → JSON must be stable for ModuleRow");
    }
}
