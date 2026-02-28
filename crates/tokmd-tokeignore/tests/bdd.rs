//! BDD-style scenarios for tokmd-tokeignore template generation.
//!
//! Organized as Given-When-Then scenarios covering all template types,
//! edge cases, and behavioral contracts.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_config::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ============================================================================
// Helpers
// ============================================================================

fn make_args(dir: PathBuf, template: InitProfile, force: bool, print: bool) -> InitArgs {
    InitArgs {
        dir,
        template,
        force,
        print,
        non_interactive: true,
    }
}

fn write_and_read(profile: InitProfile) -> String {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), profile, false, false);
    init_tokeignore(&args).unwrap();
    std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap()
}

// ============================================================================
// Scenario: Default template covers all ecosystems
// ============================================================================

mod default_template {
    use super::*;

    #[test]
    fn given_default_profile_then_rust_patterns_present() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.contains("target/"));
        assert!(content.contains("**/target/"));
    }

    #[test]
    fn given_default_profile_then_node_patterns_present() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.contains("node_modules/"));
        assert!(content.contains("**/node_modules/"));
        assert!(content.contains("dist/"));
    }

    #[test]
    fn given_default_profile_then_python_patterns_present() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.contains("__pycache__/"));
        assert!(content.contains("**/__pycache__/"));
        assert!(content.contains(".venv/"));
    }

    #[test]
    fn given_default_profile_then_vendored_patterns_present() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.contains("vendor/"));
        assert!(content.contains("third_party/"));
        assert!(content.contains("external/"));
    }

    #[test]
    fn given_default_profile_then_generated_code_patterns_present() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.contains("generated/"));
        assert!(content.contains("*.generated.*"));
        assert!(content.contains("*.gen.*"));
    }

    #[test]
    fn given_default_profile_then_coverage_patterns_present() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.contains("coverage/"));
        assert!(content.contains(".coverage"));
        assert!(content.contains("lcov.info"));
    }

    #[test]
    fn given_default_profile_then_tree_sitter_patterns_present() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.contains("tree-sitter"));
        assert!(content.contains("parser.c"));
        assert!(content.contains("scanner.c"));
    }

    #[test]
    fn given_default_profile_then_header_identifies_template() {
        let content = write_and_read(InitProfile::Default);
        assert!(content.starts_with("# .tokeignore\n"));
    }
}

// ============================================================================
// Scenario: Rust template is Rust-focused
// ============================================================================

mod rust_template {
    use super::*;

    #[test]
    fn given_rust_profile_then_target_and_backup_present() {
        let content = write_and_read(InitProfile::Rust);
        assert!(content.contains("target/"));
        assert!(content.contains("**/target/"));
        assert!(content.contains("**/*.rs.bk"));
    }

    #[test]
    fn given_rust_profile_then_header_identifies_rust() {
        let content = write_and_read(InitProfile::Rust);
        assert!(content.starts_with("# .tokeignore (Rust)"));
    }

    #[test]
    fn given_rust_profile_then_no_node_or_python_patterns() {
        let content = write_and_read(InitProfile::Rust);
        assert!(!content.contains("node_modules/"));
        assert!(!content.contains("__pycache__/"));
        assert!(!content.contains("*.pyc"));
        assert!(!content.contains("vendor/"));
    }

    #[test]
    fn given_rust_profile_then_coverage_present() {
        let content = write_and_read(InitProfile::Rust);
        assert!(content.contains("coverage/"));
    }
}

// ============================================================================
// Scenario: Node template is JS-focused
// ============================================================================

mod node_template {
    use super::*;

    #[test]
    fn given_node_profile_then_js_build_dirs_present() {
        let content = write_and_read(InitProfile::Node);
        assert!(content.contains("node_modules/"));
        assert!(content.contains("dist/"));
        assert!(content.contains("out/"));
        assert!(content.contains("build/"));
    }

    #[test]
    fn given_node_profile_then_header_identifies_node() {
        let content = write_and_read(InitProfile::Node);
        assert!(content.starts_with("# .tokeignore (Node)"));
    }

    #[test]
    fn given_node_profile_then_no_rust_or_python_patterns() {
        let content = write_and_read(InitProfile::Node);
        assert!(!content.contains("target/"));
        assert!(!content.contains("__pycache__/"));
        assert!(!content.contains("*.pyc"));
        assert!(!content.contains(".rs.bk"));
    }
}

// ============================================================================
// Scenario: Mono template covers all ecosystems
// ============================================================================

mod mono_template {
    use super::*;

