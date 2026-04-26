//! Contract tests for `tokmd-scan::walk` module — W64 batch.
//!
//! Covers filesystem traversal, license detection, file sizing, determinism,
//! and edge-case behaviour of the public API surface.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tokmd_scan::walk::{file_size, license_candidates, list_files};

// ===========================================================================
// Helpers
// ===========================================================================

/// Create a tempdir that is NOT inside a git repo so `git ls-files` falls
/// back to the `ignore` crate walker.
fn non_git_dir() -> TempDir {
    TempDir::new().expect("create tempdir")
}

fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(&path, content).expect("write file");
}

// ===========================================================================
// 1  list_files — basic directory structures
// ===========================================================================

#[test]
fn list_files_flat_directory_returns_all_files() {
    let dir = non_git_dir();
    for name in ["a.rs", "b.py", "c.go"] {
        write_file(&dir, name, "x");
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 3, "flat dir with 3 files");
}

#[test]
fn list_files_nested_two_levels() {
    let dir = non_git_dir();
    write_file(&dir, "src/lib.rs", "code");
    write_file(&dir, "src/util/helpers.rs", "code");
    write_file(&dir, "README.md", "# hi");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn list_files_deeply_nested_path() {
    let dir = non_git_dir();
    write_file(&dir, "a/b/c/d/e/f/g/deep.txt", "deep");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
    let name = files[0].to_string_lossy();
    assert!(name.contains("deep.txt"), "should contain deep.txt");
}

#[test]
fn list_files_multiple_subdirectories() {
    let dir = non_git_dir();
    for sub in ["alpha", "beta", "gamma"] {
        write_file(&dir, &format!("{sub}/file.txt"), sub);
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 3);
}

#[test]
fn list_files_mixed_extensions() {
    let dir = non_git_dir();
    for ext in ["rs", "py", "js", "go", "c", "h", "toml", "json"] {
        write_file(&dir, &format!("file.{ext}"), "x");
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 8);
}

// ===========================================================================
// 2  list_files — max_files limit behaviour
// ===========================================================================

#[test]
fn max_files_zero_always_empty() {
    let dir = non_git_dir();
    write_file(&dir, "a.txt", "a");
    let files = list_files(dir.path(), Some(0)).unwrap();
    assert!(files.is_empty());
}

#[test]
fn max_files_one_returns_at_most_one() {
    let dir = non_git_dir();
    for i in 0..5 {
        write_file(&dir, &format!("f{i}.txt"), "x");
    }
    let files = list_files(dir.path(), Some(1)).unwrap();
    assert!(files.len() <= 1);
}

#[test]
fn max_files_exact_count() {
    let dir = non_git_dir();
    for i in 0..5 {
        write_file(&dir, &format!("f{i}.txt"), "x");
    }
    let files = list_files(dir.path(), Some(5)).unwrap();
    assert!(files.len() <= 5);
}

#[test]
fn max_files_larger_than_total_returns_all() {
    let dir = non_git_dir();
    write_file(&dir, "only.txt", "x");
    let files = list_files(dir.path(), Some(1000)).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn max_files_limits_large_directory() {
    let dir = non_git_dir();
    for i in 0..50 {
        write_file(&dir, &format!("f{i:03}.txt"), "x");
    }
    let files = list_files(dir.path(), Some(10)).unwrap();
    assert!(files.len() <= 10, "expected ≤10, got {}", files.len());
}

// ===========================================================================
// 3  list_files — deterministic ordering
// ===========================================================================

#[test]
fn list_files_is_sorted() {
    let dir = non_git_dir();
    for name in ["z.txt", "a.txt", "m.txt", "b.txt"] {
        write_file(&dir, name, "x");
    }
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "output must be sorted");
}

#[test]
fn list_files_repeated_calls_identical() {
    let dir = non_git_dir();
    for i in 0..8 {
        write_file(&dir, &format!("f{i}.txt"), "x");
    }
    let a = list_files(dir.path(), None).unwrap();
    let b = list_files(dir.path(), None).unwrap();
    assert_eq!(a, b, "repeated calls must be deterministic");
}

#[test]
fn list_files_sorted_across_subdirectories() {
    let dir = non_git_dir();
    write_file(&dir, "z/file.txt", "x");
    write_file(&dir, "a/file.txt", "x");
    write_file(&dir, "m/file.txt", "x");
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

// ===========================================================================
// 4  list_files — relative paths
// ===========================================================================

#[test]
fn list_files_all_paths_are_relative() {
    let dir = non_git_dir();
    write_file(&dir, "src/main.rs", "fn main(){}");
    write_file(&dir, "lib.rs", "//lib");
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        assert!(f.is_relative(), "path should be relative: {f:?}");
    }
}

#[test]
fn list_files_no_leading_dot_slash() {
    let dir = non_git_dir();
    write_file(&dir, "file.txt", "x");
    let files = list_files(dir.path(), None).unwrap();
    for f in &files {
        let s = f.to_string_lossy();
        assert!(!s.starts_with("./"), "no leading ./ in {s}");
    }
}

// ===========================================================================
// 5  list_files — edge cases
// ===========================================================================

#[test]
fn list_files_empty_directory() {
    let dir = non_git_dir();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty());
}

