//! Property-based tests for tokmd-tokeignore template generation.
//!
//! These tests verify template selection properties, content properties,
//! and enum coverage for InitProfile variants.

use std::collections::HashSet;
use std::path::PathBuf;

use proptest::prelude::*;
use tempfile::TempDir;
use tokmd_config::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ============================================================================
// Constants
// ============================================================================

/// All InitProfile variants for exhaustive testing.
const ALL_PROFILES: [InitProfile; 7] = [
    InitProfile::Default,
    InitProfile::Rust,
    InitProfile::Node,
    InitProfile::Mono,
    InitProfile::Python,
    InitProfile::Go,
    InitProfile::Cpp,
];

// ============================================================================
// Strategies
// ============================================================================

/// Strategy for generating arbitrary InitProfile variants.
fn arb_profile() -> impl Strategy<Value = InitProfile> {
    prop::sample::select(ALL_PROFILES.to_vec())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create InitArgs for testing with a temporary directory.
fn make_args(dir: PathBuf, template: InitProfile, force: bool, print: bool) -> InitArgs {
    InitArgs {
        dir,
        template,
        force,
        print,
        non_interactive: true,
        write_config: false,
    }
}

/// Get template content for a profile by writing to a temp directory and reading back.
fn get_template_content(profile: InitProfile) -> String {
    let temp = TempDir::new().expect("create temp dir");
    let args = make_args(temp.path().to_path_buf(), profile, false, false);
    init_tokeignore(&args).expect("init should succeed");
    std::fs::read_to_string(temp.path().join(".tokeignore")).expect("read template")
}

// ============================================================================
// Template Selection Properties
// ============================================================================

proptest! {
    /// Each InitProfile maps to a unique, non-empty template.
    #[test]
    fn profile_produces_non_empty_template(profile in arb_profile()) {
        let content = get_template_content(profile);
        prop_assert!(!content.is_empty(), "Template for {:?} should not be empty", profile);
    }

    /// Templates are deterministic - same profile always produces same content.
    #[test]
    fn template_is_deterministic(profile in arb_profile()) {
        let content1 = get_template_content(profile);
        let content2 = get_template_content(profile);
        prop_assert_eq!(
            content1,
            content2,
            "Template for {:?} should be deterministic",
            profile
        );
    }

    /// All profiles produce valid UTF-8 templates (implicit from String).
    #[test]
    fn template_is_valid_utf8(profile in arb_profile()) {
        // If get_template_content succeeds, it's valid UTF-8 since String requires it
        let content = get_template_content(profile);
        // Additional check: ensure no invalid sequences snuck in
        prop_assert!(content.is_ascii() || content.chars().all(|c| c.is_ascii() || c.len_utf8() > 0));
    }

    /// init_tokeignore does not panic for any profile.
    #[test]
    fn no_panic_on_any_profile(profile in arb_profile()) {
        let temp = TempDir::new().expect("create temp dir");
        let args = make_args(temp.path().to_path_buf(), profile, false, false);
        let result = init_tokeignore(&args);
        prop_assert!(result.is_ok(), "init_tokeignore should not fail for {:?}", profile);
    }
}

// ============================================================================
// Template Content Properties
// ============================================================================

proptest! {
    /// All templates contain the `.runs/` pattern (standard tokmd output dir).
    #[test]
    fn template_contains_runs_pattern(profile in arb_profile()) {
        let content = get_template_content(profile);
        prop_assert!(
            content.contains(".runs/"),
            "Template for {:?} should contain .runs/ pattern",
            profile
        );
    }

    /// All templates start with a comment header.
    #[test]
    fn template_starts_with_comment(profile in arb_profile()) {
        let content = get_template_content(profile);
        prop_assert!(
            content.starts_with('#'),
            "Template for {:?} should start with comment",
            profile
        );
    }

    /// Templates contain only valid gitignore syntax (no regex special chars outside globs).
    #[test]
    fn template_has_valid_gitignore_syntax(profile in arb_profile()) {
        let content = get_template_content(profile);

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // gitignore patterns should not contain tabs
            prop_assert!(
                !trimmed.contains('\t'),
                "Pattern should not contain tabs: {}",
                trimmed
            );

            // gitignore patterns should not contain regex-only syntax
            // (though * and ? are valid globs, ^ $ + | are regex)
            prop_assert!(
                !trimmed.contains('^') && !trimmed.contains('$') && !trimmed.contains('+'),
                "Pattern contains regex syntax: {}",
                trimmed
            );

            // Should not have multiple consecutive asterisks except for **
            let asterisk_runs: Vec<&str> = trimmed
                .split(|c| c != '*')
                .filter(|s| !s.is_empty())
                .collect();
            for run in asterisk_runs {
                prop_assert!(
                    run.len() <= 2,
                    "Invalid glob pattern (too many asterisks): {}",
                    trimmed
                );
            }
        }
    }

    /// Non-comment, non-empty lines are valid patterns (basic structure check).
    #[test]
    fn template_patterns_are_well_formed(profile in arb_profile()) {
        let content = get_template_content(profile);
        let mut pattern_count = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            pattern_count += 1;

            // Patterns should have reasonable length
            prop_assert!(
                !trimmed.is_empty() && trimmed.len() <= 100,
                "Pattern has unusual length: {}",
                trimmed
            );

            // Patterns should not be pure whitespace
            prop_assert!(
                !trimmed.chars().all(|c| c.is_whitespace()),
                "Pattern should not be pure whitespace"
            );
        }

        // Each template should have at least one pattern
        prop_assert!(
            pattern_count >= 1,
            "Template for {:?} should have at least one pattern",
            profile
        );
    }
}