    #[test]
    fn given_mono_profile_then_rust_node_python_patterns_present() {
        let content = write_and_read(InitProfile::Mono);
        assert!(content.contains("target/"));
        assert!(content.contains("node_modules/"));
        assert!(content.contains("__pycache__/"));
    }

    #[test]
    fn given_mono_profile_then_vendored_dirs_present() {
        let content = write_and_read(InitProfile::Mono);
        assert!(content.contains("vendor/"));
        assert!(content.contains("third_party/"));
        assert!(content.contains("external/"));
    }

    #[test]
    fn given_mono_profile_then_generated_code_patterns_present() {
        let content = write_and_read(InitProfile::Mono);
        assert!(content.contains("generated/"));
        assert!(content.contains("*.generated.*"));
        assert!(content.contains("*.gen.*"));
    }

    #[test]
    fn given_mono_profile_then_tree_sitter_patterns_present() {
        let content = write_and_read(InitProfile::Mono);
        assert!(content.contains("tree-sitter"));
    }

    #[test]
    fn given_mono_profile_then_header_identifies_monorepo() {
        let content = write_and_read(InitProfile::Mono);
        assert!(content.starts_with("# .tokeignore (Monorepo)"));
    }
}

// ============================================================================
// Scenario: Python template is Python-focused
// ============================================================================

mod python_template {
    use super::*;

    #[test]
    fn given_python_profile_then_python_patterns_present() {
        let content = write_and_read(InitProfile::Python);
        assert!(content.contains("__pycache__/"));
        assert!(content.contains("*.pyc"));
        assert!(content.contains(".venv/"));
        assert!(content.contains("venv/"));
        assert!(content.contains(".tox/"));
    }

    #[test]
    fn given_python_profile_then_test_artifacts_present() {
        let content = write_and_read(InitProfile::Python);
        assert!(content.contains(".pytest_cache/"));
        assert!(content.contains("htmlcov/"));
        assert!(content.contains(".coverage"));
    }

    #[test]
    fn given_python_profile_then_header_identifies_python() {
        let content = write_and_read(InitProfile::Python);
        assert!(content.starts_with("# .tokeignore (Python)"));
    }

    #[test]
    fn given_python_profile_then_no_rust_or_node_patterns() {
        let content = write_and_read(InitProfile::Python);
        assert!(!content.contains("target/"));
        assert!(!content.contains("node_modules/"));
        assert!(!content.contains(".rs.bk"));
    }
}

// ============================================================================
// Scenario: Go template is Go-focused
// ============================================================================

mod go_template {
    use super::*;

    #[test]
    fn given_go_profile_then_go_patterns_present() {
        let content = write_and_read(InitProfile::Go);
        assert!(content.contains("vendor/"));
        assert!(content.contains("**/vendor/"));
        assert!(content.contains("bin/"));
        assert!(content.contains("**/bin/"));
    }

    #[test]
    fn given_go_profile_then_header_identifies_go() {
        let content = write_and_read(InitProfile::Go);
        assert!(content.starts_with("# .tokeignore (Go)"));
    }

    #[test]
    fn given_go_profile_then_no_unrelated_patterns() {
        let content = write_and_read(InitProfile::Go);
        assert!(!content.contains("target/"));
        assert!(!content.contains("node_modules/"));
        assert!(!content.contains("__pycache__/"));
        assert!(!content.contains("build/"));
    }

    #[test]
    fn given_go_profile_then_minimal_pattern_count() {
        let content = write_and_read(InitProfile::Go);
        let patterns: Vec<&str> = content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .collect();
        // Go is the most minimal profile
        assert!(
            patterns.len() <= 6,
            "Go should be minimal, found {} patterns",
            patterns.len()
        );
    }
}

// ============================================================================
// Scenario: C++ template is C++-focused
// ============================================================================

mod cpp_template {
    use super::*;

    #[test]
    fn given_cpp_profile_then_cpp_patterns_present() {
        let content = write_and_read(InitProfile::Cpp);
        assert!(content.contains("build/"));
        assert!(content.contains("cmake-build-*/"));
        assert!(content.contains("out/"));
        assert!(content.contains(".cache/"));
    }

    #[test]
    fn given_cpp_profile_then_header_identifies_cpp() {
        let content = write_and_read(InitProfile::Cpp);
        assert!(content.starts_with("# .tokeignore (C++)"));
    }

    #[test]
    fn given_cpp_profile_then_no_unrelated_patterns() {
        let content = write_and_read(InitProfile::Cpp);
        assert!(!content.contains("target/"));
        assert!(!content.contains("node_modules/"));
        assert!(!content.contains("__pycache__/"));
        assert!(!content.contains("vendor/"));
    }
}

