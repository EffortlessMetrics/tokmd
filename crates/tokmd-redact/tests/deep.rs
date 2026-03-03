//! Deep tests for tokmd-redact.
//!
//! Covers BLAKE3 determinism, collision resistance, path edge cases
//! (empty, long, Unicode, special characters), output format invariants,
//! irreversibility, and property-based invariants.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// ============================================================================
// 1. Determinism: same input = same output
// ============================================================================

#[test]
fn determinism_same_path_same_hash() {
    let h1 = short_hash("src/main.rs");
    let h2 = short_hash("src/main.rs");
    assert_eq!(h1, h2);
}

#[test]
fn determinism_redact_path_stable() {
    let r1 = redact_path("crates/tokmd-redact/src/lib.rs");
    let r2 = redact_path("crates/tokmd-redact/src/lib.rs");
    assert_eq!(r1, r2);
}

#[test]
fn determinism_across_many_calls() {
    let input = "deep/nested/path/to/file.rs";
    let first = short_hash(input);
    for _ in 0..100 {
        assert_eq!(short_hash(input), first);
    }
}

// ============================================================================
// 2. Different paths = different outputs
// ============================================================================

#[test]
fn different_paths_different_hashes() {
    let h1 = short_hash("src/main.rs");
    let h2 = short_hash("src/lib.rs");
    assert_ne!(h1, h2);
}

#[test]
fn different_paths_different_redactions() {
    let r1 = redact_path("src/main.rs");
    let r2 = redact_path("src/lib.rs");
    assert_ne!(r1, r2);
}

#[test]
fn similar_paths_differ() {
    // Paths differing by a single character
    assert_ne!(short_hash("src/foo.rs"), short_hash("src/foO.rs"));
    assert_ne!(short_hash("src/a.rs"), short_hash("src/b.rs"));
}

// ============================================================================
// 3. Different salts (paths acting as their own salt)
// ============================================================================

#[test]
fn prefix_variation_produces_different_hashes() {
    // The crate doesn't take an explicit salt, but different prefixes
    // should produce different hashes.
    let h1 = short_hash("project_a/src/lib.rs");
    let h2 = short_hash("project_b/src/lib.rs");
    assert_ne!(h1, h2);
}

#[test]
fn case_sensitivity_matters() {
    let h1 = short_hash("Src/Main.rs");
    let h2 = short_hash("src/main.rs");
    assert_ne!(h1, h2, "Hashing should be case-sensitive");
}

// ============================================================================
// 4. Empty path handling
// ============================================================================