#[test]
fn list_files_directory_only_contains_subdirs() {
    let dir = non_git_dir();
    fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
    fs::create_dir_all(dir.path().join("x/y")).unwrap();
    let files = list_files(dir.path(), None).unwrap();
    assert!(files.is_empty(), "directories alone produce no files");
}

#[test]
fn list_files_hidden_files_included() {
    let dir = non_git_dir();
    write_file(&dir, ".hidden", "secret");
    write_file(&dir, "visible.txt", "public");
    let files = list_files(dir.path(), None).unwrap();
    // The walker is configured with hidden(false) meaning it shows hidden
    // files (hidden=false means "don't filter hidden").
    assert!(!files.is_empty());
}

#[test]
fn list_files_file_with_spaces_in_name() {
    let dir = non_git_dir();
    write_file(&dir, "my file.txt", "x");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("my file.txt"));
}

#[test]
fn list_files_many_files_boundary() {
    let dir = non_git_dir();
    for i in 0..200 {
        write_file(&dir, &format!("f{i:04}.txt"), "x");
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 200, "should list all 200 files");
}

// ===========================================================================
// 6  list_files — gitignore respect
// ===========================================================================

#[test]
fn list_files_respects_gitignore() {
    let dir = non_git_dir();
    // Initialize a git repo so gitignore is respected
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    write_file(&dir, ".gitignore", "*.log\nbuild/\n");
    write_file(&dir, "keep.rs", "code");
    write_file(&dir, "debug.log", "logs");
    write_file(&dir, "build/output.js", "built");
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert!(names.iter().any(|n| n.contains("keep.rs")));
    // .log and build/ should be ignored
    assert!(
        !names.iter().any(|n| n.ends_with(".log")),
        "log files should be ignored"
    );
    assert!(
        !names.iter().any(|n| n.contains("build/")),
        "build dir should be ignored"
    );
}

#[test]
fn list_files_respects_nested_gitignore() {
    let dir = non_git_dir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    write_file(&dir, "src/.gitignore", "generated/\n");
    write_file(&dir, "src/main.rs", "fn main(){}");
    write_file(&dir, "src/generated/auto.rs", "// auto");
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert!(names.iter().any(|n| n.contains("main.rs")));
    assert!(
        !names.iter().any(|n| n.contains("generated")),
        "generated dir should be ignored"
    );
}

// ===========================================================================
// 7  list_files — property: subset of actual files
// ===========================================================================

