//! Wave-38 deep tests for tokmd-tokeignore template generation.
//!
//! Covers per-profile pattern counts, UTF-8 validity, recursive
//! pattern pairing, print-vs-write interactions, and template
//! content invariants not yet exercised by existing suites.

use std::fs;
use std::path::PathBuf;

use tokmd_cli_args::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

fn args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
    InitArgs {
        dir,
        force,
        print,
        template: profile,
        non_interactive: true,
    }
}

// ── Template content invariants ───────────────────────────────────

#[test]
fn all_templates_are_valid_utf8() {
    let profiles = all_profiles();
    for profile in profiles {
        let dir = tempfile::tempdir().unwrap();
        init_tokeignore(&args(profile, false, false, dir.path().into())).unwrap();
        let bytes = fs::read(dir.path().join(".tokeignore")).unwrap();
        assert!(
            std::str::from_utf8(&bytes).is_ok(),
            "{profile:?} template is not valid UTF-8"
        );
    }
}

#[test]
fn all_templates_have_at_least_three_non_comment_non_empty_lines() {
    for profile in all_profiles() {
        let dir = tempfile::tempdir().unwrap();
        init_tokeignore(&args(profile, false, false, dir.path().into())).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        let pattern_lines = content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .count();
        assert!(
            pattern_lines >= 3,
            "{profile:?} has only {pattern_lines} pattern lines (expected ≥3)"
        );
    }
}

#[test]
fn no_template_has_trailing_whitespace_on_pattern_lines() {
    for profile in all_profiles() {
        let dir = tempfile::tempdir().unwrap();
        init_tokeignore(&args(profile, false, false, dir.path().into())).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        for (i, line) in content.lines().enumerate() {
            if !line.starts_with('#') && !line.trim().is_empty() {
                assert_eq!(
                    line,
                    line.trim_end(),
                    "{profile:?} line {i} has trailing whitespace"
                );
            }
        }
    }
}

#[test]
fn recursive_patterns_paired_with_local_patterns() {
    // For profiles that list "target/", there should also be "**/target/"
    let pairs = [
        (InitProfile::Default, "target/"),
        (InitProfile::Rust, "target/"),
        (InitProfile::Node, "node_modules/"),
        (InitProfile::Python, "__pycache__/"),
        (InitProfile::Go, "vendor/"),
        (InitProfile::Cpp, "build/"),
    ];
    for (profile, pattern) in pairs {
        let dir = tempfile::tempdir().unwrap();
        init_tokeignore(&args(profile, false, false, dir.path().into())).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        assert!(
            content.contains(pattern),
            "{profile:?} missing local pattern {pattern}"
        );
        let recursive = format!("**/{pattern}");
        assert!(
            content.contains(&recursive),
            "{profile:?} missing recursive pattern {recursive}"
        );
    }
}

#[test]
fn mono_contains_rust_directory_patterns() {
    let dir_rust = tempfile::tempdir().unwrap();
    let dir_mono = tempfile::tempdir().unwrap();
    init_tokeignore(&args(
        InitProfile::Rust,
        false,
        false,
        dir_rust.path().into(),
    ))
    .unwrap();
    init_tokeignore(&args(
        InitProfile::Mono,
        false,
        false,
        dir_mono.path().into(),
    ))
    .unwrap();
    let rust_content = fs::read_to_string(dir_rust.path().join(".tokeignore")).unwrap();
    let mono_content = fs::read_to_string(dir_mono.path().join(".tokeignore")).unwrap();
    // Directory-level patterns from Rust should appear in Mono
    for line in rust_content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.ends_with('/') {
            assert!(
                mono_content.contains(trimmed),
                "Mono template missing Rust directory pattern: {trimmed}"
            );
        }
    }
}

// ── Init logic edge cases ─────────────────────────────────────────

#[test]
fn print_with_force_still_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let result =
        init_tokeignore(&args(InitProfile::Default, true, true, dir.path().into())).unwrap();
    assert!(
        result.is_none(),
        "print mode should return None even with force"
    );
    assert!(
        !dir.path().join(".tokeignore").exists(),
        "print mode should not create file"
    );
}

#[test]
fn sequential_profiles_with_force_each_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let profiles = [InitProfile::Rust, InitProfile::Node, InitProfile::Python];
    for profile in profiles {
        init_tokeignore(&args(profile, false, true, dir.path().into())).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        let expected_tag = match profile {
            InitProfile::Rust => "(Rust)",
            InitProfile::Node => "(Node)",
            InitProfile::Python => "(Python)",
            _ => unreachable!(),
        };
        assert!(
            content.contains(expected_tag),
            "after writing {profile:?}, expected {expected_tag}"
        );
    }
}

#[test]
fn written_file_is_readable() {
    let dir = tempfile::tempdir().unwrap();
    let path = init_tokeignore(&args(InitProfile::Go, false, false, dir.path().into()))
        .unwrap()
        .unwrap();
    // Should be readable without error
    let content = fs::read_to_string(&path).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn returned_path_matches_expected_location() {
    let dir = tempfile::tempdir().unwrap();
    let path = init_tokeignore(&args(InitProfile::Cpp, false, false, dir.path().into()))
        .unwrap()
        .unwrap();
    assert_eq!(path, dir.path().join(".tokeignore"));
}

#[test]
fn default_template_has_tree_sitter_patterns() {
    let dir = tempfile::tempdir().unwrap();
    init_tokeignore(&args(InitProfile::Default, false, false, dir.path().into())).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(
        content.contains("tree-sitter"),
        "Default template should mention tree-sitter"
    );
}

#[test]
fn python_template_has_pyc_pattern() {
    let dir = tempfile::tempdir().unwrap();
    init_tokeignore(&args(InitProfile::Python, false, false, dir.path().into())).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(
        content.contains("*.pyc"),
        "Python template should exclude .pyc files"
    );
}

// ── Helpers ───────────────────────────────────────────────────────

fn all_profiles() -> [InitProfile; 7] {
    [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ]
}
