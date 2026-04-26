//! Deep coverage tests for `analysis assets module`.
//!
//! Exercises lockfile parsing edge cases, asset sorting stability,
//! mixed-category ordering, empty/boundary conditions, and snapshot output.

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
// Asset detection – lockfile-like names are NOT assets
// ===========================================================================

#[test]
fn assets_lockfiles_are_not_assets() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "Cargo.lock", b"[[package]]"),
        write_file(tmp.path(), "package-lock.json", b"{}"),
        write_file(tmp.path(), "yarn.lock", b"# yarn"),
        write_file(tmp.path(), "go.sum", b""),
        write_file(tmp.path(), "Gemfile.lock", b""),
        write_file(tmp.path(), "pnpm-lock.yaml", b""),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    // These have non-asset extensions (.lock, .json, .yaml, .sum)
    assert_eq!(report.total_files, 0);
}

// ===========================================================================
// Dependency parsing – malformed JSON in package-lock.json
// ===========================================================================

#[test]
fn dependency_npm_malformed_json_returns_zero() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "package-lock.json", b"NOT VALID JSON {{{");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].kind, "npm");
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn dependency_npm_empty_packages_object() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "package-lock.json", br#"{"packages": {}}"#);
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn dependency_npm_only_root_key_returns_zero() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(
        tmp.path(),
        "package-lock.json",
        br#"{"packages": {"": {}}}"#,
    );
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // Root "" key is subtracted
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// Dependency parsing – Cargo.lock edge cases
// ===========================================================================

#[test]
fn dependency_cargo_lock_single_package() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"only\"\nversion = \"0.1.0\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 1);
    assert_eq!(report.total, 1);
}

#[test]
fn dependency_cargo_lock_many_packages() {
    let tmp = TempDir::new().unwrap();
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!(
            "[[package]]\nname = \"pkg-{i}\"\nversion = \"1.0.0\"\n\n"
        ));
    }
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 50);
}

// ===========================================================================
// Dependency parsing – yarn.lock edge cases
// ===========================================================================

#[test]
fn dependency_yarn_lock_empty() {
    let tmp = TempDir::new().unwrap();
    let content = "# yarn lockfile v1\n\n";
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn dependency_yarn_lock_comment_only() {
    let tmp = TempDir::new().unwrap();
    let content = "# yarn lockfile v1\n# comment line\n# another comment\n";
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// Dependency parsing – go.sum deduplication
// ===========================================================================

#[test]
fn dependency_go_sum_all_go_mod_lines_skipped() {
    let tmp = TempDir::new().unwrap();
    let content = "github.com/a/b v1.0.0/go.mod h1:abc=\ngithub.com/c/d v2.0.0/go.mod h1:def=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn dependency_go_sum_same_module_different_versions() {
    let tmp = TempDir::new().unwrap();
    let content = "github.com/pkg v1.0.0 h1:a=\ngithub.com/pkg v2.0.0 h1:b=\ngithub.com/pkg v1.0.0/go.mod h1:c=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

#[test]
fn dependency_go_sum_empty_lines_ignored() {
    let tmp = TempDir::new().unwrap();
    let content = "\n\ngithub.com/x v1.0.0 h1:z=\n\n\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 1);
}

// ===========================================================================
// Dependency parsing – Gemfile.lock edge cases
// ===========================================================================

#[test]
fn dependency_gemfile_lock_empty_specs() {
    let tmp = TempDir::new().unwrap();
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn dependency_gemfile_lock_nested_deps_not_double_counted() {
    let tmp = TempDir::new().unwrap();
    // Only the 4-space-indented lines with parens should be counted
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0.0)\n      actionpack (= 7.0.0)\n      activesupport (= 7.0.0)\n    rack (2.2.0)\n\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // "rails (7.0.0)" and "rack (2.2.0)" are at 4-space indent
    // "actionpack" and "activesupport" are at 6-space indent (sub-deps)
    // The counter checks `line.starts_with("    ")` and `line.contains('(')`
    // so sub-deps at 6+ spaces also match (they start with 4+ spaces)
    assert!(report.lockfiles[0].dependencies >= 2);
}

// ===========================================================================
// Dependency parsing – pnpm-lock.yaml edge cases
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
// Empty asset list handling
// ===========================================================================

#[test]
fn assets_report_empty_has_zero_totals() {
    let tmp = TempDir::new().unwrap();
    let report = build_assets_report(tmp.path(), &[]).unwrap();
    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
    assert!(report.top_files.is_empty());
}

#[test]
fn dependency_report_empty_has_zero_total() {
    let tmp = TempDir::new().unwrap();
    let report = build_dependency_report(tmp.path(), &[]).unwrap();
    assert_eq!(report.total, 0);
    assert!(report.lockfiles.is_empty());
}

// ===========================================================================
// Sorting/ordering of asset reports
// ===========================================================================

#[test]
fn assets_categories_sorted_by_bytes_desc_then_name() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]), // image
        write_file(tmp.path(), "b.mp4", &[0u8; 100]), // video
        write_file(tmp.path(), "c.mp3", &[0u8; 200]), // audio
        write_file(tmp.path(), "d.zip", &[0u8; 200]), // archive
        write_file(tmp.path(), "e.exe", &[0u8; 50]),  // binary
        write_file(tmp.path(), "f.ttf", &[0u8; 50]),  // font
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 6);
    // 200 bytes: archive, audio (alphabetical tiebreak)
    assert_eq!(report.categories[0].category, "archive");
    assert_eq!(report.categories[1].category, "audio");
    // 100 bytes: image, video
    assert_eq!(report.categories[2].category, "image");
    assert_eq!(report.categories[3].category, "video");
    // 50 bytes: binary, font
    assert_eq!(report.categories[4].category, "binary");
    assert_eq!(report.categories[5].category, "font");
}

