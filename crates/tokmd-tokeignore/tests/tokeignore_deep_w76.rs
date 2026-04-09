//! Wave-76 deep tests for tokmd-tokeignore: comment formatting, pattern
//! structure, language-specific isolation, default exclusion invariants,
//! and template generation quality.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use tokmd_cli_args::{InitArgs, InitProfile};
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

fn make_args(profile: InitProfile, print: bool, force: bool, dir: PathBuf) -> InitArgs {
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
    init_tokeignore(&make_args(profile, false, false, dir.path().into())).unwrap();
    fs::read_to_string(dir.path().join(".tokeignore")).unwrap()
}

/// Extract non-comment, non-empty lines (actual glob patterns).
fn pattern_lines(content: &str) -> Vec<&str> {
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .collect()
}

/// Extract comment lines (lines starting with #).
fn comment_lines(content: &str) -> Vec<&str> {
    content
        .lines()
        .filter(|l| l.trim_start().starts_with('#'))
        .collect()
}

// ── Comment formatting ───────────────────────────────────────────────────

#[test]
fn comment_lines_start_with_hash() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in comment_lines(&content) {
            let trimmed = line.trim();
            assert!(
                trimmed.starts_with('#'),
                "{profile:?}: comment line does not start with #: {line:?}"
            );
        }
    }
}

#[test]
fn non_empty_comments_have_space_after_hash() {
    // Comments with text should use "# text" format, not "#text"
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in comment_lines(&content) {
            let trimmed = line.trim();
            // Skip bare "#" lines (section separators)
            if trimmed == "#" {
                continue;
            }
            assert!(
                trimmed.starts_with("# "),
                "{profile:?}: comment lacks space after #: {line:?}"
            );
        }
    }
}

#[test]
fn first_line_is_a_header_comment() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        let first = content.lines().next().unwrap();
        assert!(
            first.starts_with("# .tokeignore"),
            "{profile:?}: first line should be '# .tokeignore' header, got: {first:?}"
        );
    }
}

// ── Pattern structure ────────────────────────────────────────────────────

#[test]
fn no_pattern_has_leading_whitespace() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in pattern_lines(&content) {
            assert_eq!(
                line,
                line.trim_start(),
                "{profile:?}: pattern has leading whitespace: {line:?}"
            );
        }
    }
}

#[test]
fn no_pattern_has_trailing_whitespace() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in content.lines() {
            assert_eq!(
                line,
                line.trim_end(),
                "{profile:?}: line has trailing whitespace: {line:?}"
            );
        }
    }
}

#[test]
fn no_duplicate_patterns_within_a_template() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        let patterns = pattern_lines(&content);
        let unique: HashSet<&str> = patterns.iter().copied().collect();
        assert_eq!(
            patterns.len(),
            unique.len(),
            "{profile:?}: template has duplicate patterns"
        );
    }
}

#[test]
fn no_line_exceeds_120_characters() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for (i, line) in content.lines().enumerate() {
            assert!(
                line.len() <= 120,
                "{profile:?}: line {} exceeds 120 chars (len={}): {line:?}",
                i + 1,
                line.len()
            );
        }
    }
}

// ── Language-specific pattern isolation ───────────────────────────────────

#[test]
fn rust_template_does_not_contain_python_patterns() {
    let c = write_template(InitProfile::Rust);
    assert!(
        !c.contains("__pycache__/"),
        "Rust should not have __pycache__"
    );
    assert!(!c.contains(".venv/"), "Rust should not have .venv");
    assert!(!c.contains("*.pyc"), "Rust should not have *.pyc");
}

#[test]
fn python_template_does_not_contain_rust_patterns() {
    let c = write_template(InitProfile::Python);
    assert!(!c.contains("target/"), "Python should not have target/");
    assert!(!c.contains("*.rs.bk"), "Python should not have *.rs.bk");
}

#[test]
fn node_template_does_not_contain_go_or_cpp_patterns() {
    let c = write_template(InitProfile::Node);
    assert!(
        !c.contains("cmake-build-*/"),
        "Node should not have cmake-build"
    );
    assert!(!c.contains("*.pyc"), "Node should not have *.pyc");
}

#[test]
fn go_template_is_minimal() {
    let c = write_template(InitProfile::Go);
    let patterns = pattern_lines(&c);
    // Go template should be one of the smallest
    assert!(
        patterns.len() <= 8,
        "Go template should be compact, found {} patterns",
        patterns.len()
    );
    assert!(
        !c.contains("node_modules/"),
        "Go should not have node_modules"
    );
    assert!(
        !c.contains("__pycache__/"),
        "Go should not have __pycache__"
    );
}

// ── Default exclusion entry invariants ───────────────────────────────────

#[test]
fn every_template_excludes_tokmd_runs_dir() {
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        let patterns = pattern_lines(&content);
        assert!(
            patterns.contains(&".runs/"),
            "{profile:?}: missing .runs/ pattern"
        );
        assert!(
            patterns.contains(&"**/.runs/"),
            "{profile:?}: missing **/.runs/ recursive pattern"
        );
    }
}

#[test]
fn directory_patterns_have_trailing_slash() {
    // Known directory patterns should always end with /
    let known_dirs = [
        "target",
        "node_modules",
        "__pycache__",
        "vendor",
        "dist",
        "build",
        "out",
        "coverage",
    ];
    for profile in ALL_PROFILES {
        let content = write_template(profile);
        for line in pattern_lines(&content) {
            // Check if it looks like a known directory pattern without trailing slash
            let bare = line.trim_start_matches("**/");
            for dir in &known_dirs {
                if bare == *dir {
                    panic!("{profile:?}: directory pattern '{line}' missing trailing slash");
                }
            }
        }
    }
}

#[test]
fn default_template_has_section_separator_comments() {
    let c = write_template(InitProfile::Default);
    let section_markers: Vec<&str> = c.lines().filter(|l| l.contains("---")).collect();
    assert!(
        section_markers.len() >= 3,
        "Default template should have at least 3 section separators, found {}",
        section_markers.len()
    );
}

#[test]
fn template_written_to_disk_matches_returned_path() {
    for profile in ALL_PROFILES {
        let dir = tempfile::tempdir().unwrap();
        let result = init_tokeignore(&make_args(profile, false, false, dir.path().into())).unwrap();
        let path = result.expect("should return Some(path)");
        assert_eq!(
            path,
            dir.path().join(".tokeignore"),
            "{profile:?}: returned path mismatch"
        );
        assert!(path.exists(), "{profile:?}: returned path does not exist");
    }
}

#[test]
fn consecutive_writes_with_different_profiles_overwrite_correctly() {
    let dir = tempfile::tempdir().unwrap();
    // Write Rust first
    init_tokeignore(&make_args(
        InitProfile::Rust,
        false,
        false,
        dir.path().into(),
    ))
    .unwrap();
    let c1 = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(c1.contains("(Rust)"));

    // Overwrite with Python using --force
    init_tokeignore(&make_args(
        InitProfile::Python,
        false,
        true,
        dir.path().into(),
    ))
    .unwrap();
    let c2 = fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(c2.contains("(Python)"));
    assert!(!c2.contains("(Rust)"), "old profile content should be gone");
}
