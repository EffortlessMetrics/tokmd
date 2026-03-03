//! Deep tests for `tokmd-analysis-derived`.
//!
//! Covers COCOMO model calculations, code density, distribution metrics,
//! histogram bucket boundaries, reading time, verbosity, boilerplate,
//! polyglot entropy, test density, nesting depth, integrity hashing,
//! and large-input / rounding-consistency scenarios.

use tokmd_analysis_derived::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ──────────────────────────────────────────────────────────

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

// ── COCOMO deep ──────────────────────────────────────────────────────

mod cocomo_deep {
    use super::*;

    #[test]
    fn empty_input_yields_none() {
        let r = derive_report(&export(vec![]), None);
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn zero_code_lines_yields_none() {
        let rows = vec![make_row("a.rs", "src", "Rust", 0, 10, 5, 600, 120)];
        let r = derive_report(&export(rows), None);
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn cocomo_one_line_of_code_returns_some() {
        let rows = vec![make_row("f.rs", "src", "Rust", 1, 0, 0, 40, 8)];
        let cocomo = derive_report(&export(rows), None).cocomo.unwrap();
        assert_eq!(cocomo.kloc, 0.001);
        // effort = 2.4 * 0.001^1.05 ~ 0.0017, rounds to 0.00 at 2dp
        assert_eq!(cocomo.effort_pm, 0.0);
    }

    #[test]
    fn cocomo_1000_lines() {
        let rows = vec![make_row("a.rs", "src", "Rust", 1000, 100, 50, 46000, 9200)];
        let c = derive_report(&export(rows), None).cocomo.unwrap();
        // kloc=1.0, effort=2.4*1^1.05=2.4
        assert!((c.effort_pm - 2.4).abs() < 0.1);
        assert!(c.duration_months > 0.0);
        assert!(c.staff > 0.0);
    }

    #[test]
    fn cocomo_10k_lines() {
        let rows = vec![make_row(
            "a.rs", "src", "Rust", 10000, 500, 200, 428000, 85600,
        )];
        let c = derive_report(&export(rows), None).cocomo.unwrap();
        assert!(c.effort_pm > 20.0);
        assert!(c.effort_pm < 35.0);
    }

    #[test]
    fn cocomo_100k_lines() {
        let rows = vec![make_row(
            "a.rs", "src", "Rust", 100000, 5000, 2000, 4280000, 856000,
        )];
        let c = derive_report(&export(rows), None).cocomo.unwrap();
        assert!(c.effort_pm > 200.0);
        assert!(c.duration_months > 10.0);
        assert!(c.staff >= 1.0);
    }

    #[test]
    fn cocomo_multiple_files_sums_code() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 500, 0, 0, 20000, 4000),
            make_row("b.rs", "src", "Rust", 500, 0, 0, 20000, 4000),
        ];
        let cocomo = derive_report(&export(rows), None).cocomo.unwrap();
        assert_eq!(cocomo.kloc, 1.0);
    }

    #[test]
    fn cocomo_fields_non_negative() {
        for n in [1usize, 10, 100, 1000, 10000, 100000] {
            let rows = vec![make_row("a.rs", "src", "Rust", n, 0, 0, n * 40, n * 8)];
            if let Some(c) = derive_report(&export(rows), None).cocomo {
                assert!(c.effort_pm >= 0.0, "effort negative for {n}");
                assert!(c.duration_months >= 0.0, "duration negative for {n}");
                assert!(c.staff >= 0.0, "staff negative for {n}");
            }
        }
    }

    #[test]
    fn cocomo_mode_is_organic() {
        let rows = vec![make_row("a.rs", "src", "Rust", 1000, 0, 0, 40000, 8000)];
        let c = derive_report(&export(rows), None).cocomo.unwrap();
        assert_eq!(c.mode, "organic");
    }

    #[test]
    fn cocomo_constants_correct() {
        let rows = vec![make_row("a.rs", "src", "Rust", 1000, 0, 0, 40000, 8000)];
        let c = derive_report(&export(rows), None).cocomo.unwrap();
        assert_eq!(c.a, 2.4);
        assert_eq!(c.b, 1.05);
        assert_eq!(c.c, 2.5);
        assert_eq!(c.d, 0.38);
    }
}

// ── density (doc_density) deep ───────────────────────────────────────

