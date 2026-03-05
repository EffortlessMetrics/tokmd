//! W59 — scan-args construction, redaction wiring, edge cases.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ═══════════════════════════════════════════════════════════════════
// normalize_scan_input
// ═══════════════════════════════════════════════════════════════════

#[test]
fn normalize_plain_relative_path() {
    assert_eq!(normalize_scan_input(Path::new("src/lib.rs")), "src/lib.rs");
}

#[test]
fn normalize_strips_single_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./src/lib.rs")), "src/lib.rs");
}

#[test]
fn normalize_strips_repeated_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("././src/lib.rs")), "src/lib.rs");
}

#[test]
fn normalize_empty_relative_becomes_dot() {
    assert_eq!(normalize_scan_input(Path::new("./")), ".");
}

#[test]
fn normalize_bare_dot_stays_dot() {
    assert_eq!(normalize_scan_input(Path::new(".")), ".");
}

#[test]
fn normalize_backslashes_to_forward() {
    let result = normalize_scan_input(Path::new("src\\main.rs"));
    assert!(!result.contains('\\'), "backslash found: {result}");
}

#[test]
fn normalize_preserves_non_dot_path() {
    assert_eq!(normalize_scan_input(Path::new("crates/foo/src")), "crates/foo/src");
}

#[test]
fn normalize_deep_nested_path() {
    assert_eq!(
        normalize_scan_input(Path::new("a/b/c/d/e/f/g.rs")),
        "a/b/c/d/e/f/g.rs"
    );
}

#[test]
fn normalize_idempotent() {
    let once = normalize_scan_input(Path::new("./src/lib.rs"));
    let twice = normalize_scan_input(Path::new(&once));
    assert_eq!(once, twice);
}

// ═══════════════════════════════════════════════════════════════════
// ScanArgs construction — no redaction
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scan_args_no_redaction_preserves_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.paths, vec!["src/lib.rs"]);
}

#[test]
fn scan_args_redact_none_preserves_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, Some(RedactMode::None));
    assert_eq!(args.paths, vec!["src/lib.rs"]);
}

#[test]
fn scan_args_no_redaction_preserves_exclusions() {
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
fn scan_args_multiple_paths_preserved() {
    let paths = vec![
        PathBuf::from("src"),
        PathBuf::from("tests"),
        PathBuf::from("benches"),
    ];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.paths.len(), 3);
    assert_eq!(args.paths[0], "src");
    assert_eq!(args.paths[1], "tests");
    assert_eq!(args.paths[2], "benches");
}

#[test]
fn scan_args_empty_paths_vec() {
    let paths: Vec<PathBuf> = vec![];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert!(args.paths.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Redaction wiring
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scan_args_redact_paths_mode_redacts() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_ne!(args.paths[0], "src/lib.rs", "path should be redacted");
    assert_ne!(args.excluded[0], "target", "exclusion should be redacted");
    assert!(args.excluded_redacted);
}

#[test]
fn scan_args_redact_all_mode_redacts() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, Some(RedactMode::All));
    assert_ne!(args.paths[0], "src/lib.rs");
    assert_ne!(args.excluded[0], "target");
    assert!(args.excluded_redacted);
}

#[test]
fn scan_args_redact_paths_and_all_produce_same_paths() {
    let paths = vec![PathBuf::from("crates/foo/src/lib.rs")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        ..Default::default()
    };
    let a = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let b = scan_args(&paths, &opts, Some(RedactMode::All));
    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
}

#[test]
fn scan_args_excluded_redacted_false_when_empty_exclusions() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    // No exclusions → excluded_redacted should be false even with redaction
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_excluded_redacted_false_when_no_redact() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, None);
    assert!(!args.excluded_redacted);
}

#[test]
fn scan_args_path_count_preserved_under_redaction() {
    let paths: Vec<PathBuf> = (0..5).map(|i| PathBuf::from(format!("dir{i}/file.rs"))).collect();
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(args.paths.len(), 5);
}

#[test]
fn scan_args_exclusion_count_preserved_under_redaction() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        excluded: vec!["a".into(), "b".into(), "c".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(args.excluded.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════
// Deterministic hash generation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scan_args_deterministic_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions::default();
    let a = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let b = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(a.paths, b.paths);
}

#[test]
fn scan_args_deterministic_exclusions() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        excluded: vec!["target".into(), "dist".into()],
        ..Default::default()
    };
    let a = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let b = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(a.excluded, b.excluded);
}

