//! Wave 42 property-based tests for tokmd-model.
//!
//! Covers LangRow/ModuleRow aggregation invariants, sorting stability,
//! avg() corner cases, normalize_path properties, and module_key computation.

use proptest::prelude::*;
use std::path::Path;
use tokmd_model::{avg, module_key, normalize_path};
use tokmd_types::{LangRow, ModuleRow};

// ============================================================================
// Strategies
// ============================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[A-Z][a-z]{2,10}",
        0usize..50_000,    // code
        0usize..50_000,    // comments (used to derive lines)
        0usize..50_000,    // blanks   (used to derive lines)
        0usize..500,       // files
        0usize..5_000_000, // bytes
        0usize..500_000,   // tokens
    )
        .prop_map(|(lang, code, comments, blanks, files, bytes, tokens)| {
            let lines = code + comments + blanks;
            let avg_lines = avg(lines, files);
            LangRow {
                lang,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            }
        })
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        "[a-z][a-z0-9_/]{1,15}",
        0usize..50_000,
        0usize..50_000,
        0usize..50_000,
        0usize..500,
        0usize..5_000_000,
        0usize..500_000,
    )
        .prop_map(|(module, code, comments, blanks, files, bytes, tokens)| {
            let lines = code + comments + blanks;
            let avg_lines = avg(lines, files);
            ModuleRow {
                module,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines,
            }
        })
}

// ============================================================================
// avg() properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn avg_zero_files_returns_zero(lines in 0usize..100_000) {
        prop_assert_eq!(avg(lines, 0), 0);
    }

    #[test]
    fn avg_zero_lines_returns_zero(files in 1usize..10_000) {
        prop_assert_eq!(avg(0, files), 0);
    }

    #[test]
    fn avg_result_bounded(lines in 0usize..100_000, files in 1usize..10_000) {
        let r = avg(lines, files);
        let lo = lines / files;
        let hi = if lines % files == 0 { lo } else { lo + 1 };
        prop_assert!((lo..=hi).contains(&r),
            "avg({},{})={} not in [{},{}]", lines, files, r, lo, hi);
    }

    #[test]
    fn avg_exact_division(q in 1usize..5_000, files in 1usize..5_000) {
        prop_assert_eq!(avg(q * files, files), q);
    }
}

// ============================================================================
// LangRow aggregation invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// lines >= code always holds for well-formed rows.
    #[test]
    fn lang_row_lines_ge_code(row in arb_lang_row()) {
        prop_assert!(row.lines >= row.code,
            "lines ({}) must be >= code ({})", row.lines, row.code);
    }

    /// Summing a collection of LangRows preserves total code.
    #[test]
    fn lang_row_sum_is_additive(rows in prop::collection::vec(arb_lang_row(), 0..20)) {
        let sum_code: usize = rows.iter().map(|r| r.code).sum();
        let sum_lines: usize = rows.iter().map(|r| r.lines).sum();
        let sum_files: usize = rows.iter().map(|r| r.files).sum();
        let sum_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let sum_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        // Re-summing must yield the same result (addition is associative)
        prop_assert_eq!(sum_code, rows.iter().map(|r| r.code).sum::<usize>());
        prop_assert_eq!(sum_lines, rows.iter().map(|r| r.lines).sum::<usize>());
        prop_assert_eq!(sum_files, rows.iter().map(|r| r.files).sum::<usize>());
        prop_assert_eq!(sum_bytes, rows.iter().map(|r| r.bytes).sum::<usize>());
        prop_assert_eq!(sum_tokens, rows.iter().map(|r| r.tokens).sum::<usize>());
    }

    /// Sorting LangRows by (code desc, lang asc) is stable and idempotent.
    #[test]
    fn lang_row_sort_idempotent(rows in prop::collection::vec(arb_lang_row(), 0..30)) {
        let sort_fn = |v: &mut Vec<LangRow>| {
            v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        };
        let mut once = rows.clone();
        sort_fn(&mut once);
        let mut twice = once.clone();
        sort_fn(&mut twice);
        prop_assert_eq!(once, twice);
    }

    /// Sorting preserves the number of rows.
    #[test]
    fn lang_row_sort_preserves_count(rows in prop::collection::vec(arb_lang_row(), 0..30)) {
        let mut sorted = rows.clone();
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        prop_assert_eq!(sorted.len(), rows.len());
    }

    /// Sorted LangRows are in descending code order.
    #[test]
    fn lang_row_sorted_descending(rows in prop::collection::vec(arb_lang_row(), 2..20)) {
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        for w in sorted.windows(2) {
            prop_assert!(
                w[0].code > w[1].code || (w[0].code == w[1].code && w[0].lang <= w[1].lang),
                "Sort order violated: {:?} before {:?}", w[0], w[1]
            );
        }
    }
}