mod density_deep {
    use super::*;

    #[test]
    fn density_all_code() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let d = derive_report(&export(rows), None).doc_density;
        assert_eq!(d.total.ratio, 0.0);
    }

    #[test]
    fn density_all_comments() {
        let rows = vec![make_row("a.rs", "src", "Rust", 0, 100, 0, 4000, 800)];
        let d = derive_report(&export(rows), None).doc_density;
        assert_eq!(d.total.ratio, 1.0);
    }

    #[test]
    fn density_mixed() {
        let rows = vec![make_row("a.rs", "src", "Rust", 60, 40, 10, 4400, 880)];
        let d = derive_report(&export(rows), None).doc_density;
        assert_eq!(d.total.ratio, 0.4);
    }

    #[test]
    fn density_zero_lines() {
        let r = derive_report(&export(vec![]), None);
        assert_eq!(r.doc_density.total.ratio, 0.0);
    }

    #[test]
    fn density_mixed_langs_by_lang() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 70, 30, 10, 4400, 880),
            make_row("b.py", "src", "Python", 50, 50, 5, 4200, 840),
        ];
        let d = derive_report(&export(rows), None).doc_density;
        assert!(d.by_lang.len() >= 2);
    }

    #[test]
    fn density_by_module() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 70, 30, 10, 4400, 880),
            make_row("tests/b.rs", "tests", "Rust", 50, 50, 5, 4200, 840),
        ];
        let d = derive_report(&export(rows), None).doc_density;
        assert!(d.by_module.len() >= 2);
    }
}

// ── distribution deep ────────────────────────────────────────────────

mod distribution_deep {
    use super::*;

    #[test]
    fn distribution_empty_is_zeroed() {
        let d = derive_report(&export(vec![]), None).distribution;
        assert_eq!(d.count, 0);
        assert_eq!(d.min, 0);
        assert_eq!(d.max, 0);
        assert_eq!(d.mean, 0.0);
        assert_eq!(d.median, 0.0);
        assert_eq!(d.gini, 0.0);
    }

    #[test]
    fn single_file() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let d = derive_report(&export(rows), None).distribution;
        assert_eq!(d.count, 1);
        // distribution uses `lines` = code+comments+blanks = 115
        assert_eq!(d.min, 115);
        assert_eq!(d.max, 115);
        assert_eq!(d.gini, 0.0);
    }

    #[test]
    fn two_files_stats() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.rs", "src", "Rust", 200, 0, 0, 8000, 1600),
        ];
        let d = derive_report(&export(rows), None).distribution;
        assert_eq!(d.count, 2);
        assert_eq!(d.min, 100);
        assert_eq!(d.max, 200);
        assert_eq!(d.mean, 150.0);
    }

    #[test]
    fn gini_equal_files() {
        let rows: Vec<_> = (0..10)
            .map(|i| make_row(&format!("f{i}.rs"), "src", "Rust", 100, 0, 0, 4000, 800))
            .collect();
        let d = derive_report(&export(rows), None).distribution;
        assert_eq!(d.gini, 0.0);
    }

    #[test]
    fn gini_unequal_files() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 1, 0, 0, 40, 8),
            make_row("b.rs", "src", "Rust", 1000, 0, 0, 40000, 8000),
        ];
        let d = derive_report(&export(rows), None).distribution;
        assert!(d.gini > 0.0);
    }

    #[test]
    fn percentiles_bounded() {
        let rows: Vec<_> = (0..20)
            .map(|i| {
                make_row(
                    &format!("f{i}.rs"),
                    "src",
                    "Rust",
                    (i + 1) * 50,
                    0,
                    0,
                    (i + 1) * 2000,
                    (i + 1) * 400,
                )
            })
            .collect();
        let d = derive_report(&export(rows), None).distribution;
        assert!(d.p90 >= d.median);
        assert!(d.p99 >= d.p90);
    }
}

// ── histogram deep ───────────────────────────────────────────────────

mod histogram_deep {
    use super::*;

    #[test]
    fn histogram_empty_is_empty() {
        let h = derive_report(&export(vec![]), None).histogram;
        let total: usize = h.iter().map(|b| b.files).sum();
        assert_eq!(total, 0);
    }

