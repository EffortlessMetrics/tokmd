//! Deeper tests for asset file detection, dependency lockfile detection, and edge cases.

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
// No assets at all — empty directory
// ===========================================================================

#[test]
fn empty_file_list_produces_zero_totals() {
    let tmp = TempDir::new().unwrap();
    let report = build_assets_report(tmp.path(), &[]).unwrap();

    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
    assert!(report.top_files.is_empty());
}

// ===========================================================================
// Mixed asset types in single directory
// ===========================================================================

#[test]
fn mixed_asset_types_all_detected_in_flat_directory() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "logo.png", &[0u8; 100]),
        write_file(tmp.path(), "intro.mp4", &[0u8; 500]),
        write_file(tmp.path(), "alert.wav", &[0u8; 200]),
        write_file(tmp.path(), "backup.tar", &[0u8; 300]),
        write_file(tmp.path(), "helper.dll", &[0u8; 150]),
        write_file(tmp.path(), "body.ttf", &[0u8; 80]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 6);
    assert_eq!(report.total_bytes, 1330);

    let categories: Vec<&str> = report.categories.iter().map(|c| c.category.as_str()).collect();
    assert!(categories.contains(&"image"));
    assert!(categories.contains(&"video"));
    assert!(categories.contains(&"audio"));
    assert!(categories.contains(&"archive"));
    assert!(categories.contains(&"binary"));
    assert!(categories.contains(&"font"));
}

// ===========================================================================
// Assets in deeply nested directories
// ===========================================================================

#[test]
fn nested_directory_assets_detected_with_normalized_paths() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "assets/images/icons/logo.png", &[0u8; 64]),
        write_file(tmp.path(), "assets/images/photos/bg.jpg", &[0u8; 128]),
        write_file(tmp.path(), "public/fonts/main.woff2", &[0u8; 32]),
        write_file(tmp.path(), "vendor/bin/tool.exe", &[0u8; 256]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 4);

    // All paths should use forward slashes
    for f in &report.top_files {
        assert!(
            !f.path.contains('\\'),
            "path should use forward slashes: {}",
            f.path
        );
    }

    // Verify nested paths are preserved
    let paths: Vec<&str> = report.top_files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"vendor/bin/tool.exe"));
    assert!(paths.contains(&"assets/images/photos/bg.jpg"));
}

// ===========================================================================
// Same extension in different directories → aggregated under one category
// ===========================================================================

#[test]
fn same_extension_across_directories_aggregated() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "dir_a/photo.png", &[0u8; 100]),
        write_file(tmp.path(), "dir_b/icon.png", &[0u8; 50]),
        write_file(tmp.path(), "dir_c/banner.png", &[0u8; 200]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "image");
    assert_eq!(report.categories[0].files, 3);
    assert_eq!(report.categories[0].bytes, 350);
    assert_eq!(report.categories[0].extensions, vec!["png"]);
}

// ===========================================================================
// Zero-byte asset files are counted but with zero bytes
// ===========================================================================

#[test]
fn zero_byte_asset_files_counted_with_zero_bytes() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "empty.png", &[]),
        write_file(tmp.path(), "also_empty.jpg", &[]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 2);
    assert_eq!(report.total_bytes, 0);
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].files, 2);
}

// ===========================================================================
// Only non-asset files → nothing detected
// ===========================================================================

#[test]
fn only_source_files_produce_empty_asset_report() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "main.rs", b"fn main() {}"),
        write_file(tmp.path(), "lib.py", b"print('hello')"),
        write_file(tmp.path(), "index.html", b"<html></html>"),
        write_file(tmp.path(), "style.css", b"body {}"),
        write_file(tmp.path(), "app.js", b"console.log('hi')"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 0);
    assert!(report.categories.is_empty());
}

// ===========================================================================
// Mixed known and unknown extensions — only known counted
// ===========================================================================

#[test]
fn mixed_extensions_only_asset_types_counted() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "README.md", b"# readme"),
        write_file(tmp.path(), "logo.svg", &[0u8; 80]),
        write_file(tmp.path(), "Cargo.toml", b"[package]"),
        write_file(tmp.path(), "clip.webm", &[0u8; 400]),
        write_file(tmp.path(), "test.rs", b"#[test]"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 2); // svg + webm
    assert_eq!(report.total_bytes, 480);
}

