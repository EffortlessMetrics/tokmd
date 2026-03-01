//! BDD-style scenario tests for derived metrics (density, distribution, COCOMO, context window).

use tokmd_analysis_derived::derive_report;
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

// ── Empty input ─────────────────────────────────────────────────

mod empty_input {
    use super::*;

    #[test]
    fn given_no_files_when_derived_then_totals_are_zero() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.totals.files, 0);
        assert_eq!(report.totals.code, 0);
        assert_eq!(report.totals.comments, 0);
        assert_eq!(report.totals.blanks, 0);
        assert_eq!(report.totals.lines, 0);
        assert_eq!(report.totals.bytes, 0);
        assert_eq!(report.totals.tokens, 0);
    }

    #[test]
    fn given_no_files_when_derived_then_cocomo_is_none() {
        let report = derive_report(&export(vec![]), None);
        assert!(report.cocomo.is_none());
    }

    #[test]
    fn given_no_files_when_derived_then_distribution_is_zeroed() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.distribution.count, 0);
        assert_eq!(report.distribution.min, 0);
        assert_eq!(report.distribution.max, 0);
        assert_eq!(report.distribution.mean, 0.0);
        assert_eq!(report.distribution.median, 0.0);
        assert_eq!(report.distribution.gini, 0.0);
    }

    #[test]
    fn given_no_files_when_derived_then_context_window_is_none() {
        let report = derive_report(&export(vec![]), None);
        assert!(report.context_window.is_none());
    }

    #[test]
    fn given_no_files_when_derived_then_reading_time_is_zero() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.reading_time.minutes, 0.0);
        assert_eq!(report.reading_time.basis_lines, 0);
    }
}

// ── Density (doc_density) ───────────────────────────────────────

mod density {
    use super::*;

    #[test]
    fn given_well_documented_file_when_derived_then_doc_density_is_high() {
        let rows = vec![make_row("src/lib.rs", "src", "Rust", 50, 50, 10, 3000, 600)];
        let report = derive_report(&export(rows), None);
        // 50 comments / (50 code + 50 comments) = 0.5
        assert_eq!(report.doc_density.total.ratio, 0.5);
        assert_eq!(report.doc_density.total.numerator, 50);
        assert_eq!(report.doc_density.total.denominator, 100);
    }

    #[test]
    fn given_no_comments_when_derived_then_doc_density_is_zero() {
        let rows = vec![make_row(
            "src/main.rs",
            "src",
            "Rust",
            100,
            0,
            5,
            5000,
            1000,
        )];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.total.ratio, 0.0);
    }

    #[test]
    fn given_only_comments_when_derived_then_doc_density_is_one() {
        let rows = vec![make_row("src/doc.rs", "src", "Rust", 0, 80, 0, 2400, 480)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.total.ratio, 1.0);
    }

    #[test]
    fn given_multi_lang_files_when_derived_then_density_has_by_lang_breakdown() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 80, 20, 10, 4000, 800),
            make_row("src/app.ts", "src", "TypeScript", 60, 40, 5, 3000, 600),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.by_lang.len(), 2);
        // TypeScript has higher doc ratio: 40/(60+40)=0.4 vs Rust 20/(80+20)=0.2
        assert_eq!(report.doc_density.by_lang[0].key, "TypeScript");
        assert_eq!(report.doc_density.by_lang[1].key, "Rust");
    }

    #[test]
    fn given_multi_module_files_when_derived_then_density_has_by_module_breakdown() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 80, 20, 5, 4000, 800),
            make_row("lib/b.rs", "lib", "Rust", 40, 60, 5, 3000, 600),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.by_module.len(), 2);
        // lib has 60/(40+60)=0.6 vs src 20/(80+20)=0.2
        assert_eq!(report.doc_density.by_module[0].key, "lib");
    }
}

// ── Whitespace ratio ────────────────────────────────────────────

mod whitespace {
    use super::*;

    #[test]
    fn given_many_blanks_when_derived_then_whitespace_ratio_is_high() {
        let rows = vec![make_row(
            "src/main.rs",
            "src",
            "Rust",
            20,
            10,
            70,
            2000,
            400,
        )];
        let report = derive_report(&export(rows), None);
        // blanks=70, code+comments=30 → 70/30 ≈ 2.3333
        assert!(report.whitespace.total.ratio > 2.0);
    }

    #[test]
    fn given_no_blanks_when_derived_then_whitespace_ratio_is_zero() {
        let rows = vec![make_row(
            "src/dense.rs",
            "src",
            "Rust",
            100,
            50,
            0,
            5000,
            1000,
        )];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.whitespace.total.ratio, 0.0);
    }
}

