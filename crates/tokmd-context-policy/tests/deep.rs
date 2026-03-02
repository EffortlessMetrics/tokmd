//! Deep integration tests for context policy pipeline.
//!
//! Covers: inclusion/exclusion decisions, smart excludes, file classifications,
//! token budget management, pack plan construction, determinism invariants,
//! and edge cases (zero budget, single file, all excluded).

use std::collections::BTreeMap;

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, DEFAULT_MAX_FILE_TOKENS, assign_policy,
    classify_file, compute_file_cap, is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Simulated file entry for pack-plan construction tests.
struct SimFile {
    path: &'static str,
    tokens: usize,
    lines: usize,
}

/// Run the full policy pipeline (classify → cap → assign) and return decision.
fn pipeline(path: &str, tokens: usize, lines: usize, budget: usize) -> InclusionPolicy {
    let classes = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    let (policy, _) = assign_policy(tokens, cap, &classes);
    policy
}

// ===========================================================================
// 1. Context policy: inclusion/exclusion decisions
// ===========================================================================

#[test]
fn full_pipeline_includes_small_source_files() {
    assert_eq!(
        pipeline("src/lib.rs", 500, 100, 128_000),
        InclusionPolicy::Full
    );
    assert_eq!(
        pipeline("app/main.py", 2_000, 300, 128_000),
        InclusionPolicy::Full
    );
}

#[test]
fn full_pipeline_head_tails_oversized_regular_files() {
    // A normal file that exceeds the per-file cap → HeadTail
    let policy = pipeline("src/big_module.rs", 20_000, 2_000, 100_000);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

#[test]
fn full_pipeline_skips_generated_files_exceeding_cap() {
    let policy = pipeline("api/service.pb.go", 40_000, 8_000, 100_000);
    assert_eq!(policy, InclusionPolicy::Skip);
}

#[test]
fn full_pipeline_skips_vendored_dense_blobs() {
    let policy = pipeline("vendor/lib/react.min.js", 80_000, 2, 128_000);
    assert_eq!(policy, InclusionPolicy::Skip);
}

#[test]
fn full_pipeline_head_tails_oversized_fixture_without_blob() {
    // Fixture files are NOT in the skip class (only Generated, DataBlob, Vendored are)
    let policy = pipeline("tests/fixtures/large.json", 25_000, 5_000, 100_000);
    assert_eq!(policy, InclusionPolicy::HeadTail);
}

// ===========================================================================
// 2. Smart exclude decisions are orthogonal to classification
// ===========================================================================

#[test]
fn smart_excludes_cover_all_lockfile_ecosystems() {
    let lockfiles = [
        "Cargo.lock",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "poetry.lock",
        "Pipfile.lock",
        "go.sum",
        "composer.lock",
        "Gemfile.lock",
    ];
    for lf in lockfiles {
        assert_eq!(
            smart_exclude_reason(lf),
            Some("lockfile"),
            "expected lockfile for {lf}"
        );
    }
}

#[test]
fn smart_excludes_detect_minified_and_sourcemaps_at_any_depth() {
    let cases = [
        ("dist/app.min.js", "minified"),
        ("deep/nested/path/bundle.min.css", "minified"),
        ("build/app.js.map", "sourcemap"),
        ("static/styles.css.map", "sourcemap"),
    ];
    for (path, expected) in cases {
        assert_eq!(
            smart_exclude_reason(path),
            Some(expected),
            "expected {expected} for {path}"
        );
    }
}

#[test]
fn smart_exclude_returns_none_for_normal_source_files() {
    let normals = [
        "src/main.rs",
        "lib/utils.py",
        "app/controller.go",
        "README.md",
        "Cargo.toml",
    ];
    for path in normals {
        assert_eq!(smart_exclude_reason(path), None, "expected None for {path}");
    }
}

// ===========================================================================
// 3. File classification exhaustive coverage
// ===========================================================================

#[test]
fn classify_detects_all_generated_patterns() {
    let generated = [
        "src/node-types.json",
        "api/types.pb.go",
        "api/types.pb.rs",
        "api/types_pb2.py",
        "lib/model.g.dart",
        "lib/model.freezed.dart",
        "src/schema.generated.ts",
    ];
    for path in generated {
        let classes = classify_file(path, 500, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Generated),
            "expected Generated for {path}, got {classes:?}"
        );
    }
}

