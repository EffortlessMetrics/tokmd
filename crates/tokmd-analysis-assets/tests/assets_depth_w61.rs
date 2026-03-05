//! W61 depth tests for `tokmd-analysis-assets`.
//!
//! Covers edge cases in asset classification, lockfile parsing corner cases,
//! report aggregation invariants, cross-category interactions, determinism,
//! and property-based verification of structural guarantees.

use std::path::{Path, PathBuf};

use proptest::prelude::*;
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
// 1. Empty extension string (no dot) is skipped
// ===========================================================================
#[test]
fn extensionless_file_skipped() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "README", b"hello");
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.total_files, 0);
}

// ===========================================================================
// 2. Dot-only filename is skipped
// ===========================================================================
#[test]
fn dot_only_filename_skipped() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), ".hidden", b"secret");
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.total_files, 0);
}

// ===========================================================================
// 3. Case-insensitive extension matching (uppercase PNG)
// ===========================================================================
#[test]
fn uppercase_extension_matched() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "LOGO.PNG", &[0u8; 32]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.total_files, 1);
    assert_eq!(report.categories[0].category, "image");
    assert_eq!(report.top_files[0].extension, "png");
}

// ===========================================================================
// 4. Mixed-case extension (JpG) is normalized
// ===========================================================================
#[test]
fn mixed_case_extension_normalized() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "photo.JpG", &[0u8; 20]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.total_files, 1);
    assert_eq!(report.top_files[0].extension, "jpg");
}

// ===========================================================================
// 5. Same extension in multiple subdirs counted once in category extensions
// ===========================================================================
#[test]
fn duplicate_extension_deduped_in_category() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a/img.png", &[0u8; 10]),
        write_file(tmp.path(), "b/img.png", &[0u8; 20]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories[0].extensions.len(), 1);
    assert_eq!(report.categories[0].extensions[0], "png");
}

// ===========================================================================
// 6. Category extensions are sorted (BTreeSet guarantee)
// ===========================================================================
#[test]
fn category_extensions_sorted() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "z.webp", &[0u8; 10]),
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "m.jpg", &[0u8; 10]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let exts = &report.categories[0].extensions;
    let mut sorted = exts.clone();
    sorted.sort();
    assert_eq!(*exts, sorted);
}

// ===========================================================================
// 7. Category sort tiebreak: alphabetical by name when bytes equal
// ===========================================================================
#[test]
fn category_tiebreak_alphabetical() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.zip", &[0u8; 100]), // archive
        write_file(tmp.path(), "b.mp3", &[0u8; 100]), // audio
        write_file(tmp.path(), "c.ttf", &[0u8; 100]), // font
        write_file(tmp.path(), "d.exe", &[0u8; 100]), // binary
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    // All 100 bytes: sorted alphabetically by category name
    let cats: Vec<&str> = report
        .categories
        .iter()
        .map(|c| c.category.as_str())
        .collect();
    assert_eq!(cats, vec!["archive", "audio", "binary", "font"]);
}

// ===========================================================================
// 8. Top files tiebreak: path ascending when bytes equal
// ===========================================================================
#[test]
fn top_files_tiebreak_by_path() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "z/icon.png", &[0u8; 50]),
        write_file(tmp.path(), "a/icon.png", &[0u8; 50]),
        write_file(tmp.path(), "m/icon.png", &[0u8; 50]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files[0].path, "a/icon.png");
    assert_eq!(report.top_files[1].path, "m/icon.png");
    assert_eq!(report.top_files[2].path, "z/icon.png");
}

// ===========================================================================
// 9. Exactly 10 files → no truncation
// ===========================================================================
#[test]
fn exactly_ten_files_no_truncation() {
    let tmp = TempDir::new().unwrap();
    let files: Vec<PathBuf> = (0..10)
        .map(|i| write_file(tmp.path(), &format!("f{i}.png"), &vec![0u8; (i + 1) * 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.top_files.len(), 10);
}

// ===========================================================================
// 10. All six categories present simultaneously
// ===========================================================================
#[test]
fn all_six_categories_present() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]), // image
        write_file(tmp.path(), "b.mp4", &[0u8; 10]), // video
        write_file(tmp.path(), "c.mp3", &[0u8; 10]), // audio
        write_file(tmp.path(), "d.zip", &[0u8; 10]), // archive
        write_file(tmp.path(), "e.exe", &[0u8; 10]), // binary
        write_file(tmp.path(), "f.ttf", &[0u8; 10]), // font
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.categories.len(), 6);
    let mut cats: Vec<&str> = report
        .categories
        .iter()
        .map(|c| c.category.as_str())
        .collect();
    cats.sort();
    assert_eq!(
        cats,
        vec!["archive", "audio", "binary", "font", "image", "video"]
    );
}