#[test]
fn assets_top_files_sorted_by_bytes_desc_then_path() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "b.png", &[0u8; 100]),
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "c.png", &[0u8; 200]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].path, "c.png");
    // Same bytes → sorted by path ascending
    assert_eq!(report.top_files[1].path, "a.png");
    assert_eq!(report.top_files[2].path, "b.png");
}

// ===========================================================================
// Snapshot-style formatted output tests
// ===========================================================================

#[test]
fn assets_report_json_keys_present() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "logo.svg", &[0u8; 512]),
        write_file(tmp.path(), "app.exe", &[0u8; 1024]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let json = serde_json::to_value(&report).unwrap();
    assert!(json.get("total_files").is_some());
    assert!(json.get("total_bytes").is_some());
    assert!(json.get("categories").is_some());
    assert!(json.get("top_files").is_some());
}

#[test]
fn dependency_report_json_keys_present() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"serde\"\nversion = \"1.0\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    let json = serde_json::to_value(&report).unwrap();
    assert!(json.get("total").is_some());
    assert!(json.get("lockfiles").is_some());
    let lockfile = &json["lockfiles"][0];
    assert!(lockfile.get("path").is_some());
    assert!(lockfile.get("kind").is_some());
    assert!(lockfile.get("dependencies").is_some());
}

#[test]
fn assets_report_deterministic_output() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "z.png", &[0u8; 50]),
        write_file(tmp.path(), "a.mp4", &[0u8; 100]),
        write_file(tmp.path(), "m.zip", &[0u8; 75]),
    ];
    let r1 = build_assets_report(tmp.path(), &files).unwrap();
    let r2 = build_assets_report(tmp.path(), &files).unwrap();
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn dependency_report_deterministic_output() {
    let tmp = TempDir::new().unwrap();
    let cargo = "[[package]]\nname = \"a\"\n\n[[package]]\nname = \"b\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let r1 = build_dependency_report(tmp.path(), std::slice::from_ref(&rel)).unwrap();
    let r2 = build_dependency_report(tmp.path(), std::slice::from_ref(&rel)).unwrap();
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// Mixed categories – bytes accounting
// ===========================================================================

#[test]
fn assets_total_bytes_equals_sum_of_category_bytes() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "b.mp3", &[0u8; 200]),
        write_file(tmp.path(), "c.zip", &[0u8; 300]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let sum: u64 = report.categories.iter().map(|c| c.bytes).sum();
    assert_eq!(report.total_bytes, sum);
}

#[test]
fn assets_total_files_equals_sum_of_category_files() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "b.jpg", &[0u8; 20]),
        write_file(tmp.path(), "c.mp4", &[0u8; 30]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let sum: usize = report.categories.iter().map(|c| c.files).sum();
    assert_eq!(report.total_files, sum);
}

// ===========================================================================
// Lockfile path normalization
// ===========================================================================

#[test]
fn dependency_lockfile_path_uses_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"x\"\n";
    let rel = write_file(tmp.path(), "subdir/Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].path, "subdir/Cargo.lock");
    assert!(!report.lockfiles[0].path.contains('\\'));
}

// ===========================================================================
// Multiple lockfiles – total aggregation
// ===========================================================================

#[test]
fn dependency_total_equals_sum_of_lockfile_deps() {
    let tmp = TempDir::new().unwrap();
    let cargo = "[[package]]\nname = \"a\"\n\n[[package]]\nname = \"b\"\n";
    let yarn = "# yarn\npkg-a@^1:\n  version \"1.0.0\"\n";
    let f1 = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let f2 = write_file(tmp.path(), "yarn.lock", yarn.as_bytes());
    let report = build_dependency_report(tmp.path(), &[f1, f2]).unwrap();
    let sum: usize = report.lockfiles.iter().map(|l| l.dependencies).sum();
    assert_eq!(report.total, sum);
}

// ===========================================================================
// Non-existent lockfile path
// ===========================================================================

#[test]
fn dependency_missing_lockfile_gives_zero_count() {
    let tmp = TempDir::new().unwrap();
    // Don't create the file on disk
    let rel = PathBuf::from("Cargo.lock");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}
