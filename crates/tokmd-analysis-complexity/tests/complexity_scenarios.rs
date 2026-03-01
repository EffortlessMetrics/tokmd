//! Deeper scenario tests for complexity calculations.
//!
//! Covers determinism, edge cases, and known code patterns.

use tokmd_analysis_complexity::generate_complexity_histogram;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity, FunctionComplexityDetail};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------
fn file_cx(path: &str, cyclomatic: usize) -> FileComplexity {
    FileComplexity {
        path: path.to_string(),
        module: "src".to_string(),
        function_count: 1,
        max_function_length: 10,
        cyclomatic_complexity: cyclomatic,
        cognitive_complexity: None,
        max_nesting: None,
        risk_level: ComplexityRisk::Low,
        functions: None,
    }
}

fn file_full(
    path: &str,
    cyclomatic: usize,
    cognitive: Option<usize>,
    nesting: Option<usize>,
    fn_count: usize,
    fn_len: usize,
    risk: ComplexityRisk,
) -> FileComplexity {
    FileComplexity {
        path: path.to_string(),
        module: "src".to_string(),
        function_count: fn_count,
        max_function_length: fn_len,
        cyclomatic_complexity: cyclomatic,
        cognitive_complexity: cognitive,
        max_nesting: nesting,
        risk_level: risk,
        functions: None,
    }
}

// ===========================================================================
// Determinism: same input always produces the same histogram
// ===========================================================================

#[test]
fn determinism_histogram_identical_across_runs() {
    let files = vec![
        file_cx("a.rs", 3),
        file_cx("b.rs", 17),
        file_cx("c.rs", 42),
        file_cx("d.rs", 0),
        file_cx("e.rs", 9),
    ];
    let h1 = generate_complexity_histogram(&files, 5);
    let h2 = generate_complexity_histogram(&files, 5);
    let h3 = generate_complexity_histogram(&files, 5);

    assert_eq!(h1.buckets, h2.buckets);
    assert_eq!(h1.counts, h2.counts);
    assert_eq!(h1.total, h2.total);
    assert_eq!(h2.buckets, h3.buckets);
    assert_eq!(h2.counts, h3.counts);
    assert_eq!(h2.total, h3.total);
}

#[test]
fn determinism_histogram_with_varied_bucket_sizes() {
    let files = vec![file_cx("x.rs", 10), file_cx("y.rs", 25)];
    for bs in [1, 3, 5, 7, 10, 20] {
        let a = generate_complexity_histogram(&files, bs);
        let b = generate_complexity_histogram(&files, bs);
        assert_eq!(a.buckets, b.buckets, "bucket_size={bs}");
        assert_eq!(a.counts, b.counts, "bucket_size={bs}");
        assert_eq!(a.total, b.total, "bucket_size={bs}");
    }
}

// ===========================================================================
// Edge case: empty file list
// ===========================================================================

#[test]
fn edge_empty_files_produce_zero_histogram() {
    let hist = generate_complexity_histogram(&[], 5);
    assert_eq!(hist.total, 0);
    assert!(hist.counts.iter().all(|&c| c == 0));
    assert_eq!(hist.buckets.len(), 7);
}

// ===========================================================================
// Edge case: single file with zero complexity
// ===========================================================================

#[test]
fn edge_zero_complexity_lands_in_first_bucket() {
    let hist = generate_complexity_histogram(&[file_cx("empty.rs", 0)], 5);
    assert_eq!(hist.counts[0], 1);
    assert_eq!(hist.total, 1);
    assert_eq!(hist.counts[1..].iter().sum::<u32>(), 0);
}

// ===========================================================================
// Edge case: single file with exactly bucket boundary value
// ===========================================================================

