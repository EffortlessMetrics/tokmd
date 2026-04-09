//! Depth tests for tokmd-tokeignore – W63 wave.
//!
//! Covers template generation for each profile, pattern formatting,
//! comment handling, empty checks, file I/O, determinism, and
//! property-based testing.

use std::fs;
use std::path::PathBuf;

use tokmd_cli_args::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
    InitArgs {
        dir,
        force,
        print,
        template: profile,
        non_interactive: true,
    }
}

fn template_for(profile: InitProfile) -> String {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(profile, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    fs::read_to_string(path).unwrap()
}

// All profiles as an array for iteration.
const ALL_PROFILES: &[InitProfile] = &[
    InitProfile::Default,
    InitProfile::Rust,
    InitProfile::Node,
    InitProfile::Mono,
    InitProfile::Python,
    InitProfile::Go,
    InitProfile::Cpp,
];

// ===========================================================================
// 1. Template generation for different project types
// ===========================================================================

#[test]
fn default_template_generated() {
    let t = template_for(InitProfile::Default);
    assert!(t.contains("# .tokeignore"));
}

#[test]
fn rust_template_generated() {
    let t = template_for(InitProfile::Rust);
    assert!(t.contains("(Rust)"));
}

#[test]
fn node_template_generated() {
    let t = template_for(InitProfile::Node);
    assert!(t.contains("(Node)"));
}

#[test]
fn mono_template_generated() {
    let t = template_for(InitProfile::Mono);
    assert!(t.contains("(Monorepo)"));
}

#[test]
fn python_template_generated() {
    let t = template_for(InitProfile::Python);
    assert!(t.contains("(Python)"));
}

#[test]
fn go_template_generated() {
    let t = template_for(InitProfile::Go);
    assert!(t.contains("(Go)"));
}

#[test]
fn cpp_template_generated() {
    let t = template_for(InitProfile::Cpp);
    assert!(t.contains("(C++)"));
}

// ===========================================================================
// 2. Pattern formatting correctness
// ===========================================================================

#[test]
fn default_has_rust_patterns() {
    let t = template_for(InitProfile::Default);
    assert!(t.contains("target/"));
    assert!(t.contains("**/target/"));
}

#[test]
fn default_has_node_patterns() {
    let t = template_for(InitProfile::Default);
    assert!(t.contains("node_modules/"));
    assert!(t.contains("**/node_modules/"));
}

#[test]
fn default_has_python_patterns() {
    let t = template_for(InitProfile::Default);
    assert!(t.contains("__pycache__/"));
    assert!(t.contains("**/__pycache__/"));
}

#[test]
fn rust_has_bk_pattern() {
    let t = template_for(InitProfile::Rust);
    assert!(t.contains("**/*.rs.bk"));
}

#[test]
fn python_has_pyc_pattern() {
    let t = template_for(InitProfile::Python);
    assert!(t.contains("*.pyc"));
}

#[test]
fn python_has_pytest_cache() {
    let t = template_for(InitProfile::Python);
    assert!(t.contains(".pytest_cache/"));
}

#[test]
fn go_has_vendor_pattern() {
    let t = template_for(InitProfile::Go);
    assert!(t.contains("vendor/"));
}

#[test]
fn go_has_bin_pattern() {
    let t = template_for(InitProfile::Go);
    assert!(t.contains("bin/"));
}

#[test]
fn cpp_has_cmake_build_pattern() {
    let t = template_for(InitProfile::Cpp);
    assert!(t.contains("cmake-build-*/"));
}

#[test]
fn cpp_has_cache_dir() {
    let t = template_for(InitProfile::Cpp);
    assert!(t.contains(".cache/"));
}

#[test]
fn node_has_dist_pattern() {
    let t = template_for(InitProfile::Node);
    assert!(t.contains("dist/"));
    assert!(t.contains("**/dist/"));
}

// ===========================================================================
// 3. Comment handling in templates
// ===========================================================================

#[test]
fn all_templates_start_with_comment() {
    for profile in ALL_PROFILES {
        let t = template_for(*profile);
        assert!(
            t.starts_with('#'),
            "{profile:?} template should start with a comment"
        );
    }
}

#[test]
fn comments_use_hash_prefix() {
    for profile in ALL_PROFILES {
        let t = template_for(*profile);
        for line in t.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                // Non-comment, non-empty line should look like a pattern
                assert!(
                    !trimmed.starts_with("//"),
                    "Templates should use # comments, not //: {trimmed}"
                );
            }
        }
    }
}

