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
        let huge = plan.iter().find(|i| i.path == "huge.rs").unwrap();
        let med = plan.iter().find(|i| i.path == "med.rs").unwrap();
        let small = plan.iter().find(|i| i.path == "small.rs").unwrap();
        assert_eq!(huge.complexity, Some(5));
        assert_eq!(med.complexity, Some(3));
        assert_eq!(small.complexity, Some(1));
    }
}