    #[test]
    fn histogram_single_file() {
        let rows = vec![make_row("a.rs", "src", "Rust", 50, 5, 2, 2280, 456)];
        let h = derive_report(&export(rows), None).histogram;
        let total: usize = h.iter().map(|b| b.files).sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn histogram_many_files_correct_total() {
        let rows: Vec<_> = (0..20)
            .map(|i| {
                make_row(
                    &format!("f{i}.rs"),
                    "src",
                    "Rust",
                    (i + 1) * 10,
                    0,
                    0,
                    (i + 1) * 400,
                    (i + 1) * 80,
                )
            })
            .collect();
        let h = derive_report(&export(rows), None).histogram;
        let total: usize = h.iter().map(|b| b.files).sum();
        assert_eq!(total, 20);
    }

    #[test]
    fn histogram_all_same_size() {
        let rows: Vec<_> = (0..5)
            .map(|i| make_row(&format!("f{i}.rs"), "src", "Rust", 50, 0, 0, 2000, 400))
            .collect();
        let h = derive_report(&export(rows), None).histogram;
        let non_empty: Vec<_> = h.iter().filter(|b| b.files > 0).collect();
        assert_eq!(non_empty.len(), 1);
        assert_eq!(non_empty[0].files, 5);
    }

    #[test]
    fn histogram_bucket_labels_present() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let h = derive_report(&export(rows), None).histogram;
        for b in &h {
            assert!(!b.label.is_empty());
        }
    }

    #[test]
    fn histogram_bucket_boundaries_ascending() {
        let rows: Vec<_> = (0..50)
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
        let h = derive_report(&export(rows), None).histogram;
        for w in h.windows(2) {
            assert!(w[0].min <= w[1].min, "buckets not ascending");
        }
    }

    #[test]
    fn histogram_zero_code_files_counted() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 0, 10, 5, 600, 120),
            make_row("b.rs", "src", "Rust", 100, 10, 5, 4600, 920),
        ];
        let h = derive_report(&export(rows), None).histogram;
        let total: usize = h.iter().map(|b| b.files).sum();
        assert_eq!(total, 2);
    }
}

// ── verbosity deep ───────────────────────────────────────────────────

mod verbosity_deep {
    use super::*;

    #[test]
    fn verbosity_present_for_nonempty() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 50, 10, 6400, 1280)];
        let v = derive_report(&export(rows), None).verbosity;
        assert!(v.total.rate >= 0.0);
    }

    #[test]
    fn verbosity_empty_zeroed() {
        let v = derive_report(&export(vec![]), None).verbosity;
        assert_eq!(v.total.rate, 0.0);
    }

    #[test]
    fn verbosity_by_lang() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.py", "src", "Python", 50, 0, 0, 2000, 400),
        ];
        let v = derive_report(&export(rows), None).verbosity;
        assert!(v.by_lang.len() >= 2);
    }
}

// ── whitespace deep ──────────────────────────────────────────────────

mod whitespace_deep {
    use super::*;

    #[test]
    fn whitespace_present() {
        let rows = vec![make_row("a.rs", "src", "Rust", 80, 10, 10, 4000, 800)];
        let w = derive_report(&export(rows), None).whitespace;
        assert!(w.total.ratio >= 0.0);
    }

    #[test]
    fn whitespace_all_blanks() {
        // code=0, comments=0 → denominator=0 → safe_ratio returns 0.0
        let rows = vec![make_row("a.rs", "src", "Rust", 0, 0, 100, 4000, 800)];
        let w = derive_report(&export(rows), None).whitespace;
        assert_eq!(w.total.ratio, 0.0);
    }

    #[test]
    fn whitespace_no_blanks() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let w = derive_report(&export(rows), None).whitespace;
        assert_eq!(w.total.ratio, 0.0);
    }
}

// ── boilerplate deep ─────────────────────────────────────────────────

mod boilerplate_deep {
    use super::*;

    #[test]
    fn boilerplate_with_infra() {
        let rows = vec![
            make_row("src/main.rs", "src", "Rust", 80, 10, 10, 4000, 800),
            make_row("Cargo.toml", ".", "TOML", 20, 2, 3, 1000, 200),
        ];
        let b = derive_report(&export(rows), None).boilerplate;
        assert!(b.infra_lines > 0);
    }

    #[test]
    fn boilerplate_empty() {
        let b = derive_report(&export(vec![]), None).boilerplate;
        assert_eq!(b.infra_lines, 0);
        assert_eq!(b.logic_lines, 0);
    }

