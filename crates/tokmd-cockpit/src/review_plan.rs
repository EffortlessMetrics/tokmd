use tokmd_types::cockpit::{Contracts, ReviewItem};

use crate::FileStat;
use crate::doc_artifacts_evidence::source_of_truth_path;

/// Generate review plan.
pub fn generate_review_plan(file_stats: &[FileStat], contracts: &Contracts) -> Vec<ReviewItem> {
    let mut items = Vec::new();

    for stat in file_stats {
        let lines = stat.insertions + stat.deletions;
        let base_priority = review_priority_for_lines(lines);
        let signal = ReviewSignal::for_path(&stat.path, contracts);
        let priority = base_priority.min(signal.priority());
        let complexity = if lines > 300 {
            5
        } else if lines > 100 {
            3
        } else {
            1
        };

        items.push(ReviewItem {
            path: stat.path.clone(),
            reason: signal.reason(lines),
            priority,
            complexity: Some(complexity),
            lines_changed: Some(lines),
        });
    }

    items.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| review_signal_rank(&a.path).cmp(&review_signal_rank(&b.path)))
            .then_with(|| a.path.cmp(&b.path))
    });
    items
}

fn review_priority_for_lines(lines: usize) -> u32 {
    if lines > 200 {
        1
    } else if lines > 50 {
        2
    } else {
        3
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ReviewSignal {
    SourceOfTruth,
    SchemaContract,
    ProofPolicy,
    CliContract,
    ApiSurface,
    Ordinary,
}

impl ReviewSignal {
    fn for_path(path: &str, contracts: &Contracts) -> Self {
        if source_of_truth_path(path) {
            Self::SourceOfTruth
        } else if schema_contract_path(path, contracts) {
            Self::SchemaContract
        } else if proof_policy_path(path) {
            Self::ProofPolicy
        } else if cli_contract_path(path, contracts) {
            Self::CliContract
        } else if api_surface_path(path, contracts) {
            Self::ApiSurface
        } else {
            Self::Ordinary
        }
    }

    fn priority(self) -> u32 {
        match self {
            Self::SourceOfTruth | Self::SchemaContract | Self::ProofPolicy | Self::CliContract => 1,
            Self::ApiSurface => 2,
            Self::Ordinary => 3,
        }
    }

    fn reason(self, lines: usize) -> String {
        let prefix = match self {
            Self::SourceOfTruth => "source-of-truth contract changed",
            Self::SchemaContract => "schema contract changed",
            Self::ProofPolicy => "proof or policy routing changed",
            Self::CliContract => "CLI contract changed",
            Self::ApiSurface => "API surface changed",
            Self::Ordinary => return format!("{lines} lines changed"),
        };
        format!("{prefix}; {lines} lines changed")
    }
}

fn review_signal_rank(path: &str) -> ReviewSignal {
    if source_of_truth_path(path) {
        ReviewSignal::SourceOfTruth
    } else if schema_contract_path_without_contracts(path) {
        ReviewSignal::SchemaContract
    } else if proof_policy_path(path) {
        ReviewSignal::ProofPolicy
    } else if cli_contract_path_without_contracts(path) {
        ReviewSignal::CliContract
    } else if api_surface_path_without_contracts(path) {
        ReviewSignal::ApiSurface
    } else {
        ReviewSignal::Ordinary
    }
}

fn schema_contract_path(path: &str, contracts: &Contracts) -> bool {
    contracts.schema_changed && schema_contract_path_without_contracts(path)
}

fn schema_contract_path_without_contracts(path: &str) -> bool {
    path == "docs/schema.json"
        || path == "docs/SCHEMA.md"
        || path.starts_with("docs/") && path.ends_with(".schema.json")
        || path.starts_with("crates/tokmd/schemas/")
}

fn proof_policy_path(path: &str) -> bool {
    path == "ci/proof.toml"
        || path == "codecov.yml"
        || path.starts_with("policy/")
        || path.starts_with(".github/workflows/")
}

fn cli_contract_path(path: &str, contracts: &Contracts) -> bool {
    contracts.cli_changed && cli_contract_path_without_contracts(path)
}

fn cli_contract_path_without_contracts(path: &str) -> bool {
    path.starts_with("crates/tokmd/src/commands/")
        || path.starts_with("crates/tokmd/src/cli/")
        || path == "crates/tokmd/src/config.rs"
}

fn api_surface_path(path: &str, contracts: &Contracts) -> bool {
    contracts.api_changed && api_surface_path_without_contracts(path)
}

fn api_surface_path_without_contracts(path: &str) -> bool {
    path.ends_with("lib.rs") || path.ends_with("mod.rs")
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
    fn test_review_plan_boosts_source_of_truth_contracts() {
        let stats = vec![
            make_stat("src/large.rs", 150, 100),
            make_stat("docs/review-packet.md", 3, 2),
            make_stat("docs/tutorial.md", 3, 2),
        ];
        let contracts = Contracts {
            api_changed: false,
            cli_changed: false,
            schema_changed: false,
            breaking_indicators: 0,
        };
        let plan = generate_review_plan(&stats, &contracts);
        assert_eq!(plan[0].path, "docs/review-packet.md");
        assert_eq!(plan[0].priority, 1);
        assert_eq!(
            plan[0].reason,
            "source-of-truth contract changed; 5 lines changed"
        );
        assert_eq!(plan[1].path, "src/large.rs");
        assert_eq!(plan[2].path, "docs/tutorial.md");
    }

    #[test]
    fn test_review_plan_boosts_contract_and_policy_paths() {
        let stats = vec![
            make_stat("src/large.rs", 150, 100),
            make_stat("docs/schema.json", 1, 1),
            make_stat("ci/proof.toml", 1, 1),
            make_stat("crates/tokmd/src/commands/cockpit.rs", 1, 1),
            make_stat("crates/tokmd-core/src/lib.rs", 1, 1),
        ];
        let contracts = Contracts {
            api_changed: true,
            cli_changed: true,
            schema_changed: true,
            breaking_indicators: 2,
        };
        let plan = generate_review_plan(&stats, &contracts);
        assert_eq!(plan[0].path, "docs/schema.json");
        assert_eq!(plan[0].priority, 1);
        assert_eq!(plan[1].path, "ci/proof.toml");
        assert_eq!(plan[1].priority, 1);
        assert_eq!(plan[2].path, "crates/tokmd/src/commands/cockpit.rs");
        assert_eq!(plan[2].priority, 1);
        assert_eq!(plan[3].path, "src/large.rs");
        assert_eq!(plan[3].priority, 1);
        assert_eq!(plan[4].path, "crates/tokmd-core/src/lib.rs");
        assert_eq!(plan[4].priority, 2);
        assert_eq!(plan[4].reason, "API surface changed; 2 lines changed");
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
