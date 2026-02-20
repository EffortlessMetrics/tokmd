use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;
use tokmd_analysis_types::{
    BusFactorRow, CodeAgeBucket, CodeAgeDistributionReport, CommitIntentCounts,
    CommitIntentReport, CouplingRow, FreshnessReport, GitReport, HotspotRow, ModuleIntentRow,
    ModuleFreshnessRow, TrendClass,
};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::util::{normalize_path, percentile, round_f64};

const SECONDS_PER_DAY: i64 = 86_400;
const REFRESH_WINDOW_DAYS: i64 = 30;
const REFRESH_TREND_EPSILON: f64 = 0.10;

#[cfg(feature = "git")]
pub(crate) fn build_git_report(
    repo_root: &Path,
    export: &ExportData,
    commits: &[tokmd_git::GitCommit],
) -> Result<GitReport> {
    let mut row_map: BTreeMap<String, (&FileRow, String)> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let key = normalize_path(&row.path, repo_root);
        row_map.insert(key, (row, row.module.clone()));
    }

    let mut commit_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut authors_by_module: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut last_change: BTreeMap<String, i64> = BTreeMap::new();
    let mut max_ts = 0i64;

    for commit in commits {
        max_ts = max_ts.max(commit.timestamp);
        for file in &commit.files {
            let key = normalize_git_path(file);
            if let Some((row, module)) = row_map.get(&key) {
                *commit_counts.entry(key.clone()).or_insert(0) += 1;
                authors_by_module
                    .entry(module.clone())
                    .or_default()
                    .insert(commit.author.clone());
                last_change.entry(key.clone()).or_insert(commit.timestamp);
                let _ = row;
            }
        }
    }

    let mut hotspots: Vec<HotspotRow> = commit_counts
        .iter()
        .filter_map(|(path, commits)| {
            let (row, _) = row_map.get(path)?;
            Some(HotspotRow {
                path: path.clone(),
                commits: *commits,
                lines: row.lines,
                score: row.lines * commits,
            })
        })
        .collect();
    hotspots.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));

    let mut bus_factor: Vec<BusFactorRow> = authors_by_module
        .into_iter()
        .map(|(module, authors)| BusFactorRow {
            module,
            authors: authors.len(),
        })
        .collect();
    bus_factor.sort_by(|a, b| {
        a.authors
            .cmp(&b.authors)
            .then_with(|| a.module.cmp(&b.module))
    });

    let freshness = build_freshness_report(&last_change, &row_map, max_ts);
    let age_distribution = build_code_age_distribution(&last_change, max_ts, commits);

    let coupling = build_coupling(commits, &row_map);
    let intent = build_intent_report(commits, &row_map);

    Ok(GitReport {
        commits_scanned: commits.len(),
        files_seen: commit_counts.len(),
        hotspots,
        bus_factor,
        freshness,
        coupling,
        age_distribution: Some(age_distribution),
        intent: Some(intent),
    })
}

#[cfg(feature = "git")]
fn build_freshness_report(
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
            ((reference_ts - *ts) / 86_400) as usize
        } else {
            0
        };
        total_files += 1;
        if days > threshold_days {
            stale_files += 1;
        }
        by_module.entry(module.clone()).or_default().push(days);
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

#[cfg(feature = "git")]
fn build_coupling(
    commits: &[tokmd_git::GitCommit],
    row_map: &BTreeMap<String, (&FileRow, String)>,
) -> Vec<CouplingRow> {
    let mut pairs: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut touches: BTreeMap<String, usize> = BTreeMap::new();
    let mut multi_module_commits: usize = 0;

    for commit in commits {
        let mut modules: BTreeSet<String> = BTreeSet::new();
        for file in &commit.files {
            let key = normalize_git_path(file);
            if let Some((_row, module)) = row_map.get(&key) {
                modules.insert(module.clone());
            }
        }
        if modules.len() >= 2 {
            multi_module_commits += 1;
        }
        for module in &modules {
            *touches.entry(module.clone()).or_insert(0) += 1;
        }
        let modules: Vec<String> = modules.into_iter().collect();
        for i in 0..modules.len() {
            for j in (i + 1)..modules.len() {
                let left = modules[i].clone();
                let right = modules[j].clone();
                let key = if left <= right {
                    (left, right)
                } else {
                    (right, left)
                };
                *pairs.entry(key).or_insert(0) += 1;
            }
        }
    }

    let n = multi_module_commits;

    let mut rows: Vec<CouplingRow> = pairs
        .into_iter()
        .map(|((left, right), count)| {
            let n_a = touches.get(&left).copied().unwrap_or(0);
            let n_b = touches.get(&right).copied().unwrap_or(0);

            let jaccard = if n_a + n_b > count {
                Some(round_f64(count as f64 / (n_a + n_b - count) as f64, 4))
            } else {
                None
            };

            let lift = if n > 0 && n_a > 0 && n_b > 0 {
                Some(round_f64(
                    (count as f64 * n as f64) / (n_a as f64 * n_b as f64),
                    4,
                ))
            } else {
                None
            };

            CouplingRow {
                left,
                right,
                count,
                jaccard,
                lift,
            }
        })
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.left.cmp(&b.left)));
    rows
}

#[cfg(feature = "git")]
fn build_intent_report(
    commits: &[tokmd_git::GitCommit],
    row_map: &BTreeMap<String, (&FileRow, String)>,
) -> CommitIntentReport {
    let mut overall = CommitIntentCounts::default();
    let mut by_module_counts: BTreeMap<String, CommitIntentCounts> = BTreeMap::new();

    for commit in commits {
        let kind = tokmd_git::classify_intent(&commit.subject);
        overall.increment(kind);

        // Attribute intent to all modules touched by this commit
        let mut modules: BTreeSet<String> = BTreeSet::new();
        for file in &commit.files {
            let key = normalize_git_path(file);
            if let Some((_row, module)) = row_map.get(&key) {
                modules.insert(module.clone());
            }
        }
        for module in modules {
            by_module_counts
                .entry(module)
                .or_default()
                .increment(kind);
        }
    }

    let unknown_pct = if overall.total > 0 {
        round_f64(overall.other as f64 / overall.total as f64, 4)
    } else {
        0.0
    };

    let mut by_module: Vec<ModuleIntentRow> = by_module_counts
        .into_iter()
        .map(|(module, counts)| ModuleIntentRow { module, counts })
        .collect();
    by_module.sort_by(|a, b| a.module.cmp(&b.module));

    CommitIntentReport {
        overall,
        by_module,
        unknown_pct,
    }
}

#[cfg(feature = "git")]
fn build_code_age_distribution(
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

#[cfg(feature = "git")]
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

#[cfg(feature = "git")]
fn normalize_git_path(path: &str) -> String {
    let mut out = path.replace('\\', "/");
    if let Some(stripped) = out.strip_prefix("./") {
        out = stripped.to_string();
    }
    out
}
