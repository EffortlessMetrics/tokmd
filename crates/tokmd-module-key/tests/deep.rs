//! Deep tests for tokmd-module-key: path edge cases, depth semantics,
//! multi-root interactions, and normalization guarantees not covered
//! by existing unit/bdd/property tests.

use tokmd_module_key::{module_key, module_key_from_normalized};

// ── Empty / degenerate inputs ───────────────────────────────────────

#[test]
fn empty_string_yields_root() {
    assert_eq!(module_key("", &[], 2), "(root)");
}

#[test]
fn single_dot_yields_root() {
    assert_eq!(module_key(".", &[], 2), "(root)");
}

#[test]
fn dot_slash_only_yields_root() {
    assert_eq!(module_key("./", &[], 2), "(root)");
}

#[test]
fn just_a_filename_yields_root() {
    assert_eq!(module_key("main.rs", &[], 2), "(root)");
    assert_eq!(module_key("main.rs", &["src".into()], 5), "(root)");
}

#[test]
fn slash_only_yields_root() {
    assert_eq!(module_key("/", &[], 2), "(root)");
}

// ── Depth semantics ─────────────────────────────────────────────────

#[test]
fn depth_zero_clamps_to_one_for_matching_root() {
    let roots = vec!["crates".into()];
    let k0 = module_key("crates/foo/bar/baz.rs", &roots, 0);
    let k1 = module_key("crates/foo/bar/baz.rs", &roots, 1);
    assert_eq!(k0, k1);
    assert_eq!(k0, "crates");
}

#[test]
fn depth_one_returns_root_segment_only() {
    let roots = vec!["packages".into()];
    assert_eq!(
        module_key("packages/web/src/index.ts", &roots, 1),
        "packages"
    );
}

#[test]
fn depth_matches_available_segments_exactly() {
    let roots = vec!["src".into()];
    // Path has exactly 3 dir segments: src/a/b
    assert_eq!(module_key("src/a/b/file.rs", &roots, 3), "src/a/b");
}

#[test]
fn depth_exceeds_available_segments() {
    let roots = vec!["src".into()];
    // Only 2 dir segments available
    assert_eq!(module_key("src/a/file.rs", &roots, 10), "src/a");
}

#[test]
fn increasing_depth_yields_longer_keys() {
    let roots = vec!["crates".into()];
    let path = "crates/tokmd/src/commands/analyze.rs";
    let k1 = module_key(path, &roots, 1);
    let k2 = module_key(path, &roots, 2);
    let k3 = module_key(path, &roots, 3);
    let k4 = module_key(path, &roots, 4);

    assert_eq!(k1, "crates");
    assert_eq!(k2, "crates/tokmd");
    assert_eq!(k3, "crates/tokmd/src");
    assert_eq!(k4, "crates/tokmd/src/commands");
    assert!(k1.len() < k2.len());
    assert!(k2.len() < k3.len());
    assert!(k3.len() < k4.len());
}

// ── Non-matching root: always first dir ─────────────────────────────

#[test]
fn non_matching_root_ignores_depth() {
    let roots = vec!["crates".into()];
    // "src" is not a root, so depth is irrelevant
    assert_eq!(module_key("src/a/b/c/d.rs", &roots, 1), "src");
    assert_eq!(module_key("src/a/b/c/d.rs", &roots, 5), "src");
    assert_eq!(module_key("src/a/b/c/d.rs", &roots, 100), "src");
}

#[test]
fn empty_roots_always_returns_first_dir() {
    assert_eq!(module_key("alpha/beta/gamma.rs", &[], 1), "alpha");
    assert_eq!(module_key("alpha/beta/gamma.rs", &[], 5), "alpha");
}

// ── Multiple roots ──────────────────────────────────────────────────

#[test]
fn second_root_matches_when_first_does_not() {
    let roots = vec!["crates".into(), "packages".into()];
    assert_eq!(
        module_key("packages/web/src/index.ts", &roots, 2),
        "packages/web"
    );
}

#[test]
fn three_roots_all_match_independently() {
    let roots = vec!["crates".into(), "packages".into(), "libs".into()];
    assert_eq!(module_key("crates/a/x.rs", &roots, 2), "crates/a");
    assert_eq!(module_key("packages/b/y.ts", &roots, 2), "packages/b");
    assert_eq!(module_key("libs/c/z.go", &roots, 2), "libs/c");
}

#[test]
fn root_that_is_substring_of_another_does_not_false_match() {
    let roots = vec!["lib".into(), "libs".into()];
    // "libs" should match "libs", not "lib"
    assert_eq!(module_key("libs/core/mod.rs", &roots, 2), "libs/core");
    assert_eq!(module_key("lib/utils/mod.rs", &roots, 2), "lib/utils");
}

// ── Path normalization ──────────────────────────────────────────────

#[test]
fn backslash_paths_normalized_to_forward_slash() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key(r"crates\foo\src\lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn mixed_separators_normalized() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key(r"crates/foo\bar/baz.rs", &roots, 3),
        "crates/foo/bar"
    );
}

