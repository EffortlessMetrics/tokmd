//! W57 depth tests for `tokmd-analysis-assets`.
//!
//! Covers asset detection across all six categories, lockfile parsing for
//! every supported format, classification edge cases, report generation
//! with mixed types, deterministic ordering, and serde roundtrips.

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tokmd_analysis_assets::{build_assets_report, build_dependency_report};
use tokmd_analysis_types::{
    AssetCategoryRow, AssetFileRow, AssetReport, DependencyReport, LockfileReport,
};

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
// 1. Image category covers all 9 extensions
// ===========================================================================
#[test]
fn image_all_extensions() {
    let tmp = TempDir::new().unwrap();
    let exts = [
        "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "tiff", "ico",
    ];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|e| write_file(tmp.path(), &format!("img.{e}"), &[0u8; 16]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "image");
    assert_eq!(report.categories[0].files, 9);
    assert_eq!(report.categories[0].extensions.len(), 9);
}

// ===========================================================================
// 2. Video extensions: mpeg and mpg
// ===========================================================================
#[test]
fn video_mpeg_and_mpg() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.mpeg", &[0u8; 10]),
        write_file(tmp.path(), "b.mpg", &[0u8; 10]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories[0].category, "video");
    assert_eq!(report.categories[0].files, 2);
}

// ===========================================================================
// 3. Audio category covers m4a
// ===========================================================================
#[test]
fn audio_m4a_extension() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "track.m4a", &[0u8; 10])];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories[0].category, "audio");
}

// ===========================================================================
// 4. Archive: xz and 7z
// ===========================================================================
#[test]
fn archive_xz_and_7z() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.xz", &[0u8; 10]),
        write_file(tmp.path(), "b.7z", &[0u8; 10]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories[0].category, "archive");
    assert_eq!(report.categories[0].files, 2);
}

// ===========================================================================
// 5. Binary: jar and class
// ===========================================================================
#[test]
fn binary_jar_and_class() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "app.jar", &[0u8; 10]),
        write_file(tmp.path(), "Main.class", &[0u8; 10]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories[0].category, "binary");
    assert_eq!(report.categories[0].files, 2);
}

// ===========================================================================
// 6. Font: woff and woff2
// ===========================================================================
#[test]
fn font_woff_woff2() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.woff", &[0u8; 10]),
        write_file(tmp.path(), "b.woff2", &[0u8; 10]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories[0].category, "font");
    assert_eq!(report.categories[0].files, 2);
}

// ===========================================================================
// 7. Unrecognised extensions are skipped
// ===========================================================================
#[test]
fn unrecognised_extensions_skipped() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.rs", b"fn main() {}"),
        write_file(tmp.path(), "b.py", b"print()"),
        write_file(tmp.path(), "c.toml", b"[package]"),
        write_file(tmp.path(), "d.xml", b"<root/>"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
}

// ===========================================================================
// 8. Files without extension are skipped
// ===========================================================================
#[test]
fn no_extension_files_skipped() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "Makefile", b"all:"),
        write_file(tmp.path(), "LICENSE", b"MIT"),
        write_file(tmp.path(), "Dockerfile", b"FROM rust"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 0);
}

// ===========================================================================
// 9. Empty file list → zero everything
// ===========================================================================
#[test]
fn empty_file_list_assets() {
    let tmp = TempDir::new().unwrap();
    let report = build_assets_report(tmp.path(), &[]).unwrap();
    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
    assert!(report.top_files.is_empty());
}

// ===========================================================================
// 10. Mixed categories: bytes totals are correct
// ===========================================================================
#[test]
fn mixed_categories_bytes_sum() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "b.mp4", &[0u8; 200]),
        write_file(tmp.path(), "c.mp3", &[0u8; 50]),
        write_file(tmp.path(), "d.zip", &[0u8; 150]),
        write_file(tmp.path(), "e.exe", &[0u8; 75]),
        write_file(tmp.path(), "f.ttf", &[0u8; 25]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 6);
    assert_eq!(report.total_bytes, 600);
    let sum: u64 = report.categories.iter().map(|c| c.bytes).sum();
    assert_eq!(report.total_bytes, sum);
}