// ── Distribution ────────────────────────────────────────────────

mod distribution {
    use super::*;

    #[test]
    fn given_single_file_when_derived_then_min_equals_max_equals_median() {
        let rows = vec![make_row(
            "src/only.rs",
            "src",
            "Rust",
            100,
            20,
            10,
            5000,
            1000,
        )];
        let report = derive_report(&export(rows), None);
        let d = &report.distribution;
        assert_eq!(d.count, 1);
        assert_eq!(d.min, 130); // lines = code + comments + blanks
        assert_eq!(d.max, 130);
        assert_eq!(d.median, 130.0);
        assert_eq!(d.mean, 130.0);
    }

    #[test]
    fn given_two_files_when_derived_then_median_is_average_of_sizes() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.rs", "src", "Rust", 200, 0, 0, 8000, 1600),
        ];
        let report = derive_report(&export(rows), None);
        let d = &report.distribution;
        assert_eq!(d.count, 2);
        assert_eq!(d.min, 100);
        assert_eq!(d.max, 200);
        assert_eq!(d.median, 150.0); // (100 + 200) / 2
    }

    #[test]
    fn given_uniform_file_sizes_when_derived_then_gini_is_zero() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("c.rs", "src", "Rust", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert!(report.distribution.gini < 0.01);
    }

    #[test]
    fn given_skewed_file_sizes_when_derived_then_gini_is_positive() {
        let rows = vec![
            make_row("tiny.rs", "src", "Rust", 1, 0, 0, 40, 8),
            make_row("small.rs", "src", "Rust", 5, 0, 0, 200, 40),
            make_row("huge.rs", "src", "Rust", 1000, 0, 0, 40000, 8000),
        ];
        let report = derive_report(&export(rows), None);
        assert!(report.distribution.gini > 0.3);
    }

    #[test]
    fn given_files_when_derived_then_p90_gte_median() {
        let rows: Vec<FileRow> = (0..20)
            .map(|i| {
                make_row(
                    &format!("f{i}.rs"),
                    "src",
                    "Rust",
                    (i + 1) * 10,
                    0,
                    0,
                    400,
                    80,
                )
            })
            .collect();
        let report = derive_report(&export(rows), None);
        assert!(report.distribution.p90 >= report.distribution.median);
        assert!(report.distribution.p99 >= report.distribution.p90);
    }
}

// ── COCOMO ──────────────────────────────────────────────────────

mod cocomo {
    use super::*;

    #[test]
    fn given_zero_code_when_derived_then_cocomo_is_none() {
        let rows = vec![make_row("empty.rs", "src", "Rust", 0, 10, 5, 500, 100)];
        let report = derive_report(&export(rows), None);
        assert!(report.cocomo.is_none());
    }

    #[test]
    fn given_code_when_derived_then_cocomo_uses_organic_mode() {
        let rows = vec![make_row(
            "lib.rs", "src", "Rust", 1000, 200, 50, 40000, 8000,
        )];
        let report = derive_report(&export(rows), None);
        let cocomo = report.cocomo.as_ref().unwrap();
        assert_eq!(cocomo.mode, "organic");
    }

    #[test]
    fn given_1000_loc_when_derived_then_kloc_is_1() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 1000, 0, 0, 40000, 8000)];
        let report = derive_report(&export(rows), None);
        let cocomo = report.cocomo.as_ref().unwrap();
        assert_eq!(cocomo.kloc, 1.0);
    }

    #[test]
    fn given_code_when_derived_then_cocomo_coefficients_are_standard() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 5000, 0, 0, 200000, 40000)];
        let report = derive_report(&export(rows), None);
        let cocomo = report.cocomo.as_ref().unwrap();
        assert_eq!(cocomo.a, 2.4);
        assert_eq!(cocomo.b, 1.05);
        assert_eq!(cocomo.c, 2.5);
        assert_eq!(cocomo.d, 0.38);
    }

    #[test]
    fn given_code_when_derived_then_effort_and_duration_are_positive() {
        let rows = vec![make_row(
            "lib.rs", "src", "Rust", 10000, 0, 0, 400000, 80000,
        )];
        let report = derive_report(&export(rows), None);
        let cocomo = report.cocomo.as_ref().unwrap();
        assert!(cocomo.effort_pm > 0.0);
        assert!(cocomo.duration_months > 0.0);
        assert!(cocomo.staff > 0.0);
    }

    #[test]
    fn given_more_code_when_derived_then_effort_increases() {
        let small = vec![make_row("s.rs", "src", "Rust", 1000, 0, 0, 40000, 8000)];
        let large = vec![make_row("l.rs", "src", "Rust", 10000, 0, 0, 400000, 80000)];
        let r_small = derive_report(&export(small), None);
        let r_large = derive_report(&export(large), None);
        assert!(
            r_large.cocomo.as_ref().unwrap().effort_pm > r_small.cocomo.as_ref().unwrap().effort_pm
        );
    }
}

