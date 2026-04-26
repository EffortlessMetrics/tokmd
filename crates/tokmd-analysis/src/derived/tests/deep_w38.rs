//! Deep tests (wave 38) for `analysis derived module`.
//!
//! Covers density metrics, distribution, COCOMO estimation, comment ratio,
//! edge cases (zero total, single language, many languages), and
//! deterministic ordering.

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

fn make_simple(path: &str, lang: &str, code: usize) -> FileRow {
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

// ── Density: doc density + whitespace ───────────────────────────

mod density_w38 {
    use super::*;

    #[test]
    fn doc_density_multiple_files() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 50, 0, 6000, 1200),
            make_row("src/b.rs", "src", "Rust", 200, 50, 0, 10000, 2000),
        ];
        let r = derive_report(&export(rows), None);
        // total comments=100, total code+comments=400
        assert_eq!(r.doc_density.total.numerator, 100);
        assert_eq!(r.doc_density.total.denominator, 400);
        assert!((r.doc_density.total.ratio - 0.25).abs() < 0.001);
    }

    #[test]
    fn whitespace_zero_when_no_blanks() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 100, 20, 0, 4800, 960)];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.whitespace.total.numerator, 0);
        assert_eq!(r.whitespace.total.ratio, 0.0);
    }

    #[test]
    fn whitespace_large_blanks() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 50, 10, 40, 4000, 800)];
        let r = derive_report(&export(rows), None);
        // whitespace = blanks / (code + comments) = 40 / 60
        assert_eq!(r.whitespace.total.numerator, 40);
        assert_eq!(r.whitespace.total.denominator, 60);
        assert!((r.whitespace.total.ratio - 0.6667).abs() < 0.01);
    }

    #[test]
    fn doc_density_by_module_groups_correctly() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 80, 20, 0, 4000, 800),
            make_row("lib/b.rs", "lib", "Rust", 60, 40, 0, 4000, 800),
        ];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.doc_density.by_module.len(), 2);
    }

    #[test]
    fn verbosity_rate_is_bytes_per_line() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 100, 0, 0, 3000, 600)];
        let r = derive_report(&export(rows), None);
        // verbosity = bytes / lines = 3000 / 100 = 30.0
        assert!((r.verbosity.total.rate - 30.0).abs() < 0.01);
    }
}

// ── Distribution metrics ────────────────────────────────────────

mod distribution_w38 {
    use super::*;

    #[test]
    fn four_files_even_distribution() {
        let rows: Vec<FileRow> = (1..=4)
            .map(|i| make_row(&format!("src/{i}.rs"), "src", "Rust", 100, 0, 0, 4000, 800))
            .collect();
        let r = derive_report(&export(rows), None);
        assert_eq!(r.distribution.count, 4);
        assert_eq!(r.distribution.min, 100);
        assert_eq!(r.distribution.max, 100);
        assert!((r.distribution.mean - 100.0).abs() < 0.01);
        assert!((r.distribution.median - 100.0).abs() < 0.01);
        assert!((r.distribution.gini - 0.0).abs() < 0.01);
    }

    #[test]
    fn ten_files_increasing_sizes() {
        let rows: Vec<FileRow> = (1..=10)
            .map(|i| {
                make_row(
                    &format!("src/{i}.rs"),
                    "src",
                    "Rust",
                    i * 50,
                    0,
                    0,
                    i * 2000,
                    i * 400,
                )
            })
            .collect();
        let r = derive_report(&export(rows), None);
        assert_eq!(r.distribution.count, 10);
        assert_eq!(r.distribution.min, 50);
        assert_eq!(r.distribution.max, 500);
        assert!(r.distribution.mean > 0.0);
        assert!(r.distribution.p90 >= r.distribution.median);
    }

    #[test]
    fn p90_equals_max_for_single_file() {
        let rows = vec![make_simple("src/a.rs", "Rust", 200)];
        let r = derive_report(&export(rows), None);
        assert!((r.distribution.p90 - 200.0).abs() < 0.01);
        assert!((r.distribution.p99 - 200.0).abs() < 0.01);
    }
}

// ── COCOMO estimation ───────────────────────────────────────────

mod cocomo_w38 {
    use super::*;

    #[test]
    fn cocomo_1k_lines() {
        let rows = vec![make_simple("src/a.rs", "Rust", 1000)];
        let r = derive_report(&export(rows), None);
        let c = r.cocomo.unwrap();
        assert!((c.kloc - 1.0).abs() < 0.001);
        // effort = 2.4 * 1.0^1.05 = 2.4
        assert!((c.effort_pm - 2.4).abs() < 0.1);
    }

