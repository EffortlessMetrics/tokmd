use std::fs;
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
    fs::canonicalize(path).map_err(|source| {
        if !path.exists() {
            PathViolation::Missing(path.to_path_buf())
        } else {
            PathViolation::CanonicalizeFailed {
                path: path.to_path_buf(),
                source,
            }
        }
    })
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
