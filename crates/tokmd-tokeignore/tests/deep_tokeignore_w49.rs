//! Wave-49 deep tests for tokmd-tokeignore: template generation, pattern
//! validation, custom exclusion rules, property tests, and edge cases.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_config::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ── Helpers ──────────────────────────────────────────────────────────────

const ALL_PROFILES: [InitProfile; 7] = [
    InitProfile::Default,
    InitProfile::Rust,
    InitProfile::Node,
    InitProfile::Mono,
    InitProfile::Python,
    InitProfile::Go,
    InitProfile::Cpp,
];

fn args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
    InitArgs {
        dir,
        force,
        print,
        template: profile,
        non_interactive: true,
    }
}

fn write_template(profile: InitProfile) -> String {
    let dir = tempfile::tempdir().unwrap();
    init_tokeignore(&args(profile, false, false, dir.path().into())).unwrap();
    fs::read_to_string(dir.path().join(".tokeignore")).unwrap()
}

/// Extract non-comment, non-empty lines (i.e. actual glob patterns).
fn pattern_lines(content: &str) -> Vec<&str> {
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .collect()
}

// ── Template generation per project type ─────────────────────────────────

#[test]
fn default_template_contains_multi_ecosystem_patterns() {
    let c = write_template(InitProfile::Default);
    for expected in [
        "target/",
        "node_modules/",
        "__pycache__/",
        "vendor/",
        ".runs/",
    ] {
        assert!(c.contains(expected), "Default missing {expected}");
    }
}

#[test]
fn rust_template_contains_rs_bk_pattern() {
    let c = write_template(InitProfile::Rust);
    assert!(
        c.contains("*.rs.bk"),
        "Rust template should exclude .rs.bk backup files"
    );
}

#[test]
fn node_template_contains_dist_and_out() {
    let c = write_template(InitProfile::Node);
    assert!(c.contains("dist/"));
    assert!(c.contains("out/"));
}

#[test]
fn python_template_contains_tox_and_pytest_cache() {
    let c = write_template(InitProfile::Python);
    assert!(c.contains(".tox/"));
    assert!(c.contains(".pytest_cache/"));
    assert!(c.contains("htmlcov/"));
}

#[test]
fn go_template_contains_bin_directory() {
    let c = write_template(InitProfile::Go);
    assert!(c.contains("bin/"));
    assert!(c.contains("**/bin/"));
}

#[test]
fn cpp_template_contains_cache_directory() {
    let c = write_template(InitProfile::Cpp);
    assert!(c.contains(".cache/"));
    assert!(c.contains("**/.cache/"));
}

#[test]
fn mono_template_is_superset_of_default_directory_patterns() {
    let default = write_template(InitProfile::Default);
    let mono = write_template(InitProfile::Mono);
    // Every directory pattern in default that ends with / should appear in mono
    for line in pattern_lines(&default) {
        if line.ends_with('/') {
            assert!(
                mono.contains(line),
                "Mono missing default directory pattern: {line}"
            );
        }
    }
}

// ── Pattern validation (glob patterns are well-formed) ───────────────────

#[test]
fn all_patterns_are_valid_gitignore_syntax() {
    // Gitignore patterns should not contain unbalanced brackets or empty globs
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in pattern_lines(&content) {
            let open = line.chars().filter(|&c| c == '[').count();
            let close = line.chars().filter(|&c| c == ']').count();
            assert_eq!(
                open, close,
                "{profile:?} pattern has unbalanced brackets: {line}"
            );
            // No double slashes (malformed path)
            assert!(
                !line.contains("//"),
                "{profile:?} pattern has double slash: {line}"
            );
        }
    }
}

#[test]
fn recursive_glob_patterns_use_double_star() {
    // Every recursive pattern should use **/ prefix
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in pattern_lines(&content) {
            if line.starts_with("**/") {
                // Valid recursive pattern
                assert!(
                    line.len() > 3,
                    "{profile:?} has empty recursive pattern: {line}"
                );
            }
        }
    }
}

#[test]
fn no_pattern_contains_backslash() {
    // Gitignore patterns should use forward slashes only
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in pattern_lines(&content) {
            assert!(
                !line.contains('\\'),
                "{profile:?} pattern contains backslash: {line}"
            );
        }
    }
}

#[test]
fn wildcard_patterns_have_valid_structure() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in pattern_lines(&content) {
            if line.contains('*') && !line.starts_with("**/") {
                // Glob-style patterns like *.pyc, *.generated.* should have at least one non-star char
                let non_star: String = line.chars().filter(|&c| c != '*').collect();
                assert!(
                    !non_star.is_empty(),
                    "{profile:?} pattern is all stars: {line}"
                );
            }
        }
    }
}

// ── Custom exclusion rules ───────────────────────────────────────────────

#[test]
fn default_template_excludes_generated_code() {
    let c = write_template(InitProfile::Default);
    assert!(c.contains("generated/"));
    assert!(c.contains("*.generated.*"));
    assert!(c.contains("*.gen.*"));
}

#[test]
fn default_template_excludes_coverage_artifacts() {
    let c = write_template(InitProfile::Default);
    assert!(c.contains("coverage/"));
    assert!(c.contains(".coverage"));
    assert!(c.contains("lcov.info"));
}

