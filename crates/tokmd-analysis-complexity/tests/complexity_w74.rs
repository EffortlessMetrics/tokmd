//! W74 – Unit tests for tokmd-analysis-complexity enricher.

use tokmd_analysis_complexity::generate_complexity_histogram;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};

// ---------------------------------------------------------------------------
// Helper: build a FileComplexity with sensible defaults
// ---------------------------------------------------------------------------
fn fc(path: &str, cyclomatic: usize) -> FileComplexity {
    FileComplexity {
        path: path.to_string(),
        module: "mod".to_string(),
        function_count: 1,
        max_function_length: 10,
        cyclomatic_complexity: cyclomatic,
        cognitive_complexity: None,
        max_nesting: None,
        risk_level: ComplexityRisk::Low,
        functions: None,
    }
}

// ── generate_complexity_histogram ──────────────────────────────────────────

#[test]
fn histogram_empty_input() {
    let hist = generate_complexity_histogram(&[], 5);
    assert_eq!(hist.total, 0);
    assert!(hist.counts.iter().all(|&c| c == 0));
}

#[test]
fn histogram_single_file_low_complexity() {
    let files = vec![fc("a.rs", 2)];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.total, 1);
    // Bucket 0 (0-4) should have 1
    assert_eq!(hist.counts[0], 1);
    assert_eq!(hist.counts[1..].iter().sum::<u32>(), 0);
}

#[test]
fn histogram_multiple_buckets() {
    let files = vec![
        fc("a.rs", 3),  // bucket 0 (0-4)
        fc("b.rs", 7),  // bucket 1 (5-9)
        fc("c.rs", 12), // bucket 2 (10-14)
        fc("d.rs", 18), // bucket 3 (15-19)
        fc("e.rs", 22), // bucket 4 (20-24)
        fc("f.rs", 28), // bucket 5 (25-29)
        fc("g.rs", 35), // bucket 6 (30+)
    ];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.total, 7);
    for &count in &hist.counts {
        assert_eq!(count, 1);
    }
}

#[test]
fn histogram_high_complexity_clamped_to_last_bucket() {
    let files = vec![fc("x.rs", 999)];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.total, 1);
    // Should land in the last bucket (30+)
    assert_eq!(hist.counts.last().copied().unwrap_or(0), 1);
}

#[test]
fn histogram_bucket_labels_are_multiples_of_bucket_size() {
    let hist = generate_complexity_histogram(&[], 5);
    assert_eq!(hist.buckets, vec![0, 5, 10, 15, 20, 25, 30]);
}

#[test]
fn histogram_with_bucket_size_ten() {
    let files = vec![fc("a.rs", 15)];
    let hist = generate_complexity_histogram(&files, 10);
    // bucket index = 15/10 = 1
    assert_eq!(hist.counts[1], 1);
    assert_eq!(hist.total, 1);
}

#[test]
fn histogram_all_files_in_same_bucket() {
    let files: Vec<FileComplexity> = (0..5).map(|i| fc(&format!("{i}.rs"), 2)).collect();
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.counts[0], 5);
    assert_eq!(hist.total, 5);
}

// ── FileComplexity defaults / risk levels ─────────────────────────────────

#[test]
fn file_complexity_default_optional_fields() {
    let f = fc("a.rs", 1);
    assert!(f.cognitive_complexity.is_none());
    assert!(f.max_nesting.is_none());
    assert!(f.functions.is_none());
}

#[test]
fn risk_enum_variants_exist() {
    // Ensure all four risk levels are representable
    let levels = [
        ComplexityRisk::Low,
        ComplexityRisk::Moderate,
        ComplexityRisk::High,
        ComplexityRisk::Critical,
    ];
    assert_eq!(levels.len(), 4);
}

#[test]
fn histogram_deterministic_across_calls() {
    let files = vec![fc("a.rs", 5), fc("b.rs", 15), fc("c.rs", 25)];
    let h1 = generate_complexity_histogram(&files, 5);
    let h2 = generate_complexity_histogram(&files, 5);
    assert_eq!(h1.counts, h2.counts);
    assert_eq!(h1.buckets, h2.buckets);
}

#[test]
fn histogram_boundary_value_at_bucket_edge() {
    // Value exactly at bucket boundary
    let files = vec![fc("a.rs", 5)];
    let hist = generate_complexity_histogram(&files, 5);
    // 5 / 5 = 1 → bucket 1
    assert_eq!(hist.counts[1], 1);
}

#[test]
fn histogram_zero_complexity_in_first_bucket() {
    let files = vec![fc("a.rs", 0)];
    let hist = generate_complexity_histogram(&files, 5);
    assert_eq!(hist.counts[0], 1);
}

#[test]
fn histogram_counts_sum_equals_total() {
    let files = vec![fc("a.rs", 3), fc("b.rs", 8), fc("c.rs", 50)];
    let hist = generate_complexity_histogram(&files, 5);
    let sum: u32 = hist.counts.iter().sum();
    assert_eq!(sum, hist.total);
}

#[test]
fn histogram_seven_buckets() {
    let hist = generate_complexity_histogram(&[], 5);
    assert_eq!(hist.buckets.len(), 7);
    assert_eq!(hist.counts.len(), 7);
}

#[test]
fn file_complexity_path_preserved() {
    let f = fc("src/deep/nested/file.rs", 10);
    assert_eq!(f.path, "src/deep/nested/file.rs");
    assert_eq!(f.module, "mod");
}