// ===========================================================================
// Categories sorted by bytes descending, then by name
// ===========================================================================

#[test]
fn categories_sorted_by_bytes_desc_then_name() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),   // image: 100
        write_file(tmp.path(), "b.mp4", &[0u8; 100]),   // video: 100 (same bytes, "video" > "image" alphabetically)
        write_file(tmp.path(), "c.zip", &[0u8; 500]),   // archive: 500
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    let order: Vec<&str> = report.categories.iter().map(|c| c.category.as_str()).collect();
    // archive (500) first, then image (100) before video (100)
    assert_eq!(order, vec!["archive", "image", "video"]);
}

// ===========================================================================
// Top files sorted by bytes descending then path
// ===========================================================================

#[test]
fn top_files_sorted_by_bytes_desc_then_path_asc() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "b.png", &[0u8; 100]),
        write_file(tmp.path(), "a.png", &[0u8; 100]),   // same bytes, "a" < "b"
        write_file(tmp.path(), "c.png", &[0u8; 200]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.top_files[0].path, "c.png");      // 200 bytes first
    assert_eq!(report.top_files[1].path, "a.png");       // 100 bytes, "a" before "b"
    assert_eq!(report.top_files[2].path, "b.png");
}

// ===========================================================================
// Dependency lockfile detection: multiple lockfiles in nested paths
// ===========================================================================

#[test]
fn dependency_report_detects_lockfiles_in_nested_directories() {
    let tmp = TempDir::new().unwrap();
    let cargo = write_file(
        tmp.path(),
        "backend/Cargo.lock",
        b"[[package]]\nname = \"serde\"\n",
    );
    let yarn = write_file(
        tmp.path(),
        "frontend/yarn.lock",
        b"# yarn lockfile v1\n\nreact@^18:\n  version \"18\"\n",
    );
    let report = build_dependency_report(tmp.path(), &[cargo, yarn]).unwrap();

    assert_eq!(report.lockfiles.len(), 2);
    assert_eq!(report.total, 2); // 1 cargo + 1 yarn

    let paths: Vec<&str> = report.lockfiles.iter().map(|l| l.path.as_str()).collect();
    assert!(paths.contains(&"backend/Cargo.lock"));
    assert!(paths.contains(&"frontend/yarn.lock"));
}

// ===========================================================================
// Dependency report: all lockfile types in one project
// ===========================================================================

#[test]
fn all_lockfile_types_detected_simultaneously() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(
            tmp.path(),
            "Cargo.lock",
            b"[[package]]\nname = \"a\"\n[[package]]\nname = \"b\"\n",
        ),
        write_file(
            tmp.path(),
            "package-lock.json",
            b"{\"packages\":{\"\":{},\"node_modules/x\":{}}}",
        ),
        write_file(
            tmp.path(),
            "yarn.lock",
            b"# yarn lockfile v1\n\nreact@^18:\n  version \"18\"\n",
        ),
        write_file(
            tmp.path(),
            "pnpm-lock.yaml",
            b"lockfileVersion: 5\npackages:\n  /pkg/1.0:\n    resolution: {}\n",
        ),
        write_file(
            tmp.path(),
            "go.sum",
            b"example.com/pkg v1.0.0 h1:abc=\n",
        ),
        write_file(
            tmp.path(),
            "Gemfile.lock",
            b"GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0)\n",
        ),
    ];

    let report = build_dependency_report(tmp.path(), &files).unwrap();

    assert_eq!(report.lockfiles.len(), 6);
    let kinds: Vec<&str> = report.lockfiles.iter().map(|l| l.kind.as_str()).collect();
    assert!(kinds.contains(&"cargo"));
    assert!(kinds.contains(&"npm"));
    assert!(kinds.contains(&"yarn"));
    assert!(kinds.contains(&"pnpm"));
    assert!(kinds.contains(&"go"));
    assert!(kinds.contains(&"bundler"));
}