#[test]
fn empty_path_short_hash_produces_valid_hex() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn empty_path_redact_produces_hash_only() {
    let r = redact_path("");
    // Empty string has no extension
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn empty_path_is_deterministic() {
    assert_eq!(short_hash(""), short_hash(""));
    assert_eq!(redact_path(""), redact_path(""));
}

// ============================================================================
// 5. Long path handling (>1000 chars)
// ============================================================================

#[test]
fn long_path_over_1000_chars() {
    let long = "a/".repeat(600); // 1200 chars
    let h = short_hash(&long);
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn very_long_path_with_extension() {
    let long = format!("{}.rs", "x".repeat(2000));
    let r = redact_path(&long);
    assert!(r.ends_with(".rs"));
    assert_eq!(r.len(), 16 + 1 + 2); // hash + dot + "rs"
}

#[test]
fn long_path_deterministic() {
    let long = "deep/".repeat(500);
    let h1 = short_hash(&long);
    let h2 = short_hash(&long);
    assert_eq!(h1, h2);
}

#[test]
fn long_path_differs_from_short() {
    let long = "a/".repeat(600);
    let short = "a/";
    assert_ne!(short_hash(&long), short_hash(short));
}

// ============================================================================
// 6. Unicode path handling
// ============================================================================

#[test]
fn unicode_path_chinese() {
    let h = short_hash("项目/源代码/主.rs");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn unicode_path_emoji() {
    let h = short_hash("🦀/src/lib.rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn unicode_path_deterministic() {
    let path = "données/fichier.py";
    assert_eq!(short_hash(path), short_hash(path));
}

#[test]
fn unicode_redact_preserves_extension() {
    let r = redact_path("日本語/テスト.json");
    assert!(r.ends_with(".json"));
}

#[test]
fn unicode_paths_differ() {
    assert_ne!(short_hash("café/lait"), short_hash("cafe/lait"));
}

// ============================================================================
// 7. Special characters
// ============================================================================

#[test]
fn special_chars_spaces() {
    let h = short_hash("path with spaces/file name.rs");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn special_chars_brackets_parens() {
    let h = short_hash("src/[test](foo).rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn special_chars_at_and_hash() {
    let h = short_hash("@scope/package#1.0");
    assert_eq!(h.len(), 16);
}

#[test]
fn special_chars_null_byte() {
    let h = short_hash("before\0after");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn special_chars_newlines() {
    let h = short_hash("line1\nline2");
    assert_eq!(h.len(), 16);
}

#[test]
fn special_chars_tabs() {
    let h = short_hash("col1\tcol2");
    assert_eq!(h.len(), 16);
}

// ============================================================================
// 8. Irreversibility
// ============================================================================

#[test]
fn redaction_hides_original_path() {
    let original = "secret/internal/api/keys.json";
    let redacted = redact_path(original);
    // Redacted output should not contain any segment of the original path
    assert!(!redacted.contains("secret"));
    assert!(!redacted.contains("internal"));
    assert!(!redacted.contains("api"));
    assert!(!redacted.contains("keys"));
    // Only the extension is preserved
    assert!(redacted.ends_with(".json"));
}

#[test]
fn hash_does_not_contain_original() {
    let original = "my_project/src/database/connection.rs";
    let h = short_hash(original);
    assert!(!h.contains("my_project"));
    assert!(!h.contains("database"));
    assert!(!h.contains("connection"));
}

#[test]
fn redacted_output_is_fixed_length() {
    // Regardless of original path length, hash portion is always 16 chars
    let short_r = redact_path("a.rs");
    let long_r = redact_path(&format!("{}.rs", "x".repeat(1000)));
    assert_eq!(short_r.len(), long_r.len());
    assert_eq!(short_r.len(), 16 + 1 + 2); // hash.rs
}

// ============================================================================
// 9. Output format: consistent length, valid hex
// ============================================================================

#[test]
fn output_format_hash_always_16_hex() {
    let inputs = [
        "",
        "a",
        "src/lib.rs",
        "very/long/path/to/file.txt",
        "🦀",
        "données",
        "path with spaces",
    ];
    for input in &inputs {
        let h = short_hash(input);
        assert_eq!(h.len(), 16, "Input: {}", input);
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "Not hex for: {}",
            input
        );
        // Must be lowercase
        assert_eq!(h, h.to_lowercase(), "Must be lowercase for: {}", input);
    }
}

#[test]
fn output_format_redact_with_ext_is_hash_dot_ext() {
    let r = redact_path("foo/bar.toml");
    assert_eq!(r.len(), 16 + 1 + 4); // hash + '.' + "toml"
    assert!(r[..16].chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(&r[16..17], ".");
    assert_eq!(&r[17..], "toml");
}

#[test]
fn output_format_redact_no_ext_is_hash_only() {
    let r = redact_path("Makefile");
    assert_eq!(r.len(), 16);
    assert!(r.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn output_format_redact_never_contains_path_separators() {
    let inputs = ["a/b/c.rs", "a\\b\\c.rs", "very/deep/nested/path/file.py"];
    for input in &inputs {
        let r = redact_path(input);
        assert!(!r.contains('/'), "Should not contain /: {}", r);
        assert!(!r.contains('\\'), "Should not contain \\: {}", r);
    }
}

// ============================================================================
// 10. Proptest: arbitrary strings always produce valid redacted output
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn proptest_short_hash_always_valid(input in "\\PC*") {
        let h = short_hash(&input);
        prop_assert_eq!(h.len(), 16, "Hash length must be 16 for input: {:?}", input);
        prop_assert!(
            h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "Hash must be lowercase hex, got: {} for input: {:?}",
            h,
            input
        );
    }

    #[test]
    fn proptest_redact_path_always_valid(input in "\\PC*") {
        let r = redact_path(&input);
        // Must be at least 16 chars (hash only)
        prop_assert!(r.len() >= 16, "Redacted too short: {} for input: {:?}", r, input);
        // First 16 chars must be hex
        prop_assert!(
            r[..16].chars().all(|c| c.is_ascii_hexdigit()),
            "Hash portion not hex: {} for input: {:?}",
            r,
            input
        );
        // Must not contain path separators
        prop_assert!(!r.contains('/'), "Contains /: {}", r);
        prop_assert!(!r.contains('\\'), "Contains \\: {}", r);
    }

    #[test]
    fn proptest_redact_path_extension_preserved_or_absent(
        base in "[a-z]{1,10}",
        ext in "[a-z]{1,5}"
    ) {
        let with_ext = format!("{}.{}", base, ext);
        let r = redact_path(&with_ext);
        prop_assert!(
            r.ends_with(&format!(".{}", ext)),
            "Extension not preserved: input={}, redacted={}",
            with_ext,
            r
        );
        prop_assert_eq!(r.len(), 16 + 1 + ext.len());
    }
}

// ============================================================================
// 11. Proptest: redaction is deterministic (run twice = same result)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn proptest_short_hash_deterministic(input in "\\PC*") {
        let h1 = short_hash(&input);
        let h2 = short_hash(&input);
        prop_assert_eq!(h1, h2, "short_hash not deterministic for: {:?}", input);
    }

    #[test]
    fn proptest_redact_path_deterministic(input in "\\PC*") {
        let r1 = redact_path(&input);
        let r2 = redact_path(&input);
        prop_assert_eq!(r1, r2, "redact_path not deterministic for: {:?}", input);
    }

    #[test]
    fn proptest_different_inputs_different_hashes(
        a in "[a-zA-Z0-9]{1,30}",
        b in "[a-zA-Z0-9]{1,30}"
    ) {
        prop_assume!(a != b);
        let h1 = short_hash(&a);
        let h2 = short_hash(&b);
        prop_assert_ne!(h1.clone(), h2, "Collision: {} and {} both hash to {}", a, b, h1);
    }
}

// ============================================================================
// Additional deep edge cases
// ============================================================================

#[test]
fn separator_normalization_preserves_determinism() {
    let paths = [
        ("src/lib.rs", "src\\lib.rs"),
        ("a/b/c/d.py", "a\\b\\c\\d.py"),
        ("crates/foo/src/main.rs", "crates\\foo\\src\\main.rs"),
    ];
    for (unix, win) in &paths {
        assert_eq!(short_hash(unix), short_hash(win));
        assert_eq!(redact_path(unix), redact_path(win));
    }
}

#[test]
fn dot_slash_normalization() {
    assert_eq!(short_hash("src/lib.rs"), short_hash("./src/lib.rs"));
    assert_eq!(short_hash("src/lib.rs"), short_hash("././src/lib.rs"));
    assert_eq!(redact_path("src/lib.rs"), redact_path("./src/lib.rs"));
}

#[test]
fn interior_dot_segments_collapsed() {
    assert_eq!(
        short_hash("crates/foo/src/lib.rs"),
        short_hash("crates/foo/./src/lib.rs")
    );
}

#[test]
fn trailing_dot_removed() {
    assert_eq!(short_hash("src"), short_hash("src/."));
}

#[test]
fn hash_of_only_dots_and_slashes() {
    // Degenerate inputs should still produce valid output
    let h = short_hash("./././.");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn redact_path_double_extension_preserves_last() {
    let r = redact_path("archive.tar.gz");
    assert!(r.ends_with(".gz"));
    assert!(!r.contains("tar"));
}

#[test]
fn redact_path_dotfile_no_extension() {
    let r = redact_path(".gitignore");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_path_trailing_dot_no_extension() {
    let r = redact_path("file.");
    assert_eq!(r.len(), 16);
}
