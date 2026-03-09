//! Integration-style tests for the deterministic effort engine.
//!
//! These tests are intentionally scenario-oriented rather than purely unit-level.
//! They verify the public effort builder across the main layers of the current
//! implementation:
//!
//! - authored/generated/vendored size-basis classification,
//! - deterministic baseline effort math,
//! - driver extraction from peer reports,
//! - confidence grading from signal coverage,
//! - delta estimation and blast-radius classification.
//!
//! The goal is not to pin every intermediate calculation, but to make sure the
//! engine remains:
//!
//! - non-zero for realistic repositories,
//! - deterministic for the same inputs,
//! - honest about missing signals,
//! - stable at the contract level.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use tokmd_analysis_effort::{
    EffortLayer, EffortModelKind, EffortRequest, build_effort_report, cocomo81::cocomo81_baseline,
    cocomo81::estimate_with_factors,
};
use tokmd_analysis_types::{
    ApiSurfaceReport, BoilerplateReport, BusFactorRow, ComplexityReport, CouplingRow,
    DerivedReport, DerivedTotals, DistributionReport, DuplicateReport, EffortConfidenceLevel,
    EffortDeltaClassification, FileStatRow, FreshnessReport, GitReport, HotspotRow,
    IntegrityReport, LangPurityReport, LangPurityRow, MaxFileReport, MaxFileRow,
    ModuleFreshnessRow, NestingReport, NestingRow, PolyglotReport, RateReport, RateRow,
    RatioReport, RatioRow, ReadingTimeReport, TestDensityReport, TodoReport, TopOffenders,
};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

/// Guard that removes a temporary test repository when the scenario ends.
#[derive(Debug)]
struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Create a temporary repository root for effort-engine integration scenarios.
fn mk_temp_dir(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut root = std::env::temp_dir();
    root.push(format!("{prefix}-{timestamp}-{}", std::process::id()));
    root
}

/// Write a file, creating parent directories as needed.
fn write_file(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new("."))).unwrap();
    fs::write(path, body).unwrap();
}

/// Run a git command inside a temporary scenario repository.
fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "git command failed: git {} {}",
        repo.display(),
        args.join(" ")
    );
}

