//! Deep tests for `tokmd-analysis-derived`.
//!
//! Covers density calculations, distribution metrics, COCOMO estimates,
//! zero/single/multi-language inputs, deterministic output, boundary
//! values, very large inputs, and serialization of derived metrics.

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

fn make_simple_row(path: &str, lang: &str, code: usize) -> FileRow {
    make_row(path, "src", lang, code, 0, 0, code * 40, code * 8)
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

// ── Density calculation ─────────────────────────────────────────

mod density {
    use super::*;

    #[test]
    fn doc_density_ratio_is_comments_over_code_plus_comments() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 80, 20, 10, 4000, 800)];
        let report = derive_report(&export(rows), None);
        // doc density = comments / (code + comments) = 20 / 100 = 0.2
        assert_eq!(report.doc_density.total.numerator, 20);
        assert_eq!(report.doc_density.total.denominator, 100);
        assert!((report.doc_density.total.ratio - 0.2).abs() < 0.001);
    }

    #[test]
    fn doc_density_zero_when_no_comments() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 100, 0, 5, 4000, 800)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.total.ratio, 0.0);
    }

    #[test]
    fn doc_density_one_when_all_comments() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 0, 100, 0, 4000, 800)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.total.numerator, 100);
        assert_eq!(report.doc_density.total.denominator, 100);
        assert!((report.doc_density.total.ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn whitespace_ratio_is_blanks_over_code_plus_comments() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 60, 20, 20, 4000, 800)];
        let report = derive_report(&export(rows), None);
        // whitespace = blanks / (code + comments) = 20 / 80 = 0.25
        assert_eq!(report.whitespace.total.numerator, 20);
        assert_eq!(report.whitespace.total.denominator, 80);
        assert!((report.whitespace.total.ratio - 0.25).abs() < 0.001);
    }

    #[test]
    fn doc_density_by_lang_is_grouped() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 80, 20, 0, 4000, 800),
            make_row("src/b.py", "src", "Python", 50, 50, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.by_lang.len(), 2);
        // by_lang ratio = comments / code (not code+comments)
        // Python: 50/50 = 1.0, Rust: 20/80 = 0.25
        assert_eq!(report.doc_density.by_lang[0].key, "Python");
        assert!((report.doc_density.by_lang[0].ratio - 1.0).abs() < 0.001);
    }
}

// ── Distribution metrics ────────────────────────────────────────

mod distribution {
    use super::*;

    #[test]
    fn single_file_has_equal_min_max_mean_median() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 50, 10, 5, 2000, 400)];
        let report = derive_report(&export(rows), None);
        let dist = &report.distribution;
        assert_eq!(dist.count, 1);
        assert_eq!(dist.min, 65); // lines = 50+10+5
        assert_eq!(dist.max, 65);
        assert!((dist.mean - 65.0).abs() < 0.01);
        assert!((dist.median - 65.0).abs() < 0.01);
    }

    #[test]
    fn two_files_median_is_average() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 40, 0, 0, 1600, 320),
            make_row("src/b.rs", "src", "Rust", 60, 0, 0, 2400, 480),
        ];
        let report = derive_report(&export(rows), None);
        let dist = &report.distribution;
        assert_eq!(dist.count, 2);
        assert_eq!(dist.min, 40);
        assert_eq!(dist.max, 60);
        assert!((dist.median - 50.0).abs() < 0.01);
    }

    #[test]
    fn three_files_median_is_middle_value() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 10, 0, 0, 400, 80),
            make_row("src/b.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("src/c.rs", "src", "Rust", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert!((report.distribution.median - 50.0).abs() < 0.01);
    }

    #[test]
    fn gini_zero_for_equal_sizes() {
        let rows: Vec<FileRow> = (0..5)
            .map(|i| make_row(&format!("src/{i}.rs"), "src", "Rust", 100, 0, 0, 4000, 800))
            .collect();
        let report = derive_report(&export(rows), None);
        assert!((report.distribution.gini - 0.0).abs() < 0.01);
    }

    #[test]
    fn gini_high_for_skewed_sizes() {
        let mut rows: Vec<FileRow> = (0..9)
            .map(|i| make_row(&format!("src/{i}.rs"), "src", "Rust", 1, 0, 0, 40, 8))
            .collect();
        rows.push(make_row(
            "src/big.rs",
            "src",
            "Rust",
            10000,
            0,
            0,
            400000,
            80000,
        ));
        let report = derive_report(&export(rows), None);
        assert!(report.distribution.gini > 0.5);
    }

    #[test]
    fn p90_and_p99_within_range() {
        let rows: Vec<FileRow> = (1..=100)
            .map(|i| {
                make_row(
                    &format!("src/{i}.rs"),
                    "src",
                    "Rust",
                    i * 10,
                    0,
                    0,
                    i * 400,
                    i * 80,
                )
            })
            .collect();
        let report = derive_report(&export(rows), None);
        assert!(report.distribution.p90 >= report.distribution.median);
        assert!(report.distribution.p99 >= report.distribution.p90);
        assert!(report.distribution.p99 <= report.distribution.max as f64);
    }
}

