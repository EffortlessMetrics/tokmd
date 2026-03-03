//! Deep tests for tokmd-analysis-util
//!
//! Covers normalize_path, path_depth, is_test_path, is_infra_lang,
//! empty_file_row, analysis limits, normalize_root, now_ms,
//! round_f64, safe_ratio, percentile, and gini_coefficient.

use std::path::{Path, PathBuf};
use tokmd_analysis_util::*;

// ── normalize_path deep ──────────────────────────────────────────────

mod normalize_path_deep {
    use super::*;

    fn np(path: &str) -> String {
        normalize_path(path, &PathBuf::from("repo"))
    }

    #[test]
    fn forward_slashes_unchanged() {
        assert_eq!(np("src/main.rs"), "src/main.rs");
    }

    #[test]
    fn backslashes_converted() {
        assert_eq!(np("src\\main.rs"), "src/main.rs");
    }

    #[test]
    fn mixed_slashes() {
        assert_eq!(np("src\\lib/mod.rs"), "src/lib/mod.rs");
    }

    #[test]
    fn empty_string() {
        assert_eq!(np(""), "");
    }

    #[test]
    fn single_file_no_dir() {
        assert_eq!(np("file.rs"), "file.rs");
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(np("a\\b\\c\\d\\e\\f.rs"), "a/b/c/d/e/f.rs");
    }

    #[test]
    fn trailing_slash() {
        assert_eq!(np("src\\"), "src/");
    }

    #[test]
    fn double_backslash() {
        assert_eq!(np("src\\\\lib.rs"), "src//lib.rs");
    }

    #[test]
    fn dot_slash_stripped() {
        assert_eq!(np("./src/lib.rs"), "src/lib.rs");
    }

    #[test]
    fn dot_backslash_stripped() {
        assert_eq!(np(".\\src\\lib.rs"), "src/lib.rs");
    }

    #[test]
    fn unicode_path() {
        assert_eq!(
            np("src\\\u{65e5}\u{672c}\u{8a9e}\\file.rs"),
            "src/\u{65e5}\u{672c}\u{8a9e}/file.rs"
        );
    }

    #[test]
    fn only_slashes() {
        assert_eq!(np("\\\\\\"), "///");
    }
}

// ── path_depth deep ──────────────────────────────────────────────────

mod path_depth_deep {
    use super::*;

    #[test]
    fn root_file_depth_one() {
        // path_depth always returns >= 1
        assert_eq!(path_depth("file.rs"), 1);
    }

    #[test]
    fn one_level() {
        assert_eq!(path_depth("src/file.rs"), 2);
    }

    #[test]
    fn two_levels() {
        assert_eq!(path_depth("src/lib/file.rs"), 3);
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(path_depth("a/b/c/d/e/f/g.rs"), 7);
    }

    #[test]
    fn empty_string_at_least_one() {
        // empty string has max(0,1) = 1 in original; but split yields [""]
        // count of non-empty = 0, max(0,1) = 1
        assert_eq!(path_depth(""), 1);
    }

    #[test]
    fn trailing_slash_ignored() {
        // "src/" splits into ["src", ""], non-empty count = 1, max(1,1) = 1
        assert_eq!(path_depth("src/"), 1);
    }

    #[test]
    fn backslash_is_one_segment() {
        // backslash not split by path_depth (uses '/')
        assert_eq!(path_depth("src\\lib.rs"), 1);
    }

    #[test]
    fn multiple_dots() {
        assert_eq!(path_depth("a.b.c/d.e.f/g.rs"), 3);
    }

    #[test]
    fn leading_slash() {
        // "/file.rs" → ["", "file.rs"] → non-empty = 1
        assert_eq!(path_depth("/file.rs"), 1);
    }

    #[test]
    fn triple_slash() {
        // "///" → ["", "", "", ""] → non-empty = 0, max(0,1) = 1
        assert_eq!(path_depth("///"), 1);
    }

    #[test]
    fn unicode_segments() {
        assert_eq!(path_depth("\u{65e5}\u{672c}/\u{8a9e}/file.rs"), 3);
    }

    #[test]
    fn dot_dot_segments() {
        assert_eq!(path_depth("../src/file.rs"), 3);
    }
}

