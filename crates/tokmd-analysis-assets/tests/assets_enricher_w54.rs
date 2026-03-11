//! W54: Comprehensive enricher coverage for `tokmd-analysis-assets`.
//!
//! Targets asset scanning, dependency lockfile detection, edge cases,
//! nested assets, deterministic ordering, and serialization contracts.

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tokmd_analysis_assets::{build_assets_report, build_dependency_report};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_file(dir: &Path, rel: &str, content: &[u8]) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ===========================================================================
// 1. Mixed extensions – only recognized ones counted
// ===========================================================================
#[test]
fn mixed_extensions_only_recognized_counted() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "logo.png", &[0u8; 100]),
        write_file(tmp.path(), "readme.md", b"# hi"),
        write_file(tmp.path(), "data.json", b"{}"),
        write_file(tmp.path(), "music.mp3", &[0u8; 200]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 2);
    assert_eq!(report.total_bytes, 300);
}

// ===========================================================================
// 2. Case-insensitive extension matching
// ===========================================================================
#[test]
fn extensions_case_insensitive() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "IMG.PNG", &[0u8; 50]),
        write_file(tmp.path(), "VID.MP4", &[0u8; 80]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 2);
    assert_eq!(report.categories.len(), 2);
}

// ===========================================================================
// 3. Deeply nested asset paths normalized
// ===========================================================================
#[test]
fn deeply_nested_asset_paths_normalized() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "a/b/c/d/icon.svg", &[0u8; 32]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.top_files[0].path, "a/b/c/d/icon.svg");
    assert!(!report.top_files[0].path.contains('\\'));
}

// ===========================================================================
// 4. Categories sorted by bytes descending, ties broken by name
// ===========================================================================
#[test]
fn categories_deterministic_sort_tiebreak() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.ttf", &[0u8; 50]),
        write_file(tmp.path(), "b.mp3", &[0u8; 50]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 2);
    // Same bytes → alphabetical: audio < font
    assert_eq!(report.categories[0].category, "audio");
    assert_eq!(report.categories[1].category, "font");
}

// ===========================================================================
// 5. Top files tiebreak by path ascending
// ===========================================================================
#[test]
fn top_files_tiebreak_by_path() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "z.png", &[0u8; 100]),
        write_file(tmp.path(), "a.png", &[0u8; 100]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].path, "a.png");
    assert_eq!(report.top_files[1].path, "z.png");
}

// ===========================================================================
// 6. Extension field on AssetFileRow is lowercase
// ===========================================================================
#[test]
fn asset_file_row_extension_lowercase() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "pic.PNG", &[0u8; 10])];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].extension, "png");
}

// ===========================================================================
// 7. Category field on AssetFileRow matches expected string
// ===========================================================================
#[test]
fn asset_file_row_category_correct() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "app.exe", &[0u8; 10])];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].category, "binary");
}

// ===========================================================================
// 8. All six categories discoverable in single report
// ===========================================================================
#[test]
fn all_six_categories_in_one_report() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),  // image
        write_file(tmp.path(), "b.mp4", &[0u8; 10]),  // video
        write_file(tmp.path(), "c.wav", &[0u8; 10]),  // audio
        write_file(tmp.path(), "d.zip", &[0u8; 10]),  // archive
        write_file(tmp.path(), "e.dll", &[0u8; 10]),  // binary
        write_file(tmp.path(), "f.woff", &[0u8; 10]), // font
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 6);
    let names: Vec<&str> = report
        .categories
        .iter()
        .map(|c| c.category.as_str())
        .collect();
    for cat in &["image", "video", "audio", "archive", "binary", "font"] {
        assert!(names.contains(cat), "missing category {cat}");
    }
}

// ===========================================================================
// 9. Multiple extensions aggregated into same category
// ===========================================================================
#[test]
fn multiple_extensions_same_category() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "b.jpg", &[0u8; 20]),
        write_file(tmp.path(), "c.gif", &[0u8; 30]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].files, 3);
    assert_eq!(report.categories[0].bytes, 60);
}

// ===========================================================================
// 10. Dependency report with invalid JSON in package-lock
// ===========================================================================
#[test]
fn dependency_npm_invalid_json() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "package-lock.json", b"NOT JSON");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 11. Dependency report path normalized
// ===========================================================================
#[test]
fn dependency_lockfile_path_normalized() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"x\"\n";
    let rel = write_file(tmp.path(), "sub/Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].path, "sub/Cargo.lock");
    assert!(!report.lockfiles[0].path.contains('\\'));
}

