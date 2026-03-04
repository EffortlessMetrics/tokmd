//! Deep tests for tokmd-walk (wave 48).
//!
//! Covers filesystem traversal with tempdir fixtures, .gitignore/.tokeignore
//! handling, hidden file behavior, symlink handling, asset detection,
//! property-based determinism, and edge cases.

use std::fs;
use std::path::{Path, PathBuf};
use tokmd_walk::{file_size, license_candidates, list_files};

// ============================================================================
// 1. Filesystem traversal with tempdir fixtures
// ============================================================================

#[test]
fn walk_flat_dir_finds_all_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("lib.rs"), "pub fn lib() {}").unwrap();
    fs::write(dir.path().join("README.md"), "# readme").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 3, "Expected 3 files, got {:?}", files);
}

#[test]
fn walk_multi_level_nesting() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src/util/internal")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "x").unwrap();
    fs::write(dir.path().join("src/util/helpers.rs"), "x").unwrap();
    fs::write(dir.path().join("src/util/internal/deep.rs"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 3, "Expected >=3 files, got {}", files.len());
}

#[test]
fn walk_returns_relative_paths_only() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a/b")).unwrap();
    fs::write(dir.path().join("a/b/file.rs"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(f.is_relative(), "Path {:?} must be relative to root", f);
        assert!(
            !f.to_string_lossy().contains(dir.path().to_str().unwrap()),
            "Path {:?} must not contain the root prefix",
            f
        );
    }
}

#[test]
fn walk_directories_are_excluded() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("empty_dir")).unwrap();
    fs::create_dir_all(dir.path().join("another_dir/sub")).unwrap();
    fs::write(dir.path().join("file.txt"), "content").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("file.txt"));
}

#[test]
fn walk_max_files_respected() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..20 {
        fs::write(dir.path().join(format!("file_{i:02}.txt")), "x").unwrap();
    }
    let files = list_files(dir.path(), Some(5)).unwrap();
    assert!(
        files.len() <= 5,
        "Expected at most 5 files, got {}",
        files.len()
    );
}

// ============================================================================
// 2. Respecting .gitignore patterns
// ============================================================================

fn init_git(dir: &Path) {
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .output()
        .unwrap();
}

#[test]
fn gitignore_wildcard_extension() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    fs::write(dir.path().join("app.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("debug.log"), "log data").unwrap();
    fs::write(dir.path().join("error.log"), "log data").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.ends_with(".log")),
        "*.log should be ignored, got: {:?}",
        names
    );
    assert!(names.iter().any(|n| n.ends_with("app.rs")));
}

#[test]
fn gitignore_directory_pattern() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    fs::write(dir.path().join(".gitignore"), "target/\n").unwrap();
    fs::create_dir_all(dir.path().join("target/debug")).unwrap();
    fs::write(dir.path().join("target/debug/binary"), "bin").unwrap();
    fs::write(dir.path().join("src.rs"), "fn main() {}").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("target")),
        "target/ dir should be ignored, got: {:?}",
        names
    );
}

#[test]
fn gitignore_negation_pattern() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    fs::write(dir.path().join(".gitignore"), "*.tmp\n!important.tmp\n").unwrap();
    fs::write(dir.path().join("junk.tmp"), "junk").unwrap();
    fs::write(dir.path().join("important.tmp"), "keep").unwrap();
    fs::write(dir.path().join("code.rs"), "fn main() {}").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("junk.tmp")),
        "junk.tmp should be ignored"
    );
    assert!(
        names.iter().any(|n| n.contains("important.tmp")),
        "important.tmp should be preserved by negation, got: {:?}",
        names
    );
}

#[test]
fn gitignore_nested_in_subdirectory() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    fs::create_dir_all(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/.gitignore"), "*.bak\n").unwrap();
    fs::write(dir.path().join("sub/keep.rs"), "x").unwrap();
    fs::write(dir.path().join("sub/drop.bak"), "y").unwrap();
    fs::write(dir.path().join("root.bak"), "z").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("drop.bak")),
        "sub/.gitignore should hide *.bak in sub/"
    );
}

// ============================================================================
// 3. Respecting .tokeignore patterns (uses .gitignore format via ignore crate)
// ============================================================================

