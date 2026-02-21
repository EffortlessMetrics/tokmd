//! Tests specifically designed to kill surviving mutants in tokmd-analysis.
//!
//! These tests verify specific calculated values, not just that functions run.

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
use tokmd_config::ChildIncludeMode;
use tokmd_types::{ExportData, FileKind, FileRow};

// =============================================================================
// Helper functions
// =============================================================================

fn make_context(export: ExportData) -> AnalysisContext {
    AnalysisContext {
        export,
        root: std::path::PathBuf::from("."),
        source: AnalysisSource {
            inputs: vec![".".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: "separate".to_string(),
        },
    }
}

fn make_request(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: format!("{:?}", preset).to_lowercase(),
            format: "md".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        window_tokens: None,
        git: None,
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    }
}

fn file_row(path: &str, module: &str, lang: &str, code: usize, lines: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines,
        bytes: lines * 20,
        tokens: code * 2,
    }
}

fn export_with_rows(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// =============================================================================
// Derived metrics tests - COCOMO calculations
// =============================================================================

#[test]
fn cocomo_returns_none_for_zero_code() {
    // Mutant: replace totals.code == 0 check
    let export = export_with_rows(vec![FileRow {
        path: "empty.rs".to_string(),
        module: "(root)".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 0,
        comments: 10,
        blanks: 5,
        lines: 15,
        bytes: 100,
        tokens: 0,
    }]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert!(
        derived.cocomo.is_none(),
        "COCOMO should be None when code is 0"
    );
}

#[test]
fn cocomo_calculates_correctly_for_1000_lines() {
    // Mutant: verify COCOMO math - kloc, effort, duration, staff
    let export = export_with_rows(vec![file_row("src/main.rs", "src", "Rust", 1000, 1200)]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    let cocomo = derived.cocomo.expect("COCOMO should be present");

    // kloc = 1000 / 1000 = 1.0
    assert!(
        (cocomo.kloc - 1.0).abs() < 0.0001,
        "kloc should be 1.0, got {}",
        cocomo.kloc
    );

    // effort = 2.4 * 1.0^1.05 = 2.4
    assert!(
        (cocomo.effort_pm - 2.4).abs() < 0.01,
        "effort should be ~2.4, got {}",
        cocomo.effort_pm
    );

    // duration = 2.5 * 2.4^0.38 ≈ 3.54
    assert!(
        (cocomo.duration_months - 3.54).abs() < 0.1,
        "duration should be ~3.54, got {}",
        cocomo.duration_months
    );

    // staff = effort / duration ≈ 0.68
    assert!(
        (cocomo.staff - 0.68).abs() < 0.1,
        "staff should be ~0.68, got {}",
        cocomo.staff
    );
}

#[test]
fn cocomo_calculates_correctly_for_10000_lines() {
    // Verify COCOMO scaling with larger codebase
    let export = export_with_rows(vec![file_row("src/main.rs", "src", "Rust", 10000, 12000)]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    let cocomo = derived.cocomo.expect("COCOMO should be present");

    // kloc = 10000 / 1000 = 10.0
    assert!(
        (cocomo.kloc - 10.0).abs() < 0.0001,
        "kloc should be 10.0, got {}",
        cocomo.kloc
    );

    // effort = 2.4 * 10.0^1.05 ≈ 26.9
    assert!(
        (cocomo.effort_pm - 26.9).abs() < 1.0,
        "effort should be ~26.9, got {}",
        cocomo.effort_pm
    );
}

// =============================================================================
// Distribution tests
// =============================================================================

#[test]
fn distribution_calculates_gini_coefficient() {
    // Mutant: gini_coefficient calculations
    // Equal sizes should have gini near 0
    let rows = vec![
        file_row("a.rs", "src", "Rust", 100, 100),
        file_row("b.rs", "src", "Rust", 100, 100),
        file_row("c.rs", "src", "Rust", 100, 100),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert!(
        derived.distribution.gini.abs() < 0.01,
        "Gini should be ~0 for equal distribution, got {}",
        derived.distribution.gini
    );
}

#[test]
fn distribution_calculates_gini_for_unequal() {
    // Unequal sizes should have gini > 0
    let rows = vec![
        file_row("a.rs", "src", "Rust", 10, 10),
        file_row("b.rs", "src", "Rust", 10, 10),
        file_row("c.rs", "src", "Rust", 1000, 1000),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert!(
        derived.distribution.gini > 0.3,
        "Gini should be > 0.3 for unequal distribution, got {}",
        derived.distribution.gini
    );
}

#[test]
fn distribution_calculates_median_correctly_odd() {
    // Mutant: median calculation for odd count
    let rows = vec![
        file_row("a.rs", "src", "Rust", 10, 10),
        file_row("b.rs", "src", "Rust", 20, 20),
        file_row("c.rs", "src", "Rust", 30, 30),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert!(
        (derived.distribution.median - 20.0).abs() < 0.01,
        "Median should be 20 for [10,20,30], got {}",
        derived.distribution.median
    );
}

#[test]
fn distribution_calculates_median_correctly_even() {
    // Mutant: median calculation for even count
    let rows = vec![
        file_row("a.rs", "src", "Rust", 10, 10),
        file_row("b.rs", "src", "Rust", 20, 20),
        file_row("c.rs", "src", "Rust", 30, 30),
        file_row("d.rs", "src", "Rust", 40, 40),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // Median of [10,20,30,40] = (20+30)/2 = 25
    assert!(
        (derived.distribution.median - 25.0).abs() < 0.01,
        "Median should be 25 for [10,20,30,40], got {}",
        derived.distribution.median
    );
}

#[test]
fn distribution_calculates_percentiles() {
    // Mutant: percentile calculations
    let rows = (1..=100)
        .map(|i| file_row(&format!("f{}.rs", i), "src", "Rust", i * 10, i * 10))
        .collect();
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // p90 should be close to 90th percentile value
    assert!(
        derived.distribution.p90 >= 900.0,
        "p90 should be >= 900, got {}",
        derived.distribution.p90
    );

    // p99 should be close to max
    assert!(
        derived.distribution.p99 >= 990.0,
        "p99 should be >= 990, got {}",
        derived.distribution.p99
    );
}

// =============================================================================
// Reading time tests
// =============================================================================

#[test]
fn reading_time_calculates_correctly() {
    // Mutant: reading_time calculation (code / 20)
    let export = export_with_rows(vec![file_row("a.rs", "src", "Rust", 200, 200)]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // 200 lines / 20 lines per minute = 10 minutes
    assert!(
        (derived.reading_time.minutes - 10.0).abs() < 0.01,
        "Reading time should be 10 minutes for 200 lines, got {}",
        derived.reading_time.minutes
    );
    assert_eq!(derived.reading_time.basis_lines, 200);
    assert_eq!(derived.reading_time.lines_per_minute, 20);
}

// =============================================================================
// Context window tests
// =============================================================================

#[test]
fn context_window_calculates_pct_correctly() {
    // Mutant: context window percentage calculation
    let export = export_with_rows(vec![FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 0,
        blanks: 0,
        lines: 100,
        bytes: 1000,
        tokens: 500, // 500 tokens
    }]);

    let ctx = make_context(export);
    let mut req = make_request(AnalysisPreset::Receipt);
    req.window_tokens = Some(1000); // 1000 token window

    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    let cw = derived
        .context_window
        .expect("context window should be present");

    // 500 / 1000 = 0.5
    assert!(
        (cw.pct - 0.5).abs() < 0.0001,
        "pct should be 0.5, got {}",
        cw.pct
    );
    assert!(cw.fits, "500 tokens should fit in 1000 window");
}

#[test]
fn context_window_zero_window_gives_zero_pct() {
    // Mutant: division by zero guard
    let export = export_with_rows(vec![file_row("a.rs", "src", "Rust", 100, 100)]);

    let ctx = make_context(export);
    let mut req = make_request(AnalysisPreset::Receipt);
    req.window_tokens = Some(0);

    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    let cw = derived
        .context_window
        .expect("context window should be present");
    assert!(
        (cw.pct - 0.0).abs() < 0.0001,
        "pct should be 0 for zero window"
    );
}

// =============================================================================
// Test density tests
// =============================================================================

#[test]
fn test_density_calculates_ratio() {
    // Mutant: test_density ratio calculation
    let rows = vec![
        file_row("src/lib.rs", "src", "Rust", 100, 100),
        file_row("tests/lib_test.rs", "tests", "Rust", 50, 50),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // test_lines = 50, total = 150, ratio = 50/150 = 0.3333
    assert!(
        (derived.test_density.ratio - 0.3333).abs() < 0.01,
        "Test density ratio should be ~0.333, got {}",
        derived.test_density.ratio
    );
    assert_eq!(derived.test_density.test_lines, 50);
    assert_eq!(derived.test_density.prod_lines, 100);
    assert_eq!(derived.test_density.test_files, 1);
    assert_eq!(derived.test_density.prod_files, 1);
}

// =============================================================================
// Boilerplate tests
// =============================================================================

#[test]
fn boilerplate_calculates_ratio() {
    // Mutant: boilerplate ratio calculation
    let rows = vec![
        file_row("src/lib.rs", "src", "Rust", 100, 100),
        file_row("Cargo.toml", "(root)", "TOML", 50, 50),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // infra_lines = 50 (TOML), logic_lines = 100, ratio = 50/150 = 0.3333
    assert!(
        (derived.boilerplate.ratio - 0.3333).abs() < 0.01,
        "Boilerplate ratio should be ~0.333, got {}",
        derived.boilerplate.ratio
    );
    assert_eq!(derived.boilerplate.infra_lines, 50);
    assert_eq!(derived.boilerplate.logic_lines, 100);
}

// =============================================================================
// Polyglot tests
// =============================================================================

#[test]
fn polyglot_calculates_entropy() {
    // Mutant: entropy calculation with log2
    let rows = vec![
        file_row("a.rs", "src", "Rust", 500, 500),
        file_row("b.py", "src", "Python", 500, 500),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // Equal distribution: entropy = -2 * (0.5 * log2(0.5)) = -2 * (-0.5) = 1.0
    assert!(
        (derived.polyglot.entropy - 1.0).abs() < 0.01,
        "Entropy should be 1.0 for 50-50 split, got {}",
        derived.polyglot.entropy
    );
    assert_eq!(derived.polyglot.lang_count, 2);
}

#[test]
fn polyglot_dominant_lang_correct() {
    // Mutant: dominant language selection
    let rows = vec![
        file_row("a.rs", "src", "Rust", 700, 700),
        file_row("b.py", "src", "Python", 300, 300),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(derived.polyglot.dominant_lang, "Rust");
    assert_eq!(derived.polyglot.dominant_lines, 700);
    assert!((derived.polyglot.dominant_pct - 0.7).abs() < 0.01);
}

// =============================================================================
// Histogram tests
// =============================================================================

#[test]
fn histogram_bucket_counts() {
    // Mutant: histogram bucket range checks
    let rows = vec![
        file_row("tiny.rs", "src", "Rust", 30, 30), // Tiny: 0-50
        file_row("small.rs", "src", "Rust", 100, 100), // Small: 51-200
        file_row("medium.rs", "src", "Rust", 300, 300), // Medium: 201-500
        file_row("large.rs", "src", "Rust", 800, 800), // Large: 501-1000
        file_row("huge.rs", "src", "Rust", 1500, 1500), // Huge: 1001+
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    let tiny = derived
        .histogram
        .iter()
        .find(|b| b.label == "Tiny")
        .unwrap();
    let small = derived
        .histogram
        .iter()
        .find(|b| b.label == "Small")
        .unwrap();
    let medium = derived
        .histogram
        .iter()
        .find(|b| b.label == "Medium")
        .unwrap();
    let large = derived
        .histogram
        .iter()
        .find(|b| b.label == "Large")
        .unwrap();
    let huge = derived
        .histogram
        .iter()
        .find(|b| b.label == "Huge")
        .unwrap();

    assert_eq!(tiny.files, 1, "Tiny bucket should have 1 file");
    assert_eq!(small.files, 1, "Small bucket should have 1 file");
    assert_eq!(medium.files, 1, "Medium bucket should have 1 file");
    assert_eq!(large.files, 1, "Large bucket should have 1 file");
    assert_eq!(huge.files, 1, "Huge bucket should have 1 file");

    // Each bucket has 1/5 = 0.2 = 20%
    assert!((tiny.pct - 0.2).abs() < 0.001, "Each bucket should be 20%");
}

// =============================================================================
// Nesting tests
// =============================================================================

#[test]
fn nesting_calculates_depth() {
    // Mutant: path_depth calculation
    let rows = vec![
        file_row("a.rs", "(root)", "Rust", 10, 10),  // depth 1
        file_row("src/b.rs", "src", "Rust", 10, 10), // depth 2
        file_row("src/foo/c.rs", "src/foo", "Rust", 10, 10), // depth 3
        file_row("src/foo/bar/d.rs", "src/foo/bar", "Rust", 10, 10), // depth 4
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(derived.nesting.max, 4, "Max depth should be 4");
    // avg = (1+2+3+4)/4 = 2.5
    assert!(
        (derived.nesting.avg - 2.5).abs() < 0.01,
        "Avg depth should be 2.5, got {}",
        derived.nesting.avg
    );
}

// =============================================================================
// Doc density tests
// =============================================================================

#[test]
fn doc_density_calculates_ratio() {
    // Mutant: doc_density ratio = comments / (code + comments)
    let export = export_with_rows(vec![FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 80,
        comments: 20, // 20 comments
        blanks: 10,
        lines: 110,
        bytes: 1000,
        tokens: 100,
    }]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // ratio = 20 / (80 + 20) = 0.2
    assert!(
        (derived.doc_density.total.ratio - 0.2).abs() < 0.0001,
        "Doc density should be 0.2, got {}",
        derived.doc_density.total.ratio
    );
}

// =============================================================================
// Lang purity tests
// =============================================================================

#[test]
fn lang_purity_calculates_dominant_pct() {
    // Mutant: dominant_pct calculation
    let rows = vec![
        file_row("a.rs", "src", "Rust", 70, 70),
        file_row("b.py", "src", "Python", 30, 30),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    let src_purity = derived
        .lang_purity
        .rows
        .iter()
        .find(|r| r.module == "src")
        .unwrap();

    // Rust: 70 lines, Python: 30 lines, total: 100
    // dominant_pct = 70 / 100 = 0.7
    assert!(
        (src_purity.dominant_pct - 0.7).abs() < 0.01,
        "Dominant pct should be 0.7, got {}",
        src_purity.dominant_pct
    );
    assert_eq!(src_purity.dominant_lang, "Rust");
    assert_eq!(src_purity.lang_count, 2);
}

// =============================================================================
// Fun report / Eco-label tests
// =============================================================================

#[test]
fn eco_label_grade_a_for_small_codebase() {
    // Mutant: eco label thresholds (<=1MB gets A)
    let export = export_with_rows(vec![FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 0,
        blanks: 0,
        lines: 100,
        bytes: 500_000, // 500KB - should be grade A
        tokens: 200,
    }]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Fun);
    let receipt = analyze(ctx, req).unwrap();

    let fun = receipt.fun.expect("Fun report should be present");
    let eco = fun.eco_label.expect("Eco label should be present");

    assert_eq!(eco.label, "A", "500KB should be grade A");
    assert!(
        (eco.score - 95.0).abs() < 0.01,
        "Grade A score should be 95"
    );
}

#[test]
fn eco_label_grade_b_for_medium_codebase() {
    // Mutant: eco label threshold (1-10MB gets B)
    let export = export_with_rows(vec![FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 0,
        blanks: 0,
        lines: 100,
        bytes: 5_000_000, // 5MB - should be grade B
        tokens: 200,
    }]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Fun);
    let receipt = analyze(ctx, req).unwrap();

    let fun = receipt.fun.expect("Fun report should be present");
    let eco = fun.eco_label.expect("Eco label should be present");

    assert_eq!(eco.label, "B", "5MB should be grade B");
    assert!(
        (eco.score - 80.0).abs() < 0.01,
        "Grade B score should be 80"
    );
}

#[test]
fn eco_label_grade_c_for_larger_codebase() {
    // Mutant: eco label threshold (10-50MB gets C)
    let export = export_with_rows(vec![FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 0,
        blanks: 0,
        lines: 100,
        bytes: 30_000_000, // 30MB - should be grade C
        tokens: 200,
    }]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Fun);
    let receipt = analyze(ctx, req).unwrap();

    let fun = receipt.fun.expect("Fun report should be present");
    let eco = fun.eco_label.expect("Eco label should be present");

    assert_eq!(eco.label, "C", "30MB should be grade C");
    assert!(
        (eco.score - 65.0).abs() < 0.01,
        "Grade C score should be 65"
    );
}

#[test]
fn eco_label_grade_d_for_large_codebase() {
    // Mutant: eco label threshold (50-200MB gets D)
    let export = export_with_rows(vec![FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 0,
        blanks: 0,
        lines: 100,
        bytes: 100_000_000, // 100MB - should be grade D
        tokens: 200,
    }]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Fun);
    let receipt = analyze(ctx, req).unwrap();

    let fun = receipt.fun.expect("Fun report should be present");
    let eco = fun.eco_label.expect("Eco label should be present");

    assert_eq!(eco.label, "D", "100MB should be grade D");
    assert!(
        (eco.score - 45.0).abs() < 0.01,
        "Grade D score should be 45"
    );
}

#[test]
fn eco_label_grade_e_for_huge_codebase() {
    // Mutant: eco label threshold (>200MB gets E)
    let export = export_with_rows(vec![FileRow {
        path: "a.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 0,
        blanks: 0,
        lines: 100,
        bytes: 300_000_000, // 300MB - should be grade E
        tokens: 200,
    }]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Fun);
    let receipt = analyze(ctx, req).unwrap();

    let fun = receipt.fun.expect("Fun report should be present");
    let eco = fun.eco_label.expect("Eco label should be present");

    assert_eq!(eco.label, "E", "300MB should be grade E");
    assert!(
        (eco.score - 30.0).abs() < 0.01,
        "Grade E score should be 30"
    );
}

// =============================================================================
// Max file tests
// =============================================================================

#[test]
fn max_file_selects_largest() {
    // Mutant: max file selection logic
    let rows = vec![
        file_row("small.rs", "src", "Rust", 50, 50),
        file_row("large.rs", "src", "Rust", 200, 200),
        file_row("medium.rs", "src", "Rust", 100, 100),
    ];
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(
        derived.max_file.overall.path, "large.rs",
        "Max file should be large.rs with 200 lines"
    );
    assert_eq!(derived.max_file.overall.lines, 200);
}

#[test]
fn max_file_empty_export() {
    // Mutant: empty rows handling
    let export = export_with_rows(vec![]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(derived.max_file.overall.lines, 0);
    assert_eq!(derived.max_file.overall.path, "");
}

// =============================================================================
// Top offenders tests
// =============================================================================

#[test]
fn top_offenders_sorted_correctly() {
    // Mutant: sorting logic for top offenders
    let rows: Vec<FileRow> = (1..=15)
        .map(|i| file_row(&format!("file{:02}.rs", i), "src", "Rust", i * 100, i * 100))
        .collect();
    let export = export_with_rows(rows);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    // Should have top 10, largest first
    assert_eq!(derived.top.largest_lines.len(), 10);
    assert_eq!(
        derived.top.largest_lines[0].lines, 1500,
        "Largest should be 1500 lines"
    );
    assert_eq!(
        derived.top.largest_lines[9].lines, 600,
        "10th should be 600 lines"
    );
}

// =============================================================================
// Integrity hash tests
// =============================================================================

#[test]
fn integrity_hash_changes_with_content() {
    // Mutant: integrity hash calculation
    let export1 = export_with_rows(vec![file_row("a.rs", "src", "Rust", 100, 100)]);
    let export2 = export_with_rows(vec![file_row("a.rs", "src", "Rust", 200, 200)]);

    let ctx1 = make_context(export1);
    let ctx2 = make_context(export2);
    let req = make_request(AnalysisPreset::Receipt);

    let receipt1 = analyze(ctx1, req.clone()).unwrap();
    let receipt2 = analyze(ctx2, req).unwrap();

    let hash1 = receipt1.derived.unwrap().integrity.hash;
    let hash2 = receipt2.derived.unwrap().integrity.hash;

    assert_ne!(
        hash1, hash2,
        "Different content should produce different hashes"
    );
}

// =============================================================================
// Empty distribution tests
// =============================================================================

#[test]
fn distribution_empty_is_zeroed() {
    // Mutant: empty distribution handling
    let export = export_with_rows(vec![]);

    let ctx = make_context(export);
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(derived.distribution.count, 0);
    assert_eq!(derived.distribution.min, 0);
    assert_eq!(derived.distribution.max, 0);
    assert!((derived.distribution.mean - 0.0).abs() < 0.001);
    assert!((derived.distribution.median - 0.0).abs() < 0.001);
    assert!((derived.distribution.gini - 0.0).abs() < 0.001);
}
