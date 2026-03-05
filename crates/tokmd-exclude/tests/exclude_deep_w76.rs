//! Deep tests for tokmd-exclude — W76
//!
//! Covers: normalize_exclude_pattern for absolute/relative/edge-case paths,
//! has_exclude_pattern matching semantics, add_exclude_pattern dedup and
//! ordering, cross-platform slash handling, and idempotency.

use std::path::Path;

use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

// ═══════════════════════════════════════════════════════════════════
// 1. normalize_exclude_pattern — relative path variants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_normalize_relative_plain() {
    let root = Path::new("/repo");
    assert_eq!(normalize_exclude_pattern(root, Path::new("out")), "out");
}

#[test]
fn w76_normalize_relative_dot_prefix() {
    let root = Path::new("/repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./build/out")),
        "build/out"
    );
}

#[test]
fn w76_normalize_relative_deep_nesting() {
    let root = Path::new("/repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("./a/b/c/d/e.txt")),
        "a/b/c/d/e.txt"
    );
}

#[test]
fn w76_normalize_relative_backslash_to_forward() {
    let root = Path::new("/repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("out\\dist\\bundle.js")),
        "out/dist/bundle.js"
    );
}

#[test]
fn w76_normalize_relative_mixed_slashes() {
    let root = Path::new("/repo");
    assert_eq!(
        normalize_exclude_pattern(root, Path::new("a/b\\c/d")),
        "a/b/c/d"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 2. normalize_exclude_pattern — absolute path stripping
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_normalize_absolute_under_root_stripped() {
    let root = std::env::temp_dir().join("w76-root");
    let abs = root.join("target").join("debug");
    let got = normalize_exclude_pattern(&root, &abs);
    assert_eq!(got, "target/debug");
}

#[test]
fn w76_normalize_absolute_outside_root_kept() {
    let root = std::env::temp_dir().join("w76-root");
    let outside = std::env::temp_dir().join("w76-other").join("file.txt");
    let got = normalize_exclude_pattern(&root, &outside);
    // Should contain the outside path segments, not be stripped
    assert!(got.contains("w76-other"));
    assert!(got.contains("file.txt"));
}

#[test]
fn w76_normalize_absolute_single_component_under_root() {
    let root = std::env::temp_dir().join("w76-root");
    let abs = root.join("node_modules");
    let got = normalize_exclude_pattern(&root, &abs);
    assert_eq!(got, "node_modules");
}

// ═══════════════════════════════════════════════════════════════════
// 3. has_exclude_pattern — matching semantics
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_has_exact_match() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out/bundle"));
}

#[test]
fn w76_has_match_normalizes_dot_prefix() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "./out/bundle"));
}

#[test]
fn w76_has_match_normalizes_backslash() {
    let existing = vec!["out/bundle".to_string()];
    assert!(has_exclude_pattern(&existing, "out\\bundle"));
}

#[test]
fn w76_has_no_match_different_path() {
    let existing = vec!["out/bundle".to_string()];
    assert!(!has_exclude_pattern(&existing, "dist/app"));
}

#[test]
fn w76_has_match_empty_existing_always_false() {
    assert!(!has_exclude_pattern(&[], "anything"));
}

#[test]
fn w76_has_match_existing_also_normalized() {
    // Existing entry has backslash; query is forward slash — should match
    let existing = vec!["./a\\b".to_string()];
    assert!(has_exclude_pattern(&existing, "a/b"));
}

// ═══════════════════════════════════════════════════════════════════
// 4. add_exclude_pattern — dedup, ordering, edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_add_new_pattern_returns_true() {
    let mut v = vec![];
    assert!(add_exclude_pattern(&mut v, "target".to_string()));
    assert_eq!(v, vec!["target"]);
}

#[test]
fn w76_add_duplicate_returns_false() {
    let mut v = vec!["target".to_string()];
    assert!(!add_exclude_pattern(&mut v, "target".to_string()));
    assert_eq!(v.len(), 1);
}

#[test]
fn w76_add_normalized_duplicate_returns_false() {
    let mut v = vec!["out/dist".to_string()];
    assert!(!add_exclude_pattern(&mut v, "./out/dist".to_string()));
    assert_eq!(v.len(), 1);
}

#[test]
fn w76_add_empty_rejected() {
    let mut v = vec![];
    assert!(!add_exclude_pattern(&mut v, String::new()));
    assert!(v.is_empty());
}

#[test]
fn w76_add_preserves_insertion_order() {
    let mut v = vec![];
    add_exclude_pattern(&mut v, "c".to_string());
    add_exclude_pattern(&mut v, "a".to_string());
    add_exclude_pattern(&mut v, "b".to_string());
    assert_eq!(v, vec!["c", "a", "b"]);
}

#[test]
fn w76_add_backslash_deduped_against_forward_slash() {
    let mut v = vec!["a/b".to_string()];
    assert!(!add_exclude_pattern(&mut v, "a\\b".to_string()));
    assert_eq!(v.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// 5. Idempotency / normalization invariants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_normalize_idempotent() {
    let root = Path::new("/repo");
    let first = normalize_exclude_pattern(root, Path::new("./out/bundle"));
    let second = normalize_exclude_pattern(root, Path::new(&first));
    assert_eq!(first, second, "normalization must be idempotent");
}
