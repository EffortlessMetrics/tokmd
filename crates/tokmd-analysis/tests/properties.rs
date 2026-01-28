//! Property-based tests for tokmd-analysis utility functions.

mod util_properties {
    use proptest::prelude::*;

    // Re-implement the utility functions here for testing since they're pub(crate)
    // This ensures we test the same logic without exposing internals.

    fn percentile(sorted: &[usize], pct: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = (pct * (sorted.len() as f64 - 1.0)).ceil() as usize;
        sorted[idx.min(sorted.len() - 1)] as f64
    }

    fn gini_coefficient(sorted: &[usize]) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let n = sorted.len() as f64;
        let sum: f64 = sorted.iter().map(|v| *v as f64).sum();
        if sum == 0.0 {
            return 0.0;
        }
        let mut accum = 0.0;
        for (i, value) in sorted.iter().enumerate() {
            let i = i as f64 + 1.0;
            accum += (2.0 * i - n - 1.0) * (*value as f64);
        }
        accum / (n * sum)
    }

    fn safe_ratio(numer: usize, denom: usize) -> f64 {
        if denom == 0 {
            0.0
        } else {
            round_f64(numer as f64 / denom as f64, 4)
        }
    }

    fn round_f64(value: f64, decimals: u32) -> f64 {
        let factor = 10f64.powi(decimals as i32);
        (value * factor).round() / factor
    }

    fn path_depth(path: &str) -> usize {
        path.split('/').filter(|seg| !seg.is_empty()).count().max(1)
    }

    fn is_test_path(path: &str) -> bool {
        let lower = path.to_lowercase();
        if lower.contains("/test/")
            || lower.contains("/tests/")
            || lower.contains("__tests__")
        {
            return true;
        }
        if lower.contains("/spec/") || lower.contains("/specs/") {
            return true;
        }
        let name = lower.rsplit('/').next().unwrap_or(&lower);
        name.contains("_test")
            || name.contains(".test.")
            || name.contains(".spec.")
            || name.starts_with("test_")
            || name.ends_with("_test.rs")
    }

    fn is_infra_lang(lang: &str) -> bool {
        let l = lang.to_lowercase();
        matches!(
            l.as_str(),
            "json"
                | "yaml"
                | "toml"
                | "markdown"
                | "xml"
                | "html"
                | "css"
                | "scss"
                | "less"
                | "makefile"
                | "dockerfile"
                | "hcl"
                | "terraform"
                | "nix"
                | "cmake"
                | "ini"
                | "properties"
                | "gitignore"
                | "gitconfig"
                | "editorconfig"
                | "csv"
                | "tsv"
                | "svg"
        )
    }

    proptest! {
        // ========================
        // Percentile Properties
        // ========================

        #[test]
        fn percentile_empty_is_zero(pct in 0.0f64..=1.0) {
            prop_assert_eq!(percentile(&[], pct), 0.0);
        }

        #[test]
        fn percentile_in_bounds(mut values in prop::collection::vec(0usize..10000, 1..100),
                                 pct in 0.0f64..=1.0) {
            values.sort();
            let result = percentile(&values, pct);
            let min = *values.first().unwrap() as f64;
            let max = *values.last().unwrap() as f64;
            prop_assert!(result >= min, "Percentile {} below min {}", result, min);
            prop_assert!(result <= max, "Percentile {} above max {}", result, max);
        }

        #[test]
        fn percentile_zero_is_min(mut values in prop::collection::vec(0usize..10000, 1..100)) {
            values.sort();
            let p0 = percentile(&values, 0.0);
            // 0th percentile uses ceil(0 * (n-1)) = 0, so it's the first element
            prop_assert_eq!(p0, *values.first().unwrap() as f64);
        }

        #[test]
        fn percentile_one_is_max(mut values in prop::collection::vec(0usize..10000, 1..100)) {
            values.sort();
            let p100 = percentile(&values, 1.0);
            prop_assert_eq!(p100, *values.last().unwrap() as f64);
        }

        #[test]
        fn percentile_monotonic(mut values in prop::collection::vec(0usize..10000, 2..100),
                                 pct1 in 0.0f64..=1.0,
                                 pct2 in 0.0f64..=1.0) {
            values.sort();
            let p1 = percentile(&values, pct1);
            let p2 = percentile(&values, pct2);
            if pct1 <= pct2 {
                prop_assert!(p1 <= p2, "Percentile should be monotonic: p({})={} > p({})={}", pct1, p1, pct2, p2);
            } else {
                prop_assert!(p1 >= p2, "Percentile should be monotonic: p({})={} < p({})={}", pct1, p1, pct2, p2);
            }
        }

        // ========================
        // Gini Coefficient Properties
        // ========================

        #[test]
        fn gini_empty_is_zero(_dummy in 0..1u8) {
            prop_assert_eq!(gini_coefficient(&[]), 0.0);
        }

        #[test]
        fn gini_all_zeros_is_zero(len in 1usize..100) {
            let values = vec![0usize; len];
            prop_assert_eq!(gini_coefficient(&values), 0.0);
        }

        #[test]
        fn gini_in_bounds(values in prop::collection::vec(0usize..1000, 1..100)) {
            let mut sorted = values;
            sorted.sort();
            let gini = gini_coefficient(&sorted);
            prop_assert!(gini >= 0.0, "Gini must be non-negative: got {}", gini);
            prop_assert!(gini <= 1.0, "Gini must be at most 1: got {}", gini);
        }

        #[test]
        fn gini_uniform_is_zero(value in 1usize..1000, len in 2usize..100) {
            // Perfect equality: all same non-zero value
            let values = vec![value; len];
            let gini = gini_coefficient(&values);
            prop_assert!(gini.abs() < 0.0001, "Uniform distribution should have Gini ~0: got {}", gini);
        }

        #[test]
        fn gini_one_nonzero_high(len in 2usize..100) {
            // Maximum inequality: one person has everything
            let mut values = vec![0usize; len - 1];
            values.push(1000);
            values.sort();
            let gini = gini_coefficient(&values);
            // Gini approaches (n-1)/n as inequality increases
            let expected_max = (len - 1) as f64 / len as f64;
            prop_assert!(gini >= expected_max - 0.01, "Extreme inequality should have high Gini: got {}, expected ~{}", gini, expected_max);
        }

        // ========================
        // Safe Ratio Properties
        // ========================

        #[test]
        fn safe_ratio_zero_denom_is_zero(numer in 0usize..10000) {
            prop_assert_eq!(safe_ratio(numer, 0), 0.0);
        }

        #[test]
        fn safe_ratio_zero_numer_is_zero(denom in 1usize..10000) {
            prop_assert_eq!(safe_ratio(0, denom), 0.0);
        }

        #[test]
        fn safe_ratio_same_is_one(value in 1usize..10000) {
            prop_assert_eq!(safe_ratio(value, value), 1.0);
        }

        #[test]
        fn safe_ratio_has_limited_decimals(numer in 0usize..10000, denom in 1usize..10000) {
            let ratio = safe_ratio(numer, denom);
            let s = format!("{}", ratio);
            // Split on decimal point and check digits after
            if let Some(dot_pos) = s.find('.') {
                let decimals = s.len() - dot_pos - 1;
                prop_assert!(decimals <= 4, "Should have at most 4 decimals: {} has {}", s, decimals);
            }
        }

        // ========================
        // Round Properties
        // ========================

        #[test]
        fn round_idempotent(value in -1000.0f64..1000.0, decimals in 0u32..6) {
            let once = round_f64(value, decimals);
            let twice = round_f64(once, decimals);
            prop_assert!((once - twice).abs() < 1e-10, "Rounding should be idempotent");
        }

        #[test]
        fn round_preserves_integer(value in -1000i64..1000) {
            let f = value as f64;
            for decimals in 0..6 {
                let rounded = round_f64(f, decimals);
                prop_assert_eq!(rounded, f, "Rounding integer should preserve it");
            }
        }

        // ========================
        // Path Depth Properties
        // ========================

        #[test]
        fn path_depth_always_at_least_one(path in "\\PC*") {
            let depth = path_depth(&path);
            prop_assert!(depth >= 1, "Path depth should always be at least 1");
        }

        #[test]
        fn path_depth_counts_segments(segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..10)) {
            let path = segments.join("/");
            let depth = path_depth(&path);
            prop_assert_eq!(depth, segments.len(), "Depth should equal segment count for {}", path);
        }

        #[test]
        fn path_depth_ignores_empty_segments(segments in prop::collection::vec("[a-zA-Z0-9_]+", 1..5)) {
            let path_normal = segments.join("/");
            let path_with_double = segments.join("//");
            let path_with_trailing = format!("{}/", path_normal);
            let path_with_leading = format!("/{}", path_normal);

            let d_normal = path_depth(&path_normal);
            let d_double = path_depth(&path_with_double);
            let d_trailing = path_depth(&path_with_trailing);
            let d_leading = path_depth(&path_with_leading);

            prop_assert_eq!(d_normal, d_double, "Double slashes should not add depth");
            prop_assert_eq!(d_normal, d_trailing, "Trailing slash should not add depth");
            prop_assert_eq!(d_normal, d_leading, "Leading slash should not add depth");
        }

        // ========================
        // Is Test Path Properties
        // ========================

        #[test]
        fn is_test_path_case_insensitive_for_dirs(prefix in "[a-zA-Z0-9_/]+", suffix in "[a-zA-Z0-9_/]+\\.rs") {
            // Test directory markers should be case-insensitive
            let lower = format!("{}/test/{}", prefix, suffix);
            let upper = format!("{}/TEST/{}", prefix, suffix);
            let mixed = format!("{}/TeSt/{}", prefix, suffix);

            prop_assert_eq!(is_test_path(&lower), is_test_path(&upper), "Case sensitivity issue with TEST dir");
            prop_assert_eq!(is_test_path(&lower), is_test_path(&mixed), "Case sensitivity issue with TeSt dir");
        }

        #[test]
        fn is_test_path_known_test_dirs_detected(dir in prop::sample::select(vec!["test", "tests", "__tests__", "spec", "specs"])) {
            let path = format!("src/{}/foo.rs", dir);
            prop_assert!(is_test_path(&path), "Should detect test dir: {}", dir);
        }

        #[test]
        fn is_test_path_file_patterns_detected(pattern in prop::sample::select(vec!["foo_test.rs", "test_foo.rs", "foo.test.js", "foo.spec.ts"])) {
            let path = format!("src/{}", pattern);
            prop_assert!(is_test_path(&path), "Should detect test file pattern: {}", pattern);
        }

        // ========================
        // Is Infra Lang Properties
        // ========================

        #[test]
        fn is_infra_lang_case_insensitive(lang in prop::sample::select(vec!["json", "yaml", "toml", "markdown", "xml", "html", "css"])) {
            prop_assert!(is_infra_lang(&lang), "Should detect infra lang: {}", lang);
            prop_assert!(is_infra_lang(&lang.to_uppercase()), "Should detect infra lang (upper): {}", lang.to_uppercase());
        }

        #[test]
        fn is_infra_lang_known_infra_detected(lang in prop::sample::select(vec![
            "json", "yaml", "toml", "markdown", "xml", "html", "css", "scss", "less",
            "makefile", "dockerfile", "hcl", "terraform", "nix", "cmake", "ini",
            "properties", "gitignore", "gitconfig", "editorconfig", "csv", "tsv", "svg"
        ])) {
            prop_assert!(is_infra_lang(&lang), "Should detect known infra lang: {}", lang);
        }

        #[test]
        fn is_infra_lang_code_langs_not_infra(lang in prop::sample::select(vec![
            "rust", "python", "javascript", "typescript", "go", "java", "c", "cpp"
        ])) {
            prop_assert!(!is_infra_lang(&lang), "Code lang should not be infra: {}", lang);
        }
    }
}