// ===========================================================================
// 11. Categories sorted by bytes desc, then name asc
// ===========================================================================
#[test]
fn categories_sort_order() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.ttf", &[0u8; 100]), // font
        write_file(tmp.path(), "b.exe", &[0u8; 100]), // binary
        write_file(tmp.path(), "c.mp3", &[0u8; 200]), // audio
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories[0].category, "audio"); // 200
    assert_eq!(report.categories[1].category, "binary"); // 100
    assert_eq!(report.categories[2].category, "font"); // 100
}

// ===========================================================================
// 12. Top files capped at 10
// ===========================================================================
#[test]
fn top_files_cap_at_10() {
    let tmp = TempDir::new().unwrap();
    let files: Vec<PathBuf> = (0..20)
        .map(|i| write_file(tmp.path(), &format!("f{i}.png"), &vec![0u8; (i + 1) * 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files.len(), 10);
    // First file has the most bytes
    assert!(report.top_files[0].bytes >= report.top_files[9].bytes);
}

// ===========================================================================
// 13. Top files tiebreak by path ascending
// ===========================================================================
#[test]
fn top_files_tiebreak_path_asc() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "z.png", &[0u8; 50]),
        write_file(tmp.path(), "a.png", &[0u8; 50]),
        write_file(tmp.path(), "m.png", &[0u8; 50]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].path, "a.png");
    assert_eq!(report.top_files[1].path, "m.png");
    assert_eq!(report.top_files[2].path, "z.png");
}

// ===========================================================================
// 14. Paths normalized to forward slashes
// ===========================================================================
#[test]
fn paths_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "deep/nested/dir/logo.svg", &[0u8; 32]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.top_files[0].path, "deep/nested/dir/logo.svg");
    assert!(!report.top_files[0].path.contains('\\'));
}

// ===========================================================================
// 15. Extension stored lowercase
// ===========================================================================
#[test]
fn extension_lowercase() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "IMG.PNG", &[0u8; 10])];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].extension, "png");
}

// ===========================================================================
// 16. AssetReport serde roundtrip
// ===========================================================================
#[test]
fn asset_report_serde_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "b.mp4", &[0u8; 200]),
        write_file(tmp.path(), "c.zip", &[0u8; 50]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    let rt: AssetReport = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.total_files, report.total_files);
    assert_eq!(rt.total_bytes, report.total_bytes);
    assert_eq!(rt.categories.len(), report.categories.len());
    assert_eq!(rt.top_files.len(), report.top_files.len());
}

// ===========================================================================
// 17. AssetCategoryRow serde roundtrip
// ===========================================================================
#[test]
fn asset_category_row_serde_roundtrip() {
    let row = AssetCategoryRow {
        category: "image".to_string(),
        files: 5,
        bytes: 1024,
        extensions: vec!["png".to_string(), "jpg".to_string()],
    };
    let json = serde_json::to_string(&row).unwrap();
    let rt: AssetCategoryRow = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.category, "image");
    assert_eq!(rt.files, 5);
    assert_eq!(rt.bytes, 1024);
    assert_eq!(rt.extensions, vec!["png", "jpg"]);
}

// ===========================================================================
// 18. AssetFileRow serde roundtrip
// ===========================================================================
#[test]
fn asset_file_row_serde_roundtrip() {
    let row = AssetFileRow {
        path: "icons/logo.svg".to_string(),
        bytes: 2048,
        category: "image".to_string(),
        extension: "svg".to_string(),
    };
    let json = serde_json::to_string(&row).unwrap();
    let rt: AssetFileRow = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.path, "icons/logo.svg");
    assert_eq!(rt.bytes, 2048);
    assert_eq!(rt.category, "image");
    assert_eq!(rt.extension, "svg");
}