/// Build a compact `ExportData` fixture with predictable per-file inventory.
///
/// Use this when the test cares about deterministic effort math more than
/// real-world path classification.
fn make_export(code: usize) -> ExportData {
    ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code,
            comments: 0,
            blanks: 0,
            lines: code,
            bytes: code.saturating_mul(4),
            tokens: code.saturating_mul(2),
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

/// Build a `DerivedReport` fixture with controllable totals, test density, and
/// polyglot spread for driver/confidence scenarios.
fn make_derived(total_code: usize, test_ratio: f64, polyglot_count: usize) -> DerivedReport {
    let test_lines = (total_code as f64 * test_ratio).round() as usize;
    let prod_lines = total_code.saturating_sub(test_lines);
    let test_density_total = test_lines.saturating_add(prod_lines);
    let test_density_ratio = if test_density_total == 0 {
        0.0
    } else {
        test_lines as f64 / (test_density_total as f64)
    };

    let stat_row = FileStatRow {
        path: "src/main.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: total_code,
        comments: 0,
        blanks: 0,
        lines: total_code,
        bytes: total_code.saturating_mul(4),
        tokens: total_code.saturating_mul(2),
        doc_pct: None,
        bytes_per_line: None,
        depth: 2,
    };

    DerivedReport {
        totals: DerivedTotals {
            files: 1,
            code: total_code,
            comments: 0,
            blanks: 0,
            lines: total_code,
            bytes: total_code.saturating_mul(4),
            tokens: total_code.saturating_mul(2),
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "all".to_string(),
                numerator: total_code,
                denominator: total_code.max(1),
                ratio: 1.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "all".to_string(),
                numerator: 0,
                denominator: total_code.max(1),
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "all".to_string(),
                numerator: total_code,
                denominator: total_code.max(1),
                rate: 1.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: stat_row.clone(),
            by_lang: vec![MaxFileRow {
                key: "Rust".to_string(),
                file: stat_row.clone(),
            }],
            by_module: vec![MaxFileRow {
                key: "src".to_string(),
                file: stat_row.clone(),
            }],
        },
        lang_purity: LangPurityReport {
            rows: vec![LangPurityRow {
                module: "src".to_string(),
                lang_count: polyglot_count,
                dominant_lang: "Rust".to_string(),
                dominant_lines: total_code,
                dominant_pct: 1.0,
            }],
        },
        nesting: NestingReport {
            max: 0,
            avg: 0.0,
            by_module: vec![NestingRow {
                key: "src".to_string(),
                max: 0,
                avg: 0.0,
            }],
        },
        test_density: TestDensityReport {
            test_lines,
            prod_lines,
            test_files: if test_lines > 0 { 1 } else { 0 },
            prod_files: if prod_lines > 0 { 1 } else { 0 },
            ratio: test_density_ratio,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: total_code,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: polyglot_count,
            entropy: 0.7,
            dominant_lang: "Rust".to_string(),
            dominant_lines: total_code,
            dominant_pct: 1.0,
        },
        distribution: DistributionReport {
            count: 1,
            min: total_code,
            max: total_code,
            mean: total_code as f64,
            median: total_code as f64,
            p90: total_code as f64,
            p99: total_code as f64,
            gini: 0.0,
        },
        histogram: vec![],
        top: TopOffenders {
            largest_lines: vec![],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: total_code as f64 / 200.0,
            lines_per_minute: 200,
            basis_lines: total_code,
        },
        context_window: None,
        cocomo: None,
        todo: Some(TodoReport {
            total: 0,
            density_per_kloc: 0.0,
            tags: vec![],
        }),
        integrity: IntegrityReport {
            algo: "sha256".to_string(),
            hash: "000000".to_string(),
            entries: 1,
        },
    }
}

// -----------------------------------------------------------------------------
// Baseline effort math
// -----------------------------------------------------------------------------

#[test]
fn cocomo81_baseline_matches_internal_factorization() {
    let kloc = 1.0;
    let (
        expected_low,
        expected_mid,
        expected_high,
        expected_schedule_low,
        expected_schedule_mid,
        expected_schedule_high,
        _,
        _,
        _,
    ) = estimate_with_factors(kloc, 0.15, 0.30);
    let baseline = cocomo81_baseline(kloc);

    assert!((baseline.effort_pm_low - expected_low).abs() < 1e-9);
    assert!((baseline.effort_pm_p50 - expected_mid).abs() < 1e-9);
    assert!((baseline.effort_pm_p80 - expected_high).abs() < 1e-9);
    assert!((baseline.schedule_months_low - expected_schedule_low).abs() < 1e-9);
    assert!((baseline.schedule_months_p50 - expected_schedule_mid).abs() < 1e-9);
    assert!((baseline.schedule_months_p80 - expected_schedule_high).abs() < 1e-9);
}

#[test]
fn build_effort_report_returns_nonzero_results_for_real_input() {
    let export = make_export(1_050);
    let derived = make_derived(1_050, 0.08, 2);
    let root = mk_temp_dir("tokmd-effort-report");
    fs::create_dir_all(&root).unwrap();
    let _guard = TempDirGuard(root.clone());

    let req = EffortRequest {
        model: EffortModelKind::Cocomo81Basic,
        layer: EffortLayer::Full,
        base_ref: None,
        head_ref: None,
        monte_carlo: false,
        mc_iterations: 10_000,
        mc_seed: None,
    };

    let report =
        build_effort_report(&root, &export, &derived, None, None, None, None, &req).unwrap();
    assert_eq!(report.size_basis.authored_lines, 1_050);
    assert!(report.results.effort_pm_p50 > 0.0);
    assert!(report.results.effort_pm_low > 0.0);
    assert!(report.results.effort_pm_p80 >= report.results.effort_pm_p50);
    assert!(report.results.schedule_months_low > 0.0);
    assert!(report.results.staff_p50 > 0.0);
}

/// Verifies that the top-level effort builder reports authored/generated/vendored
/// size basis fields from file classification, not just total code lines.
#[test]
fn build_effort_report_reports_authored_generated_and_vendored_breakdown() {
    let root = mk_temp_dir("tokmd-effort-size-basis");
    fs::create_dir_all(&root).unwrap();
    let _guard = TempDirGuard(root.clone());

    write_file(&root.join("src/main.rs"), "fn main() {}\n");
    write_file(
        &root.join("target/generated/bundle.min.js"),
        "console.log(1)\n",
    );
    write_file(
        &root.join("src/vendor/lib/external.rs"),
        "pub fn vendored() {}\n",
    );

    let export = ExportData {
        rows: vec![
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 100,
                comments: 0,
                blanks: 0,
                lines: 100,
                bytes: 1200,
                tokens: 180,
            },
            FileRow {
                path: "target/generated/bundle.min.js".to_string(),
                module: "target/generated".to_string(),
                lang: "JavaScript".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 0,
                blanks: 0,
                lines: 50,
                bytes: 600,
                tokens: 80,
            },
            FileRow {
                path: "src/vendor/lib/external.rs".to_string(),
                module: "src/vendor/lib".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 20,
                comments: 0,
                blanks: 0,
                lines: 20,
                bytes: 240,
                tokens: 32,
            },
        ],
        module_roots: vec![
            "src".to_string(),
            "target/generated".to_string(),
            "src/vendor/lib".to_string(),
        ],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let derived = make_derived(170, 0.08, 2);

    let req = EffortRequest {
        model: EffortModelKind::Cocomo81Basic,
        layer: EffortLayer::Full,
        base_ref: None,
        head_ref: None,
        monte_carlo: false,
        mc_iterations: 10_000,
        mc_seed: None,
    };

    let report =
        build_effort_report(&root, &export, &derived, None, None, None, None, &req).unwrap();
    assert_eq!(report.size_basis.total_lines, 170);
    assert_eq!(report.size_basis.authored_lines, 100);
    assert_eq!(report.size_basis.generated_lines, 50);
    assert_eq!(report.size_basis.vendored_lines, 20);
    assert_eq!(report.size_basis.kloc_total, 0.17);
    assert_eq!(report.size_basis.kloc_authored, 0.1);
    assert!((report.size_basis.generated_pct - (50.0 / 170.0)).abs() < 1e-9);
    assert!((report.size_basis.vendored_pct - (20.0 / 170.0)).abs() < 1e-9);
    let generated_tag = report
        .size_basis
        .by_tag
        .iter()
        .find(|row| row.tag == "generated")
        .expect("generated tag row");
    let vendored_tag = report
        .size_basis
        .by_tag
        .iter()
        .find(|row| row.tag == "vendored")
        .expect("vendored tag row");
    assert_eq!(generated_tag.lines, 50);
    assert_eq!(vendored_tag.lines, 20);
}

// -----------------------------------------------------------------------------
// Driver extraction
// -----------------------------------------------------------------------------

#[test]
fn effort_driver_extraction_tracks_expected_signal_types() {
    let export = ExportData {
        rows: vec![
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 400,
                comments: 0,
                blanks: 0,
                lines: 400,
                bytes: 1_600,
                tokens: 800,
            },
            FileRow {
                path: "target/generated/bundle.min.js".to_string(),
                module: "target/generated".to_string(),
                lang: "JavaScript".to_string(),
                kind: FileKind::Parent,
                code: 800,
                comments: 0,
                blanks: 0,
                lines: 800,
                bytes: 2_400,
                tokens: 1_200,
            },
            FileRow {
                path: "src/vendor/lib/external.rs".to_string(),
                module: "src/vendor/lib".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 700,
                comments: 0,
                blanks: 0,
                lines: 700,
                bytes: 1_400,
                tokens: 250,
            },
        ],
        module_roots: vec![
            "src".to_string(),
            "target/generated".to_string(),
            "src/vendor/lib".to_string(),
        ],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let derived = make_derived(1_000, 0.05, 10);
    let git = GitReport {
        commits_scanned: 20,
        files_seen: 8,
        hotspots: vec![HotspotRow {
            path: "src/main.rs".to_string(),
            commits: 3,
            lines: 120,
            score: 42,
        }],
        bus_factor: vec![BusFactorRow {
            module: "src".to_string(),
            authors: 2,
        }],
        freshness: FreshnessReport {
            threshold_days: 30,
            stale_files: 3,
            total_files: 8,
            stale_pct: 0.40,
            by_module: vec![ModuleFreshnessRow {
                module: "src".to_string(),
                avg_days: 12.3,
                p90_days: 25.1,
                stale_pct: 0.40,
            }],
        },
        coupling: vec![
            CouplingRow {
                left: "src".to_string(),
                right: "src/ui".to_string(),
                count: 4,
                jaccard: Some(0.12),
                lift: Some(1.4),
                n_left: Some(20),
                n_right: Some(8),
            },
            CouplingRow {
                left: "src".to_string(),
                right: "src/db".to_string(),
                count: 2,
                jaccard: Some(0.08),
                lift: Some(1.1),
                n_left: Some(20),
                n_right: Some(6),
            },
        ],
        age_distribution: None,
        intent: None,
    };
    let complexity = ComplexityReport {
        total_functions: 120,
        avg_function_length: 18.0,
        max_function_length: 128,
        avg_cyclomatic: 5.4,
        max_cyclomatic: 46,
        avg_cognitive: Some(6.1),
        max_cognitive: Some(18),
        avg_nesting_depth: Some(4.2),
        max_nesting_depth: Some(12),
        high_risk_files: 3,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![],
    };
    let api = ApiSurfaceReport {
        total_items: 120,
        public_items: 18,
        internal_items: 102,
        public_ratio: 0.18,
        documented_ratio: 0.30,
        by_language: BTreeMap::new(),
        by_module: vec![],
        top_exporters: vec![],
    };
    let dup = DuplicateReport {
        groups: vec![],
        wasted_bytes: 512,
        strategy: "simhash".to_string(),
        density: None,
        near: None,
    };

    let req = EffortRequest {
        model: EffortModelKind::Cocomo81Basic,
        layer: EffortLayer::Full,
        base_ref: None,
        head_ref: None,
        monte_carlo: false,
        mc_iterations: 10_000,
        mc_seed: None,
    };
    let report = build_effort_report(
        &Path::new("."),
        &export,
        &derived,
        Some(&git),
        Some(&complexity),
        Some(&api),
        Some(&dup),
        &req,
    )
    .unwrap();

    let keys: Vec<&str> = report
        .drivers
        .iter()
        .map(|driver| driver.key.as_str())
        .collect();
    assert!(keys.contains(&"generated_files"));
    assert!(keys.contains(&"vendored_files"));
    assert!(keys.contains(&"freshness_staleness"));
    assert!(keys.contains(&"module_coupling"));
    assert!(keys.contains(&"complexity_hotspots"));
    assert!(keys.contains(&"complexity_breadth"));
    assert!(keys.contains(&"api_documentation"));
    assert!(keys.contains(&"api_documented_ratio"));
    assert!(keys.contains(&"duplication"));
    assert!(keys.contains(&"test_density"));
    assert!(keys.contains(&"polyglot_spread"));
}

// -----------------------------------------------------------------------------
// Confidence grading
// -----------------------------------------------------------------------------

#[test]
fn effort_confidence_distinguishes_signal_coverage() {
    let export_low = ExportData {
        rows: vec![FileRow {
            path: "mystery/random.bin".to_string(),
            module: "mystery".to_string(),
            lang: "Binary".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 0,
            blanks: 0,
            lines: 100,
            bytes: 400,
            tokens: 200,
        }],
        module_roots: vec!["mystery".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let req = EffortRequest {
        model: EffortModelKind::Cocomo81Basic,
        layer: EffortLayer::Full,
        base_ref: None,
        head_ref: None,
        monte_carlo: false,
        mc_iterations: 10_000,
        mc_seed: None,
    };
    let report_low = build_effort_report(
        &Path::new("."),
        &export_low,
        &make_derived(100, 0.0, 0),
        None,
        None,
        None,
        None,
        &req,
    )
    .unwrap();
    let low_conf = &report_low.confidence;

    assert_eq!(low_conf.level, EffortConfidenceLevel::Low);
    assert!(
        low_conf
            .reasons
            .iter()
            .any(|r: &String| r.contains("git data missing"))
    );
    assert!(
        low_conf
            .reasons
            .iter()
            .any(|r: &String| r.contains("classification used fallback"))
    );
    assert!(low_conf.data_coverage_pct.unwrap_or(1.0) <= 0.55);

    let derived_high = make_derived(1_000, 0.05, 3);
    let git = GitReport {
        commits_scanned: 10,
        files_seen: 5,
        hotspots: vec![HotspotRow {
            path: "src/main.rs".to_string(),
            commits: 1,
            lines: 20,
            score: 5,
        }],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 30,
            stale_files: 1,
            total_files: 5,
            stale_pct: 0.1,
            by_module: vec![ModuleFreshnessRow {
                module: "src".to_string(),
                avg_days: 8.0,
                p90_days: 12.0,
                stale_pct: 0.1,
            }],
        },
        coupling: vec![CouplingRow {
            left: "src".to_string(),
            right: "src/api".to_string(),
            count: 2,
            jaccard: Some(0.2),
            lift: Some(1.2),
            n_left: Some(12),
            n_right: Some(7),
        }],
        age_distribution: None,
        intent: None,
    };
    let complexity = ComplexityReport {
        total_functions: 20,
        avg_function_length: 10.0,
        max_function_length: 64,
        avg_cyclomatic: 3.0,
        max_cyclomatic: 12,
        avg_cognitive: Some(2.5),
        max_cognitive: Some(8),
        avg_nesting_depth: Some(2.0),
        max_nesting_depth: Some(4),
        high_risk_files: 1,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![],
    };
    let api = ApiSurfaceReport {
        total_items: 60,
        public_items: 50,
        internal_items: 10,
        public_ratio: 0.83,
        documented_ratio: 0.72,
        by_language: BTreeMap::new(),
        by_module: vec![],
        top_exporters: vec![],
    };
    let dup = DuplicateReport {
        groups: vec![],
        wasted_bytes: 0,
        strategy: "simhash".to_string(),
        density: None,
        near: None,
    };

    let report_high = build_effort_report(
        &Path::new("."),
        &make_export(1_000),
        &derived_high,
        Some(&git),
        Some(&complexity),
        Some(&api),
        Some(&dup),
        &req,
    )
    .unwrap();
    let high_conf = &report_high.confidence;
    assert_eq!(high_conf.level, EffortConfidenceLevel::High);
    assert!(high_conf.reasons.is_empty());
    assert!(high_conf.data_coverage_pct.unwrap_or(0.0) > 0.72);
}

// -----------------------------------------------------------------------------
// Delta / blast radius
// -----------------------------------------------------------------------------

/// Verifies that providing `base_ref` and `head_ref` can produce a populated delta
/// section with changed-file counts, blast radius, and bounded effort impact when
/// git delta support is enabled.
#[test]
fn effort_delta_infers_changed_files_modules_and_blast_radius() {
    let repo = mk_temp_dir("tokmd-effort-delta");
    fs::create_dir_all(repo.join("src")).unwrap();
    let _guard = TempDirGuard(repo.clone());

    git(&repo, &["init"]);
    git(&repo, &["config", "user.name", "tokmd-tests"]);
    git(&repo, &["config", "user.email", "tokmd-tests@example.com"]);

    write_file(&repo.join("src/main.rs"), "fn main() {}\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "initial"]);

    write_file(
        &repo.join("src/main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    );
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "add output"]);

    let export = make_export(12);
    let derived = make_derived(12, 0.0, 1);
    let req = EffortRequest {
        model: EffortModelKind::Cocomo81Basic,
        layer: EffortLayer::Full,
        base_ref: Some("HEAD~1".to_string()),
        head_ref: Some("HEAD".to_string()),
        monte_carlo: false,
        mc_iterations: 10_000,
        mc_seed: None,
    };

    let report =
        build_effort_report(&repo, &export, &derived, None, None, None, None, &req).unwrap();
    if cfg!(feature = "git") {
        let delta = report
            .delta
            .expect("delta should be populated when base/head refs are provided");

        assert_eq!(delta.base, "HEAD~1");
        assert_eq!(delta.head, "HEAD");
        assert_eq!(delta.files_changed, 1);
        assert_eq!(delta.modules_changed, 1);
        assert_eq!(delta.langs_changed, 1);
        assert_eq!(delta.hotspot_files_touched, 0);
        assert_eq!(delta.coupled_neighbors_touched, 0);
        assert_eq!(delta.classification, EffortDeltaClassification::Medium);
        assert!(delta.blast_radius >= 1.0);
        assert!(delta.effort_pm_est > 0.0);
        assert!(delta.effort_pm_low > 0.0);
        assert!(delta.effort_pm_high >= delta.effort_pm_est);
    } else {
        assert!(report.delta.is_none());
    }
}
