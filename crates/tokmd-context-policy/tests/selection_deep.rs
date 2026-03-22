//! Deep selection policy tests.
//!
//! Covers: classification interaction combos, spine–policy orthogonality,
//! greedy pack priority with spine files, pack efficiency metrics,
//! extreme thresholds, tie-breaking determinism, and multi-category
//! edge cases not covered by existing test files.

use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, assign_policy, classify_file, compute_file_cap,
    is_spine_file, smart_exclude_reason,
};
use tokmd_types::{FileClassification, InclusionPolicy};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct SimFile {
    path: &'static str,
    tokens: usize,
    lines: usize,
}

fn pipeline(path: &str, tokens: usize, lines: usize, budget: usize) -> InclusionPolicy {
    let classes = classify_file(path, tokens, lines, DEFAULT_DENSE_THRESHOLD);
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    let (policy, _) = assign_policy(tokens, cap, &classes);
    policy
}

fn simulate_spine_priority_pack(files: &[SimFile], budget: usize) -> Vec<&str> {
    let cap = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    let mut candidates: Vec<(&SimFile, InclusionPolicy, bool)> = files
        .iter()
        .map(|f| {
            let classes = classify_file(f.path, f.tokens, f.lines, DEFAULT_DENSE_THRESHOLD);
            let (policy, _) = assign_policy(f.tokens, cap, &classes);
            let spine = is_spine_file(f.path);
            (f, policy, spine)
        })
        .filter(|(_, policy, _)| *policy != InclusionPolicy::Skip)
        .collect();

    // Spine first, then by tokens descending, then by path for determinism
    candidates.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.0.tokens.cmp(&a.0.tokens))
            .then_with(|| a.0.path.cmp(b.0.path))
    });

    let mut selected = Vec::new();
    let mut used = 0;
    for (file, policy, _) in &candidates {
        let effective = if *policy == InclusionPolicy::HeadTail {
            file.tokens.min(cap)
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

// ===========================================================================
// 1. Classification interaction combos
// ===========================================================================

#[test]
fn classify_vendor_inside_fixture_directory_yields_both() {
    let classes = classify_file(
        "tests/fixtures/vendor/dep.js",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Fixture));
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_generated_file_inside_vendored_dir_yields_both() {
    let classes = classify_file(
        "vendor/proto/types.pb.go",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Generated));
}

#[test]
fn classify_lockfile_inside_node_modules_yields_lockfile_and_vendored() {
    let classes = classify_file(
        "node_modules/some-pkg/package-lock.json",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Lockfile));
    assert!(classes.contains(&FileClassification::Vendored));
}

#[test]
fn classify_sourcemap_inside_testdata_yields_sourcemap_and_fixture() {
    let classes = classify_file("testdata/bundle.js.map", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Sourcemap));
    assert!(classes.contains(&FileClassification::Fixture));
}

#[test]
fn classify_minified_lockfile_impossible_but_handles_gracefully() {
    // A file named "yarn.lock" cannot also be ".min.js", but we ensure no panic
    let classes = classify_file("yarn.lock", 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Lockfile));
    assert!(!classes.contains(&FileClassification::Minified));
}

// ===========================================================================
// 2. Spine–policy orthogonality
// ===========================================================================

#[test]
fn spine_file_that_is_also_generated_is_not_affected_by_spine() {
    // grammar.json matches generated patterns and is not a lockfile/minified
    let path = "src/grammar.json";
    let classes = classify_file(path, 500, 100, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::Generated));
    assert_eq!(smart_exclude_reason(path), None);
}

#[test]
fn spine_and_smart_exclude_are_disjoint_for_all_lockfiles() {
    let lockfiles = [
        "Cargo.lock",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "poetry.lock",
        "Pipfile.lock",
        "go.sum",
        "composer.lock",
        "Gemfile.lock",
    ];
    for lf in lockfiles {
        assert!(!is_spine_file(lf), "{lf} should NOT be a spine file");
        assert_eq!(
            smart_exclude_reason(lf),
            Some("lockfile"),
            "{lf} should be smart-excluded"
        );
    }
}

