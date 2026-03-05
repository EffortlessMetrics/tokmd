//! Depth tests for complexity analysis (W56).

use tokmd_analysis_complexity::generate_complexity_histogram;
use tokmd_analysis_types::{ComplexityRisk, FileComplexity};

// ───────────────────── helpers ─────────────────────

fn make_fc(path: &str, cyclomatic: usize, risk: ComplexityRisk) -> FileComplexity {
    FileComplexity {
        path: path.to_string(),
        module: "mod".to_string(),
        function_count: 1,
        max_function_length: 10,
        cyclomatic_complexity: cyclomatic,
        cognitive_complexity: None,
        max_nesting: None,
        risk_level: risk,
        functions: None,
    }
}

// ───────────────────── histogram – empty input ─────────────────────

#[test]
fn histogram_empty_files() {
    let h = generate_complexity_histogram(&[], 5);
    assert_eq!(h.total, 0);
    assert!(h.counts.iter().all(|&c| c == 0));
    assert_eq!(h.buckets.len(), 7);
}

// ───────────────────── histogram – bucket boundaries ─────────────────────

#[test]
fn histogram_bucket_boundaries() {
    let h = generate_complexity_histogram(&[], 5);
    assert_eq!(h.buckets, vec![0, 5, 10, 15, 20, 25, 30]);
}

#[test]
fn histogram_single_low_complexity_file() {
    let files = vec![make_fc("a.rs", 2, ComplexityRisk::Low)];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.total, 1);
    assert_eq!(h.counts[0], 1); // bucket 0-4
    assert_eq!(h.counts[1], 0);
}

#[test]
fn histogram_boundary_value_4() {
    let files = vec![make_fc("a.rs", 4, ComplexityRisk::Low)];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.counts[0], 1); // 4 / 5 = 0 → bucket 0
}

#[test]
fn histogram_boundary_value_5() {
    let files = vec![make_fc("a.rs", 5, ComplexityRisk::Low)];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.counts[1], 1); // 5 / 5 = 1 → bucket 1 (5-9)
}

#[test]
fn histogram_boundary_value_30() {
    let files = vec![make_fc("a.rs", 30, ComplexityRisk::High)];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.counts[6], 1); // 30+ bucket
}

#[test]
fn histogram_very_high_complexity_lands_in_last_bucket() {
    let files = vec![make_fc("a.rs", 500, ComplexityRisk::Critical)];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.counts[6], 1); // clamped to last bucket
}

// ───────────────────── histogram – distribution ─────────────────────

#[test]
fn histogram_spread_across_buckets() {
    let files = vec![
        make_fc("a.rs", 1, ComplexityRisk::Low),
        make_fc("b.rs", 7, ComplexityRisk::Low),
        make_fc("c.rs", 12, ComplexityRisk::Moderate),
        make_fc("d.rs", 18, ComplexityRisk::Moderate),
        make_fc("e.rs", 22, ComplexityRisk::High),
        make_fc("f.rs", 27, ComplexityRisk::High),
        make_fc("g.rs", 35, ComplexityRisk::Critical),
    ];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.total, 7);
    assert_eq!(h.counts, vec![1, 1, 1, 1, 1, 1, 1]);
}

#[test]
fn histogram_all_files_in_single_bucket() {
    let files: Vec<_> = (0..10)
        .map(|i| make_fc(&format!("{i}.rs"), 3, ComplexityRisk::Low))
        .collect();
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.total, 10);
    assert_eq!(h.counts[0], 10);
    for i in 1..7 {
        assert_eq!(h.counts[i], 0);
    }
}

// ───────────────────── histogram – determinism ─────────────────────

#[test]
fn histogram_deterministic() {
    let files = vec![
        make_fc("a.rs", 3, ComplexityRisk::Low),
        make_fc("b.rs", 15, ComplexityRisk::Moderate),
        make_fc("c.rs", 45, ComplexityRisk::Critical),
    ];
    let h1 = generate_complexity_histogram(&files, 5);
    let h2 = generate_complexity_histogram(&files, 5);
    assert_eq!(h1.buckets, h2.buckets);
    assert_eq!(h1.counts, h2.counts);
    assert_eq!(h1.total, h2.total);
}

// ───────────────────── histogram – total consistency ─────────────────────

#[test]
fn histogram_total_equals_file_count() {
    let files: Vec<_> = (0..25)
        .map(|i| make_fc(&format!("{i}.rs"), i * 2, ComplexityRisk::Low))
        .collect();
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.total, 25);
    let sum: u32 = h.counts.iter().sum();
    assert_eq!(sum, 25);
}

// ───────────────────── histogram – zero complexity ─────────────────────

#[test]
fn histogram_zero_complexity_goes_first_bucket() {
    let files = vec![make_fc("a.rs", 0, ComplexityRisk::Low)];
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.counts[0], 1);
}

// ───────────────────── ComplexityRisk enum coverage ─────────────────────