// ===========================================================================
// 12. Cargo.lock with many packages
// ===========================================================================
#[test]
fn dependency_cargo_lock_many_packages() {
    let tmp = TempDir::new().unwrap();
    let mut content = String::new();
    use std::fmt::Write;
    for i in 0..50 {
        let _ = write!(content, "[[package]]\nname = \"pkg-{i}\"\n\n");
    }
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 50);
    assert_eq!(report.total, 50);
}

// ===========================================================================
// 13. go.sum deduplicates by module@version
// ===========================================================================
#[test]
fn dependency_go_sum_deduplicates() {
    let tmp = TempDir::new().unwrap();
    let content = "\
github.com/a/b v1.0.0 h1:aaa=\n\
github.com/a/b v1.0.0 h1:bbb=\n\
github.com/c/d v2.0.0 h1:ccc=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // a/b@v1.0.0 appears twice but counted once; c/d@v2.0.0 once
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 14. Gemfile.lock with nested dependencies
// ===========================================================================
#[test]
fn dependency_gemfile_lock_nested() {
    let tmp = TempDir::new().unwrap();
    // The parser counts lines that start with 4 spaces and contain '('.
    // Sub-dependencies have 6+ spaces so they're also counted.
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0.0)\n      actionpack (= 7.0.0)\n    rack (2.2.0)\n\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // rails + actionpack (sub-dep with 6 spaces still starts_with 4 spaces) + rack = 3
    assert_eq!(report.lockfiles[0].dependencies, 3);
}

// ===========================================================================
// 15. Assets report deterministic across repeated calls
// ===========================================================================
#[test]
fn assets_deterministic_ordering() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "c.zip", &[0u8; 30]),
        write_file(tmp.path(), "a.png", &[0u8; 20]),
        write_file(tmp.path(), "b.mp3", &[0u8; 10]),
    ];
    let r1 = build_assets_report(tmp.path(), &files).unwrap();
    let r2 = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(r1.categories.len(), r2.categories.len());
    for (a, b) in r1.categories.iter().zip(r2.categories.iter()) {
        assert_eq!(a.category, b.category);
        assert_eq!(a.bytes, b.bytes);
    }
}

// ===========================================================================
// 16. Dependency report deterministic
// ===========================================================================
#[test]
fn dependency_deterministic_ordering() {
    let tmp = TempDir::new().unwrap();
    let cargo = "[[package]]\nname = \"z\"\n\n[[package]]\nname = \"a\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let r1 = build_dependency_report(tmp.path(), std::slice::from_ref(&rel)).unwrap();
    let r2 = build_dependency_report(tmp.path(), std::slice::from_ref(&rel)).unwrap();
    assert_eq!(r1.total, r2.total);
    assert_eq!(r1.lockfiles[0].dependencies, r2.lockfiles[0].dependencies);
}

// ===========================================================================
// 17. Yarn.lock with comment-only content
// ===========================================================================
#[test]
fn dependency_yarn_lock_comments_only() {
    let tmp = TempDir::new().unwrap();
    let content = "# THIS IS AN AUTOGENERATED FILE.\n# yarn lockfile v1\n\n";
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 18. pnpm-lock with no package lines
// ===========================================================================
#[test]
fn dependency_pnpm_lock_empty_packages() {
    let tmp = TempDir::new().unwrap();
    let content = "lockfileVersion: 5.4\n\npackages:\n";
    let rel = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 19. Assets with zero-byte file (extension recognized)
// ===========================================================================
#[test]
fn assets_zero_byte_file() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "empty.png", &[])];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 1);
    assert_eq!(report.total_bytes, 0);
}

// ===========================================================================
// 20. package-lock.json with only root key
// ===========================================================================
#[test]
fn dependency_npm_only_root_key() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{"packages": {"": {"name": "myapp"}}}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 21. Exactly 10 top files returned when there are 10 assets
// ===========================================================================
#[test]
fn top_files_exactly_ten() {
    let tmp = TempDir::new().unwrap();
    let files: Vec<PathBuf> = (0..10)
        .map(|i| write_file(tmp.path(), &format!("f{i}.png"), &vec![0u8; (i + 1) * 5]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files.len(), 10);
}

// ===========================================================================
// 22. Fewer than 10 assets returns all
// ===========================================================================
#[test]
fn top_files_fewer_than_ten() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "b.mp4", &[0u8; 20]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files.len(), 2);
}
