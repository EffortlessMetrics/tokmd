//! W67 deep tests for tokmd-tokeignore crate.
//!
//! ~15 tests covering: template generation for each profile, pattern
//! correctness, cross-profile isolation, .runs/ invariant, comment format,
//! file I/O (create, overwrite, print mode, error), determinism, and
//! snapshot tests.

use std::fs;
use std::path::PathBuf;

use tokmd_cli_args::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
    InitArgs {
        dir,
        force,
        print,
        template: profile,
        non_interactive: true,
    }
}

/// Write a profile to a temp dir and return the file content.
fn template_content(profile: InitProfile) -> String {
    let dir = tempfile::tempdir().unwrap();
    let a = args(profile, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&a).unwrap().unwrap();
    fs::read_to_string(path).unwrap()
}

const ALL: &[InitProfile] = &[
    InitProfile::Default,
    InitProfile::Rust,
    InitProfile::Node,
    InitProfile::Mono,
    InitProfile::Python,
    InitProfile::Go,
    InitProfile::Cpp,
];

// ═══════════════════════════════════════════════════════════════════════════
// 1. Template generation per profile
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_each_profile_generates_non_empty_template() {
    for p in ALL {
        let t = template_content(*p);
        assert!(!t.is_empty(), "{p:?} should not be empty");
        assert!(t.starts_with('#'), "{p:?} should start with comment");
    }
}

#[test]
fn w67_default_mentions_tokeignore() {
    assert!(template_content(InitProfile::Default).contains("# .tokeignore"));
}

#[test]
fn w67_rust_header() {
    assert!(template_content(InitProfile::Rust).contains("(Rust)"));
}

#[test]
fn w67_node_header() {
    assert!(template_content(InitProfile::Node).contains("(Node)"));
}

#[test]
fn w67_python_header() {
    assert!(template_content(InitProfile::Python).contains("(Python)"));
}

#[test]
fn w67_go_header() {
    assert!(template_content(InitProfile::Go).contains("(Go)"));
}

#[test]
fn w67_cpp_header() {
    assert!(template_content(InitProfile::Cpp).contains("(C++)"));
}

#[test]
fn w67_mono_header() {
    assert!(template_content(InitProfile::Mono).contains("(Monorepo)"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Cross-profile isolation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_rust_excludes_foreign_patterns() {
    let t = template_content(InitProfile::Rust);
    assert!(
        !t.contains("node_modules"),
        "Rust should not mention node_modules"
    );
    assert!(
        !t.contains("__pycache__"),
        "Rust should not mention __pycache__"
    );
}

#[test]
fn w67_node_excludes_foreign_patterns() {
    let t = template_content(InitProfile::Node);
    assert!(!t.contains("target/"), "Node should not mention target/");
    assert!(
        !t.contains("__pycache__"),
        "Node should not mention __pycache__"
    );
}

#[test]
fn w67_mono_covers_all_ecosystems() {
    let t = template_content(InitProfile::Mono);
    assert!(t.contains("target/"));
    assert!(t.contains("node_modules/"));
    assert!(t.contains("__pycache__/"));
    assert!(t.contains("vendor/"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. .runs/ invariant across all profiles
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_all_profiles_include_runs_dir() {
    for p in ALL {
        let t = template_content(*p);
        assert!(t.contains(".runs/"), "{p:?} must include .runs/");
        assert!(t.contains("**/.runs/"), "{p:?} must include **/.runs/");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Comment format (# not //)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_all_templates_use_hash_comments() {
    for p in ALL {
        let t = template_content(*p);
        for line in t.lines() {
            assert!(
                !line.trim().starts_with("//"),
                "{p:?}: found // comment in template"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Templates end with newline
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_all_templates_end_with_newline() {
    for p in ALL {
        let t = template_content(*p);
        assert!(t.ends_with('\n'), "{p:?} should end with newline");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. File I/O
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_init_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let a = args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&a).unwrap().unwrap();
    assert!(path.exists());
    assert_eq!(path, dir.path().join(".tokeignore"));
}

#[test]
fn w67_init_refuses_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "existing").unwrap();
    let a = args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let err = init_tokeignore(&a).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn w67_init_overwrites_with_force() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "old").unwrap();
    let a = args(InitProfile::Default, false, true, dir.path().to_path_buf());
    let path = init_tokeignore(&a).unwrap().unwrap();
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("# .tokeignore"));
}

#[test]
fn w67_print_mode_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let a = args(InitProfile::Default, true, false, dir.path().to_path_buf());
    assert!(init_tokeignore(&a).unwrap().is_none());
    assert!(!dir.path().join(".tokeignore").exists());
}

#[test]
fn w67_nonexistent_dir_errors() {
    let a = args(
        InitProfile::Default,
        false,
        false,
        PathBuf::from("/nonexistent/w67/dir"),
    );
    let err = init_tokeignore(&a).unwrap_err();
    assert!(err.to_string().contains("does not exist"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_deterministic_all_profiles() {
    for p in ALL {
        let a = template_content(*p);
        let b = template_content(*p);
        assert_eq!(a, b, "{p:?} must be deterministic");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Snapshot tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn w67_snapshot_default_template() {
    insta::assert_snapshot!("w67_default", template_content(InitProfile::Default));
}

#[test]
fn w67_snapshot_rust_template() {
    insta::assert_snapshot!("w67_rust", template_content(InitProfile::Rust));
}

#[test]
fn w67_snapshot_node_template() {
    insta::assert_snapshot!("w67_node", template_content(InitProfile::Node));
}