// ============================================================================
// Scenario: Return value contract
// ============================================================================

mod return_value {
    use super::*;

    #[test]
    fn given_write_mode_then_returns_some_path() {
        let temp = TempDir::new().unwrap();
        let args = make_args(
            temp.path().to_path_buf(),
            InitProfile::Default,
            false,
            false,
        );
        let result = init_tokeignore(&args).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), temp.path().join(".tokeignore"));
    }

    #[test]
    fn given_print_mode_then_returns_none() {
        let temp = TempDir::new().unwrap();
        let args = make_args(temp.path().to_path_buf(), InitProfile::Default, false, true);
        let result = init_tokeignore(&args).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn given_force_write_then_returns_some_path() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".tokeignore"), "old").unwrap();
        let args = make_args(temp.path().to_path_buf(), InitProfile::Rust, true, false);
        let result = init_tokeignore(&args).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), temp.path().join(".tokeignore"));
    }

    #[test]
    fn given_all_profiles_in_write_mode_then_all_return_some() {
        for profile in [
            InitProfile::Default,
            InitProfile::Rust,
            InitProfile::Node,
            InitProfile::Mono,
            InitProfile::Python,
            InitProfile::Go,
            InitProfile::Cpp,
        ] {
            let temp = TempDir::new().unwrap();
            let args = make_args(temp.path().to_path_buf(), profile, false, false);
            let result = init_tokeignore(&args).unwrap();
            assert!(
                result.is_some(),
                "Profile {:?} should return Some(path)",
                profile
            );
        }
    }
}

// ============================================================================
// Scenario: Error paths
// ============================================================================

mod errors {
    use super::*;

    #[test]
    fn given_nonexistent_dir_then_error_contains_does_not_exist() {
        let args = make_args(
            PathBuf::from("__this_dir_does_not_exist_99__"),
            InitProfile::Default,
            false,
            false,
        );
        let err = init_tokeignore(&args).unwrap_err().to_string();
        assert!(
            err.contains("does not exist"),
            "Expected 'does not exist' in: {err}"
        );
    }

    #[test]
    fn given_file_exists_no_force_then_error_contains_already_exists() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".tokeignore"), "x").unwrap();
        let args = make_args(
            temp.path().to_path_buf(),
            InitProfile::Default,
            false,
            false,
        );
        let err = init_tokeignore(&args).unwrap_err().to_string();
        assert!(
            err.contains("already exists"),
            "Expected 'already exists' in: {err}"
        );
    }

    #[test]
    fn given_file_exists_no_force_then_error_suggests_force_flag() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".tokeignore"), "x").unwrap();
        let args = make_args(
            temp.path().to_path_buf(),
            InitProfile::Default,
            false,
            false,
        );
        let err = init_tokeignore(&args).unwrap_err().to_string();
        assert!(
            err.contains("--force"),
            "Error should mention --force: {err}"
        );
    }

    #[test]
    fn given_file_exists_no_force_then_error_suggests_print_flag() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".tokeignore"), "x").unwrap();
        let args = make_args(
            temp.path().to_path_buf(),
            InitProfile::Default,
            false,
            false,
        );
        let err = init_tokeignore(&args).unwrap_err().to_string();
        assert!(
            err.contains("--print"),
            "Error should mention --print: {err}"
        );
    }

    #[test]
    fn given_nonexistent_dir_all_profiles_then_all_fail() {
        for profile in [
            InitProfile::Default,
            InitProfile::Rust,
            InitProfile::Node,
            InitProfile::Mono,
            InitProfile::Python,
            InitProfile::Go,
            InitProfile::Cpp,
        ] {
            let args = make_args(PathBuf::from("__nonexistent_dir__"), profile, false, false);
            assert!(
                init_tokeignore(&args).is_err(),
                "Profile {:?} should fail for nonexistent dir",
                profile
            );
        }
    }

    #[test]
    fn given_nonexistent_dir_with_force_then_still_fails() {
        let args = make_args(
            PathBuf::from("__nonexistent_force__"),
            InitProfile::Default,
            true,
            false,
        );
        assert!(
            init_tokeignore(&args).is_err(),
            "Force should not bypass directory existence check"
        );
    }
}

// ============================================================================
// Scenario: Force overwrite behavior
// ============================================================================

mod force_overwrite {
    use super::*;

