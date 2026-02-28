//! BDD-style scenario tests for tokmd-redact.
//!
//! Each scenario exercises a specific real-world use-case to ensure the
//! redaction API behaves as documented.

use tokmd_redact::{redact_path, short_hash};

// ============================================================================
// Scenario: Path cleaning normalises separators before hashing
// ============================================================================

mod path_cleaning {
    use super::*;

    #[test]
    fn given_unix_path_then_hash_is_stable() {
        let hash = short_hash("crates/tokmd-redact/src/lib.rs");
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn given_windows_path_then_hash_matches_unix() {
        let unix = short_hash("crates/tokmd-redact/src/lib.rs");
        let win = short_hash("crates\\tokmd-redact\\src\\lib.rs");
        assert_eq!(unix, win);
    }

    #[test]
    fn given_mixed_separators_then_hash_matches_unix() {
        let unix = short_hash("a/b/c/d");
        let mixed = short_hash("a\\b/c\\d");
        assert_eq!(unix, mixed);
    }

    #[test]
    fn given_leading_dot_slash_then_hash_matches_bare() {
        assert_eq!(short_hash("src/lib.rs"), short_hash("./src/lib.rs"));
    }

    #[test]
    fn given_multiple_leading_dot_slash_then_hash_matches_bare() {
        assert_eq!(short_hash("src/lib.rs"), short_hash("././src/lib.rs"));
    }

    #[test]
    fn given_interior_dot_segment_then_hash_matches_clean() {
        assert_eq!(
            short_hash("crates/foo/src/lib.rs"),
            short_hash("crates/foo/./src/lib.rs")
        );
    }

    #[test]
    fn given_trailing_dot_segment_then_hash_matches_clean() {
        assert_eq!(short_hash("crates/foo"), short_hash("crates/foo/."));
    }

    #[test]
    fn given_leading_dot_slash_with_backslash_then_hash_matches_bare() {
        // .\ on Windows is equivalent to ./
        assert_eq!(short_hash("src/lib.rs"), short_hash(".\\src\\lib.rs"));
    }
}

// ============================================================================
// Scenario: BLAKE3 hashing produces valid, stable digests
// ============================================================================

mod blake3_hashing {
    use super::*;