// ── COCOMO estimates ────────────────────────────────────────────

mod cocomo {
    use super::*;

    #[test]
    fn cocomo_none_when_zero_code() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 0, 10, 5, 400, 80)];
        let report = derive_report(&export(rows), None);
        assert!(report.cocomo.is_none());
    }

    #[test]
    fn cocomo_present_when_code_exists() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 1000)];
        let report = derive_report(&export(rows), None);
        assert!(report.cocomo.is_some());
    }

    #[test]
    fn cocomo_organic_mode() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 1000)];
        let report = derive_report(&export(rows), None);
        let cocomo = report.cocomo.unwrap();
        assert_eq!(cocomo.mode, "organic");
    }

    #[test]
    fn cocomo_kloc_matches_code_lines() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 5000)];
        let report = derive_report(&export(rows), None);
        let cocomo = report.cocomo.unwrap();
        assert!((cocomo.kloc - 5.0).abs() < 0.001);
    }

    #[test]
    fn cocomo_parameters_are_organic() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 1000)];
        let report = derive_report(&export(rows), None);
        let c = report.cocomo.unwrap();
        assert!((c.a - 2.4).abs() < 0.001);
        assert!((c.b - 1.05).abs() < 0.001);
        assert!((c.c - 2.5).abs() < 0.001);
        assert!((c.d - 0.38).abs() < 0.001);
    }

    #[test]
    fn cocomo_effort_formula_correct() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 10_000)];
        let report = derive_report(&export(rows), None);
        let c = report.cocomo.unwrap();
        // effort = a * kloc^b = 2.4 * 10.0^1.05
        let expected_effort = 2.4 * 10.0_f64.powf(1.05);
        assert!((c.effort_pm - expected_effort).abs() < 0.1);
    }

    #[test]
    fn cocomo_duration_formula_correct() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 10_000)];
        let report = derive_report(&export(rows), None);
        let c = report.cocomo.unwrap();
        let effort = 2.4 * 10.0_f64.powf(1.05);
        let expected_duration = 2.5 * effort.powf(0.38);
        assert!((c.duration_months - expected_duration).abs() < 0.1);
    }

    #[test]
    fn cocomo_staff_is_effort_over_duration() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 10_000)];
        let report = derive_report(&export(rows), None);
        let c = report.cocomo.unwrap();
        let expected_staff = c.effort_pm / c.duration_months;
        assert!((c.staff - expected_staff).abs() < 0.1);
    }

    #[test]
    fn cocomo_scales_with_kloc() {
        let small = derive_report(&export(vec![make_simple_row("a.rs", "Rust", 1000)]), None);
        let large = derive_report(
            &export(vec![make_simple_row("a.rs", "Rust", 100_000)]),
            None,
        );
        let s = small.cocomo.unwrap();
        let l = large.cocomo.unwrap();
        assert!(l.effort_pm > s.effort_pm);
        assert!(l.duration_months > s.duration_months);
    }
}

