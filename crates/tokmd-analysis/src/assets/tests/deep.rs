//! Deep tests for `analysis assets module`.
//!
//! Exercises build_assets_report and build_dependency_report with various
//! file types, lockfile formats, edge cases, and serialization roundtrips.

use std::path::{Path, PathBuf};

use crate::assets::{build_assets_report, build_dependency_report};
use tempfile::TempDir;

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
// 1. Empty file list → zero totals
// ===========================================================================
#[test]
fn assets_empty_files() {
    let tmp = TempDir::new().unwrap();
    let report = build_assets_report(tmp.path(), &[]).unwrap();
    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
    assert!(report.top_files.is_empty());
}

// ===========================================================================
// 2. Files with no extension are skipped
// ===========================================================================
#[test]
fn assets_no_extension_skipped() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "Makefile", b"all: build");
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.total_files, 0);
}

// ===========================================================================
// 3. Files with unrecognized extension are skipped
// ===========================================================================
#[test]
fn assets_unknown_extension_skipped() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "data.csv", b"a,b,c");
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.total_files, 0);
}

// ===========================================================================
// 4. Image category: png, jpg, gif, svg, webp, bmp, ico
// ===========================================================================
#[test]
fn assets_image_extensions() {
    let tmp = TempDir::new().unwrap();
    let exts = [
        "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "tiff", "ico",
    ];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("img.{ext}"), &[0u8; 64]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, exts.len());
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "image");
    assert_eq!(report.categories[0].files, exts.len());
}

// ===========================================================================
// 5. Video category
// ===========================================================================
#[test]
fn assets_video_extensions() {
    let tmp = TempDir::new().unwrap();
    let exts = ["mp4", "mov", "avi", "mkv", "webm"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("vid.{ext}"), &[0u8; 128]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "video");
}

// ===========================================================================
// 6. Audio category
// ===========================================================================
#[test]
fn assets_audio_extensions() {
    let tmp = TempDir::new().unwrap();
    let exts = ["mp3", "wav", "flac", "ogg", "aac", "m4a"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("snd.{ext}"), &[0u8; 32]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "audio");
}

// ===========================================================================
// 7. Archive category
// ===========================================================================
#[test]
fn assets_archive_extensions() {
    let tmp = TempDir::new().unwrap();
    let exts = ["zip", "tar", "gz", "bz2", "xz", "7z", "rar"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("arch.{ext}"), &[0u8; 16]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "archive");
}

// ===========================================================================
// 8. Binary category
// ===========================================================================
#[test]
fn assets_binary_extensions() {
    let tmp = TempDir::new().unwrap();
    let exts = ["exe", "dll", "so", "dylib", "bin", "class", "jar"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("prog.{ext}"), &[0u8; 48]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "binary");
}

// ===========================================================================
// 9. Font category
// ===========================================================================
#[test]
fn assets_font_extensions() {
    let tmp = TempDir::new().unwrap();
    let exts = ["ttf", "otf", "woff", "woff2"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("font.{ext}"), &[0u8; 80]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "font");
}

// ===========================================================================
// 10. Multiple categories in one report
// ===========================================================================
#[test]
fn assets_multiple_categories() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "img.png", &[0u8; 100]),
        write_file(tmp.path(), "vid.mp4", &[0u8; 200]),
        write_file(tmp.path(), "snd.mp3", &[0u8; 50]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 3);
    assert_eq!(report.total_bytes, 350);
    assert_eq!(report.categories.len(), 3);
    // Sorted by bytes desc: video(200), image(100), audio(50)
    assert_eq!(report.categories[0].category, "video");
    assert_eq!(report.categories[1].category, "image");
    assert_eq!(report.categories[2].category, "audio");
}

// ===========================================================================
// 11. Top files limited to 10
// ===========================================================================
#[test]
fn assets_top_files_capped_at_ten() {
    let tmp = TempDir::new().unwrap();
    let files: Vec<PathBuf> = (0..15)
        .map(|i| write_file(tmp.path(), &format!("img{i}.png"), &vec![0u8; (i + 1) * 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files.len(), 10);
    // Sorted by bytes desc
    assert!(report.top_files[0].bytes >= report.top_files[9].bytes);
}

// ===========================================================================
// 12. Top files sorted by bytes descending
// ===========================================================================
#[test]
fn assets_top_files_sorted_desc() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "small.png", &[0u8; 10]),
        write_file(tmp.path(), "large.png", &[0u8; 1000]),
        write_file(tmp.path(), "medium.png", &[0u8; 500]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].path, "large.png");
    assert_eq!(report.top_files[1].path, "medium.png");
    assert_eq!(report.top_files[2].path, "small.png");
}

// ===========================================================================
// 13. AssetReport JSON serialization roundtrip
// ===========================================================================
#[test]
fn assets_report_serialization_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "logo.png", &[0u8; 256]),
        write_file(tmp.path(), "data.zip", &[0u8; 512]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::AssetReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total_files, report.total_files);
    assert_eq!(deser.total_bytes, report.total_bytes);
    assert_eq!(deser.categories.len(), report.categories.len());
}

