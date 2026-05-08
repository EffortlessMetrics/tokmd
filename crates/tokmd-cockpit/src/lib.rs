//! # tokmd-cockpit
//!
//! **Tier 2 (Computation & Rendering)**
//!
//! Cockpit PR metrics computation and rendering for tokmd.
//! Provides functions to compute change surface, code health, risk,
//! composition, evidence gates, and review plans for pull requests.
//!
//! ## What belongs here
//! * Cockpit metric computation functions
//! * Evidence gate computation (mutation, diff coverage, complexity, etc.)
//! * Markdown/JSON/sections rendering
//! * Determinism hashing helpers
//!
//! ## What does NOT belong here
//! * CLI argument parsing (use `tokmd::cli`)
//! * Type definitions (use tokmd-types::cockpit)

mod composition;
pub mod determinism;
mod display;
#[cfg(feature = "git")]
mod gates;
mod proof_evidence;
pub mod render;
mod review_plan;
#[cfg(feature = "git")]
mod supply_chain;
mod trend;

#[cfg(feature = "git")]
use std::path::{Path, PathBuf};

use anyhow::Result;
#[cfg(feature = "git")]
use anyhow::{Context, bail};
pub use composition::compute_composition;
pub use display::{format_signed_f64, now_iso8601, round_pct, sparkline, trend_direction_label};
#[cfg(feature = "git")]
pub use gates::compute_determinism_gate;
#[cfg(feature = "git")]
use gates::compute_evidence;
pub use review_plan::generate_review_plan;
#[cfg(all(test, feature = "git"))]
use tokmd_analysis::source_complexity::analyze_rust_function_complexity;
pub use trend::{compute_complexity_trend, compute_metric_trend, load_and_compute_trend};
// Re-export types from tokmd_types::cockpit for convenience
pub use tokmd_types::cockpit::*;

/// Cyclomatic complexity threshold for high complexity.
pub const COMPLEXITY_THRESHOLD: u32 = 15;

/// File stat from git diff --numstat.
/// File stat from git diff --numstat.
#[derive(Debug, Clone)]
pub struct FileStat {
    pub path: String,
    pub insertions: usize,
    pub deletions: usize,
}

impl AsRef<str> for FileStat {
    fn as_ref(&self) -> &str {
        &self.path
    }
}

// =============================================================================
// Core cockpit computation
// =============================================================================

/// Compute the full cockpit receipt for a PR.
#[cfg(feature = "git")]
pub fn compute_cockpit(
    repo_root: &PathBuf,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
    baseline_path: Option<&Path>,
) -> Result<CockpitReceipt> {
    let generated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Get changed files with their stats
    let file_stats = get_file_stats(repo_root, base, head, range_mode)?;

    // Get change surface from git
    let change_surface = compute_change_surface(repo_root, base, head, &file_stats, range_mode)?;

    // Compute composition with test ratio
    let composition = compute_composition(&file_stats);

    // Detect contract changes
    let contracts = detect_contracts(&file_stats);

    // Compute code health
    let code_health = compute_code_health(&file_stats, &contracts);

    // Compute all gate evidence
    let evidence = compute_evidence(
        repo_root,
        base,
        head,
        &file_stats,
        &contracts,
        range_mode,
        baseline_path,
    )?;

    // Generate review plan with complexity scores
    let review_plan = generate_review_plan(&file_stats, &contracts);

    // Compute risk based on various factors
    let risk = compute_risk_owned(file_stats, &contracts, &code_health);

    Ok(CockpitReceipt {
        schema_version: COCKPIT_SCHEMA_VERSION,
        mode: "cockpit".to_string(),
        generated_at_ms,
        base_ref: base.to_string(),
        head_ref: head.to_string(),
        change_surface,
        composition,
        code_health,
        risk,
        contracts,
        evidence,
        review_plan,
        trend: None, // Populated by caller if --baseline is provided
    })
}

// =============================================================================
// File stats and change surface
// =============================================================================

/// Get file stats for changed files.
#[cfg(feature = "git")]
pub fn get_file_stats(
    repo_root: &Path,
    base: &str,
    head: &str,
    range_mode: tokmd_git::GitRangeMode,
) -> Result<Vec<FileStat>> {
    let range = range_mode.format(base, head);
    let output = tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["diff", "--numstat", &range])
        .output()
        .context("Failed to run git diff --numstat")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git diff --numstat failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut stats = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() == 3 {
            let insertions = parts[0].parse().unwrap_or(0);
            let deletions = parts[1].parse().unwrap_or(0);
            let path = parts[2].to_string();
            stats.push(FileStat {
                path,
                insertions,
                deletions,
            });
        }
    }

    Ok(stats)
}

