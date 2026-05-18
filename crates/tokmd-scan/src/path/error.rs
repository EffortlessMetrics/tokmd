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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    fn io_err() -> io::Error {
        io::Error::new(io::ErrorKind::PermissionDenied, "denied")
    }

    // ---------- RootViolation ----------

    #[test]
    fn root_violation_empty_display_is_stable() {
        let msg = RootViolation::Empty.to_string();
        assert_eq!(msg, "Scan root must not be empty");
    }

    #[test]
    fn root_violation_missing_display_includes_path() {
        let msg = RootViolation::Missing(PathBuf::from("/no/such/dir")).to_string();
        assert!(
            msg.starts_with("Path not found: "),
            "unexpected prefix: {msg}"
        );
        assert!(msg.contains("no") && msg.contains("such") && msg.contains("dir"));
    }

    #[test]
    fn root_violation_canonicalize_failed_display_includes_path_and_source() {
        let msg = RootViolation::CanonicalizeFailed {
            path: PathBuf::from("/tmp/x"),
            source: io_err(),
        }
        .to_string();
        assert!(msg.contains("Failed to resolve scan root"));
        assert!(msg.contains("tmp") && msg.contains("x"));
        assert!(msg.contains("denied"));
    }

    #[test]
    fn root_violation_source_exposed_only_for_canonicalize_failed() {
        let empty = RootViolation::Empty;
        let missing = RootViolation::Missing(PathBuf::from("/x"));
        let canon = RootViolation::CanonicalizeFailed {
            path: PathBuf::from("/x"),
            source: io_err(),
        };
        assert!(empty.source().is_none());
        assert!(missing.source().is_none());
        assert!(canon.source().is_some());
    }

    #[test]
    fn root_violation_debug_shape_is_stable() {
        // Spot-check the Debug derivation produces variant names. We deliberately
        // assert only a substring so field ordering or formatter tweaks don't
        // make the test brittle.
        let debug = format!("{:?}", RootViolation::Empty);
        assert!(debug.contains("Empty"), "unexpected debug: {debug}");

        let debug = format!("{:?}", RootViolation::Missing(PathBuf::from("/m")));
        assert!(debug.contains("Missing"), "unexpected debug: {debug}");

        let debug = format!(
            "{:?}",
            RootViolation::CanonicalizeFailed {
                path: PathBuf::from("/x"),
                source: io_err(),
            }
        );
        assert!(
            debug.contains("CanonicalizeFailed"),
            "unexpected debug: {debug}"
        );
    }

    // ---------- PathViolation ----------

    #[test]
    fn path_violation_empty_display_is_stable() {
        assert_eq!(
            PathViolation::Empty.to_string(),
            "Bounded path must not be empty"
        );
    }

    #[test]
    fn path_violation_absolute_display_includes_path() {
        let msg = PathViolation::Absolute(PathBuf::from("/abs/file.rs")).to_string();
        assert!(msg.contains("must be relative"));
        assert!(msg.contains("abs") && msg.contains("file.rs"));
    }

    #[test]
    fn path_violation_parent_traversal_display_includes_path() {
        let msg = PathViolation::ParentTraversal(PathBuf::from("../escape.rs")).to_string();
        assert!(msg.contains("parent traversal"));
        assert!(msg.contains("escape.rs"));
    }

    #[test]
    fn path_violation_missing_display_includes_path() {
        let msg = PathViolation::Missing(PathBuf::from("nope.rs")).to_string();
        assert!(msg.starts_with("Bounded path not found: "));
        assert!(msg.contains("nope.rs"));
    }

    #[test]
    fn path_violation_root_escape_display_includes_root_and_path() {
        let msg = PathViolation::RootEscape {
            root: PathBuf::from("/r"),
            path: PathBuf::from("/elsewhere/secret.txt"),
        }
        .to_string();
        assert!(msg.contains("escapes scan root"));
        // Root and path both surface in the rendered message
        assert!(msg.contains("/r") || msg.contains("\\r"));
        assert!(msg.contains("secret.txt"));
    }

    #[test]
    fn path_violation_canonicalize_failed_display_includes_path_and_source() {
        let msg = PathViolation::CanonicalizeFailed {
            path: PathBuf::from("/tmp/y"),
            source: io_err(),
        }
        .to_string();
        assert!(msg.contains("Failed to resolve bounded path"));
        assert!(msg.contains("tmp") && msg.contains("y"));
        assert!(msg.contains("denied"));
    }

    #[test]
    fn path_violation_source_exposed_only_for_canonicalize_failed() {
        let cases: [PathViolation; 5] = [
            PathViolation::Empty,
            PathViolation::Absolute(PathBuf::from("/a")),
            PathViolation::ParentTraversal(PathBuf::from("..")),
            PathViolation::Missing(PathBuf::from("m")),
            PathViolation::RootEscape {
                root: PathBuf::from("/r"),
                path: PathBuf::from("/x"),
            },
        ];
        for v in &cases {
            assert!(v.source().is_none(), "expected None for {v:?}");
        }

        let canon = PathViolation::CanonicalizeFailed {
            path: PathBuf::from("/x"),
            source: io_err(),
        };
        assert!(canon.source().is_some());
    }

    #[test]
    fn path_violation_debug_shape_is_stable() {
        // Same intent as root_violation_debug_shape_is_stable: only assert the
        // variant name appears in the debug output.
        for (variant, expected) in [
            (PathViolation::Empty, "Empty"),
            (PathViolation::Absolute(PathBuf::from("/a")), "Absolute"),
            (
                PathViolation::ParentTraversal(PathBuf::from("..")),
                "ParentTraversal",
            ),
            (PathViolation::Missing(PathBuf::from("m")), "Missing"),
            (
                PathViolation::RootEscape {
                    root: PathBuf::from("/r"),
                    path: PathBuf::from("/x"),
                },
                "RootEscape",
            ),
            (
                PathViolation::CanonicalizeFailed {
                    path: PathBuf::from("/x"),
                    source: io_err(),
                },
                "CanonicalizeFailed",
            ),
        ] {
            let debug = format!("{variant:?}");
            assert!(
                debug.contains(expected),
                "expected debug to contain {expected:?}, got {debug:?}"
            );
        }
    }
}