// ===========================================================================
// 14. Empty dependency report
// ===========================================================================
#[test]
fn dependency_empty() {
    let tmp = TempDir::new().unwrap();
    let report = build_dependency_report(tmp.path(), &[]).unwrap();
    assert_eq!(report.total, 0);
    assert!(report.lockfiles.is_empty());
}

// ===========================================================================
// 15. Cargo.lock counting
// ===========================================================================
#[test]
fn dependency_cargo_lock() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"a\"\nversion = \"1.0\"\n\n[[package]]\nname = \"b\"\nversion = \"2.0\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles.len(), 1);
    assert_eq!(report.lockfiles[0].kind, "cargo");
    assert_eq!(report.lockfiles[0].dependencies, 2);
    assert_eq!(report.total, 2);
}

// ===========================================================================
// 16. package-lock.json with "packages" field
// ===========================================================================
#[test]
fn dependency_npm_packages() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{"packages": {"": {}, "node_modules/a": {}, "node_modules/b": {}}}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "npm");
    // 3 keys minus 1 for root "" = 2
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 17. package-lock.json with "dependencies" field (legacy)
// ===========================================================================
#[test]
fn dependency_npm_legacy() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{"dependencies": {"a": {"version": "1.0"}, "b": {"version": "2.0"}}}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 18. yarn.lock counting
// ===========================================================================
#[test]
fn dependency_yarn_lock() {
    let tmp = TempDir::new().unwrap();
    let content = "# yarn lockfile v1\n\npackage-a@^1.0.0:\n  version \"1.0.0\"\n\npackage-b@^2.0.0:\n  version \"2.0.0\"\n";
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "yarn");
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 19. go.sum counting (dedup /go.mod entries)
// ===========================================================================
#[test]
fn dependency_go_sum() {
    let tmp = TempDir::new().unwrap();
    let content = "github.com/foo/bar v1.0.0 h1:abc=\ngithub.com/foo/bar v1.0.0/go.mod h1:def=\ngithub.com/baz/qux v2.0.0 h1:ghi=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "go");
    // foo/bar v1.0.0 counted once (go.mod line skipped), baz/qux once
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 20. Gemfile.lock counting
// ===========================================================================
#[test]
fn dependency_gemfile_lock() {
    let tmp = TempDir::new().unwrap();
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0.0)\n    rack (2.2.0)\n\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "bundler");
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 21. Multiple lockfiles summed in total
// ===========================================================================
#[test]
fn dependency_multiple_lockfiles() {
    let tmp = TempDir::new().unwrap();
    let cargo =
        "[[package]]\nname = \"a\"\n\n[[package]]\nname = \"b\"\n\n[[package]]\nname = \"c\"\n";
    let npm = r#"{"packages": {"": {}, "node_modules/x": {}}}"#;
    let f1 = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let f2 = write_file(tmp.path(), "package-lock.json", npm.as_bytes());
    let report = build_dependency_report(tmp.path(), &[f1, f2]).unwrap();
    assert_eq!(report.lockfiles.len(), 2);
    assert_eq!(report.total, 4); // 3 cargo + 1 npm
}

// ===========================================================================
// 22. Non-lockfile names ignored by dependency report
// ===========================================================================
#[test]
fn dependency_ignores_non_lockfiles() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "README.md", b"# Hello");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert!(report.lockfiles.is_empty());
    assert_eq!(report.total, 0);
}

// ===========================================================================
// 23. DependencyReport JSON serialization roundtrip
// ===========================================================================
#[test]
fn dependency_report_serialization_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"serde\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let deser: tokmd_analysis_types::DependencyReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total, report.total);
    assert_eq!(deser.lockfiles.len(), report.lockfiles.len());
}

// ===========================================================================
// 24. Asset paths normalized to forward slashes
// ===========================================================================
#[test]
fn assets_path_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "sub/dir/img.png", &[0u8; 32]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.top_files[0].path, "sub/dir/img.png");
    assert!(!report.top_files[0].path.contains('\\'));
}

// ===========================================================================
// 25. Category extensions list contains all seen extensions
// ===========================================================================
#[test]
fn assets_category_extensions_complete() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "b.jpg", &[0u8; 20]),
        write_file(tmp.path(), "c.gif", &[0u8; 30]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    let exts = &report.categories[0].extensions;
    assert!(exts.contains(&"png".to_string()));
    assert!(exts.contains(&"jpg".to_string()));
    assert!(exts.contains(&"gif".to_string()));
}

// ===========================================================================
// 26. pnpm-lock.yaml counting
// ===========================================================================
#[test]
fn dependency_pnpm_lock() {
    let tmp = TempDir::new().unwrap();
    let content = "lockfileVersion: 5.4\n\npackages:\n  /react/18.2.0:\n    resolution: {integrity: sha512-xxx}\n  /react-dom/18.2.0:\n    resolution: {integrity: sha512-yyy}\n";
    let rel = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "pnpm");
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 27. Cargo.lock with zero packages
// ===========================================================================
#[test]
fn dependency_cargo_lock_empty() {
    let tmp = TempDir::new().unwrap();
    let content = "# This is an empty lockfile\n";
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}