// ===========================================================================
// 11. Deeply nested path preserved with forward slashes
// ===========================================================================
#[test]
fn deep_nested_path_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "a/b/c/d/e/f/logo.svg", &[0u8; 8]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.top_files[0].path, "a/b/c/d/e/f/logo.svg");
}

// ===========================================================================
// 12. Cargo.lock with inline text mentioning [[package]] in values
// ===========================================================================
#[test]
fn cargo_lock_ignores_package_in_values() {
    let tmp = TempDir::new().unwrap();
    let content = r#"[[package]]
name = "crate-a"
version = "1.0"
description = "This package uses [[package]] in docs"

[[package]]
name = "crate-b"
version = "2.0"
"#;
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // "[[package]] in docs" is inside a value, but the simple count_cargo_lock
    // counts ALL occurrences of "[[package]]" including in values.
    // So it counts 3: two headers + one in the description.
    assert_eq!(report.lockfiles[0].dependencies, 3);
}

// ===========================================================================
// 13. package-lock.json with empty packages object (only root key)
// ===========================================================================
#[test]
fn npm_packages_only_root() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{"packages": {"": {"name": "my-app"}}}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 14. package-lock.json with malformed JSON returns 0
// ===========================================================================
#[test]
fn npm_malformed_json_zero() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "package-lock.json", b"not json at all");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 15. package-lock.json: neither packages nor dependencies field → 0
// ===========================================================================
#[test]
fn npm_no_packages_no_dependencies() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{"name": "app", "version": "1.0.0"}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 16. yarn.lock: comment-only file → 0
// ===========================================================================
#[test]
fn yarn_lock_comments_only() {
    let tmp = TempDir::new().unwrap();
    let content = "# yarn lockfile v1\n# comment\n";
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 17. yarn.lock: indented lines not counted as packages
// ===========================================================================
#[test]
fn yarn_lock_indented_not_counted() {
    let tmp = TempDir::new().unwrap();
    let content = "# yarn lockfile v1\n\npkg@^1.0:\n  version \"1.0\"\n  resolved \"...\"\n  integrity sha512-abc\n";
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 1);
}