// ===========================================================================
// 19. DependencyReport serde roundtrip
// ===========================================================================
#[test]
fn dependency_report_serde_roundtrip() {
    let report = DependencyReport {
        total: 5,
        lockfiles: vec![
            LockfileReport {
                path: "Cargo.lock".to_string(),
                kind: "cargo".to_string(),
                dependencies: 3,
            },
            LockfileReport {
                path: "yarn.lock".to_string(),
                kind: "yarn".to_string(),
                dependencies: 2,
            },
        ],
    };
    let json = serde_json::to_string(&report).unwrap();
    let rt: DependencyReport = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.total, 5);
    assert_eq!(rt.lockfiles.len(), 2);
    assert_eq!(rt.lockfiles[0].kind, "cargo");
    assert_eq!(rt.lockfiles[1].kind, "yarn");
}

// ===========================================================================
// 20. LockfileReport serde roundtrip
// ===========================================================================
#[test]
fn lockfile_report_serde_roundtrip() {
    let lf = LockfileReport {
        path: "sub/Cargo.lock".to_string(),
        kind: "cargo".to_string(),
        dependencies: 42,
    };
    let json = serde_json::to_string(&lf).unwrap();
    let rt: LockfileReport = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.path, "sub/Cargo.lock");
    assert_eq!(rt.kind, "cargo");
    assert_eq!(rt.dependencies, 42);
}

// ===========================================================================
// 21. Cargo.lock counting: text containing "[[package]]" inside values
// ===========================================================================
#[test]
fn cargo_lock_counts_only_package_markers() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"serde\"\nversion = \"1.0\"\n\n\
                   [[package]]\nname = \"tokei\"\nversion = \"12.0\"\n\n\
                   [[package]]\nname = \"clap\"\nversion = \"4.0\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 3);
    assert_eq!(report.total, 3);
}

// ===========================================================================
// 22. package-lock.json: packages field preferred over dependencies
// ===========================================================================
#[test]
fn npm_packages_field_preferred() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{
        "packages": {"": {}, "node_modules/a": {}, "node_modules/b": {}, "node_modules/c": {}},
        "dependencies": {"x": {}, "y": {}}
    }"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // packages: 4 keys - 1 root = 3 (not 2 from dependencies)
    assert_eq!(report.lockfiles[0].dependencies, 3);
}

// ===========================================================================
// 23. yarn.lock: multiple versions of same package
// ===========================================================================
#[test]
fn yarn_lock_multiple_versions() {
    let tmp = TempDir::new().unwrap();
    let content = "# yarn lockfile v1\n\n\
                   lodash@^4.0.0:\n  version \"4.17.21\"\n\n\
                   lodash@^3.0.0:\n  version \"3.10.1\"\n\n\
                   react@^18.0.0:\n  version \"18.2.0\"\n";
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "yarn");
    assert_eq!(report.lockfiles[0].dependencies, 3);
}

// ===========================================================================
// 24. go.sum: empty file
// ===========================================================================
#[test]
fn go_sum_empty() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "go.sum", b"");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "go");
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 25. Gemfile.lock: multiple specs sections
// ===========================================================================
#[test]
fn gemfile_lock_multiple_specs() {
    let tmp = TempDir::new().unwrap();
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0.0)\n    rack (2.2.0)\n\nPATH\n  remote: .\n  specs:\n    myapp (1.0.0)\n\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "bundler");
    // rails + rack in GEM specs, myapp in PATH specs
    assert!(report.lockfiles[0].dependencies >= 2);
}

// ===========================================================================
// 26. Multiple lockfiles: total equals sum
// ===========================================================================
#[test]
fn multiple_lockfiles_total_sum() {
    let tmp = TempDir::new().unwrap();
    let cargo = "[[package]]\nname = \"a\"\n\n[[package]]\nname = \"b\"\n";
    let yarn = "# yarn\npkg@^1:\n  version \"1.0\"\n";
    let go = "example.com/x v1.0.0 h1:abc=\n";
    let f1 = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let f2 = write_file(tmp.path(), "yarn.lock", yarn.as_bytes());
    let f3 = write_file(tmp.path(), "go.sum", go.as_bytes());
    let report = build_dependency_report(tmp.path(), &[f1, f2, f3]).unwrap();
    let sum: usize = report.lockfiles.iter().map(|l| l.dependencies).sum();
    assert_eq!(report.total, sum);
    assert_eq!(report.lockfiles.len(), 3);
}

