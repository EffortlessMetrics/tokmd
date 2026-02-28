//! Property-based tests for `tokmd-analysis-complexity`.
//!
//! Uses `proptest` to verify invariants that must hold for all inputs.

use proptest::prelude::*;

use tokmd_analysis_complexity::generate_complexity_histogram;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};

// ---------------------------------------------------------------------------
// Strategy: generate an arbitrary FileComplexity
// ---------------------------------------------------------------------------
fn arb_risk() -> impl Strategy<Value = ComplexityRisk> {
    prop_oneof![
        Just(ComplexityRisk::Low),
        Just(ComplexityRisk::Moderate),
        Just(ComplexityRisk::High),
        Just(ComplexityRisk::Critical),
    ]
}

fn arb_file_complexity() -> impl Strategy<Value = FileComplexity> {
    (
        "[a-z]{1,8}\\.rs",                 // path
        "[a-z]{1,5}",                      // module
        0..200usize,                       // function_count
        0..500usize,                       // max_function_length
        0..500usize,                       // cyclomatic_complexity
        proptest::option::of(0..300usize), // cognitive_complexity
        proptest::option::of(0..20usize),  // max_nesting
        arb_risk(),
    )
        .prop_map(
            |(path, module, fc, mfl, cc, cog, nest, risk)| FileComplexity {
                path,
                module,
                function_count: fc,
                max_function_length: mfl,
                cyclomatic_complexity: cc,
                cognitive_complexity: cog,
                max_nesting: nest,
                risk_level: risk,
                functions: None,
            },
        )
}

fn arb_file_vec() -> impl Strategy<Value = Vec<FileComplexity>> {
    proptest::collection::vec(arb_file_complexity(), 0..50)
}

// ===========================================================================
// Property: histogram total always equals number of input files
// ===========================================================================
proptest! {
    #[test]
    fn prop_histogram_total_equals_file_count(files in arb_file_vec()) {
        let hist = generate_complexity_histogram(&files, 5);
        prop_assert_eq!(hist.total, files.len() as u32);
    }
}

// ===========================================================================
// Property: sum of histogram counts equals total
// ===========================================================================
proptest! {
    #[test]
    fn prop_histogram_counts_sum_to_total(files in arb_file_vec()) {
        let hist = generate_complexity_histogram(&files, 5);
        let sum: u32 = hist.counts.iter().sum();
        prop_assert_eq!(sum, hist.total);
    }
}

// ===========================================================================
// Property: histogram always has 7 buckets for bucket_size = 5
// ===========================================================================
proptest! {
    #[test]
    fn prop_histogram_always_7_buckets(files in arb_file_vec()) {
        let hist = generate_complexity_histogram(&files, 5);
        prop_assert_eq!(hist.buckets.len(), 7);
        prop_assert_eq!(hist.counts.len(), 7);
    }
}

// ===========================================================================
// Property: bucket labels are monotonically increasing
// ===========================================================================
proptest! {
    #[test]
    fn prop_histogram_buckets_monotonic(
        files in arb_file_vec(),
        bucket_size in 1u32..20,
    ) {
        let hist = generate_complexity_histogram(&files, bucket_size);
        for window in hist.buckets.windows(2) {
            prop_assert!(window[0] < window[1], "buckets must be strictly increasing");
        }
    }
}

// ===========================================================================
// Property: empty input produces all-zero histogram
// ===========================================================================
proptest! {
    #[test]
    fn prop_histogram_empty_all_zeros(bucket_size in 1u32..100) {
        let hist = generate_complexity_histogram(&[], bucket_size);
        prop_assert_eq!(hist.total, 0);
        prop_assert!(hist.counts.iter().all(|&c| c == 0));
    }
}

// ===========================================================================
// Property: single file always appears in exactly one bucket
// ===========================================================================
proptest! {
    #[test]
    fn prop_single_file_one_bucket(file in arb_file_complexity()) {
        let hist = generate_complexity_histogram(&[file], 5);
        let nonzero_buckets = hist.counts.iter().filter(|&&c| c > 0).count();
        prop_assert_eq!(nonzero_buckets, 1, "single file must be in exactly one bucket");
        prop_assert_eq!(hist.total, 1);
    }
}

// ===========================================================================
// Property: histogram is deterministic (same input → same output)
// ===========================================================================
proptest! {
    #[test]
    fn prop_histogram_deterministic(files in arb_file_vec()) {
        let h1 = generate_complexity_histogram(&files, 5);
        let h2 = generate_complexity_histogram(&files, 5);
        prop_assert_eq!(h1.buckets, h2.buckets);
        prop_assert_eq!(h1.counts, h2.counts);
        prop_assert_eq!(h1.total, h2.total);
    }
}

// ===========================================================================
// Property: no bucket count exceeds total
// ===========================================================================
proptest! {
    #[test]
    fn prop_no_bucket_exceeds_total(files in arb_file_vec()) {
        let hist = generate_complexity_histogram(&files, 5);
        for (i, &count) in hist.counts.iter().enumerate() {
            prop_assert!(
                count <= hist.total,
                "bucket {} count {} exceeds total {}", i, count, hist.total,
            );
        }
    }
}

// ===========================================================================
// Property: bucket boundaries start at 0 and step by bucket_size
// ===========================================================================
proptest! {
    #[test]
    fn prop_bucket_boundaries_correct(bucket_size in 1u32..50) {
        let hist = generate_complexity_histogram(&[], bucket_size);
        for (i, &b) in hist.buckets.iter().enumerate() {
            prop_assert_eq!(b, (i as u32) * bucket_size);
        }
    }
}

// ===========================================================================
// Property: adding a file never decreases any bucket count
// ===========================================================================
proptest! {
    #[test]
    fn prop_adding_file_monotone(
        files in arb_file_vec(),
        extra in arb_file_complexity(),
    ) {
        let h_before = generate_complexity_histogram(&files, 5);
        let mut extended = files.clone();
        extended.push(extra);
        let h_after = generate_complexity_histogram(&extended, 5);

        prop_assert_eq!(h_after.total, h_before.total + 1);
        for (i, (&before, &after)) in h_before.counts.iter().zip(h_after.counts.iter()).enumerate() {
            prop_assert!(
                after >= before,
                "bucket {} decreased from {} to {}", i, before, after,
            );
        }
    }
}

// ===========================================================================
// Property: files with identical cyclomatic land in the same bucket
// ===========================================================================
proptest! {
    #[test]
    fn prop_same_cyclomatic_same_bucket(
        cyclo in 0..200usize,
        count in 1..10usize,
    ) {
        let files: Vec<FileComplexity> = (0..count)
            .map(|i| {
                FileComplexity {
                    path: format!("f{i}.rs"),
                    module: "src".to_string(),
                    function_count: 1,
                    max_function_length: 5,
                    cyclomatic_complexity: cyclo,
                    cognitive_complexity: None,
                    max_nesting: None,
                    risk_level: ComplexityRisk::Low,
                    functions: None,
                }
            })
            .collect();

        let hist = generate_complexity_histogram(&files, 5);
        let nonzero: Vec<usize> = hist.counts.iter().enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(i, _)| i)
            .collect();

        prop_assert_eq!(nonzero.len(), 1, "all files with same cyclomatic should be in one bucket");
        prop_assert_eq!(hist.counts[nonzero[0]], count as u32);
    }
}