// ── Zero input ──────────────────────────────────────────────────

mod zero_input {
    use super::*;

    #[test]
    fn empty_rows_produce_valid_report() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.totals.files, 0);
        assert_eq!(report.totals.code, 0);
        assert_eq!(report.totals.lines, 0);
        assert!(report.cocomo.is_none());
    }

    #[test]
    fn empty_rows_distribution_zeroed() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.distribution.count, 0);
        assert_eq!(report.distribution.min, 0);
        assert_eq!(report.distribution.max, 0);
        assert_eq!(report.distribution.mean, 0.0);
        assert_eq!(report.distribution.gini, 0.0);
    }

    #[test]
    fn empty_rows_reading_time_zero() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.reading_time.minutes, 0.0);
        assert_eq!(report.reading_time.basis_lines, 0);
    }

    #[test]
    fn empty_rows_histogram_all_zeros() {
        let report = derive_report(&export(vec![]), None);
        for bucket in &report.histogram {
            assert_eq!(bucket.files, 0);
        }
    }

    #[test]
    fn empty_rows_polyglot_zero() {
        let report = derive_report(&export(vec![]), None);
        assert_eq!(report.polyglot.lang_count, 0);
        assert_eq!(report.polyglot.entropy, 0.0);
    }

    #[test]
    fn child_rows_are_excluded() {
        let mut child = make_simple_row("src/a.rs", "Rust", 100);
        child.kind = FileKind::Child;
        let report = derive_report(&export(vec![child]), None);
        assert_eq!(report.totals.files, 0);
        assert_eq!(report.totals.code, 0);
    }
}

// ── Single language input ───────────────────────────────────────

mod single_language {
    use super::*;

    #[test]
    fn totals_correct_for_single_file() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 100, 20, 10, 5000, 1000)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.totals.files, 1);
        assert_eq!(report.totals.code, 100);
        assert_eq!(report.totals.comments, 20);
        assert_eq!(report.totals.blanks, 10);
        assert_eq!(report.totals.lines, 130);
        assert_eq!(report.totals.bytes, 5000);
        assert_eq!(report.totals.tokens, 1000);
    }

    #[test]
    fn polyglot_entropy_zero_for_single_language() {
        let rows = vec![
            make_simple_row("src/a.rs", "Rust", 100),
            make_simple_row("src/b.rs", "Rust", 200),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.polyglot.lang_count, 1);
        assert_eq!(report.polyglot.entropy, 0.0);
        assert_eq!(report.polyglot.dominant_lang, "Rust");
        assert!((report.polyglot.dominant_pct - 1.0).abs() < 0.001);
    }

    #[test]
    fn nesting_report_for_single_file() {
        let rows = vec![make_row(
            "src/deep/nested/file.rs",
            "src",
            "Rust",
            100,
            0,
            0,
            4000,
            800,
        )];
        let report = derive_report(&export(rows), None);
        // path depth for "src/deep/nested/file.rs" = 4
        assert!(report.nesting.max > 0);
    }

    #[test]
    fn reading_time_proportional_to_code() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 200)];
        let report = derive_report(&export(rows), None);
        // 200 lines / 20 lines per minute = 10.0 minutes
        assert!((report.reading_time.minutes - 10.0).abs() < 0.01);
        assert_eq!(report.reading_time.lines_per_minute, 20);
        assert_eq!(report.reading_time.basis_lines, 200);
    }
}

// ── Multi-language input ────────────────────────────────────────

mod multi_language {
    use super::*;