    #[test]
    fn cocomo_50k_lines() {
        let rows = vec![make_simple("src/a.rs", "Rust", 50_000)];
        let r = derive_report(&export(rows), None);
        let c = r.cocomo.unwrap();
        assert!((c.kloc - 50.0).abs() < 0.001);
        let expected = 2.4 * 50.0_f64.powf(1.05);
        assert!((c.effort_pm - expected).abs() < 0.5);
    }

    #[test]
    fn cocomo_staff_positive() {
        let rows = vec![make_simple("src/a.rs", "Rust", 5000)];
        let r = derive_report(&export(rows), None);
        let c = r.cocomo.unwrap();
        assert!(c.staff > 0.0);
    }

    #[test]
    fn cocomo_duration_positive() {
        let rows = vec![make_simple("src/a.rs", "Rust", 5000)];
        let r = derive_report(&export(rows), None);
        let c = r.cocomo.unwrap();
        assert!(c.duration_months > 0.0);
    }

    #[test]
    fn cocomo_aggregates_across_files() {
        let rows = vec![
            make_simple("src/a.rs", "Rust", 500),
            make_simple("src/b.rs", "Rust", 500),
        ];
        let r = derive_report(&export(rows), None);
        let c = r.cocomo.unwrap();
        assert!((c.kloc - 1.0).abs() < 0.001);
    }
}

// ── Comment ratio ───────────────────────────────────────────────

mod comment_ratio_w38 {
    use super::*;

    #[test]
    fn ratio_fifty_fifty() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 50, 50, 0, 4000, 800)];
        let r = derive_report(&export(rows), None);
        assert!((r.doc_density.total.ratio - 0.5).abs() < 0.001);
    }

    #[test]
    fn ratio_by_lang_sorted_descending() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 90, 10, 0, 4000, 800),
            make_row("src/b.py", "src", "Python", 20, 80, 0, 4000, 800),
            make_row("src/c.go", "src", "Go", 50, 50, 0, 4000, 800),
        ];
        let r = derive_report(&export(rows), None);
        let ratios: Vec<f64> = r.doc_density.by_lang.iter().map(|r| r.ratio).collect();
        for w in ratios.windows(2) {
            assert!(w[0] >= w[1], "by_lang not sorted descending: {:?}", ratios);
        }
    }

    #[test]
    fn ratio_zero_for_zero_total() {
        let r = derive_report(&export(vec![]), None);
        assert_eq!(r.doc_density.total.ratio, 0.0);
        assert_eq!(r.whitespace.total.ratio, 0.0);
    }
}

// ── Edge cases ──────────────────────────────────────────────────

mod edge_cases_w38 {
    use super::*;

    #[test]
    fn zero_total_lines_no_panic() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 0, 0, 0, 0, 0)];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.totals.files, 1);
        assert_eq!(r.totals.lines, 0);
    }

    #[test]
    fn hundred_languages() {
        let rows: Vec<FileRow> = (0..100)
            .map(|i| make_simple(&format!("src/{i}.ext"), &format!("Lang{i}"), 10))
            .collect();
        let r = derive_report(&export(rows), None);
        assert_eq!(r.polyglot.lang_count, 100);
        assert!(r.polyglot.entropy > 0.0);
        assert_eq!(r.totals.files, 100);
        assert_eq!(r.totals.code, 1000);
    }

    #[test]
    fn single_language_entropy_zero() {
        let rows = vec![
            make_simple("src/a.rs", "Rust", 100),
            make_simple("src/b.rs", "Rust", 200),
            make_simple("src/c.rs", "Rust", 300),
        ];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.polyglot.entropy, 0.0);
        assert_eq!(r.polyglot.lang_count, 1);
    }

    #[test]
    fn only_child_rows_empty_report() {
        let mut row = make_simple("src/a.rs", "Rust", 100);
        row.kind = FileKind::Child;
        let r = derive_report(&export(vec![row]), None);
        assert_eq!(r.totals.files, 0);
        assert_eq!(r.totals.code, 0);
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn all_blanks_no_code() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 0, 0, 100, 0, 0)];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.totals.blanks, 100);
        assert_eq!(r.totals.code, 0);
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn all_comments_no_code() {
        let rows = vec![make_row("src/a.rs", "src", "Rust", 0, 100, 0, 4000, 800)];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.totals.comments, 100);
        assert_eq!(r.totals.code, 0);
        assert!(r.cocomo.is_none());
        assert!((r.doc_density.total.ratio - 1.0).abs() < 0.001);
    }
}

