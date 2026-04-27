use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum RootViolation {
    Empty,
    Missing(PathBuf),
    CanonicalizeFailed { path: PathBuf, source: io::Error },
}

impl fmt::Display for RootViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Scan root must not be empty"),
            Self::Missing(path) => write!(f, "Path not found: {}", path.display()),
            Self::CanonicalizeFailed { path, source } => {
                write!(
                    f,
                    "Failed to resolve scan root {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for RootViolation {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CanonicalizeFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum PathViolation {
    Empty,
    Absolute(PathBuf),
    ParentTraversal(PathBuf),
    Missing(PathBuf),
    RootEscape { root: PathBuf, path: PathBuf },
    CanonicalizeFailed { path: PathBuf, source: io::Error },
}

impl fmt::Display for PathViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Bounded path must not be empty"),
            Self::Absolute(path) => write!(f, "Bounded path must be relative: {}", path.display()),
            Self::ParentTraversal(path) => {
                write!(
                    f,
                    "Bounded path must not contain parent traversal: {}",
                    path.display()
                )
            }
            Self::Missing(path) => write!(f, "Bounded path not found: {}", path.display()),
            Self::RootEscape { root, path } => write!(
                f,
                "Bounded path escapes scan root {}: {}",
                root.display(),
                path.display()
            ),
            Self::CanonicalizeFailed { path, source } => {
                write!(
                    f,
                    "Failed to resolve bounded path {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for PathViolation {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CanonicalizeFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}
