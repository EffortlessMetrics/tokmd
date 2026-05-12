use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;
use tokmd_analysis_types::{
    BusFactorRow, CommitIntentCounts, CommitIntentReport, CouplingRow, GitReport, HotspotRow,
    ModuleIntentRow,
};
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_types::normalize_path;
use tokmd_scan::round_f64;

mod churn;
mod freshness;

pub(crate) use churn::build_predictive_churn_report;
use freshness::{build_code_age_distribution, build_freshness_report};

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
                if let Some(val) = commit_counts.get_mut(&key) {
                    *val += 1;
                } else {
                    commit_counts.insert(key.clone(), 1);
                }
                if let Some(val) = authors_by_module.get_mut(module) {
                    val.insert(commit.author.clone());
                } else {
                    let mut set = BTreeSet::new();
                    set.insert(commit.author.clone());
                    authors_by_module.insert(module.clone(), set);
                }
                if !last_change.contains_key(&key) {
                    last_change.insert(key.clone(), commit.timestamp);
                }
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

fn build_coupling(
    commits: &[tokmd_git::GitCommit],
    row_map: &BTreeMap<String, (&FileRow, String)>,
) -> Vec<CouplingRow> {
    let mut pairs: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let mut touches: BTreeMap<&str, usize> = BTreeMap::new();
    let mut commits_considered: usize = 0;

    for commit in commits {
        let mut modules: BTreeSet<&str> = BTreeSet::new();
        for file in &commit.files {
            let key = normalize_git_path(file);
            if let Some((_row, module)) = row_map.get(&key) {
                modules.insert(module.as_str());
            }
        }
        // Only count commits where at least one file maps to a module
        if modules.is_empty() {
            continue;
        }
        commits_considered += 1;
        for m in &modules {
            if let Some(val) = touches.get_mut(m) {
                *val += 1;
            } else {
                touches.insert(*m, 1);
            }
        }
        let modules: Vec<&str> = modules.into_iter().collect();
        for i in 0..modules.len() {
            let left = modules[i];
            for right in modules.iter().skip(i + 1) {
                let key = (left, *right);
                *pairs.entry(key).or_insert(0) += 1;
            }
        }
    }

    let n = commits_considered;

    let mut rows: Vec<CouplingRow> = pairs
        .into_iter()
        .map(|((left, right), count)| {
            let n_a = touches.get(left).copied().unwrap_or(0);
            let n_b = touches.get(right).copied().unwrap_or(0);
            let denom = (n_a + n_b).saturating_sub(count);
            let jaccard = if denom > 0 {
                Some(round_f64(count as f64 / denom as f64, 4))
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
                left: left.to_string(),
                right: right.to_string(),
                count,
                jaccard,
                lift,
                n_left: Some(n_a),
                n_right: Some(n_b),
            }
        })
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.left.cmp(&b.left)));
    rows
}

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
        let mut modules: BTreeSet<&str> = BTreeSet::new();
        for file in &commit.files {
            let key = normalize_git_path(file);
            if let Some((_row, module)) = row_map.get(&key) {
                modules.insert(module.as_str());
            }
        }
        for module in modules {
            by_module_counts
                .entry(module.to_string())
                .or_default()
                .increment(kind);
        }
    }

    let unknown_pct = if overall.total > 0 {
        round_f64(overall.other as f64 / overall.total as f64, 4)
    } else {
        0.0
    };

    let corrective_ratio = if overall.total > 0 {
        Some(round_f64(
            (overall.fix + overall.revert) as f64 / overall.total as f64,
            4,
        ))
    } else {
        None
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
        corrective_ratio,
    }
}

fn normalize_git_path(path: &str) -> String {
    let mut out = path.replace('\\', "/");
    if let Some(stripped) = out.strip_prefix("./") {
        out = stripped.to_string();
    }
    out
}

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;
