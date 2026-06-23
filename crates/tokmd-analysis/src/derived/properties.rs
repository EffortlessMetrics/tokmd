//! Property-based tests for derived metrics and integrity.

use super::integrity::{build_integrity_report, compare_integrity_rows};
use proptest::prelude::*;
use tokmd_types::{FileKind, FileRow};

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-zA-Z0-9_/\\.]{1,50}",
        0usize..1000000,
        0usize..100000,
    ).prop_map(|(path, bytes, lines)| FileRow {
        path,
        module: "mod".to_string(),
        lang: "rust".to_string(),
        kind: FileKind::Parent,
        code: 0,
        comments: 0,
        blanks: 0,
        lines,
        bytes,
        tokens: 0,
    })
}

proptest! {
    #[test]
    fn prop_compare_integrity_rows_matches_string_sort(
        a in arb_file_row(),
        b in arb_file_row(),
    ) {
        let s1 = format!("{}:{}:{}", a.path, a.bytes, a.lines);
        let s2 = format!("{}:{}:{}", b.path, b.bytes, b.lines);
        let expected = s1.cmp(&s2);
        let actual = compare_integrity_rows(&a, &b);
        prop_assert_eq!(actual, expected, "Failed for {} vs {}", s1, s2);
    }
}