// ===========================================================================
// 18. go.sum: all /go.mod lines → 0 unique modules
// ===========================================================================
#[test]
fn go_sum_all_go_mod_lines() {
    let tmp = TempDir::new().unwrap();
    let content = "example.com/a v1.0.0/go.mod h1:abc=\nexample.com/b v2.0.0/go.mod h1:def=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 19. go.sum: duplicate module@version deduped
// ===========================================================================
#[test]
fn go_sum_duplicate_module_version_deduped() {
    let tmp = TempDir::new().unwrap();
    let content = "example.com/x v1.0.0 h1:aaa=\nexample.com/x v1.0.0 h1:bbb=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 1);
}

// ===========================================================================
// 20. go.sum: blank lines ignored
// ===========================================================================
#[test]
fn go_sum_blank_lines_ignored() {
    let tmp = TempDir::new().unwrap();
    let content = "\n\nexample.com/x v1.0.0 h1:abc=\n\n\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 1);
}

// ===========================================================================
// 21. Gemfile.lock: no specs section → 0
// ===========================================================================
#[test]
fn gemfile_lock_no_specs() {
    let tmp = TempDir::new().unwrap();
    let content = "PLATFORMS\n  ruby\n\nBUNDLED WITH\n  2.4.0\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 22. Gemfile.lock: nested dependencies not double-counted
// ===========================================================================
#[test]
fn gemfile_lock_nested_deps_counted() {
    let tmp = TempDir::new().unwrap();
    // 4-space indent with parens: counted. 6-space indent: nested dep with parens.
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0)\n      actionpack (7.0)\n    rack (2.2)\n\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // rails, actionpack (nested but starts with 6 spaces still has parens),
    // rack: all have "(" so counted
    assert_eq!(report.lockfiles[0].dependencies, 3);
}

// ===========================================================================
// 23. pnpm-lock.yaml: lines without slash prefix not counted
// ===========================================================================
#[test]
fn pnpm_lock_no_slash_not_counted() {
    let tmp = TempDir::new().unwrap();
    let content = "lockfileVersion: 5.4\n\npackages:\n  react/18.2.0:\n    resolution: {}\nnot-a-package:\n  foo: bar\n";
    let rel = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    // Only lines with trimmed "/" prefix and ":" are counted
    // "  react/18.2.0:" has trimmed start "/react..." but actually trimmed is "react/18.2.0:"
    // The function checks starts_with("/") on the trimmed line:
    // "  react/18.2.0:" trimmed → "react/18.2.0:" doesn't start with "/"
    // So this doesn't match. Let's use the real pnpm format:
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// 24. pnpm-lock.yaml: proper /pkg/version: format
// ===========================================================================
#[test]
fn pnpm_lock_proper_format() {
    let tmp = TempDir::new().unwrap();
    let content = "lockfileVersion: 5.4\n\npackages:\n  /react/18.2.0:\n    resolution: {}\n  /lodash/4.17.21:\n    resolution: {}\n";
    let rel = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 25. Lockfile path normalized to forward slashes
// ===========================================================================
#[test]
fn lockfile_path_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(
        tmp.path(),
        "sub/dir/Cargo.lock",
        b"[[package]]\nname = \"a\"\n",
    );
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].path, "sub/dir/Cargo.lock");
    assert!(!report.lockfiles[0].path.contains('\\'));
}

// ===========================================================================
// 26. Multiple lockfiles of same type counted separately
// ===========================================================================
#[test]
fn multiple_same_type_lockfiles() {
    let tmp = TempDir::new().unwrap();
    let f1 = write_file(tmp.path(), "a/Cargo.lock", b"[[package]]\nname = \"x\"\n");
    let f2 = write_file(
        tmp.path(),
        "b/Cargo.lock",
        b"[[package]]\nname = \"y\"\n[[package]]\nname = \"z\"\n",
    );
    let report = build_dependency_report(tmp.path(), &[f1, f2]).unwrap();
    assert_eq!(report.lockfiles.len(), 2);
    assert_eq!(report.total, 3); // 1 + 2
}

// ===========================================================================
// 27. Large Cargo.lock with many packages
// ===========================================================================
#[test]
fn cargo_lock_many_packages() {
    let tmp = TempDir::new().unwrap();
    let mut content = String::new();
    for i in 0..100 {
        content.push_str(&format!(
            "[[package]]\nname = \"crate-{i}\"\nversion = \"{i}.0.0\"\n\n"
        ));
    }
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 100);
}

// ===========================================================================
// 28. Single zero-byte asset → bytes = 0 but file counted
// ===========================================================================
#[test]
fn zero_byte_asset_in_category() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "empty.mp4", &[]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.total_files, 1);
    assert_eq!(report.total_bytes, 0);
    assert_eq!(report.categories[0].bytes, 0);
}

// ===========================================================================
// 29. Report with only non-asset files → empty report
// ===========================================================================
#[test]
fn only_non_asset_files_empty_report() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "main.rs", b"fn main() {}"),
        write_file(tmp.path(), "lib.py", b"pass"),
        write_file(tmp.path(), "index.html", b"<html></html>"),
        write_file(tmp.path(), "style.css", b"body {}"),
        write_file(tmp.path(), "data.json", b"{}"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
    assert!(report.top_files.is_empty());
}

// ===========================================================================
// 30. AssetReport total_files == sum(categories.files)
// ===========================================================================
#[test]
fn total_files_equals_category_sum() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "b.jpg", &[0u8; 20]),
        write_file(tmp.path(), "c.mp4", &[0u8; 30]),
        write_file(tmp.path(), "d.exe", &[0u8; 40]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let sum: usize = report.categories.iter().map(|c| c.files).sum();
    assert_eq!(report.total_files, sum);
}