#[test]
fn default_template_has_section_comments() {
    let t = template_for(InitProfile::Default);
    assert!(t.contains("# --- Rust"));
    assert!(t.contains("# --- Node"));
    assert!(t.contains("# --- Python"));
}

// ===========================================================================
// 4. All templates end with newline
// ===========================================================================

#[test]
fn all_templates_end_with_newline() {
    for profile in ALL_PROFILES {
        let t = template_for(*profile);
        assert!(
            t.ends_with('\n'),
            "{profile:?} template should end with newline"
        );
    }
}

// ===========================================================================
// 5. All templates have .runs/ pattern
// ===========================================================================

#[test]
fn all_templates_have_runs_dir() {
    for profile in ALL_PROFILES {
        let t = template_for(*profile);
        assert!(
            t.contains(".runs/"),
            "{profile:?} template should include .runs/"
        );
    }
}

#[test]
fn all_templates_have_recursive_runs() {
    for profile in ALL_PROFILES {
        let t = template_for(*profile);
        assert!(
            t.contains("**/.runs/"),
            "{profile:?} template should include **/.runs/"
        );
    }
}

// ===========================================================================
// 6. Language-specific patterns (cross-profile checks)
// ===========================================================================

#[test]
fn rust_no_node_modules() {
    let t = template_for(InitProfile::Rust);
    assert!(!t.contains("node_modules"));
}

#[test]
fn rust_no_pycache() {
    let t = template_for(InitProfile::Rust);
    assert!(!t.contains("__pycache__"));
}

#[test]
fn node_no_target() {
    let t = template_for(InitProfile::Node);
    assert!(!t.contains("target/"));
}

#[test]
fn node_no_pycache() {
    let t = template_for(InitProfile::Node);
    assert!(!t.contains("__pycache__"));
}

#[test]
fn python_no_node_modules() {
    let t = template_for(InitProfile::Python);
    assert!(!t.contains("node_modules"));
}

#[test]
fn go_no_pycache() {
    let t = template_for(InitProfile::Go);
    assert!(!t.contains("__pycache__"));
}

#[test]
fn mono_has_all_ecosystems() {
    let t = template_for(InitProfile::Mono);
    assert!(t.contains("target/"));
    assert!(t.contains("node_modules/"));
    assert!(t.contains("__pycache__/"));
    assert!(t.contains("vendor/"));
}

#[test]
fn default_has_generated_code_patterns() {
    let t = template_for(InitProfile::Default);
    assert!(t.contains("generated/"));
    assert!(t.contains("*.generated.*"));
    assert!(t.contains("*.gen.*"));
}

#[test]
fn default_has_coverage_patterns() {
    let t = template_for(InitProfile::Default);
    assert!(t.contains("coverage/"));
    assert!(t.contains("lcov.info"));
}

// ===========================================================================
// 7. File I/O behavior
// ===========================================================================

#[test]
fn init_creates_tokeignore_file() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_some());
    assert!(dir.path().join(".tokeignore").exists());
}

#[test]
fn init_returns_correct_path() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Rust, false, false, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    assert_eq!(path, dir.path().join(".tokeignore"));
}

#[test]
fn init_refuses_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "existing").unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let err = init_tokeignore(&args).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn init_overwrites_with_force() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "old").unwrap();
    let args = make_args(InitProfile::Default, false, true, dir.path().to_path_buf());
    let path = init_tokeignore(&args).unwrap().unwrap();
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("# .tokeignore"));
    assert!(!content.contains("old"));
}

#[test]
fn init_print_returns_none_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(InitProfile::Default, true, false, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_none());
    assert!(!dir.path().join(".tokeignore").exists());
}

#[test]
fn init_nonexistent_dir_errors() {
    let args = make_args(
        InitProfile::Default,
        false,
        false,
        PathBuf::from("/nonexistent/w63/dir"),
    );
    let err = init_tokeignore(&args).unwrap_err();
    assert!(err.to_string().contains("does not exist"));
}

#[test]
fn init_each_profile_writes_correct_content() {
    for profile in ALL_PROFILES {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(*profile, false, false, dir.path().to_path_buf());
        let path = init_tokeignore(&args).unwrap().unwrap();
        let content = fs::read_to_string(path).unwrap();
        // Every template starts with a comment
        assert!(content.starts_with('#'), "{profile:?}");
        // Every template has .runs/
        assert!(content.contains(".runs/"), "{profile:?}");
    }
}