#[test]
fn edge_exact_boundary_values() {
    // bucket_size=5: boundaries are 0,5,10,15,20,25,30
    // Value 5 → bucket index 1 (5/5=1)
    let hist = generate_complexity_histogram(&[file_cx("at5.rs", 5)], 5);
    assert_eq!(hist.counts[1], 1, "value 5 should land in bucket 1 (5-9)");

    // Value 4 → bucket index 0 (4/5=0)
    let hist = generate_complexity_histogram(&[file_cx("at4.rs", 4)], 5);
    assert_eq!(hist.counts[0], 1, "value 4 should land in bucket 0 (0-4)");

    // Value 30 → clamped to bucket 6 (last)
    let hist = generate_complexity_histogram(&[file_cx("at30.rs", 30)], 5);
    assert_eq!(hist.counts[6], 1, "value 30 should land in last bucket");
}

// ===========================================================================
// Edge case: very large complexity value
// ===========================================================================

#[test]
fn edge_very_large_complexity_clamped() {
    let hist = generate_complexity_histogram(&[file_cx("huge.rs", 10_000)], 5);
    assert_eq!(hist.counts[6], 1, "very large values clamp to last bucket");
    assert_eq!(hist.total, 1);
}

// ===========================================================================
// Known patterns: increasing complexity spreads across buckets
// ===========================================================================

#[test]
fn known_pattern_linear_spread() {
    let files: Vec<FileComplexity> = (0..35).map(|i| file_cx(&format!("f{i}.rs"), i)).collect();
    let hist = generate_complexity_histogram(&files, 5);

    assert_eq!(hist.total, 35);
    // Bucket 0 (0-4): values 0,1,2,3,4 → 5 files
    assert_eq!(hist.counts[0], 5);
    // Bucket 1 (5-9): values 5,6,7,8,9 → 5 files
    assert_eq!(hist.counts[1], 5);
    // Bucket 2 (10-14): 5 files
    assert_eq!(hist.counts[2], 5);
    // Bucket 3 (15-19): 5 files
    assert_eq!(hist.counts[3], 5);
    // Bucket 4 (20-24): 5 files
    assert_eq!(hist.counts[4], 5);
    // Bucket 5 (25-29): 5 files
    assert_eq!(hist.counts[5], 5);
    // Bucket 6 (30+): 30,31,32,33,34 → 5 files
    assert_eq!(hist.counts[6], 5);
}

// ===========================================================================
// Known pattern: all files in the same bucket
// ===========================================================================

#[test]
fn known_pattern_all_same_bucket() {
    let files: Vec<FileComplexity> = (0..20).map(|i| file_cx(&format!("f{i}.rs"), 3)).collect();
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.counts[0], 20);
    assert_eq!(hist.counts[1..].iter().sum::<u32>(), 0);
}

// ===========================================================================
// FileComplexity metadata does not affect histogram placement
// ===========================================================================

#[test]
fn metadata_cognitive_nesting_ignored_by_histogram() {
    let mut f = file_cx("rich.rs", 7);
    f.cognitive_complexity = Some(100);
    f.max_nesting = Some(10);
    f.function_count = 50;
    f.max_function_length = 500;
    f.risk_level = ComplexityRisk::Critical;

    let hist = generate_complexity_histogram(&[f], 5);
    // cyclomatic 7 → bucket 1 (5-9), regardless of other metadata
    assert_eq!(hist.counts[1], 1);
    assert_eq!(hist.total, 1);
}

#[test]
fn metadata_function_details_ignored_by_histogram() {
    let mut f = file_cx("detailed.rs", 12);
    f.functions = Some(vec![
        FunctionComplexityDetail {
            name: "fn_a".to_string(),
            line_start: 1,
            line_end: 50,
            length: 50,
            cyclomatic: 6,
            cognitive: Some(20),
            max_nesting: Some(4),
            param_count: Some(5),
        },
        FunctionComplexityDetail {
            name: "fn_b".to_string(),
            line_start: 52,
            line_end: 100,
            length: 49,
            cyclomatic: 6,
            cognitive: Some(15),
            max_nesting: Some(3),
            param_count: Some(2),
        },
    ]);

    let hist = generate_complexity_histogram(&[f], 5);
    // File-level cyclomatic 12 → bucket 2 (10-14)
    assert_eq!(hist.counts[2], 1);
}