#[test]
fn default_template_excludes_third_party_dirs() {
    let c = write_template(InitProfile::Default);
    assert!(c.contains("third_party/"));
    assert!(c.contains("external/"));
}

#[test]
fn default_template_excludes_tree_sitter_vendored_files() {
    let c = write_template(InitProfile::Default);
    assert!(c.contains("**/tree-sitter*/src/parser.c"));
    assert!(c.contains("**/tree-sitter*/src/scanner.c"));
}

#[test]
fn rust_template_excludes_coverage() {
    let c = write_template(InitProfile::Rust);
    assert!(c.contains("coverage/"));
}

#[test]
fn python_template_excludes_pyc_files() {
    let c = write_template(InitProfile::Python);
    assert!(c.contains("*.pyc"));
}

// ── Property tests ──────────────────────────────────────────────────────

fn arb_profile() -> impl Strategy<Value = InitProfile> {
    prop::sample::select(ALL_PROFILES.to_vec())
}

proptest! {
    #[test]
    fn generated_templates_are_never_empty(profile in arb_profile()) {
        let content = write_template(profile);
        prop_assert!(!content.is_empty(), "template for {profile:?} was empty");
    }

    #[test]
    fn generated_templates_always_start_with_comment(profile in arb_profile()) {
        let content = write_template(profile);
        prop_assert!(content.starts_with('#'), "template for {profile:?} should start with #");
    }

    #[test]
    fn generated_templates_always_end_with_newline(profile in arb_profile()) {
        let content = write_template(profile);
        prop_assert!(content.ends_with('\n'), "template for {profile:?} should end with newline");
    }

    #[test]
    fn generated_templates_always_contain_runs_exclusion(profile in arb_profile()) {
        let content = write_template(profile);
        prop_assert!(content.contains(".runs/"), "template for {profile:?} missing .runs/");
    }

    #[test]
    fn generated_templates_have_at_least_one_pattern(profile in arb_profile()) {
        let content = write_template(profile);
        let count = pattern_lines(&content).len();
        prop_assert!(count >= 1, "template for {profile:?} has no patterns");
    }
}

// ── Edge cases ───────────────────────────────────────────────────────────

#[test]
fn init_into_empty_directory_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    // Directory is empty — should succeed
    let result = init_tokeignore(&args(InitProfile::Default, false, false, dir.path().into()));
    assert!(result.is_ok());
    assert!(dir.path().join(".tokeignore").exists());
}

#[test]
fn init_into_directory_with_other_files_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# Hello").unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    let result = init_tokeignore(&args(InitProfile::Rust, false, false, dir.path().into()));
    assert!(result.is_ok());
    // Other files untouched
    assert_eq!(
        fs::read_to_string(dir.path().join("README.md")).unwrap(),
        "# Hello"
    );
}

#[test]
fn init_nonexistent_path_returns_error() {
    let path = PathBuf::from("C:\\nonexistent_w49_test_dir_999");
    let result = init_tokeignore(&args(InitProfile::Default, false, false, path));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("does not exist"));
}

#[test]
fn print_mode_does_not_create_file() {
    let dir = tempfile::tempdir().unwrap();
    let result = init_tokeignore(&args(InitProfile::Default, true, false, dir.path().into()));
    assert!(result.unwrap().is_none());
    assert!(!dir.path().join(".tokeignore").exists());
}

#[test]
fn print_with_force_still_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let result = init_tokeignore(&args(InitProfile::Node, true, true, dir.path().into()));
    assert!(result.unwrap().is_none());
}

#[test]
fn force_overwrite_replaces_content_completely() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "old sentinel content 12345").unwrap();
    init_tokeignore(&args(InitProfile::Go, false, true, dir.path().into())).unwrap();
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(!content.contains("old sentinel content 12345"));
    assert!(content.contains("(Go)"));
}

#[test]
fn refuse_overwrite_preserves_original_content() {
    let dir = tempfile::tempdir().unwrap();
    let original = "custom ignore rules\ntarget/\n";
    fs::write(dir.path().join(".tokeignore"), original).unwrap();
    let result = init_tokeignore(&args(InitProfile::Default, false, false, dir.path().into()));
    assert!(result.is_err());
    // Original content preserved
    let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert_eq!(content, original);
}

#[test]
fn all_profiles_produce_unique_content() {
    let mut seen = BTreeSet::new();
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        assert!(
            seen.insert(content.clone()),
            "{profile:?} produced duplicate content"
        );
    }
}

#[test]
fn returned_path_is_inside_target_directory() {
    let dir = tempfile::tempdir().unwrap();
    let path = init_tokeignore(&args(InitProfile::Cpp, false, false, dir.path().into()))
        .unwrap()
        .unwrap();
    assert!(path.starts_with(dir.path()));
    assert!(path.ends_with(".tokeignore"));
}

#[test]
fn idempotent_force_writes() {
    let dir = tempfile::tempdir().unwrap();
    let a = args(InitProfile::Python, false, true, dir.path().into());
    init_tokeignore(&a).unwrap();
    let c1 = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    init_tokeignore(&a).unwrap();
    let c2 = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert_eq!(c1, c2, "repeated force writes should be idempotent");
}
