//! Edge-case tests for the redaction API surface.
//!
//! Covers determinism, collision resistance, lexical normalization,
//! Unicode handling, long paths, empty paths, and directory-structure
//! consistency.

use tokmd_redact::{redact_path, short_hash};

// ========================
// Determinism
// ========================

#[test]
fn short_hash_is_deterministic_across_calls() {
    let inputs = [
        "src/lib.rs",
        "Cargo.toml",
        "",
        "deeply/nested/path/to/file.txt",
        "日本語/パス.rs",
    ];
    for input in &inputs {
        assert_eq!(
            short_hash(input),
            short_hash(input),
            "short_hash must be deterministic for {:?}",
            input
        );
    }
}

#[test]
fn redact_path_is_deterministic_across_calls() {
    let inputs = [
        "src/main.rs",
        "README.md",
        "no_extension",
        ".hidden",
        "a/b/c/d.json",
    ];
    for input in &inputs {
        assert_eq!(
            redact_path(input),
            redact_path(input),
            "redact_path must be deterministic for {:?}",
            input
        );
    }
}

// ========================
// Collision resistance
// ========================

#[test]
fn common_paths_produce_distinct_hashes() {
    let paths = [
        "src/lib.rs",
        "src/main.rs",
        "src/lib.ts",
        "lib/src.rs",
        "Cargo.toml",
        "package.json",
        "README.md",
        "LICENSE",
        "src/mod.rs",
        "tests/integration.rs",
    ];
    let hashes: Vec<String> = paths.iter().map(|p| short_hash(p)).collect();
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i], hashes[j],
                "collision between {:?} and {:?}",
                paths[i], paths[j]
            );
        }
    }
}

#[test]
fn similar_paths_differing_by_one_char_are_distinct() {
    let pairs = [
        ("src/foo.rs", "src/foO.rs"),
        ("src/bar.rs", "src/baz.rs"),
        ("a/b/c", "a/b/d"),
        ("file1.txt", "file2.txt"),
    ];
    for (a, b) in &pairs {
        assert_ne!(
            short_hash(a),
            short_hash(b),
            "near-collision between {:?} and {:?}",
            a,
            b
        );
    }
}

#[test]
fn redact_path_distinguishes_same_stem_different_extension() {
    let r1 = redact_path("config.json");
    let r2 = redact_path("config.yaml");
    // Different extensions AND different hashes
    assert_ne!(r1, r2);
    assert!(r1.ends_with(".json"));
    assert!(r2.ends_with(".yaml"));
}

// ========================
// Lexical normalization
// ========================

#[test]
fn dot_prefix_is_normalized() {
    assert_eq!(short_hash("./src/lib.rs"), short_hash("src/lib.rs"));
    assert_eq!(redact_path("./src/lib.rs"), redact_path("src/lib.rs"));
}

#[test]
fn multiple_dot_prefixes_are_normalized() {
    assert_eq!(short_hash("./././a/b.rs"), short_hash("a/b.rs"));
}

#[test]
fn interior_dot_segments_are_normalized() {
    assert_eq!(
        short_hash("crates/foo/./src/./lib.rs"),
        short_hash("crates/foo/src/lib.rs")
    );
}

#[test]
fn trailing_dot_is_normalized() {
    assert_eq!(short_hash("src/."), short_hash("src"));
}

#[test]
fn backslash_separators_are_normalized() {
    assert_eq!(
        short_hash("crates\\redact\\src\\lib.rs"),
        short_hash("crates/redact/src/lib.rs")
    );
    assert_eq!(
        redact_path("crates\\redact\\src\\lib.rs"),
        redact_path("crates/redact/src/lib.rs")
    );
}

#[test]
fn mixed_separators_with_dots_normalize_consistently() {
    assert_eq!(short_hash(".\\src\\.\\lib.rs"), short_hash("src/lib.rs"),);
}

// ========================
// Unicode paths
// ========================

