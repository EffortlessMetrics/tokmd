//! Context packing algorithms for LLM context window optimization.

use std::collections::BTreeMap;

use tokmd_config::{ContextStrategy, ValueMetric};
use tokmd_types::{ContextFileRow, FileKind, FileRow};

use crate::git_scoring::GitScores;

/// Parse a budget string with optional k/m suffix into token count.
pub fn parse_budget(budget: &str) -> anyhow::Result<usize> {
    let budget = budget.trim().to_lowercase();
    if let Some(num) = budget.strip_suffix('k') {
        let n: f64 = num.trim().parse()?;
        Ok((n * 1000.0) as usize)
    } else if let Some(num) = budget.strip_suffix('m') {
        let n: f64 = num.trim().parse()?;
        Ok((n * 1_000_000.0) as usize)
    } else {
        Ok(budget.parse()?)
    }
}

/// Get the value of a file row based on the selected metric.
fn get_value(row: &FileRow, metric: ValueMetric, git_scores: Option<&GitScores>) -> usize {
    let path = normalize_path(&row.path);
    match metric {
        ValueMetric::Code => row.code,
        ValueMetric::Tokens => row.tokens,
        ValueMetric::Hotspot => git_scores
            .and_then(|gs| gs.hotspots.get(&path).copied())
            .unwrap_or(row.code),
        ValueMetric::Churn => {
            // Use commit count as churn proxy, scaled by code lines for tie-breaking
            git_scores
                .and_then(|gs| gs.commit_counts.get(&path).copied())
                .map(|commits| commits * 1000 + row.code)
                .unwrap_or(row.code)
        }
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Pack files using greedy strategy: select by value until budget exhausted.
pub fn pack_greedy(
    rows: &[FileRow],
    budget: usize,
    metric: ValueMetric,
    git_scores: Option<&GitScores>,
) -> Vec<ContextFileRow> {
    // Filter to parent files only and sort by value descending
    let mut candidates: Vec<_> = rows.iter().filter(|r| r.kind == FileKind::Parent).collect();

    candidates.sort_by(|a, b| {
        let va = get_value(a, metric, git_scores);
        let vb = get_value(b, metric, git_scores);
        vb.cmp(&va).then_with(|| a.path.cmp(&b.path))
    });

    let mut selected = Vec::new();
    let mut used_tokens = 0;

    for row in candidates {
        if used_tokens + row.tokens <= budget {
            used_tokens += row.tokens;
            selected.push(to_context_row(row, metric, git_scores));
        }
    }

    selected
}

/// Pack files using spread strategy: round-robin across groups, then greedy fill.
pub fn pack_spread(
    rows: &[FileRow],
    budget: usize,
    metric: ValueMetric,
    git_scores: Option<&GitScores>,
) -> Vec<ContextFileRow> {
    // Filter to parent files only
    let parents: Vec<_> = rows.iter().filter(|r| r.kind == FileKind::Parent).collect();

    // Group by (module, lang)
    let mut groups: BTreeMap<(String, String), Vec<&FileRow>> = BTreeMap::new();
    for row in &parents {
        let key = (row.module.clone(), row.lang.clone());
        groups.entry(key).or_default().push(row);
    }

    // Sort each group by value descending
    for group in groups.values_mut() {
        group.sort_by(|a, b| {
            let va = get_value(a, metric, git_scores);
            let vb = get_value(b, metric, git_scores);
            vb.cmp(&va).then_with(|| a.path.cmp(&b.path))
        });
    }

    let mut selected = Vec::new();
    let mut used_tokens = 0;
    let spread_budget = (budget as f64 * 0.7) as usize; // 70% for spread

    // Round-robin selection
    let mut group_indices: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut made_progress = true;

    while made_progress && used_tokens < spread_budget {
        made_progress = false;
        for (key, group) in &groups {
            let idx = group_indices.entry(key.clone()).or_insert(0);
            if *idx < group.len() {
                let row = group[*idx];
                if used_tokens + row.tokens <= spread_budget {
                    used_tokens += row.tokens;
                    selected.push(to_context_row(row, metric, git_scores));
                    *idx += 1;
                    made_progress = true;
                } else {
                    *idx += 1; // Skip this file, try next
                }
            }
        }
    }

    // Greedy fill remaining budget
    let mut remaining: Vec<_> = parents
        .iter()
        .filter(|r| !selected.iter().any(|s| s.path == r.path))
        .collect();

    remaining.sort_by(|a, b| {
        let va = get_value(a, metric, git_scores);
        let vb = get_value(b, metric, git_scores);
        vb.cmp(&va).then_with(|| a.path.cmp(&b.path))
    });

    for row in remaining {
        if used_tokens + row.tokens <= budget {
            used_tokens += row.tokens;
            selected.push(to_context_row(row, metric, git_scores));
        }
    }

    selected
}

/// Select files based on strategy.
pub fn select_files(
    rows: &[FileRow],
    budget: usize,
    strategy: ContextStrategy,
    metric: ValueMetric,
    git_scores: Option<&GitScores>,
) -> Vec<ContextFileRow> {
    match strategy {
        ContextStrategy::Greedy => pack_greedy(rows, budget, metric, git_scores),
        ContextStrategy::Spread => pack_spread(rows, budget, metric, git_scores),
    }
}

fn to_context_row(
    row: &FileRow,
    metric: ValueMetric,
    git_scores: Option<&GitScores>,
) -> ContextFileRow {
    ContextFileRow {
        path: row.path.clone(),
        module: row.module.clone(),
        lang: row.lang.clone(),
        tokens: row.tokens,
        code: row.code,
        lines: row.lines,
        bytes: row.bytes,
        value: get_value(row, metric, git_scores),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_budget() {
        assert_eq!(parse_budget("128k").unwrap(), 128_000);
        assert_eq!(parse_budget("1m").unwrap(), 1_000_000);
        assert_eq!(parse_budget("50000").unwrap(), 50_000);
        assert_eq!(parse_budget("1.5k").unwrap(), 1_500);
    }
}