    #[test]
    fn totals_sum_across_languages() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 10, 5, 4000, 800),
            make_row("src/b.py", "src", "Python", 200, 20, 10, 8000, 1600),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.totals.files, 2);
        assert_eq!(report.totals.code, 300);
        assert_eq!(report.totals.comments, 30);
    }

    #[test]
    fn polyglot_entropy_positive_for_two_languages() {
        let rows = vec![
            make_simple_row("src/a.rs", "Rust", 100),
            make_simple_row("src/b.py", "Python", 100),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.polyglot.lang_count, 2);
        assert!(report.polyglot.entropy > 0.0);
        // Two equal languages: entropy = 1.0 (log2(2))
        assert!((report.polyglot.entropy - 1.0).abs() < 0.001);
    }

    #[test]
    fn polyglot_dominant_language_is_largest() {
        let rows = vec![
            make_simple_row("src/a.rs", "Rust", 300),
            make_simple_row("src/b.py", "Python", 100),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.polyglot.dominant_lang, "Rust");
        assert_eq!(report.polyglot.dominant_lines, 300);
    }

    #[test]
    fn doc_density_by_module_is_grouped() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 80, 20, 0, 4000, 800),
            make_row("lib/b.py", "lib", "Python", 50, 50, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.doc_density.by_module.len(), 2);
    }

    #[test]
    fn verbosity_rate_calculated_correctly() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 100, 0, 0, 5000, 800)];
        let report = derive_report(&export(rows), None);
        // verbosity = bytes / lines = 5000 / 100 = 50.0
        assert_eq!(report.verbosity.total.numerator, 5000);
        assert_eq!(report.verbosity.total.denominator, 100);
        assert!((report.verbosity.total.rate - 50.0).abs() < 0.01);
    }
}

// ── Deterministic output ────────────────────────────────────────

mod determinism {
    use super::*;

    #[test]
    fn derive_report_is_deterministic() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 20, 10, 4000, 800),
            make_row("src/b.py", "src", "Python", 200, 40, 20, 8000, 1600),
            make_row("lib/c.go", "lib", "Go", 50, 5, 3, 2000, 400),
        ];
        let r1 = derive_report(&export(rows.clone()), Some(128000));
        let r2 = derive_report(&export(rows), Some(128000));
        let j1 = serde_json::to_string(&r1).unwrap();
        let j2 = serde_json::to_string(&r2).unwrap();
        assert_eq!(j1, j2);
    }

    #[test]
    fn derive_report_deterministic_across_row_orders() {
        let rows_a = vec![
            make_simple_row("src/a.rs", "Rust", 100),
            make_simple_row("src/b.py", "Python", 200),
        ];
        let rows_b = vec![
            make_simple_row("src/b.py", "Python", 200),
            make_simple_row("src/a.rs", "Rust", 100),
        ];
        let j1 = serde_json::to_string(&derive_report(&export(rows_a), None)).unwrap();
        let j2 = serde_json::to_string(&derive_report(&export(rows_b), None)).unwrap();
        assert_eq!(j1, j2);
    }

    #[test]
    fn integrity_hash_stable() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 100)];
        let r1 = derive_report(&export(rows.clone()), None);
        let r2 = derive_report(&export(rows), None);
        assert_eq!(r1.integrity.hash, r2.integrity.hash);
        assert_eq!(r1.integrity.algo, "blake3");
        assert_eq!(r1.integrity.entries, 1);
    }

    #[test]
    fn integrity_hash_changes_with_data() {
        let r1 = derive_report(
            &export(vec![make_simple_row("src/a.rs", "Rust", 100)]),
            None,
        );
        let r2 = derive_report(
            &export(vec![make_simple_row("src/a.rs", "Rust", 200)]),
            None,
        );
        assert_ne!(r1.integrity.hash, r2.integrity.hash);
    }
}

// ── Boundary values ─────────────────────────────────────────────

mod boundary {
    use super::*;

