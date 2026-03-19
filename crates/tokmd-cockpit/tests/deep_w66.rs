//! Wave-66 deep tests for tokmd-cockpit.
//!
//! Coverage targets:
//! - compute_composition: all-code, all-test, mixed, empty, unrecognised
//! - detect_contracts: API, CLI, schema, none, combined
//! - compute_code_health: no large files, large files, many large files, contracts
//! - compute_risk: low/medium/high/critical
//! - generate_review_plan: ordering, priority, complexity
//! - sparkline, round_pct, format_signed_f64, trend helpers
//! - compute_metric_trend directions
//! - determinism: same input → same output
//! - Feature-gated git tests with #[cfg(feature = "git")]

use tokmd_cockpit::*;

// =========================================================================
// Helpers
// =========================================================================

fn stat(path: &str, ins: usize, del: usize) -> FileStat {
    FileStat {
        path: path.to_string(),
        insertions: ins,
        deletions: del,
    }
}

fn no_contracts() -> Contracts {
    Contracts {
        api_changed: false,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 0,
    }
}

// =========================================================================
// 1. compute_composition
// =========================================================================

#[test]
fn composition_empty_input() {
    let empty: Vec<&str> = vec![];
    let c = compute_composition(&empty);
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_pct, 0.0);
    assert_eq!(c.docs_pct, 0.0);
    assert_eq!(c.config_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

#[test]
fn composition_pure_code() {
    let files = vec!["src/lib.rs", "src/main.rs"];
    let c = compute_composition(&files);
    assert_eq!(c.code_pct, 1.0);
    assert_eq!(c.test_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

#[test]
fn composition_pure_tests() {
    let files = vec!["tests/test_a.rs", "tests/test_b.rs"];
    let c = compute_composition(&files);
    assert_eq!(c.test_pct, 1.0);
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_ratio, 1.0);
}

#[test]
fn composition_mixed_four_categories() {
    let files = vec![
        "src/lib.rs",      // code
        "tests/test_a.rs", // test
        "README.md",       // docs
        "Cargo.toml",      // config
    ];
    let c = compute_composition(&files);
    assert_eq!(c.code_pct, 0.25);
    assert_eq!(c.test_pct, 0.25);
    assert_eq!(c.docs_pct, 0.25);
    assert_eq!(c.config_pct, 0.25);
    assert_eq!(c.test_ratio, 1.0); // 1 test : 1 code
}

#[test]
fn composition_unrecognised_files_ignored() {
    let files = vec!["image.png", "data.bin"];
    let c = compute_composition(&files);
    assert_eq!(c.code_pct, 0.0);
    assert_eq!(c.test_ratio, 0.0);
}

#[test]
fn composition_docs_via_md_extension() {
    let files = vec!["docs/design.md"];
    let c = compute_composition(&files);
    assert_eq!(c.docs_pct, 1.0);
}

#[test]
fn composition_docs_in_nested_docs_subdir() {
    let files = vec!["project/docs/guide.txt"];
    let c = compute_composition(&files);
    assert_eq!(c.docs_pct, 1.0);
}

// =========================================================================
// 2. detect_contracts
// =========================================================================

#[test]
fn contracts_none_when_no_relevant_files() {
    let files = vec!["src/utils.rs", "README.md"];
    let c = detect_contracts(&files);
    assert!(!c.api_changed);
    assert!(!c.cli_changed);
    assert!(!c.schema_changed);
    assert_eq!(c.breaking_indicators, 0);
}

#[test]
fn contracts_api_changed_from_lib_rs() {
    let files = vec!["crates/tokmd-gate/src/lib.rs"];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_cli_changed_from_commands() {
    let files = vec!["crates/tokmd/src/commands/gate.rs"];
    let c = detect_contracts(&files);
    assert!(c.cli_changed);
}

#[test]
fn contracts_schema_changed() {
    let files = vec!["docs/schema.json"];
    let c = detect_contracts(&files);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 1);
}

#[test]
fn contracts_all_flags() {
    let files = vec![
        "crates/tokmd-core/src/lib.rs",
        "crates/tokmd/src/commands/lang.rs",
        "docs/schema.json",
    ];
    let c = detect_contracts(&files);
    assert!(c.api_changed);
    assert!(c.cli_changed);
    assert!(c.schema_changed);
    assert_eq!(c.breaking_indicators, 2); // api + schema
}

// =========================================================================
// 3. compute_code_health
// =========================================================================

#[test]
fn health_no_large_files_score_100() {
    let stats = vec![stat("a.rs", 10, 5)];
    let h = compute_code_health(&stats, &no_contracts());
    assert_eq!(h.score, 100);
    assert_eq!(h.grade, "A");
    assert_eq!(h.large_files_touched, 0);
}

#[test]
fn health_one_large_file_deducts() {
    let stats = vec![stat("big.rs", 400, 200)];
    let h = compute_code_health(&stats, &no_contracts());
    assert_eq!(h.large_files_touched, 1);
    assert_eq!(h.score, 90);
    assert_eq!(h.grade, "A");
}

#[test]
fn health_many_large_files_caps() {
    let stats: Vec<FileStat> = (0..12)
        .map(|i| stat(&format!("f{i}.rs"), 400, 200))
        .collect();
    let h = compute_code_health(&stats, &no_contracts());
    assert!(h.score < 100);
    assert_eq!(h.large_files_touched, 12);
}

#[test]
fn health_breaking_contracts_reduce_score() {
    let stats = vec![stat("a.rs", 10, 5)];
    let contracts = Contracts {
        api_changed: true,
        cli_changed: false,
        schema_changed: false,
        breaking_indicators: 1,
    };
    let h = compute_code_health(&stats, &contracts);
    assert_eq!(h.score, 80); // 100 - 20 for breaking
    assert_eq!(h.grade, "B");
}

#[test]
fn health_empty_stats() {
    let h = compute_code_health(&[], &no_contracts());
    assert_eq!(h.score, 100);
    assert_eq!(h.avg_file_size, 0);
}

// =========================================================================
// 4. compute_risk
// =========================================================================

#[test]
fn risk_low_when_small_changes() {
    let stats = vec![stat("a.rs", 10, 5)];
    let health = compute_code_health(&stats, &no_contracts());
    let r = compute_risk(&stats, &no_contracts(), &health);
    assert_eq!(r.level, RiskLevel::Low);
    assert!(r.hotspots_touched.is_empty());
}

#[test]
fn risk_medium_with_hotspot() {
    let stats = vec![stat("hot.rs", 200, 200)];
    let health = compute_code_health(&stats, &no_contracts());
    let r = compute_risk(&stats, &no_contracts(), &health);
    assert!(!r.hotspots_touched.is_empty());
    assert!(r.score > 0);
}

#[test]
fn risk_score_capped_at_100() {
    let stats: Vec<FileStat> = (0..20)
        .map(|i| stat(&format!("f{i}.rs"), 500, 500))
        .collect();
    let health = compute_code_health(&stats, &no_contracts());
    let r = compute_risk(&stats, &no_contracts(), &health);
    assert!(r.score <= 100);
}

// =========================================================================
// 5. generate_review_plan
// =========================================================================

#[test]
fn review_plan_sorted_by_priority() {
    let stats = vec![
        stat("small.rs", 5, 5),    // low priority (3)
        stat("medium.rs", 40, 20), // medium priority (2)
        stat("big.rs", 150, 100),  // high priority (1)
    ];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan.len(), 3);
    assert!(plan[0].priority <= plan[1].priority);
    assert!(plan[1].priority <= plan[2].priority);
}

#[test]
fn review_plan_empty_input() {
    let plan = generate_review_plan(&[], &no_contracts());
    assert!(plan.is_empty());
}

#[test]
fn review_plan_complexity_scales_with_lines() {
    let stats = vec![stat("huge.rs", 400, 200)];
    let plan = generate_review_plan(&stats, &no_contracts());
    assert_eq!(plan[0].complexity, Some(5));
}

// =========================================================================
// 6. Utility helpers
// =========================================================================

#[test]
fn round_pct_various() {
    assert_eq!(round_pct(0.0), 0.0);
    assert_eq!(round_pct(0.999), 1.0);
    assert_eq!(round_pct(0.123), 0.12);
    assert_eq!(round_pct(-0.567), -0.57);
}

#[test]
fn format_signed_positive_negative_zero() {
    assert_eq!(format_signed_f64(5.0), "+5.00");
    assert_eq!(format_signed_f64(-2.5), "-2.50");
    assert_eq!(format_signed_f64(0.0), "0.00");
}

#[test]
fn sparkline_empty() {
    assert_eq!(sparkline(&[]), "");
}

#[test]
fn sparkline_ascending_bars() {
    let s = sparkline(&[0.0, 50.0, 100.0]);
    assert_eq!(s.chars().count(), 3);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], '\u{2581}'); // lowest
    assert_eq!(chars[2], '\u{2588}'); // highest
}

