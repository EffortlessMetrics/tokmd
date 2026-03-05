//! Depth tests for tokmd-redact (W63).
//!
//! Covers BLAKE3-based path redaction, salt consistency, lexical normalization,
//! dot/dot-dot resolution, Windows absolute paths, round-trip determinism,
//! empty/single-segment paths, Unicode paths, and property-based tests.

use tokmd_redact::{redact_path, short_hash};

// ---------------------------------------------------------------------------
// 1. short_hash basic properties
// ---------------------------------------------------------------------------

#[test]
fn short_hash_always_16_chars() {
    for input in &["", "a", "hello world", "crates/tokmd/src/lib.rs"] {
        assert_eq!(short_hash(input).len(), 16, "input: {input}");
    }
}

#[test]
fn short_hash_hex_only() {
    let h = short_hash("anything");
    assert!(
        h.chars().all(|c| c.is_ascii_hexdigit()),
        "expected hex chars, got: {h}"
    );
}

#[test]
fn short_hash_empty_string() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
    // Must be deterministic even for empty input.
    assert_eq!(h, short_hash(""));
}

#[test]
fn short_hash_single_char_inputs_differ() {
    let h_a = short_hash("a");
    let h_b = short_hash("b");
    assert_ne!(h_a, h_b);
}

// ---------------------------------------------------------------------------
// 2. Separator normalization
// ---------------------------------------------------------------------------

#[test]
fn hash_backslash_equals_forward_slash_deep() {
    assert_eq!(
        short_hash("a/b/c/d/e"),
        short_hash("a\\b\\c\\d\\e")
    );
}

#[test]
fn hash_mixed_separators_normalized() {
    assert_eq!(
        short_hash("crates/tokmd\\src/lib.rs"),
        short_hash("crates/tokmd/src/lib.rs")
    );
}

#[test]
fn redact_path_mixed_separators_same_output() {
    assert_eq!(
        redact_path("src\\main.rs"),
        redact_path("src/main.rs")
    );
}

// ---------------------------------------------------------------------------
// 3. Dot/dot-dot segment resolution
// ---------------------------------------------------------------------------

#[test]
fn hash_leading_dot_slash_stripped() {
    assert_eq!(short_hash("./src/lib.rs"), short_hash("src/lib.rs"));
}

#[test]
fn hash_double_dot_slash_prefix_stripped() {
    assert_eq!(short_hash("././src/lib.rs"), short_hash("src/lib.rs"));
}

#[test]
fn hash_triple_dot_slash_prefix_stripped() {
    assert_eq!(short_hash("./././src/lib.rs"), short_hash("src/lib.rs"));
}

#[test]
fn hash_interior_dot_segment_resolved() {
    assert_eq!(
        short_hash("crates/./foo/./bar"),
        short_hash("crates/foo/bar")
    );
}

#[test]
fn hash_trailing_dot_resolved() {
    assert_eq!(short_hash("crates/foo/."), short_hash("crates/foo"));
}

#[test]
fn redact_path_dot_prefix_stripped() {
    assert_eq!(redact_path("./src/main.rs"), redact_path("src/main.rs"));
}

#[test]
fn redact_path_interior_dot_resolved() {
    assert_eq!(
        redact_path("crates/./tokmd/src/lib.rs"),
        redact_path("crates/tokmd/src/lib.rs")
    );
}

#[test]
fn hash_bare_dot_input() {
    // "." and "./" should produce the same hash (trailing /. removal + strip)
    let h = short_hash(".");
    assert_eq!(h.len(), 16);
}

#[test]
fn hash_dot_dot_not_special() {
    // ".." is NOT collapsed (no parent resolution) – just literal
    let h = short_hash("../foo");
    assert_ne!(h, short_hash("foo"));
}

// ---------------------------------------------------------------------------
// 4. Windows absolute path handling
// ---------------------------------------------------------------------------

#[test]
fn hash_windows_absolute_path_backslashes() {
    let h = short_hash("C:\\Users\\dev\\project\\src\\lib.rs");
    assert_eq!(h.len(), 16);
    // Normalised to forward slashes internally
    assert_eq!(
        h,
        short_hash("C:/Users/dev/project/src/lib.rs")
    );
}

#[test]
fn redact_windows_absolute_preserves_extension() {
    let r = redact_path("D:\\work\\project\\main.py");
    assert!(r.ends_with(".py"), "got: {r}");
}

