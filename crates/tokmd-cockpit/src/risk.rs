//! Risk metric computation for cockpit receipts.

use crate::FileStat;
use tokmd_types::cockpit::{CodeHealth, Contracts, Risk, RiskLevel};

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
pub(crate) fn compute_risk_owned(
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
    use crate::health::compute_code_health;

    fn make_stat(path: &str, insertions: usize, deletions: usize) -> FileStat {
        FileStat {
            path: path.to_string(),
            insertions,
            deletions,
        }
    }

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
}