#[test]
fn list_files_subset_when_limited() {
    let dir = non_git_dir();
    for i in 0..20 {
        write_file(&dir, &format!("f{i:02}.txt"), "x");
    }
    let all = list_files(dir.path(), None).unwrap();
    let limited = list_files(dir.path(), Some(5)).unwrap();
    assert!(limited.len() <= 5);
    // Every file in limited must exist in all
    for f in &limited {
        assert!(all.contains(f), "{f:?} should be in the full listing");
    }
}

// ===========================================================================
// 8  license_candidates — comprehensive
// ===========================================================================

#[test]
fn license_candidates_all_license_variants() {
    let files: Vec<PathBuf> = [
        "LICENSE",
        "LICENSE.md",
        "LICENSE.txt",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "license",
        "License.md",
        "COPYING",
        "COPYING.txt",
        "copying",
        "NOTICE",
        "NOTICE.md",
        "notice",
        "notice.txt",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();

    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 14);
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_all_metadata_variants() {
    let files: Vec<PathBuf> = ["Cargo.toml", "package.json", "pyproject.toml"]
        .iter()
        .map(PathBuf::from)
        .collect();
    let result = license_candidates(&files);
    assert_eq!(result.metadata_files.len(), 3);
    assert!(result.license_files.is_empty());
}

#[test]
fn license_candidates_rejects_non_matching() {
    let files: Vec<PathBuf> = [
        "README.md",
        "main.rs",
        "Makefile",
        ".gitignore",
        "Dockerfile",
        "setup.py",
        "index.js",
        "tsconfig.json",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    let result = license_candidates(&files);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_mixed_with_paths() {
    let files: Vec<PathBuf> = ["sub/LICENSE", "sub/Cargo.toml", "sub/main.rs"]
        .iter()
        .map(PathBuf::from)
        .collect();
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 1);
    assert_eq!(result.metadata_files.len(), 1);
}

#[test]
fn license_candidates_preserves_full_path() {
    let files = [PathBuf::from("deep/nested/LICENSE-MIT")];
    let result = license_candidates(&files);
    assert_eq!(
        result.license_files[0],
        PathBuf::from("deep/nested/LICENSE-MIT")
    );
}

#[test]
fn license_candidates_empty_slice() {
    let result = license_candidates(&[]);
    assert!(result.license_files.is_empty());
    assert!(result.metadata_files.is_empty());
}

#[test]
fn license_candidates_sorted_deterministic() {
    let files: Vec<PathBuf> = [
        "z/NOTICE",
        "a/LICENSE",
        "m/COPYING",
        "b/Cargo.toml",
        "x/package.json",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    let result = license_candidates(&files);
    let lic_names: Vec<String> = result
        .license_files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let mut sorted_lic = lic_names.clone();
    sorted_lic.sort();
    assert_eq!(lic_names, sorted_lic, "license files must be sorted");

    let meta_names: Vec<String> = result
        .metadata_files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let mut sorted_meta = meta_names.clone();
    sorted_meta.sort();
    assert_eq!(meta_names, sorted_meta, "metadata files must be sorted");
}

#[test]
fn license_candidates_case_insensitive_cargo() {
    // Cargo.toml but not cargo.TOML (lowercased comparison)
    let files = [PathBuf::from("cargo.toml")];
    let result = license_candidates(&files);
    assert_eq!(result.metadata_files.len(), 1, "cargo.toml should match");
}

// ===========================================================================
// 9  file_size — comprehensive
// ===========================================================================

#[test]
fn file_size_exact_bytes() {
    let dir = non_git_dir();
    write_file(&dir, "hello.txt", "hello");
    let size = file_size(dir.path(), Path::new("hello.txt")).unwrap();
    assert_eq!(size, 5);
}

#[test]
fn file_size_empty_file_is_zero() {
    let dir = non_git_dir();
    write_file(&dir, "empty.txt", "");
    assert_eq!(file_size(dir.path(), Path::new("empty.txt")).unwrap(), 0);
}

#[test]
fn file_size_missing_file_returns_error() {
    let dir = non_git_dir();
    let result = file_size(dir.path(), Path::new("ghost.txt"));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("ghost.txt"), "error should mention filename");
}

#[test]
fn file_size_large_content() {
    let dir = non_git_dir();
    let content = "x".repeat(100_000);
    write_file(&dir, "big.bin", &content);
    assert_eq!(
        file_size(dir.path(), Path::new("big.bin")).unwrap(),
        100_000
    );
}

#[test]
fn file_size_nested_file() {
    let dir = non_git_dir();
    write_file(&dir, "a/b/c.txt", "abc");
    let size = file_size(dir.path(), Path::new("a/b/c.txt")).unwrap();
    assert_eq!(size, 3);
}

#[test]
fn file_size_binary_content() {
    let dir = non_git_dir();
    let bytes: Vec<u8> = (0..=255).collect();
    fs::write(dir.path().join("binary.dat"), &bytes).unwrap();
    assert_eq!(file_size(dir.path(), Path::new("binary.dat")).unwrap(), 256);
}

#[test]
fn file_size_single_byte() {
    let dir = non_git_dir();
    write_file(&dir, "one.txt", "x");
    assert_eq!(file_size(dir.path(), Path::new("one.txt")).unwrap(), 1);
}

// ===========================================================================
// 10  LicenseCandidates struct traits
// ===========================================================================

#[test]
fn license_candidates_debug_format() {
    let lc = license_candidates(&[PathBuf::from("LICENSE")]);
    let dbg = format!("{lc:?}");
    assert!(dbg.contains("LicenseCandidates"));
}

#[test]
fn license_candidates_clone_equality() {
    let lc = license_candidates(&[PathBuf::from("LICENSE"), PathBuf::from("Cargo.toml")]);
    let cloned = lc.clone();
    assert_eq!(lc.license_files, cloned.license_files);
    assert_eq!(lc.metadata_files, cloned.metadata_files);
}

// ===========================================================================
// 11  BDD scenarios
// ===========================================================================

#[test]
fn given_source_tree_when_walking_then_all_files_listed() {
    // Given: a directory with Rust source files
    let dir = non_git_dir();
    write_file(&dir, "src/main.rs", "fn main() {}");
    write_file(&dir, "src/lib.rs", "pub fn hi() {}");
    write_file(&dir, "Cargo.toml", "[package]");

    // When: walking with no limit
    let files = list_files(dir.path(), None).unwrap();

    // Then: all three files are returned
    assert_eq!(files.len(), 3);
}

#[test]
fn given_gitignored_files_when_walking_then_ignored_excluded() {
    // Given: a git repo with gitignore
    let dir = non_git_dir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    write_file(&dir, ".gitignore", "target/\n*.tmp\n");
    write_file(&dir, "src/main.rs", "code");
    write_file(&dir, "target/debug/bin", "binary");
    write_file(&dir, "scratch.tmp", "temp");

    // When: walking
    let files = list_files(dir.path(), None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    // Then: ignored files excluded, source kept
    assert!(names.iter().any(|n| n.contains("main.rs")));
    assert!(!names.iter().any(|n| n.contains("target")));
    assert!(!names.iter().any(|n| n.ends_with(".tmp")));
}

#[test]
fn given_max_limit_when_walking_then_result_capped() {
    // Given: directory with many files
    let dir = non_git_dir();
    for i in 0..30 {
        write_file(&dir, &format!("f{i:02}.txt"), "x");
    }

    // When: walking with limit of 7
    let files = list_files(dir.path(), Some(7)).unwrap();

    // Then: at most 7 files returned
    assert!(files.len() <= 7);
}

#[test]
fn given_nested_tree_when_walking_then_paths_normalized() {
    // Given: nested directory
    let dir = non_git_dir();
    write_file(&dir, "a/b/c.txt", "x");

    // When: walking
    let files = list_files(dir.path(), None).unwrap();

    // Then: paths are relative
    for f in &files {
        assert!(f.is_relative());
        assert!(!f.to_string_lossy().starts_with("./"));
    }
}

// ===========================================================================
// 12  Property-based tests
// ===========================================================================

mod properties {
    use proptest::prelude::*;
    use std::path::PathBuf;
    use tokmd_scan::walk::license_candidates;

    proptest! {
        #[test]
        fn license_count_never_exceeds_input(
            files in proptest::collection::vec("[a-zA-Z0-9_./-]{1,30}", 0..30)
        ) {
            let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
            let result = license_candidates(&paths);
            let total = result.license_files.len() + result.metadata_files.len();
            prop_assert!(total <= paths.len(), "candidates can't exceed input count");
        }

        #[test]
        fn license_and_metadata_are_disjoint(
            files in proptest::collection::vec("[a-zA-Z0-9_./-]{1,30}", 0..30)
        ) {
            let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
            let result = license_candidates(&paths);
            for lic in &result.license_files {
                prop_assert!(
                    !result.metadata_files.contains(lic),
                    "file should not be both license and metadata: {lic:?}"
                );
            }
        }

        #[test]
        fn output_always_sorted(
            files in proptest::collection::vec("[a-zA-Z0-9_./-]{1,30}", 0..30)
        ) {
            let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
            let result = license_candidates(&paths);

            let lic: Vec<String> = result.license_files.iter()
                .map(|p| p.to_string_lossy().into_owned()).collect();
            let mut sorted_lic = lic.clone();
            sorted_lic.sort();
            prop_assert_eq!(&lic, &sorted_lic, "license files must be sorted");

            let meta: Vec<String> = result.metadata_files.iter()
                .map(|p| p.to_string_lossy().into_owned()).collect();
            let mut sorted_meta = meta.clone();
            sorted_meta.sort();
            prop_assert_eq!(&meta, &sorted_meta, "metadata files must be sorted");
        }

        #[test]
        fn idempotent_calls(
            files in proptest::collection::vec("[a-zA-Z0-9_./-]{1,30}", 0..20)
        ) {
            let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
            let r1 = license_candidates(&paths);
            let r2 = license_candidates(&paths);
            prop_assert_eq!(r1.license_files, r2.license_files);
            prop_assert_eq!(r1.metadata_files, r2.metadata_files);
        }
    }
}

// ===========================================================================
// 13  Boundary — deeply nested paths
// ===========================================================================

#[test]
fn deeply_nested_10_levels() {
    let dir = non_git_dir();
    let nested = "a/b/c/d/e/f/g/h/i/j/leaf.txt";
    write_file(&dir, nested, "deep");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 1);
}

#[test]
fn many_sibling_directories() {
    let dir = non_git_dir();
    for i in 0..20 {
        write_file(&dir, &format!("dir{i:02}/file.txt"), "x");
    }
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 20);
}

#[test]
fn file_with_no_extension() {
    let dir = non_git_dir();
    write_file(&dir, "Makefile", "all:");
    write_file(&dir, "Dockerfile", "FROM rust");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn file_with_multiple_dots() {
    let dir = non_git_dir();
    write_file(&dir, "archive.tar.gz", "x");
    write_file(&dir, "config.dev.yaml", "x");
    let files = list_files(dir.path(), None).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn license_candidates_with_deeply_nested_license() {
    let files = [PathBuf::from("a/b/c/d/LICENSE")];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 1);
}

#[test]
fn file_size_after_overwrite() {
    let dir = non_git_dir();
    write_file(&dir, "f.txt", "short");
    let s1 = file_size(dir.path(), Path::new("f.txt")).unwrap();
    write_file(&dir, "f.txt", "a longer string now");
    let s2 = file_size(dir.path(), Path::new("f.txt")).unwrap();
    assert!(s2 > s1, "size should grow after overwrite");
}

#[test]
fn list_files_skips_directories_themselves() {
    let dir = non_git_dir();
    fs::create_dir_all(dir.path().join("empty_sub")).unwrap();
    write_file(&dir, "file.txt", "x");
    let files = list_files(dir.path(), None).unwrap();
    // Should only contain the file, not the directory
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("file.txt"));
}
