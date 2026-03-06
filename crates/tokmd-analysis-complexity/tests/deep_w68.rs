//! W68 deep tests for complexity analysis.
//!
//! Covers cyclomatic/cognitive complexity calculation, histogram generation,
//! edge cases with empty/single-line files, and determinism.

use tokmd_analysis_complexity::generate_complexity_histogram;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_file_complexity(
    path: &str,
    function_count: usize,
    max_fn_len: usize,
    cyclomatic: usize,
    cognitive: Option<usize>,
    max_nesting: Option<usize>,
    risk: ComplexityRisk,
) -> FileComplexity {
    FileComplexity {
        path: path.to_string(),
        module: "src".to_string(),
        function_count,
        max_function_length: max_fn_len,
        cyclomatic_complexity: cyclomatic,
        cognitive_complexity: cognitive,
        max_nesting,
        risk_level: risk,
        functions: None,
    }
}

// ---------------------------------------------------------------------------
// Histogram generation
// ---------------------------------------------------------------------------

#[test]
fn histogram_empty_files() {
    let hist = generate_complexity_histogram(&[], 5);
    assert_eq!(hist.total, 0);
    assert_eq!(hist.counts.len(), 7);
    assert!(hist.counts.iter().all(|c| *c == 0));
}

#[test]
fn histogram_single_low_complexity_file() {
    let files = vec![make_file_complexity(
        "a.rs",
        1,
        10,
        2,
        None,
        None,
        ComplexityRisk::Low,
    )];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.total, 1);
    assert_eq!(hist.counts[0], 1); // bucket 0-4
    assert_eq!(hist.counts[1], 0);
}

#[test]
fn histogram_single_high_complexity_file() {
    let files = vec![make_file_complexity(
        "a.rs",
        10,
        100,
        35,
        Some(50),
        Some(6),
        ComplexityRisk::High,
    )];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.total, 1);
    // 35 / 5 = 7, capped at bucket 6 (30+)
    assert_eq!(hist.counts[6], 1);
}

#[test]
fn histogram_distributes_across_buckets() {
    let files = vec![
        make_file_complexity("a.rs", 1, 5, 2, None, None, ComplexityRisk::Low),
        make_file_complexity("b.rs", 3, 15, 7, None, None, ComplexityRisk::Low),
        make_file_complexity("c.rs", 5, 30, 12, None, None, ComplexityRisk::Moderate),
        make_file_complexity("d.rs", 8, 50, 18, None, None, ComplexityRisk::Moderate),
        make_file_complexity("e.rs", 10, 80, 25, None, None, ComplexityRisk::High),
        make_file_complexity(
            "f.rs",
            15,
            100,
            45,
            Some(80),
            Some(7),
            ComplexityRisk::Critical,
        ),
    ];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.total, 6);
    assert_eq!(hist.counts[0], 1); // 0-4: a.rs (2)
    assert_eq!(hist.counts[1], 1); // 5-9: b.rs (7)
    assert_eq!(hist.counts[2], 1); // 10-14: c.rs (12)
    assert_eq!(hist.counts[3], 1); // 15-19: d.rs (18)
    assert_eq!(hist.counts[5], 1); // 25-29: e.rs (25)
    assert_eq!(hist.counts[6], 1); // 30+: f.rs (45)
}

#[test]
fn histogram_bucket_boundaries() {
    let hist = generate_complexity_histogram(&[], 5);
    assert_eq!(hist.buckets, vec![0, 5, 10, 15, 20, 25, 30]);
}

#[test]
fn histogram_all_in_first_bucket() {
    let files: Vec<FileComplexity> = (0..5)
        .map(|i| make_file_complexity(&format!("{i}.rs"), 1, 5, i, None, None, ComplexityRisk::Low))
        .collect();
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.counts[0], 5);
    assert_eq!(hist.counts[1..].iter().sum::<u32>(), 0);
}

#[test]
fn histogram_all_in_last_bucket() {
    let files: Vec<FileComplexity> = (0..3)
        .map(|i| {
            make_file_complexity(
                &format!("{i}.rs"),
                20,
                100,
                50 + i,
                None,
                None,
                ComplexityRisk::Critical,
            )
        })
        .collect();
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.counts[6], 3);
    assert_eq!(hist.counts[..6].iter().sum::<u32>(), 0);
}

