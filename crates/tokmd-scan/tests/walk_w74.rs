//! W74 deep tests for tokmd-scan walk helpers: file walking, gitignore patterns,
//! empty directories, nested structures, asset/binary detection,
//! license candidate edge cases, and file-size boundaries.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_scan::walk::{file_size, license_candidates, list_files};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn git_in(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(dir);
    cmd
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

struct TempRepo {
    path: PathBuf,
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn make_repo(tag: &str) -> Option<TempRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "w74-walk-{}-{}-{:?}",
        tag,
        std::process::id(),
        std::thread::current().id(),
    );
    let dir = std::env::temp_dir().join(format!("tokmd-scan-walk-w74-{}", id));
    if dir.exists() {
        fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).ok()?;

    let ok = git_in(&dir).arg("init").output().ok()?.status.success();
    if !ok {
        fs::remove_dir_all(&dir).ok();
        return None;
    }
    git_in(&dir)
        .args(["config", "user.email", "w74@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "W74 Test"])
        .output()
        .ok()?;

    Some(TempRepo { path: dir })
}

fn commit_all(dir: &Path, msg: &str) {
    git_in(dir).args(["add", "."]).output().unwrap();
    git_in(dir).args(["commit", "-m", msg]).output().unwrap();
}

// ===========================================================================
// 1. Walking empty directories
// ===========================================================================

#[test]
fn walk_empty_dir_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let files = list_files(tmp.path(), None).unwrap();
    assert!(files.is_empty(), "empty dir should yield no files");
}

#[test]
fn walk_empty_dir_with_max_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let files = list_files(tmp.path(), Some(100)).unwrap();
    assert!(files.is_empty());
}

// ===========================================================================
// 2. Walking directories with nested structure
// ===========================================================================

#[test]
fn walk_nested_structure_finds_all_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("src/util")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("README.md"), "# hello").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("src/util/helpers.rs"), "// helpers").unwrap();
    fs::write(root.join("tests/test_one.rs"), "// test").unwrap();

    let files = list_files(root, None).unwrap();
    assert_eq!(files.len(), 4, "should find all 4 files");
}

#[test]
fn walk_nested_returns_relative_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("a/b/c")).unwrap();
    fs::write(root.join("a/b/c/deep.txt"), "deep").unwrap();

    let files = list_files(root, None).unwrap();
    assert_eq!(files.len(), 1);
    let p = files[0].to_string_lossy().replace('\\', "/");
    assert_eq!(p, "a/b/c/deep.txt", "path should be relative to root");
}

#[test]
fn walk_excludes_directories_from_results() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::create_dir_all(root.join("empty_dir/nested")).unwrap();
    fs::write(root.join("file.txt"), "x").unwrap();

    let files = list_files(root, None).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("file.txt"));
}

#[test]
fn walk_deeply_nested_directory_five_levels() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let deep = root.join("a/b/c/d/e");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("leaf.rs"), "leaf").unwrap();
    fs::write(root.join("top.rs"), "top").unwrap();

    let files = list_files(root, None).unwrap();
    assert_eq!(files.len(), 2);
}

// ===========================================================================
// 3. Max-files limiting
// ===========================================================================

#[test]
fn walk_max_files_limits_non_git_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    for i in 0..20 {
        fs::write(root.join(format!("file_{:02}.txt", i)), "x").unwrap();
    }

    let files = list_files(root, Some(5)).unwrap();
    assert!(files.len() <= 5, "should respect max_files limit");
}

#[test]
fn walk_max_none_returns_all_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    for i in 0..7 {
        fs::write(root.join(format!("f{i}.txt")), "x").unwrap();
    }

    let files = list_files(root, None).unwrap();
    assert_eq!(files.len(), 7);
}

// ===========================================================================
// 4. Gitignore-style pattern matching (via git repo)
// ===========================================================================

#[test]
fn walk_git_repo_respects_gitignore() {
    let repo = match make_repo("gitignore") {
        Some(r) => r,
        None => return,
    };
    let root = &repo.path;

    fs::write(root.join(".gitignore"), "*.log\nbuild/\n").unwrap();
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("debug.log"), "log data").unwrap();
    fs::create_dir_all(root.join("build")).unwrap();
    fs::write(root.join("build/output.bin"), "binary").unwrap();

    commit_all(root, "init with gitignore");

    let files = list_files(root, None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();

    assert!(
        names.contains(&"main.rs".to_string()),
        "tracked file should appear"
    );
    assert!(
        names.contains(&".gitignore".to_string()),
        ".gitignore should be tracked"
    );
    assert!(
        !names.iter().any(|n| n.ends_with(".log")),
        "*.log should be ignored"
    );
    assert!(
        !names.iter().any(|n| n.contains("build")),
        "build/ should be ignored"
    );
}

