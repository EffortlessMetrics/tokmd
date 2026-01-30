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

    fn make_test_row(path: &str, module: &str, lang: &str, tokens: usize, code: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: module.to_string(),
            lang: lang.to_string(),
            kind: FileKind::Parent,
            code,
            comments: 0,
            blanks: 0,
            lines: code,
            bytes: code * 10,
            tokens,
        }
    }

    fn make_child_row(path: &str, module: &str, lang: &str, tokens: usize, code: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: module.to_string(),
            lang: lang.to_string(),
            kind: FileKind::Child,
            code,
            comments: 0,
            blanks: 0,
            lines: code,
            bytes: code * 10,
            tokens,
        }
    }

    #[test]
    fn test_parse_budget() {
        assert_eq!(parse_budget("128k").unwrap(), 128_000);
        assert_eq!(parse_budget("1m").unwrap(), 1_000_000);
        assert_eq!(parse_budget("50000").unwrap(), 50_000);
        assert_eq!(parse_budget("1.5k").unwrap(), 1_500);
    }

    #[test]
    fn test_parse_budget_with_whitespace() {
        assert_eq!(parse_budget("  10k  ").unwrap(), 10_000);
        assert_eq!(parse_budget(" 5m ").unwrap(), 5_000_000);
    }

    #[test]
    fn test_parse_budget_case_insensitive() {
        assert_eq!(parse_budget("10K").unwrap(), 10_000);
        assert_eq!(parse_budget("2M").unwrap(), 2_000_000);
    }

    #[test]
    fn test_parse_budget_multiplication_k() {
        // Ensure multiplication is correct (not addition or division)
        assert_eq!(parse_budget("2k").unwrap(), 2_000);
        assert_eq!(parse_budget("0.5k").unwrap(), 500);
    }

    #[test]
    fn test_parse_budget_multiplication_m() {
        // Ensure multiplication is correct (not addition or division)
        assert_eq!(parse_budget("2m").unwrap(), 2_000_000);
        assert_eq!(parse_budget("0.5m").unwrap(), 500_000);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("foo/bar"), "foo/bar");
        assert_eq!(normalize_path("foo\\bar"), "foo/bar");
        assert_eq!(normalize_path("foo\\bar\\baz"), "foo/bar/baz");
    }

    #[test]
    fn test_get_value_code_metric() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        assert_eq!(get_value(&row, ValueMetric::Code, None), 50);
    }

    #[test]
    fn test_get_value_tokens_metric() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        assert_eq!(get_value(&row, ValueMetric::Tokens, None), 100);
    }

    #[test]
    fn test_get_value_hotspot_without_git() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        // Without git scores, falls back to code
        assert_eq!(get_value(&row, ValueMetric::Hotspot, None), 50);
    }

    #[test]
    fn test_get_value_hotspot_with_git() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        let mut hotspots = BTreeMap::new();
        hotspots.insert("test.rs".to_string(), 999);
        let git_scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };
        assert_eq!(
            get_value(&row, ValueMetric::Hotspot, Some(&git_scores)),
            999
        );
    }

    #[test]
    fn test_get_value_churn_without_git() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        // Without git scores, falls back to code
        assert_eq!(get_value(&row, ValueMetric::Churn, None), 50);
    }

    #[test]
    fn test_get_value_churn_with_git() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        let mut commit_counts = BTreeMap::new();
        commit_counts.insert("test.rs".to_string(), 5);
        let git_scores = GitScores {
            hotspots: BTreeMap::new(),
            commit_counts,
        };
        // churn = commits * 1000 + code = 5 * 1000 + 50 = 5050
        assert_eq!(get_value(&row, ValueMetric::Churn, Some(&git_scores)), 5050);
    }

    #[test]
    fn test_pack_greedy_empty_rows() {
        let rows: Vec<FileRow> = vec![];
        let result = pack_greedy(&rows, 1000, ValueMetric::Code, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_pack_greedy_budget_exceeded() {
        let rows = vec![
            make_test_row("a.rs", "mod", "Rust", 500, 100),
            make_test_row("b.rs", "mod", "Rust", 600, 200),
        ];
        // Budget of 500 can only fit file a (500 tokens)
        // b.rs has 600 tokens which exceeds budget
        // Even though b.rs has higher value (200 > 100), it doesn't fit
        let result = pack_greedy(&rows, 500, ValueMetric::Code, None);
        assert_eq!(result.len(), 1);
        // Should pick a.rs because b.rs doesn't fit in the budget
        assert_eq!(result[0].path, "a.rs");
    }

    #[test]
    fn test_pack_greedy_filters_child_rows() {
        let rows = vec![
            make_test_row("parent.rs", "mod", "Rust", 100, 50),
            make_child_row("child.rs", "mod", "Rust", 50, 25),
        ];
        let result = pack_greedy(&rows, 1000, ValueMetric::Code, None);
        // Only parent should be included
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "parent.rs");
    }

    #[test]
    fn test_pack_greedy_selects_by_value() {
        let rows = vec![
            make_test_row("low.rs", "mod", "Rust", 100, 10),
            make_test_row("high.rs", "mod", "Rust", 100, 90),
            make_test_row("mid.rs", "mod", "Rust", 100, 50),
        ];
        // Budget can fit all files (300 tokens total)
        let result = pack_greedy(&rows, 300, ValueMetric::Code, None);
        assert_eq!(result.len(), 3);
        // Should be sorted by value descending
        assert_eq!(result[0].path, "high.rs");
        assert_eq!(result[1].path, "mid.rs");
        assert_eq!(result[2].path, "low.rs");
    }

    #[test]
    fn test_pack_greedy_respects_token_budget() {
        let rows = vec![
            make_test_row("big.rs", "mod", "Rust", 500, 100),
            make_test_row("small.rs", "mod", "Rust", 100, 50),
        ];
        // Budget only allows small file
        let result = pack_greedy(&rows, 150, ValueMetric::Code, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "small.rs");
    }

    #[test]
    fn test_pack_greedy_accumulates_tokens() {
        let rows = vec![
            make_test_row("a.rs", "mod", "Rust", 100, 50),
            make_test_row("b.rs", "mod", "Rust", 100, 40),
            make_test_row("c.rs", "mod", "Rust", 100, 30),
        ];
        // Budget of 250 allows 2 files (200 tokens used)
        let result = pack_greedy(&rows, 250, ValueMetric::Code, None);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_pack_spread_empty_rows() {
        let rows: Vec<FileRow> = vec![];
        let result = pack_spread(&rows, 1000, ValueMetric::Code, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_pack_spread_filters_child_rows() {
        let rows = vec![
            make_test_row("parent.rs", "mod", "Rust", 100, 50),
            make_child_row("child.rs", "mod", "Rust", 50, 25),
        ];
        let result = pack_spread(&rows, 1000, ValueMetric::Code, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "parent.rs");
    }

    #[test]
    fn test_pack_spread_distributes_across_groups() {
        let rows = vec![
            make_test_row("rust1.rs", "mod1", "Rust", 100, 50),
            make_test_row("rust2.rs", "mod1", "Rust", 100, 60),
            make_test_row("python1.py", "mod2", "Python", 100, 70),
            make_test_row("python2.py", "mod2", "Python", 100, 80),
        ];
        // Large budget to fit all
        let result = pack_spread(&rows, 1000, ValueMetric::Code, None);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_pack_spread_round_robin_fills_70_percent() {
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 100, 50),
            make_test_row("b.py", "mod2", "Python", 100, 60),
        ];
        // Budget of 200 - spread uses 70% = 140 tokens
        let result = pack_spread(&rows, 200, ValueMetric::Code, None);
        // Both should fit in spread phase
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_pack_spread_greedy_fills_remaining_30_percent() {
        // Create files that exceed spread budget but fit in greedy phase
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 50, 50),
            make_test_row("b.py", "mod2", "Python", 50, 60),
            make_test_row("c.rs", "mod1", "Rust", 50, 40),
        ];
        // Budget 200, spread budget = 140
        // After spread: a.rs (50) + b.py (50) = 100 tokens
        // Greedy phase can add c.rs (50) since 100 + 50 = 150 <= 200
        let result = pack_spread(&rows, 200, ValueMetric::Code, None);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_select_files_greedy_strategy() {
        let rows = vec![make_test_row("a.rs", "mod", "Rust", 100, 50)];
        let result = select_files(
            &rows,
            1000,
            ContextStrategy::Greedy,
            ValueMetric::Code,
            None,
        );
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_select_files_spread_strategy() {
        let rows = vec![make_test_row("a.rs", "mod", "Rust", 100, 50)];
        let result = select_files(
            &rows,
            1000,
            ContextStrategy::Spread,
            ValueMetric::Code,
            None,
        );
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_to_context_row_fields() {
        let row = make_test_row("test.rs", "mymod", "Rust", 100, 50);
        let ctx_row = to_context_row(&row, ValueMetric::Code, None);

        assert_eq!(ctx_row.path, "test.rs");
        assert_eq!(ctx_row.module, "mymod");
        assert_eq!(ctx_row.lang, "Rust");
        assert_eq!(ctx_row.tokens, 100);
        assert_eq!(ctx_row.code, 50);
        assert_eq!(ctx_row.lines, 50);
        assert_eq!(ctx_row.bytes, 500);
        assert_eq!(ctx_row.value, 50); // Code metric
    }

    #[test]
    fn test_to_context_row_value_from_tokens_metric() {
        let row = make_test_row("test.rs", "mymod", "Rust", 200, 50);
        let ctx_row = to_context_row(&row, ValueMetric::Tokens, None);
        assert_eq!(ctx_row.value, 200);
    }

    #[test]
    fn test_pack_greedy_budget_boundary() {
        let rows = vec![make_test_row("exact.rs", "mod", "Rust", 100, 50)];
        // Budget exactly matches tokens
        let result = pack_greedy(&rows, 100, ValueMetric::Code, None);
        assert_eq!(result.len(), 1);

        // Budget one less than tokens
        let result = pack_greedy(&rows, 99, ValueMetric::Code, None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_pack_spread_skips_large_files_in_spread_phase() {
        let rows = vec![
            make_test_row("small.rs", "mod1", "Rust", 50, 100),
            make_test_row("large.rs", "mod2", "Rust", 200, 50), // Too large for spread budget
        ];
        // Budget 200, spread budget = 140
        // large.rs (200 tokens) > 140, so skipped in spread
        // small.rs (50 tokens) fits in spread
        // After spread, remaining budget = 200 - 50 = 150
        // large.rs (200) > 150, so doesn't fit in greedy either
        let result = pack_spread(&rows, 200, ValueMetric::Code, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "small.rs");
    }

    #[test]
    fn test_pack_spread_no_duplicates() {
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 50, 100),
            make_test_row("b.rs", "mod1", "Rust", 50, 90),
        ];
        let result = pack_spread(&rows, 500, ValueMetric::Code, None);
        // Both should be included exactly once
        let paths: Vec<_> = result.iter().map(|r| &r.path).collect();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&&"a.rs".to_string()));
        assert!(paths.contains(&&"b.rs".to_string()));
    }

    #[test]
    fn test_normalize_path_with_backslash() {
        // This tests the path normalization in get_value
        let row = FileRow {
            path: "foo\\bar\\test.rs".to_string(),
            module: "mod".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 50,
            comments: 0,
            blanks: 0,
            lines: 50,
            bytes: 500,
            tokens: 100,
        };

        // Create git scores with forward slash path
        let mut hotspots = BTreeMap::new();
        hotspots.insert("foo/bar/test.rs".to_string(), 999);
        let git_scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };

        // Should find the hotspot despite backslash in original path
        assert_eq!(
            get_value(&row, ValueMetric::Hotspot, Some(&git_scores)),
            999
        );
    }

    // =======================================================================
    // Mutant-killing tests: Hard invariants that catch replacement mutations
    // =======================================================================

    #[test]
    fn test_select_files_non_empty_when_budget_allows() {
        // Kills "select_files -> vec![]" mutant
        let rows = vec![make_test_row("small.rs", "src", "Rust", 50, 100)];
        let result = select_files(
            &rows,
            1000,
            ContextStrategy::Greedy,
            ValueMetric::Code,
            None,
        );
        assert!(
            !result.is_empty(),
            "select_files must return non-empty when files fit budget"
        );
    }

    #[test]
    fn test_budget_invariant_greedy() {
        // Kills arithmetic mutants: += -> -=, <= -> <, etc.
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 100, 50),
            make_test_row("b.rs", "mod2", "Rust", 150, 40),
            make_test_row("c.rs", "mod3", "Rust", 200, 30),
        ];
        let budget = 250;
        let result = pack_greedy(&rows, budget, ValueMetric::Code, None);
        let total_tokens: usize = result.iter().map(|r| r.tokens).sum();
        assert!(
            total_tokens <= budget,
            "Total tokens ({total_tokens}) must not exceed budget ({budget})"
        );
    }

    #[test]
    fn test_budget_invariant_spread() {
        // Kills arithmetic mutants in pack_spread
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 100, 50),
            make_test_row("b.rs", "mod2", "Python", 150, 40),
            make_test_row("c.rs", "mod3", "Go", 200, 30),
        ];
        let budget = 250;
        let result = pack_spread(&rows, budget, ValueMetric::Code, None);
        let total_tokens: usize = result.iter().map(|r| r.tokens).sum();
        assert!(
            total_tokens <= budget,
            "Total tokens ({total_tokens}) must not exceed budget ({budget})"
        );
    }

    #[test]
    fn test_parent_only_invariant_greedy() {
        // Kills kind filter mutants: == -> != for FileKind::Parent
        let rows = vec![
            make_test_row("parent1.rs", "mod", "Rust", 100, 50),
            make_child_row("child1.rs", "mod", "Rust", 50, 25),
            make_test_row("parent2.rs", "mod", "Rust", 100, 40),
            make_child_row("child2.rs", "mod", "Rust", 50, 20),
        ];
        let result = pack_greedy(&rows, 1000, ValueMetric::Code, None);

        // All selected files must be parent files
        for ctx_row in &result {
            let original = rows.iter().find(|r| r.path == ctx_row.path).unwrap();
            assert_eq!(
                original.kind,
                FileKind::Parent,
                "Selected file {} must be a Parent, not a Child",
                ctx_row.path
            );
        }

        // And we should have selected parents
        assert_eq!(result.len(), 2, "Should select both parent files");
    }

    #[test]
    fn test_parent_only_invariant_spread() {
        // Same invariant test for spread strategy
        let rows = vec![
            make_test_row("parent1.rs", "mod1", "Rust", 100, 50),
            make_child_row("child1.rs", "mod1", "Rust", 50, 25),
            make_test_row("parent2.py", "mod2", "Python", 100, 40),
            make_child_row("child2.py", "mod2", "Python", 50, 20),
        ];
        let result = pack_spread(&rows, 1000, ValueMetric::Code, None);

        for ctx_row in &result {
            let original = rows.iter().find(|r| r.path == ctx_row.path).unwrap();
            assert_eq!(
                original.kind,
                FileKind::Parent,
                "Selected file {} must be a Parent",
                ctx_row.path
            );
        }
    }

    #[test]
    fn test_determinism_greedy() {
        // Kills ordering / tie-break mutants
        let rows = vec![
            make_test_row("a.rs", "mod", "Rust", 100, 50),
            make_test_row("b.rs", "mod", "Rust", 100, 50), // Same value as a.rs
            make_test_row("c.rs", "mod", "Rust", 100, 30),
        ];

        let result1 = pack_greedy(&rows, 1000, ValueMetric::Code, None);
        let result2 = pack_greedy(&rows, 1000, ValueMetric::Code, None);

        let paths1: Vec<_> = result1.iter().map(|r| &r.path).collect();
        let paths2: Vec<_> = result2.iter().map(|r| &r.path).collect();

        assert_eq!(paths1, paths2, "pack_greedy must be deterministic");
    }

    #[test]
    fn test_determinism_spread() {
        // Kills ordering / tie-break mutants in spread
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 100, 50),
            make_test_row("b.py", "mod2", "Python", 100, 50),
            make_test_row("c.rs", "mod1", "Rust", 100, 50),
        ];

        let result1 = pack_spread(&rows, 1000, ValueMetric::Code, None);
        let result2 = pack_spread(&rows, 1000, ValueMetric::Code, None);

        let paths1: Vec<_> = result1.iter().map(|r| &r.path).collect();
        let paths2: Vec<_> = result2.iter().map(|r| &r.path).collect();

        assert_eq!(paths1, paths2, "pack_spread must be deterministic");
    }

    #[test]
    fn test_tiebreaker_by_path() {
        // Kills "wrong comparator" mutants - when values equal, sort by path
        let rows = vec![
            make_test_row("z.rs", "mod", "Rust", 100, 50),
            make_test_row("a.rs", "mod", "Rust", 100, 50), // Same value, earlier path
        ];

        let result = pack_greedy(&rows, 1000, ValueMetric::Code, None);

        // With same values, should be sorted by path alphabetically
        assert_eq!(
            result[0].path, "a.rs",
            "Files with equal value should tie-break by path"
        );
        assert_eq!(result[1].path, "z.rs");
    }

    #[test]
    fn test_spread_distributes_before_greedy_fill() {
        // Kills "spread vs greedy collapse" mutants
        // Two modules, each with high-value files. Spread should pick from both before greedy fill.
        let rows = vec![
            // Module 1: high value file
            make_test_row("mod1/best.rs", "mod1", "Rust", 50, 100),
            make_test_row("mod1/okay.rs", "mod1", "Rust", 50, 20),
            // Module 2: high value file
            make_test_row("mod2/best.py", "mod2", "Python", 50, 100),
            make_test_row("mod2/okay.py", "mod2", "Python", 50, 20),
        ];

        // Budget allows all files (200 tokens total, budget 300)
        let spread_result = pack_spread(&rows, 300, ValueMetric::Code, None);

        // First two picks should be the best from each module (round-robin)
        let first_two: Vec<_> = spread_result.iter().take(2).map(|r| &r.path).collect();

        // Both best files should be in the first two picks
        assert!(
            first_two.contains(&&"mod1/best.rs".to_string()),
            "Spread should pick best from mod1 early"
        );
        assert!(
            first_two.contains(&&"mod2/best.py".to_string()),
            "Spread should pick best from mod2 early"
        );
    }

    #[test]
    fn test_greedy_picks_highest_value_first() {
        // Kills value comparison mutants: vb.cmp(&va) -> va.cmp(&vb)
        let rows = vec![
            make_test_row("low.rs", "mod", "Rust", 100, 10),
            make_test_row("high.rs", "mod", "Rust", 100, 90),
        ];

        let result = pack_greedy(&rows, 100, ValueMetric::Code, None);

        // Should pick high.rs (value 90) not low.rs (value 10)
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].path, "high.rs",
            "Greedy should pick highest value file"
        );
        assert_eq!(result[0].value, 90);
    }

    #[test]
    fn test_pack_greedy_skips_files_that_dont_fit() {
        // Kills boundary condition mutants: <= -> <, + -> -
        let rows = vec![
            make_test_row("big.rs", "mod", "Rust", 200, 100), // Won't fit
            make_test_row("small.rs", "mod", "Rust", 50, 10), // Will fit
        ];

        // Budget 100: big.rs (200 tokens) doesn't fit, small.rs (50) does
        let result = pack_greedy(&rows, 100, ValueMetric::Code, None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "small.rs");
    }

    #[test]
    fn test_select_files_routes_to_correct_strategy() {
        // Kills strategy routing mutants
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 50, 50),
            make_test_row("b.py", "mod2", "Python", 50, 50),
        ];

        let greedy = select_files(&rows, 100, ContextStrategy::Greedy, ValueMetric::Code, None);
        let spread = select_files(&rows, 100, ContextStrategy::Spread, ValueMetric::Code, None);

        // Both should return results
        assert!(!greedy.is_empty(), "Greedy strategy should return results");
        assert!(!spread.is_empty(), "Spread strategy should return results");
    }

    #[test]
    fn test_to_context_row_preserves_all_fields() {
        // Kills field mapping mutants in to_context_row
        let row = FileRow {
            path: "test/path.rs".to_string(),
            module: "test_mod".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 42,
            comments: 10,
            blanks: 5,
            lines: 57,
            bytes: 1234,
            tokens: 99,
        };

        let ctx = to_context_row(&row, ValueMetric::Code, None);

        assert_eq!(ctx.path, "test/path.rs");
        assert_eq!(ctx.module, "test_mod");
        assert_eq!(ctx.lang, "Rust");
        assert_eq!(ctx.tokens, 99);
        assert_eq!(ctx.code, 42);
        assert_eq!(ctx.lines, 57);
        assert_eq!(ctx.bytes, 1234);
        assert_eq!(ctx.value, 42); // Code metric
    }

    #[test]
    fn test_churn_metric_formula() {
        // Kills formula mutants: commits * 1000 + code
        let row = make_test_row("test.rs", "mod", "Rust", 100, 7);
        let mut commit_counts = BTreeMap::new();
        commit_counts.insert("test.rs".to_string(), 3);
        let git_scores = GitScores {
            hotspots: BTreeMap::new(),
            commit_counts,
        };

        // Expected: 3 * 1000 + 7 = 3007
        assert_eq!(get_value(&row, ValueMetric::Churn, Some(&git_scores)), 3007);
    }

    #[test]
    fn test_hotspot_metric_multiplication() {
        // This tests the hotspot value is retrieved correctly
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        let mut hotspots = BTreeMap::new();
        // Hotspot = lines * commits, pre-computed
        hotspots.insert("test.rs".to_string(), 150); // 50 lines * 3 commits
        let git_scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };

        assert_eq!(
            get_value(&row, ValueMetric::Hotspot, Some(&git_scores)),
            150
        );
    }

    #[test]
    fn test_spread_70_percent_allocation() {
        // Kills the 0.7 constant mutation
        // With budget 1000, spread budget should be 700
        let rows = vec![
            make_test_row("a.rs", "mod1", "Rust", 350, 50),
            make_test_row("b.rs", "mod1", "Rust", 350, 40),
            make_test_row("c.py", "mod2", "Python", 350, 30),
        ];

        // Budget 1000, spread = 700
        // After spread phase (700 budget): can fit 2 files (700 tokens)
        // Greedy phase can add the third if total <= 1000
        let result = pack_spread(&rows, 1000, ValueMetric::Code, None);

        // With these token sizes, should get 2 files in spread, possibly 1 more in greedy
        assert!(result.len() >= 2, "Should select at least 2 files");
    }

    #[test]
    fn test_normalize_path_preserves_forward_slashes() {
        // Kills "normalize_path -> empty string" mutant
        assert_eq!(normalize_path("foo/bar/baz.rs"), "foo/bar/baz.rs");
        assert!(!normalize_path("test.rs").is_empty());
    }

    #[test]
    fn test_normalize_path_not_xyzzy() {
        // Kills "normalize_path -> xyzzy" mutant
        assert_ne!(normalize_path("foo/bar"), "xyzzy");
        assert_ne!(normalize_path("test.rs"), "xyzzy");
    }

    // ==================== parse_budget error cases ====================

    #[test]
    fn test_parse_budget_invalid_alpha() {
        assert!(parse_budget("abc").is_err());
    }

    #[test]
    fn test_parse_budget_invalid_empty() {
        assert!(parse_budget("").is_err());
    }

    #[test]
    fn test_parse_budget_invalid_suffix_only() {
        assert!(parse_budget("k").is_err());
        assert!(parse_budget("m").is_err());
    }

    // ==================== All-child rows produce empty result ====================

    #[test]
    fn test_pack_greedy_all_children_empty() {
        let rows = vec![
            make_child_row("a.rs", "mod", "Rust", 100, 50),
            make_child_row("b.rs", "mod", "Rust", 100, 50),
        ];
        let result = pack_greedy(&rows, 1000, ValueMetric::Code, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_pack_spread_all_children_empty() {
        let rows = vec![
            make_child_row("a.rs", "mod", "Rust", 100, 50),
            make_child_row("b.rs", "mod", "Rust", 100, 50),
        ];
        let result = pack_spread(&rows, 1000, ValueMetric::Code, None);
        assert!(result.is_empty());
    }

    // ==================== select_files with git_scores ====================

    #[test]
    fn test_select_files_greedy_with_git_scores() {
        let rows = vec![make_test_row("a.rs", "mod", "Rust", 100, 50)];
        let mut hotspots = BTreeMap::new();
        hotspots.insert("a.rs".to_string(), 999);
        let git_scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };
        let result = select_files(
            &rows,
            1000,
            ContextStrategy::Greedy,
            ValueMetric::Hotspot,
            Some(&git_scores),
        );
        assert_eq!(result[0].value, 999);
    }

    #[test]
    fn test_select_files_spread_with_git_scores() {
        let rows = vec![make_test_row("a.rs", "mod", "Rust", 100, 50)];
        let mut commit_counts = BTreeMap::new();
        commit_counts.insert("a.rs".to_string(), 5);
        let git_scores = GitScores {
            hotspots: BTreeMap::new(),
            commit_counts,
        };
        let result = select_files(
            &rows,
            1000,
            ContextStrategy::Spread,
            ValueMetric::Churn,
            Some(&git_scores),
        );
        // 5 * 1000 + 50 = 5050
        assert_eq!(result[0].value, 5050);
    }

    // ==================== to_context_row value field tests ====================

    #[test]
    fn test_to_context_row_value_hotspot() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        let mut hotspots = BTreeMap::new();
        hotspots.insert("test.rs".to_string(), 777);
        let git_scores = GitScores {
            hotspots,
            commit_counts: BTreeMap::new(),
        };
        let ctx_row = to_context_row(&row, ValueMetric::Hotspot, Some(&git_scores));
        assert_eq!(ctx_row.value, 777);
    }

    #[test]
    fn test_to_context_row_value_churn() {
        let row = make_test_row("test.rs", "mod", "Rust", 100, 50);
        let mut commit_counts = BTreeMap::new();
        commit_counts.insert("test.rs".to_string(), 3);
        let git_scores = GitScores {
            hotspots: BTreeMap::new(),
            commit_counts,
        };
        let ctx_row = to_context_row(&row, ValueMetric::Churn, Some(&git_scores));
        // 3 * 1000 + 50 = 3050
        assert_eq!(ctx_row.value, 3050);
    }

    // ==================== Churn formula exact verification ====================

    #[test]
    fn test_get_value_churn_formula_exact() {
        // Verify exact formula: commits * 1000 + code
        let row = make_test_row("test.rs", "mod", "Rust", 100, 42);
        let mut commit_counts = BTreeMap::new();
        commit_counts.insert("test.rs".to_string(), 7);
        let git_scores = GitScores {
            hotspots: BTreeMap::new(),
            commit_counts,
        };
        // 7 * 1000 + 42 = 7042 (NOT 7 + 42 = 49, NOT 7 * 100 + 42 = 742)
        assert_eq!(get_value(&row, ValueMetric::Churn, Some(&git_scores)), 7042);
    }

    // ==================== Spread 70% boundary ====================

    #[test]
    fn test_pack_spread_70_percent_exact() {
        // Budget 100, spread = 70
        // File with 70 tokens should fit in spread phase
        let rows = vec![make_test_row("a.rs", "mod1", "Rust", 70, 50)];
        let result = pack_spread(&rows, 100, ValueMetric::Code, None);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_pack_spread_71_percent_needs_greedy() {
        // Budget 100, spread = 70, file = 71 tokens
        // Should NOT fit in spread, but DOES fit in greedy (total budget 100)
        let rows = vec![make_test_row("a.rs", "mod1", "Rust", 71, 50)];
        let result = pack_spread(&rows, 100, ValueMetric::Code, None);
        assert_eq!(result.len(), 1); // Added in greedy fill phase
    }

    // ==================== Sorting tie-breaker by path ====================

    #[test]
    fn test_pack_greedy_tiebreaker_by_path_explicit() {
        // All files have same value and tokens - should sort by path alphabetically
        let rows = vec![
            make_test_row("zzz.rs", "mod", "Rust", 100, 50),
            make_test_row("aaa.rs", "mod", "Rust", 100, 50),
            make_test_row("mmm.rs", "mod", "Rust", 100, 50),
        ];
        let result = pack_greedy(&rows, 300, ValueMetric::Code, None);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].path, "aaa.rs");
        assert_eq!(result[1].path, "mmm.rs");
        assert_eq!(result[2].path, "zzz.rs");
    }
}