#[test]
fn classify_detects_all_vendored_directories() {
    let vendored = [
        "vendor/lib/foo.rs",
        "third_party/sqlite/sqlite3.c",
        "third-party/lib/bar.js",
        "node_modules/react/index.js",
    ];
    for path in vendored {
        let classes = classify_file(path, 500, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Vendored),
            "expected Vendored for {path}, got {classes:?}"
        );
    }
}

#[test]
fn classify_detects_all_fixture_directories() {
    let fixtures = [
        "tests/fixtures/sample.json",
        "testdata/input.txt",
        "test_data/config.yaml",
        "__snapshots__/Component.test.js.snap",
        "golden/expected.txt",
    ];
    for path in fixtures {
        let classes = classify_file(path, 500, 100, DEFAULT_DENSE_THRESHOLD);
        assert!(
            classes.contains(&FileClassification::Fixture),
            "expected Fixture for {path}, got {classes:?}"
        );
    }
}

#[test]
fn classify_multiple_classes_on_same_file() {
    // A vendored minified dense file
    let classes = classify_file(
        "vendor/lib/react.min.js",
        100_000,
        1,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Minified));
    assert!(classes.contains(&FileClassification::DataBlob));
    assert_eq!(classes.len(), 3);
}

#[test]
fn classify_returns_empty_for_normal_source_file() {
    let classes = classify_file("src/main.rs", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.is_empty());
}

#[test]
fn classify_dense_blob_threshold_boundary() {
    // Exactly at threshold: tokens_per_line = 50/1 = 50.0, NOT > 50.0
    let at_boundary = classify_file("data.bin", 50, 1, 50.0);
    assert!(
        !at_boundary.contains(&FileClassification::DataBlob),
        "at threshold should not be DataBlob"
    );

    // Just above threshold: 51/1 = 51.0 > 50.0
    let above = classify_file("data.bin", 51, 1, 50.0);
    assert!(
        above.contains(&FileClassification::DataBlob),
        "above threshold should be DataBlob"
    );
}

// ===========================================================================
// 4. Token budget management
// ===========================================================================

#[test]
fn file_cap_respects_percentage_and_hard_cap() {
    // budget=100_000, pct=0.15 → 15_000, hard_cap=16_000 → min(15_000, 16_000) = 15_000
    let cap = compute_file_cap(100_000, 0.15, Some(16_000));
    assert_eq!(cap, 15_000);

    // budget=200_000, pct=0.15 → 30_000, hard_cap=16_000 → min(30_000, 16_000) = 16_000
    let cap = compute_file_cap(200_000, 0.15, Some(16_000));
    assert_eq!(cap, 16_000);
}

#[test]
fn file_cap_with_zero_budget_is_zero() {
    let cap = compute_file_cap(0, 0.15, Some(16_000));
    assert_eq!(cap, 0);
}

#[test]
fn file_cap_with_usize_max_budget_returns_max() {
    let cap = compute_file_cap(usize::MAX, 0.15, Some(16_000));
    assert_eq!(cap, usize::MAX);
}

#[test]
fn file_cap_without_hard_cap_uses_default() {
    // budget=200_000, pct=0.15 → 30_000, default=16_000 → 16_000
    let cap = compute_file_cap(200_000, 0.15, None);
    assert_eq!(cap, DEFAULT_MAX_FILE_TOKENS);
}

#[test]
fn file_cap_tiny_budget_pct_dominates() {
    // budget=100, pct=0.15 → 15, hard_cap=16_000 → 15
    let cap = compute_file_cap(100, 0.15, Some(16_000));
    assert_eq!(cap, 15);
}

// ===========================================================================
// 5. Pack plan construction: priority ordering simulation
// ===========================================================================

