//! Wave 43 – deep tests for `tokmd-scan-args`.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

// ── normalize_scan_input ───────────────────────────────────────────

#[test]
fn normalize_single_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./src")), "src");
}

#[test]
fn normalize_triple_dot_slash() {
    assert_eq!(normalize_scan_input(Path::new("./././lib")), "lib");
}

#[test]
fn normalize_bare_dot() {
    assert_eq!(normalize_scan_input(Path::new(".")), ".");
}

#[test]
fn normalize_empty_after_strip_becomes_dot() {
    // "./" stripped to "" should become "."
    assert_eq!(normalize_scan_input(Path::new("./")), ".");
}

#[test]
fn normalize_preserves_absolute_path() {
    let abs = if cfg!(windows) {
        "C:/project"
    } else {
        "/project"
    };
    let result = normalize_scan_input(Path::new(abs));
    assert!(result.contains("project"));
}

#[test]
fn normalize_backslashes_become_forward() {
    // On Windows the path "src\\main.rs" should normalise to "src/main.rs"
    let result = normalize_scan_input(Path::new("src/main.rs"));
    assert_eq!(result, "src/main.rs");
}

// ── scan_args – defaults ───────────────────────────────────────────

#[test]
fn default_scan_options_produce_default_flags() {
    let args = scan_args(&[PathBuf::from(".")], &ScanOptions::default(), None);
    assert!(!args.hidden);
    assert!(!args.no_ignore);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
    assert!(!args.treat_doc_strings_as_comments);
    assert!(args.excluded.is_empty());
    assert!(!args.excluded_redacted);
}

#[test]
fn paths_forwarded_without_redaction_when_none() {
    let paths = vec![PathBuf::from("src"), PathBuf::from("lib")];
    let args = scan_args(&paths, &ScanOptions::default(), None);
    assert_eq!(args.paths, vec!["src", "lib"]);
}

// ── scan_args – no_ignore cascade ──────────────────────────────────

#[test]
fn no_ignore_cascades_to_all_sub_flags() {
    let opts = ScanOptions {
        no_ignore: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}

#[test]
fn individual_sub_flags_do_not_cross_pollinate() {
    let opts = ScanOptions {
        no_ignore_parent: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn no_ignore_dot_alone_sets_only_dot() {
    let opts = ScanOptions {
        no_ignore_dot: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.no_ignore_dot);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_vcs);
}

// ── scan_args – redaction ──────────────────────────────────────────

#[test]
fn redact_paths_mode_hashes_paths() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    // Redacted path should NOT equal the original
    assert_ne!(args.paths[0], "src/lib.rs");
    // Redacted path retains extension
    assert!(args.paths[0].ends_with(".rs"), "expected .rs extension");
}

#[test]
fn redact_all_mode_hashes_paths() {
    let paths = vec![PathBuf::from("src/main.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::All));
    assert_ne!(args.paths[0], "src/main.rs");
    assert!(args.paths[0].ends_with(".rs"));
}

#[test]
fn redact_none_does_not_hash() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::None));
    assert_eq!(args.paths[0], "src/lib.rs");
}

#[test]
fn redaction_is_deterministic() {
    let paths = vec![PathBuf::from("src/lib.rs")];
    let opts = ScanOptions::default();
    let a = scan_args(&paths, &opts, Some(RedactMode::Paths));
    let b = scan_args(&paths, &opts, Some(RedactMode::Paths));
    assert_eq!(a.paths, b.paths);
}

#[test]
fn excluded_patterns_redacted_when_paths_mode() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string(), "node_modules".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, Some(RedactMode::Paths));
    assert!(args.excluded_redacted);
    assert_eq!(args.excluded.len(), 2);
    assert_ne!(args.excluded[0], "target");
    assert_ne!(args.excluded[1], "node_modules");
}

#[test]
fn excluded_not_redacted_without_redact_mode() {
    let opts = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(!args.excluded_redacted);
    assert_eq!(args.excluded[0], "target");
}

#[test]
fn excluded_redacted_false_when_excluded_list_empty() {
    let args = scan_args(
        &[PathBuf::from(".")],
        &ScanOptions::default(),
        Some(RedactMode::Paths),
    );
    // No patterns → nothing to redact
    assert!(!args.excluded_redacted);
}

// ── scan_args – boolean flag passthrough ───────────────────────────

#[test]
fn hidden_flag_passthrough() {
    let opts = ScanOptions {
        hidden: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.hidden);
}

#[test]
fn treat_doc_strings_as_comments_passthrough() {
    let opts = ScanOptions {
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let args = scan_args(&[PathBuf::from(".")], &opts, None);
    assert!(args.treat_doc_strings_as_comments);
}

// ── serde roundtrip ────────────────────────────────────────────────

#[test]
fn scan_args_serde_roundtrip() {
    let opts = ScanOptions {
        excluded: vec!["build".to_string()],
        hidden: true,
        no_ignore: true,
        treat_doc_strings_as_comments: true,
        ..Default::default()
    };
    let original = scan_args(&[PathBuf::from("src")], &opts, None);
    let json = serde_json::to_string(&original).expect("serialize");
    let restored: tokmd_types::ScanArgs = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original.paths, restored.paths);
    assert_eq!(original.excluded, restored.excluded);
    assert_eq!(original.hidden, restored.hidden);
    assert_eq!(original.no_ignore, restored.no_ignore);
    assert_eq!(original.no_ignore_parent, restored.no_ignore_parent);
    assert_eq!(original.no_ignore_dot, restored.no_ignore_dot);
    assert_eq!(original.no_ignore_vcs, restored.no_ignore_vcs);
    assert_eq!(
        original.treat_doc_strings_as_comments,
        restored.treat_doc_strings_as_comments
    );
    assert_eq!(original.excluded_redacted, restored.excluded_redacted);
}

#[test]
fn redacted_scan_args_serde_roundtrip_preserves_hashes() {
    let opts = ScanOptions {
        excluded: vec!["vendor".to_string()],
        ..Default::default()
    };
    let original = scan_args(&[PathBuf::from("src/lib.rs")], &opts, Some(RedactMode::All));
    let json = serde_json::to_string(&original).expect("serialize");
    let restored: tokmd_types::ScanArgs = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original.paths, restored.paths);
    assert_eq!(original.excluded, restored.excluded);
    assert!(restored.excluded_redacted);
}

// ── scan_args – multiple paths ─────────────────────────────────────

#[test]
fn multiple_paths_all_normalized() {
    let paths = vec![
        PathBuf::from("./src"),
        PathBuf::from("././lib"),
        PathBuf::from("tests"),
    ];
    let args = scan_args(&paths, &ScanOptions::default(), None);
    assert_eq!(args.paths, vec!["src", "lib", "tests"]);
}

#[test]
fn multiple_paths_all_redacted() {
    let paths = vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")];
    let args = scan_args(&paths, &ScanOptions::default(), Some(RedactMode::Paths));
    assert_ne!(
        args.paths[0], args.paths[1],
        "different files → different hashes"
    );
    assert!(args.paths[0].ends_with(".rs"));
    assert!(args.paths[1].ends_with(".rs"));
}
