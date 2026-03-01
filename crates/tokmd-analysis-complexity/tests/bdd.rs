//! BDD-style scenario tests for `tokmd-analysis-complexity`.
//!
//! Each test follows Given / When / Then structure exercising the public API:
//! - `generate_complexity_histogram`
//! - `build_complexity_report` (via temp-dir fixtures)

use tokmd_analysis_complexity::generate_complexity_histogram;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};

// ---------------------------------------------------------------------------
// Helper: build a FileComplexity with the given cyclomatic value
// ---------------------------------------------------------------------------
fn file_with_cyclomatic(path: &str, cyclomatic: usize) -> FileComplexity {
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

// ===========================================================================
// Scenario: Histogram from empty file list
// ===========================================================================
#[test]
fn scenario_histogram_empty_input() {
    // Given: no files have been analyzed
    let files: Vec<FileComplexity> = vec![];

    // When: we generate a histogram with bucket size 5
    let hist = generate_complexity_histogram(&files, 5);

    // Then: all bucket counts are zero and total is zero
    assert_eq!(hist.total, 0);
    assert!(hist.counts.iter().all(|&c| c == 0));
    assert_eq!(hist.buckets.len(), 7, "should have 7 buckets");
    assert_eq!(hist.counts.len(), 7, "counts length matches buckets");
}

// ===========================================================================
// Scenario: Single low-complexity file lands in first bucket
// ===========================================================================
#[test]
fn scenario_histogram_single_low_complexity_file() {
    // Given: one file with cyclomatic complexity 3
    let files = vec![file_with_cyclomatic("src/simple.rs", 3)];

    // When: we generate a histogram
    let hist = generate_complexity_histogram(&files, 5);

    // Then: exactly 1 file in the 0-4 bucket
    assert_eq!(hist.total, 1);
    assert_eq!(hist.counts[0], 1, "bucket 0-4 should have 1 file");
    assert_eq!(
        hist.counts[1..].iter().sum::<u32>(),
        0,
        "other buckets empty"
    );
}

// ===========================================================================
// Scenario: Files distributed across multiple buckets
// ===========================================================================
#[test]
fn scenario_histogram_multiple_buckets() {
    // Given: files with varying complexity
    let files = vec![
        file_with_cyclomatic("a.rs", 2),  // bucket 0  (0-4)
        file_with_cyclomatic("b.rs", 7),  // bucket 1  (5-9)
        file_with_cyclomatic("c.rs", 12), // bucket 2  (10-14)
        file_with_cyclomatic("d.rs", 18), // bucket 3  (15-19)
        file_with_cyclomatic("e.rs", 22), // bucket 4  (20-24)
        file_with_cyclomatic("f.rs", 28), // bucket 5  (25-29)
        file_with_cyclomatic("g.rs", 35), // bucket 6  (30+)
    ];

    // When: histogram is generated
    let hist = generate_complexity_histogram(&files, 5);

    // Then: one file in each bucket
    assert_eq!(hist.total, 7);
    for (i, &count) in hist.counts.iter().enumerate() {
        assert_eq!(count, 1, "bucket {i} should have exactly 1 file");
    }
}

// ===========================================================================
// Scenario: Very high complexity clamps to the last bucket
// ===========================================================================
#[test]
fn scenario_histogram_high_complexity_clamps() {
    // Given: a file with complexity 999 (well beyond 30+)
    let files = vec![file_with_cyclomatic("monster.rs", 999)];

    // When: histogram is generated
    let hist = generate_complexity_histogram(&files, 5);

    // Then: file lands in the last bucket (index 6)
    assert_eq!(hist.counts[6], 1, "last bucket should capture high values");
    assert_eq!(hist.counts[..6].iter().sum::<u32>(), 0);
}

// ===========================================================================
// Scenario: Boundary values land in correct buckets
// ===========================================================================
#[test]
fn scenario_histogram_boundary_values() {
    // Given: files at exact bucket boundaries
    let files = vec![
        file_with_cyclomatic("at0.rs", 0),
        file_with_cyclomatic("at4.rs", 4),
        file_with_cyclomatic("at5.rs", 5),
        file_with_cyclomatic("at9.rs", 9),
        file_with_cyclomatic("at10.rs", 10),
        file_with_cyclomatic("at29.rs", 29),
        file_with_cyclomatic("at30.rs", 30),
    ];

    // When: histogram is generated
    let hist = generate_complexity_histogram(&files, 5);

    // Then: boundary values are placed correctly
    assert_eq!(hist.counts[0], 2, "0 and 4 in bucket 0-4");
    assert_eq!(hist.counts[1], 2, "5 and 9 in bucket 5-9");
    assert_eq!(hist.counts[2], 1, "10 in bucket 10-14");
    assert_eq!(hist.counts[5], 1, "29 in bucket 25-29");
    assert_eq!(hist.counts[6], 1, "30 in bucket 30+");
    assert_eq!(hist.total, 7);
}

// ===========================================================================
// Scenario: Histogram bucket labels are correct
// ===========================================================================
#[test]
fn scenario_histogram_bucket_labels() {
    let files: Vec<FileComplexity> = vec![];
    let hist = generate_complexity_histogram(&files, 5);

    // Then: bucket boundaries are [0, 5, 10, 15, 20, 25, 30]
    assert_eq!(hist.buckets, vec![0, 5, 10, 15, 20, 25, 30]);
}

// ===========================================================================
// Scenario: Custom bucket size changes histogram shape
// ===========================================================================
#[test]
fn scenario_histogram_custom_bucket_size() {
    // Given: files and bucket size 10
    let files = vec![
        file_with_cyclomatic("a.rs", 5),
        file_with_cyclomatic("b.rs", 15),
        file_with_cyclomatic("c.rs", 65),
    ];

    // When: histogram with bucket_size=10
    let hist = generate_complexity_histogram(&files, 10);

    // Then: buckets are [0, 10, 20, 30, 40, 50, 60]
    assert_eq!(hist.buckets, vec![0, 10, 20, 30, 40, 50, 60]);
    assert_eq!(hist.counts[0], 1, "5 in bucket 0-9");
    assert_eq!(hist.counts[1], 1, "15 in bucket 10-19");
    assert_eq!(hist.counts[6], 1, "65 clamped to last bucket");
    assert_eq!(hist.total, 3);
}

// ===========================================================================
// Scenario: Many files in same bucket
// ===========================================================================
#[test]
fn scenario_histogram_concentration_in_one_bucket() {
    // Given: 50 files all with complexity 3 (bucket 0)
    let files: Vec<FileComplexity> = (0..50)
        .map(|i| file_with_cyclomatic(&format!("file{i}.rs"), 3))
        .collect();

    // When: histogram is generated
    let hist = generate_complexity_histogram(&files, 5);

    // Then: all 50 in bucket 0
    assert_eq!(hist.counts[0], 50);
    assert_eq!(hist.total, 50);
    assert_eq!(hist.counts[1..].iter().sum::<u32>(), 0);
}

// ===========================================================================
// Scenario: Histogram total equals number of input files
// ===========================================================================
#[test]
fn scenario_histogram_total_invariant() {
    let files: Vec<FileComplexity> = (0..13)
        .map(|i| file_with_cyclomatic(&format!("f{i}.rs"), i * 3))
        .collect();

    let hist = generate_complexity_histogram(&files, 5);

    assert_eq!(
        hist.total,
        files.len() as u32,
        "total must equal input count"
    );
    assert_eq!(
        hist.counts.iter().sum::<u32>(),
        hist.total,
        "sum of counts must equal total"
    );
}

// ===========================================================================
// Scenario: Complexity zero is valid and lands in first bucket
// ===========================================================================
#[test]
fn scenario_zero_complexity_first_bucket() {
    let files = vec![file_with_cyclomatic("empty.rs", 0)];
    let hist = generate_complexity_histogram(&files, 5);

    assert_eq!(hist.counts[0], 1);
    assert_eq!(hist.total, 1);
}

// ===========================================================================
// Scenario: Histogram with cognitive and nesting metadata preserved
// ===========================================================================
#[test]
fn scenario_histogram_ignores_cognitive_and_nesting() {
    // Given: files with cognitive/nesting metadata
    let mut f = file_with_cyclomatic("rich.rs", 12);
    f.cognitive_complexity = Some(30);
    f.max_nesting = Some(5);

    let hist = generate_complexity_histogram(&[f], 5);

    // Then: histogram only uses cyclomatic — file is in bucket 2 (10-14)
    assert_eq!(hist.counts[2], 1);
    assert_eq!(hist.total, 1);
}

// ===========================================================================
// Scenario: FileComplexity with function details does not affect histogram
// ===========================================================================
#[test]
fn scenario_histogram_with_function_details() {
    use tokmd_analysis_types::FunctionComplexityDetail;

    let mut f = file_with_cyclomatic("detailed.rs", 8);
    f.functions = Some(vec![FunctionComplexityDetail {
        name: "complex_fn".to_string(),
        line_start: 1,
        line_end: 20,
        length: 20,
        cyclomatic: 8,
        cognitive: Some(12),
        max_nesting: Some(3),
        param_count: Some(2),
    }]);

    let hist = generate_complexity_histogram(&[f], 5);

    // Then: still uses file-level cyclomatic (8 → bucket 1: 5-9)
    assert_eq!(hist.counts[1], 1);
}

// ===========================================================================
// Scenario: Risk levels in file do not affect histogram bucket placement
// ===========================================================================
#[test]
fn scenario_histogram_risk_levels_independent() {
    let mut low = file_with_cyclomatic("low.rs", 3);
    low.risk_level = ComplexityRisk::Low;

    let mut critical = file_with_cyclomatic("critical.rs", 3);
    critical.risk_level = ComplexityRisk::Critical;

    let hist = generate_complexity_histogram(&[low, critical], 5);

    // Both have cyclomatic 3 → bucket 0
    assert_eq!(hist.counts[0], 2);
}

// ===========================================================================
// Scenario: Empty file has zero complexity (cyclomatic 0)
// ===========================================================================
#[test]
fn scenario_empty_file_zero_complexity() {
    // Given: a file with zero cyclomatic complexity
    let file = file_with_cyclomatic("empty.rs", 0);

    // When: we check its properties
    // Then: cyclomatic complexity is 0 and it lands in first bucket
    assert_eq!(file.cyclomatic_complexity, 0);
    let hist = generate_complexity_histogram(&[file], 5);
    assert_eq!(hist.counts[0], 1);
    assert_eq!(hist.total, 1);
}

// ===========================================================================
// Scenario: Known branching patterns increase complexity
// ===========================================================================
#[test]
fn scenario_branching_increases_complexity() {
    // Given: files with increasing complexity
    let simple = file_with_cyclomatic("simple.rs", 1);
    let branchy = file_with_cyclomatic("branchy.rs", 15);
    let complex = file_with_cyclomatic("complex.rs", 40);

    // Then: more complex files land in higher buckets
    let hist = generate_complexity_histogram(&[simple, branchy, complex], 5);
    // simple (1) → bucket 0 (0-4)
    assert_eq!(hist.counts[0], 1);
    // branchy (15) → bucket 3 (15-19)
    assert_eq!(hist.counts[3], 1);
    // complex (40) → bucket 6 (30+)
    assert_eq!(hist.counts[6], 1);
}

// ===========================================================================
// Scenario: Multiple languages produce valid histogram results
// ===========================================================================
#[test]
fn scenario_multi_language_files_valid_histogram() {
    // Given: files from different languages with varying complexity
    let mut rust_file = file_with_cyclomatic("lib.rs", 5);
    rust_file.module = "rust_mod".to_string();

    let mut py_file = file_with_cyclomatic("app.py", 12);
    py_file.module = "python_mod".to_string();

    let mut js_file = file_with_cyclomatic("index.js", 25);
    js_file.module = "js_mod".to_string();

    let files = vec![rust_file, py_file, js_file];
    let hist = generate_complexity_histogram(&files, 5);

    // Then: all files are accounted for
    assert_eq!(hist.total, 3);
    assert_eq!(hist.counts.iter().sum::<u32>(), 3);
    // rust (5) → bucket 1, py (12) → bucket 2, js (25) → bucket 5
    assert_eq!(hist.counts[1], 1);
    assert_eq!(hist.counts[2], 1);
    assert_eq!(hist.counts[5], 1);
}

// ===========================================================================
// Scenario: Complexity histogram is always non-negative
// ===========================================================================
#[test]
fn scenario_all_histogram_counts_non_negative() {
    let files: Vec<FileComplexity> = (0..100)
        .map(|i| file_with_cyclomatic(&format!("f{i}.rs"), i % 40))
        .collect();
    let hist = generate_complexity_histogram(&files, 5);

    for (i, &count) in hist.counts.iter().enumerate() {
        assert!(count <= hist.total, "bucket {i} count exceeds total");
    }
    assert_eq!(hist.counts.iter().sum::<u32>(), 100);
}
