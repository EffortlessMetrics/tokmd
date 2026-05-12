//! Freshness and code-age report construction for git analysis.

use std::collections::{BTreeMap, BTreeSet};

use tokmd_analysis_types::{
    CodeAgeBucket, CodeAgeDistributionReport, FreshnessReport, ModuleFreshnessRow, TrendClass,
};
use tokmd_scan::{percentile, round_f64};
use tokmd_types::FileRow;

use super::normalize_git_path;

const SECONDS_PER_DAY: i64 = 86_400;
const REFRESH_WINDOW_DAYS: i64 = 30;
const REFRESH_TREND_EPSILON: f64 = 0.10;

pub(super) fn build_freshness_report(
    last_change: &BTreeMap<String, i64>,
    row_map: &BTreeMap<String, (&FileRow, String)>,
    reference_ts: i64,
) -> FreshnessReport {
    let threshold_days = 365usize;
    let mut stale_files = 0usize;
    let mut total_files = 0usize;
    let mut by_module: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for (path, ts) in last_change {
        let (_, module) = match row_map.get(path) {
            Some(v) => v,
            None => continue,
        };
        let days = if reference_ts > *ts {
            ((reference_ts - *ts) / SECONDS_PER_DAY) as usize
        } else {
            0
        };
        total_files += 1;
        if days > threshold_days {
            stale_files += 1;
        }
        if let Some(list) = by_module.get_mut(module) {
            list.push(days);
        } else {
            by_module.insert(module.clone(), vec![days]);
        }
    }

    let stale_pct = if total_files == 0 {
        0.0
    } else {
        round_f64(stale_files as f64 / total_files as f64, 4)
    };

    let mut module_rows: Vec<ModuleFreshnessRow> = Vec::new();
    for (module, mut days) in by_module {
        days.sort();
        let avg = if days.is_empty() {
            0.0
        } else {
            round_f64(days.iter().sum::<usize>() as f64 / days.len() as f64, 2)
        };
        let p90 = if days.is_empty() {
            0.0
        } else {
            round_f64(percentile(&days, 0.90), 2)
        };
        let stale = days.iter().filter(|d| **d > threshold_days).count();
        let pct = if days.is_empty() {
            0.0
        } else {
            round_f64(stale as f64 / days.len() as f64, 4)
        };
        module_rows.push(ModuleFreshnessRow {
            module,
            avg_days: avg,
            p90_days: p90,
            stale_pct: pct,
        });
    }
    module_rows.sort_by(|a, b| a.module.cmp(&b.module));

    FreshnessReport {
        threshold_days,
        stale_files,
        total_files,
        stale_pct,
        by_module: module_rows,
    }
}

pub(super) fn build_code_age_distribution(
    last_change: &BTreeMap<String, i64>,
    reference_ts: i64,
    commits: &[tokmd_git::GitCommit],
) -> CodeAgeDistributionReport {
    let mut ages_days: Vec<usize> = last_change
        .values()
        .map(|ts| {
            if reference_ts > *ts {
                ((reference_ts - *ts) / SECONDS_PER_DAY) as usize
            } else {
                0
            }
        })
        .collect();
    ages_days.sort_unstable();

    let buckets = vec![
        ("0-30d", 0usize, Some(30usize)),
        ("31-90d", 31usize, Some(90usize)),
        ("91-180d", 91usize, Some(180usize)),
        ("181-365d", 181usize, Some(365usize)),
        ("366d+", 366usize, None),
    ];

    let mut counts = vec![0usize; buckets.len()];
    for age in &ages_days {
        for (idx, (_label, min_days, max_days)) in buckets.iter().enumerate() {
            let in_range = if let Some(max_days) = max_days {
                *age >= *min_days && *age <= *max_days
            } else {
                *age >= *min_days
            };
            if in_range {
                counts[idx] += 1;
                break;
            }
        }
    }

    let total_files = ages_days.len();
    let bucket_rows: Vec<CodeAgeBucket> = buckets
        .into_iter()
        .zip(counts)
        .map(|((label, min_days, max_days), files)| CodeAgeBucket {
            label: label.to_string(),
            min_days,
            max_days,
            files,
            pct: if total_files == 0 {
                0.0
            } else {
                round_f64(files as f64 / total_files as f64, 4)
            },
        })
        .collect();

    let tracked_paths: BTreeSet<String> = last_change.keys().cloned().collect();
    let (recent_refreshes, prior_refreshes, refresh_trend) =
        compute_refresh_trend(commits, reference_ts, &tracked_paths);

    CodeAgeDistributionReport {
        buckets: bucket_rows,
        recent_refreshes,
        prior_refreshes,
        refresh_trend,
    }
}

fn compute_refresh_trend(
    commits: &[tokmd_git::GitCommit],
    reference_ts: i64,
    tracked_paths: &BTreeSet<String>,
) -> (usize, usize, TrendClass) {
    if commits.is_empty() || tracked_paths.is_empty() || reference_ts <= 0 {
        return (0, 0, TrendClass::Flat);
    }

    let recent_start = reference_ts - REFRESH_WINDOW_DAYS * SECONDS_PER_DAY;
    let prior_start = recent_start - REFRESH_WINDOW_DAYS * SECONDS_PER_DAY;

    let mut recent_files: BTreeSet<String> = BTreeSet::new();
    let mut prior_files: BTreeSet<String> = BTreeSet::new();

    for commit in commits {
        if commit.timestamp >= recent_start {
            for file in &commit.files {
                let normalized = normalize_git_path(file);
                if tracked_paths.contains(&normalized) {
                    recent_files.insert(normalized);
                }
            }
        } else if commit.timestamp >= prior_start {
            for file in &commit.files {
                let normalized = normalize_git_path(file);
                if tracked_paths.contains(&normalized) {
                    prior_files.insert(normalized);
                }
            }
        }
    }

    let recent = recent_files.len();
    let prior = prior_files.len();
    let trend = if prior == 0 {
        if recent > 0 {
            TrendClass::Rising
        } else {
            TrendClass::Flat
        }
    } else {
        let delta_pct = (recent as f64 - prior as f64) / prior as f64;
        if delta_pct > REFRESH_TREND_EPSILON {
            TrendClass::Rising
        } else if delta_pct < -REFRESH_TREND_EPSILON {
            TrendClass::Falling
        } else {
            TrendClass::Flat
        }
    };

    (recent, prior, trend)
}