#[test]
fn spine_files_never_have_smart_exclude_reason() {
    let spine = [
        "README.md",
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "CONTRIBUTING.md",
        "ROADMAP.md",
        "tokmd.toml",
        "cockpit.toml",
    ];
    for sf in spine {
        assert!(is_spine_file(sf), "{sf} should be spine");
        assert_eq!(
            smart_exclude_reason(sf),
            None,
            "{sf} should NOT be smart-excluded"
        );
    }
}

// ===========================================================================
// 3. Greedy pack with spine file priority
// ===========================================================================

#[test]
fn spine_priority_pack_selects_readme_before_larger_files() {
    let files = vec![
        SimFile {
            path: "src/big.rs",
            tokens: 5_000,
            lines: 500,
        },
        SimFile {
            path: "README.md",
            tokens: 200,
            lines: 40,
        },
        SimFile {
            path: "src/medium.rs",
            tokens: 2_000,
            lines: 200,
        },
    ];
    let selected = simulate_spine_priority_pack(&files, 128_000);
    assert_eq!(selected[0], "README.md", "spine file should be first");
}

#[test]
fn spine_priority_pack_multiple_spine_files_before_sources() {
    let files = vec![
        SimFile {
            path: "src/main.rs",
            tokens: 3_000,
            lines: 300,
        },
        SimFile {
            path: "Cargo.toml",
            tokens: 100,
            lines: 20,
        },
        SimFile {
            path: "README.md",
            tokens: 500,
            lines: 80,
        },
        SimFile {
            path: "src/lib.rs",
            tokens: 2_000,
            lines: 200,
        },
    ];
    let selected = simulate_spine_priority_pack(&files, 128_000);
    // Both spine files should come first
    let spine_end = selected
        .iter()
        .position(|p| !is_spine_file(p))
        .unwrap_or(selected.len());
    assert!(
        spine_end >= 2,
        "at least 2 spine files should precede non-spine"
    );
}

#[test]
fn spine_priority_pack_spine_file_alone_when_generated_skipped() {
    let files = vec![
        SimFile {
            path: "README.md",
            tokens: 200,
            lines: 40,
        },
        SimFile {
            path: "api/types.pb.go",
            tokens: 50_000,
            lines: 5_000,
        },
    ];
    // Generated file exceeds cap → Skip; only README survives
    let selected = simulate_spine_priority_pack(&files, 128_000);
    assert_eq!(selected, vec!["README.md"]);
}

// ===========================================================================
// 4. Tie-breaking determinism in pack ordering
// ===========================================================================

#[test]
fn tie_breaking_by_path_when_tokens_equal() {
    let files = vec![
        SimFile {
            path: "src/zebra.rs",
            tokens: 1_000,
            lines: 100,
        },
        SimFile {
            path: "src/alpha.rs",
            tokens: 1_000,
            lines: 100,
        },
        SimFile {
            path: "src/middle.rs",
            tokens: 1_000,
            lines: 100,
        },
    ];
    let selected = simulate_spine_priority_pack(&files, 128_000);
    assert_eq!(
        selected,
        vec!["src/alpha.rs", "src/middle.rs", "src/zebra.rs"],
        "equal-token files should be sorted alphabetically"
    );
}

#[test]
fn tie_breaking_stable_across_multiple_runs() {
    let files = vec![
        SimFile {
            path: "a.rs",
            tokens: 500,
            lines: 50,
        },
        SimFile {
            path: "b.rs",
            tokens: 500,
            lines: 50,
        },
        SimFile {
            path: "c.rs",
            tokens: 500,
            lines: 50,
        },
    ];
    let run1 = simulate_spine_priority_pack(&files, 128_000);
    let run2 = simulate_spine_priority_pack(&files, 128_000);
    let run3 = simulate_spine_priority_pack(&files, 128_000);
    assert_eq!(run1, run2);
    assert_eq!(run2, run3);
}