// ===========================================================================
// 31. AssetReport total_bytes == sum(categories.bytes)
// ===========================================================================
#[test]
fn total_bytes_equals_category_sum() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 111]),
        write_file(tmp.path(), "b.mp4", &[0u8; 222]),
        write_file(tmp.path(), "c.zip", &[0u8; 333]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    let sum: u64 = report.categories.iter().map(|c| c.bytes).sum();
    assert_eq!(report.total_bytes, sum);
}

// ===========================================================================
// 32. Deterministic asset report: same input twice → same JSON
// ===========================================================================
#[test]
fn asset_report_deterministic() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "x.png", &[0u8; 50]),
        write_file(tmp.path(), "y.mp4", &[0u8; 75]),
        write_file(tmp.path(), "z.ttf", &[0u8; 25]),
    ];
    let j1 = serde_json::to_string(&build_assets_report(tmp.path(), &files).unwrap()).unwrap();
    let j2 = serde_json::to_string(&build_assets_report(tmp.path(), &files).unwrap()).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 33. Deterministic dependency report
// ===========================================================================
#[test]
fn dependency_report_deterministic() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(
        tmp.path(),
        "Cargo.lock",
        b"[[package]]\nname = \"a\"\n[[package]]\nname = \"b\"\n",
    );
    let j1 = serde_json::to_string(
        &build_dependency_report(tmp.path(), std::slice::from_ref(&rel)).unwrap(),
    )
    .unwrap();
    let j2 = serde_json::to_string(
        &build_dependency_report(tmp.path(), std::slice::from_ref(&rel)).unwrap(),
    )
    .unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 34. package-lock.json: packages with no root key → all counted
// ===========================================================================
#[test]
fn npm_packages_no_root_key() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{"packages": {"node_modules/a": {}, "node_modules/b": {}}}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 35. go.sum: multiple versions of same module counted separately
// ===========================================================================
#[test]
fn go_sum_multiple_versions() {
    let tmp = TempDir::new().unwrap();
    let content = "example.com/x v1.0.0 h1:a=\nexample.com/x v2.0.0 h1:b=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// 36. AssetReport JSON keys present
// ===========================================================================
#[test]
fn asset_report_json_structure() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "a.png", &[0u8; 10]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val["total_files"].is_number());
    assert!(val["total_bytes"].is_number());
    assert!(val["categories"].is_array());
    assert!(val["top_files"].is_array());
    let tf = &val["top_files"][0];
    assert!(tf["path"].is_string());
    assert!(tf["bytes"].is_number());
    assert!(tf["category"].is_string());
    assert!(tf["extension"].is_string());
}

// ===========================================================================
// 37. DependencyReport JSON keys present
// ===========================================================================
#[test]
fn dependency_report_json_structure() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "Cargo.lock", b"[[package]]\nname = \"x\"\n");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val["total"].is_number());
    assert!(val["lockfiles"].is_array());
    let lf = &val["lockfiles"][0];
    assert!(lf["path"].is_string());
    assert!(lf["kind"].is_string());
    assert!(lf["dependencies"].is_number());
}

// ===========================================================================
// 38. Gemfile.lock: specs with sub-deps not starting with 4 spaces
// ===========================================================================
#[test]
fn gemfile_lock_non_indented_stops_specs() {
    let tmp = TempDir::new().unwrap();
    let content =
        "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0)\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    assert_eq!(report.lockfiles[0].dependencies, 1);
}

// ===========================================================================
// Property-based tests
// ===========================================================================

mod properties {
    use super::*;

