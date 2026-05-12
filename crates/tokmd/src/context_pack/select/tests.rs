use super::*;
use std::collections::BTreeMap;

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
    assert_eq!(ctx_row.rank_reason, ""); // Default empty
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
        let original = rows
            .iter()
            .find(|r| r.path == ctx_row.path)
            .expect("failed to extract value that should be present by construction");
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
        let original = rows
            .iter()
            .find(|r| r.path == ctx_row.path)
            .expect("failed to extract value that should be present by construction");
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
fn test_pack_spread_module_first_avoids_language_bias() {
    let rows = vec![
        make_test_row("mod1/a.rs", "mod1", "Rust", 50, 100),
        make_test_row("mod1/a.py", "mod1", "Python", 50, 90),
        make_test_row("mod2/b.rs", "mod2", "Rust", 50, 80),
    ];

    let result = pack_spread(&rows, 200, ValueMetric::Code, None);
    let first_two: Vec<_> = result.iter().take(2).map(|r| r.path.as_str()).collect();

    assert!(
        first_two.iter().any(|p| p.starts_with("mod1/")),
        "First two picks should include mod1"
    );
    assert!(
        first_two.iter().any(|p| p.starts_with("mod2/")),
        "First two picks should include mod2"
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

// ==================== Smart exclude tests ====================

#[test]
fn test_is_smart_excluded_lockfiles() {
    assert_eq!(is_smart_excluded("Cargo.lock"), Some("lockfile"));
    assert_eq!(is_smart_excluded("package-lock.json"), Some("lockfile"));
    assert_eq!(is_smart_excluded("yarn.lock"), Some("lockfile"));
    assert_eq!(is_smart_excluded("go.sum"), Some("lockfile"));
    assert_eq!(is_smart_excluded("poetry.lock"), Some("lockfile"));
    assert_eq!(is_smart_excluded("Gemfile.lock"), Some("lockfile"));
    assert_eq!(is_smart_excluded("some/dir/Cargo.lock"), Some("lockfile"));
}

#[test]
fn test_is_smart_excluded_minified() {
    assert_eq!(is_smart_excluded("app.min.js"), Some("minified"));
    assert_eq!(is_smart_excluded("style.min.css"), Some("minified"));
    assert_eq!(is_smart_excluded("vendor/app.min.js"), Some("minified"));
}

#[test]
fn test_is_smart_excluded_sourcemaps() {
    assert_eq!(is_smart_excluded("app.js.map"), Some("sourcemap"));
    assert_eq!(is_smart_excluded("style.css.map"), Some("sourcemap"));
}

#[test]
fn test_is_smart_excluded_normal_files() {
    assert_eq!(is_smart_excluded("main.rs"), None);
    assert_eq!(is_smart_excluded("Cargo.toml"), None);
    assert_eq!(is_smart_excluded("app.js"), None);
    assert_eq!(is_smart_excluded("style.css"), None);
}

#[test]
fn test_is_spine_file() {
    assert!(is_spine_file("README.md"));
    assert!(is_spine_file("Cargo.toml"));
    assert!(is_spine_file("ROADMAP.md"));
    assert!(is_spine_file("CONTRIBUTING.md"));
    assert!(is_spine_file("package.json"));
    assert!(is_spine_file("docs/architecture.md"));
    assert!(is_spine_file("some/path/README.md"));
    assert!(!is_spine_file("src/main.rs"));
    assert!(!is_spine_file("README_backup.md"));
}

#[test]
fn test_select_files_with_options_smart_exclude() {
    let rows = vec![
        make_test_row("src/main.rs", "src", "Rust", 100, 50),
        make_test_row("Cargo.lock", ".", "TOML", 500, 200),
        make_test_row("src/lib.rs", "src", "Rust", 100, 40),
    ];
    let result = select_files_with_options(
        &rows,
        1000,
        ContextStrategy::Greedy,
        ValueMetric::Code,
        None,
        &SelectOptions {
            no_smart_exclude: false,
            ..Default::default()
        },
    );

    // Cargo.lock should be smart-excluded
    assert_eq!(result.smart_excluded.len(), 1);
    assert_eq!(result.smart_excluded[0].path, "Cargo.lock");
    assert_eq!(result.smart_excluded[0].reason, "lockfile");

    // Only non-excluded files selected
    let paths: Vec<&str> = result.selected.iter().map(|r| r.path.as_str()).collect();
    assert!(!paths.contains(&"Cargo.lock"));
    assert!(paths.contains(&"src/main.rs"));
    assert!(paths.contains(&"src/lib.rs"));
}

#[test]
fn test_select_files_with_options_no_smart_exclude() {
    let rows = vec![
        make_test_row("src/main.rs", "src", "Rust", 100, 50),
        make_test_row("Cargo.lock", ".", "TOML", 100, 200),
    ];
    let result = select_files_with_options(
        &rows,
        1000,
        ContextStrategy::Greedy,
        ValueMetric::Code,
        None,
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    );

    // No smart excludes when disabled
    assert!(result.smart_excluded.is_empty());
    assert_eq!(result.selected.len(), 2);
}

#[test]
fn test_select_files_with_options_spine_reservation() {
    let rows = vec![
        make_test_row("README.md", ".", "Markdown", 50, 30),
        make_test_row("src/big.rs", "src", "Rust", 100, 100),
        make_test_row("src/small.rs", "src", "Rust", 50, 50),
    ];
    // Budget 2000: spine_budget = min(2000*0.05, 5000) = 100
    // README.md (50 tokens) fits in spine budget
    let result = select_files_with_options(
        &rows,
        2000,
        ContextStrategy::Greedy,
        ValueMetric::Code,
        None,
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    );

    // README.md should be spine-reserved and appear first
    assert!(!result.selected.is_empty());
    let readme_entry = result.selected.iter().find(|f| f.path == "README.md");
    assert!(
        readme_entry.is_some(),
        "README.md should be in selected files"
    );
    assert_eq!(
        readme_entry
            .expect("failed to extract value that should be present by construction")
            .rank_reason,
        "spine"
    );
}

#[test]
fn test_select_files_with_options_rank_reason() {
    let rows = vec![make_test_row("src/main.rs", "src", "Rust", 100, 50)];
    let result = select_files_with_options(
        &rows,
        1000,
        ContextStrategy::Greedy,
        ValueMetric::Code,
        None,
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    );

    assert_eq!(result.selected.len(), 1);
    assert_eq!(result.selected[0].rank_reason, "code");
}

#[test]
fn test_select_files_with_options_rank_reason_hotspot_fallback() {
    // Without git scores, hotspot falls back to code
    let rows = vec![make_test_row("src/main.rs", "src", "Rust", 100, 50)];
    let result = select_files_with_options(
        &rows,
        1000,
        ContextStrategy::Greedy,
        ValueMetric::Hotspot,
        None,
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    );

    // Effective metric is "code" due to fallback
    assert_eq!(result.selected[0].rank_reason, "code");
    assert_eq!(result.rank_by_effective, "code");
    assert!(result.fallback_reason.is_some());
    assert!(
        result
            .fallback_reason
            .as_ref()
            .expect("failed to extract value that should be present by construction")
            .contains("hotspot")
    );
}

#[test]
fn test_select_files_with_options_rank_reason_hotspot_with_git() {
    // With git scores, hotspot metric is preserved
    let rows = vec![make_test_row("src/main.rs", "src", "Rust", 100, 50)];
    let mut hotspots = BTreeMap::new();
    hotspots.insert("src/main.rs".to_string(), 999);
    let git_scores = GitScores {
        hotspots,
        commit_counts: BTreeMap::new(),
    };
    let result = select_files_with_options(
        &rows,
        1000,
        ContextStrategy::Greedy,
        ValueMetric::Hotspot,
        Some(&git_scores),
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    );

    assert_eq!(result.selected[0].rank_reason, "hotspot");
    assert_eq!(result.rank_by_effective, "hotspot");
    assert!(result.fallback_reason.is_none());
}

// ==================== Classification tests ====================

#[test]
fn test_classify_lockfile() {
    let classes = classify_file("Cargo.lock", 1000, 100, 50.0);
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn test_classify_nested_lockfile() {
    let classes = classify_file("some/dir/package-lock.json", 1000, 100, 50.0);
    assert!(classes.contains(&FileClassification::Lockfile));
}

#[test]
fn test_classify_generated() {
    let classes = classify_file("src/parser/node-types.json", 5000, 10, 50.0);
    assert!(classes.contains(&FileClassification::Generated));
    // Also DataBlob due to high tokens/line
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn test_classify_generated_pb_rs() {
    let classes = classify_file("proto/types.pb.rs", 1000, 200, 50.0);
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn test_classify_vendored() {
    let classes = classify_file("vendor/github.com/lib/pq/conn.go", 500, 100, 50.0);
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn test_classify_fixture() {
    let classes = classify_file("tests/fixtures/sample.json", 200, 50, 50.0);
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn test_classify_minified() {
    let classes = classify_file("dist/app.min.js", 50000, 1, 50.0);
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn test_classify_sourcemap() {
    let classes = classify_file("dist/app.js.map", 30000, 1, 50.0);
    assert!(classes.contains(&FileClassification::Sourcemap));
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn test_classify_dense_blob() {
    // 1000 tokens / 10 lines = 100 tpl > 50 threshold
    let classes = classify_file("src/data.rs", 1000, 10, 50.0);
    assert!(classes.contains(&FileClassification::DataBlob));
}

#[test]
fn test_classify_normal_file() {
    // 100 tokens / 50 lines = 2 tpl < 50 threshold
    let classes = classify_file("src/main.rs", 100, 50, 50.0);
    assert!(classes.is_empty());
}

// ==================== Compute file cap tests ====================

#[test]
fn test_compute_file_cap_default() {
    let opts = SelectOptions::default();
    // budget 128000 * 0.15 = 19200, min(19200, 16000) = 16000
    let cap = compute_file_cap(128_000, &opts);
    assert_eq!(cap, 16_000);
}

#[test]
fn test_compute_file_cap_small_budget() {
    let opts = SelectOptions::default();
    // budget 10000 * 0.15 = 1500, min(1500, 16000) = 1500
    let cap = compute_file_cap(10_000, &opts);
    assert_eq!(cap, 1_500);
}

#[test]
fn test_compute_file_cap_custom() {
    let opts = SelectOptions {
        max_file_pct: 0.25,
        max_file_tokens: Some(5_000),
        ..Default::default()
    };
    // budget 100000 * 0.25 = 25000, min(25000, 5000) = 5000
    let cap = compute_file_cap(100_000, &opts);
    assert_eq!(cap, 5_000);
}

#[test]
fn test_compute_file_cap_unlimited_budget() {
    let opts = SelectOptions::default();
    let cap = compute_file_cap(usize::MAX, &opts);
    assert_eq!(cap, usize::MAX);
}

// ==================== Assign policy tests ====================

#[test]
fn test_assign_policy_under_cap_is_full() {
    let (policy, reason) = assign_policy(100, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());
}

#[test]
fn test_assign_policy_over_cap_normal_is_head_tail() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
    assert!(
        reason
            .expect("failed to extract value that should be present by construction")
            .contains("head+tail")
    );
}

#[test]
fn test_assign_policy_over_cap_generated_is_skip() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Generated]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.is_some());
    assert!(
        reason
            .expect("failed to extract value that should be present by construction")
            .contains("generated")
    );
}

#[test]
fn test_assign_policy_over_cap_data_blob_is_skip() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::DataBlob]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.is_some());
}

#[test]
fn test_assign_policy_over_cap_vendored_is_skip() {
    let (policy, reason) = assign_policy(20_000, 16_000, &[FileClassification::Vendored]);
    assert_eq!(policy, InclusionPolicy::Skip);
    assert!(reason.is_some());
}

#[test]
fn test_assign_policy_over_cap_fixture_is_head_tail() {
    // Fixtures over cap get HeadTail, not Skip (fixture is not in the skip list)
    let (policy, _) = assign_policy(20_000, 16_000, &[FileClassification::Fixture]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

// ==================== Resolve metric tests ====================

#[test]
fn test_resolve_metric_code_no_fallback() {
    let resolved = resolve_metric(ValueMetric::Code, None);
    assert_eq!(resolved.effective, ValueMetric::Code);
    assert!(resolved.fallback_reason.is_none());
}

#[test]
fn test_resolve_metric_hotspot_falls_back() {
    let resolved = resolve_metric(ValueMetric::Hotspot, None);
    assert_eq!(resolved.effective, ValueMetric::Code);
    assert!(resolved.fallback_reason.is_some());
    assert!(
        resolved
            .fallback_reason
            .expect("failed to extract value that should be present by construction")
            .contains("hotspot")
    );
}

#[test]
fn test_resolve_metric_churn_falls_back() {
    let resolved = resolve_metric(ValueMetric::Churn, None);
    assert_eq!(resolved.effective, ValueMetric::Code);
    assert!(resolved.fallback_reason.is_some());
    assert!(
        resolved
            .fallback_reason
            .expect("failed to extract value that should be present by construction")
            .contains("churn")
    );
}

#[test]
fn test_resolve_metric_hotspot_with_git_no_fallback() {
    let git_scores = GitScores {
        hotspots: BTreeMap::new(),
        commit_counts: BTreeMap::new(),
    };
    let resolved = resolve_metric(ValueMetric::Hotspot, Some(&git_scores));
    assert_eq!(resolved.effective, ValueMetric::Hotspot);
    assert!(resolved.fallback_reason.is_none());
}

// ==================== Budget uses effective tokens ====================

#[test]
fn test_budget_uses_effective_tokens() {
    // A 20k-token file should be capped to fit within budget
    // With default max_file_pct=0.15, budget=128k → cap=16000
    let rows = vec![
        make_test_row("big.rs", "src", "Rust", 20_000, 1000),
        make_test_row("small.rs", "src", "Rust", 100, 50),
    ];
    let result = select_files_with_options(
        &rows,
        128_000,
        ContextStrategy::Greedy,
        ValueMetric::Code,
        None,
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    );

    // Both should be selected
    assert_eq!(result.selected.len(), 2);

    // big.rs should have HeadTail policy
    let big = result
        .selected
        .iter()
        .find(|f| f.path == "big.rs")
        .expect("failed to extract value that should be present by construction");
    assert_eq!(big.policy, InclusionPolicy::HeadTail);
    assert!(big.effective_tokens.is_some());
    assert!(
        big.effective_tokens
            .expect("failed to extract value that should be present by construction")
            <= 16_000
    );

    // small.rs should have Full policy
    let small = result
        .selected
        .iter()
        .find(|f| f.path == "small.rs")
        .expect("failed to extract value that should be present by construction");
    assert_eq!(small.policy, InclusionPolicy::Full);
    assert!(small.effective_tokens.is_none());
}

// ==================== Policy-excluded files tracked ====================

#[test]
fn test_generated_over_cap_excluded_by_policy() {
    // node-types.json with very high tokens → should be skip'd
    let rows = vec![
        make_test_row("src/main.rs", "src", "Rust", 100, 50),
        FileRow {
            path: "src/parser/node-types.json".to_string(),
            module: "src".to_string(),
            lang: "JSON".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 500_000,
            tokens: 117_000,
        },
    ];
    let result = select_files_with_options(
        &rows,
        128_000,
        ContextStrategy::Greedy,
        ValueMetric::Code,
        None,
        &SelectOptions {
            no_smart_exclude: true,
            ..Default::default()
        },
    );

    // node-types.json should be excluded by policy
    assert_eq!(result.excluded_by_policy.len(), 1);
    assert!(
        result.excluded_by_policy[0]
            .path
            .contains("node-types.json")
    );
    assert_eq!(result.excluded_by_policy[0].policy, InclusionPolicy::Skip);

    // main.rs should be selected
    assert!(result.selected.iter().any(|f| f.path == "src/main.rs"));
    // node-types.json should NOT be in selected
    assert!(
        !result
            .selected
            .iter()
            .any(|f| f.path.contains("node-types.json"))
    );
}