#[test]
fn sparkline_constant_all_same() {
    let s = sparkline(&[42.0, 42.0, 42.0]);
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], chars[1]);
    assert_eq!(chars[1], chars[2]);
}

#[test]
fn trend_direction_labels_all() {
    assert_eq!(
        trend_direction_label(TrendDirection::Improving),
        "improving"
    );
    assert_eq!(trend_direction_label(TrendDirection::Stable), "stable");
    assert_eq!(
        trend_direction_label(TrendDirection::Degrading),
        "degrading"
    );
}

// =========================================================================
// 7. compute_metric_trend
// =========================================================================

#[test]
fn metric_trend_improving_higher_is_better() {
    let t = compute_metric_trend(90.0, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Improving);
    assert_eq!(t.delta, 10.0);
}

#[test]
fn metric_trend_degrading_higher_is_better() {
    let t = compute_metric_trend(70.0, 80.0, true);
    assert_eq!(t.direction, TrendDirection::Degrading);
}

#[test]
fn metric_trend_improving_lower_is_better() {
    let t = compute_metric_trend(5.0, 15.0, false);
    assert_eq!(t.direction, TrendDirection::Improving);
}

#[test]
fn metric_trend_stable_when_small_delta() {
    let t = compute_metric_trend(80.0, 80.5, true);
    assert_eq!(t.direction, TrendDirection::Stable);
}

