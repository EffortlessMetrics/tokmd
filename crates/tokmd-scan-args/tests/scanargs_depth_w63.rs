//! Depth tests for tokmd-scan-args (W63).
//!
//! Covers ScanArgs construction, metadata field correctness, redaction wiring,
//! default values, path normalization, deterministic generation, and property tests.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ---------------------------------------------------------------------------
// 1. normalize_scan_input basics
// ---------------------------------------------------------------------------

#[test]
fn normalize_plain_relative_path() {
    assert_eq!(normalize_scan_input(Path::new("src/lib.rs")), "src/lib.rs");
}

#[test]
fn normalize_strips_single_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./src")), "src");
}

#[test]
fn normalize_strips_multiple_dot_slashes() {
    assert_eq!(
        normalize_scan_input(Path::new("./././foo/bar")),
        "foo/bar"
    );
}

#[test]
fn normalize_bare_dot_becomes_dot() {
    assert_eq!(normalize_scan_input(Path::new(".")), ".");
}

#[test]
fn normalize_dot_slash_becomes_dot() {
    assert_eq!(normalize_scan_input(Path::new("./")), ".");
}

#[test]
fn normalize_preserves_absolute_unix_path() {
    // On Windows this is a relative path but forward slashes are kept
    let n = normalize_scan_input(Path::new("/usr/local/bin"));
    assert!(n.contains("usr"));
}

#[test]
fn normalize_forward_slashes_preserved() {
    let n = normalize_scan_input(Path::new("a/b/c/d"));
    assert_eq!(n, "a/b/c/d");
}

// ---------------------------------------------------------------------------
// 2. scan_args with no redaction
// ---------------------------------------------------------------------------

#[test]
fn scan_args_no_redact_passes_paths_through() {
    let paths = vec![PathBuf::from("src"), PathBuf::from("tests")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.paths, vec!["src", "tests"]);
}

#[test]
fn scan_args_no_redact_passes_excluded_through() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        excluded: vec!["target".into(), "node_modules".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.excluded, vec!["target", "node_modules"]);
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_redact_none_is_same_as_no_redact() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions::default();
    let a1 = scan_args(&paths, &opts, None);
    let a2 = scan_args(&paths, &opts, Some(RedactMode::None));
    assert_eq!(a1.paths, a2.paths);
    assert_eq!(a1.excluded, a2.excluded);
}

// ---------------------------------------------------------------------------
// 3. scan_args with RedactMode::Paths
// ---------------------------------------------------------------------------

#[test]
fn scan_args_paths_mode_redacts_paths() {
    let paths = vec![PathBuf::from("src/main.rs")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_ne!(args.paths[0], "src/main.rs");
    // Redacted path should preserve .rs extension
    assert!(args.paths[0].ends_with(".rs"), "got: {}", args.paths[0]);
}

#[test]
fn scan_args_paths_mode_redacts_exclusions() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_ne!(args.excluded[0], "target");
    assert!(args.excluded_redacted);
}

#[test]
fn scan_args_paths_mode_empty_excluded_not_redacted() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert!(!args.excluded_redacted);
    assert!(args.excluded.is_empty());
}

#[test]
fn scan_args_all_mode_also_redacts() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions {
        excluded: vec!["vendor".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, Some(RedactMode::All));
    assert_ne!(args.paths[0], "src/lib.rs");
    assert_ne!(args.excluded[0], "vendor");
    assert!(args.excluded_redacted);
}

// ---------------------------------------------------------------------------
// 4. Boolean flag propagation
// ---------------------------------------------------------------------------

#[test]
fn hidden_flag_propagated() {
    let opts = ScanOptions {
        hidden: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.hidden);
}