    #[test]
    fn boilerplate_infra_vs_logic() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("Cargo.toml", ".", "TOML", 50, 0, 0, 2000, 400),
        ];
        let b = derive_report(&export(rows), None).boilerplate;
        assert!(b.infra_lines > 0);
        assert!(b.logic_lines > 0);
    }

    #[test]
    fn boilerplate_ratio_range() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 100, 10, 5, 4600, 920),
            make_row("Makefile", ".", "Makefile", 20, 5, 2, 1080, 216),
        ];
        let b = derive_report(&export(rows), None).boilerplate;
        assert!(b.ratio >= 0.0 && b.ratio <= 1.0);
    }

    #[test]
    fn boilerplate_infra_langs_populated() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("Cargo.toml", ".", "TOML", 50, 0, 0, 2000, 400),
            make_row("config.yaml", ".", "YAML", 30, 0, 0, 1200, 240),
        ];
        let b = derive_report(&export(rows), None).boilerplate;
        assert!(!b.infra_langs.is_empty());
    }
}

// ── polyglot deep ────────────────────────────────────────────────────

mod polyglot_deep {
    use super::*;

    #[test]
    fn polyglot_single_lang() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let p = derive_report(&export(rows), None).polyglot;
        assert_eq!(p.lang_count, 1);
        assert_eq!(p.entropy, 0.0);
    }

    #[test]
    fn polyglot_two_equal_langs() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("b.py", "src", "Python", 50, 0, 0, 2000, 400),
        ];
        let p = derive_report(&export(rows), None).polyglot;
        assert_eq!(p.lang_count, 2);
        // Shannon entropy (log2) of uniform 2-category = 1.0
        assert!(p.entropy > 0.9 && p.entropy < 1.1);
    }

    #[test]
    fn polyglot_empty() {
        let p = derive_report(&export(vec![]), None).polyglot;
        assert_eq!(p.lang_count, 0);
    }

    #[test]
    fn polyglot_dominant_lang() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 90, 0, 0, 3600, 720),
            make_row("b.py", "src", "Python", 10, 0, 0, 400, 80),
        ];
        let p = derive_report(&export(rows), None).polyglot;
        assert_eq!(p.dominant_lang, "Rust");
        // dominant_pct is safe_ratio → 0.0-1.0 range
        assert!(p.dominant_pct > 0.8);
    }

    #[test]
    fn polyglot_entropy_increases_with_langs() {
        let two_rows = vec![
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("b.py", "src", "Python", 50, 0, 0, 2000, 400),
        ];
        let three_rows = vec![
            make_row("a.rs", "src", "Rust", 34, 0, 0, 1360, 272),
            make_row("b.py", "src", "Python", 33, 0, 0, 1320, 264),
            make_row("c.go", "src", "Go", 33, 0, 0, 1320, 264),
        ];
        let two = derive_report(&export(two_rows), None).polyglot;
        let three = derive_report(&export(three_rows), None).polyglot;
        assert!(three.entropy > two.entropy);
    }
}

// ── test density deep ────────────────────────────────────────────────

mod test_density_deep {
    use super::*;

    #[test]
    fn test_density_no_tests() {
        let rows = vec![make_row("src/lib.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let t = derive_report(&export(rows), None).test_density;
        assert_eq!(t.test_lines, 0);
        assert_eq!(t.ratio, 0.0);
    }

    #[test]
    fn test_density_with_tests() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 80, 0, 0, 3200, 640),
            make_row("tests/test_lib.rs", "tests", "Rust", 20, 0, 0, 800, 160),
        ];
        let t = derive_report(&export(rows), None).test_density;
        assert!(t.test_lines > 0);
        assert!(t.ratio > 0.0);
    }

    #[test]
    fn test_density_all_test_files() {
        let rows = vec![
            make_row("src/tests/a.rs", "src/tests", "Rust", 50, 0, 0, 2000, 400),
            make_row("src/tests/b.rs", "src/tests", "Rust", 50, 0, 0, 2000, 400),
        ];
        let t = derive_report(&export(rows), None).test_density;
        assert_eq!(t.prod_lines, 0);
        assert!(t.test_lines > 0);
    }

    #[test]
    fn test_density_file_counts() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("src/b.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("src/tests/t.rs", "src/tests", "Rust", 50, 0, 0, 2000, 400),
        ];
        let t = derive_report(&export(rows), None).test_density;
        assert_eq!(t.test_files, 1);
        assert_eq!(t.prod_files, 2);
    }
}

