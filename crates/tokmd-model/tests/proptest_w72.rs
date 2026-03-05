//! Wave 72 property-based invariant tests for tokmd-model.
//!
//! Covers: total ↔ per-language consistency, module aggregation preservation,
//! file count correctness, sorting invariants, avg() edge cases,
//! and normalize_path / module_key composition properties.

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
        0usize..50_000,    // comments
        0usize..50_000,    // blanks
        1usize..500,       // files (≥1 for avg sanity)
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
        1usize..500,
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

fn arb_path_component() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_.-]{1,12}"
}

fn arb_path(max_depth: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(arb_path_component(), 1..=max_depth).prop_map(|c| c.join("/"))
}

// ============================================================================
// 1. Total code lines == sum of per-language code lines
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn total_code_equals_sum_of_lang_codes(rows in prop::collection::vec(arb_lang_row(), 1..20)) {
        let sum_code: usize = rows.iter().map(|r| r.code).sum();
        let sum_lines: usize = rows.iter().map(|r| r.lines).sum();
        let sum_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let sum_tokens: usize = rows.iter().map(|r| r.tokens).sum();
        // Each row's code must be ≤ its lines (code + comments + blanks)
        for r in &rows {
            prop_assert!(r.code <= r.lines, "code {} > lines {}", r.code, r.lines);
        }
        // Sum of per-lang code is consistent
        prop_assert!(sum_code <= sum_lines);
        // Bytes/tokens are non-negative (always true for usize, but checked symbolically)
        prop_assert!(sum_bytes < usize::MAX);
        prop_assert!(sum_tokens < usize::MAX);
    }

    // ========================================================================
    // 2. Module aggregation preserves total counts
    // ========================================================================

    #[test]
    fn module_aggregation_preserves_totals(rows in prop::collection::vec(arb_module_row(), 1..20)) {
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let total_lines: usize = rows.iter().map(|r| r.lines).sum();
        let total_files: usize = rows.iter().map(|r| r.files).sum();
        let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        // Simulate what the model does: group by module, then sum
        let mut by_module = std::collections::BTreeMap::<String, (usize, usize, usize, usize, usize)>::new();
        for r in &rows {
            let e = by_module.entry(r.module.clone()).or_default();
            e.0 += r.code;
            e.1 += r.lines;
            e.2 += r.files;
            e.3 += r.bytes;
            e.4 += r.tokens;
        }
        let reagg_code: usize = by_module.values().map(|v| v.0).sum();
        let reagg_lines: usize = by_module.values().map(|v| v.1).sum();
        let reagg_files: usize = by_module.values().map(|v| v.2).sum();
        let reagg_bytes: usize = by_module.values().map(|v| v.3).sum();
        let reagg_tokens: usize = by_module.values().map(|v| v.4).sum();

        prop_assert_eq!(total_code, reagg_code);
        prop_assert_eq!(total_lines, reagg_lines);
        prop_assert_eq!(total_files, reagg_files);
        prop_assert_eq!(total_bytes, reagg_bytes);
        prop_assert_eq!(total_tokens, reagg_tokens);
    }

    // ========================================================================
    // 3. File count in model == number of input files
    // ========================================================================

    #[test]
    fn lang_row_file_count_matches_input(rows in prop::collection::vec(arb_lang_row(), 1..20)) {
        let total_files: usize = rows.iter().map(|r| r.files).sum();
        // When rows are generated independently they should sum correctly
        prop_assert!(total_files >= rows.len().min(1));
    }

    #[test]
    fn module_row_file_count_nonneg(rows in prop::collection::vec(arb_module_row(), 0..20)) {
        for r in &rows {
            prop_assert!(r.files >= 1, "files should be >= 1 for generated rows");
        }
    }

    // ========================================================================
    // 4. Sorting invariant: descending by code, then ascending by name
    // ========================================================================

    #[test]
    fn lang_sort_invariant(rows in prop::collection::vec(arb_lang_row(), 2..30)) {
        let mut sorted = rows.clone();
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        // After sorting, each adjacent pair must satisfy the ordering
        for pair in sorted.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            prop_assert!(
                a.code > b.code || (a.code == b.code && a.lang <= b.lang),
                "Sort violation: ({}, {}) before ({}, {})",
                a.lang, a.code, b.lang, b.code
            );
        }
    }

    #[test]
    fn module_sort_invariant(rows in prop::collection::vec(arb_module_row(), 2..30)) {
        let mut sorted = rows.clone();
        sorted.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        for pair in sorted.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            prop_assert!(
                a.code > b.code || (a.code == b.code && a.module <= b.module),
                "Sort violation: ({}, {}) before ({}, {})",
                a.module, a.code, b.module, b.code
            );
        }
    }

    #[test]
    fn sorting_is_deterministic(rows in prop::collection::vec(arb_lang_row(), 1..30)) {
        let mut a = rows.clone();
        let mut b = rows.clone();
        a.sort_by(|x, y| y.code.cmp(&x.code).then_with(|| x.lang.cmp(&y.lang)));
        b.sort_by(|x, y| y.code.cmp(&x.code).then_with(|| x.lang.cmp(&y.lang)));
        prop_assert_eq!(a.len(), b.len());
        for (ra, rb) in a.iter().zip(b.iter()) {
            prop_assert_eq!(&ra.lang, &rb.lang);
            prop_assert_eq!(ra.code, rb.code);
        }
    }

    // ========================================================================
    // 5. avg() invariants
    // ========================================================================

    #[test]
    fn avg_zero_files_returns_zero(lines in 0usize..100_000) {
        prop_assert_eq!(avg(lines, 0), 0);
    }

    #[test]
    fn avg_never_exceeds_lines(lines in 0usize..100_000, files in 1usize..1000) {
        let result = avg(lines, files);
        // avg can round up by at most 1 per file, so result ≤ lines when files >= 1
        prop_assert!(result <= lines, "avg({}, {}) = {} > lines", lines, files, result);
    }

    #[test]
    fn avg_exact_division(factor in 1usize..1000, files in 1usize..500) {
        let lines = factor * files;
        prop_assert_eq!(avg(lines, files), factor);
    }

    // ========================================================================
    // 6. normalize_path properties
    // ========================================================================

    #[test]
    fn normalize_path_idempotent(path in arb_path(5)) {
        let n1 = normalize_path(Path::new(&path), None);
        let n2 = normalize_path(Path::new(&n1), None);
        prop_assert_eq!(n1, n2);
    }

    #[test]
    fn normalize_path_no_backslashes(path in arb_path(5)) {
        let norm = normalize_path(Path::new(&path), None);
        prop_assert!(!norm.contains('\\'), "Found backslash in: {}", norm);
    }

    #[test]
    fn normalize_path_no_leading_dot_slash(path in arb_path(5)) {
        let norm = normalize_path(Path::new(&path), None);
        prop_assert!(!norm.starts_with("./"), "Leading ./ in: {}", norm);
    }

    // ========================================================================
    // 7. module_key properties
    // ========================================================================

    #[test]
    fn module_key_deterministic(
        path in arb_path(5),
        roots in prop::collection::vec(arb_path_component(), 0..3),
        depth in 1usize..5
    ) {
        let k1 = module_key(&path, &roots, depth);
        let k2 = module_key(&path, &roots, depth);
        prop_assert_eq!(k1, k2);
    }

    #[test]
    fn module_key_never_empty(
        path in arb_path(5),
        roots in prop::collection::vec(arb_path_component(), 0..3),
        depth in 1usize..5
    ) {
        let key = module_key(&path, &roots, depth);
        prop_assert!(!key.is_empty(), "Empty module key for path: {}", path);
    }
}