#[test]
fn leading_dot_slash_stripped() {
    let roots = vec!["src".into()];
    assert_eq!(module_key("./src/main.rs", &roots, 2), "src");
}

#[test]
fn leading_dot_backslash_stripped() {
    let roots = vec!["src".into()];
    assert_eq!(module_key(r".\src\main.rs", &roots, 2), "src");
}

#[test]
fn leading_slash_stripped() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("/crates/foo/lib.rs", &roots, 2), "crates/foo");
}

#[test]
fn multiple_leading_slashes_stripped() {
    let roots = vec!["crates".into()];
    assert_eq!(module_key("///crates/foo/lib.rs", &roots, 2), "crates/foo");
}

// ── module_key_from_normalized edge cases ───────────────────────────

#[test]
fn from_normalized_empty_string_yields_root() {
    assert_eq!(module_key_from_normalized("", &[], 2), "(root)");
}

#[test]
fn from_normalized_bare_filename_yields_root() {
    assert_eq!(module_key_from_normalized("lib.rs", &[], 2), "(root)");
}

#[test]
fn from_normalized_double_slash_skips_empty_segments() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key_from_normalized("crates//foo//src/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn from_normalized_dot_segments_are_skipped() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key_from_normalized("crates/./foo/./bar/lib.rs", &roots, 2),
        "crates/foo"
    );
}

#[test]
fn from_normalized_agrees_with_module_key_for_clean_paths() {
    let roots = vec!["crates".into(), "src".into()];
    let paths = [
        "crates/foo/src/lib.rs",
        "src/commands/run.rs",
        "README.md",
        "tools/gen.sh",
    ];
    for path in paths {
        assert_eq!(
            module_key(path, &roots, 2),
            module_key_from_normalized(path, &roots, 2),
            "Disagreement on path: {path}"
        );
    }
}

// ── Special characters in path segments ─────────────────────────────

#[test]
fn unicode_dir_names_preserved() {
    let roots = vec!["crates".into()];
    assert_eq!(
        module_key("crates/données/src/lib.rs", &roots, 2),
        "crates/données"
    );
}

#[test]
fn spaces_in_dir_names_preserved() {
    let roots = vec!["my projects".into()];
    assert_eq!(
        module_key("my projects/app/main.rs", &roots, 2),
        "my projects/app"
    );
}

#[test]
fn numeric_dir_names() {
    let roots = vec!["v2".into()];
    assert_eq!(module_key("v2/api/handler.go", &roots, 2), "v2/api");
}

#[test]
fn dotted_dir_names_not_treated_as_dot_segment() {
    let roots = vec!["crates".into()];
    // ".hidden" is a real directory, not a dot segment
    assert_eq!(
        module_key("crates/.hidden/lib.rs", &roots, 2),
        "crates/.hidden"
    );
}

// ── Determinism ─────────────────────────────────────────────────────

#[test]
fn deterministic_across_repeated_calls() {
    let roots = vec!["crates".into()];
    let path = "crates/tokmd-math/src/lib.rs";
    let results: Vec<String> = (0..100).map(|_| module_key(path, &roots, 2)).collect();
    assert!(results.iter().all(|r| r == &results[0]));
}

#[test]
fn equivalent_path_forms_produce_same_key() {
    let roots = vec!["crates".into()];
    let variants = [
        "crates/foo/src/lib.rs",
        "./crates/foo/src/lib.rs",
        r"crates\foo\src\lib.rs",
        r".\crates\foo\src\lib.rs",
        "/crates/foo/src/lib.rs",
    ];
    let keys: Vec<String> = variants.iter().map(|p| module_key(p, &roots, 2)).collect();
    assert!(
        keys.iter().all(|k| k == "crates/foo"),
        "All variants should produce 'crates/foo': {keys:?}"
    );
}

// ── Output format guarantees ────────────────────────────────────────

#[test]
fn output_never_contains_backslash() {
    let roots = vec!["crates".into()];
    let key = module_key(r"crates\foo\bar\baz.rs", &roots, 3);
    assert!(!key.contains('\\'));
}

#[test]
fn output_never_ends_with_slash() {
    let roots = vec!["crates".into()];
    for depth in 1..=5 {
        let key = module_key("crates/a/b/c/d/e.rs", &roots, depth);
        assert!(!key.ends_with('/'), "depth {depth}: key={key}");
    }
}

#[test]
fn output_never_starts_with_slash() {
    let roots = vec!["crates".into()];
    let key = module_key("/crates/foo/lib.rs", &roots, 2);
    assert!(!key.starts_with('/'));
}

#[test]
fn output_is_either_root_marker_or_path_segment() {
    let roots = vec!["crates".into()];
    let paths = ["README.md", "src/lib.rs", "crates/a/b.rs"];
    for path in paths {
        let key = module_key(path, &roots, 2);
        assert!(
            key == "(root)" || !key.contains('('),
            "Key should be (root) or a path: {key}"
        );
    }
}
