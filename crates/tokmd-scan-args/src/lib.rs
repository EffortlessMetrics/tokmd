//! Deterministic scan argument construction for receipt metadata.

use std::path::{Path, PathBuf};

use tokmd_redact::{redact_path, short_hash};
use tokmd_settings::ScanOptions;
use tokmd_types::{RedactMode, ScanArgs};

/// Normalize a path to forward slashes and strip leading `./` segments.
#[must_use]
pub fn normalize_scan_input(p: &Path) -> String {
    let mut normalized = tokmd_path::normalize_slashes(&p.display().to_string());

    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }

    if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized
    }
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
}