#[test]
fn scan_args_deterministic_all_fields() {
    let paths = vec![PathBuf::from("crates/a"), PathBuf::from("crates/b")];
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        hidden: true,
        no_ignore: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let a = scan_args(&paths, &opts, Some(RedactMode::All));
    let b = scan_args(&paths, &opts, Some(RedactMode::All));
    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
    assert_eq!(a.excluded_redacted, b.excluded_redacted);
    assert_eq!(a.hidden, b.hidden);
    assert_eq!(a.no_ignore, b.no_ignore);
    assert_eq!(a.no_ignore_parent, b.no_ignore_parent);
    assert_eq!(a.no_ignore_dot, b.no_ignore_dot);
    assert_eq!(a.no_ignore_vcs, b.no_ignore_vcs);
    assert_eq!(a.treat_doc_strings_as_comments, b.treat_doc_strings_as_comments);
}

// ═══════════════════════════════════════════════════════════════════
// Boolean flag wiring
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scan_args_no_ignore_enables_all_sub_flags() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        no_ignore: true,
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, None);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn scan_args_sub_flags_independent_when_no_ignore_false() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, None);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn scan_args_hidden_flag_passed_through() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        hidden: true,
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, None);
    assert!(args.hidden);
}

#[test]
fn scan_args_treat_doc_strings_flag_passed_through() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, None);
    assert!(args.treat_doc_strings_as_comments);
}

#[test]
fn scan_args_boolean_flags_independent_of_redaction() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        hidden: true,
        no_ignore: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let without = scan_args(&paths, &opts, None);
    let with_paths = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let with_all = scan_args(&paths, &opts, Some(RedactMode::All));

    assert_eq!(without.hidden, with_paths.hidden);
    assert_eq!(without.hidden, with_all.hidden);
    assert_eq!(without.no_ignore, with_paths.no_ignore);
    assert_eq!(without.treat_doc_strings_as_comments, with_all.treat_doc_strings_as_comments);
}

// ═══════════════════════════════════════════════════════════════════
// Edge cases: empty paths, special characters
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scan_args_dot_path_normalizes() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.paths, vec!["."]);
}

#[test]
fn scan_args_dot_slash_path_normalizes() {
    let paths = vec![PathBuf::from("./")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.paths, vec!["."]);
}

#[test]
fn scan_args_repeated_dot_slash_normalizes() {
    let paths = vec![PathBuf::from("././src")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.paths, vec!["src"]);
}

#[test]
fn scan_args_paths_with_spaces() {
    let paths = vec![PathBuf::from("my project/src/lib.rs")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert_eq!(args.paths[0], "my project/src/lib.rs");
}

#[test]
fn scan_args_paths_with_unicode() {
    let paths = vec![PathBuf::from("données/résumé.rs")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    assert!(args.paths[0].contains("données"));
}

#[test]
fn scan_args_redaction_hides_original_long_segments() {
    let paths = vec![PathBuf::from("secret_project/internal/api.rs")];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let redacted = &args.paths[0];
    assert!(
        !redacted.contains("secret_project"),
        "original segment leaked: {redacted}"
    );
    assert!(
        !redacted.contains("internal"),
        "original segment leaked: {redacted}"
    );
}

#[test]
fn scan_args_redaction_hides_exclusion_values() {
    let paths = vec![PathBuf::from(".")];
    let opts = ScanOptions {
        excluded: vec!["my_secret_dir".into()],
        ..Default::default()
    };
    let args = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert!(
        !args.excluded[0].contains("my_secret_dir"),
        "exclusion leaked: {}",
        args.excluded[0]
    );
}

#[test]
fn scan_args_no_backslashes_in_output_paths() {
    let paths = vec![
        PathBuf::from("src\\lib.rs"),
        PathBuf::from("tests\\test.rs"),
    ];
    let opts = ScanOptions::default();
    let args = scan_args(&paths, &opts, None);
    for p in &args.paths {
        assert!(!p.contains('\\'), "backslash in output: {p}");
    }
}

#[test]
fn scan_args_single_path_via_slice_from_ref() {
    let p = PathBuf::from("src/lib.rs");
    let opts = ScanOptions::default();
    let args = scan_args(std::slice::from_ref(&p), &opts, None);
    assert_eq!(args.paths, vec!["src/lib.rs"]);
}