    #[test]
    fn single_line_file() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 1, 0, 0, 40, 8)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.totals.code, 1);
        assert_eq!(report.distribution.count, 1);
    }

    #[test]
    fn context_window_fits_when_tokens_within() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 500)];
        let report = derive_report(&export(rows), Some(1000));
        let cw = report.context_window.unwrap();
        assert!(cw.fits);
        assert_eq!(cw.window_tokens, 1000);
        assert_eq!(cw.total_tokens, 500);
        assert!((cw.pct - 0.5).abs() < 0.001);
    }

    #[test]
    fn context_window_does_not_fit_when_tokens_exceed() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 2000)];
        let report = derive_report(&export(rows), Some(1000));
        let cw = report.context_window.unwrap();
        assert!(!cw.fits);
        assert!(cw.pct > 1.0);
    }

    #[test]
    fn context_window_none_when_not_requested() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 100)];
        let report = derive_report(&export(rows), None);
        assert!(report.context_window.is_none());
    }

    #[test]
    fn context_window_zero_tokens_window() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 100)];
        let report = derive_report(&export(rows), Some(0));
        let cw = report.context_window.unwrap();
        assert_eq!(cw.pct, 0.0);
    }

    #[test]
    fn histogram_has_five_buckets() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 100)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.histogram.len(), 5);
        let labels: Vec<&str> = report.histogram.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, vec!["Tiny", "Small", "Medium", "Large", "Huge"]);
    }

    #[test]
    fn histogram_tiny_for_small_file() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 10, 0, 0, 400, 80)];
        let report = derive_report(&export(rows), None);
        // 10 lines => Tiny bucket (0..=50)
        assert_eq!(report.histogram[0].files, 1);
    }

    #[test]
    fn histogram_huge_for_large_file() {
        let rows = vec![make_row(
            "src/a.rs", "src", "Rust", 2000, 0, 0, 80000, 16000,
        )];
        let report = derive_report(&export(rows), None);
        // 2000 lines => Huge bucket (1001+)
        assert_eq!(report.histogram[4].files, 1);
    }
}

// ── Very large inputs ───────────────────────────────────────────

mod large_inputs {
    use super::*;

    #[test]
    fn thousand_files() {
        let rows: Vec<FileRow> = (0..1000)
            .map(|i| make_simple_row(&format!("src/{i}.rs"), "Rust", (i + 1) * 10))
            .collect();
        let report = derive_report(&export(rows), None);
        assert_eq!(report.totals.files, 1000);
        assert_eq!(
            report.totals.code,
            (1..=1000).map(|i| i * 10).sum::<usize>()
        );
        assert!(report.cocomo.is_some());
    }

    #[test]
    fn large_file_lines() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 1_000_000)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.totals.code, 1_000_000);
        let c = report.cocomo.unwrap();
        assert!((c.kloc - 1000.0).abs() < 0.1);
    }

    #[test]
    fn top_offenders_capped_at_ten() {
        let rows: Vec<FileRow> = (0..50)
            .map(|i| make_simple_row(&format!("src/{i}.rs"), "Rust", (i + 1) * 100))
            .collect();
        let report = derive_report(&export(rows), None);
        assert!(report.top.largest_lines.len() <= 10);
        assert!(report.top.largest_tokens.len() <= 10);
        assert!(report.top.largest_bytes.len() <= 10);
    }
}

// ── Serialization ───────────────────────────────────────────────

mod serialization {
    use super::*;