// ── nesting deep ─────────────────────────────────────────────────────

mod nesting_deep {
    use super::*;

    #[test]
    fn nesting_present() {
        let rows = vec![
            make_row("src/lib.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("src/a/b/c.rs", "src/a/b", "Rust", 50, 0, 0, 2000, 400),
        ];
        let n = derive_report(&export(rows), None).nesting;
        assert!(n.max >= 1);
    }

    #[test]
    fn nesting_flat_structure() {
        let rows = vec![
            make_row("a.rs", ".", "Rust", 100, 0, 0, 4000, 800),
            make_row("b.rs", ".", "Rust", 100, 0, 0, 4000, 800),
        ];
        let n = derive_report(&export(rows), None).nesting;
        assert!(n.avg >= 1.0);
    }

    #[test]
    fn nesting_deep_path() {
        let rows = vec![make_row(
            "a/b/c/d/e/f/deep.rs",
            "a/b/c/d/e/f",
            "Rust",
            100,
            0,
            0,
            4000,
            800,
        )];
        let n = derive_report(&export(rows), None).nesting;
        assert!(n.max >= 5);
    }

    #[test]
    fn nesting_by_module() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("lib/b.rs", "lib", "Rust", 100, 0, 0, 4000, 800),
        ];
        let n = derive_report(&export(rows), None).nesting;
        assert!(n.by_module.len() >= 2);
    }
}

// ── reading time deep ────────────────────────────────────────────────

mod reading_time_deep {
    use super::*;

    #[test]
    fn reading_time_present() {
        let rows = vec![make_row("a.rs", "src", "Rust", 1000, 100, 50, 46000, 9200)];
        let rt = derive_report(&export(rows), None).reading_time;
        assert!(rt.minutes > 0.0);
    }

    #[test]
    fn reading_time_empty() {
        let rt = derive_report(&export(vec![]), None).reading_time;
        assert_eq!(rt.minutes, 0.0);
    }

    #[test]
    fn reading_time_scales_with_lines() {
        let small = vec![make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let large = vec![make_row("a.rs", "src", "Rust", 10000, 0, 0, 400000, 80000)];
        let sm = derive_report(&export(small), None).reading_time;
        let lg = derive_report(&export(large), None).reading_time;
        assert!(lg.minutes > sm.minutes);
    }

    #[test]
    fn reading_time_basis_lines() {
        let rows = vec![make_row("a.rs", "src", "Rust", 200, 100, 50, 14000, 2800)];
        let rt = derive_report(&export(rows), None).reading_time;
        // basis_lines = totals.code = 200
        assert_eq!(rt.basis_lines, 200);
    }
}

// ── context window deep ─────────────────────────────────────────────

mod context_window_deep {
    use super::*;

    #[test]
    fn context_window_none_without_arg() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        assert!(derive_report(&export(rows), None).context_window.is_none());
    }

    #[test]
    fn context_window_some_with_arg() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let cw = derive_report(&export(rows), Some(128_000))
            .context_window
            .unwrap();
        assert_eq!(cw.window_tokens, 128_000);
    }

    #[test]
    fn context_window_total_tokens() {
        let rows = vec![make_row("a.rs", "src", "Rust", 1000, 100, 50, 46000, 9200)];
        let cw = derive_report(&export(rows), Some(128_000))
            .context_window
            .unwrap();
        assert_eq!(cw.total_tokens, 9200);
    }

    #[test]
    fn context_window_fits() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 0, 0, 4000, 800)];
        let cw = derive_report(&export(rows), Some(128_000))
            .context_window
            .unwrap();
        assert!(cw.fits);
        assert!(cw.pct < 100.0);
    }

    #[test]
    fn context_window_does_not_fit() {
        let rows = vec![make_row(
            "a.rs", "src", "Rust", 100000, 0, 0, 4000000, 800000,
        )];
        let cw = derive_report(&export(rows), Some(1000))
            .context_window
            .unwrap();
        assert!(!cw.fits);
        assert!(cw.pct > 100.0);
    }
}