#[test]
fn tokeignore_pattern_via_custom_ignore_file() {
    // The ignore crate supports custom ignore files; tokmd-walk's WalkBuilder
    // uses git_ignore(true) which picks up .gitignore. Test the pattern with
    // a .gitignore since the walker doesn't explicitly load .tokeignore.
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    fs::write(dir.path().join(".gitignore"), "generated/\n*.gen.rs\n").unwrap();
    fs::create_dir_all(dir.path().join("generated")).unwrap();
    fs::write(dir.path().join("generated/output.rs"), "x").unwrap();
    fs::write(dir.path().join("auto.gen.rs"), "x").unwrap();
    fs::write(dir.path().join("manual.rs"), "x").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("generated")),
        "generated/ should be ignored"
    );
    assert!(
        !names.iter().any(|n| n.ends_with(".gen.rs")),
        "*.gen.rs should be ignored"
    );
    assert!(names.iter().any(|n| n.contains("manual.rs")));
}

// ============================================================================
// 4. Hidden file handling
// ============================================================================

#[test]
fn hidden_files_are_included_by_walker() {
    // WalkBuilder is configured with hidden(false) = "do not skip hidden files"
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".hidden_config"), "config").unwrap();
    fs::write(dir.path().join("visible.txt"), "visible").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.contains(".hidden_config")),
        "Hidden files should be included (hidden(false) means 'don't skip hidden'), got: {:?}",
        names
    );
}

#[test]
fn hidden_directories_are_traversed() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".hidden_dir")).unwrap();
    fs::write(dir.path().join(".hidden_dir/secret.txt"), "secret").unwrap();
    fs::write(dir.path().join("public.txt"), "public").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("secret.txt")),
        "Files in hidden directories should be included, got: {:?}",
        names
    );
}

// ============================================================================
// 5. Symlink handling
// ============================================================================

#[cfg(unix)]
#[test]
fn symlink_to_file_does_not_cause_panic() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("target.txt"), "content").unwrap();
    std::os::unix::fs::symlink(dir.path().join("target.txt"), dir.path().join("link.txt")).unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(
        !files.is_empty(),
        "Should still find files with symlinks present"
    );
}

#[cfg(unix)]
#[test]
fn symlink_to_directory_not_traversed() {
    let dir = tempfile::tempdir().unwrap();
    let ext_dir = tempfile::tempdir().unwrap();
    fs::write(ext_dir.path().join("external.txt"), "external").unwrap();
    std::os::unix::fs::symlink(ext_dir.path(), dir.path().join("linked")).unwrap();
    fs::write(dir.path().join("local.txt"), "local").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        !names.iter().any(|n| n.contains("external.txt")),
        "Symlinked directory should not be traversed (follow_links=false)"
    );
}

#[cfg(unix)]
#[test]
fn dangling_symlink_does_not_crash() {
    let dir = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(
        "/nonexistent/path/file.txt",
        dir.path().join("dangling.txt"),
    )
    .unwrap();
    fs::write(dir.path().join("real.txt"), "content").unwrap();
    // Should not panic
    let files = list_files(dir.path(), None).unwrap();
    assert!(
        files
            .iter()
            .any(|f| f.to_string_lossy().contains("real.txt")),
        "Real files should still be found despite dangling symlinks"
    );
}

#[cfg(windows)]
#[test]
fn symlink_creation_does_not_panic_walker() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("real.txt"), "content").unwrap();
    // Symlink creation may require admin privileges on Windows
    let _ = std::os::windows::fs::symlink_file(
        dir.path().join("real.txt"),
        dir.path().join("link.txt"),
    );
    let files = list_files(dir.path(), None).unwrap();
    assert!(!files.is_empty());
}

// ============================================================================
// 6. Asset detection (images, binaries, lockfiles)
// ============================================================================

#[test]
fn asset_like_files_detected_by_license_candidates() {
    // license_candidates detects Cargo.toml, package.json, pyproject.toml as metadata
    let files = vec![
        PathBuf::from("Cargo.lock"),
        PathBuf::from("package-lock.json"),
        PathBuf::from("yarn.lock"),
        PathBuf::from("poetry.lock"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
    ];
    let result = license_candidates(&files);
    // Lockfiles are not detected as license or metadata
    assert_eq!(
        result.metadata_files.len(),
        2,
        "Only Cargo.toml and package.json are metadata"
    );
    assert!(result.license_files.is_empty());
}

#[test]
fn walk_includes_binary_extension_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("image.png"), &[0x89, 0x50, 0x4E, 0x47]).unwrap();
    fs::write(dir.path().join("data.bin"), &[0u8; 100]).unwrap();
    fs::write(dir.path().join("code.rs"), "fn main() {}").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.ends_with(".png")),
        "Images should be walked"
    );
    assert!(
        names.iter().any(|n| n.ends_with(".bin")),
        "Binary files should be walked"
    );
    assert!(names.iter().any(|n| n.ends_with(".rs")));
}