// ── is_test_path deep ────────────────────────────────────────────────

mod is_test_path_deep {
    use super::*;

    #[test]
    fn tests_dir() {
        assert!(is_test_path("src/tests/integration.rs"));
    }

    #[test]
    fn test_dir() {
        assert!(is_test_path("src/test/unit.rs"));
    }

    #[test]
    fn spec_dir() {
        assert!(is_test_path("src/spec/model_spec.rb"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn dunder_tests_dir() {
        assert!(is_test_path("__tests__/component.test.js"));
    }

    #[test]
    fn test_suffix() {
        assert!(is_test_path("src/lib_test.go"));
    }

    #[test]
    fn test_prefix() {
        assert!(is_test_path("src/test_parser.rs"));
    }

    #[test]
    fn spec_suffix() {
        assert!(is_test_path("app/models/user.spec.js"));
    }

    #[test]
    fn src_not_test() {
        assert!(!is_test_path("src/main.rs"));
    }

    #[test]
    fn lib_not_test() {
        assert!(!is_test_path("src/lib.rs"));
    }

    #[test]
    fn empty_path() {
        assert!(!is_test_path(""));
    }

    #[test]
    fn nested_test_dir() {
        assert!(is_test_path("crates/mylib/tests/deep.rs"));
    }

    #[test]
    fn dot_test_js() {
        assert!(is_test_path("src/component.test.js"));
    }

    #[test]
    fn dot_spec_ts() {
        assert!(is_test_path("src/component.spec.ts"));
    }

    #[test]
    fn pytest_file() {
        assert!(is_test_path("src/tests/test_main.py"));
    }

    #[test]
    fn test_helpers_in_tests_dir() {
        assert!(is_test_path("src/tests/helpers/setup.rs"));
    }

    #[test]
    fn case_insensitive_test_dir() {
        assert!(is_test_path("src/TEST/foo.rs"));
    }

    #[test]
    fn case_insensitive_tests_dir() {
        assert!(is_test_path("src/TESTS/foo.rs"));
    }
}

// ── is_infra_lang deep ───────────────────────────────────────────────

mod is_infra_lang_deep {
    use super::*;

    #[test]
    fn toml_is_infra() {
        assert!(is_infra_lang("TOML"));
    }

    #[test]
    fn yaml_is_infra() {
        assert!(is_infra_lang("YAML"));
    }

    #[test]
    fn json_is_infra() {
        assert!(is_infra_lang("JSON"));
    }

    #[test]
    fn dockerfile_is_infra() {
        assert!(is_infra_lang("Dockerfile"));
    }

    #[test]
    fn makefile_is_infra() {
        assert!(is_infra_lang("Makefile"));
    }

    #[test]
    fn markdown_is_infra() {
        assert!(is_infra_lang("Markdown"));
    }

    #[test]
    fn rust_not_infra() {
        assert!(!is_infra_lang("Rust"));
    }

    #[test]
    fn python_not_infra() {
        assert!(!is_infra_lang("Python"));
    }

    #[test]
    fn empty_not_infra() {
        assert!(!is_infra_lang(""));
    }

    #[test]
    fn case_insensitive_json() {
        assert!(is_infra_lang("json"));
        assert!(is_infra_lang("JSON"));
        assert!(is_infra_lang("Json"));
    }

    #[test]
    fn css_is_infra() {
        assert!(is_infra_lang("CSS"));
    }

    #[test]
    fn html_is_infra() {
        assert!(is_infra_lang("HTML"));
    }
}

// ── empty_file_row deep ─────────────────────────────────────────────

mod empty_file_row_deep {
    use super::*;

    #[test]
    fn all_counts_zero() {
        let r = empty_file_row();
        assert_eq!(r.code, 0);
        assert_eq!(r.comments, 0);
        assert_eq!(r.blanks, 0);
        assert_eq!(r.lines, 0);
        assert_eq!(r.bytes, 0);
        assert_eq!(r.tokens, 0);
    }

    #[test]
    fn path_is_empty() {
        let r = empty_file_row();
        assert!(r.path.is_empty());
    }

    #[test]
    fn lang_is_empty() {
        let r = empty_file_row();
        assert!(r.lang.is_empty());
    }

    #[test]
    fn module_is_empty() {
        let r = empty_file_row();
        assert!(r.module.is_empty());
    }

    #[test]
    fn depth_is_zero() {
        let r = empty_file_row();
        assert_eq!(r.depth, 0);
    }
}

// ── analysis limits deep ─────────────────────────────────────────────

mod analysis_limits_deep {
    use super::*;

    #[test]
    fn default_max_files_is_none() {
        let lim = AnalysisLimits::default();
        assert!(lim.max_files.is_none());
    }

    #[test]
    fn default_max_bytes_is_none() {
        let lim = AnalysisLimits::default();
        assert!(lim.max_bytes.is_none());
    }

    #[test]
    fn default_max_file_bytes_is_none() {
        let lim = AnalysisLimits::default();
        assert!(lim.max_file_bytes.is_none());
    }

    #[test]
    fn default_max_commits_is_none() {
        let lim = AnalysisLimits::default();
        assert!(lim.max_commits.is_none());
    }

    #[test]
    fn default_max_commit_files_is_none() {
        let lim = AnalysisLimits::default();
        assert!(lim.max_commit_files.is_none());
    }

    #[test]
    fn can_set_limits() {
        let lim = AnalysisLimits {
            max_files: Some(1000),
            max_bytes: Some(10_000_000),
            max_file_bytes: Some(1_000_000),
            max_commits: Some(500),
            max_commit_files: Some(100),
        };
        assert_eq!(lim.max_files, Some(1000));
        assert_eq!(lim.max_bytes, Some(10_000_000));
    }
}

// ── normalize_root deep ─────────────────────────────────────────────

mod normalize_root_deep {
    use super::*;

    #[test]
    fn returns_pathbuf() {
        let r = normalize_root(Path::new("src"));
        // normalize_root calls canonicalize, which may fail; fallback is identity
        assert!(!r.as_os_str().is_empty());
    }

    #[test]
    fn nonexistent_path_returns_input() {
        let input = Path::new("nonexistent_dir_xyz_12345");
        let r = normalize_root(input);
        assert_eq!(r, input.to_path_buf());
    }
}

// ── now_ms deep ──────────────────────────────────────────────────────

mod now_ms_deep {
    use super::*;

    #[test]
    fn returns_positive() {
        assert!(now_ms() > 0);
    }

    #[test]
    fn monotonic_ish() {
        let a = now_ms();
        let b = now_ms();
        assert!(b >= a);
    }

    #[test]
    fn reasonable_epoch() {
        // Should be after 2020-01-01 in milliseconds
        let ms_2020: u128 = 1_577_836_800_000;
        assert!(now_ms() > ms_2020);
    }
}

// ── round_f64 deep ───────────────────────────────────────────────────

mod round_f64_deep {
    use super::*;

    #[test]
    fn round_zero() {
        assert_eq!(round_f64(0.0, 2), 0.0);
    }

    #[test]
    fn round_positive() {
        assert_eq!(round_f64(1.2345, 2), 1.23);
    }

    #[test]
    fn round_up() {
        assert_eq!(round_f64(1.235, 2), 1.24);
    }

    #[test]
    fn round_negative() {
        assert_eq!(round_f64(-1.2345, 2), -1.23);
    }

    #[test]
    fn round_zero_decimals() {
        assert_eq!(round_f64(1.9, 0), 2.0);
    }

    #[test]
    fn round_four_decimals() {
        assert_eq!(round_f64(0.12345, 4), 0.1235);
    }

    #[test]
    fn round_already_rounded() {
        assert_eq!(round_f64(1.0, 2), 1.0);
    }

    #[test]
    fn round_very_small() {
        assert_eq!(round_f64(0.001, 2), 0.0);
    }

    #[test]
    fn round_large_number() {
        assert_eq!(round_f64(123456.789, 1), 123456.8);
    }
}

// ── safe_ratio deep ──────────────────────────────────────────────────

mod safe_ratio_deep {
    use super::*;

    #[test]
    fn normal_ratio() {
        assert_eq!(safe_ratio(1, 2), 0.5);
    }

    #[test]
    fn zero_denominator() {
        assert_eq!(safe_ratio(1, 0), 0.0);
    }

    #[test]
    fn zero_numerator() {
        assert_eq!(safe_ratio(0, 10), 0.0);
    }

    #[test]
    fn both_zero() {
        assert_eq!(safe_ratio(0, 0), 0.0);
    }

    #[test]
    fn ratio_one() {
        assert_eq!(safe_ratio(5, 5), 1.0);
    }

    #[test]
    fn rounded_to_four_decimals() {
        let r = safe_ratio(1, 3);
        assert_eq!(r, 0.3333);
    }

    #[test]
    fn large_values() {
        let r = safe_ratio(1_000_000, 3_000_000);
        assert_eq!(r, 0.3333);
    }

    #[test]
    fn quarter() {
        assert_eq!(safe_ratio(1, 4), 0.25);
    }
}

// ── percentile deep ──────────────────────────────────────────────────

mod percentile_deep {
    use super::*;

    #[test]
    fn p50_of_sorted() {
        let data: Vec<usize> = vec![1, 2, 3, 4, 5];
        assert_eq!(percentile(&data, 0.5), 3.0);
    }

    #[test]
    fn p0_returns_min() {
        let data: Vec<usize> = vec![10, 20, 30];
        assert_eq!(percentile(&data, 0.0), 10.0);
    }

    #[test]
    fn p100_returns_max() {
        let data: Vec<usize> = vec![10, 20, 30];
        assert_eq!(percentile(&data, 1.0), 30.0);
    }

    #[test]
    fn single_element() {
        assert_eq!(percentile(&[42usize], 0.5), 42.0);
    }

    #[test]
    fn empty_returns_zero() {
        let empty: &[usize] = &[];
        assert_eq!(percentile(empty, 0.5), 0.0);
    }

    #[test]
    fn p90_large_dataset() {
        let data: Vec<usize> = (1..=100).collect();
        let p = percentile(&data, 0.9);
        assert!(p >= 89.0 && p <= 100.0);
    }

    #[test]
    fn p25_quartile() {
        let data: Vec<usize> = (1..=100).collect();
        let p = percentile(&data, 0.25);
        assert!(p >= 24.0 && p <= 26.0);
    }
}

// ── gini_coefficient deep ────────────────────────────────────────────

mod gini_deep {
    use super::*;

    #[test]
    fn perfect_equality() {
        let data: Vec<usize> = vec![10, 10, 10, 10];
        let g = gini_coefficient(&data);
        assert!(g.abs() < 1e-10);
    }

    #[test]
    fn maximum_inequality() {
        let data: Vec<usize> = vec![0, 0, 0, 100];
        let g = gini_coefficient(&data);
        assert!(g > 0.7);
    }

    #[test]
    fn empty_returns_zero() {
        let empty: &[usize] = &[];
        assert_eq!(gini_coefficient(empty), 0.0);
    }

    #[test]
    fn single_element_zero() {
        assert_eq!(gini_coefficient(&[42usize]), 0.0);
    }

    #[test]
    fn two_equal() {
        assert_eq!(gini_coefficient(&[5usize, 5]), 0.0);
    }

    #[test]
    fn gini_range() {
        let data: Vec<usize> = vec![1, 2, 3, 4, 5];
        let g = gini_coefficient(&data);
        assert!(g >= 0.0 && g <= 1.0);
    }

    #[test]
    fn gini_moderate() {
        let data: Vec<usize> = vec![1, 1, 1, 1, 100];
        let g = gini_coefficient(&data);
        assert!(g > 0.3 && g < 1.0);
    }

    #[test]
    fn all_zeros() {
        assert_eq!(gini_coefficient(&[0usize, 0, 0]), 0.0);
    }

    #[test]
    fn gini_deterministic() {
        let data: Vec<usize> = vec![1, 5, 10, 20, 50];
        let g1 = gini_coefficient(&data);
        let g2 = gini_coefficient(&data);
        assert_eq!(g1, g2);
    }

    #[test]
    fn gini_symmetry() {
        // gini expects sorted input; same sorted data = same result
        let data: Vec<usize> = vec![1, 2, 3];
        let g1 = gini_coefficient(&data);
        let g2 = gini_coefficient(&data);
        assert_eq!(g1, g2);
    }
}