// ============================================================================
// ModuleRow aggregation invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// lines >= code for well-formed ModuleRows.
    #[test]
    fn module_row_lines_ge_code(row in arb_module_row()) {
        prop_assert!(row.lines >= row.code,
            "lines ({}) must be >= code ({})", row.lines, row.code);
    }

    /// Sorting ModuleRows by (code desc, module asc) is idempotent.
    #[test]
    fn module_row_sort_idempotent(rows in prop::collection::vec(arb_module_row(), 0..30)) {
        let sort_fn = |v: &mut Vec<ModuleRow>| {
            v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        };
        let mut once = rows.clone();
        sort_fn(&mut once);
        let mut twice = once.clone();
        sort_fn(&mut twice);
        prop_assert_eq!(once, twice);
    }

    /// Sorted ModuleRows are in descending code order.
    #[test]
    fn module_row_sorted_descending(rows in prop::collection::vec(arb_module_row(), 2..20)) {
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        for w in sorted.windows(2) {
            prop_assert!(
                w[0].code > w[1].code || (w[0].code == w[1].code && w[0].module <= w[1].module),
                "Sort order violated: {:?} before {:?}", w[0], w[1]
            );
        }
    }

    /// Sorting preserves sum of code across all rows.
    #[test]
    fn module_row_sort_preserves_code_sum(rows in prop::collection::vec(arb_module_row(), 0..30)) {
        let sum_before: usize = rows.iter().map(|r| r.code).sum();
        let mut sorted = rows;
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        let sum_after: usize = sorted.iter().map(|r| r.code).sum();
        prop_assert_eq!(sum_before, sum_after);
    }
}

// ============================================================================
// normalize_path properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// Normalized paths never contain backslashes.
    #[test]
    fn normalize_no_backslash(s in "[a-zA-Z0-9_./\\\\]{1,40}") {
        let p = Path::new(&s);
        let norm = normalize_path(p, None);
        prop_assert!(!norm.contains('\\'), "Backslash in: {}", norm);
    }

    /// Normalized paths never start with "./".
    #[test]
    fn normalize_no_leading_dot_slash(s in "[a-zA-Z0-9_./]{1,30}") {
        let p = Path::new(&s);
        let norm = normalize_path(p, None);
        prop_assert!(!norm.starts_with("./"), "Leading ./ in: {}", norm);
    }

    /// Normalization is idempotent.
    #[test]
    fn normalize_idempotent(s in "[a-zA-Z0-9_/]{1,30}") {
        let p = Path::new(&s);
        let n1 = normalize_path(p, None);
        let n2 = normalize_path(Path::new(&n1), None);
        prop_assert_eq!(n1, n2);
    }
}

// ============================================================================
// module_key properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// module_key never panics on arbitrary input.
    #[test]
    fn module_key_never_panics(
        path in "[a-zA-Z0-9_/.]{1,40}",
        roots in prop::collection::vec("[a-zA-Z0-9_]+", 0..4),
        depth in 0usize..8
    ) {
        let _ = module_key(&path, &roots, depth);
    }

    /// module_key is deterministic.
    #[test]
    fn module_key_deterministic(
        path in "[a-zA-Z0-9_/]+\\.[a-z]+",
        roots in prop::collection::vec("[a-zA-Z0-9_]+", 0..3),
        depth in 1usize..5
    ) {
        let k1 = module_key(&path, &roots, depth);
        let k2 = module_key(&path, &roots, depth);
        prop_assert_eq!(k1, k2);
    }

    /// Root-level files always map to "(root)".
    #[test]
    fn module_key_root_file(filename in "[a-zA-Z0-9_]+\\.[a-z]+") {
        let k = module_key(&filename, &[], 2);
        prop_assert_eq!(k.clone(), "(root)", "Single file '{}' should be (root), got '{}'", filename, k);
    }

    /// module_key output never contains backslashes.
    #[test]
    fn module_key_no_backslash(
        path in "[a-zA-Z0-9_/\\\\]+\\.[a-z]+",
        roots in prop::collection::vec("[a-zA-Z0-9_]+", 0..3),
        depth in 1usize..5
    ) {
        let k = module_key(&path, &roots, depth);
        prop_assert!(!k.contains('\\'), "Module key contains backslash: {}", k);
    }
}