// ── Context window ──────────────────────────────────────────────

mod context_window {
    use super::*;

    #[test]
    fn given_no_window_when_derived_then_context_window_is_none() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let report = derive_report(&export(rows), None);
        assert!(report.context_window.is_none());
    }

    #[test]
    fn given_tokens_within_budget_when_derived_then_fits_is_true() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 100, 10, 5, 4000, 500)];
        let report = derive_report(&export(rows), Some(128_000));
        let cw = report.context_window.as_ref().unwrap();
        assert!(cw.fits);
        assert_eq!(cw.total_tokens, 500);
        assert_eq!(cw.window_tokens, 128_000);
        assert!(cw.pct < 1.0);
    }

    #[test]
    fn given_tokens_exceeding_budget_when_derived_then_fits_is_false() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 100, 0, 0, 4000, 200_000)];
        let report = derive_report(&export(rows), Some(128_000));
        let cw = report.context_window.as_ref().unwrap();
        assert!(!cw.fits);
        assert!(cw.pct > 1.0);
    }

    #[test]
    fn given_exact_fit_when_derived_then_fits_is_true_and_pct_is_one() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 100, 0, 0, 4000, 1000)];
        let report = derive_report(&export(rows), Some(1000));
        let cw = report.context_window.as_ref().unwrap();
        assert!(cw.fits);
        assert_eq!(cw.pct, 1.0);
    }

    #[test]
    fn given_zero_window_when_derived_then_pct_is_zero() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 100, 0, 0, 4000, 500)];
        let report = derive_report(&export(rows), Some(0));
        let cw = report.context_window.as_ref().unwrap();
        assert_eq!(cw.pct, 0.0);
    }

    #[test]
    fn given_multi_file_tokens_when_derived_then_total_is_summed() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 300),
            make_row("b.rs", "src", "Rust", 50, 0, 0, 2000, 700),
        ];
        let report = derive_report(&export(rows), Some(128_000));
        let cw = report.context_window.as_ref().unwrap();
        assert_eq!(cw.total_tokens, 1000);
    }
}

// ── Totals aggregation ──────────────────────────────────────────

mod totals {
    use super::*;

    #[test]
    fn given_multiple_files_when_derived_then_totals_are_summed() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 20, 10, 5000, 1000),
            make_row("b.rs", "src", "Rust", 200, 30, 20, 8000, 1500),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.totals.files, 2);
        assert_eq!(report.totals.code, 300);
        assert_eq!(report.totals.comments, 50);
        assert_eq!(report.totals.blanks, 30);
        assert_eq!(report.totals.lines, 380);
        assert_eq!(report.totals.bytes, 13000);
        assert_eq!(report.totals.tokens, 2500);
    }

    #[test]
    fn given_child_rows_when_derived_then_children_are_excluded_from_totals() {
        let mut rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 5000, 1000)];
        rows.push(FileRow {
            path: "a.rs::embedded".to_string(),
            module: "src".to_string(),
            lang: "HTML".to_string(),
            kind: FileKind::Child,
            code: 50,
            comments: 5,
            blanks: 2,
            lines: 57,
            bytes: 2000,
            tokens: 500,
        });
        let report = derive_report(&export(rows), None);
        assert_eq!(report.totals.files, 1);
        assert_eq!(report.totals.code, 100);
    }
}

// ── Histogram ───────────────────────────────────────────────────

mod histogram {
    use super::*;