/// Simulate a greedy pack plan: sort files by tokens descending, fill budget.
///
/// Uses `explicit_cap` when provided; otherwise derives the cap from budget.
fn simulate_greedy_pack_with_cap(files: &[SimFile], budget: usize, file_cap: usize) -> Vec<&str> {
    // Classify and filter
    let mut candidates: Vec<(&SimFile, InclusionPolicy)> = files
        .iter()
        .map(|f| {
            let classes = classify_file(f.path, f.tokens, f.lines, DEFAULT_DENSE_THRESHOLD);
            let (policy, _) = assign_policy(f.tokens, file_cap, &classes);
            (f, policy)
        })
        .filter(|(_, policy)| *policy != InclusionPolicy::Skip)
        .collect();

    // Sort by tokens descending (simulating Code metric)
    candidates.sort_by(|a, b| {
        b.0.tokens
            .cmp(&a.0.tokens)
            .then_with(|| a.0.path.cmp(b.0.path))
    });

    let mut selected = Vec::new();
    let mut used = 0;

    for (file, policy) in &candidates {
        let effective = if *policy == InclusionPolicy::HeadTail {
            file.tokens.min(file_cap)
        } else {
            file.tokens
        };
        if effective > 0 && used + effective <= budget {
            used += effective;
            selected.push(file.path);
        }
    }

    selected
}

fn simulate_greedy_pack(files: &[SimFile], budget: usize) -> Vec<&str> {
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    simulate_greedy_pack_with_cap(files, budget, cap)
}

#[test]
fn greedy_pack_selects_largest_files_first() {
    let files = vec![
        SimFile {
            path: "src/small.rs",
            tokens: 100,
            lines: 20,
        },
        SimFile {
            path: "src/big.rs",
            tokens: 5_000,
            lines: 500,
        },
        SimFile {
            path: "src/medium.rs",
            tokens: 1_000,
            lines: 100,
        },
    ];
    let selected = simulate_greedy_pack(&files, 128_000);
    assert_eq!(
        selected,
        vec!["src/big.rs", "src/medium.rs", "src/small.rs"]
    );
}

#[test]
fn greedy_pack_respects_budget_limit() {
    // Use a large cap so files get Full policy (no HeadTail capping)
    let files = vec![
        SimFile {
            path: "src/a.rs",
            tokens: 500,
            lines: 100,
        },
        SimFile {
            path: "src/b.rs",
            tokens: 400,
            lines: 80,
        },
        SimFile {
            path: "src/c.rs",
            tokens: 300,
            lines: 60,
        },
    ];
    // Use explicit large cap so all files are Full, budget only fits first two
    let selected = simulate_greedy_pack_with_cap(&files, 900, 100_000);
    assert_eq!(selected, vec!["src/a.rs", "src/b.rs"]);
}

#[test]
fn greedy_pack_excludes_generated_oversized_files() {
    let files = vec![
        SimFile {
            path: "src/lib.rs",
            tokens: 1_000,
            lines: 200,
        },
        SimFile {
            path: "api/types.pb.go",
            tokens: 40_000,
            lines: 8_000,
        },
        SimFile {
            path: "src/main.rs",
            tokens: 500,
            lines: 100,
        },
    ];
    let selected = simulate_greedy_pack(&files, 128_000);
    // Generated file exceeds cap → Skip → not selected
    assert!(selected.contains(&"src/lib.rs"));
    assert!(selected.contains(&"src/main.rs"));
    assert!(!selected.contains(&"api/types.pb.go"));
}

#[test]
fn greedy_pack_includes_head_tailed_files_at_capped_cost() {
    let files = vec![
        SimFile {
            path: "src/big.rs",
            tokens: 20_000,
            lines: 2_000,
        },
        SimFile {
            path: "src/small.rs",
            tokens: 100,
            lines: 20,
        },
    ];
    // budget=18_000, cap = 18_000*0.15 = 2_700 (no hard cap override → min with 16_000)
    // Actually: cap = min(18_000*0.15, 16_000) = min(2_700, 16_000) = 2_700
    // big.rs: 20_000 > 2_700 → HeadTail, effective = 2_700
    // small.rs: 100 ≤ 2_700 → Full
    // Budget: 2_700 + 100 = 2_800 ≤ 18_000 ✓
    let selected = simulate_greedy_pack(&files, 18_000);
    assert!(selected.contains(&"src/big.rs"));
    assert!(selected.contains(&"src/small.rs"));
}