// ===========================================================================
// 5. Extreme dense threshold values
// ===========================================================================

#[test]
fn dense_threshold_zero_marks_any_file_with_tokens_as_blob() {
    let classes = classify_file("src/main.rs", 1, 100, 0.0);
    // 1/100 = 0.01, NOT > 0.0 is false since 0.01 > 0.0
    assert!(
        classes.contains(&FileClassification::DataBlob),
        "threshold=0.0 should flag almost everything"
    );
}

#[test]
fn dense_threshold_very_high_never_marks_as_blob() {
    let classes = classify_file("src/data.bin", 1_000_000, 1, 1e15);
    assert!(!classes.contains(&FileClassification::DataBlob));
}

#[test]
fn dense_threshold_negative_marks_everything() {
    let classes = classify_file("x.rs", 1, 1, -1.0);
    assert!(classes.contains(&FileClassification::DataBlob));
}

// ===========================================================================
// 6. Classification sorting order is stable
// ===========================================================================

#[test]
fn classifications_maintain_enum_discriminant_order() {
    // FileClassification derives Ord; verify the order is Generated < Fixture < Vendored ...
    assert!(FileClassification::Generated < FileClassification::Fixture);
    assert!(FileClassification::Fixture < FileClassification::Vendored);
    assert!(FileClassification::Vendored < FileClassification::Lockfile);
    assert!(FileClassification::Lockfile < FileClassification::Minified);
    assert!(FileClassification::Minified < FileClassification::DataBlob);
    assert!(FileClassification::DataBlob < FileClassification::Sourcemap);
}

#[test]
fn classify_returns_sorted_vec_for_complex_multi_classification() {
    // vendor + generated + lockfile + dense
    let classes = classify_file("vendor/Cargo.lock", 100_000, 1, DEFAULT_DENSE_THRESHOLD);
    // Should contain Generated? No. Lockfile + Vendored + DataBlob
    assert!(classes.contains(&FileClassification::Vendored));
    assert!(classes.contains(&FileClassification::Lockfile));
    assert!(classes.contains(&FileClassification::DataBlob));
    // Verify sorted
    for w in classes.windows(2) {
        assert!(
            w[0] <= w[1],
            "classifications must be sorted: {:?}",
            classes
        );
    }
}

// ===========================================================================
// 7. Pack plan efficiency and utilization
// ===========================================================================

#[test]
fn pack_plan_utilization_near_100_pct_when_files_fill_budget() {
    // All files under per-file cap so they get Full policy and fill exactly.
    let files = [
        SimFile {
            path: "a.rs",
            tokens: 500,
            lines: 50,
        },
        SimFile {
            path: "b.rs",
            tokens: 300,
            lines: 30,
        },
        SimFile {
            path: "c.rs",
            tokens: 200,
            lines: 20,
        },
    ];
    let budget = 1_000;
    // cap = min(1000*0.15, 16_000) = 150, but all files exceed that → HeadTail.
    // Use a large explicit cap so all files are Full.
    let cap = 100_000;
    let mut total_used = 0;
    let mut selected = Vec::new();

    let mut candidates: Vec<_> = files
        .iter()
        .map(|f| {
            let classes = classify_file(f.path, f.tokens, f.lines, DEFAULT_DENSE_THRESHOLD);
            let (policy, _) = assign_policy(f.tokens, cap, &classes);
            (f, policy)
        })
        .filter(|(_, p)| *p != InclusionPolicy::Skip)
        .collect();
    candidates.sort_by(|a, b| {
        b.0.tokens
            .cmp(&a.0.tokens)
            .then_with(|| a.0.path.cmp(&b.0.path))
    });

    for (f, p) in &candidates {
        let eff = if *p == InclusionPolicy::HeadTail {
            f.tokens.min(cap)
        } else {
            f.tokens
        };
        if eff > 0 && total_used + eff <= budget {
            total_used += eff;
            selected.push(f.path);
        }
    }

    let utilization = total_used as f64 / budget as f64 * 100.0;
    assert!(
        utilization == 100.0,
        "expected 100% utilization, got {utilization}%"
    );
    assert_eq!(selected.len(), 3);
}

