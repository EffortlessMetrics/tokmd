use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use super::{PathViolation, ValidatedRoot};

#[derive(Debug, Clone)]
pub(crate) struct BoundedPath {
    relative: PathBuf,
    canonical: PathBuf,
}

impl BoundedPath {
    pub(crate) fn existing_relative(
        root: &ValidatedRoot,
        relative: &Path,
    ) -> Result<Self, PathViolation> {
        let normalized = normalize_bounded_relative_path(relative)?;
        let candidate = root.canonical().join(&normalized);
        let canonical = canonicalize_existing(&candidate)?;
        ensure_under_root(root, &canonical)?;

        Ok(Self {
            relative: normalized,
            canonical,
        })
    }

    pub(crate) fn existing_child(
        root: &ValidatedRoot,
        child: &Path,
    ) -> Result<Self, PathViolation> {
        let canonical = canonicalize_existing(child)?;
        ensure_under_root(root, &canonical)?;
        let relative = canonical
            .strip_prefix(root.canonical())
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| child.file_name().map(PathBuf::from).unwrap_or_default());

        if relative.as_os_str().is_empty() {
            return Err(PathViolation::Empty);
        }

        Ok(Self {
            relative,
            canonical,
        })
    }

    pub(crate) fn relative(&self) -> &Path {
        &self.relative
    }

    pub(crate) fn canonical(&self) -> &Path {
        &self.canonical
    }
}