#[test]
fn metric_trend_zero_previous() {
    let t = compute_metric_trend(10.0, 0.0, true);
    assert_eq!(t.delta_pct, 100.0);
}

// =========================================================================
// 8. Determinism property tests
// =========================================================================

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn composition_deterministic(n_code in 0usize..10, n_test in 0usize..10) {
            let mut files: Vec<String> = Vec::new();
            for i in 0..n_code {
                files.push(format!("src/f{i}.rs"));
            }
            for i in 0..n_test {
                files.push(format!("tests/t{i}.rs"));
            }
            let refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
            let c1 = compute_composition(&refs);
            let c2 = compute_composition(&refs);
            prop_assert_eq!(c1.code_pct.to_bits(), c2.code_pct.to_bits());
            prop_assert_eq!(c1.test_pct.to_bits(), c2.test_pct.to_bits());
        }

        #[test]
        fn review_plan_deterministic(ins in 0usize..1000, del in 0usize..1000) {
            let stats = vec![stat("f.rs", ins, del)];
            let p1 = generate_review_plan(&stats, &no_contracts());
            let p2 = generate_review_plan(&stats, &no_contracts());
            prop_assert_eq!(p1.len(), p2.len());
            if !p1.is_empty() {
                prop_assert_eq!(p1[0].priority, p2[0].priority);
            }
        }
    }
}

// =========================================================================
// 9. Feature-gated git tests
// =========================================================================

#[cfg(feature = "git")]
mod git_tests {

    use std::fs;

    #[test]
    fn hash_files_from_paths_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();

        let h1 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["a.rs", "b.rs"])
            .unwrap();
        let h2 = tokmd_cockpit::determinism::hash_files_from_paths(dir.path(), &["b.rs", "a.rs"])
            .unwrap();
        assert_eq!(h1, h2, "hash should be order-independent");
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn hash_cargo_lock_absent_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let result = tokmd_cockpit::determinism::hash_cargo_lock(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn hash_cargo_lock_present_returns_hex() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.lock"), "[[package]]\nname=\"x\"").unwrap();
        let result = tokmd_cockpit::determinism::hash_cargo_lock(dir.path()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 64);
    }
}