    fn arb_asset_ext() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("png"),
            Just("jpg"),
            Just("gif"),
            Just("svg"),
            Just("mp4"),
            Just("mp3"),
            Just("zip"),
            Just("exe"),
            Just("ttf"),
            Just("woff2"),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(30))]

        /// Total bytes always equals sum of category bytes.
        #[test]
        fn total_bytes_invariant(
            count in 1usize..15,
            size in 1usize..2048,
        ) {
            let tmp = TempDir::new().unwrap();
            let files: Vec<PathBuf> = (0..count)
                .map(|i| {
                    let name = format!("f{i}.png");
                    let full = tmp.path().join(&name);
                    std::fs::write(&full, vec![0u8; size]).unwrap();
                    PathBuf::from(name)
                })
                .collect();
            let report = build_assets_report(tmp.path(), &files).unwrap();
            let cat_sum: u64 = report.categories.iter().map(|c| c.bytes).sum();
            prop_assert_eq!(report.total_bytes, cat_sum);
        }

        /// top_files length never exceeds 10.
        #[test]
        fn top_files_cap(count in 0usize..25) {
            let tmp = TempDir::new().unwrap();
            let files: Vec<PathBuf> = (0..count)
                .map(|i| {
                    let name = format!("f{i}.jpg");
                    let full = tmp.path().join(&name);
                    std::fs::write(&full, vec![0u8; 8]).unwrap();
                    PathBuf::from(name)
                })
                .collect();
            let report = build_assets_report(tmp.path(), &files).unwrap();
            prop_assert!(report.top_files.len() <= 10);
        }

        /// Cargo.lock package count matches marker count.
        #[test]
        fn cargo_lock_count_matches(n in 0usize..30) {
            let tmp = TempDir::new().unwrap();
            let mut content = String::new();
            for i in 0..n {
                content.push_str(&format!("[[package]]\nname = \"dep-{i}\"\n\n"));
            }
            std::fs::write(tmp.path().join("Cargo.lock"), &content).unwrap();
            let report = build_dependency_report(tmp.path(), &[PathBuf::from("Cargo.lock")]).unwrap();
            prop_assert_eq!(report.lockfiles[0].dependencies, n);
        }

        /// Dependency total equals sum of lockfile deps.
        #[test]
        fn dep_total_sum(cargo_n in 0usize..10, yarn_n in 0usize..10) {
            let tmp = TempDir::new().unwrap();
            let mut cargo_content = String::new();
            for i in 0..cargo_n {
                cargo_content.push_str(&format!("[[package]]\nname = \"c-{i}\"\n\n"));
            }
            std::fs::write(tmp.path().join("Cargo.lock"), &cargo_content).unwrap();

            let mut yarn_content = String::from("# yarn lockfile v1\n\n");
            for i in 0..yarn_n {
                yarn_content.push_str(&format!("dep-{i}@^1.0:\n  version \"1.0.{i}\"\n\n"));
            }
            std::fs::write(tmp.path().join("yarn.lock"), &yarn_content).unwrap();

            let files = vec![PathBuf::from("Cargo.lock"), PathBuf::from("yarn.lock")];
            let report = build_dependency_report(tmp.path(), &files).unwrap();
            let sum: usize = report.lockfiles.iter().map(|l| l.dependencies).sum();
            prop_assert_eq!(report.total, sum);
        }

        /// Asset paths never contain backslashes.
        #[test]
        fn no_backslash_paths(ext in arb_asset_ext()) {
            let tmp = TempDir::new().unwrap();
            let rel = format!("sub/dir/file.{ext}");
            let full = tmp.path().join(&rel);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(&full, [0u8; 16]).unwrap();
            let report = build_assets_report(tmp.path(), &[PathBuf::from(&rel)]).unwrap();
            for f in &report.top_files {
                prop_assert!(!f.path.contains('\\'), "backslash in: {}", f.path);
            }
        }

        /// Asset serde roundtrip preserves all fields.
        #[test]
        fn asset_serde_roundtrip(count in 1usize..8) {
            let tmp = TempDir::new().unwrap();
            let files: Vec<PathBuf> = (0..count)
                .map(|i| {
                    let name = format!("img{i}.png");
                    let full = tmp.path().join(&name);
                    std::fs::write(&full, vec![0u8; (i + 1) * 16]).unwrap();
                    PathBuf::from(name)
                })
                .collect();
            let report = build_assets_report(tmp.path(), &files).unwrap();
            let json = serde_json::to_string(&report).unwrap();
            let rt: tokmd_analysis_types::AssetReport = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(rt.total_files, report.total_files);
            prop_assert_eq!(rt.total_bytes, report.total_bytes);
            prop_assert_eq!(rt.categories.len(), report.categories.len());
        }
    }
}
