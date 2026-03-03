//! Extended property tests for scan-args: config preservation, hash format, extension handling.

use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::{ConfigMode, RedactMode};

fn pathish_string() -> impl Strategy<Value = String> {
    let alphabet: Vec<char> = "/\\._abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    prop::collection::vec(prop::sample::select(alphabet), 1..64)
        .prop_map(|chars| chars.into_iter().collect())
}

fn redact_mode_strategy() -> impl Strategy<Value = Option<RedactMode>> {
    prop_oneof![
        Just(None),
        Just(Some(RedactMode::None)),
        Just(Some(RedactMode::Paths)),
        Just(Some(RedactMode::All)),
    ]
}

proptest! {
    // ── Config field is always preserved regardless of redaction ──────

    #[test]
    fn config_mode_preserved_across_all_redact_modes(
        redact in redact_mode_strategy(),
        config in prop_oneof![Just(ConfigMode::Auto), Just(ConfigMode::None)],
    ) {
        let paths = vec![PathBuf::from("src")];
        let scan_options = ScanOptions {
            config,
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, redact);
        prop_assert_eq!(args.config, config);
    }

    // ── Redacted exclusion hashes are exactly 16 hex chars ───────────

    #[test]
    fn redacted_exclusions_are_16_hex_chars(
        excluded in prop::collection::vec("[a-z]{3,20}", 1..5),
    ) {
        let paths = vec![PathBuf::from(".")];
        let scan_options = ScanOptions {
            excluded,
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        for hash in &args.excluded {
            prop_assert_eq!(hash.len(), 16, "hash length: {} for '{}'", hash.len(), hash);
            prop_assert!(hash.chars().all(|c| c.is_ascii_hexdigit()),
                "non-hex char in: {}", hash);
        }
    }

    // ── Redact mode None and Option::None produce identical results ──

    #[test]
    fn redact_none_and_option_none_are_equivalent(
        path_values in prop::collection::vec(pathish_string(), 1..4),
        excluded in prop::collection::vec(pathish_string(), 0..4),
        hidden in any::<bool>(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions {
            excluded,
            hidden,
            ..Default::default()
        };

        let a = scan_args(&paths, &scan_options, Some(RedactMode::None));
        let b = scan_args(&paths, &scan_options, None);

        prop_assert_eq!(a.paths, b.paths);
        prop_assert_eq!(a.excluded, b.excluded);
        prop_assert_eq!(a.excluded_redacted, b.excluded_redacted);
        prop_assert_eq!(a.hidden, b.hidden);
    }

    // ── Redacted paths with file extensions preserve the extension ────

    #[test]
    fn redacted_paths_preserve_file_extension(
        stem in "[a-z]{3,12}",
        ext in prop_oneof!["rs", "py", "js", "json", "toml", "md"],
    ) {
        let filename = format!("{}.{}", stem, ext);
        let paths = vec![PathBuf::from(&filename)];
        let scan_options = ScanOptions::default();

        let args = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        let expected_suffix = format!(".{}", ext);
        prop_assert!(
            args.paths[0].ends_with(&expected_suffix),
            "expected suffix '{}', got path '{}'", expected_suffix, args.paths[0]
        );
    }

    // ── Duplicate paths get identical hashes after redaction ──────────

    #[test]
    fn duplicate_paths_get_identical_redacted_hashes(
        path in "[a-z]{3,20}/[a-z]{3,10}\\.rs",
    ) {
        let paths = vec![PathBuf::from(&path), PathBuf::from(&path)];
        let scan_options = ScanOptions::default();

        let args = scan_args(&paths, &scan_options, Some(RedactMode::Paths));
        prop_assert_eq!(&args.paths[0], &args.paths[1]);
    }

    // ── Normalized output is always non-empty ────────────────────────

    #[test]
    fn scan_args_paths_are_never_empty_strings(
        path_values in prop::collection::vec(pathish_string(), 1..8),
        redact in redact_mode_strategy(),
    ) {
        let paths: Vec<PathBuf> = path_values.iter().map(PathBuf::from).collect();
        let scan_options = ScanOptions::default();

        let args = scan_args(&paths, &scan_options, redact);
        for p in &args.paths {
            prop_assert!(!p.is_empty(), "path should never be empty");
        }
    }

    // ── Parent-relative paths (..) are preserved through normalization ─

    #[test]
    fn parent_relative_paths_preserved(
        segments in prop::collection::vec("[a-z]{2,8}", 1..4),
    ) {
        let path_str = format!("../{}", segments.join("/"));
        let normalized = normalize_scan_input(&PathBuf::from(&path_str));
        prop_assert!(normalized.starts_with(".."), "expected '..' prefix, got: {}", normalized);
    }
}