#[test]
fn treat_doc_strings_propagated() {
    let opts = ScanOptions {
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn no_ignore_enables_all_sub_flags() {
    let opts = ScanOptions {
        no_ignore: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn no_ignore_parent_independent() {
    let opts = ScanOptions {
        no_ignore_parent: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(!args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn no_ignore_dot_independent() {
    let opts = ScanOptions {
        no_ignore_dot: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore_dot);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn no_ignore_vcs_independent() {
    let opts = ScanOptions {
        no_ignore_vcs: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore_vcs);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
}

// ---------------------------------------------------------------------------
// 5. Default values
// ---------------------------------------------------------------------------

#[test]
fn default_scan_options_produces_clean_args() {
    let args = scan_args(&[PathBuf::from(".")], &ScanOptions::default(), None);
    assert_eq!(args.paths, vec!["."]);
    assert!(args.excluded.is_empty());
    assert!(!args.excluded_redacted);
    assert!(!args.hidden);
    assert!(!args.no_ignore);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
    assert!(!args.treat_doc_strings_as_comments);
}

#[test]
fn empty_paths_produces_empty_args_paths() {
    let args = scan_args(&[], &ScanOptions::default(), None);
    assert!(args.paths.is_empty());
}

// ---------------------------------------------------------------------------
// 6. Path normalization in args
// ---------------------------------------------------------------------------

#[test]
fn scan_args_normalizes_dot_prefix_in_paths() {
    let paths = vec![PathBuf::from("./src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), None);
    assert_eq!(args.paths[0], "src/lib.rs");
}

#[test]
fn scan_args_normalizes_multiple_paths() {
    let paths = vec![
        PathBuf::from("./src"),
        PathBuf::from("./tests"),
        PathBuf::from("benches"),
    ];
    let args = scan_args(&paths, &ScanOptions::default(), None);
    assert_eq!(args.paths, vec!["src", "tests", "benches"]);
}

// ---------------------------------------------------------------------------
// 7. Deterministic arg generation
// ---------------------------------------------------------------------------

#[test]
fn scan_args_deterministic_no_redact() {
    let paths = vec![PathBuf::from("src"), PathBuf::from("tests")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        hidden: true,
        ..Default::default()
    };
    let a1 = scan_args(&paths, &opts, None);
    let a2 = scan_args(&paths, &opts, None);
    assert_eq!(format!("{a1:?}"), format!("{a2:?}"));
}

#[test]
fn scan_args_deterministic_with_redact() {
    let paths = vec![PathBuf::from("src/main.rs")];
    let opts = ScanOptions {
        excluded: vec!["target".into(), "vendor".into()],
        ..Default::default()
    };
    let a1 = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let a2 = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(a1.paths, a2.paths);
    assert_eq!(a1.excluded, a2.excluded);
}

#[test]
fn scan_args_100_calls_identical() {
    let paths = vec![PathBuf::from("src")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        ..Default::default()
    };
    let expected = scan_args(&paths, &opts, Some(RedactMode::All));
    for _ in 0..100 {
        let actual = scan_args(&paths, &opts, Some(RedactMode::All));
        assert_eq!(expected.paths, actual.paths);
        assert_eq!(expected.excluded, actual.excluded);
    }
}

// ---------------------------------------------------------------------------
// 8. Redaction wiring correctness
// ---------------------------------------------------------------------------

#[test]
fn redacted_paths_are_hex_prefixed() {
    let paths = vec![PathBuf::from("src/main.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    let redacted = &args.paths[0];
    // hash.ext format: first 16 chars are hex
    let hash_part: String = redacted.chars().take(16).collect();
    assert!(
        hash_part.chars().all(|c| c.is_ascii_hexdigit()),
        "expected hex prefix, got: {hash_part}"
    );
}

#[test]
fn redacted_excluded_are_16_hex() {
    let opts = ScanOptions {
        excluded: vec!["target".into(), "*.log".into()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, Some(RedactMode::All));
    for ex in &args.excluded {
        assert_eq!(ex.len(), 16, "excluded hash wrong length: {ex}");
        assert!(ex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn redacted_path_preserves_extension() {
    let paths = vec![
        PathBuf::from("src/main.rs"),
        PathBuf::from("data/config.json"),
        PathBuf::from("Makefile"),
    ];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    assert!(args.paths[0].ends_with(".rs"));
    assert!(args.paths[1].ends_with(".json"));
    assert!(!args.paths[2].contains('.'));
}

#[test]
fn redaction_does_not_leak_original_path() {
    let paths = vec![PathBuf::from("secrets/passwords/vault.json")];
    let opts = ScanOptions {
        excluded: vec!["private_data".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, Some(RedactMode::All));
    assert!(!args.paths[0].contains("secrets"));
    assert!(!args.paths[0].contains("vault"));
    assert!(!args.excluded[0].contains("private"));
}

// ---------------------------------------------------------------------------
// 9. Multiple paths and exclusions
// ---------------------------------------------------------------------------

#[test]
fn scan_args_multiple_exclusions_all_redacted() {
    let opts = ScanOptions {
        excluded: vec![
            "target".into(),
            "node_modules".into(),
            "dist".into(),
            ".git".into(),
        ],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, Some(RedactMode::Paths));
    assert_eq!(args.excluded.len(), 4);
    assert!(args.excluded_redacted);
    for ex in &args.excluded {
        assert_eq!(ex.len(), 16);
    }
}

#[test]
fn scan_args_multiple_paths_all_redacted() {
    let paths: Vec<PathBuf> = vec!["src", "tests", "benches", "examples"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::All));
    assert_eq!(args.paths.len(), 4);
    for p in &args.paths {
        assert_eq!(p.len(), 16, "no ext, should be bare hash: {p}");
    }
}

// ---------------------------------------------------------------------------
// 10. Config mode propagation
// ---------------------------------------------------------------------------

#[test]
fn config_mode_auto_propagated() {
    let opts = ScanOptions::default();
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert_eq!(
        format!("{:?}", args.config),
        format!("{:?}", tokmd_types::ConfigMode::Auto)
    );
}

#[test]
fn config_mode_none_propagated() {
    let opts = ScanOptions {
        config: tokmd_types::ConfigMode::None,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert_eq!(
        format!("{:?}", args.config),
        format!("{:?}", tokmd_types::ConfigMode::None)
    );
}

// ---------------------------------------------------------------------------
// 11. Property tests (proptest)
// ---------------------------------------------------------------------------

mod property_tests {
    use std::path::PathBuf;

    use proptest::prelude::*;
    use tokmd_scan_args::{normalize_scan_input, scan_args};
    use tokmd_settings::ScanOptions;
    use tokmd_types::RedactMode;

    proptest! {
        #[test]
        fn normalize_never_empty(path in "[a-z./]{1,30}") {
            let p = std::path::Path::new(&path);
            let n = normalize_scan_input(p);
            prop_assert!(!n.is_empty(), "normalized to empty for: {path}");
        }

        #[test]
        fn normalize_no_leading_dot_slash(path in "[a-z]{1,5}(/[a-z]{1,5}){0,3}") {
            let with_dot = format!("./{path}");
            let n = normalize_scan_input(std::path::Path::new(&with_dot));
            prop_assert!(!n.starts_with("./"), "still has dot prefix: {n}");
        }

        #[test]
        fn scan_args_paths_count_matches(
            count in 1usize..10,
        ) {
            let paths: Vec<PathBuf> = (0..count).map(|i| PathBuf::from(format!("dir{i}"))).collect();
            let args = scan_args(&paths, &ScanOptions::default(), None);
            prop_assert_eq!(args.paths.len(), count);
        }

        #[test]
        fn scan_args_excluded_count_matches(
            count in 0usize..10,
        ) {
            let opts = ScanOptions {
                excluded: (0..count).map(|i| format!("ex{i}")).collect(),
                ..Default::default()
            };
            let args = scan_args(&[PathBuf::from(".")], &opts, None);
            prop_assert_eq!(args.excluded.len(), count);
        }

        #[test]
        fn redacted_paths_always_have_hex_prefix(
            name in "[a-z]{1,10}",
            ext in "[a-z]{1,4}",
        ) {
            let path = PathBuf::from(format!("{name}.{ext}"));
            let args = scan_args(&[path], &ScanOptions::default(), Some(RedactMode::Paths));
            let redacted = &args.paths[0];
            let hash_part: String = redacted.chars().take(16).collect();
            prop_assert_eq!(hash_part.len(), 16);
            prop_assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn redacted_excluded_always_16_hex(
            ex in "[a-z_]{1,20}",
        ) {
            let opts = ScanOptions {
                excluded: vec![ex.clone()],
                ..Default::default()
            };
            let args = scan_args(&[PathBuf::from(".")], &opts, Some(RedactMode::All));
            let h = &args.excluded[0];
            prop_assert_eq!(h.len(), 16);
            prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn scan_args_deterministic(
            name in "[a-z]{1,8}",
            ex in "[a-z]{1,8}",
        ) {
            let paths = vec![PathBuf::from(&name)];
            let opts = ScanOptions {
                excluded: vec![ex],
                ..Default::default()
            };
            let a1 = scan_args(&paths, &opts, Some(RedactMode::Paths));
            let a2 = scan_args(&paths, &opts, Some(RedactMode::Paths));
            prop_assert_eq!(a1.paths, a2.paths);
            prop_assert_eq!(a1.excluded, a2.excluded);
        }

        #[test]
        fn no_ignore_implies_sub_flags(
            no_ignore in proptest::bool::ANY,
        ) {
            let opts = ScanOptions {
                no_ignore,
                ..Default::default()
            };
            let args = scan_args(&[PathBuf::from(".")], &opts, None);
            if no_ignore {
                prop_assert!(args.no_ignore_parent);
                prop_assert!(args.no_ignore_dot);
                prop_assert!(args.no_ignore_vcs);
            }
        }
    }
}