    #[test]
    fn given_force_then_old_content_replaced() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join(".tokeignore"),
            "# old custom content\nmy_pattern/\n",
        )
        .unwrap();
        let args = make_args(temp.path().to_path_buf(), InitProfile::Default, true, false);
        init_tokeignore(&args).unwrap();
        let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
        assert!(!content.contains("my_pattern/"));
        assert!(content.contains("target/"));
    }

    #[test]
    fn given_force_with_each_profile_then_content_matches_profile() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join(".tokeignore"), "placeholder").unwrap();

        for profile in [
            InitProfile::Default,
            InitProfile::Rust,
            InitProfile::Node,
            InitProfile::Mono,
            InitProfile::Python,
            InitProfile::Go,
            InitProfile::Cpp,
        ] {
            let args = make_args(temp.path().to_path_buf(), profile, true, false);
            init_tokeignore(&args).unwrap();
            let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
            assert!(
                content.contains(".runs/"),
                "Profile {:?} should contain .runs/",
                profile
            );
            assert!(!content.contains("placeholder"));
        }
    }

    #[test]
    fn given_force_switching_profiles_then_content_updates() {
        let temp = TempDir::new().unwrap();

        // Write Rust template
        let args = make_args(temp.path().to_path_buf(), InitProfile::Rust, false, false);
        init_tokeignore(&args).unwrap();
        let rust_content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
        assert!(rust_content.contains(".rs.bk"));

        // Force-overwrite with Node template
        let args = make_args(temp.path().to_path_buf(), InitProfile::Node, true, false);
        init_tokeignore(&args).unwrap();
        let node_content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
        assert!(node_content.contains("node_modules/"));
        assert!(!node_content.contains(".rs.bk"));
    }
}

// ============================================================================
// Scenario: Print mode isolation
// ============================================================================

mod print_mode {
    use super::*;

