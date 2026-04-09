//! Deep2 tests for tokmd-tokeignore: structural invariants, template
//! comparison, pattern classification, and cross-profile consistency
//! not covered by existing deep/bdd/properties/init/snapshot tests.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;

use tokmd_cli_args::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ── Helpers ─────────────────────────────────────────────────────────

const ALL_PROFILES: [InitProfile; 7] = [
    InitProfile::Default,
    InitProfile::Rust,
    InitProfile::Node,
    InitProfile::Mono,
    InitProfile::Python,
    InitProfile::Go,
    InitProfile::Cpp,
];

fn make_args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
    InitArgs {
        dir,
        force,
        print,
        template: profile,
        non_interactive: true,
    }
}

fn get_content(profile: InitProfile) -> String {
    let dir = tempfile::tempdir().unwrap();
    let args = make_args(profile, false, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    fs::read_to_string(dir.path().join(".tokeignore")).unwrap()
}

fn patterns(content: &str) -> Vec<&str> {
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

// ── Relative template sizes ─────────────────────────────────────────

#[test]
fn go_is_smallest_template() {
    let go_count = patterns(&get_content(InitProfile::Go)).len();
    for profile in [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Cpp,
    ] {
        let count = patterns(&get_content(profile)).len();
        assert!(
            go_count <= count,
            "Go ({go_count} patterns) should be <= {profile:?} ({count} patterns)"
        );
    }
}

#[test]
fn mono_and_default_are_largest_templates() {
    let mono_count = patterns(&get_content(InitProfile::Mono)).len();
    let default_count = patterns(&get_content(InitProfile::Default)).len();
    let large = mono_count.max(default_count);

    for profile in [
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let count = patterns(&get_content(profile)).len();
        assert!(
            count <= large,
            "{profile:?} ({count}) should be <= max(mono, default) ({large})"
        );
    }
}

// ── Pattern classification ──────────────────────────────────────────

#[test]
fn directory_patterns_end_with_slash() {
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        for pat in patterns(&content) {
            // Skip glob-only patterns like "*.gen.*"
            if pat.contains('.') && !pat.ends_with('/') && !pat.starts_with("**/") {
                continue;
            }
            if !pat.contains('*') && !pat.contains('.') {
                assert!(
                    pat.ends_with('/'),
                    "{profile:?}: directory pattern should end with /: {pat}"
                );
            }
        }
    }
}

#[test]
fn doublestar_patterns_start_with_doublestar_slash() {
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        for pat in patterns(&content) {
            if pat.contains("**") {
                assert!(
                    pat.starts_with("**/"),
                    "{profile:?}: doublestar pattern should start with **: {pat}"
                );
            }
        }
    }
}

// ── Cross-profile pattern consistency ───────────────────────────────

#[test]
fn runs_pattern_is_identical_across_all_profiles() {
    let mut runs_lines: Vec<(InitProfile, Vec<String>)> = Vec::new();
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        let runs: Vec<String> = patterns(&content)
            .iter()
            .filter(|p| p.contains(".runs"))
            .map(|p| p.to_string())
            .collect();
        runs_lines.push((profile, runs));
    }
    let reference = &runs_lines[0].1;
    for (profile, runs) in &runs_lines[1..] {
        assert_eq!(
            runs, reference,
            "{profile:?} .runs/ patterns differ from Default"
        );
    }
}

#[test]
fn each_profile_has_unique_header_comment() {
    let mut headers = BTreeMap::new();
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        let header = content.lines().next().unwrap().to_string();
        headers.insert(format!("{profile:?}"), header);
    }
    let unique: HashSet<&String> = headers.values().collect();
    assert_eq!(
        unique.len(),
        ALL_PROFILES.len(),
        "Each profile should have a unique header: {headers:#?}"
    );
}

// ── Template content: no contradictions ──────────────────────────────

#[test]
fn no_negation_patterns_in_any_template() {
    // Templates should not use negation (!) patterns since they're
    // meant to be additive excludes.
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        for pat in patterns(&content) {
            assert!(
                !pat.starts_with('!'),
                "{profile:?}: negation pattern found: {pat}"
            );
        }
    }
}

#[test]
fn no_anchored_absolute_paths_in_patterns() {
    // Patterns should be relative, not absolute
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        for pat in patterns(&content) {
            // Skip doublestar patterns
            if pat.starts_with("**/") {
                continue;
            }
            assert!(
                !pat.starts_with('/'),
                "{profile:?}: absolute pattern found: {pat}"
            );
        }
    }
}

// ── Init logic edge cases ───────────────────────────────────────────

#[test]
fn init_all_profiles_write_non_empty_files() {
    for profile in ALL_PROFILES {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(profile, false, false, dir.path().to_path_buf());
        init_tokeignore(&args).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        assert!(!content.is_empty(), "{profile:?} wrote empty file");
    }
}

#[test]
fn init_force_all_profiles_cycle() {
    // Write each profile in sequence using force, verify content updates
    let dir = tempfile::tempdir().unwrap();
    let mut prev_content = String::new();

    for profile in ALL_PROFILES {
        let args = make_args(profile, false, true, dir.path().to_path_buf());
        init_tokeignore(&args).unwrap();
        let content = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
        if !prev_content.is_empty() {
            assert_ne!(
                content, prev_content,
                "Force overwrite with {profile:?} should differ from previous"
            );
        }
        prev_content = content;
    }
}

