//! Single-responsibility path normalization for deterministic matching.

/// Normalize path separators to `/`.
///
/// # Examples
///
/// ```
/// use tokmd_path::normalize_slashes;
///
/// assert_eq!(normalize_slashes(r"foo\bar\baz.rs"), "foo/bar/baz.rs");
/// assert_eq!(normalize_slashes("already/fine"), "already/fine");
/// ```
#[must_use]
pub fn normalize_slashes(path: &str) -> String {
    if path.contains('\\') {
        path.replace('\\', "/")
    } else {
        path.to_string()
    }
}

/// Normalize a relative path for matching:
/// - converts `\` to `/`
/// - strips one leading `./`
///
/// # Examples
///
/// ```
/// use tokmd_path::normalize_rel_path;
///
/// assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
/// assert_eq!(normalize_rel_path(r".\src\main.rs"), "src/main.rs");
/// assert_eq!(normalize_rel_path("src/lib.rs"), "src/lib.rs");
/// ```
#[must_use]
pub fn normalize_rel_path(path: &str) -> String {
    let normalized = normalize_slashes(path);
    if let Some(stripped) = normalized.strip_prefix("./") {
        stripped.to_string()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn normalize_slashes_replaces_backslash() {
        assert_eq!(normalize_slashes(r"foo\bar\baz.rs"), "foo/bar/baz.rs");
    }

    #[test]
    fn normalize_rel_path_strips_dot_slash() {
        assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
    }

    #[test]
    fn normalize_rel_path_strips_dot_backslash() {
        assert_eq!(normalize_rel_path(r".\src\main.rs"), "src/main.rs");
    }

    #[test]
    fn normalize_rel_path_preserves_non_relative_prefix() {
        assert_eq!(normalize_rel_path("../src/main.rs"), "../src/main.rs");
    }

    proptest! {
        #[test]
        fn normalize_slashes_no_backslashes(path in "\\PC*") {
            let normalized = normalize_slashes(&path);
            prop_assert!(!normalized.contains('\\'));
        }

        #[test]
        fn normalize_slashes_idempotent(path in "\\PC*") {
            let once = normalize_slashes(&path);
            let twice = normalize_slashes(&once);
            prop_assert_eq!(once, twice);
        }

        #[test]
        fn normalize_rel_path_no_backslashes(path in "\\PC*") {
            let normalized = normalize_rel_path(&path);
            prop_assert!(!normalized.contains('\\'));
        }

        #[test]
        fn normalize_rel_path_idempotent(path in "\\PC*") {
            let once = normalize_rel_path(&path);
            let twice = normalize_rel_path(&once);
            prop_assert_eq!(once, twice);
        }
    }
}
