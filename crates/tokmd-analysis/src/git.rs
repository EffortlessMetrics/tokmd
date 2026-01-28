use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;
use tokmd_analysis_types::{
    BusFactorRow, CouplingRow, FreshnessReport, GitReport, HotspotRow, ModuleFreshnessRow,
};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::util::{normalize_path, percentile, round_f64};

#[cfg(feature = "git")]
pub(crate) fn build_git_report(
    repo_root: &Path,
    export: &ExportData,
    commits: &[tokmd_git::GitCommit],
) -> Result<GitReport> {
    let mut row_map: BTreeMap<String, (&FileRow, String)> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let key = normalize_path(&row.path, &repo_root);
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

    let coupling = build_coupling(commits, &row_map);

    Ok(GitReport {
        commits_scanned: commits.len(),
        files_seen: commit_counts.len(),
        hotspots,
        bus_factor,
        freshness,
        coupling,
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

    for commit in commits {
        let mut modules: BTreeSet<String> = BTreeSet::new();
        for file in &commit.files {
            let key = normalize_git_path(file);
            if let Some((_row, module)) = row_map.get(&key) {
                modules.insert(module.clone());
            }
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

    let mut rows: Vec<CouplingRow> = pairs
        .into_iter()
        .map(|((left, right), count)| CouplingRow { left, right, count })
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.left.cmp(&b.left)));
    rows
}

#[cfg(feature = "git")]
fn normalize_git_path(path: &str) -> String {
    let mut out = path.replace('\\', "/");
    if let Some(stripped) = out.strip_prefix("./") {
        out = stripped.to_string();
    }
    out
}
