//! BDD-style scenario tests for path normalization.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── normalize_slashes ────────────────────────────────────────────

mod normalize_slashes_scenarios {
    use super::*;

    #[test]
    fn given_windows_backslash_path_then_forward_slashes() {
        assert_eq!(normalize_slashes(r"foo\bar\baz.rs"), "foo/bar/baz.rs");
    }

    #[test]
    fn given_already_forward_slashes_then_unchanged() {
        assert_eq!(normalize_slashes("foo/bar/baz.rs"), "foo/bar/baz.rs");
    }

    #[test]
    fn given_mixed_separators_then_all_forward() {
        assert_eq!(normalize_slashes(r"foo/bar\baz.rs"), "foo/bar/baz.rs");
    }

    #[test]
    fn given_empty_path_then_empty_string() {
        assert_eq!(normalize_slashes(""), "");
    }

    #[test]
    fn given_single_backslash_then_single_forward_slash() {
        assert_eq!(normalize_slashes("\\"), "/");
    }

    #[test]
    fn given_root_forward_slash_then_unchanged() {
        assert_eq!(normalize_slashes("/"), "/");
    }

    #[test]
    fn given_unc_path_then_forward_slashes() {
        assert_eq!(
            normalize_slashes(r"\\server\share\file.txt"),
            "//server/share/file.txt"
        );
    }

    #[test]
    fn given_windows_drive_path_then_forward_slashes() {
        assert_eq!(
            normalize_slashes(r"C:\Users\me\file.rs"),
            "C:/Users/me/file.rs"
        );
    }

    #[test]
    fn given_path_with_spaces_then_preserves_spaces() {
        assert_eq!(
            normalize_slashes(r"foo\bar baz\qux.rs"),
            "foo/bar baz/qux.rs"
        );
    }

    #[test]
    fn given_unicode_path_then_preserves_unicode() {
        assert_eq!(
            normalize_slashes(r"données\résumé\日本語.rs"),
            "données/résumé/日本語.rs"
        );
    }

    #[test]
    fn given_consecutive_backslashes_then_consecutive_forward_slashes() {
        assert_eq!(normalize_slashes("a\\\\b"), "a//b");
    }

    #[test]
    fn given_dot_segments_then_preserves_them() {
        assert_eq!(normalize_slashes(r".\foo\..\bar"), "./foo/../bar");
    }

    #[test]
    fn given_trailing_backslash_then_trailing_forward_slash() {
        assert_eq!(normalize_slashes(r"foo\bar\"), "foo/bar/");
    }

    #[test]
    fn given_filename_only_then_unchanged() {
        assert_eq!(normalize_slashes("file.rs"), "file.rs");
    }
}

// ── normalize_rel_path ───────────────────────────────────────────

mod normalize_rel_path_scenarios {
    use super::*;

    #[test]
    fn given_dot_slash_prefix_then_stripped() {
        assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
    }

    #[test]
    fn given_dot_backslash_prefix_then_stripped_and_normalized() {
        assert_eq!(normalize_rel_path(r".\src\main.rs"), "src/main.rs");
    }

    #[test]
    fn given_no_relative_prefix_then_unchanged_except_slashes() {
        assert_eq!(normalize_rel_path(r"src\main.rs"), "src/main.rs");
    }

    #[test]
    fn given_double_dot_prefix_then_preserved() {
        assert_eq!(normalize_rel_path("../src/main.rs"), "../src/main.rs");
    }

    #[test]
    fn given_double_dot_backslash_prefix_then_normalized_but_preserved() {
        assert_eq!(normalize_rel_path(r"..\src\main.rs"), "../src/main.rs");
    }

    #[test]
    fn given_bare_dot_slash_then_empty_string() {
        assert_eq!(normalize_rel_path("./"), "");
    }

    #[test]
    fn given_bare_dot_backslash_then_empty_string() {
        // `.\` → normalize_slashes → `./` → strip `./` → ``
        assert_eq!(normalize_rel_path(r".\"), "");
    }

    #[test]
    fn given_empty_path_then_empty_string() {
        assert_eq!(normalize_rel_path(""), "");
    }

    #[test]
    fn given_absolute_path_then_slashes_normalized() {
        assert_eq!(normalize_rel_path(r"C:\src\main.rs"), "C:/src/main.rs");
    }

    #[test]
    fn given_path_with_spaces_then_preserves_spaces() {
        assert_eq!(
            normalize_rel_path(r".\my project\src\main.rs"),
            "my project/src/main.rs"
        );
    }

    #[test]
    fn given_unicode_relative_path_then_normalized() {
        assert_eq!(
            normalize_rel_path(r".\données\résumé.rs"),
            "données/résumé.rs"
        );
    }

    #[test]
    fn given_only_dot_slash_prefix_stripped_once() {
        // `././foo` → normalize_slashes is noop → strip `./` → `./foo`
        // Only ONE leading `./` is stripped per the doc contract.
        assert_eq!(normalize_rel_path("././foo"), "./foo");
    }

    #[test]
    fn given_dot_no_slash_then_preserved() {
        assert_eq!(normalize_rel_path(".hidden"), ".hidden");
    }

    #[test]
    fn given_root_slash_then_unchanged() {
        assert_eq!(normalize_rel_path("/"), "/");
    }
}