#[test]
fn complexity_risk_debug_display() {
    let low = ComplexityRisk::Low;
    let moderate = ComplexityRisk::Moderate;
    let high = ComplexityRisk::High;
    let critical = ComplexityRisk::Critical;
    assert_eq!(format!("{low:?}"), "Low");
    assert_eq!(format!("{moderate:?}"), "Moderate");
    assert_eq!(format!("{high:?}"), "High");
    assert_eq!(format!("{critical:?}"), "Critical");
}

#[test]
fn complexity_risk_clone_eq() {
    let a = ComplexityRisk::High;
    let b = a;
    assert_eq!(a, b);
}

// ───────────────────── FileComplexity fields ─────────────────────

#[test]
fn file_complexity_with_cognitive() {
    let fc = FileComplexity {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        function_count: 5,
        max_function_length: 30,
        cyclomatic_complexity: 12,
        cognitive_complexity: Some(20),
        max_nesting: Some(4),
        risk_level: ComplexityRisk::Moderate,
        functions: None,
    };
    assert_eq!(fc.cognitive_complexity, Some(20));
    assert_eq!(fc.max_nesting, Some(4));
}

#[test]
fn file_complexity_without_cognitive() {
    let fc = make_fc("test.rs", 5, ComplexityRisk::Low);
    assert_eq!(fc.cognitive_complexity, None);
    assert_eq!(fc.max_nesting, None);
}

// ───────────────────── serialization round-trip ─────────────────────

#[test]
fn file_complexity_json_round_trip() {
    let fc = FileComplexity {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        function_count: 3,
        max_function_length: 15,
        cyclomatic_complexity: 8,
        cognitive_complexity: Some(12),
        max_nesting: Some(3),
        risk_level: ComplexityRisk::Low,
        functions: None,
    };
    let json = serde_json::to_string(&fc).unwrap();
    let parsed: FileComplexity = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.path, "src/main.rs");
    assert_eq!(parsed.cyclomatic_complexity, 8);
    assert_eq!(parsed.cognitive_complexity, Some(12));
    assert_eq!(parsed.risk_level, ComplexityRisk::Low);
}

#[test]
fn histogram_json_round_trip() {
    let files = vec![
        make_fc("a.rs", 3, ComplexityRisk::Low),
        make_fc("b.rs", 25, ComplexityRisk::High),
    ];
    let h = generate_complexity_histogram(&files, 5);
    let json = serde_json::to_string(&h).unwrap();
    let parsed: tokmd_analysis_types::ComplexityHistogram = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.total, h.total);
    assert_eq!(parsed.counts, h.counts);
}

// ───────────────────── histogram with custom bucket size ─────────────────────

#[test]
fn histogram_bucket_size_10() {
    let files = vec![
        make_fc("a.rs", 5, ComplexityRisk::Low),
        make_fc("b.rs", 15, ComplexityRisk::Moderate),
    ];
    let h = generate_complexity_histogram(&files, 10);
    // With bucket_size=10: buckets at 0,10,20,30,40,50,60
    assert_eq!(h.buckets, vec![0, 10, 20, 30, 40, 50, 60]);
    assert_eq!(h.counts[0], 1); // 5 → bucket 0 (0-9)
    assert_eq!(h.counts[1], 1); // 15 → bucket 1 (10-19)
}

// ───────────────────── large file set ─────────────────────

#[test]
fn histogram_large_file_set() {
    let files: Vec<_> = (0..1000)
        .map(|i| make_fc(&format!("{i}.rs"), i % 40, ComplexityRisk::Low))
        .collect();
    let h = generate_complexity_histogram(&files, 5);
    assert_eq!(h.total, 1000);
    let sum: u32 = h.counts.iter().sum();
    assert_eq!(sum, 1000);
}

// ───────────────────── deeply nested code scenario ─────────────────────

#[test]
fn high_complexity_file_data() {
    let fc = FileComplexity {
        path: "src/parser.rs".to_string(),
        module: "src".to_string(),
        function_count: 50,
        max_function_length: 200,
        cyclomatic_complexity: 80,
        cognitive_complexity: Some(150),
        max_nesting: Some(10),
        risk_level: ComplexityRisk::Critical,
        functions: None,
    };
    assert_eq!(fc.risk_level, ComplexityRisk::Critical);
    assert!(fc.cyclomatic_complexity > 50);
    assert!(fc.max_nesting.unwrap() > 8);
}

// ───────────────────── linear code scenario ─────────────────────

#[test]
fn linear_code_low_complexity() {
    let fc = FileComplexity {
        path: "src/constants.rs".to_string(),
        module: "src".to_string(),
        function_count: 0,
        max_function_length: 0,
        cyclomatic_complexity: 1,
        cognitive_complexity: Some(0),
        max_nesting: Some(0),
        risk_level: ComplexityRisk::Low,
        functions: None,
    };
    assert_eq!(fc.cyclomatic_complexity, 1);
    assert_eq!(fc.risk_level, ComplexityRisk::Low);
}
