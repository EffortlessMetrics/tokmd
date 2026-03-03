//! Deep property-based tests for tokmd-scan-args.
//!
//! Covers: redacted hash properties, normalization composition,
//! determinism of the full pipeline, and config propagation.

use std::path::{Path, PathBuf};

use proptest::prelude::*;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

fn pathish_string() -> impl Strategy<Value = String> {
    let alphabet: Vec<char> = "/\\._abcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    prop::collection::vec(prop::sample::select(alphabet), 1..48)
        .prop_map(|chars| chars.into_iter().collect())
}

// =========================================================================
// Normalization composition: normalize(a/b) consistent parts
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn normalize_join_no_backslash(
        parts in prop::collection::vec("[a-z]{2,8}", 2..5),
    ) {
        let joined = parts.join("\\");
        let normalized = normalize_scan_input(Path::new(&joined));
        prop_assert!(!normalized.contains('\\'),
            "Normalized '{}' still contains backslash", normalized);
    }

    #[test]
    fn normalize_preserves_path_structure(
        parts in prop::collection::vec("[a-z]{2,8}", 1..5),
    ) {
        let path = parts.join("/");
        let normalized = normalize_scan_input(Path::new(&path));
        // Each part should appear in the normalized result
        for part in &parts {
            prop_assert!(normalized.contains(part.as_str()),
                "Normalized '{}' missing part '{}'", normalized, part);
        }
    }

    #[test]
    fn normalize_dot_prefix_stripped_consistently(
        parts in prop::collection::vec("[a-z]{2,6}", 1..4),
    ) {
        let path = parts.join("/");
        let dotted = format!("./{}", path);
        let n1 = normalize_scan_input(Path::new(&path));
        let n2 = normalize_scan_input(Path::new(&dotted));
        prop_assert_eq!(n1, n2, "./ prefix should be stripped");
    }
}

// =========================================================================
// Redacted output: hash format invariants
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn redacted_paths_are_valid_hashes(
        stem in "[a-z]{3,12}",
        ext in prop_oneof!["rs", "py", "js", "json"],
    ) {
        let filename = format!("{}.{}", stem, ext);
        let paths = vec![PathBuf::from(&filename)];
        let scan_options = ScanOptions::default();
        let args = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        let redacted = &args.paths[0];
        // Should be 16-char hex hash + "." + extension
        let expected_len = 16 + 1 + ext.len();
        prop_assert_eq!(redacted.len(), expected_len,
            "Redacted path '{}' wrong length (expected {})", redacted, expected_len);
        let hash_part = &redacted[..16];
        prop_assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash part '{}' not all hex", hash_part);
    }

    #[test]
    fn redacted_exclusions_are_deterministic(
        excluded in prop::collection::vec("[a-z]{4,12}", 1..5),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            ..Default::default()
        };
        let args1 = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        let args2 = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        prop_assert_eq!(args1.excluded, args2.excluded,
            "Redacted exclusions should be deterministic");
    }
}

// =========================================================================
// Full pipeline determinism with all flag combinations
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn full_scan_args_deterministic(
        path_values in prop::collection::vec(pathish_string(), 1..4),
        excluded in prop::collection::vec("[a-z]{3,10}", 0..3),
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        treat_doc_strings_as_comments in any::<bool>(),
        redact_idx in 0usize..4,
    ) {
        let redact = [None, Some(RedactMode::None), Some(RedactMode::Paths), Some(RedactMode::All)][redact_idx];
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions {
            excluded,
            hidden,
            no_ignore,
            treat_doc_strings_as_comments,
            ..Default::default()
        };
        let a = scan_args(&paths, &scan_options, redact);
        let b = scan_args(&paths, &scan_options, redact);
        prop_assert_eq!(a.paths, b.paths);
        prop_assert_eq!(a.excluded, b.excluded);
        prop_assert_eq!(a.hidden, b.hidden);
        prop_assert_eq!(a.no_ignore, b.no_ignore);
    }
}

// =========================================================================
// no_ignore propagation: parent flag implies sub-flags
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn no_ignore_true_forces_sub_flags(
        path_values in prop::collection::vec("[a-z]{2,8}", 1..3),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions {
            no_ignore: true,
            no_ignore_parent: false,
            no_ignore_dot: false,
            no_ignore_vcs: false,
            ..Default::default()
        };
        let args = scan_args(&paths, &scan_options, None);
        prop_assert!(args.no_ignore_parent, "no_ignore should force no_ignore_parent");
        prop_assert!(args.no_ignore_dot, "no_ignore should force no_ignore_dot");
        prop_assert!(args.no_ignore_vcs, "no_ignore should force no_ignore_vcs");
    }
}
