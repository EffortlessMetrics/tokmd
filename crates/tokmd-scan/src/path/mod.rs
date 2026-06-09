//! Single-responsibility path normalization and root bounding.

mod bounded_path;
mod error;
#[cfg(test)]
mod tests;
mod validated_root;

use std::path::{Path, PathBuf};

pub(crate) use bounded_path::{BoundedPath, normalize_bounded_relative_path};
pub(crate) use error::{PathViolation, RootViolation};
pub(crate) use validated_root::ValidatedRoot;

/// Normalize path separators to `/`.
///
/// # Examples
///
/// ```
/// use tokmd_scan::normalize_slashes;
///
/// assert_eq!(normalize_slashes("src\\lib.rs"), "src/lib.rs");
/// assert_eq!(normalize_slashes("already/forward"), "already/forward");
/// ```
///
/// Mixed separators are all converted:
///
/// ```
/// use tokmd_scan::normalize_slashes;
///
/// assert_eq!(normalize_slashes("a\\b/c\\d"), "a/b/c/d");
/// // Already-normalized paths pass through unchanged
/// assert_eq!(normalize_slashes("no/change"), "no/change");
/// ```
#[must_use]
pub fn normalize_slashes(path: &str) -> std::borrow::Cow<'_, str> {
    normalize_slashes_cow(path)
}

pub(crate) fn normalize_slashes_cow(path: &str) -> std::borrow::Cow<'_, str> {
    if path.contains('\\') {
        std::borrow::Cow::Owned(path.replace('\\', "/"))
    } else {
        std::borrow::Cow::Borrowed(path)
    }
}

/// Normalize a relative path for matching:
/// - converts `\` to `/`
/// - strips all leading `./` segments
///
/// This is a formatting normalizer, not a traversal validator. It intentionally
/// preserves parent traversal segments so callers that compare already-bounded
/// receipt paths do not silently rewrite meaning. Use [`normalize_bounded_rel_path`]
/// or [`canonicalize_bounded_path`] before using untrusted relative paths for
/// filesystem access.
///
/// # Examples
///
/// ```
/// use tokmd_scan::normalize_rel_path;
///
/// assert_eq!(normalize_rel_path("./src/main.rs"), "src/main.rs");
/// assert_eq!(normalize_rel_path(".\\src\\main.rs"), "src/main.rs");
/// assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");
/// assert_eq!(normalize_rel_path("././src/lib.rs"), "src/lib.rs");
/// ```
///
/// Idempotency — normalizing twice gives the same result:
///
/// ```
/// use tokmd_scan::normalize_rel_path;
///
/// let once = normalize_rel_path(".\\src\\lib.rs");
/// let twice = normalize_rel_path(&once);
/// assert_eq!(once, twice);
/// assert_eq!(once, "src/lib.rs");
/// ```
#[must_use]
pub fn normalize_rel_path(path: &str) -> std::borrow::Cow<'_, str> {
    let normalized = normalize_slashes_cow(path);
    let mut s = normalized.as_ref();
    let mut stripped = false;
    while let Some(rest) = s.strip_prefix("./") {
        s = rest;
        stripped = true;
    }

    let drain_len = if stripped {
        normalized.len() - s.len()
    } else {
        0
    };

    match normalized {
        std::borrow::Cow::Borrowed(_) if !stripped => std::borrow::Cow::Borrowed(path),
        std::borrow::Cow::Borrowed(_) => std::borrow::Cow::Borrowed(&path[path.len() - s.len()..]),
        std::borrow::Cow::Owned(owned) if !stripped => std::borrow::Cow::Owned(owned),
        std::borrow::Cow::Owned(mut owned) => {
            owned.drain(..drain_len);
            std::borrow::Cow::Owned(owned)
        }
    }
}

/// Normalize a root-relative path and reject traversal or absolute inputs.
///
/// This helper performs only lexical validation. It does not touch the
/// filesystem, so it is suitable for logical paths such as browser/in-memory
/// inputs. Use [`canonicalize_bounded_path`] when the path must exist under a
/// specific root.
///
/// # Errors
///
/// Returns an error when `path` is empty, absolute, resolves to only `.` segments,
/// or contains parent traversal.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use tokmd_scan::normalize_bounded_rel_path;
///
/// assert_eq!(
///     normalize_bounded_rel_path("./src/./lib.rs").unwrap(),
///     PathBuf::from("src/lib.rs")
/// );
/// assert!(normalize_bounded_rel_path("../secret.txt").is_err());
/// ```
pub fn normalize_bounded_rel_path(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    Ok(normalize_bounded_relative_path(path.as_ref())?)
}

/// Canonicalize an existing root-relative path and verify it stays under `root`.
///
/// This is the filesystem boundary check for untrusted relative paths. Both the
/// root and candidate are resolved with `std::fs::canonicalize`, then the resolved
/// candidate must remain below the resolved root. Symlinks that escape the root are
/// rejected.
///
/// # Errors
///
/// Returns an error when `root` is empty/missing/unresolvable, when `relative` is
/// empty/absolute/escaping/missing/unresolvable, or when symlink resolution leaves
/// the root.
pub fn canonicalize_bounded_path(
    root: impl AsRef<Path>,
    relative: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let root = ValidatedRoot::new(root)?;
    let bounded = BoundedPath::existing_relative(&root, relative.as_ref())?;
    Ok(bounded.canonical().to_path_buf())
}

#[cfg(test)]
mod normalization_tests {
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

    #[test]
    fn normalize_bounded_rel_path_rejects_parent_traversal() {
        let err = normalize_bounded_rel_path("../src/main.rs").unwrap_err();

        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn normalize_bounded_rel_path_rejects_absolute_path() {
        let err = normalize_bounded_rel_path("/src/main.rs").unwrap_err();

        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn canonicalize_bounded_path_returns_canonical_child() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        let child = dir.path().join("src/lib.rs");
        std::fs::write(&child, "pub fn lib() {}\n").unwrap();

        let bounded = canonicalize_bounded_path(dir.path(), "./src/lib.rs").unwrap();

        assert_eq!(bounded, std::fs::canonicalize(child).unwrap());
    }

    #[test]
    fn canonicalize_bounded_path_rejects_parent_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let err = canonicalize_bounded_path(dir.path(), "../secret.txt").unwrap_err();

        assert!(err.to_string().contains("parent traversal"));
    }

    #[test]
    fn canonicalize_bounded_path_rejects_symlink_escape_when_supported() {
        let root_dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("secret.txt");
        let link = root_dir.path().join("secret-link.txt");
        std::fs::write(&outside_file, "secret").unwrap();

        if create_file_symlink(&outside_file, &link).is_err() {
            return;
        }

        let err = canonicalize_bounded_path(root_dir.path(), "secret-link.txt").unwrap_err();

        assert!(err.to_string().contains("escapes scan root"));
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
            prop_assert_eq!(once.as_ref(), twice.as_ref());
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
            prop_assert_eq!(once.as_ref(), twice.as_ref());
        }
    }

    #[cfg(unix)]
    fn create_file_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(src, dst)
    }

    #[cfg(windows)]
    fn create_file_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(src, dst)
    }
}