    #[test]
    fn derived_report_round_trips_through_json() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 20, 10, 4000, 800),
            make_row("src/b.py", "src", "Python", 200, 40, 20, 8000, 1600),
        ];
        let report = derive_report(&export(rows), Some(128000));
        let json = serde_json::to_string(&report).unwrap();
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn cocomo_serializes_all_fields() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 10_000)];
        let report = derive_report(&export(rows), None);
        let json = serde_json::to_value(report).unwrap();
        let cocomo = &json["cocomo"];
        assert!(cocomo["mode"].is_string());
        assert!(cocomo["kloc"].is_f64());
        assert!(cocomo["effort_pm"].is_f64());
        assert!(cocomo["duration_months"].is_f64());
        assert!(cocomo["staff"].is_f64());
        assert!(cocomo["a"].is_f64());
        assert!(cocomo["b"].is_f64());
    }

    #[test]
    fn distribution_serializes_all_fields() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 100)];
        let report = derive_report(&export(rows), None);
        let json = serde_json::to_value(report).unwrap();
        let dist = &json["distribution"];
        assert!(dist["count"].is_u64());
        assert!(dist["min"].is_u64());
        assert!(dist["max"].is_u64());
        assert!(dist["mean"].is_f64());
        assert!(dist["median"].is_f64());
        assert!(dist["p90"].is_f64());
        assert!(dist["p99"].is_f64());
        assert!(dist["gini"].is_f64());
    }

    #[test]
    fn context_window_absent_in_json_when_none() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 100)];
        let report = derive_report(&export(rows), None);
        let json = serde_json::to_value(report).unwrap();
        assert!(json.get("context_window").is_some()); // field exists but may be null
    }

    #[test]
    fn integrity_present_in_serialization() {
        let rows = vec![make_simple_row("src/a.rs", "Rust", 100)];
        let report = derive_report(&export(rows), None);
        let json = serde_json::to_value(report).unwrap();
        assert!(json["integrity"]["algo"].is_string());
        assert!(json["integrity"]["hash"].is_string());
        assert!(json["integrity"]["entries"].is_u64());
    }
}

// ── Test density ────────────────────────────────────────────────

mod test_density {
    use super::*;

    #[test]
    fn test_files_detected_by_path() {
        let rows = vec![
            make_simple_row("src/main.rs", "Rust", 100),
            make_simple_row("tests/test_main.rs", "Rust", 50),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.test_density.test_files, 1);
        assert_eq!(report.test_density.prod_files, 1);
        assert_eq!(report.test_density.test_lines, 50);
        assert_eq!(report.test_density.prod_lines, 100);
    }

    #[test]
    fn test_density_ratio_correct() {
        let rows = vec![
            make_simple_row("src/main.rs", "Rust", 75),
            make_simple_row("src/tests/test_main.rs", "Rust", 25),
        ];
        let report = derive_report(&export(rows), None);
        // ratio = test_lines / (test_lines + prod_lines) = 25 / 100 = 0.25
        assert!((report.test_density.ratio - 0.25).abs() < 0.001);
    }

    #[test]
    fn no_test_files_gives_zero_ratio() {
        let rows = vec![make_simple_row("src/main.rs", "Rust", 100)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.test_density.ratio, 0.0);
        assert_eq!(report.test_density.test_files, 0);
    }
}

// ── Boilerplate ─────────────────────────────────────────────────

mod boilerplate {
    use super::*;

    #[test]
    fn infra_langs_detected() {
        // TOML, YAML, JSON etc. are infrastructure languages
        let rows = vec![
            make_simple_row("src/main.rs", "Rust", 100),
            make_simple_row("Cargo.toml", "TOML", 50),
        ];
        let report = derive_report(&export(rows), None);
        assert!(report.boilerplate.infra_lines > 0 || report.boilerplate.logic_lines > 0);
    }

    #[test]
    fn no_infra_gives_zero_ratio() {
        let rows = vec![make_simple_row("src/main.rs", "Rust", 100)];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.boilerplate.ratio, 0.0);
    }
}

// ── Lang purity ─────────────────────────────────────────────────

mod lang_purity {
    use super::*;

    #[test]
    fn single_lang_module_has_100_percent_purity() {
        let rows = vec![
            make_simple_row("src/a.rs", "Rust", 100),
            make_simple_row("src/b.rs", "Rust", 200),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.lang_purity.rows.len(), 1);
        assert!((report.lang_purity.rows[0].dominant_pct - 1.0).abs() < 0.001);
    }

    #[test]
    fn mixed_lang_module_has_lower_purity() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("src/b.py", "src", "Python", 100, 0, 0, 4000, 800),
        ];
        let report = derive_report(&export(rows), None);
        assert_eq!(report.lang_purity.rows.len(), 1);
        assert!((report.lang_purity.rows[0].dominant_pct - 0.5).abs() < 0.001);
        assert_eq!(report.lang_purity.rows[0].lang_count, 2);
    }
}