#[test]
fn init_returns_path_ending_in_tokeignore() {
    for profile in ALL_PROFILES {
        let dir = tempfile::tempdir().unwrap();
        let args = make_args(profile, false, false, dir.path().to_path_buf());
        let result = init_tokeignore(&args).unwrap().unwrap();
        assert!(
            result.ends_with(".tokeignore"),
            "{profile:?}: path should end with .tokeignore: {}",
            result.display()
        );
    }
}

#[test]
fn init_print_then_write_does_not_conflict() {
    let dir = tempfile::tempdir().unwrap();
    // Print first (no file created)
    let args = make_args(InitProfile::Rust, true, false, dir.path().to_path_buf());
    init_tokeignore(&args).unwrap();
    assert!(!dir.path().join(".tokeignore").exists());

    // Then write (should succeed without force)
    let args = make_args(InitProfile::Rust, false, false, dir.path().to_path_buf());
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_some());
    assert!(dir.path().join(".tokeignore").exists());
}

#[test]
fn init_error_message_includes_path() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".tokeignore"), "existing").unwrap();
    let args = make_args(InitProfile::Default, false, false, dir.path().to_path_buf());
    let err = init_tokeignore(&args).unwrap_err().to_string();
    assert!(
        err.contains(".tokeignore"),
        "Error should mention the file path: {err}"
    );
}

// ── Template content: line structure ────────────────────────────────

#[test]
fn comment_sections_separate_pattern_groups() {
    // Templates with multiple ecosystems should have section header comments
    // (lines starting with # that are not the first-line header)
    for profile in [InitProfile::Default, InitProfile::Mono] {
        let content = get_content(profile);
        let section_comments: Vec<&str> = content
            .lines()
            .skip(3) // skip header lines
            .filter(|l| {
                let t = l.trim();
                t.starts_with('#') && !t.is_empty()
            })
            .collect();
        assert!(
            !section_comments.is_empty(),
            "{profile:?}: should have section comment separators"
        );
    }
}

#[test]
fn all_templates_are_valid_utf8_and_ascii() {
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        // Templates should be pure ASCII for maximum compatibility
        for (i, line) in content.lines().enumerate() {
            assert!(
                line.is_ascii(),
                "{profile:?}, line {}: non-ASCII content: {line}",
                i + 1
            );
        }
    }
}

#[test]
fn all_templates_have_no_windows_line_endings() {
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        assert!(
            !content.contains('\r'),
            "{profile:?}: contains \\r (Windows line ending)"
        );
    }
}

// ── Template consistency: paired patterns ───────────────────────────

#[test]
fn bare_dir_patterns_have_recursive_counterparts() {
    // For each "foo/" pattern, there should be a "**/foo/" pattern (for dir patterns
    // that don't contain wildcards)
    for profile in ALL_PROFILES {
        let content = get_content(profile);
        let pats: Vec<&str> = patterns(&content);
        for pat in &pats {
            if pat.ends_with('/')
                && !pat.starts_with("**/")
                && !pat.contains('*')
                && !pat.contains('/')
            {
                // pat is like "target/" — should have "**/target/"
                // Only enforce for single-segment bare directory patterns
                let recursive = format!("**/{pat}");
                assert!(
                    pats.contains(&recursive.as_str()),
                    "{profile:?}: bare pattern {pat} missing recursive counterpart {recursive}"
                );
            }
        }
    }
}

// ── Language-specific pattern exclusivity ────────────────────────────

#[test]
fn rust_has_no_python_or_node_specific() {
    let content = get_content(InitProfile::Rust);
    assert!(!content.contains("node_modules"));
    assert!(!content.contains("__pycache__"));
    assert!(!content.contains("*.pyc"));
    assert!(!content.contains("cmake-build"));
    assert!(!content.contains("vendor/"));
}

#[test]
fn node_has_no_rust_or_python_specific() {
    let content = get_content(InitProfile::Node);
    assert!(!content.contains("target/"));
    assert!(!content.contains(".rs.bk"));
    assert!(!content.contains("__pycache__"));
    assert!(!content.contains("*.pyc"));
    assert!(!content.contains("cmake-build"));
}

#[test]
fn python_has_no_rust_or_node_specific() {
    let content = get_content(InitProfile::Python);
    assert!(!content.contains("target/"));
    assert!(!content.contains(".rs.bk"));
    assert!(!content.contains("node_modules"));
    assert!(!content.contains("cmake-build"));
}

#[test]
fn go_has_no_rust_node_python_cpp_specific() {
    let content = get_content(InitProfile::Go);
    assert!(!content.contains("target/"));
    assert!(!content.contains("node_modules"));
    assert!(!content.contains("__pycache__"));
    assert!(!content.contains("cmake-build"));
    assert!(!content.contains(".rs.bk"));
}

#[test]
fn cpp_has_no_rust_node_python_go_specific() {
    let content = get_content(InitProfile::Cpp);
    assert!(!content.contains("target/"));
    assert!(!content.contains("node_modules"));
    assert!(!content.contains("__pycache__"));
    assert!(!content.contains("vendor/"));
    assert!(!content.contains(".rs.bk"));
}