// ===========================================================================
// 6. Spine file detection
// ===========================================================================

#[test]
fn spine_file_detection_covers_all_patterns() {
    let spine_files = [
        "README.md",
        "README",
        "README.rst",
        "README.txt",
        "ROADMAP.md",
        "CONTRIBUTING.md",
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "docs/architecture.md",
        "docs/design.md",
        "tokmd.toml",
        "cockpit.toml",
    ];
    for sf in spine_files {
        assert!(is_spine_file(sf), "expected spine file: {sf}");
    }
}

#[test]
fn spine_file_detection_with_deep_nesting() {
    assert!(is_spine_file("a/b/c/README.md"));
    assert!(is_spine_file("project/sub/Cargo.toml"));
    assert!(is_spine_file("repo/docs/architecture.md"));
}

#[test]
fn spine_file_rejects_non_spine_paths() {
    assert!(!is_spine_file("src/main.rs"));
    assert!(!is_spine_file("tests/test_main.py"));
    assert!(!is_spine_file("README.md.bak"));
    assert!(!is_spine_file("docs/random.md"));
}

#[test]
fn spine_file_handles_backslash_normalization() {
    assert!(is_spine_file("docs\\architecture.md"));
    assert!(is_spine_file("project\\docs\\design.md"));
}

// ===========================================================================
// 7. Determinism: same input → identical output
// ===========================================================================

#[test]
fn classify_file_is_deterministic_across_calls() {
    let path = "vendor/lib/react.min.js";
    let tokens = 50_000;
    let lines = 2;
    let a = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
    let b = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
    assert_eq!(a, b);
}

#[test]
fn assign_policy_is_deterministic_across_calls() {
    let classes = vec![FileClassification::Generated, FileClassification::DataBlob];
    let a = assign_policy(20_000, 16_000, &classes);
    let b = assign_policy(20_000, 16_000, &classes);
    assert_eq!(a.0, b.0);
    assert_eq!(a.1, b.1);
}

#[test]
fn greedy_pack_selection_is_deterministic() {
    let files = vec![
        SimFile {
            path: "src/a.rs",
            tokens: 500,
            lines: 100,
        },
        SimFile {
            path: "src/b.rs",
            tokens: 500,
            lines: 100,
        },
        SimFile {
            path: "src/c.rs",
            tokens: 300,
            lines: 60,
        },
        SimFile {
            path: "src/d.rs",
            tokens: 300,
            lines: 60,
        },
    ];
    let a = simulate_greedy_pack(&files, 1_200);
    let b = simulate_greedy_pack(&files, 1_200);
    assert_eq!(a, b, "greedy pack should be deterministic");
}

#[test]
fn classification_order_is_stable_and_sorted() {
    // Multiple classifications must be sorted and deduped
    let classes = classify_file(
        "vendor/lib/react.min.js",
        100_000,
        1,
        DEFAULT_DENSE_THRESHOLD,
    );
    let mut sorted = classes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        classes, sorted,
        "classifications should be sorted and unique"
    );
}

// ===========================================================================
// 8. Edge cases
// ===========================================================================

#[test]
fn zero_budget_selects_nothing() {
    let files = vec![SimFile {
        path: "src/a.rs",
        tokens: 1,
        lines: 1,
    }];
    let selected = simulate_greedy_pack(&files, 0);
    // With zero budget, cap=0, HeadTail effective=0, which is skipped (effective > 0 check)
    assert!(selected.is_empty(), "zero budget should select no files");
}

#[test]
fn single_file_fits_within_budget() {
    let files = vec![SimFile {
        path: "src/main.rs",
        tokens: 500,
        lines: 100,
    }];
    let selected = simulate_greedy_pack(&files, 128_000);
    assert_eq!(selected, vec!["src/main.rs"]);
}

#[test]
fn single_file_exceeds_budget() {
    // Use explicit large cap so files get Full policy, then budget is too small
    let files = vec![SimFile {
        path: "src/main.rs",
        tokens: 500,
        lines: 100,
    }];
    let selected = simulate_greedy_pack_with_cap(&files, 100, 100_000);
    assert!(
        selected.is_empty(),
        "file exceeding budget should not be selected"
    );
}

