//! Expanded BDD-style tests for tokmd-redact.
//!
//! Focuses on: original-path-component leakage, idempotency of re-redaction,
//! prefix/salt sensitivity, and additional empty/boundary scenarios.

use tokmd_redact::{redact_path, short_hash};

// ============================================================================
// Scenario: Redacted output never leaks original path components
// ============================================================================

mod redacted_output_hides_path_structure {
    use super::*;

    #[test]
    fn given_deep_path_then_redacted_contains_no_directory_names() {
        let redacted = redact_path("secrets/credentials/api_keys/prod.json");
        assert!(!redacted.contains("secrets"));
        assert!(!redacted.contains("credentials"));
        assert!(!redacted.contains("api_keys"));
        assert!(!redacted.contains("prod"));
        // Only the extension should survive
        assert!(redacted.ends_with(".json"));
    }

    #[test]
    fn given_windows_absolute_path_then_drive_letter_and_dirs_are_hidden() {
        let redacted = redact_path("C:\\Users\\admin\\Documents\\secret.txt");
        assert!(!redacted.contains("Users"));
        assert!(!redacted.contains("admin"));
        assert!(!redacted.contains("Documents"));
        assert!(!redacted.contains("secret"));
        assert!(!redacted.contains("C:"));
    }

    #[test]
    fn given_path_with_username_then_username_is_hidden() {
        let redacted = redact_path("home/johndoe/.ssh/id_rsa.pub");
        assert!(!redacted.contains("johndoe"));
        assert!(!redacted.contains("home"));
        assert!(!redacted.contains(".ssh"));
        assert!(!redacted.contains("id_rsa"));
    }

    #[test]
    fn given_path_with_basename_matching_hash_chars_then_still_hidden() {
        // Even if the original basename happens to be hex-like
        let redacted = redact_path("src/deadbeef.rs");
        // The hash portion should NOT be "deadbeef" padded
        assert_ne!(&redacted[..8], "deadbeef");
    }

    #[test]
    fn given_path_with_slashes_then_no_separators_in_output() {
        let redacted = redact_path("a/b/c/d/e/f/g.py");
        assert!(!redacted.contains('/'));
        assert!(!redacted.contains('\\'));
    }
}

// ============================================================================
// Scenario: Re-redacting already-redacted output is stable
// ============================================================================

mod redaction_re_application {
    use super::*;

    #[test]
    fn given_redacted_hash_when_hashed_again_then_produces_valid_output() {
        let first = short_hash("sensitive/path/config.yaml");
        let second = short_hash(&first);
        assert_eq!(second.len(), 16);
        assert!(second.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn given_redacted_path_when_redacted_again_then_produces_valid_extension() {
        let first = redact_path("secrets/token.json");
        // first is like "abc123def456abcd.json"
        let second = redact_path(&first);
        // Re-redacting should still preserve the .json extension
        assert!(second.ends_with(".json"));
        assert_eq!(second.len(), 16 + 1 + 4);
    }

    #[test]
    fn given_extensionless_redacted_output_when_redacted_again_then_no_extension() {
        let first = redact_path("Makefile");
        assert_eq!(first.len(), 16);
        // Re-redacting a 16-char hex string (no dots) should still give 16 chars
        let second = redact_path(&first);
        assert_eq!(second.len(), 16);
        assert!(!second.contains('.'));
    }

    #[test]
    fn given_short_hash_when_called_repeatedly_then_each_output_is_valid() {
        let mut current = "initial/input/path".to_string();
        for _ in 0..10 {
            current = short_hash(&current);
            assert_eq!(current.len(), 16);
            assert!(current.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }
}

// ============================================================================
// Scenario: Prefix sensitivity (salt-like behavior)
// ============================================================================

mod prefix_sensitivity {
    use super::*;

    #[test]
    fn given_same_filename_different_directories_then_hashes_differ() {
        let r1 = redact_path("project_a/src/main.rs");
        let r2 = redact_path("project_b/src/main.rs");
        assert_ne!(r1, r2);
        // Both should still have the same extension
        assert!(r1.ends_with(".rs"));
        assert!(r2.ends_with(".rs"));
    }

    #[test]
    fn given_same_basename_at_different_depths_then_hashes_differ() {
        let shallow = short_hash("lib.rs");
        let deep = short_hash("crates/tokmd/src/lib.rs");
        assert_ne!(shallow, deep);
    }

    #[test]
    fn given_prepended_prefix_then_hash_changes() {
        let bare = short_hash("config.toml");
        let prefixed = short_hash("salt/config.toml");
        assert_ne!(bare, prefixed);
    }

    #[test]
    fn given_appended_suffix_then_hash_changes() {
        let bare = short_hash("src/main");
        let suffixed = short_hash("src/main/extra");
        assert_ne!(bare, suffixed);
    }

    #[test]
    fn given_two_near_identical_paths_differing_by_one_char_then_hashes_differ() {
        let h1 = short_hash("src/module_a.rs");
        let h2 = short_hash("src/module_b.rs");
        assert_ne!(h1, h2);
    }
}

// ============================================================================
// Scenario: Empty and minimal input handling
// ============================================================================

mod empty_and_minimal_inputs {
    use super::*;

    #[test]
    fn given_empty_string_then_short_hash_is_valid() {
        let h = short_hash("");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn given_empty_string_then_redact_path_produces_hash_only() {
        let r = redact_path("");
        assert_eq!(r.len(), 16);
        assert!(!r.contains('.'));
    }

    #[test]
    fn given_single_dot_then_produces_valid_hash() {
        let h = short_hash(".");
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn given_only_separators_then_produces_valid_hash() {
        let h1 = short_hash("///");
        let h2 = short_hash("\\\\\\");
        assert_eq!(h1.len(), 16);
        assert_eq!(h2.len(), 16);
        // These should be equal after normalization
        assert_eq!(h1, h2);
    }

    #[test]
    fn given_single_character_extension_then_preserved() {
        let r = redact_path("archive.z");
        assert!(r.ends_with(".z"));
        assert_eq!(r.len(), 16 + 1 + 1); // hash + dot + "z"
    }

    #[test]
    fn given_very_long_extension_then_preserved() {
        let r = redact_path("document.markdown");
        assert!(r.ends_with(".markdown"));
        assert_eq!(r.len(), 16 + 1 + 8); // hash + dot + "markdown"
    }
}