// ===========================================================================
// Dependency report: total is sum of all lockfile dependency counts
// ===========================================================================

#[test]
fn dependency_total_equals_sum_of_all_lockfiles() {
    let tmp = TempDir::new().unwrap();
    let cargo = write_file(
        tmp.path(),
        "Cargo.lock",
        b"[[package]]\nname=\"a\"\n[[package]]\nname=\"b\"\n[[package]]\nname=\"c\"\n",
    );
    let go = write_file(
        tmp.path(),
        "go.sum",
        b"example.com/x v1.0 h1:abc=\nexample.com/y v2.0 h1:def=\n",
    );
    let report = build_dependency_report(tmp.path(), &[cargo, go]).unwrap();

    let individual_sum: usize = report.lockfiles.iter().map(|l| l.dependencies).sum();
    assert_eq!(report.total, individual_sum);
    assert_eq!(report.total, 5); // 3 cargo + 2 go
}

// ===========================================================================
// Asset report determinism: same inputs → identical outputs
// ===========================================================================

#[test]
fn asset_report_is_deterministic() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "b.mp4", &[0u8; 500]),
        write_file(tmp.path(), "c.woff2", &[0u8; 50]),
    ];

    let r1 = build_assets_report(tmp.path(), &files).unwrap();
    let r2 = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(r1.total_files, r2.total_files);
    assert_eq!(r1.total_bytes, r2.total_bytes);
    assert_eq!(r1.categories.len(), r2.categories.len());
    for (a, b) in r1.categories.iter().zip(r2.categories.iter()) {
        assert_eq!(a.category, b.category);
        assert_eq!(a.files, b.files);
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.extensions, b.extensions);
    }
    assert_eq!(r1.top_files.len(), r2.top_files.len());
    for (a, b) in r1.top_files.iter().zip(r2.top_files.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.bytes, b.bytes);
    }
}

// ===========================================================================
// Dependency report determinism
// ===========================================================================

#[test]
fn dependency_report_is_deterministic() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(
            tmp.path(),
            "Cargo.lock",
            b"[[package]]\nname=\"a\"\n[[package]]\nname=\"b\"\n",
        ),
    ];

    let r1 = build_dependency_report(tmp.path(), &files).unwrap();
    let r2 = build_dependency_report(tmp.path(), &files).unwrap();

    assert_eq!(r1.total, r2.total);
    assert_eq!(r1.lockfiles.len(), r2.lockfiles.len());
    for (a, b) in r1.lockfiles.iter().zip(r2.lockfiles.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.dependencies, b.dependencies);
    }
}

// ===========================================================================
// Files without extensions are ignored completely
// ===========================================================================

#[test]
fn extensionless_files_ignored_in_asset_report() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "Makefile", b"all: build"),
        write_file(tmp.path(), "Dockerfile", b"FROM rust"),
        write_file(tmp.path(), "LICENSE", b"MIT License"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
}

// ===========================================================================
// Large Cargo.lock with many packages
// ===========================================================================

#[test]
fn large_cargo_lock_counts_all_packages() {
    let tmp = TempDir::new().unwrap();
    let mut content = String::new();
    for i in 0..100 {
        content.push_str(&format!("[[package]]\nname = \"pkg-{i}\"\nversion = \"1.0.{i}\"\n\n"));
    }
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].dependencies, 100);
    assert_eq!(report.total, 100);
}

// ===========================================================================
// go.sum deduplicates module+version pairs
// ===========================================================================

#[test]
fn go_sum_deduplicates_same_module_version() {
    let tmp = TempDir::new().unwrap();
    // Duplicate entries (source + go.mod lines) for the same module@version
    let content = "\
example.com/pkg v1.0.0 h1:abc=\n\
example.com/pkg v1.0.0/go.mod h1:def=\n\
example.com/pkg v1.0.0 h1:abc=\n\
example.com/other v2.0.0 h1:ghi=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    // Only 2 unique module@version pairs (go.mod lines skipped, dups merged)
    assert_eq!(report.lockfiles[0].dependencies, 2);
}