// ===========================================================================
// 27. Non-lockfile names ignored by dependency report
// ===========================================================================
#[test]
fn non_lockfile_names_ignored() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "README.md", b"# Hello"),
        write_file(tmp.path(), "package.json", b"{}"),
        write_file(tmp.path(), "Cargo.toml", b"[package]"),
    ];
    let report = build_dependency_report(tmp.path(), &files).unwrap();
    assert!(report.lockfiles.is_empty());
    assert_eq!(report.total, 0);
}

// ===========================================================================
// 28. Empty dependency report
// ===========================================================================
#[test]
fn empty_dependency_report() {
    let tmp = TempDir::new().unwrap();
    let report = build_dependency_report(tmp.path(), &[]).unwrap();
    assert_eq!(report.total, 0);
    assert!(report.lockfiles.is_empty());
}

// ===========================================================================
// 29. Deterministic: same input → same JSON
// ===========================================================================
#[test]
fn asset_report_deterministic_json() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "z.png", &[0u8; 50]),
        write_file(tmp.path(), "a.mp4", &[0u8; 100]),
        write_file(tmp.path(), "m.zip", &[0u8; 75]),
        write_file(tmp.path(), "x.ttf", &[0u8; 25]),
    ];
    let j1 = serde_json::to_string(&build_assets_report(tmp.path(), &files).unwrap()).unwrap();
    let j2 = serde_json::to_string(&build_assets_report(tmp.path(), &files).unwrap()).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 30. Deterministic dependency report
// ===========================================================================
#[test]
fn dependency_report_deterministic_json() {
    let tmp = TempDir::new().unwrap();
    let cargo = "[[package]]\nname = \"z\"\n\n[[package]]\nname = \"a\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let j1 = serde_json::to_string(&build_dependency_report(tmp.path(), std::slice::from_ref(&rel)).unwrap())
        .unwrap();
    let j2 = serde_json::to_string(&build_dependency_report(tmp.path(), &[rel]).unwrap()).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 31. Zero-byte asset file is still counted
// ===========================================================================
#[test]
fn zero_byte_asset_counted() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "empty.png", &[])];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 1);
    assert_eq!(report.total_bytes, 0);
    assert_eq!(report.top_files.len(), 1);
    assert_eq!(report.top_files[0].bytes, 0);
}

// ===========================================================================
// 32. Missing lockfile on disk gives zero dependencies
// ===========================================================================
#[test]
fn missing_lockfile_zero_deps() {
    let tmp = TempDir::new().unwrap();
    // Don't write the file on disk
    let rel = PathBuf::from("Cargo.lock");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 33. JSON contains expected keys for asset report
// ===========================================================================
#[test]
fn asset_report_json_keys() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "a.png", &[0u8; 10])];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val.get("total_files").is_some());
    assert!(val.get("total_bytes").is_some());
    assert!(val.get("categories").is_some());
    assert!(val.get("top_files").is_some());
}

// ===========================================================================
// 34. JSON contains expected keys for dependency report
// ===========================================================================
#[test]
fn dependency_report_json_keys() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "Cargo.lock", b"[[package]]\nname = \"x\"\n");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val.get("total").is_some());
    assert!(val.get("lockfiles").is_some());
    let lf = &val["lockfiles"][0];
    assert!(lf.get("path").is_some());
    assert!(lf.get("kind").is_some());
    assert!(lf.get("dependencies").is_some());
}

// ===========================================================================
// 35. pnpm-lock.yaml with many packages
// ===========================================================================
#[test]
fn pnpm_lock_many_packages() {
    let tmp = TempDir::new().unwrap();
    let mut content = String::from("lockfileVersion: 5.4\n\npackages:\n");
    for i in 0..20 {
        content.push_str(&format!(
            "  /pkg-{i}/1.0.{i}:\n    resolution: {{integrity: sha512-xxx}}\n"
        ));
    }
    let rel = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "pnpm");
    // Lines starting with "/" and containing ":"
    assert_eq!(report.lockfiles[0].dependencies, 20);
}