#[test]
fn walk_git_repo_untracked_excluded() {
    let repo = match make_repo("untracked") {
        Some(r) => r,
        None => return,
    };
    let root = &repo.path;

    fs::write(root.join("tracked.rs"), "tracked").unwrap();
    commit_all(root, "add tracked");

    fs::write(root.join("untracked.txt"), "not committed").unwrap();

    let files = list_files(root, None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();

    assert!(names.contains(&"tracked.rs".to_string()));
    assert!(
        !names.contains(&"untracked.txt".to_string()),
        "untracked files should NOT appear via git ls-files"
    );
}

#[test]
fn walk_git_repo_nested_gitignore() {
    let repo = match make_repo("nested-ignore") {
        Some(r) => r,
        None => return,
    };
    let root = &repo.path;

    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("sub/.gitignore"), "*.tmp\n").unwrap();
    fs::write(root.join("sub/keep.rs"), "keep").unwrap();
    fs::write(root.join("sub/discard.tmp"), "tmp").unwrap();
    fs::write(root.join("top.rs"), "top").unwrap();

    commit_all(root, "nested gitignore");

    let files = list_files(root, None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();

    assert!(names.iter().any(|n| n.contains("keep.rs")));
    assert!(
        !names.iter().any(|n| n.ends_with(".tmp")),
        "*.tmp should be ignored by sub/.gitignore"
    );
}

// ===========================================================================
// 5. Asset / binary file detection via license_candidates
// ===========================================================================

#[test]
fn license_candidates_ignores_image_and_binary_extensions() {
    let files = vec![
        PathBuf::from("logo.png"),
        PathBuf::from("icon.ico"),
        PathBuf::from("data.bin"),
        PathBuf::from("photo.jpg"),
        PathBuf::from("src/main.rs"),
    ];
    let result = license_candidates(&files);
    assert!(
        result.license_files.is_empty(),
        "binary/image files are not license files"
    );
    assert!(
        result.metadata_files.is_empty(),
        "binary/image files are not metadata"
    );
}

#[test]
fn license_candidates_detects_notice_variants() {
    let files = vec![
        PathBuf::from("NOTICE"),
        PathBuf::from("NOTICE.md"),
        PathBuf::from("NOTICE.txt"),
        PathBuf::from("notice"),
    ];
    let result = license_candidates(&files);
    assert_eq!(
        result.license_files.len(),
        4,
        "all NOTICE variants should match"
    );
}

#[test]
fn license_candidates_with_nested_metadata() {
    let files = vec![
        PathBuf::from("packages/a/Cargo.toml"),
        PathBuf::from("packages/b/package.json"),
        PathBuf::from("packages/c/pyproject.toml"),
        PathBuf::from("Cargo.toml"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.metadata_files.len(), 4);
    // Verify sorted order
    let first = result.metadata_files[0]
        .to_string_lossy()
        .replace('\\', "/");
    assert_eq!(first, "Cargo.toml", "root Cargo.toml sorts first");
}

#[test]
fn license_candidates_copying_with_extension() {
    let files = vec![
        PathBuf::from("COPYING.LIB"),
        PathBuf::from("COPYING.LESSER"),
    ];
    let result = license_candidates(&files);
    assert_eq!(result.license_files.len(), 2);
}

// ===========================================================================
// 6. File size edge cases
// ===========================================================================

#[test]
fn file_size_large_binary() {
    let tmp = tempfile::tempdir().unwrap();
    let data = vec![0xFFu8; 65536]; // 64 KiB
    fs::write(tmp.path().join("big.bin"), &data).unwrap();

    let sz = file_size(tmp.path(), Path::new("big.bin")).unwrap();
    assert_eq!(sz, 65536);
}

#[test]
fn file_size_in_subdirectory() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("a/b")).unwrap();
    fs::write(tmp.path().join("a/b/nested.txt"), "nested content!").unwrap();

    let sz = file_size(tmp.path(), Path::new("a/b/nested.txt")).unwrap();
    assert_eq!(sz, "nested content!".len() as u64);
}

#[test]
fn file_size_nonexistent_in_subdir_errors() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("sub")).unwrap();
    let result = file_size(tmp.path(), Path::new("sub/missing.txt"));
    assert!(result.is_err());
}

// ===========================================================================
// 7. Git repo walking with max_files
// ===========================================================================

#[test]
fn walk_git_max_files_truncates_tracked() {
    let repo = match make_repo("max-trunc") {
        Some(r) => r,
        None => return,
    };
    let root = &repo.path;

    for i in 0..10 {
        fs::write(root.join(format!("file_{:02}.txt", i)), "data").unwrap();
    }
    commit_all(root, "add 10 files");

    let files = list_files(root, Some(3)).unwrap();
    assert_eq!(
        files.len(),
        3,
        "max_files should truncate git ls-files output"
    );
}

#[test]
fn walk_git_max_files_larger_than_count() {
    let repo = match make_repo("max-large") {
        Some(r) => r,
        None => return,
    };
    let root = &repo.path;

    fs::write(root.join("only.txt"), "one").unwrap();
    commit_all(root, "single file");

    let files = list_files(root, Some(100)).unwrap();
    assert!(!files.is_empty());
    assert!(
        files.len() <= 2,
        "should return all tracked files (README + only.txt or just only.txt)"
    );
}

// ===========================================================================
// 8. Sorted output guarantee
// ===========================================================================

#[test]
fn walk_non_git_output_is_sorted() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    fs::write(root.join("z.txt"), "z").unwrap();
    fs::write(root.join("a.txt"), "a").unwrap();
    fs::write(root.join("m.txt"), "m").unwrap();

    let files = list_files(root, None).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().to_string())
        .collect();

    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "output must be alphabetically sorted");
}
