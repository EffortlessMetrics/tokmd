//! Deep tests for tokmd-scan-args — W76
//!
//! Covers: normalize_scan_input edge cases, scan_args construction with
//! every option combination, redaction wiring (None/Paths/All),
//! deterministic serialization, and JSON round-trip fidelity.

use std::path::{Path, PathBuf};

use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::{ConfigMode, RedactMode};

// ── helpers ─────────────────────────────────────────────────────────

fn opts_with_excluded(excluded: Vec<&str>) -> ScanOptions {
    ScanOptions {
        excluded: excluded.into_iter().map(String::from).collect(),
        ..Default::default()
    }
}

fn paths(ps: &[&str]) -> Vec<PathBuf> {
    ps.iter().map(PathBuf::from).collect()
}

// ═══════════════════════════════════════════════════════════════════
// 1. normalize_scan_input — deeper edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_normalize_empty_string_becomes_dot() {
    assert_eq!(normalize_scan_input(Path::new("")), ".");
}

#[test]
fn w76_normalize_single_segment_preserved() {
    assert_eq!(normalize_scan_input(Path::new("crates")), "crates");
}

#[test]
fn w76_normalize_trailing_slash_stripped_by_path() {
    // Path::new strips the trailing separator on most OSes
    let n = normalize_scan_input(Path::new("src/"));
    assert!(n == "src" || n == "src/");
}

#[test]
fn w76_normalize_deeply_nested_relative() {
    assert_eq!(
        normalize_scan_input(Path::new("./a/b/c/d/e/f.rs")),
        "a/b/c/d/e/f.rs"
    );
}

#[test]
fn w76_normalize_dot_dot_prefix_kept() {
    let n = normalize_scan_input(Path::new("../../other"));
    assert!(n.starts_with(".."), "parent traversals must be preserved");
}

#[test]
fn w76_normalize_no_backslashes_in_output() {
    let n = normalize_scan_input(Path::new("a\\b\\c"));
    assert!(!n.contains('\\'), "backslashes must be normalized: {n}");
}

// ═══════════════════════════════════════════════════════════════════
// 2. scan_args — construction & field mapping
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_scan_args_empty_paths_produces_empty_vec() {
    let args = scan_args(&[], &ScanOptions::default(), None);
    assert!(args.paths.is_empty());
}

#[test]
fn w76_scan_args_preserves_excluded_order() {
    let opts = opts_with_excluded(vec!["z", "a", "m"]);
    let args = scan_args(&paths(&["."]), &opts, None);
    assert_eq!(args.excluded, vec!["z", "a", "m"]);
}

#[test]
fn w76_scan_args_config_mode_auto_default() {
    let args = scan_args(&paths(&["."]), &ScanOptions::default(), None);
    assert_eq!(args.config, ConfigMode::Auto);
}

#[test]
fn w76_scan_args_config_mode_none_propagated() {
    let opts = ScanOptions {
        config: ConfigMode::None,
        ..Default::default()
    };
    let args = scan_args(&paths(&["."]), &opts, None);
    assert_eq!(args.config, ConfigMode::None);
}

#[test]
fn w76_scan_args_all_flags_default_false() {
    let args = scan_args(&paths(&["."]), &ScanOptions::default(), None);
    assert!(!args.hidden);
    assert!(!args.no_ignore);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
    assert!(!args.no_ignore_vcs);
    assert!(!args.treat_doc_strings_as_comments);
    assert!(!args.excluded_redacted);
}

// ═══════════════════════════════════════════════════════════════════
// 3. scan_args — no_ignore cascading logic
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_no_ignore_dot_independent_when_no_ignore_false() {
    let opts = ScanOptions {
        no_ignore_dot: true,
        ..Default::default()
    };
    let args = scan_args(&paths(&["."]), &opts, None);
    assert!(args.no_ignore_dot);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_vcs);
}

#[test]
fn w76_no_ignore_vcs_independent_when_no_ignore_false() {
    let opts = ScanOptions {
        no_ignore_vcs: true,
        ..Default::default()
    };
    let args = scan_args(&paths(&["."]), &opts, None);
    assert!(args.no_ignore_vcs);
    assert!(!args.no_ignore_parent);
    assert!(!args.no_ignore_dot);
}

#[test]
fn w76_all_sub_flags_true_when_no_ignore_set() {
    let opts = ScanOptions {
        no_ignore: true,
        ..Default::default()
    };
    let args = scan_args(&paths(&["."]), &opts, None);
    assert!(args.no_ignore_parent, "parent must cascade");
    assert!(args.no_ignore_dot, "dot must cascade");
    assert!(args.no_ignore_vcs, "vcs must cascade");
}