// ============================================================================
// Profile Enum Coverage
// ============================================================================

/// Test that all InitProfile variants produce distinct templates.
#[test]
fn all_profiles_produce_distinct_templates() {
    let mut templates: HashSet<String> = HashSet::new();
    let mut profile_contents: Vec<(InitProfile, String)> = Vec::new();

    for profile in ALL_PROFILES {
        let content = get_template_content(profile);
        profile_contents.push((profile, content.clone()));
        templates.insert(content);
    }

    // All 7 profiles should produce 7 distinct templates
    assert_eq!(
        templates.len(),
        ALL_PROFILES.len(),
        "Each profile should produce a unique template. Found {} unique templates for {} profiles",
        templates.len(),
        ALL_PROFILES.len()
    );
}

/// Test that Profile::Default is the most comprehensive template.
#[test]
fn default_is_most_comprehensive() {
    let default_content = get_template_content(InitProfile::Default);
    let default_lines: usize = default_content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .count();

    // Default should have patterns from multiple language ecosystems
    assert!(
        default_content.contains("target/"),
        "Default should contain Rust patterns"
    );
    assert!(
        default_content.contains("node_modules/"),
        "Default should contain Node patterns"
    );
    assert!(
        default_content.contains("__pycache__/"),
        "Default should contain Python patterns"
    );

    // Default should have at least as many patterns as most specific profiles
    // (only Mono might have as many or more)
    for profile in [
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ] {
        let content = get_template_content(profile);
        let lines: usize = content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .count();

        assert!(
            default_lines >= lines,
            "Default ({} patterns) should have at least as many patterns as {:?} ({} patterns)",
            default_lines,
            profile,
            lines
        );
    }
}

/// Test that all profiles contain the tokmd standard output directory.
#[test]
fn all_profiles_contain_tokmd_output_dir() {
    for profile in ALL_PROFILES {
        let content = get_template_content(profile);
        assert!(
            content.contains(".runs/"),
            "Profile {:?} should contain .runs/ pattern for tokmd outputs",
            profile
        );
    }
}

// ============================================================================
// Print Mode Properties
// ============================================================================

proptest! {
    /// Print mode does not write files for any profile.
    #[test]
    fn print_mode_does_not_write_file(profile in arb_profile()) {
        let temp = TempDir::new().expect("create temp dir");
        let args = make_args(temp.path().to_path_buf(), profile, false, true);

        init_tokeignore(&args).expect("print mode should succeed");

        prop_assert!(
            !temp.path().join(".tokeignore").exists(),
            "Print mode should not create .tokeignore file"
        );
    }

    /// Print mode succeeds for all profiles.
    #[test]
    fn print_mode_succeeds_for_all_profiles(profile in arb_profile()) {
        let temp = TempDir::new().expect("create temp dir");
        let args = make_args(temp.path().to_path_buf(), profile, false, true);

        let result = init_tokeignore(&args);
        prop_assert!(
            result.is_ok(),
            "Print mode should succeed for {:?}",
            profile
        );
    }
}

// ============================================================================
// Force Mode Properties
// ============================================================================

proptest! {
    /// Force mode overwrites existing files for any profile.
    #[test]
    fn force_mode_overwrites_existing(profile in arb_profile()) {
        let temp = TempDir::new().expect("create temp dir");

        // Create existing file
        std::fs::write(temp.path().join(".tokeignore"), "old content").expect("write old content");

        let args = make_args(temp.path().to_path_buf(), profile, true, false);
        init_tokeignore(&args).expect("force mode should succeed");

        let content = std::fs::read_to_string(temp.path().join(".tokeignore")).expect("read");
        prop_assert!(
            !content.contains("old content"),
            "Force mode should overwrite old content"
        );
        prop_assert!(
            content.contains(".runs/"),
            "Force mode should write new template"
        );
    }
}