#[test]
fn all_files_excluded_by_smart_exclude() {
    let lockfiles = vec![
        SimFile {
            path: "Cargo.lock",
            tokens: 5_000,
            lines: 500,
        },
        SimFile {
            path: "package-lock.json",
            tokens: 10_000,
            lines: 1_000,
        },
    ];
    // Verify all are smart-excluded
    for f in &lockfiles {
        assert!(smart_exclude_reason(f.path).is_some());
    }
}

#[test]
fn all_files_excluded_by_classification_policy() {
    let files = vec![
        SimFile {
            path: "api/types.pb.go",
            tokens: 40_000,
            lines: 8_000,
        },
        SimFile {
            path: "vendor/lib/big.js",
            tokens: 50_000,
            lines: 10_000,
        },
    ];
    let selected = simulate_greedy_pack(&files, 128_000);
    // Both are generated/vendored and exceed cap → Skip
    assert!(
        selected.is_empty(),
        "all skip-classified files should be excluded"
    );
}

#[test]
fn empty_file_list_produces_empty_selection() {
    let files: Vec<SimFile> = Vec::new();
    let selected = simulate_greedy_pack(&files, 128_000);
    assert!(selected.is_empty());
}

#[test]
fn classify_file_with_zero_tokens_and_zero_lines() {
    let classes = classify_file("empty.rs", 0, 0, DEFAULT_DENSE_THRESHOLD);
    // 0/max(0,1) = 0.0 which is not > threshold
    assert!(
        !classes.contains(&FileClassification::DataBlob),
        "zero tokens should not be DataBlob"
    );
}

#[test]
fn assign_policy_at_exact_cap_boundary() {
    // tokens == cap → Full
    let (policy, reason) = assign_policy(16_000, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::Full);
    assert!(reason.is_none());

    // tokens == cap + 1 → HeadTail (no skip classes)
    let (policy, reason) = assign_policy(16_001, 16_000, &[]);
    assert_eq!(policy, InclusionPolicy::HeadTail);
    assert!(reason.is_some());
}

// ===========================================================================
// 9. Pack plan with BTreeMap ordering (deterministic file ordering)
// ===========================================================================

#[test]
fn btreemap_based_module_grouping_is_deterministic() {
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    groups.entry("src").or_default().push("src/main.rs");
    groups.entry("lib").or_default().push("lib/utils.rs");
    groups.entry("tests").or_default().push("tests/test.rs");
    groups.entry("src").or_default().push("src/lib.rs");

    let keys: Vec<&&str> = groups.keys().collect();
    assert_eq!(keys, vec![&"lib", &"src", &"tests"]);

    // Same operation a second time yields same order
    let mut groups2: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    groups2.entry("tests").or_default().push("tests/test.rs");
    groups2.entry("src").or_default().push("src/main.rs");
    groups2.entry("lib").or_default().push("lib/utils.rs");
    groups2.entry("src").or_default().push("src/lib.rs");

    let keys2: Vec<&&str> = groups2.keys().collect();
    assert_eq!(
        keys, keys2,
        "BTreeMap iteration order should be deterministic"
    );
}

// ===========================================================================
// 10. Policy reason messages contain expected info
// ===========================================================================

#[test]
fn skip_reason_includes_classification_names() {
    let (_, reason) = assign_policy(
        20_000,
        16_000,
        &[FileClassification::Generated, FileClassification::Vendored],
    );
    let reason = reason.expect("skip should have reason");
    assert!(reason.contains("generated"));
    assert!(reason.contains("vendored"));
    assert!(reason.contains("20000"));
    assert!(reason.contains("16000"));
}

#[test]
fn head_tail_reason_includes_token_counts() {
    let (_, reason) = assign_policy(20_000, 16_000, &[]);
    let reason = reason.expect("head_tail should have reason");
    assert!(reason.contains("head+tail"));
    assert!(reason.contains("20000"));
    assert!(reason.contains("16000"));
}