/// Compute change surface metrics.
#[cfg(feature = "git")]
fn compute_change_surface(
    repo_root: &Path,
    base: &str,
    head: &str,
    file_stats: &[FileStat],
    range_mode: tokmd_git::GitRangeMode,
) -> Result<ChangeSurface> {
    let range = range_mode.format(base, head);
    let output = tokmd_git::git_cmd()
        .arg("-C")
        .arg(repo_root)
        .args(["rev-list", "--count", &range])
        .output()
        .context("Failed to run git rev-list --count")?;

    let commits = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    let files_changed = file_stats.len();
    let insertions = file_stats.iter().map(|s| s.insertions).sum();
    let deletions = file_stats.iter().map(|s| s.deletions).sum();
    let net_lines = (insertions as i64) - (deletions as i64);

    let churn_velocity = if commits > 0 {
        (insertions + deletions) as f64 / commits as f64
    } else {
        0.0
    };

    // Simple change concentration: what % of changes are in top 20% of files
    let mut changes: Vec<usize> = file_stats
        .iter()
        .map(|s| s.insertions + s.deletions)
        .collect();
    changes.sort_unstable_by(|a, b| b.cmp(a));

    let top_count = (files_changed as f64 * 0.2).ceil() as usize;
    let total_changes: usize = changes.iter().sum();
    let top_changes: usize = changes.iter().take(top_count).sum();

    let change_concentration = if total_changes > 0 {
        top_changes as f64 / total_changes as f64
    } else {
        0.0
    };

    Ok(ChangeSurface {
        commits,
        files_changed,
        insertions,
        deletions,
        net_lines,
        churn_velocity,
        change_concentration,
    })
}

// =============================================================================
// Composition, contracts, health, risk, review plan
// =============================================================================

/// Detect contract changes.
pub fn detect_contracts<S: AsRef<str>>(files: &[S]) -> Contracts {
    let mut api_changed = false;
    let mut cli_changed = false;
    let mut schema_changed = false;
    let mut breaking_indicators = 0;

    for file in files.iter() {
        if file.as_ref().ends_with("lib.rs") || file.as_ref().ends_with("mod.rs") {
            api_changed = true;
        }
        if file.as_ref().contains("crates/tokmd/src/commands/")
            || file.as_ref().contains("crates/tokmd/src/cli/")
            || file.as_ref() == "crates/tokmd/src/config.rs"
        {
            cli_changed = true;
        }
        if file.as_ref() == "docs/schema.json" || file.as_ref() == "docs/SCHEMA.md" {
            schema_changed = true;
        }
    }

    if api_changed {
        breaking_indicators += 1;
    }
    if schema_changed {
        breaking_indicators += 1;
    }

    Contracts {
        api_changed,
        cli_changed,
        schema_changed,
        breaking_indicators,
    }
}

/// Compute code health metrics.
pub fn compute_code_health(file_stats: &[FileStat], contracts: &Contracts) -> CodeHealth {
    let mut large_files_touched = 0;
    let mut total_lines = 0;

    for stat in file_stats {
        let lines = stat.insertions + stat.deletions;
        if lines > 500 {
            large_files_touched += 1;
        }
        total_lines += lines;
    }

    let avg_file_size = if !file_stats.is_empty() {
        total_lines / file_stats.len()
    } else {
        0
    };

    let complexity_indicator = if large_files_touched > 5 {
        ComplexityIndicator::Critical
    } else if large_files_touched > 2 {
        ComplexityIndicator::High
    } else if large_files_touched > 0 {
        ComplexityIndicator::Medium
    } else {
        ComplexityIndicator::Low
    };

    let mut warnings = Vec::new();
    for stat in file_stats {
        if stat.insertions + stat.deletions > 500 {
            warnings.push(HealthWarning {
                path: stat.path.clone(),
                warning_type: WarningType::LargeFile,
                message: "Large file touched".to_string(),
            });
        }
    }

    let mut score: u32 = 100;
    score = score.saturating_sub((large_files_touched * 10) as u32);
    if contracts.breaking_indicators > 0 {
        score = score.saturating_sub(20);
    }

    let grade = match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
    .to_string();

    CodeHealth {
        score,
        grade,
        large_files_touched,
        avg_file_size,
        complexity_indicator,
        warnings,
    }
}

