//! Edge-case and determinism tests for `analysis derived module`.
//!
//! Supplements the existing BDD, integration, and property tests with
//! scenarios that exercise max_file, lang_purity tie-breaking, and
//! output determinism.

use crate::derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn make_row(
    path: &str,
    module: &str,
    lang: &str,
    code: usize,
    comments: usize,
    blanks: usize,
    bytes: usize,
    tokens: usize,
) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments,
        blanks,
        lines: code + comments + blanks,
        bytes,
        tokens,
    }
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ── Max-file report ─────────────────────────────────────────────

mod max_file {
    use super::*;

    #[test]
    fn given_single_file_when_derived_then_max_file_overall_matches() {
        let rows = vec![make_row(
            "src/only.rs",
            "src",
            "Rust",
            100,
            10,
            5,
            4000,
            800,
        )];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.max_file.overall.path, "src/only.rs");
        assert_eq!(report.max_file.overall.lines, 115);
    }

    #[test]
    fn given_two_files_when_derived_then_overall_max_is_largest_by_lines() {
        let rows = vec![
            make_row("small.rs", "src", "Rust", 10, 0, 0, 400, 80),
            make_row("big.rs", "src", "Rust", 500, 0, 0, 20000, 4000),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.max_file.overall.path, "big.rs");
        assert_eq!(report.max_file.overall.lines, 500);
    }

    #[test]
    fn given_tie_in_lines_when_derived_then_max_file_is_deterministic() {
        let rows = vec![
            make_row("z.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
        ];
        let r1 = derive_report(&export(rows.clone()), None);
        let r2 = derive_report(&export(rows), None);
        // Tie-breaking is deterministic across runs
        assert_eq!(r1.max_file.overall.path, r2.max_file.overall.path);
        assert_eq!(r1.max_file.overall.lines, 100);
    }

    #[test]
    fn given_multi_lang_files_when_derived_then_max_file_has_by_lang_entries() {
        let rows = vec![
            make_row("lib.rs", "src", "Rust", 200, 0, 0, 8000, 1600),
            make_row("app.py", "src", "Python", 300, 0, 0, 12000, 2400),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.max_file.by_lang.len(), 2);
        let rust_entry = report
            .max_file
            .by_lang
            .iter()
            .find(|e| e.key == "Rust")
            .unwrap();
        assert_eq!(rust_entry.file.path, "lib.rs");
    }

    #[test]
    fn given_multi_module_files_when_derived_then_max_file_has_by_module_entries() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("lib/b.rs", "lib", "Rust", 300, 0, 0, 12000, 2400),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.max_file.by_module.len(), 2);
        let lib_entry = report
            .max_file
            .by_module
            .iter()
            .find(|e| e.key == "lib")
            .unwrap();
        assert_eq!(lib_entry.file.path, "lib/b.rs");
    }

    #[test]
    fn given_empty_input_when_derived_then_max_file_overall_is_empty() {
        let report = derive_report(&export(vec![]), None);
        assert!(report.max_file.overall.path.is_empty());
        assert_eq!(report.max_file.overall.lines, 0);
        assert!(report.max_file.by_lang.is_empty());
        assert!(report.max_file.by_module.is_empty());
    }
}

// ── Lang purity tie-breaking ────────────────────────────────────

mod lang_purity_tiebreak {
    use super::*;

    #[test]
    fn given_equal_lang_lines_when_derived_then_dominant_is_alphabetically_first() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("a.py", "src", "Python", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        let purity = &report.lang_purity.rows[0];
        // Equal lines → alphabetically first lang wins
        assert_eq!(purity.dominant_lang, "Python");
        assert_eq!(purity.dominant_pct, 0.5);
        assert_eq!(purity.lang_count, 2);
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism {
    use super::*;

    #[test]
    fn given_same_input_when_derived_twice_then_output_is_identical() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 200, 50, 20, 10000, 2000),
            make_row("src/app.ts", "src", "TypeScript", 150, 30, 10, 7000, 1500),
            make_row("tests/test.rs", "tests", "Rust", 80, 10, 5, 3500, 700),
        ];
        let r1 = derive_report(&export(rows.clone()), Some(128_000));
        let r2 = derive_report(&export(rows), Some(128_000));

        assert_eq!(r1.totals.code, r2.totals.code);
        assert_eq!(r1.totals.files, r2.totals.files);
        assert_eq!(r1.integrity.hash, r2.integrity.hash);
        assert_eq!(r1.distribution.gini, r2.distribution.gini);
        assert_eq!(r1.polyglot.entropy, r2.polyglot.entropy);
        assert_eq!(
            r1.cocomo.as_ref().unwrap().effort_pm,
            r2.cocomo.as_ref().unwrap().effort_pm
        );
    }

    #[test]
    fn given_rows_in_different_order_when_derived_then_integrity_hash_is_same() {
        let row_a = make_row("a.rs", "src", "Rust", 100, 10, 5, 4000, 800);
        let row_b = make_row("b.rs", "src", "Rust", 200, 20, 10, 8000, 1600);

        let r1 = derive_report(&export(vec![row_a.clone(), row_b.clone()]), None);
        let r2 = derive_report(&export(vec![row_b, row_a]), None);

        assert_eq!(
            r1.integrity.hash, r2.integrity.hash,
            "integrity hash must be order-independent"
        );
    }
}

// ── COCOMO edge cases ───────────────────────────────────────────

mod cocomo_edges {
    use super::*;

    #[test]
    fn given_very_small_code_when_derived_then_cocomo_kloc_is_fractional() {
        let rows = vec![make_row("tiny.rs", "src", "Rust", 50, 0, 0, 2000, 400)];
        let report = derive_report(&export(rows), None);
        let cocomo = report.cocomo.as_ref().unwrap();
        assert_eq!(cocomo.kloc, 0.05);
        assert!(cocomo.effort_pm > 0.0);
    }

    #[test]
    fn given_large_codebase_when_derived_then_cocomo_scales_superlinearly() {
        let small = vec![make_row("s.rs", "src", "Rust", 1000, 0, 0, 40000, 8000)];
        let big = vec![make_row(
            "b.rs", "src", "Rust", 100_000, 0, 0, 4_000_000, 800_000,
        )];
        let r_small = derive_report(&export(small), None);
        let r_big = derive_report(&export(big), None);

        let small_effort = r_small.cocomo.as_ref().unwrap().effort_pm;
        let big_effort = r_big.cocomo.as_ref().unwrap().effort_pm;

        // With b=1.05, effort scales superlinearly: 100x code should give >100x effort
        assert!(
            big_effort / small_effort > 100.0,
            "effort should scale superlinearly: small={small_effort}, big={big_effort}"
        );
    }
}