// ===========================================================================
// 8. Determinism: same profile produces same template
// ===========================================================================

#[test]
fn deterministic_default() {
    assert_eq!(
        template_for(InitProfile::Default),
        template_for(InitProfile::Default)
    );
}

#[test]
fn deterministic_rust() {
    assert_eq!(
        template_for(InitProfile::Rust),
        template_for(InitProfile::Rust)
    );
}

#[test]
fn deterministic_all_profiles() {
    for profile in ALL_PROFILES {
        let a = template_for(*profile);
        let b = template_for(*profile);
        assert_eq!(a, b, "{profile:?} should be deterministic");
    }
}

// ===========================================================================
// 9. Template non-emptiness
// ===========================================================================

#[test]
fn all_templates_non_empty() {
    for profile in ALL_PROFILES {
        let t = template_for(*profile);
        assert!(!t.is_empty(), "{profile:?} template should not be empty");
    }
}

#[test]
fn all_templates_have_at_least_one_pattern() {
    for profile in ALL_PROFILES {
        let t = template_for(*profile);
        let has_pattern = t.lines().any(|l| {
            let trimmed = l.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        });
        assert!(has_pattern, "{profile:?} should have at least one pattern");
    }
}

// ===========================================================================
// 10. Property tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_profile() -> impl Strategy<Value = InitProfile> {
        prop_oneof![
            Just(InitProfile::Default),
            Just(InitProfile::Rust),
            Just(InitProfile::Node),
            Just(InitProfile::Mono),
            Just(InitProfile::Python),
            Just(InitProfile::Go),
            Just(InitProfile::Cpp),
        ]
    }

    proptest! {
        #[test]
        fn template_always_starts_with_hash(profile in arb_profile()) {
            let t = template_for(profile);
            prop_assert!(t.starts_with('#'));
        }

        #[test]
        fn template_always_ends_with_newline(profile in arb_profile()) {
            let t = template_for(profile);
            prop_assert!(t.ends_with('\n'));
        }

        #[test]
        fn template_always_has_runs(profile in arb_profile()) {
            let t = template_for(profile);
            prop_assert!(t.contains(".runs/"));
        }

        #[test]
        fn template_always_non_empty(profile in arb_profile()) {
            let t = template_for(profile);
            prop_assert!(!t.is_empty());
        }

        #[test]
        fn template_lines_are_valid_gitignore(profile in arb_profile()) {
            let t = template_for(profile);
            for line in t.lines() {
                let trimmed = line.trim();
                // Valid gitignore: empty, comment, or pattern
                prop_assert!(
                    trimmed.is_empty()
                        || trimmed.starts_with('#')
                        || trimmed.starts_with('!')
                        || trimmed.starts_with('*')
                        || trimmed.starts_with('.')
                        || trimmed.starts_with('[')
                        || trimmed.chars().next().is_none_or(|c| c.is_alphanumeric() || c == '_'),
                    "Invalid gitignore line: {}", trimmed
                );
            }
        }

        #[test]
        fn template_no_double_slash_comments(profile in arb_profile()) {
            let t = template_for(profile);
            for line in t.lines() {
                prop_assert!(
                    !line.trim().starts_with("//"),
                    "Templates must use # comments: {}", line
                );
            }
        }

        #[test]
        fn template_is_deterministic(profile in arb_profile()) {
            let a = template_for(profile);
            let b = template_for(profile);
            prop_assert_eq!(a, b);
        }

        #[test]
        fn init_creates_file_for_any_profile(profile in arb_profile()) {
            let dir = tempfile::tempdir().unwrap();
            let args = make_args(profile, false, false, dir.path().to_path_buf());
            let path = init_tokeignore(&args).unwrap().unwrap();
            prop_assert!(path.exists());
            let content = std::fs::read_to_string(&path).unwrap();
            prop_assert!(content.starts_with('#'));
        }

        #[test]
        fn print_mode_never_creates_file(profile in arb_profile()) {
            let dir = tempfile::tempdir().unwrap();
            let args = make_args(profile, true, false, dir.path().to_path_buf());
            let result = init_tokeignore(&args).unwrap();
            prop_assert!(result.is_none());
            prop_assert!(!dir.path().join(".tokeignore").exists());
        }
    }
}
