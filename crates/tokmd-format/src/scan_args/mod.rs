//! Deterministic scan argument construction for receipt metadata.

use std::path::{Path, PathBuf};

use crate::redact::{redact_path, short_hash};
use tokmd_settings::ScanOptions;
use tokmd_types::{RedactMode, ScanArgs};

/// Normalize a path to forward slashes and strip leading `./` segments.
#[must_use]
pub fn normalize_scan_input(p: &Path) -> String {
    let mut normalized = normalize_rel_path(&p.display().to_string());

    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }

    if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized
    }
}

/// Normalize a relative path for matching:
/// - converts `\` to `/`
/// - strips all leading `./` segments
#[must_use]
fn normalize_rel_path(path: &str) -> String {
    let normalized = if path.contains('\\') {
        path.replace('\\', "/")
    } else {
        path.to_string()
    };

    let mut normalized_str = normalized.as_str();
    while let Some(rest) = normalized_str.strip_prefix("./") {
        normalized_str = rest;
    }

    // Resolve parent directory segments (..)
    let mut parts: Vec<&str> = Vec::new();
    for part in normalized_str.split('/') {
        if part == ".." {
            let last = parts.last().copied().unwrap_or("");
            if last != ".." && !last.is_empty() {
                parts.pop();
                continue;
            }
        }
        if !part.is_empty() || parts.is_empty() {
            parts.push(part);
        }
    }

    parts.join("/")
}

/// Construct `ScanArgs` with optional path and exclusion redaction.
#[must_use]
pub fn scan_args(paths: &[PathBuf], global: &ScanOptions, redact: Option<RedactMode>) -> ScanArgs {
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
        excluded_redacted,
        config: global.config,
        hidden: global.hidden,
        no_ignore: global.no_ignore,
        no_ignore_parent: global.no_ignore || global.no_ignore_parent,
        no_ignore_dot: global.no_ignore || global.no_ignore_dot,
        no_ignore_vcs: global.no_ignore || global.no_ignore_vcs,
        treat_doc_strings_as_comments: global.treat_doc_strings_as_comments,
    };

    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn normalize_scan_input_parent_directory() {
        assert_eq!(normalize_scan_input(Path::new("foo/../bar")), "bar");
        assert_eq!(normalize_scan_input(Path::new("foo/../bar/baz")), "bar/baz");
    }

    #[test]
    fn normalize_scan_input_strips_repeated_dot_slash() {
        let normalized = normalize_scan_input(Path::new("././src/lib.rs"));
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    fn normalize_scan_input_keeps_dot_for_empty_relative() {
        let normalized = normalize_scan_input(Path::new("./"));
        assert_eq!(normalized, ".");
    }

    #[test]
    fn scan_args_paths_mode_redacts_scan_paths_and_exclusions() {
        let paths = vec![PathBuf::from("src/lib.rs")];
        let scan_options = ScanOptions {
            excluded: vec!["target".to_string()],
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        assert_ne!(args.paths[0], "src/lib.rs");
        assert_ne!(args.excluded[0], "target");
        assert!(args.excluded_redacted);
    }

    #[test]
    fn scan_args_no_ignore_enables_sub_flags() {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            no_ignore: true,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, None);
        assert!(args.no_ignore_parent);
        assert!(args.no_ignore_dot);
        assert!(args.no_ignore_vcs);
    }

    proptest! {
        #[test]
        fn scan_args_preserves_redaction_and_ignore_invariants(
            paths in prop::collection::vec("[a-zA-Z0-9_\\-\\./\\\\]+", 1..10),
            excluded in prop::collection::vec("[a-zA-Z0-9_\\-\\.*]+", 0..5),
            redact_mode in prop::sample::select(vec![
                None,
                Some(RedactMode::None),
                Some(RedactMode::Paths),
                Some(RedactMode::All),
            ]),
            hidden in any::<bool>(),
            no_ignore in any::<bool>(),
            no_ignore_parent in any::<bool>(),
            no_ignore_dot in any::<bool>(),
            no_ignore_vcs in any::<bool>(),
            treat_doc_strings_as_comments in any::<bool>(),
        ) {
            let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
            let scan_options = ScanOptions {
                excluded: excluded.clone(),
                hidden,
                no_ignore,
                no_ignore_parent,
                no_ignore_dot,
                no_ignore_vcs,
                treat_doc_strings_as_comments,
                ..Default::default()
            };

            let args = scan_args(&path_bufs, &scan_options, redact_mode);
            let repeat = scan_args(&path_bufs, &scan_options, redact_mode);

            prop_assert_eq!(&args.paths, &repeat.paths);
            prop_assert_eq!(&args.excluded, &repeat.excluded);
            prop_assert_eq!(args.excluded_redacted, repeat.excluded_redacted);
            prop_assert_eq!(args.config, repeat.config);
            prop_assert_eq!(args.hidden, repeat.hidden);
            prop_assert_eq!(args.no_ignore, repeat.no_ignore);
            prop_assert_eq!(args.no_ignore_parent, repeat.no_ignore_parent);
            prop_assert_eq!(args.no_ignore_dot, repeat.no_ignore_dot);
            prop_assert_eq!(args.no_ignore_vcs, repeat.no_ignore_vcs);
            prop_assert_eq!(
                args.treat_doc_strings_as_comments,
                repeat.treat_doc_strings_as_comments
            );
            prop_assert_eq!(args.paths.len(), paths.len());
            prop_assert_eq!(args.hidden, hidden);
            prop_assert_eq!(args.no_ignore, no_ignore);
            prop_assert_eq!(args.treat_doc_strings_as_comments, treat_doc_strings_as_comments);

            let should_redact = matches!(redact_mode, Some(RedactMode::Paths | RedactMode::All));
            prop_assert_eq!(args.excluded_redacted, should_redact && !excluded.is_empty());

            if should_redact {
                prop_assert_eq!(args.excluded.len(), excluded.len());
                for value in &args.excluded {
                    prop_assert_eq!(value.len(), 16);
                    prop_assert!(value.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
                }
                for path in &args.paths {
                    prop_assert!(!path.contains('/'));
                    prop_assert!(!path.contains('\\'));
                }
            } else {
                let expected_paths: Vec<String> =
                    path_bufs.iter().map(|p| normalize_scan_input(p)).collect();
                prop_assert_eq!(&args.paths, &expected_paths);
                prop_assert_eq!(&args.excluded, &excluded);
                for path in &args.paths {
                    prop_assert!(!path.contains('\\'));
                }
            }

            if no_ignore {
                prop_assert!(args.no_ignore_parent);
                prop_assert!(args.no_ignore_dot);
                prop_assert!(args.no_ignore_vcs);
            } else {
                prop_assert_eq!(args.no_ignore_parent, no_ignore_parent);
                prop_assert_eq!(args.no_ignore_dot, no_ignore_dot);
                prop_assert_eq!(args.no_ignore_vcs, no_ignore_vcs);
            }
        }
    }
}