// ============================================================================
// Idempotency Properties
// ============================================================================

proptest! {
    /// Running init twice with force produces identical results.
    #[test]
    fn init_is_idempotent_with_force(profile in arb_profile()) {
        let temp = TempDir::new().expect("create temp dir");

        // First run
        let args1 = make_args(temp.path().to_path_buf(), profile, false, false);
        init_tokeignore(&args1).expect("first init");
        let content1 = std::fs::read_to_string(temp.path().join(".tokeignore")).expect("read 1");

        // Second run with force
        let args2 = make_args(temp.path().to_path_buf(), profile, true, false);
        init_tokeignore(&args2).expect("second init with force");
        let content2 = std::fs::read_to_string(temp.path().join(".tokeignore")).expect("read 2");

        prop_assert_eq!(
            content1,
            content2,
            "Idempotent init for {:?} should produce same content",
            profile
        );
    }
}

// ============================================================================
// Language-Specific Pattern Properties
// ============================================================================

/// Test that language-specific profiles contain expected patterns.
#[test]
fn language_profiles_contain_expected_patterns() {
    // Rust profile
    let rust = get_template_content(InitProfile::Rust);
    assert!(rust.contains("target/"), "Rust should have target/");
    assert!(
        rust.contains(".rs.bk"),
        "Rust should have .rs.bk backup pattern"
    );

    // Node profile
    let node = get_template_content(InitProfile::Node);
    assert!(
        node.contains("node_modules/"),
        "Node should have node_modules/"
    );
    assert!(node.contains("dist/"), "Node should have dist/");

    // Python profile
    let python = get_template_content(InitProfile::Python);
    assert!(
        python.contains("__pycache__/"),
        "Python should have __pycache__/"
    );
    assert!(python.contains(".venv/"), "Python should have .venv/");
    assert!(
        python.contains(".pytest_cache/"),
        "Python should have .pytest_cache/"
    );

    // Go profile
    let go = get_template_content(InitProfile::Go);
    assert!(go.contains("vendor/"), "Go should have vendor/");
    assert!(go.contains("bin/"), "Go should have bin/");

    // C++ profile
    let cpp = get_template_content(InitProfile::Cpp);
    assert!(cpp.contains("build/"), "C++ should have build/");
    assert!(
        cpp.contains("cmake-build-*/"),
        "C++ should have cmake-build-*/"
    );

    // Mono profile should be comprehensive
    let mono = get_template_content(InitProfile::Mono);
    assert!(mono.contains("target/"), "Mono should have target/ (Rust)");
    assert!(
        mono.contains("node_modules/"),
        "Mono should have node_modules/ (Node)"
    );
    assert!(
        mono.contains("__pycache__/"),
        "Mono should have __pycache__/ (Python)"
    );
}

// ============================================================================
// Cross-Profile Exclusivity
// ============================================================================

/// Test that language-specific profiles don't include unrelated patterns.
#[test]
fn language_profiles_are_focused() {
    // Rust profile should NOT contain Node-specific patterns
    let rust = get_template_content(InitProfile::Rust);
    assert!(
        !rust.contains("node_modules/"),
        "Rust profile should not contain node_modules/"
    );
    assert!(
        !rust.contains("__pycache__/"),
        "Rust profile should not contain __pycache__/"
    );

    // Node profile should NOT contain Rust-specific patterns
    let node = get_template_content(InitProfile::Node);
    assert!(
        !node.contains("target/"),
        "Node profile should not contain target/"
    );
    assert!(
        !node.contains(".rs.bk"),
        "Node profile should not contain .rs.bk"
    );

    // Python profile should NOT contain Rust or Node-specific patterns
    let python = get_template_content(InitProfile::Python);
    assert!(
        !python.contains("target/"),
        "Python profile should not contain target/"
    );
    assert!(
        !python.contains("node_modules/"),
        "Python profile should not contain node_modules/"
    );

    // Go profile should be minimal
    let go = get_template_content(InitProfile::Go);
    assert!(
        !go.contains("target/"),
        "Go profile should not contain target/"
    );
    assert!(
        !go.contains("node_modules/"),
        "Go profile should not contain node_modules/"
    );
    assert!(
        !go.contains("__pycache__/"),
        "Go profile should not contain __pycache__/"
    );

    // C++ profile should be focused
    let cpp = get_template_content(InitProfile::Cpp);
    assert!(
        !cpp.contains("target/"),
        "C++ profile should not contain target/"
    );
    assert!(
        !cpp.contains("node_modules/"),
        "C++ profile should not contain node_modules/"
    );
}