    #[test]
    fn given_empty_input_when_derived_then_histogram_has_five_zero_buckets() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.histogram.len(), 5);
        for bucket in &report.histogram {
            assert_eq!(bucket.files, 0);
            assert_eq!(bucket.pct, 0.0);
        }
    }

    #[test]
    fn given_tiny_files_when_derived_then_all_in_tiny_bucket() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 10, 0, 0, 400, 80),
            make_row("b.rs", "src", "Rust", 20, 0, 0, 800, 160),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.histogram[0].label, "Tiny");
        assert_eq!(report.histogram[0].files, 2);
        assert_eq!(report.histogram[0].pct, 1.0);
    }

    #[test]
    fn given_huge_file_when_derived_then_in_huge_bucket() {
        let rows = vec![make_row("big.rs", "src", "Rust", 2000, 0, 0, 80000, 16000)];
        let report = derive_report(&export(rows), None);
        let huge = report.histogram.iter().find(|b| b.label == "Huge").unwrap();
        assert_eq!(huge.files, 1);
    }

    #[test]
    fn given_files_when_derived_then_histogram_pcts_sum_to_one() {
        let rows: Vec<FileRow> = (0..10)
            .map(|i| {
                make_row(
                    &format!("f{i}.rs"),
                    "src",
                    "Rust",
                    (i + 1) * 100,
                    0,
                    0,
                    (i + 1) * 4000,
                    (i + 1) * 800,
                )
            })
            .collect();
        let report = derive_report(&export(rows), None);
        let total_pct: f64 = report.histogram.iter().map(|b| b.pct).sum();
        assert!((total_pct - 1.0).abs() < 0.01);
    }
}

// ── Reading time ────────────────────────────────────────────────

mod reading_time {
    use super::*;

    #[test]
    fn given_200_code_lines_when_derived_then_reading_time_is_10_minutes() {
        let rows = vec![make_row("lib.rs", "src", "Rust", 200, 0, 0, 8000, 1600)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.reading_time.minutes, 10.0);
        assert_eq!(report.reading_time.lines_per_minute, 20);
        assert_eq!(report.reading_time.basis_lines, 200);
    }
}

// ── Polyglot ────────────────────────────────────────────────────

mod polyglot {
    use super::*;

    #[test]
    fn given_single_language_when_derived_then_entropy_is_zero() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.rs", "src", "Rust", 200, 0, 0, 8000, 1600),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.polyglot.lang_count, 1);
        assert_eq!(report.polyglot.entropy, 0.0);
        assert_eq!(report.polyglot.dominant_lang, "Rust");
        assert_eq!(report.polyglot.dominant_pct, 1.0);
    }

    #[test]
    fn given_two_equal_languages_when_derived_then_entropy_is_one() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.py", "src", "Python", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.polyglot.lang_count, 2);
        assert_eq!(report.polyglot.entropy, 1.0);
    }
}

// ── Test density ────────────────────────────────────────────────

mod test_density {
    use super::*;

    #[test]
    fn given_test_and_prod_files_when_derived_then_ratio_reflects_test_proportion() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 200, 0, 0, 8000, 1600),
            make_row("tests/test_lib.rs", "tests", "Rust", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.test_density.prod_lines, 200);
        assert_eq!(report.test_density.test_lines, 100);
        assert_eq!(report.test_density.prod_files, 1);
        assert_eq!(report.test_density.test_files, 1);
    }

    #[test]
    fn given_no_tests_when_derived_then_test_ratio_is_zero() {
        let rows = vec![make_row(
            "src/main.rs",
            "src",
            "Rust",
            300,
            0,
            0,
            12000,
            2400,
        )];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.test_density.ratio, 0.0);
        assert_eq!(report.test_density.test_files, 0);
    }
}

// ── Integrity ───────────────────────────────────────────────────

mod integrity {
    use super::*;

    #[test]
    fn given_files_when_derived_then_integrity_uses_blake3() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.integrity.algo, "blake3");
        assert_eq!(report.integrity.entries, 1);
        assert!(!report.integrity.hash.is_empty());
    }

    #[test]
    fn given_same_files_when_derived_twice_then_hash_is_deterministic() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.rs", "src", "Rust", 200, 0, 0, 8000, 1600),
        ];
        let r1 = derive_report(&export(rows.clone()), None);
        let r2 = derive_report(&export(rows), None);
        assert_eq!(r1.integrity.hash, r2.integrity.hash);
    }

    #[test]
    fn given_different_files_when_derived_then_hashes_differ() {
        let rows_a = vec![make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let rows_b = vec![make_row("b.rs", "src", "Rust", 200, 0, 0, 8000, 1600)];
        let r1 = derive_report(&export(rows_a), None);
        let r2 = derive_report(&export(rows_b), None);
        assert_ne!(r1.integrity.hash, r2.integrity.hash);
    }
}

// ── Nesting ─────────────────────────────────────────────────────

mod nesting {
    use super::*;

    #[test]
    fn given_deep_path_when_derived_then_nesting_reflects_depth() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("a/b/c/d.rs", "src", "Rust", 50, 0, 0, 2000, 400),
        ];
        let report = derive_report(&export(rows), None);
        assert!(report.nesting.max >= 3);
    }

    #[test]
    fn given_flat_files_when_derived_then_nesting_max_is_one() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.rs", "src", "Rust", 50, 0, 0, 2000, 400),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.nesting.max, 1);
    }
}