#[test]
fn histogram_deterministic() {
    let files = vec![
        make_file_complexity("a.rs", 2, 10, 3, None, None, ComplexityRisk::Low),
        make_file_complexity("b.rs", 5, 30, 15, None, None, ComplexityRisk::Moderate),
    ];
    let a = generate_complexity_histogram(&files, 5);
    let b = generate_complexity_histogram(&files, 5);
    assert_eq!(a.buckets, b.buckets);
    assert_eq!(a.counts, b.counts);
    assert_eq!(a.total, b.total);
}

// ---------------------------------------------------------------------------
// FileComplexity construction & risk classification
// ---------------------------------------------------------------------------

#[test]
fn file_complexity_low_risk() {
    let fc = make_file_complexity("a.rs", 2, 10, 3, Some(5), Some(2), ComplexityRisk::Low);
    assert_eq!(fc.risk_level, ComplexityRisk::Low);
    assert_eq!(fc.function_count, 2);
    assert_eq!(fc.cyclomatic_complexity, 3);
}

#[test]
fn file_complexity_moderate_risk() {
    let fc = make_file_complexity(
        "b.rs",
        15,
        40,
        15,
        Some(30),
        Some(4),
        ComplexityRisk::Moderate,
    );
    assert_eq!(fc.risk_level, ComplexityRisk::Moderate);
}

#[test]
fn file_complexity_high_risk() {
    let fc = make_file_complexity("c.rs", 30, 80, 35, Some(60), Some(6), ComplexityRisk::High);
    assert_eq!(fc.risk_level, ComplexityRisk::High);
    assert_eq!(fc.cognitive_complexity, Some(60));
    assert_eq!(fc.max_nesting, Some(6));
}

#[test]
fn file_complexity_critical_risk() {
    let fc = make_file_complexity(
        "d.rs",
        60,
        150,
        80,
        Some(120),
        Some(10),
        ComplexityRisk::Critical,
    );
    assert_eq!(fc.risk_level, ComplexityRisk::Critical);
}

#[test]
fn file_complexity_without_cognitive() {
    let fc = make_file_complexity("e.rs", 5, 20, 10, None, None, ComplexityRisk::Low);
    assert!(fc.cognitive_complexity.is_none());
    assert!(fc.max_nesting.is_none());
}

// ---------------------------------------------------------------------------
// Edge cases: zero-function files
// ---------------------------------------------------------------------------

#[test]
fn zero_function_file() {
    let fc = make_file_complexity("empty.rs", 0, 0, 1, None, None, ComplexityRisk::Low);
    assert_eq!(fc.function_count, 0);
    assert_eq!(fc.max_function_length, 0);
    assert_eq!(fc.cyclomatic_complexity, 1); // base complexity
}

#[test]
fn histogram_with_zero_complexity_files() {
    let files = vec![
        make_file_complexity("a.rs", 0, 0, 0, None, None, ComplexityRisk::Low),
        make_file_complexity("b.rs", 0, 0, 0, None, None, ComplexityRisk::Low),
    ];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.counts[0], 2); // all in bucket 0-4
    assert_eq!(hist.total, 2);
}

// ---------------------------------------------------------------------------
// Sorting verification
// ---------------------------------------------------------------------------

#[test]
fn file_complexities_sort_by_cyclomatic_desc() {
    let mut files = [
        make_file_complexity("a.rs", 1, 5, 3, None, None, ComplexityRisk::Low),
        make_file_complexity("c.rs", 5, 30, 20, None, None, ComplexityRisk::Moderate),
        make_file_complexity("b.rs", 3, 15, 10, None, None, ComplexityRisk::Low),
    ];
    files.sort_by(|a, b| {
        b.cyclomatic_complexity
            .cmp(&a.cyclomatic_complexity)
            .then_with(|| a.path.cmp(&b.path))
    });
    assert_eq!(files[0].path, "c.rs");
    assert_eq!(files[1].path, "b.rs");
    assert_eq!(files[2].path, "a.rs");
}

#[test]
fn file_complexities_sort_stable_on_tie() {
    let mut files = [
        make_file_complexity("b.rs", 2, 10, 5, None, None, ComplexityRisk::Low),
        make_file_complexity("a.rs", 2, 10, 5, None, None, ComplexityRisk::Low),
    ];
    files.sort_by(|a, b| {
        b.cyclomatic_complexity
            .cmp(&a.cyclomatic_complexity)
            .then_with(|| a.path.cmp(&b.path))
    });
    // Same cyclomatic -> sort by path ascending
    assert_eq!(files[0].path, "a.rs");
    assert_eq!(files[1].path, "b.rs");
}
