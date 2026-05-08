use tokmd_types::cockpit::{Contracts, ReviewItem};

use crate::FileStat;

/// Generate review plan.
pub fn generate_review_plan(file_stats: &[FileStat], _contracts: &Contracts) -> Vec<ReviewItem> {
    let mut items = Vec::new();

    for stat in file_stats {
        let lines = stat.insertions + stat.deletions;
        let priority = if lines > 200 {
            1
        } else if lines > 50 {
            2
        } else {
            3
        };
        let complexity = if lines > 300 {
            5
        } else if lines > 100 {
            3
        } else {
            1
        };

        items.push(ReviewItem {
            path: stat.path.clone(),
            reason: format!("{} lines changed", lines),
            priority,
            complexity: Some(complexity),
            lines_changed: Some(lines),
        });
    }

    items.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    items
}