// ── integrity deep ───────────────────────────────────────────────────

mod integrity_deep {
    use super::*;

    #[test]
    fn integrity_present() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let integ = derive_report(&export(rows), None).integrity;
        assert!(!integ.hash.is_empty());
    }

    #[test]
    fn integrity_algo() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let integ = derive_report(&export(rows), None).integrity;
        assert_eq!(integ.algo, "blake3");
    }

    #[test]
    fn integrity_deterministic() {
        let rows = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let h1 = derive_report(&export(rows.clone()), None).integrity.hash;
        let h2 = derive_report(&export(rows), None).integrity.hash;
        assert_eq!(h1, h2);
    }

    #[test]
    fn integrity_changes_with_input() {
        let r1 = vec![make_row("a.rs", "src", "Rust", 100, 10, 5, 4600, 920)];
        let r2 = vec![make_row("a.rs", "src", "Rust", 101, 10, 5, 4640, 928)];
        let h1 = derive_report(&export(r1), None).integrity.hash;
        let h2 = derive_report(&export(r2), None).integrity.hash;
        assert_ne!(h1, h2);
    }

    #[test]
    fn integrity_entries_count() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("b.rs", "src", "Rust", 50, 0, 0, 2000, 400),
        ];
        let integ = derive_report(&export(rows), None).integrity;
        assert_eq!(integ.entries, 2);
    }
}

// ── lang purity deep ─────────────────────────────────────────────────

mod lang_purity_deep {
    use super::*;

    #[test]
    fn lang_purity_single_module_single_lang() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("b.rs", "src", "Rust", 50, 0, 0, 2000, 400),
        ];
        let lp = derive_report(&export(rows), None).lang_purity;
        assert!(!lp.rows.is_empty());
        assert_eq!(lp.rows[0].lang_count, 1);
    }

    #[test]
    fn lang_purity_mixed_module() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 80, 0, 0, 3200, 640),
            make_row("b.py", "src", "Python", 20, 0, 0, 800, 160),
        ];
        let lp = derive_report(&export(rows), None).lang_purity;
        assert!(!lp.rows.is_empty());
        assert!(lp.rows[0].lang_count >= 2);
    }

    #[test]
    fn lang_purity_multiple_modules() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("lib/b.py", "lib", "Python", 50, 0, 0, 2000, 400),
        ];
        let lp = derive_report(&export(rows), None).lang_purity;
        assert!(lp.rows.len() >= 2);
    }
}

// ── top offenders deep ───────────────────────────────────────────────

mod top_offenders_deep {
    use super::*;

    #[test]
    fn top_largest_lines() {
        let rows: Vec<_> = (0..20)
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
        let top = derive_report(&export(rows), None).top;
        assert!(!top.largest_lines.is_empty());
        assert!(top.largest_lines.len() <= 10);
    }

    #[test]
    fn top_largest_lines_sorted_desc() {
        let rows: Vec<_> = (0..20)
            .map(|i| {
                make_row(
                    &format!("f{i}.rs"),
                    "src",
                    "Rust",
                    (i + 1) * 100,
                    (i + 1) * 10,
                    (i + 1) * 5,
                    (i + 1) * 4000,
                    (i + 1) * 800,
                )
            })
            .collect();
        let top = derive_report(&export(rows), None).top;
        for w in top.largest_lines.windows(2) {
            assert!(w[0].lines >= w[1].lines);
        }
    }

    #[test]
    fn top_largest_tokens() {
        let rows: Vec<_> = (0..15)
            .map(|i| {
                make_row(
                    &format!("f{i}.rs"),
                    "src",
                    "Rust",
                    100,
                    0,
                    0,
                    4000,
                    (i + 1) * 200,
                )
            })
            .collect();
        let top = derive_report(&export(rows), None).top;
        assert!(!top.largest_tokens.is_empty());
    }
}

// ── large input ──────────────────────────────────────────────────────

mod large_input {
    use super::*;

    #[test]
    fn thousand_files() {
        let rows: Vec<_> = (0..1000)
            .map(|i| {
                make_row(
                    &format!("src/f{i}.rs"),
                    "src",
                    "Rust",
                    100,
                    10,
                    5,
                    4600,
                    920,
                )
            })
            .collect();
        let r = derive_report(&export(rows), None);
        assert!(r.cocomo.is_some());
        assert!(r.distribution.count == 1000);
    }