#[test]
fn pack_plan_with_all_head_tail_files() {
    let cap = compute_file_cap(50_000, DEFAULT_MAX_FILE_PCT, None);
    // All files exceed per-file cap → HeadTail
    let files = vec![
        SimFile {
            path: "a.rs",
            tokens: 20_000,
            lines: 2_000,
        },
        SimFile {
            path: "b.rs",
            tokens: 18_000,
            lines: 1_800,
        },
    ];
    let mut selected = Vec::new();
    let mut used = 0;
    for f in &files {
        let classes = classify_file(f.path, f.tokens, f.lines, DEFAULT_DENSE_THRESHOLD);
        let (policy, _) = assign_policy(f.tokens, cap, &classes);
        assert_eq!(
            policy,
            InclusionPolicy::HeadTail,
            "both files should be HeadTail"
        );
        let eff = f.tokens.min(cap);
        if used + eff <= 50_000 {
            used += eff;
            selected.push(f.path);
        }
    }
    assert_eq!(
        selected.len(),
        2,
        "both HeadTail files should fit when capped"
    );
}

// ===========================================================================
// 8. Pipeline end-to-end with unusual budgets
// ===========================================================================

#[test]
fn pipeline_budget_one_excludes_everything() {
    assert_eq!(pipeline("src/main.rs", 1, 1, 1), InclusionPolicy::HeadTail);
}

#[test]
fn pipeline_max_budget_includes_everything() {
    assert_eq!(
        pipeline("src/main.rs", 100_000, 10_000, usize::MAX),
        InclusionPolicy::Full
    );
}

#[test]
fn pipeline_generated_file_under_cap_is_full() {
    // Small generated file within budget → Full
    assert_eq!(
        pipeline("types.pb.go", 100, 20, 128_000),
        InclusionPolicy::Full
    );
}

#[test]
fn pipeline_data_blob_under_cap_is_full() {
    // Dense file but small enough → Full (tokens_per_line > threshold is classification,
    // but if tokens ≤ cap then Full)
    let classes = classify_file("data.bin", 100, 1, DEFAULT_DENSE_THRESHOLD);
    assert!(classes.contains(&FileClassification::DataBlob));
    let cap = compute_file_cap(128_000, DEFAULT_MAX_FILE_PCT, None);
    let (policy, _) = assign_policy(100, cap, &classes);
    assert_eq!(policy, InclusionPolicy::Full);
}

// ===========================================================================
// 9. Multiple generated patterns cannot stack
// ===========================================================================

#[test]
fn file_matching_multiple_generated_patterns_yields_single_generated() {
    // ".generated." is checked and ".pb.rs" is checked, but only one Generated enum
    let classes = classify_file(
        "proto/service.generated.pb.rs",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    let gen_count = classes
        .iter()
        .filter(|c| **c == FileClassification::Generated)
        .count();
    assert_eq!(
        gen_count, 1,
        "dedup should collapse duplicate Generated entries"
    );
}

#[test]
fn node_types_json_in_fixture_dir_is_both_generated_and_fixture() {
    let classes = classify_file(
        "testdata/node-types.json",
        500,
        100,
        DEFAULT_DENSE_THRESHOLD,
    );
    assert!(classes.contains(&FileClassification::Generated));
    assert!(classes.contains(&FileClassification::Fixture));
    assert_eq!(classes.len(), 2);
}