#[test]
fn unicode_path_produces_valid_hex_hash() {
    let h = short_hash("日本語/ソース/メイン.rs");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn unicode_path_redaction_preserves_extension() {
    let r = redact_path("données/café.py");
    assert!(r.ends_with(".py"));
}

#[test]
fn emoji_path_is_handled() {
    let h = short_hash("🚀/launch.rs");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn unicode_determinism() {
    let path = "中文/路径/文件.txt";
    assert_eq!(short_hash(path), short_hash(path));
    assert_eq!(redact_path(path), redact_path(path));
}

// ========================
// Very long paths
// ========================

#[test]
fn very_long_path_produces_fixed_length_hash() {
    let long_segment = "a".repeat(1000);
    let long_path = format!("{}/{}/{}.rs", long_segment, long_segment, long_segment);
    let h = short_hash(&long_path);
    assert_eq!(
        h.len(),
        16,
        "hash length must be 16 regardless of input length"
    );
}

#[test]
fn very_long_path_redaction_preserves_extension() {
    let long_path = format!("{}/file.json", "deep/".repeat(200));
    let r = redact_path(&long_path);
    assert!(r.ends_with(".json"));
    // hash(16) + ".json"(5) = 21
    assert_eq!(r.len(), 21);
}

// ========================
// Empty and minimal paths
// ========================

#[test]
fn empty_string_hash_is_valid() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn empty_string_redact_has_no_extension() {
    let r = redact_path("");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn single_char_path() {
    let h = short_hash("x");
    assert_eq!(h.len(), 16);
    assert_ne!(h, short_hash("y"));
}

#[test]
fn extension_only_path() {
    // ".rs" is a dotfile in Rust's Path semantics → no extension
    let r = redact_path(".rs");
    assert_eq!(r.len(), 16);
}

// ========================
// Directory structure consistency
// ========================

#[test]
fn parent_child_paths_have_different_hashes() {
    let parent = short_hash("src");
    let child = short_hash("src/lib.rs");
    assert_ne!(parent, child, "parent and child must hash differently");
}

#[test]
fn sibling_files_in_same_directory_have_different_hashes() {
    let a = redact_path("src/foo.rs");
    let b = redact_path("src/bar.rs");
    assert_ne!(a, b);
}

#[test]
fn same_filename_in_different_directories_have_different_hashes() {
    let a = redact_path("crate_a/src/lib.rs");
    let b = redact_path("crate_b/src/lib.rs");
    assert_ne!(
        a, b,
        "same filename in different dirs must hash differently"
    );
}

// ========================
// Roundtrip / mapping consistency
// ========================

#[test]
fn same_salt_free_input_always_maps_to_same_output() {
    // Since there's no salt parameter, "roundtrip" means:
    // the same input always produces the same redacted form,
    // so a lookup table built from one pass is valid for subsequent passes.
    let inputs = [
        "src/lib.rs",
        "src/main.rs",
        "tests/integration.rs",
        "Cargo.toml",
    ];
    let pass1: Vec<(String, String)> = inputs
        .iter()
        .map(|p| (short_hash(p), redact_path(p)))
        .collect();
    let pass2: Vec<(String, String)> = inputs
        .iter()
        .map(|p| (short_hash(p), redact_path(p)))
        .collect();
    assert_eq!(pass1, pass2, "two passes must produce identical mappings");
}

#[test]
fn redact_path_hash_prefix_matches_short_hash() {
    // For paths without extension, redact_path == short_hash
    let path = "Makefile";
    assert_eq!(redact_path(path), short_hash(path));
}

#[test]
fn redact_path_hash_component_matches_short_hash_for_extensioned_files() {
    // For paths with extension, the hash prefix of redact_path
    // equals short_hash of the full path (not just the stem).
    let path = "src/lib.rs";
    let redacted = redact_path(path);
    let hash = short_hash(path);
    assert!(
        redacted.starts_with(&hash),
        "redacted {:?} should start with hash {:?}",
        redacted,
        hash
    );
}