    #[test]
    fn million_lines_single_file() {
        let rows = vec![make_row(
            "a.rs", "src", "Rust", 1_000_000, 100_000, 50_000, 46_000_000, 9_200_000,
        )];
        let r = derive_report(&export(rows), None);
        let c = r.cocomo.unwrap();
        assert!(c.effort_pm > 1000.0);
    }
}

// ── determinism / rounding ───────────────────────────────────────────

mod determinism_rounding {
    use super::*;

    #[test]
    fn derive_is_deterministic() {
        let rows = vec![
            make_row("src/a.rs", "src", "Rust", 150, 30, 20, 8000, 1600),
            make_row("src/b.py", "src", "Python", 80, 10, 5, 3800, 760),
            make_row("tests/t.rs", "tests", "Rust", 40, 5, 3, 1920, 384),
        ];
        let r1 = derive_report(&export(rows.clone()), None);
        let r2 = derive_report(&export(rows), None);
        assert_eq!(
            r1.cocomo.as_ref().map(|c| c.effort_pm),
            r2.cocomo.as_ref().map(|c| c.effort_pm)
        );
        assert_eq!(r1.integrity.hash, r2.integrity.hash);
    }

    #[test]
    fn rounding_no_nan_or_inf() {
        let edge_cases = vec![
            vec![make_row("a.rs", "src", "Rust", 1, 0, 0, 40, 8)],
            vec![make_row("a.rs", "src", "Rust", 0, 0, 1, 40, 8)],
        ];
        for rows in &edge_cases {
            let r = derive_report(&export(rows.clone()), None);
            assert!(!r.doc_density.total.ratio.is_nan());
            assert!(!r.doc_density.total.ratio.is_infinite());
        }
    }

    #[test]
    fn input_order_independence() {
        let rows_a = vec![
            make_row("z.rs", "src", "Rust", 100, 0, 0, 4000, 800),
            make_row("a.py", "src", "Python", 50, 0, 0, 2000, 400),
        ];
        let rows_b = vec![
            make_row("a.py", "src", "Python", 50, 0, 0, 2000, 400),
            make_row("z.rs", "src", "Rust", 100, 0, 0, 4000, 800),
        ];
        let ra = derive_report(&export(rows_a), None);
        let rb = derive_report(&export(rows_b), None);
        assert_eq!(
            ra.cocomo.as_ref().map(|c| c.effort_pm),
            rb.cocomo.as_ref().map(|c| c.effort_pm)
        );
    }
}

// ── extra edge-cases ─────────────────────────────────────────────────

mod edge_cases_extra {
    use super::*;

    #[test]
    fn unicode_path_names() {
        let rows = vec![make_row(
            "src/\u{65e5}\u{672c}\u{8a9e}.rs",
            "src",
            "Rust",
            100,
            10,
            5,
            4600,
            920,
        )];
        let r = derive_report(&export(rows), None);
        assert!(r.cocomo.is_some());
    }

    #[test]
    fn deeply_nested_path() {
        let rows = vec![make_row(
            "a/b/c/d/e/f/g/h/i/j/k/l/deep.rs",
            "a/b/c/d/e/f/g/h/i/j/k/l",
            "Rust",
            100,
            0,
            0,
            4000,
            800,
        )];
        let r = derive_report(&export(rows), None);
        assert!(r.nesting.max >= 10);
    }

    #[test]
    fn all_zeros_except_path() {
        let rows = vec![make_row("a.rs", "src", "Rust", 0, 0, 0, 0, 0)];
        let r = derive_report(&export(rows), None);
        assert!(r.cocomo.is_none());
    }

    #[test]
    fn duplicate_paths() {
        let rows = vec![
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
            make_row("a.rs", "src", "Rust", 50, 0, 0, 2000, 400),
        ];
        let r = derive_report(&export(rows), None);
        assert!(r.cocomo.is_some());
    }

    #[test]
    fn max_file_report() {
        let rows = vec![
            make_row("small.rs", "src", "Rust", 10, 0, 0, 400, 80),
            make_row("big.rs", "src", "Rust", 1000, 0, 0, 40000, 8000),
        ];
        let r = derive_report(&export(rows), None);
        assert_eq!(r.max_file.overall.code, 1000);
    }
}