#[test]
fn walk_includes_lockfiles() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Cargo.lock"), "[metadata]").unwrap();
    fs::write(dir.path().join("package-lock.json"), "{}").unwrap();
    fs::write(dir.path().join("yarn.lock"), "# yarn").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 3, "All lockfiles should be walked");
}

// ============================================================================
// 7. Property test: walk results are sorted deterministically
// ============================================================================

mod properties {
    use proptest::prelude::*;
    use std::fs;
    use tokmd_walk::list_files;

    proptest! {
        #[test]
        fn walk_results_always_sorted(
            file_count in 1usize..15,
            seed in any::<u64>()
        ) {
            let _ = seed;
            let dir = tempfile::tempdir().unwrap();
            for i in 0..file_count {
                // Use reverse naming to stress sort
                let name = format!("file_{:02}.txt", file_count - i);
                fs::write(dir.path().join(&name), "x").unwrap();
            }
            let files = list_files(dir.path(), None).unwrap();
            let names: Vec<String> = files.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let mut sorted = names.clone();
            sorted.sort();
            prop_assert_eq!(&names, &sorted, "Walk results must be sorted");
        }

        #[test]
        fn walk_deterministic_across_calls(file_count in 1usize..10) {
            let dir = tempfile::tempdir().unwrap();
            for i in 0..file_count {
                fs::write(dir.path().join(format!("f{i}.rs")), "x").unwrap();
            }
            let a = list_files(dir.path(), None).unwrap();
            let b = list_files(dir.path(), None).unwrap();
            prop_assert_eq!(a, b, "Repeated calls must yield identical results");
        }
    }
}

// ============================================================================
// 8. Edge cases
// ============================================================================

#[test]
fn edge_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty());
}

#[test]
fn edge_deeply_nested_single_file() {
    let dir = tempfile::tempdir().unwrap();
    let deep = dir.path().join("a/b/c/d/e/f/g/h");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("leaf.txt"), "leaf").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
    let name = files[0].to_string_lossy().to_string();
    assert!(name.contains("leaf.txt"));
}

#[test]
fn edge_directory_with_only_hidden_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".env"), "SECRET=x").unwrap();
    fs::write(dir.path().join(".config"), "key=val").unwrap();
    fs::write(dir.path().join(".gitkeep"), "").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    // hidden(false) means hidden files are included
    assert!(
        files.len() >= 3,
        "All hidden files should be found, got {}",
        files.len()
    );
}

#[test]
fn edge_unicode_filenames() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("données.txt"), "data").unwrap();
    fs::write(dir.path().join("日本語.rs"), "fn main() {}").unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.len() >= 2, "Unicode filenames should be traversed");
}

#[test]
fn edge_many_sibling_directories() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..10 {
        let sub = dir.path().join(format!("mod_{i:02}"));
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("lib.rs"), format!("// mod {i}")).unwrap();
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 10, "Should find one file per directory");
}

#[test]
fn edge_file_size_zero_byte_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("zero.bin"), "").unwrap();
    let size = file_size(dir.path(), Path::new("zero.bin")).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn edge_file_size_binary_content() {
    let dir = tempfile::tempdir().unwrap();
    let data: Vec<u8> = (0..=255).cycle().take(4096).collect();
    fs::write(dir.path().join("blob.bin"), &data).unwrap();
    let size = file_size(dir.path(), Path::new("blob.bin")).unwrap();
    assert_eq!(size, 4096);
}

#[test]
fn edge_max_zero_on_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let files = list_files(dir.path(), Some(0)).unwrap();
    assert!(files.is_empty());
}

#[test]
fn edge_max_one_with_nested() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("a")).unwrap();
    fs::write(dir.path().join("a/one.rs"), "x").unwrap();
    fs::write(dir.path().join("a/two.rs"), "y").unwrap();
    fs::write(dir.path().join("root.rs"), "z").unwrap();
    let files = list_files(dir.path(), Some(1)).unwrap();
    assert!(files.len() <= 1);
}