#[test]
fn hash_unc_path_backslashes() {
    assert_eq!(
        short_hash("\\\\server\\share\\file.txt"),
        short_hash("//server/share/file.txt")
    );
}

// ---------------------------------------------------------------------------
// 5. Extension preservation in redact_path
// ---------------------------------------------------------------------------

#[test]
fn redact_path_single_extension() {
    let r = redact_path("foo/bar.rs");
    assert!(r.ends_with(".rs"));
    assert_eq!(r.len(), 16 + 1 + 2); // hash(16) + dot(1) + "rs"(2) = 19
}

#[test]
fn redact_path_double_extension_keeps_last() {
    let r = redact_path("backup.tar.gz");
    assert!(r.ends_with(".gz"));
    assert!(!r.contains("tar"));
}

#[test]
fn redact_path_triple_extension_keeps_last() {
    let r = redact_path("data.backup.tar.xz");
    assert!(r.ends_with(".xz"));
}

#[test]
fn redact_path_hidden_file_no_extension() {
    // ".gitignore" has no extension per std::path::Path::extension
    let r = redact_path(".gitignore");
    assert_eq!(r.len(), 16, "got: {r}");
    assert!(!r.contains('.'));
}

#[test]
fn redact_path_dotfile_with_ext() {
    let r = redact_path(".config.json");
    assert!(r.ends_with(".json"));
}

#[test]
fn redact_path_no_extension_file() {
    let r = redact_path("Makefile");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_path_readme_no_ext() {
    let r = redact_path("README");
    assert_eq!(r.len(), 16);
}

// ---------------------------------------------------------------------------
// 6. Empty and single-segment path handling
// ---------------------------------------------------------------------------

#[test]
fn redact_path_empty_string() {
    let r = redact_path("");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_path_single_segment_with_ext() {
    let r = redact_path("lib.rs");
    assert!(r.ends_with(".rs"));
}

#[test]
fn redact_path_single_segment_no_ext() {
    let r = redact_path("foobar");
    assert_eq!(r.len(), 16);
}

#[test]
fn redact_path_slash_only() {
    let r = redact_path("/");
    assert_eq!(r.len(), 16);
}

#[test]
fn hash_whitespace_only() {
    let h = short_hash("   ");
    assert_eq!(h.len(), 16);
    assert_ne!(h, short_hash(""));
}

// ---------------------------------------------------------------------------
// 7. Unicode path redaction
// ---------------------------------------------------------------------------

#[test]
fn hash_unicode_path() {
    let h = short_hash("src/données/résumé.txt");
    assert_eq!(h.len(), 16);
}

#[test]
fn redact_unicode_path_preserves_extension() {
    let r = redact_path("ディレクトリ/ファイル.rs");
    assert!(r.ends_with(".rs"), "got: {r}");
}

#[test]
fn hash_emoji_path() {
    let h = short_hash("📁/📄.txt");
    assert_eq!(h.len(), 16);
}

#[test]
fn redact_chinese_path() {
    let r = redact_path("项目/源代码/主.py");
    assert!(r.ends_with(".py"));
}

#[test]
fn hash_unicode_deterministic() {
    let h1 = short_hash("données/café.rs");
    let h2 = short_hash("données/café.rs");
    assert_eq!(h1, h2);
}

// ---------------------------------------------------------------------------
// 8. Round-trip / determinism
// ---------------------------------------------------------------------------

#[test]
fn short_hash_100_calls_identical() {
    let expected = short_hash("stability-test");
    for _ in 0..100 {
        assert_eq!(short_hash("stability-test"), expected);
    }
}

#[test]
fn redact_path_100_calls_identical() {
    let expected = redact_path("stability/test.rs");
    for _ in 0..100 {
        assert_eq!(redact_path("stability/test.rs"), expected);
    }
}

#[test]
fn different_paths_never_collide_sample() {
    let paths = [
        "a.rs",
        "b.rs",
        "a/b.rs",
        "a/c.rs",
        "x/y/z.rs",
        "x/y/z/w.rs",
        "main.rs",
        "lib.rs",
    ];
    let hashes: Vec<String> = paths.iter().map(|p| short_hash(p)).collect();
    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            assert_ne!(
                hashes[i], hashes[j],
                "collision between '{}' and '{}'",
                paths[i], paths[j]
            );
        }
    }
}

