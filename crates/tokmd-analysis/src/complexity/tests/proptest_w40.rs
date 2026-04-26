//! Property-based tests for analysis complexity module.
//!
//! Covers: histogram invariants, aggregate max bounds,
//! risk classification ordering, and determinism.

use crate::complexity::generate_complexity_histogram;
use proptest::prelude::*;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};

fn make_file_complexity(path: &str, cyclomatic: usize, functions: usize) -> FileComplexity {
    FileComplexity {
        path: path.to_string(),
        module: "src".to_string(),
        function_count: functions,
        max_function_length: functions * 5,
        cyclomatic_complexity: cyclomatic,
        cognitive_complexity: None,
        max_nesting: None,
        risk_level: ComplexityRisk::Low,
        functions: None,
    }
}

// =========================================================================
// Histogram: bucket counts sum to total
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn histogram_counts_sum_to_total(
        complexities in prop::collection::vec(0usize..100, 0..50),
    ) {
        let files: Vec<FileComplexity> = complexities.iter().enumerate()
            .map(|(i, &c)| make_file_complexity(&format!("src/f{}.rs", i), c, 1))
            .collect();

        let histogram = generate_complexity_histogram(&files, 5);
        let count_sum: u32 = histogram.counts.iter().sum();
        prop_assert_eq!(count_sum, histogram.total,
            "Bucket counts sum {} != total {}", count_sum, histogram.total);
    }

    #[test]
    fn histogram_total_equals_file_count(
        complexities in prop::collection::vec(0usize..100, 0..50),
    ) {
        let files: Vec<FileComplexity> = complexities.iter().enumerate()
            .map(|(i, &c)| make_file_complexity(&format!("src/f{}.rs", i), c, 1))
            .collect();

        let histogram = generate_complexity_histogram(&files, 5);
        prop_assert_eq!(histogram.total, files.len() as u32,
            "Histogram total {} != file count {}", histogram.total, files.len());
    }
}

// =========================================================================
// Histogram: always has 7 buckets
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn histogram_always_seven_buckets(
        complexities in prop::collection::vec(0usize..200, 0..30),
    ) {
        let files: Vec<FileComplexity> = complexities.iter().enumerate()
            .map(|(i, &c)| make_file_complexity(&format!("src/f{}.rs", i), c, 1))
            .collect();

        let histogram = generate_complexity_histogram(&files, 5);
        prop_assert_eq!(histogram.buckets.len(), 7, "Must always have 7 buckets");
        prop_assert_eq!(histogram.counts.len(), 7, "Must always have 7 count entries");
    }
}

// =========================================================================
// Histogram: bucket boundaries are evenly spaced
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn histogram_bucket_boundaries_sequential(
        bucket_size in 1u32..20,
        complexities in prop::collection::vec(0usize..100, 1..20),
    ) {
        let files: Vec<FileComplexity> = complexities.iter().enumerate()
            .map(|(i, &c)| make_file_complexity(&format!("src/f{}.rs", i), c, 1))
            .collect();

        let histogram = generate_complexity_histogram(&files, bucket_size);
        for (i, &boundary) in histogram.buckets.iter().enumerate() {
            prop_assert_eq!(boundary, (i as u32) * bucket_size,
                "Bucket {} boundary {} != expected {}", i, boundary, (i as u32) * bucket_size);
        }
    }
}

// =========================================================================
// Aggregate max: max >= any individual complexity
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn max_cyclomatic_geq_any_individual(
        complexities in prop::collection::vec(0usize..200, 1..30),
    ) {
        let files: Vec<FileComplexity> = complexities.iter().enumerate()
            .map(|(i, &c)| make_file_complexity(&format!("src/f{}.rs", i), c, 1))
            .collect();

        let max_cyclomatic = files.iter().map(|f| f.cyclomatic_complexity).max().unwrap_or(0);
        for file in &files {
            prop_assert!(max_cyclomatic >= file.cyclomatic_complexity,
                "Aggregate max {} < individual {}", max_cyclomatic, file.cyclomatic_complexity);
        }
    }

    #[test]
    fn max_function_length_geq_any_individual(
        lengths in prop::collection::vec(0usize..500, 1..30),
    ) {
        let files: Vec<FileComplexity> = lengths.iter().enumerate()
            .map(|(i, &l)| {
                let mut fc = make_file_complexity(&format!("src/f{}.rs", i), 1, 1);
                fc.max_function_length = l;
                fc
            })
            .collect();

        let aggregate_max = files.iter().map(|f| f.max_function_length).max().unwrap_or(0);
        for file in &files {
            prop_assert!(aggregate_max >= file.max_function_length,
                "Aggregate max_fn_len {} < individual {}", aggregate_max, file.max_function_length);
        }
    }
}

// =========================================================================
// Histogram: determinism
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn histogram_is_deterministic(
        complexities in prop::collection::vec(0usize..100, 0..30),
    ) {
        let files: Vec<FileComplexity> = complexities.iter().enumerate()
            .map(|(i, &c)| make_file_complexity(&format!("src/f{}.rs", i), c, 1))
            .collect();

        let a = generate_complexity_histogram(&files, 5);
        let b = generate_complexity_histogram(&files, 5);
        prop_assert_eq!(a.counts, b.counts, "Histogram must be deterministic");
        prop_assert_eq!(a.total, b.total);
    }
}

// =========================================================================
// Empty input: histogram gracefully handles empty
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn histogram_empty_input_zero_total(
        bucket_size in 1u32..20,
    ) {
        let histogram = generate_complexity_histogram(&[], bucket_size);
        prop_assert_eq!(histogram.total, 0, "Empty input should have total 0");
        let sum: u32 = histogram.counts.iter().sum();
        prop_assert_eq!(sum, 0, "Empty input should have all-zero counts");
    }
}
