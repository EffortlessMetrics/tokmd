use std::fs;
use std::path::{Path, PathBuf};

use super::RootViolation;

#[derive(Debug, Clone)]
pub(crate) struct ValidatedRoot {
    input: PathBuf,
    canonical: PathBuf,
}

impl ValidatedRoot {
    pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, RootViolation> {
        let input = path.as_ref().to_path_buf();
        if input.as_os_str().is_empty() {
            return Err(RootViolation::Empty);
        }
        if !input.exists() {
            return Err(RootViolation::Missing(input));
        }

        let canonical =
            fs::canonicalize(&input).map_err(|source| RootViolation::CanonicalizeFailed {
                path: input.clone(),
                source,
            })?;

        Ok(Self { input, canonical })
    }

    pub(crate) fn input(&self) -> &Path {
        &self.input
    }

    pub(crate) fn canonical(&self) -> &Path {
        &self.canonical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_path() {
        let err = ValidatedRoot::new("").unwrap_err();
        assert!(matches!(err, RootViolation::Empty));
        assert_eq!(err.to_string(), "Scan root must not be empty");
    }

    #[test]
    fn rejects_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");

        let err = ValidatedRoot::new(&missing).unwrap_err();

        match err {
            RootViolation::Missing(p) => assert_eq!(p, missing),
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    #[test]
    fn accepts_existing_directory_and_preserves_input() {
        let dir = tempfile::tempdir().unwrap();
        let root = ValidatedRoot::new(dir.path()).unwrap();

        assert_eq!(root.input(), dir.path());
        assert!(root.canonical().is_absolute());
        // canonical() always resolves to an existing directory
        assert!(root.canonical().is_dir());
    }

    #[test]
    fn accepts_existing_file_path() {
        // ValidatedRoot does NOT enforce directory-only — callers (e.g. tokei) handle
        // file inputs separately. Document the actual behavior so anyone tightening
        // this constraint must update the test deliberately.
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, "hi").unwrap();

        let root = ValidatedRoot::new(&file).unwrap();
        assert_eq!(root.input(), file);
        assert!(root.canonical().is_absolute());
    }

    #[test]
    fn canonicalizes_through_intermediate_symlink_when_supported() {
        let real_dir = tempfile::tempdir().unwrap();
        let link_parent = tempfile::tempdir().unwrap();
        let link = link_parent.path().join("link-to-real");

        if create_dir_symlink(real_dir.path(), &link).is_err() {
            // Platforms without symlink permission (e.g. Windows w/o developer mode)
            // simply skip — this matches the pattern in path/tests.rs.
            return;
        }

        let root = ValidatedRoot::new(&link).unwrap();
        assert_eq!(root.input(), link);
        // canonical() resolves the symlink to the real directory
        assert_eq!(
            root.canonical(),
            std::fs::canonicalize(real_dir.path()).unwrap()
        );
    }

    #[cfg(unix)]
    fn create_dir_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(src, dst)
    }

    #[cfg(windows)]
    fn create_dir_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(src, dst)
    }
}