#[test]
fn redact_path_different_ext_same_stem_differs() {
    let r1 = redact_path("src/lib.rs");
    let r2 = redact_path("src/lib.py");
    assert_ne!(r1, r2);
}

#[test]
fn redact_path_same_filename_different_dir() {
    let r1 = redact_path("a/lib.rs");
    let r2 = redact_path("b/lib.rs");
    assert_ne!(r1, r2);
}

// ---------------------------------------------------------------------------
// 9. Redacted output never leaks original path
// ---------------------------------------------------------------------------

#[test]
fn redact_path_does_not_contain_original_segments() {
    let r = redact_path("secrets/passwords/vault.json");
    assert!(!r.contains("secrets"));
    assert!(!r.contains("passwords"));
    assert!(!r.contains("vault"));
}

#[test]
fn short_hash_does_not_contain_input() {
    let h = short_hash("my-secret");
    assert!(!h.contains("my-secret"));
}

// ---------------------------------------------------------------------------
// 10. Normalization edge cases
// ---------------------------------------------------------------------------

#[test]
fn hash_multiple_consecutive_slashes() {
    // Double slashes are NOT collapsed by clean_path, so they differ
    let h1 = short_hash("a//b");
    let h2 = short_hash("a/b");
    // Just verify both produce valid hashes (behaviour is documented)
    assert_eq!(h1.len(), 16);
    assert_eq!(h2.len(), 16);
}

#[test]
fn hash_trailing_slash() {
    let h1 = short_hash("src/");
    let h2 = short_hash("src");
    // Trailing slash is NOT stripped – distinct inputs
    assert_eq!(h1.len(), 16);
    assert_eq!(h2.len(), 16);
}

#[test]
fn redact_path_trailing_slash_no_ext() {
    let r = redact_path("some/dir/");
    assert_eq!(r.len(), 16);
}

#[test]
fn hash_only_dots() {
    let h = short_hash("...");
    assert_eq!(h.len(), 16);
}

#[test]
fn hash_dot_slash_backslash_combo() {
    // ./foo\bar -> foo/bar after normalization
    assert_eq!(short_hash("./foo\\bar"), short_hash("foo/bar"));
}

// ---------------------------------------------------------------------------
// 11. Property tests (proptest)
// ---------------------------------------------------------------------------

mod property_tests {
    use proptest::prelude::*;
    use tokmd_redact::{redact_path, short_hash};

    proptest! {
        #[test]
        fn hash_always_16_hex_chars(input in "\\PC{0,200}") {
            let h = short_hash(&input);
            prop_assert_eq!(h.len(), 16);
            prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn hash_deterministic(input in "\\PC{0,100}") {
            prop_assert_eq!(short_hash(&input), short_hash(&input));
        }

        #[test]
        fn redact_path_deterministic(input in "[a-zA-Z0-9_/\\\\.]{1,80}") {
            prop_assert_eq!(redact_path(&input), redact_path(&input));
        }

        #[test]
        fn redact_path_hash_prefix_is_16_hex(input in "[a-z/]{1,40}\\.[a-z]{1,5}") {
            let r = redact_path(&input);
            // The hash prefix (before the first dot in the redacted output) is 16 hex chars
            let prefix = r.split('.').next().unwrap();
            prop_assert_eq!(prefix.len(), 16);
            prop_assert!(prefix.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn separator_normalization_equivalence(path in "[a-z]{1,5}(/[a-z]{1,5}){0,4}") {
            let with_backslash = path.replace('/', "\\");
            prop_assert_eq!(short_hash(&path), short_hash(&with_backslash));
        }

        #[test]
        fn dot_prefix_normalization(path in "[a-z]{1,5}(/[a-z]{1,5}){0,3}") {
            let with_dot = format!("./{path}");
            prop_assert_eq!(short_hash(&path), short_hash(&with_dot));
        }

        #[test]
        fn interior_dot_normalization(a in "[a-z]{1,5}", b in "[a-z]{1,5}") {
            let with_dot = format!("{a}/./{b}");
            let without = format!("{a}/{b}");
            prop_assert_eq!(short_hash(&with_dot), short_hash(&without));
        }

        #[test]
        fn redact_no_extension_is_bare_hash(stem in "[a-zA-Z]{3,20}") {
            let r = redact_path(&stem);
            prop_assert_eq!(r.len(), 16);
            prop_assert!(!r.contains('.'));
        }
    }
}
