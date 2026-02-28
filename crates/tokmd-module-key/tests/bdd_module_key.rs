use tokmd_module_key::module_key;

#[test]
fn given_workspace_root_path_when_module_key_computed_then_depth_limits_segments() {
    // Given: a path under a configured monorepo root
    let roots = vec!["crates".to_string(), "packages".to_string()];
    let path = "crates/tokmd-model/src/lib.rs";

    // When: module key is computed with depth 2
    let key = module_key(path, &roots, 2);

    // Then: key includes root + one child directory
    assert_eq!(key, "crates/tokmd-model");
}

#[test]
fn given_non_root_path_when_module_key_computed_then_first_directory_is_used() {
    // Given: a path whose first segment is not a configured root
    let roots = vec!["crates".to_string()];
    let path = "src/commands/analyze.rs";

    // When: module key is computed
    let key = module_key(path, &roots, 5);

    // Then: key is the top-level directory
    assert_eq!(key, "src");
}

#[test]
fn given_windows_style_path_when_module_key_computed_then_result_is_forward_slash_deterministic() {
    // Given: equivalent path forms
    let roots = vec!["crates".to_string()];
    let win = r".\crates\tokmd\src\main.rs";
    let unix = "./crates/tokmd/src/main.rs";

    // When: keys are computed
    let win_key = module_key(win, &roots, 2);
    let unix_key = module_key(unix, &roots, 2);

    // Then: normalized output is stable across separators
    assert_eq!(win_key, unix_key);
    assert_eq!(win_key, "crates/tokmd");
}

// ── Depth / root edge-cases ──────────────────────────────────────

#[test]
fn given_depth_zero_when_module_key_computed_then_clamped_to_one() {
    // depth 0 should behave the same as depth 1 (max(1))
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/foo/src/lib.rs", &roots, 0);
    assert_eq!(key, "crates");
}

#[test]
fn given_empty_roots_when_module_key_computed_then_first_dir_is_key() {
    let key = module_key("crates/foo/src/lib.rs", &[], 5);
    assert_eq!(key, "crates");
}

#[test]
fn given_depth_one_when_matching_root_then_only_root_segment_returned() {
    let roots = vec!["packages".to_string()];
    let key = module_key("packages/web/src/index.ts", &roots, 1);
    assert_eq!(key, "packages");
}

#[test]
fn given_depth_three_when_matching_root_then_three_segments() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/tokmd-model/src/lib.rs", &roots, 3);
    assert_eq!(key, "crates/tokmd-model/src");
}

#[test]
fn given_multiple_roots_when_second_root_matches_then_depth_applied() {
    let roots = vec!["crates".to_string(), "libs".to_string()];
    let key = module_key("libs/core/util/helper.rs", &roots, 2);
    assert_eq!(key, "libs/core");
}

// ── Very deep paths ─────────────────────────────────────────────

#[test]
fn given_very_deep_path_when_depth_two_then_only_two_segments() {
    let roots = vec!["src".to_string()];
    let path = "src/a/b/c/d/e/f/g/h/i/j/file.rs";
    let key = module_key(path, &roots, 2);
    assert_eq!(key, "src/a");
}

#[test]
fn given_very_deep_path_when_depth_exceeds_dirs_then_all_dirs_used() {
    let roots = vec!["src".to_string()];
    let path = "src/a/b/file.rs";
    let key = module_key(path, &roots, 20);
    assert_eq!(key, "src/a/b");
}

// ── Special characters ──────────────────────────────────────────

#[test]
fn given_hyphenated_dir_names_when_module_key_computed_then_preserved() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/my-crate/src/lib.rs", &roots, 2);
    assert_eq!(key, "crates/my-crate");
}

#[test]
fn given_dir_names_with_dots_when_not_dot_segment_then_preserved() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/v1.2.3/src/lib.rs", &roots, 2);
    assert_eq!(key, "crates/v1.2.3");
}

#[test]
fn given_dir_names_with_at_sign_when_module_key_computed_then_preserved() {
    let roots = vec!["packages".to_string()];
    let key = module_key("packages/@scope/pkg/index.js", &roots, 3);
    assert_eq!(key, "packages/@scope/pkg");
}

#[test]
fn given_underscored_dir_names_then_preserved() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/__internal__/src/lib.rs", &roots, 2);
    assert_eq!(key, "crates/__internal__");
}

// ── Dot segments and normalization ──────────────────────────────

#[test]
fn given_multiple_dot_segments_when_module_key_then_dots_skipped() {
    let roots = vec!["crates".to_string()];
    let key = module_key("./crates/./foo/./src/lib.rs", &roots, 2);
    assert_eq!(key, "crates/foo");
}

#[test]
fn given_leading_slash_when_module_key_then_stripped() {
    let roots = vec!["crates".to_string()];
    let key = module_key("/crates/foo/src/lib.rs", &roots, 2);
    assert_eq!(key, "crates/foo");
}

#[test]
fn given_double_leading_slash_when_module_key_then_stripped() {
    let roots = vec!["crates".to_string()];
    let key = module_key("//crates/foo/src/lib.rs", &roots, 2);
    assert_eq!(key, "crates/foo");
}

// ── Windows paths ───────────────────────────────────────────────

#[test]
fn given_pure_backslash_path_when_module_key_then_normalized() {
    let roots = vec!["crates".to_string()];
    let key = module_key(r"crates\foo\src\lib.rs", &roots, 2);
    assert_eq!(key, "crates/foo");
}

#[test]
fn given_mixed_separators_when_module_key_then_normalized() {
    let roots = vec!["crates".to_string()];
    let key = module_key(r"crates/foo\src\lib.rs", &roots, 2);
    assert_eq!(key, "crates/foo");
}

#[test]
fn given_windows_dot_backslash_prefix_when_module_key_then_stripped() {
    let roots = vec!["src".to_string()];
    let key = module_key(r".\src\main.rs", &roots, 2);
    assert_eq!(key, "src");
}

// ── Bare-directory files (file directly under root segment) ─────

#[test]
fn given_file_directly_under_root_segment_when_module_key_then_root_only() {
    let roots = vec!["crates".to_string()];
    let key = module_key("crates/Cargo.toml", &roots, 2);
    assert_eq!(key, "crates");
}

#[test]
fn given_file_directly_under_non_root_dir_then_first_dir() {
    let key = module_key("docs/README.md", &[], 2);
    assert_eq!(key, "docs");
}

// ── Root-level edge-cases ───────────────────────────────────────

#[test]
fn given_dotfile_at_root_then_root() {
    assert_eq!(module_key(".gitignore", &[], 2), "(root)");
}

#[test]
fn given_file_with_no_extension_at_root_then_root() {
    assert_eq!(module_key("Makefile", &[], 2), "(root)");
}

#[test]
fn given_dot_slash_dotfile_then_root() {
    assert_eq!(module_key("./.tokeignore", &[], 2), "(root)");
}