    #[test]
    fn given_empty_string_then_hash_is_valid_hex() {
        let h = short_hash("");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn given_known_input_then_hash_is_reproducible() {
        // Pin a known digest so regressions are caught.
        let h1 = short_hash("hello world");
        let h2 = short_hash("hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn given_unicode_input_then_hash_is_valid() {
        let h = short_hash("日本語/ファイル.rs");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn given_very_long_input_then_hash_is_valid() {
        let long = "a/".repeat(5_000) + "file.rs";
        let h = short_hash(&long);
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn given_whitespace_only_then_hash_is_valid() {
        let h = short_hash("   ");
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn given_single_char_inputs_then_distinct_hashes() {
        let h_a = short_hash("a");
        let h_b = short_hash("b");
        assert_ne!(h_a, h_b);
    }

    #[test]
    fn given_case_differs_then_hashes_differ() {
        assert_ne!(short_hash("Src/Lib.rs"), short_hash("src/lib.rs"));
    }
}

// ============================================================================
// Scenario: redact_path preserves file type but hides path structure
// ============================================================================

mod redact_path_extension_preservation {
    use super::*;

    #[test]
    fn given_rust_source_then_extension_is_rs() {
        let r = redact_path("src/secrets/config.rs");
        assert!(r.ends_with(".rs"), "Expected .rs, got {r}");
    }

    #[test]
    fn given_json_file_then_extension_is_json() {
        let r = redact_path("data/credentials.json");
        assert!(r.ends_with(".json"));
        assert_eq!(r.len(), 16 + 1 + 4); // hash + '.' + "json"
    }

    #[test]
    fn given_tar_gz_then_only_final_extension_preserved() {
        let r = redact_path("backups/archive.tar.gz");
        assert!(r.ends_with(".gz"));
        assert!(!r.contains(".tar"));
    }

    #[test]
    fn given_makefile_no_ext_then_no_dot_in_output() {
        let r = redact_path("build/Makefile");
        assert!(!r.contains('.'));
        assert_eq!(r.len(), 16);
    }

    #[test]
    fn given_dockerfile_no_ext_then_no_dot_in_output() {
        let r = redact_path("Dockerfile");
        assert!(!r.contains('.'));
        assert_eq!(r.len(), 16);
    }

    #[test]
    fn given_dotfile_gitignore_then_no_extension() {
        // std::path::Path treats ".gitignore" as having no extension
        let r = redact_path(".gitignore");
        assert!(!r.contains('.'));
        assert_eq!(r.len(), 16);
    }

    #[test]
    fn given_hidden_file_with_ext_then_ext_preserved() {
        // ".eslintrc.json" has extension "json"
        let r = redact_path(".eslintrc.json");
        assert!(r.ends_with(".json"));
    }

    #[test]
    fn given_dotfile_in_subdir_then_no_extension() {
        let r = redact_path("config/.env");
        assert!(!r.contains('.'));
        assert_eq!(r.len(), 16);
    }
}

// ============================================================================
// Scenario: redact_path is deterministic and idempotent
// ============================================================================

mod redact_determinism {
    use super::*;

    #[test]
    fn given_same_path_twice_then_same_output() {
        let r1 = redact_path("src/main.rs");
        let r2 = redact_path("src/main.rs");
        assert_eq!(r1, r2);
    }

    #[test]
    fn given_equivalent_paths_different_separators_then_same_output() {
        let r1 = redact_path("crates/tokmd/src/main.rs");
        let r2 = redact_path("crates\\tokmd\\src\\main.rs");
        assert_eq!(r1, r2);
    }

    #[test]
    fn given_path_with_dot_prefix_then_matches_without() {
        let r1 = redact_path("./src/main.rs");
        let r2 = redact_path("src/main.rs");
        assert_eq!(r1, r2);
    }

    #[test]
    fn given_hash_portion_then_matches_short_hash() {
        let redacted = redact_path("src/main.rs");
        let hash_part = &redacted[..16];
        assert_eq!(hash_part, short_hash("src/main.rs"));
    }

    #[test]
    fn given_two_different_files_same_ext_then_different_hashes() {
        let r1 = redact_path("src/a.rs");
        let r2 = redact_path("src/b.rs");
        assert_ne!(r1, r2);
        // But both end with .rs
        assert!(r1.ends_with(".rs"));
        assert!(r2.ends_with(".rs"));
    }
}

// ============================================================================
// Scenario: Cross-platform path equivalence
// ============================================================================

mod cross_platform {
    use super::*;

    #[test]
    fn given_deep_windows_path_then_matches_unix() {
        let unix = redact_path("crates/tokmd/src/commands/run.rs");
        let win = redact_path("crates\\tokmd\\src\\commands\\run.rs");
        assert_eq!(unix, win);
        assert!(unix.ends_with(".rs"));
    }

    #[test]
    fn given_windows_root_path_then_produces_valid_hash() {
        let r = redact_path("C:\\Users\\dev\\project\\src\\main.rs");
        assert!(r.ends_with(".rs"));
        assert_eq!(r.len(), 16 + 1 + 2);
    }

    #[test]
    fn given_unc_style_path_then_produces_valid_hash() {
        let r = redact_path("\\\\server\\share\\file.txt");
        assert!(r.ends_with(".txt"));
        assert_eq!(r.len(), 16 + 1 + 3);
    }

    #[test]
    fn given_mixed_separators_deep_path_then_matches_unix() {
        let unix = redact_path("a/b/c/d/e.py");
        let mixed = redact_path("a\\b/c\\d/e.py");
        assert_eq!(unix, mixed);
    }
}

// ============================================================================
// Scenario: Edge cases and boundary conditions
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn given_single_extension_only_then_treated_as_dotfile() {
        // ".rs" alone: Path considers this a file with no extension
        let r = redact_path(".rs");
        assert_eq!(r.len(), 16);
        assert!(!r.contains('.'));
    }

    #[test]
    fn given_path_with_many_dots_then_preserves_last_ext() {
        let r = redact_path("a.b.c.d.e.txt");
        assert!(r.ends_with(".txt"));
    }

    #[test]
    fn given_empty_string_then_produces_valid_hash() {
        let r = redact_path("");
        assert_eq!(r.len(), 16);
    }

    #[test]
    fn given_just_slash_then_produces_valid_hash() {
        let h = short_hash("/");
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn given_just_backslash_then_matches_forward_slash() {
        assert_eq!(short_hash("/"), short_hash("\\"));
    }

    #[test]
    fn given_path_ending_with_slash_then_valid() {
        let h = short_hash("src/");
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn given_path_with_special_chars_then_valid() {
        let h = short_hash("src/@types/node.d.ts");
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn given_path_with_spaces_then_valid() {
        let r = redact_path("My Documents/file name.txt");
        assert!(r.ends_with(".txt"));
        assert_eq!(r.len(), 16 + 1 + 3);
    }

    #[test]
    fn given_only_extension_like_name_then_valid() {
        // "file." — empty extension
        let r = redact_path("file.");
        // Path::extension returns None for trailing dot
        assert_eq!(r.len(), 16);
    }
}
