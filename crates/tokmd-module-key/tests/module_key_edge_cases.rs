use tokmd_module_key::{module_key, module_key_from_normalized};

fn roots(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}

// ── root-level files ───────────────────────────────────────────────

#[test]
fn root_file_plain() {
    assert_eq!(module_key("Cargo.toml", &roots(&["crates"]), 2), "(root)");
}

#[test]
fn root_file_with_dot_slash_prefix() {
    assert_eq!(
        module_key("./Cargo.toml", &roots(&["crates"]), 2),
        "(root)"
    );
}

#[test]
fn root_file_with_leading_slash() {
    assert_eq!(
        module_key("/README.md", &roots(&["crates"]), 2),
        "(root)"
    );
}

#[test]
fn root_file_hidden_dotfile() {
    assert_eq!(
        module_key(".gitignore", &roots(&["crates"]), 2),
        "(root)"
    );
}

// ── path normalization ─────────────────────────────────────────────

#[test]
fn backslashes_normalized_to_forward_slashes() {
    assert_eq!(
        module_key("crates\\foo\\src\\lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn mixed_slashes_normalized() {
    assert_eq!(
        module_key("crates/foo\\src\\lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn leading_dot_slash_stripped() {
    assert_eq!(
        module_key("./src/lib.rs", &roots(&["crates"]), 2),
        "src"
    );
}

#[test]
fn leading_slash_stripped() {
    assert_eq!(
        module_key("/src/lib.rs", &roots(&["crates"]), 2),
        "src"
    );
}

// ── depth parameter behavior ───────────────────────────────────────

#[test]
fn depth_one_returns_only_root_segment() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 1),
        "crates"
    );
}

#[test]
fn depth_two_returns_root_plus_one() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn depth_three_returns_three_segments() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 3),
        "crates/foo/src"
    );
}

#[test]
fn depth_larger_than_available_stops_at_last_dir() {
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 10),
        "crates/foo/src"
    );
}

#[test]
fn depth_zero_treated_as_one() {
    // depth.max(1) in source means 0 becomes 1
    assert_eq!(
        module_key("crates/foo/src/lib.rs", &roots(&["crates"]), 0),
        "crates"
    );
}

#[test]
fn depth_does_not_include_filename() {
    // Only one dir segment "crates" exists between root and file
    assert_eq!(
        module_key("crates/foo.rs", &roots(&["crates"]), 2),
        "crates"
    );
}

// ── module roots behavior ──────────────────────────────────────────

#[test]
fn multiple_module_roots() {
    let r = roots(&["crates", "packages"]);
    assert_eq!(module_key("crates/foo/src/lib.rs", &r, 2), "crates/foo");
    assert_eq!(
        module_key("packages/bar/src/main.rs", &r, 2),
        "packages/bar"
    );
}

#[test]
fn non_root_dir_uses_first_segment_only() {
    assert_eq!(
        module_key("src/deep/nested/file.rs", &roots(&["crates"]), 2),
        "src"
    );
}

#[test]
fn no_module_roots_means_first_segment_always() {
    let empty: Vec<String> = vec![];
    assert_eq!(module_key("crates/foo/lib.rs", &empty, 2), "crates");
    assert_eq!(module_key("src/lib.rs", &empty, 2), "src");
}

// ── dots in paths ──────────────────────────────────────────────────

#[test]
fn dots_in_directory_names_preserved() {
    assert_eq!(
        module_key("node_modules/.cache/file.js", &roots(&["node_modules"]), 2),
        "node_modules/.cache"
    );
}

#[test]
fn dotted_filenames_in_root() {
    assert_eq!(
        module_key(".eslintrc.json", &roots(&["crates"]), 2),
        "(root)"
    );
}

#[test]
fn extension_like_dir_names() {
    assert_eq!(
        module_key("src.backup/main.rs", &roots(&["crates"]), 2),
        "src.backup"
    );
}

// ── module_key_from_normalized: special segments ───────────────────

#[test]
fn normalized_double_slash_skips_empty_segments() {
    assert_eq!(
        module_key_from_normalized("crates//foo/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn normalized_dot_segment_skipped() {
    assert_eq!(
        module_key_from_normalized("crates/./foo/lib.rs", &roots(&["crates"]), 2),
        "crates/foo"
    );
}

#[test]
fn normalized_root_file_returns_root() {
    assert_eq!(
        module_key_from_normalized("README.md", &roots(&["crates"]), 2),
        "(root)"
    );
}

#[test]
fn normalized_only_dot_dir_is_root() {
    assert_eq!(
        module_key_from_normalized("./lib.rs", &roots(&["crates"]), 2),
        "(root)"
    );
}

// ── deeply nested paths ────────────────────────────────────────────

#[test]
fn deeply_nested_non_root_uses_first_segment() {
    assert_eq!(
        module_key("tools/gen/v2/internal/file.rs", &roots(&["crates"]), 2),
        "tools"
    );
}

#[test]
fn deeply_nested_module_root_respects_depth() {
    assert_eq!(
        module_key(
            "crates/tokmd-scan/src/internal/detail/scanner.rs",
            &roots(&["crates"]),
            2
        ),
        "crates/tokmd-scan"
    );
}

// ── determinism: same input → same output ──────────────────────────

#[test]
fn deterministic_across_calls() {
    let r = roots(&["crates"]);
    let path = "crates/foo/src/lib.rs";
    let first = module_key(path, &r, 2);
    let second = module_key(path, &r, 2);
    assert_eq!(first, second);
}