pub(crate) fn normalize_bounded_relative_path(path: &Path) -> Result<PathBuf, PathViolation> {
    if path.as_os_str().is_empty() {
        return Err(PathViolation::Empty);
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            Component::ParentDir => return Err(PathViolation::ParentTraversal(path.to_path_buf())),
            Component::RootDir | Component::Prefix(_) => {
                return Err(PathViolation::Absolute(path.to_path_buf()));
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(PathViolation::Empty);
    }

    Ok(normalized)
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, PathViolation> {
    match fs::canonicalize(path) {
        Ok(canonical) => Ok(canonical),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            match fs::symlink_metadata(path) {
                Err(meta_err) if meta_err.kind() == io::ErrorKind::NotFound => {
                    Err(PathViolation::Missing(path.to_path_buf()))
                }
                Ok(_) => Err(PathViolation::CanonicalizeFailed {
                    path: path.to_path_buf(),
                    source,
                }),
                Err(source) => Err(PathViolation::CanonicalizeFailed {
                    path: path.to_path_buf(),
                    source,
                }),
            }
        }
        Err(source) => Err(PathViolation::CanonicalizeFailed {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn ensure_under_root(root: &ValidatedRoot, canonical: &Path) -> Result<(), PathViolation> {
    if canonical.starts_with(root.canonical()) {
        Ok(())
    } else {
        Err(PathViolation::RootEscape {
            root: root.canonical().to_path_buf(),
            path: canonical.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn root_with_files() -> (TempDir, ValidatedRoot) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn lib() {}\n").unwrap();
        std::fs::write(dir.path().join("README.md"), "# hi\n").unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    // ---------- normalize_bounded_relative_path ----------

    #[test]
    fn normalize_rejects_empty_path() {
        let err = normalize_bounded_relative_path(Path::new("")).unwrap_err();
        assert!(matches!(err, PathViolation::Empty));
    }

    #[test]
    fn normalize_rejects_only_current_dir_segments() {
        // "./" alone normalizes to empty — must be rejected as Empty.
        let err = normalize_bounded_relative_path(Path::new("./")).unwrap_err();
        assert!(matches!(err, PathViolation::Empty));

        let err = normalize_bounded_relative_path(Path::new("./.")).unwrap_err();
        assert!(matches!(err, PathViolation::Empty));
    }

    #[test]
    fn normalize_rejects_absolute_unix_style() {
        let err = normalize_bounded_relative_path(Path::new("/etc/passwd")).unwrap_err();
        match err {
            PathViolation::Absolute(p) => assert_eq!(p, PathBuf::from("/etc/passwd")),
            other => panic!("expected Absolute, got {other:?}"),
        }
    }

    #[test]
    fn normalize_rejects_parent_at_any_position() {
        for input in ["..", "../a", "a/..", "a/../b"] {
            let err = normalize_bounded_relative_path(Path::new(input)).unwrap_err();
            assert!(
                matches!(err, PathViolation::ParentTraversal(_)),
                "input {input:?} produced {err:?}"
            );
        }
    }

    #[test]
    fn normalize_strips_curdir_and_preserves_segments() {
        let out = normalize_bounded_relative_path(Path::new("./a/./b/./c.rs")).unwrap();
        assert_eq!(out, PathBuf::from("a/b/c.rs"));
    }

    // ---------- BoundedPath::existing_relative ----------

    #[test]
    fn existing_relative_canonical_lives_under_root() {
        let (_dir, root) = root_with_files();
        let bp = BoundedPath::existing_relative(&root, Path::new("src/lib.rs")).unwrap();

        assert_eq!(bp.relative(), Path::new("src/lib.rs"));
        assert!(bp.canonical().starts_with(root.canonical()));
        // canonical resolves to the real file
        assert!(bp.canonical().is_file());
    }

    #[test]
    fn existing_relative_rejects_empty_relative() {
        let (_dir, root) = root_with_files();
        let err = BoundedPath::existing_relative(&root, Path::new("")).unwrap_err();
        assert!(matches!(err, PathViolation::Empty));
    }

    #[test]
    fn existing_relative_rejects_absolute_relative() {
        let (_dir, root) = root_with_files();
        let err = BoundedPath::existing_relative(&root, Path::new("/etc/hosts")).unwrap_err();
        assert!(matches!(err, PathViolation::Absolute(_)));
    }

    #[test]
    fn existing_relative_rejects_parent_traversal() {
        let (_dir, root) = root_with_files();
        let err = BoundedPath::existing_relative(&root, Path::new("../oops.rs")).unwrap_err();
        assert!(matches!(err, PathViolation::ParentTraversal(_)));
    }

    #[test]
    fn existing_relative_missing_distinguished_from_canonicalize_failed() {
        let (_dir, root) = root_with_files();
        let err = BoundedPath::existing_relative(&root, Path::new("ghost.rs")).unwrap_err();
        assert!(matches!(err, PathViolation::Missing(_)));
        assert!(err.to_string().contains("Bounded path not found"));
    }

    // ---------- BoundedPath::existing_child ----------

    #[test]
    fn existing_child_strips_root_prefix() {
        let (dir, root) = root_with_files();
        let inside = dir.path().join("src").join("lib.rs");

        let bp = BoundedPath::existing_child(&root, &inside).unwrap();

        // The relative component is below the root; it must not be absolute and
        // must not start at the root.
        assert!(!bp.relative().is_absolute());
        // After canonical strip_prefix the relative path is "src/lib.rs"
        // (file separator may be platform-native — verify via PathBuf equality).
        assert_eq!(bp.relative(), Path::new("src/lib.rs"));
        assert!(bp.canonical().starts_with(root.canonical()));
    }

    #[test]
    fn existing_child_rejects_outside_root() {
        let (_dir, root) = root_with_files();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("elsewhere.rs");
        std::fs::write(&outside_file, "").unwrap();

        let err = BoundedPath::existing_child(&root, &outside_file).unwrap_err();
        match err {
            PathViolation::RootEscape { .. } => {}
            other => panic!("expected RootEscape, got {other:?}"),
        }
    }

    #[test]
    fn existing_child_rejects_missing_path() {
        let (dir, root) = root_with_files();
        let missing = dir.path().join("nope.rs");

        let err = BoundedPath::existing_child(&root, &missing).unwrap_err();
        assert!(matches!(err, PathViolation::Missing(_)));
    }

    #[test]
    fn existing_child_rejects_root_itself_as_empty_relative() {
        // Passing the root as the child has no remaining relative component;
        // the constructor surfaces this as Empty.
        let (dir, root) = root_with_files();

        let err = BoundedPath::existing_child(&root, dir.path()).unwrap_err();
        assert!(
            matches!(err, PathViolation::Empty),
            "expected Empty, got {err:?}"
        );
    }

    // ---------- normalization invariant ----------
    //
    // The repo invariant (`agents/shared/repo.md`) says output paths should be
    // forward-slash normalized. `BoundedPath::relative()` returns native separators
    // (it's a PathBuf), so the invariant is enforced at the *string* layer via
    // `normalize_slashes`. We verify the round-trip here so a refactor that
    // ever returns absolute or otherwise-malformed relatives is caught.
    #[test]
    fn relative_path_round_trips_through_slash_normalizer() {
        let (_dir, root) = root_with_files();
        let bp = BoundedPath::existing_relative(&root, Path::new("./src/lib.rs")).unwrap();

        let as_str = bp.relative().to_string_lossy();
        let normalized = super::super::normalize_slashes(&as_str);
        assert_eq!(normalized, "src/lib.rs");
        assert!(!normalized.contains('\\'));
    }
}
