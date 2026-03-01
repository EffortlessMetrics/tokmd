//! Deeper tests for .tokeignore template generation.

use std::path::PathBuf;

use tempfile::TempDir;
use tokmd_config::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_args(dir: PathBuf, template: InitProfile, force: bool, print: bool) -> InitArgs {
    InitArgs {
        dir,
        template,
        force,
        print,
        non_interactive: true,
    }
}

fn read_template(profile: InitProfile) -> String {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), profile, false, false);
    init_tokeignore(&args).unwrap();
    std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap()
}

const ALL_PROFILES: [InitProfile; 7] = [
    InitProfile::Default,
    InitProfile::Rust,
    InitProfile::Node,
    InitProfile::Mono,
    InitProfile::Python,
    InitProfile::Go,
    InitProfile::Cpp,
];

// ---------------------------------------------------------------------------
// Template generation includes common ignore patterns
// ---------------------------------------------------------------------------

#[test]
fn default_template_includes_common_patterns() {
    let content = read_template(InitProfile::Default);
    let expected = [
        "target/",
        "node_modules/",
        "__pycache__/",
        "vendor/",
        "build/",
        "dist/",
        "coverage/",
        ".runs/",
        "generated/",
        "third_party/",
    ];
    for pattern in expected {
        assert!(
            content.contains(pattern),
            "Default template missing pattern: {pattern}"
        );
    }
}

#[test]
fn every_template_includes_runs_dir() {
    for profile in ALL_PROFILES {
        let content = read_template(profile);
        assert!(
            content.contains(".runs/"),
            "{profile:?} template missing .runs/"
        );
    }
}

#[test]
fn every_template_starts_with_comment_header() {
    for profile in ALL_PROFILES {
        let content = read_template(profile);
        assert!(
            content.starts_with('#'),
            "{profile:?} template should start with a comment header"
        );
    }
}

#[test]
fn every_template_is_non_empty() {
    for profile in ALL_PROFILES {
        let content = read_template(profile);
        assert!(!content.is_empty(), "{profile:?} template is empty");
        let pattern_lines: Vec<&str> = content
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#')
            })
            .collect();
        assert!(
            !pattern_lines.is_empty(),
            "{profile:?} has no actual patterns"
        );
    }
}

// ---------------------------------------------------------------------------
// Template is valid gitignore syntax
// ---------------------------------------------------------------------------

#[test]
fn no_tabs_in_any_template() {
    for profile in ALL_PROFILES {
        let content = read_template(profile);
        for (i, line) in content.lines().enumerate() {
            assert!(
                !line.contains('\t'),
                "{profile:?} line {}: contains tab: {line}",
                i + 1
            );
        }
    }
}

#[test]
fn no_trailing_whitespace_in_pattern_lines() {
    for profile in ALL_PROFILES {
        let content = read_template(profile);
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            assert_eq!(
                line.trim_end(),
                line,
                "{profile:?} line {}: trailing whitespace in pattern",
                i + 1
            );
        }
    }
}

#[test]
fn patterns_use_forward_slashes() {
    for profile in ALL_PROFILES {
        let content = read_template(profile);
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            assert!(
                !trimmed.contains('\\'),
                "{profile:?}: pattern uses backslash: {trimmed}"
            );
        }
    }
}

#[test]
fn no_duplicate_patterns_in_any_template() {
    for profile in ALL_PROFILES {
        let content = read_template(profile);
        let patterns: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        let mut seen = std::collections::HashSet::new();
        for p in &patterns {
            assert!(seen.insert(*p), "{profile:?}: duplicate pattern: {p}");
        }
    }
}

// ---------------------------------------------------------------------------
// Determinism: same config → same template
// ---------------------------------------------------------------------------

#[test]
fn same_profile_produces_identical_output() {
    for profile in ALL_PROFILES {
        let content1 = read_template(profile);
        let content2 = read_template(profile);
        assert_eq!(
            content1, content2,
            "{profile:?} template is not deterministic"
        );
    }
}