// ═══════════════════════════════════════════════════════════════════
// 4. scan_args — redaction wiring
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_redact_none_no_transformation() {
    let opts = opts_with_excluded(vec!["vendor"]);
    let args = scan_args(&paths(&["src"]), &opts, Some(RedactMode::None));
    assert_eq!(args.paths, vec!["src"]);
    assert_eq!(args.excluded, vec!["vendor"]);
    assert!(!args.excluded_redacted);
}

#[test]
fn w76_redact_paths_transforms_paths_and_excluded() {
    let opts = opts_with_excluded(vec!["build"]);
    let args = scan_args(&paths(&["my/code"]), &opts, Some(RedactMode::Paths));
    assert_ne!(args.paths[0], "my/code");
    assert_ne!(args.excluded[0], "build");
    assert!(args.excluded_redacted);
}

#[test]
fn w76_redact_all_transforms_like_paths() {
    let opts = opts_with_excluded(vec!["tmp"]);
    let a = scan_args(&paths(&["src"]), &opts, Some(RedactMode::Paths));
    let b = scan_args(&paths(&["src"]), &opts, Some(RedactMode::All));
    // Paths and All both redact paths+exclusions identically
    assert_eq!(a.paths, b.paths);
    assert_eq!(a.excluded, b.excluded);
    assert_eq!(a.excluded_redacted, b.excluded_redacted);
}

#[test]
fn w76_redact_with_empty_excluded_not_flagged() {
    let args = scan_args(&paths(&["src"]), &ScanOptions::default(), Some(RedactMode::Paths));
    assert!(!args.excluded_redacted, "no exclusions ⇒ flag stays false");
}

#[test]
fn w76_redact_multiple_excluded_all_hashed() {
    let opts = opts_with_excluded(vec!["target", "node_modules", ".cache"]);
    let args = scan_args(&paths(&["."]), &opts, Some(RedactMode::Paths));
    assert_eq!(args.excluded.len(), 3);
    for (i, original) in ["target", "node_modules", ".cache"].iter().enumerate() {
        assert_ne!(
            &args.excluded[i], original,
            "excluded[{i}] should be hashed"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// 5. deterministic serialization & JSON round-trip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w76_json_round_trip_all_flags_true() {
    let opts = ScanOptions {
        excluded: vec!["dist".to_string()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let args = scan_args(&paths(&["a", "b"]), &opts, None);
    let json = serde_json::to_string_pretty(&args).unwrap();
    let back: tokmd_types::ScanArgs = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, args.paths);
    assert_eq!(back.excluded, args.excluded);
    assert_eq!(back.hidden, args.hidden);
    assert_eq!(back.no_ignore, args.no_ignore);
    assert_eq!(back.no_ignore_parent, args.no_ignore_parent);
    assert_eq!(back.no_ignore_dot, args.no_ignore_dot);
    assert_eq!(back.no_ignore_vcs, args.no_ignore_vcs);
    assert_eq!(back.config, args.config);
    assert_eq!(
        back.treat_doc_strings_as_comments,
        args.treat_doc_strings_as_comments
    );
}

#[test]
fn w76_json_excluded_redacted_present_when_true() {
    let opts = opts_with_excluded(vec!["secret"]);
    let args = scan_args(&paths(&["."]), &opts, Some(RedactMode::Paths));
    let json = serde_json::to_string(&args).unwrap();
    assert!(
        json.contains("excluded_redacted"),
        "excluded_redacted must appear when true"
    );
}

#[test]
fn w76_json_excluded_redacted_absent_when_false() {
    let args = scan_args(&paths(&["."]), &ScanOptions::default(), None);
    let json = serde_json::to_string(&args).unwrap();
    assert!(
        !json.contains("excluded_redacted"),
        "excluded_redacted must be skipped when false"
    );
}

#[test]
fn w76_deterministic_across_repeated_calls() {
    let opts = ScanOptions {
        excluded: vec!["a".to_string(), "b".to_string()],
        hidden: true,
        ..Default::default()
    };
    let ps = paths(&["x", "y"]);
    let jsons: Vec<String> = (0..5)
        .map(|_| serde_json::to_string(&scan_args(&ps, &opts, Some(RedactMode::All))).unwrap())
        .collect();
    for j in &jsons[1..] {
        assert_eq!(&jsons[0], j, "all serializations must be identical");
    }
}