// ── Deterministic ordering ──────────────────────────────────────

mod determinism_w38 {
    use super::*;

    #[test]
    fn report_identical_across_runs() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 20, 5, 4000, 800),
            make_row("lib/b.py", "lib", "Python", 200, 40, 10, 8000, 1600),
            make_row("src/c.go", "src", "Go", 50, 10, 3, 2000, 400),
        ];
        let j1 =
            serde_json::to_string(&derive_report(&export(rows.clone()), Some(128000))).unwrap();
        let j2 = serde_json::to_string(&derive_report(&export(rows), Some(128000))).unwrap();
        assert_eq!(j1, j2);
    }

    #[test]
    fn report_stable_regardless_of_input_order() {
        let a = vec![
            make_simple("src/z.rs", "Rust", 300),
            make_simple("src/a.rs", "Rust", 100),
            make_simple("src/m.rs", "Rust", 200),
        ];
        let b = vec![
            make_simple("src/a.rs", "Rust", 100),
            make_simple("src/m.rs", "Rust", 200),
            make_simple("src/z.rs", "Rust", 300),
        ];
        let j1 = serde_json::to_string(&derive_report(&export(a), None)).unwrap();
        let j2 = serde_json::to_string(&derive_report(&export(b), None)).unwrap();
        assert_eq!(j1, j2);
    }

    #[test]
    fn integrity_hash_differs_for_different_data() {
        let r1 = derive_report(&export(vec![make_simple("a.rs", "Rust", 100)]), None);
        let r2 = derive_report(&export(vec![make_simple("b.rs", "Rust", 100)]), None);
        assert_ne!(r1.integrity.hash, r2.integrity.hash);
    }

    #[test]
    fn top_offenders_sorted_by_lines_desc() {
        let rows: Vec<FileRow> = (1..=20)
            .map(|i| make_simple(&format!("src/{i}.rs"), "Rust", i * 100))
            .collect();
        let r = derive_report(&export(rows), None);
        let lines: Vec<usize> = r.top.largest_lines.iter().map(|f| f.lines).collect();
        for w in lines.windows(2) {
            assert!(w[0] >= w[1], "not sorted desc: {:?}", lines);
        }
    }
}

// ── Histogram buckets ───────────────────────────────────────────

mod histogram_w38 {
    use super::*;

    #[test]
    fn files_fall_into_correct_buckets() {
        let rows = vec![
            make_row("src/tiny.rs", "src", "Rust", 10, 0, 0, 400, 80),
            make_row("src/small.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("src/med.rs", "src", "Rust", 300, 0, 0, 12000, 2400),
            make_row("src/large.rs", "src", "Rust", 700, 0, 0, 28000, 5600),
            make_row("src/huge.rs", "src", "Rust", 2000, 0, 0, 80000, 16000),
        ];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.histogram.len(), 5);
        assert_eq!(r.histogram[0].label, "Tiny");
        assert_eq!(r.histogram[0].files, 1);
        assert_eq!(r.histogram[1].label, "Small");
        assert_eq!(r.histogram[1].files, 1);
        assert_eq!(r.histogram[2].label, "Medium");
        assert_eq!(r.histogram[2].files, 1);
        assert_eq!(r.histogram[3].label, "Large");
        assert_eq!(r.histogram[3].files, 1);
        assert_eq!(r.histogram[4].label, "Huge");
        assert_eq!(r.histogram[4].files, 1);
    }

    #[test]
    fn histogram_pct_sums_to_one() {
        let rows: Vec<FileRow> = (1..=10)
            .map(|i| make_simple(&format!("src/{i}.rs"), "Rust", i * 100))
            .collect();
        let r = derive_report(&export(rows), None);
        let total_pct: f64 = r.histogram.iter().map(|b| b.pct).sum();
        assert!((total_pct - 1.0).abs() < 0.01);
    }
}

// ── Nesting report ──────────────────────────────────────────────

mod nesting_w38 {
    use super::*;

    #[test]
    fn nesting_tracks_depth() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("src/b.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("src/deep/c.rs", "src", "Rust", 50, 0, 0, 2000, 400),
        ];
        let r = derive_report(&export(rows), None);
        assert!(r.nesting.max >= 2);
        assert!(r.nesting.avg > 0.0);
    }
}