#[test]
fn force_rewrite_is_identical_to_fresh_write() {
    for profile in ALL_PROFILES {
        let temp = TempDir::new().unwrap();

        // Fresh write
        let args = make_args(temp.path().to_path_buf(), profile, false, false);
        init_tokeignore(&args).unwrap();
        let fresh = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();

        // Force overwrite
        let args2 = make_args(temp.path().to_path_buf(), profile, true, false);
        init_tokeignore(&args2).unwrap();
        let overwritten = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();

        assert_eq!(
            fresh, overwritten,
            "{profile:?}: force overwrite differs from fresh write"
        );
    }
}

#[test]
fn different_directories_same_output() {
    for profile in ALL_PROFILES {
        let temp_a = TempDir::new().unwrap();
        let temp_b = TempDir::new().unwrap();

        let args_a = make_args(temp_a.path().to_path_buf(), profile, false, false);
        let args_b = make_args(temp_b.path().to_path_buf(), profile, false, false);
        init_tokeignore(&args_a).unwrap();
        init_tokeignore(&args_b).unwrap();

        let content_a = std::fs::read_to_string(temp_a.path().join(".tokeignore")).unwrap();
        let content_b = std::fs::read_to_string(temp_b.path().join(".tokeignore")).unwrap();
        assert_eq!(
            content_a, content_b,
            "{profile:?}: different dirs produced different content"
        );
    }
}

// ---------------------------------------------------------------------------
// Language-specific patterns
// ---------------------------------------------------------------------------

#[test]
fn rust_profile_has_rust_specific_patterns() {
    let content = read_template(InitProfile::Rust);
    assert!(content.contains("target/"));
    assert!(content.contains("*.rs.bk"));
    // Should NOT contain unrelated language patterns
    assert!(!content.contains("node_modules/"));
    assert!(!content.contains("__pycache__/"));
}

#[test]
fn node_profile_has_node_specific_patterns() {
    let content = read_template(InitProfile::Node);
    assert!(content.contains("node_modules/"));
    assert!(content.contains("dist/"));
    assert!(content.contains("out/"));
    // Should NOT contain Rust-specific patterns
    assert!(!content.contains("target/"));
    assert!(!content.contains("*.rs.bk"));
}

#[test]
fn python_profile_has_python_specific_patterns() {
    let content = read_template(InitProfile::Python);
    assert!(content.contains("__pycache__/"));
    assert!(content.contains(".venv/"));
    assert!(content.contains("*.pyc"));
    assert!(content.contains(".tox/"));
    assert!(content.contains(".pytest_cache/"));
}

#[test]
fn go_profile_has_go_specific_patterns() {
    let content = read_template(InitProfile::Go);
    assert!(content.contains("vendor/"));
    assert!(content.contains("bin/"));
}

#[test]
fn cpp_profile_has_cpp_specific_patterns() {
    let content = read_template(InitProfile::Cpp);
    assert!(content.contains("build/"));
    assert!(content.contains("cmake-build-*/"));
    assert!(content.contains(".cache/"));
}

#[test]
fn mono_profile_is_superset_of_language_patterns() {
    let mono = read_template(InitProfile::Mono);
    // Mono should include patterns from multiple language ecosystems
    assert!(mono.contains("target/"), "mono missing Rust target/");
    assert!(mono.contains("node_modules/"), "mono missing node_modules/");
    assert!(mono.contains("__pycache__/"), "mono missing __pycache__/");
    assert!(mono.contains("vendor/"), "mono missing vendor/");
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn init_returns_path_on_success() {
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
fn print_mode_returns_none() {
    let temp = TempDir::new().unwrap();
    let args = make_args(temp.path().to_path_buf(), InitProfile::Default, false, true);
    let result = init_tokeignore(&args).unwrap();
    assert!(result.is_none());
}

#[test]
fn error_on_nonexistent_directory() {
    let temp = TempDir::new().unwrap();
    let bad_path = temp.path().join("nonexistent");
    let args = make_args(bad_path, InitProfile::Default, false, false);
    let result = init_tokeignore(&args);
    assert!(result.is_err());
}
