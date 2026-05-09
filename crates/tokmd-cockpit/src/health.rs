//! Code-health metric computation for cockpit receipts.

use crate::FileStat;
use tokmd_types::cockpit::{
    CodeHealth, ComplexityIndicator, Contracts, HealthWarning, WarningType,
};

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

#[cfg(test)]
mod tests {
    use super::*;

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
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };

        let health = compute_code_health(&[], &contracts);
        assert_eq!(health.complexity_indicator, ComplexityIndicator::Low);

        let stats = vec![make_stat("big.rs", 300, 300)];
        let health = compute_code_health(&stats, &contracts);
        assert_eq!(health.complexity_indicator, ComplexityIndicator::Medium);
    }
}
