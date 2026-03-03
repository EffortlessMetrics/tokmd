//! Deep tests for tokmd-redact: BLAKE3 hashing, path redaction, edge cases.

use tokmd_redact::{redact_path, short_hash};

// ==============================
// short_hash basics
// ==============================

#[test]
fn short_hash_always_16_chars() {
    for input in &[
        "",
        "a",
        "hello world",
        "src/lib.rs",
        "very/deep/nested/path/to/file.txt",
    ] {
        assert_eq!(short_hash(input).len(), 16, "failed for input: {input}");
    }
}

#[test]
fn short_hash_hex_only() {
    let h = short_hash("test-input");
    assert!(
        h.chars().all(|c| c.is_ascii_hexdigit()),
        "hash should be hex: {h}"
    );
}

#[test]
fn short_hash_deterministic() {
    let a = short_hash("deterministic");
    let b = short_hash("deterministic");
    let c = short_hash("deterministic");
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn short_hash_different_inputs_differ() {
    let h1 = short_hash("alpha");
    let h2 = short_hash("beta");
    assert_ne!(h1, h2);
}

#[test]
fn short_hash_empty_string() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
    // Empty string still produces a valid hash
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

// ==============================
// Cross-platform normalization
// ==============================

#[test]
fn short_hash_backslash_forward_slash_equal() {
    assert_eq!(short_hash("src/main.rs"), short_hash("src\\main.rs"));
}

#[test]
fn short_hash_mixed_separators() {
    let a = short_hash("a/b/c/d");
    let b = short_hash("a\\b\\c\\d");
    let c = short_hash("a/b\\c/d");
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn short_hash_dot_prefix_normalized() {
    assert_eq!(short_hash("src/lib.rs"), short_hash("./src/lib.rs"));
}

#[test]
fn short_hash_double_dot_prefix_normalized() {
    assert_eq!(short_hash("src/lib.rs"), short_hash("././src/lib.rs"));
}

#[test]
fn short_hash_interior_dot_segment_normalized() {
    assert_eq!(
        short_hash("crates/foo/src/lib.rs"),
        short_hash("crates/foo/./src/lib.rs")
    );
}

#[test]
fn short_hash_trailing_dot_normalized() {
    assert_eq!(short_hash("src"), short_hash("src/."));
}

// ==============================
// redact_path basics
// ==============================

#[test]
fn redact_path_preserves_rs_extension() {
    let r = redact_path("src/lib.rs");
    assert!(r.ends_with(".rs"), "expected .rs suffix: {r}");
}

#[test]
fn redact_path_preserves_json_extension() {
    let r = redact_path("config/settings.json");
    assert!(r.ends_with(".json"));
}

#[test]
fn redact_path_preserves_py_extension() {
    let r = redact_path("scripts/deploy.py");
    assert!(r.ends_with(".py"));
}

#[test]
fn redact_path_no_extension_just_hash() {
    let r = redact_path("Makefile");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_path_double_extension_keeps_last() {
    let r = redact_path("backup.tar.gz");
    assert!(r.ends_with(".gz"));
    assert!(!r.contains(".tar"));
}

#[test]
fn redact_path_deterministic() {
    let a = redact_path("secret/credentials.yaml");
    let b = redact_path("secret/credentials.yaml");
    assert_eq!(a, b);
}

#[test]
fn redact_path_different_paths_differ() {
    let a = redact_path("a/file.rs");
    let b = redact_path("b/file.rs");
    assert_ne!(a, b);
}

// ==============================
// redact_path cross-platform
// ==============================

#[test]
fn redact_path_backslash_equals_forward() {
    assert_eq!(redact_path("src/main.rs"), redact_path("src\\main.rs"));
}

#[test]
fn redact_path_dot_prefix_equivalent() {
    assert_eq!(redact_path("src/main.rs"), redact_path("./src/main.rs"));
}

#[test]
fn redact_path_deep_backslash_normalization() {
    assert_eq!(redact_path("a/b/c/d.txt"), redact_path("a\\b\\c\\d.txt"));
}

// ==============================
// Edge cases
// ==============================

#[test]
fn redact_path_empty_string() {
    let r = redact_path("");
    assert_eq!(r.len(), 16);
}

#[test]
fn redact_path_dot_only() {
    // "." has no extension
    let r = redact_path(".");
    // After normalization "." becomes empty-ish, but still produces 16-char hash
    assert_eq!(r.len(), 16);
}

#[test]
fn redact_path_unicode_filename() {
    let r = redact_path("日本語/ファイル.rs");
    assert!(r.ends_with(".rs"));
    assert_eq!(r.len(), 16 + 3); // hash + ".rs"
}

#[test]
fn redact_path_spaces_in_path() {
    let r = redact_path("my project/source file.rs");
    assert!(r.ends_with(".rs"));
}

#[test]
fn short_hash_unicode_deterministic() {
    let a = short_hash("日本語パス");
    let b = short_hash("日本語パス");
    assert_eq!(a, b);
    assert_eq!(a.len(), 16);
}

#[test]
fn redact_path_hidden_file_no_extension() {
    let r = redact_path(".hidden_config");
    // ".hidden_config" has no extension per std::path::Path rules
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_path_length_with_extension() {
    let r = redact_path("any/path.toml");
    // 16 hex + "." + "toml" = 21
    assert_eq!(r.len(), 16 + 1 + 4);
}