fn compute_risk_from_iter<I>(_contracts: &Contracts, health: &CodeHealth, file_stats: I) -> Risk
where
    I: IntoIterator<Item = String>,
{
    let mut hotspots_touched = Vec::new();
    let bus_factor_warnings = Vec::new();

    for path in file_stats {
        hotspots_touched.push(path);
    }

    let score = (hotspots_touched.len() * 15 + (100 - health.score) as usize).min(100) as u32;

    let level = match score {
        0..=20 => RiskLevel::Low,
        21..=50 => RiskLevel::Medium,
        51..=80 => RiskLevel::High,
        _ => RiskLevel::Critical,
    };

    Risk {
        hotspots_touched,
        bus_factor_warnings,
        level,
        score,
    }
}

/// Compute risk metrics for borrowed file stats.
pub fn compute_risk(file_stats: &[FileStat], contracts: &Contracts, health: &CodeHealth) -> Risk {
    compute_risk_from_iter(
        contracts,
        health,
        file_stats
            .iter()
            .filter(|stat| stat.insertions + stat.deletions > 300)
            .map(|stat| stat.path.clone()),
    )
}

/// Internal fast path used by cockpit assembly when it already owns the stats.
#[cfg(feature = "git")]
fn compute_risk_owned(
    file_stats: Vec<FileStat>,
    contracts: &Contracts,
    health: &CodeHealth,
) -> Risk {
    compute_risk_from_iter(
        contracts,
        health,
        file_stats
            .into_iter()
            .filter(|stat| stat.insertions + stat.deletions > 300)
            .map(|stat| stat.path),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- compute_metric_trend ----

    #[test]
    fn test_metric_trend_improving_higher_is_better() {
        let trend = compute_metric_trend(90.0, 80.0, true);
        assert_eq!(trend.direction, TrendDirection::Improving);
        assert_eq!(trend.delta, 10.0);
        assert!(trend.delta_pct > 0.0);
    }

    #[test]
    fn test_metric_trend_degrading_higher_is_better() {
        let trend = compute_metric_trend(70.0, 80.0, true);
        assert_eq!(trend.direction, TrendDirection::Degrading);
        assert_eq!(trend.delta, -10.0);
    }

    #[test]
    fn test_metric_trend_stable() {
        let trend = compute_metric_trend(80.0, 80.0, true);
        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_metric_trend_improving_lower_is_better() {
        // Risk: lower is better
        let trend = compute_metric_trend(30.0, 50.0, false);
        assert_eq!(trend.direction, TrendDirection::Improving);
    }

    #[test]
    fn test_metric_trend_degrading_lower_is_better() {
        let trend = compute_metric_trend(50.0, 30.0, false);
        assert_eq!(trend.direction, TrendDirection::Degrading);
    }

    #[test]
    fn test_metric_trend_from_zero() {
        let trend = compute_metric_trend(10.0, 0.0, true);
        assert_eq!(trend.delta_pct, 100.0);
    }

    #[test]
    fn test_metric_trend_both_zero() {
        let trend = compute_metric_trend(0.0, 0.0, true);
        assert_eq!(trend.delta_pct, 0.0);
        assert_eq!(trend.direction, TrendDirection::Stable);
    }

    // ---- compute_composition ----

    #[test]
    fn test_composition_mixed_files() {
        let files = vec![
            "src/main.rs",
            "src/lib.rs",
            "tests/test_main.rs",
            "README.md",
            "Cargo.toml",
        ];
        let comp = compute_composition(&files);
        assert!(comp.code_pct > 0.0);
        assert!(comp.test_pct > 0.0);
        assert!(comp.docs_pct > 0.0);
        assert!(comp.config_pct > 0.0);
    }

    #[test]
    fn test_composition_empty_input() {
        let files: Vec<&str> = vec![];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 0.0);
        assert_eq!(comp.test_pct, 0.0);
        assert_eq!(comp.test_ratio, 0.0);
    }

    #[test]
    fn test_composition_only_code() {
        let files = vec!["src/main.rs", "src/lib.rs"];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 1.0);
        assert_eq!(comp.test_pct, 0.0);
        assert_eq!(comp.test_ratio, 0.0);
    }

    #[test]
    fn test_composition_test_ratio() {
        let files = vec![
            "src/main.rs",
            "src/lib.rs",
            "tests/test_main.rs",
            "tests/test_lib.rs",
        ];
        let comp = compute_composition(&files);
        // 2 code files, 2 test files → ratio = 1.0
        assert_eq!(comp.test_ratio, 1.0);
    }

    #[test]
    fn test_composition_only_tests() {
        let files = vec!["tests/test_main.rs", "tests/test_lib.rs"];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 0.0);
        assert_eq!(comp.test_pct, 1.0);
        // No code files, but tests exist → test_ratio = 1.0
        assert_eq!(comp.test_ratio, 1.0);
    }

    // ---- detect_contracts ----

    #[test]
    fn test_detect_contracts_api() {
        let files = vec!["crates/tokmd-types/src/lib.rs"];
        let contracts = detect_contracts(&files);
        assert!(contracts.api_changed);
        assert!(!contracts.cli_changed);
        assert!(!contracts.schema_changed);
        assert_eq!(contracts.breaking_indicators, 1);
    }

    #[test]
    fn test_detect_contracts_cli() {
        let files = vec!["crates/tokmd/src/commands/lang.rs"];
        let contracts = detect_contracts(&files);
        assert!(!contracts.api_changed);
        assert!(contracts.cli_changed);
    }

    #[test]
    fn test_detect_contracts_schema() {
        let files = vec!["docs/schema.json"];
        let contracts = detect_contracts(&files);
        assert!(contracts.schema_changed);
        assert_eq!(contracts.breaking_indicators, 1);
    }

    #[test]
    fn test_detect_contracts_none() {
        let files = vec!["README.md", "src/utils.rs"];
        let contracts = detect_contracts(&files);
        assert!(!contracts.api_changed);
        assert!(!contracts.cli_changed);
        assert!(!contracts.schema_changed);
        assert_eq!(contracts.breaking_indicators, 0);
    }

    #[test]
    fn test_detect_contracts_all() {
        let files = vec![
            "crates/tokmd-types/src/lib.rs",
            "crates/tokmd/src/commands/lang.rs",
            "docs/schema.json",
        ];
        let contracts = detect_contracts(&files);
        assert!(contracts.api_changed);
        assert!(contracts.cli_changed);
        assert!(contracts.schema_changed);
        assert_eq!(contracts.breaking_indicators, 2); // api + schema
    }

    // ---- compute_code_health ----

    fn make_stat(path: &str, insertions: usize, deletions: usize) -> FileStat {
        FileStat {
            path: path.to_string(),
            insertions,
            deletions,
        }
    }

    #[test]
    fn test_code_health_perfect_score() {
        let stats = vec![make_stat("src/main.rs", 10, 5)];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&stats, &contracts);
        assert_eq!(health.score, 100);
        assert_eq!(health.grade, "A");
        assert_eq!(health.large_files_touched, 0);
    }

    #[test]
    fn test_code_health_large_file_penalty() {
        let stats = vec![make_stat("src/huge.rs", 400, 200)]; // >500 lines
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&stats, &contracts);
        assert!(health.score < 100);
        assert_eq!(health.large_files_touched, 1);
        assert!(!health.warnings.is_empty());
    }

    #[test]
    fn test_code_health_breaking_changes_penalty() {
        let stats = vec![make_stat("src/lib.rs", 10, 5)];
        let contracts = Contracts {
            api_changed: true,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 1,
        };
        let health = compute_code_health(&stats, &contracts);
        assert_eq!(health.score, 80); // 100 - 20 for breaking
    }

    #[test]
    fn test_code_health_empty_stats() {
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&[], &contracts);
        assert_eq!(health.score, 100);
        assert_eq!(health.avg_file_size, 0);
    }

    #[test]
    fn test_code_health_complexity_indicators() {
        // 0 large files = Low
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&[], &contracts);
        assert_eq!(health.complexity_indicator, ComplexityIndicator::Low);

        // 1 large file = Medium
        let stats = vec![make_stat("big.rs", 300, 300)];
        let health = compute_code_health(&stats, &contracts);
        assert_eq!(health.complexity_indicator, ComplexityIndicator::Medium);
    }

    // ---- compute_risk ----

    #[test]
    fn test_risk_no_hotspots() {
        let stats = vec![make_stat("src/main.rs", 10, 5)];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&stats, &contracts);
        let risk = compute_risk(&stats, &contracts, &health);
        assert_eq!(risk.level, RiskLevel::Low);
        assert!(risk.hotspots_touched.is_empty());
    }

    #[test]
    fn test_risk_with_hotspots() {
        let stats = vec![
            make_stat("src/huge.rs", 200, 200), // >300 lines total
            make_stat("src/big.rs", 200, 200),  // >300 lines total
        ];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let health = compute_code_health(&stats, &contracts);
        let risk = compute_risk(&stats, &contracts, &health);
        assert!(!risk.hotspots_touched.is_empty());
        assert!(risk.score > 0);
    }

    // ---- generate_review_plan ----

    #[test]
    fn test_review_plan_sorted_by_priority() {
        let stats = vec![
            make_stat("small.rs", 10, 5),    // priority 3
            make_stat("medium.rs", 40, 30),  // priority 2
            make_stat("large.rs", 150, 100), // priority 1
        ];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&stats, &contracts);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].priority, 1);
        assert_eq!(plan[1].priority, 2);
        assert_eq!(plan[2].priority, 3);
    }

    #[test]
    fn test_review_plan_tiebreaks_by_path_within_priority() {
        let stats = vec![
            make_stat("zeta.rs", 120, 20),
            make_stat("alpha.rs", 110, 10),
            make_stat("middle.rs", 60, 0),
        ];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&stats, &contracts);
        assert_eq!(plan[0].path, "alpha.rs");
        assert_eq!(plan[1].path, "middle.rs");
        assert_eq!(plan[2].path, "zeta.rs");
    }

    #[test]
    fn test_review_plan_empty() {
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&[], &contracts);
        assert!(plan.is_empty());
    }

    #[test]
    fn test_review_plan_complexity_scores() {
        let stats = vec![
            make_stat("huge.rs", 200, 200), // >300 lines: complexity 5
            make_stat("med.rs", 60, 60),    // >100 lines: complexity 3
            make_stat("small.rs", 5, 5),    // <=100 lines: complexity 1
        ];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&stats, &contracts);
        // Find each item by path
        let huge = plan.iter().find(|i| i.path == "huge.rs").unwrap();
        let med = plan.iter().find(|i| i.path == "med.rs").unwrap();
        let small = plan.iter().find(|i| i.path == "small.rs").unwrap();
        assert_eq!(huge.complexity, Some(5));
        assert_eq!(med.complexity, Some(3));
        assert_eq!(small.complexity, Some(1));
    }

    #[test]
    #[cfg(feature = "git")]
    fn test_rust_complexity_ignores_decisions_in_strings_and_comments() {
        let analysis = analyze_rust_function_complexity(
            r###"
fn only_real_branch(flag: bool) {
    let _normal = "if while for loop match && || ? => { }";
    let _raw = r##"if while for loop match && || ? => { }"##;
    let _char = '?';
    /* if outer /* while nested */ match ignored => */
    if flag {
        println!("ok"); // else if ignored && ||
    }
}
"###,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 2);
        assert_eq!(analysis.max_complexity, 2);
    }

    #[test]
    #[cfg(feature = "git")]
    fn test_rust_complexity_counts_code_before_trailing_comment() {
        let analysis = analyze_rust_function_complexity(
            r#"
fn branch_before_comment(flag: bool) {
    if flag { return; } // if ignored && ||
}
"#,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 2);
        assert_eq!(analysis.max_complexity, 2);
    }

    #[test]
    #[cfg(feature = "git")]
    fn test_rust_complexity_counts_else_if_once() {
        let analysis = analyze_rust_function_complexity(
            r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else if x == 0 {
        0
    } else {
        42
    }
}
"#,
        );

        assert_eq!(analysis.function_count, 1);
        assert_eq!(analysis.total_complexity, 4);
        assert_eq!(analysis.max_complexity, 4);
    }

    // ---- FileStat AsRef ----

    #[test]
    fn test_filestat_as_ref() {
        let stat = FileStat {
            path: "src/main.rs".to_string(),
            insertions: 10,
            deletions: 5,
        };
        let s: &str = stat.as_ref();
        assert_eq!(s, "src/main.rs");
    }
}
