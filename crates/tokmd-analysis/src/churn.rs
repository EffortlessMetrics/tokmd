use std::collections::{BTreeMap, BTreeSet};

use tokmd_analysis_types::{ChurnTrend, PredictiveChurnReport, TrendClass};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::util::normalize_path;

const SECONDS_PER_WEEK: i64 = 7 * 86_400;
const RECENT_WEEKS: i64 = 4;
const SLOPE_EPSILON: f64 = 0.01;

pub(crate) fn build_predictive_churn_report(
    export: &ExportData,
    commits: &[tokmd_git::GitCommit],
    repo_root: &std::path::Path,
) -> PredictiveChurnReport {
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, repo_root), row);
    }

    let mut series: BTreeMap<String, BTreeMap<i64, i64>> = BTreeMap::new();
    for commit in commits {
        let week = commit.timestamp / SECONDS_PER_WEEK;
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for file in &commit.files {
            let key = normalize_git_path(file);
            if let Some(row) = row_map.get(&key) {
                seen.insert(row.module.clone());
            }
        }
        for module in seen {
            let entry = series.entry(module).or_default();
            *entry.entry(week).or_insert(0) += 1;
        }
    }

    let mut per_module: BTreeMap<String, ChurnTrend> = BTreeMap::new();
    for (module, points) in series {
        let (slope, r2) = regression(&points);
        let recent_change = recent_delta(&points);
        let classification = classify_trend(slope);
        per_module.insert(
            module,
            ChurnTrend {
                slope,
                r2,
                recent_change,
                classification,
            },
        );
    }

    PredictiveChurnReport { per_module }
}

fn regression(points: &BTreeMap<i64, i64>) -> (f64, f64) {
    let n = points.len();
    if n < 2 {
        return (0.0, 0.0);
    }

    let xs: Vec<f64> = points.keys().map(|v| *v as f64).collect();
    let ys: Vec<f64> = points.values().map(|v| *v as f64).collect();
    let mean_x = xs.iter().sum::<f64>() / n as f64;
    let mean_y = ys.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return (0.0, 0.0);
    }

    let slope = cov / var_x;
    let intercept = mean_y - slope * mean_x;

    let mut ss_res = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let pred = intercept + slope * x;
        let err = y - pred;
        ss_res += err * err;
    }
    let r2 = (1.0 - ss_res / var_y).clamp(0.0, 1.0);
    (slope, r2)
}

fn recent_delta(points: &BTreeMap<i64, i64>) -> i64 {
    if points.is_empty() {
        return 0;
    }
    let last_week = *points.keys().max().unwrap_or(&0);
    let recent_start = last_week - (RECENT_WEEKS - 1);
    let prev_start = recent_start - RECENT_WEEKS;

    let mut recent = 0i64;
    let mut prev = 0i64;
    for week in prev_start..=last_week {
        let count = points.get(&week).copied().unwrap_or(0);
        if week >= recent_start {
            recent += count;
        } else {
            prev += count;
        }
    }
    recent - prev
}

fn classify_trend(slope: f64) -> TrendClass {
    if slope > SLOPE_EPSILON {
        TrendClass::Rising
    } else if slope < -SLOPE_EPSILON {
        TrendClass::Falling
    } else {
        TrendClass::Flat
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
mod tests {
    use super::*;
    use tokmd_config::ChildIncludeMode;
    use tokmd_types::{ExportData, FileKind, FileRow};

    fn export_with_paths(paths: &[&str]) -> ExportData {
        let rows = paths
            .iter()
            .map(|p| FileRow {
                path: (*p).to_string(),
                module: "core".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 1,
                comments: 0,
                blanks: 0,
                lines: 1,
                bytes: 10,
                tokens: 2,
            })
            .collect();
        ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        }
    }

    #[test]
    fn slope_signs_make_sense() {
        let export = export_with_paths(&["src/lib.rs"]);
        let commits = vec![
            tokmd_git::GitCommit {
                timestamp: SECONDS_PER_WEEK,
                author: "a@acme.com".to_string(),
                subject: String::new(),
                files: vec!["src/lib.rs".to_string()],
            },
            tokmd_git::GitCommit {
                timestamp: 2 * SECONDS_PER_WEEK,
                author: "a@acme.com".to_string(),
                subject: String::new(),
                files: vec!["src/lib.rs".to_string()],
            },
            tokmd_git::GitCommit {
                timestamp: 3 * SECONDS_PER_WEEK,
                author: "a@acme.com".to_string(),
                subject: String::new(),
                files: vec!["src/lib.rs".to_string()],
            },
        ];
        let report = build_predictive_churn_report(&export, &commits, std::path::Path::new("."));
        let trend = report.per_module.get("core").unwrap();
        assert!(trend.slope >= 0.0);
    }
}
