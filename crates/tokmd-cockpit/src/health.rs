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