// ===========================================================================
// Deeply nested code pattern: high complexity → high bucket
// ===========================================================================

#[test]
fn known_pattern_deeply_nested_high_bucket() {
    // Simulate a file with very high cyclomatic complexity from deep nesting
    let f = file_full(
        "nested.rs",
        45,
        Some(80),
        Some(8),
        3,
        200,
        ComplexityRisk::Critical,
    );
    let hist = generate_complexity_histogram(&[f], 5);
    assert_eq!(hist.counts[6], 1, "cyclomatic 45 → last bucket (30+)");
}

// ===========================================================================
// Single-line function pattern: minimal complexity
// ===========================================================================

#[test]
fn known_pattern_single_line_functions_low_complexity() {
    // Files with only single-line functions: minimal cyclomatic
    let files: Vec<FileComplexity> = (0..5)
        .map(|i| {
            file_full(
                &format!("oneliner{i}.rs"),
                1,
                Some(0),
                Some(0),
                1,
                1,
                ComplexityRisk::Low,
            )
        })
        .collect();
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.counts[0], 5, "cyclomatic 1 → bucket 0");
}

// ===========================================================================
// Histogram count sum invariant holds for varied inputs
// ===========================================================================

#[test]
fn invariant_sum_of_counts_equals_total() {
    let test_cases: Vec<Vec<FileComplexity>> = vec![
        vec![],
        vec![file_cx("a.rs", 0)],
        vec![file_cx("a.rs", 0), file_cx("b.rs", 50)],
        (0..100)
            .map(|i| file_cx(&format!("f{i}.rs"), i % 40))
            .collect(),
    ];

    for (idx, files) in test_cases.iter().enumerate() {
        let hist = generate_complexity_histogram(files, 5);
        assert_eq!(
            hist.counts.iter().sum::<u32>(),
            hist.total,
            "case {idx}: counts sum must equal total"
        );
        assert_eq!(
            hist.total,
            files.len() as u32,
            "case {idx}: total must equal file count"
        );
    }
}

// ===========================================================================
// Different risk levels with same cyclomatic land in same bucket
// ===========================================================================

#[test]
fn risk_levels_do_not_affect_bucket_placement() {
    let risks = [
        ComplexityRisk::Low,
        ComplexityRisk::Moderate,
        ComplexityRisk::High,
        ComplexityRisk::Critical,
    ];
    let files: Vec<FileComplexity> = risks
        .iter()
        .enumerate()
        .map(|(i, &risk)| {
            let mut f = file_cx(&format!("r{i}.rs"), 7);
            f.risk_level = risk;
            f
        })
        .collect();

    let hist = generate_complexity_histogram(&files, 5);
    // All have cyclomatic 7 → bucket 1 (5-9)
    assert_eq!(hist.counts[1], 4);
    assert_eq!(hist.total, 4);
}

// ===========================================================================
// Custom bucket sizes produce correct bucket boundaries
// ===========================================================================

#[test]
fn custom_bucket_size_boundaries() {
    for bs in [1u32, 2, 3, 5, 10, 15, 20] {
        let hist = generate_complexity_histogram(&[], bs);
        assert_eq!(hist.buckets.len(), 7);
        for (i, &b) in hist.buckets.iter().enumerate() {
            assert_eq!(b, (i as u32) * bs, "bucket_size={bs}, index={i}");
        }
    }
}

// ===========================================================================
// Mixed modules don't affect histogram
// ===========================================================================

#[test]
fn mixed_modules_all_counted() {
    let mut files = vec![
        file_cx("src/a.rs", 2),
        file_cx("lib/b.rs", 8),
        file_cx("tests/c.rs", 15),
        file_cx("benches/d.rs", 32),
    ];
    files[0].module = "src".to_string();
    files[1].module = "lib".to_string();
    files[2].module = "tests".to_string();
    files[3].module = "benches".to_string();

    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.total, 4);
    assert_eq!(hist.counts.iter().sum::<u32>(), 4);
}