    #[test]
    fn given_print_mode_then_no_file_side_effects() {
        let temp = TempDir::new().unwrap();
        let before: Vec<_> = std::fs::read_dir(temp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(before.is_empty());

        let args = make_args(temp.path().to_path_buf(), InitProfile::Default, false, true);
        init_tokeignore(&args).unwrap();

        let after: Vec<_> = std::fs::read_dir(temp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(after.is_empty(), "Print mode should create no files");
    }

    #[test]
    fn given_print_mode_with_existing_file_then_file_unchanged() {
        let temp = TempDir::new().unwrap();
        let original = "# my custom tokeignore\nmy_dir/\n";
        std::fs::write(temp.path().join(".tokeignore"), original).unwrap();

        let args = make_args(temp.path().to_path_buf(), InitProfile::Rust, false, true);
        init_tokeignore(&args).unwrap();

        let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
        assert_eq!(
            content, original,
            "Print mode should not modify existing file"
        );
    }

    #[test]
    fn given_print_and_force_then_no_file_written() {
        let temp = TempDir::new().unwrap();
        let args = make_args(temp.path().to_path_buf(), InitProfile::Default, true, true);
        let result = init_tokeignore(&args).unwrap();
        assert!(result.is_none());
        assert!(!temp.path().join(".tokeignore").exists());
    }
}

// ============================================================================
// Scenario: Subdirectory targeting
// ============================================================================

mod subdirectory {
    use super::*;

    #[test]
    fn given_nested_subdir_then_file_written_in_correct_location() {
        let temp = TempDir::new().unwrap();
        let nested = temp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();

        let args = make_args(nested.clone(), InitProfile::Default, false, false);
        let result = init_tokeignore(&args).unwrap();

        assert_eq!(result.unwrap(), nested.join(".tokeignore"));
        assert!(nested.join(".tokeignore").exists());
        assert!(!temp.path().join(".tokeignore").exists());
        assert!(!temp.path().join("a").join(".tokeignore").exists());
    }

    #[test]
    fn given_two_subdirs_then_independent_files() {
        let temp = TempDir::new().unwrap();
        let dir_a = temp.path().join("project_a");
        let dir_b = temp.path().join("project_b");
        std::fs::create_dir_all(&dir_a).unwrap();
        std::fs::create_dir_all(&dir_b).unwrap();

        let args_a = make_args(dir_a.clone(), InitProfile::Rust, false, false);
        let args_b = make_args(dir_b.clone(), InitProfile::Node, false, false);
        init_tokeignore(&args_a).unwrap();
        init_tokeignore(&args_b).unwrap();

        let content_a = std::fs::read_to_string(dir_a.join(".tokeignore")).unwrap();
        let content_b = std::fs::read_to_string(dir_b.join(".tokeignore")).unwrap();
        assert!(content_a.contains("target/"));
        assert!(content_b.contains("node_modules/"));
        assert_ne!(content_a, content_b);
    }
}

// ============================================================================
// Scenario: Template structure invariants
// ============================================================================

mod template_structure {
    use super::*;

    const ALL_PROFILES: [InitProfile; 7] = [
        InitProfile::Default,
        InitProfile::Rust,
        InitProfile::Node,
        InitProfile::Mono,
        InitProfile::Python,
        InitProfile::Go,
        InitProfile::Cpp,
    ];

    #[test]
    fn all_templates_end_with_newline() {
        for profile in ALL_PROFILES {
            let content = write_and_read(profile);
            assert!(
                content.ends_with('\n'),
                "Template for {:?} should end with newline",
                profile
            );
        }
    }

    #[test]
    fn all_templates_contain_only_lf_line_endings() {
        for profile in ALL_PROFILES {
            let content = write_and_read(profile);
            assert!(
                !content.contains("\r\n"),
                "Template for {:?} should use LF, not CRLF",
                profile
            );
        }
    }

    #[test]
    fn all_templates_have_no_trailing_whitespace_on_lines() {
        for profile in ALL_PROFILES {
            let content = write_and_read(profile);
            for (i, line) in content.lines().enumerate() {
                assert!(
                    line == line.trim_end(),
                    "Template {:?}, line {}: trailing whitespace in {:?}",
                    profile,
                    i + 1,
                    line,
                );
            }
        }
    }

    #[test]
    fn all_templates_have_no_empty_pattern_lines() {
        // A pattern line is non-comment, non-blank; it should have real content
        for profile in ALL_PROFILES {
            let content = write_and_read(profile);
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                assert!(
                    trimmed.len() >= 2,
                    "Template {:?}: pattern too short: {:?}",
                    profile,
                    trimmed
                );
            }
        }
    }

    #[test]
    fn all_templates_have_some_glob_double_star_patterns() {
        // Templates should use **/ patterns for at least some directories
        for profile in ALL_PROFILES {
            let content = write_and_read(profile);
            let doublestar_count = content
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#') && l.starts_with("**/"))
                .count();
            assert!(
                doublestar_count >= 1,
                "Template {:?} should have at least one **/ pattern, got {}",
                profile,
                doublestar_count,
            );
        }
    }

    #[test]
    fn all_templates_have_at_least_two_patterns() {
        for profile in ALL_PROFILES {
            let content = write_and_read(profile);
            let count = content
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    !t.is_empty() && !t.starts_with('#')
                })
                .count();
            assert!(
                count >= 2,
                "Template {:?} should have >= 2 patterns, got {}",
                profile,
                count
            );
        }
    }

    #[test]
    fn no_duplicate_patterns_in_any_template() {
        use std::collections::HashSet;
        for profile in ALL_PROFILES {
            let content = write_and_read(profile);
            let patterns: Vec<&str> = content
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
            let unique: HashSet<&str> = patterns.iter().copied().collect();
            assert_eq!(
                patterns.len(),
                unique.len(),
                "Template {:?} has duplicate patterns",
                profile
            );
        }
    }
}

// ============================================================================
// Scenario: Default and Mono are supersets of specific profiles (patterns)
// ============================================================================

mod superset_relationships {
    use super::*;

    fn pattern_set(content: &str) -> std::collections::HashSet<String> {
        content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    }

    #[test]
    fn mono_is_superset_of_rust_core_patterns() {
        let mono = pattern_set(&write_and_read(InitProfile::Mono));
        // Rust core patterns that Mono should have
        assert!(mono.contains("target/"));
        assert!(mono.contains("**/target/"));
    }

    #[test]
    fn mono_is_superset_of_node_core_patterns() {
        let mono = pattern_set(&write_and_read(InitProfile::Mono));
        assert!(mono.contains("node_modules/"));
        assert!(mono.contains("**/node_modules/"));
        assert!(mono.contains("dist/"));
    }

    #[test]
    fn mono_is_superset_of_python_core_patterns() {
        let mono = pattern_set(&write_and_read(InitProfile::Mono));
        assert!(mono.contains("__pycache__/"));
        assert!(mono.contains("**/__pycache__/"));
        assert!(mono.contains(".venv/"));
    }

    #[test]
    fn default_and_mono_share_vendored_patterns() {
        let default_patterns = pattern_set(&write_and_read(InitProfile::Default));
        let mono_patterns = pattern_set(&write_and_read(InitProfile::Mono));
        for p in ["vendor/", "**/vendor/", "third_party/", "**/third_party/"] {
            assert!(default_patterns.contains(p), "Default missing {p}");
            assert!(mono_patterns.contains(p), "Mono missing {p}");
        }
    }
}
