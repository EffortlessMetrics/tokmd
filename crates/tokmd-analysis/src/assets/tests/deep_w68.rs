//! W68 deep tests for `analysis assets module`.
//!
//! Exercises asset detection edge cases, lockfile parsing for all formats,
//! determinism, sorting invariants, path normalization, and structural
//! guarantees.

use std::path::{Path, PathBuf};

use crate::assets::{build_assets_report, build_dependency_report};
use tempfile::TempDir;

// ── Helper ──────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, content: &[u8]) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ═══════════════════════════════════════════════════════════════════
// § 1. Asset report determinism
// ═══════════════════════════════════════════════════════════════════

#[test]
fn asset_report_deterministic() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "b.mp4", &[0u8; 200]),
        write_file(tmp.path(), "c.zip", &[0u8; 50]),
    ];
    let r1 = build_assets_report(tmp.path(), &files).unwrap();
    let r2 = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(r1.total_files, r2.total_files);
    assert_eq!(r1.total_bytes, r2.total_bytes);
    assert_eq!(r1.categories.len(), r2.categories.len());
    for (a, b) in r1.categories.iter().zip(r2.categories.iter()) {
        assert_eq!(a.category, b.category);
        assert_eq!(a.bytes, b.bytes);
        assert_eq!(a.files, b.files);
    }
    assert_eq!(r1.top_files.len(), r2.top_files.len());
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Extension case normalization (uppercase → lowercase)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn uppercase_extension_categorized() {
    let tmp = TempDir::new().unwrap();
    let f = write_file(tmp.path(), "image.PNG", &[0u8; 32]);
    let r = build_assets_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.total_files, 1);
    assert_eq!(r.categories[0].category, "image");
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Path normalization – backslashes → forward slashes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn path_uses_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let f = write_file(tmp.path(), "sub/dir/logo.png", &[0u8; 64]);
    let r = build_assets_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.top_files[0].path, "sub/dir/logo.png");
    assert!(!r.top_files[0].path.contains('\\'));
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Category sorting tie-break by name
// ═══════════════════════════════════════════════════════════════════

#[test]
fn categories_with_same_bytes_sorted_by_name() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.zip", &[0u8; 100]),
        write_file(tmp.path(), "b.png", &[0u8; 100]),
    ];
    let r = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(r.categories.len(), 2);
    // Same bytes → alphabetical: archive < image
    assert_eq!(r.categories[0].category, "archive");
    assert_eq!(r.categories[1].category, "image");
}

// ═══════════════════════════════════════════════════════════════════
// § 5. Top files tie-break by path
// ═══════════════════════════════════════════════════════════════════

#[test]
fn top_files_same_bytes_sorted_by_path() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "z.png", &[0u8; 50]),
        write_file(tmp.path(), "a.png", &[0u8; 50]),
    ];
    let r = build_assets_report(tmp.path(), &files).unwrap();
    assert_eq!(r.top_files[0].path, "a.png");
    assert_eq!(r.top_files[1].path, "z.png");
}

// ═══════════════════════════════════════════════════════════════════
// § 6. Extensions list within a category is complete
// ═══════════════════════════════════════════════════════════════════

#[test]
fn category_extensions_collected() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "b.jpg", &[0u8; 10]),
        write_file(tmp.path(), "c.gif", &[0u8; 10]),
    ];
    let r = build_assets_report(tmp.path(), &files).unwrap();
    let exts = &r.categories[0].extensions;
    assert!(exts.contains(&"png".to_string()));
    assert!(exts.contains(&"jpg".to_string()));
    assert!(exts.contains(&"gif".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// § 7. Cargo.lock dependency counting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cargo_lock_counts_packages() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"a\"\nversion = \"1.0.0\"\n\n[[package]]\nname = \"b\"\nversion = \"2.0.0\"\n";
    let f = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.total, 2);
    assert_eq!(r.lockfiles.len(), 1);
    assert_eq!(r.lockfiles[0].kind, "cargo");
    assert_eq!(r.lockfiles[0].dependencies, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 8. package-lock.json packages field counting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn package_lock_json_counts_packages() {
    let tmp = TempDir::new().unwrap();
    let json = r#"{"packages": {"": {}, "node_modules/a": {}, "node_modules/b": {}}}"#;
    let f = write_file(tmp.path(), "package-lock.json", json.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.total, 2); // root "" excluded
    assert_eq!(r.lockfiles[0].kind, "npm");
}

// ═══════════════════════════════════════════════════════════════════
// § 9. package-lock.json fallback to dependencies field
// ═══════════════════════════════════════════════════════════════════

#[test]
fn package_lock_json_fallback_dependencies() {
    let tmp = TempDir::new().unwrap();
    let json = r#"{"dependencies": {"lodash": {}, "express": {}, "chalk": {}}}"#;
    let f = write_file(tmp.path(), "package-lock.json", json.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.total, 3);
}

// ═══════════════════════════════════════════════════════════════════
// § 10. package-lock.json malformed content → zero count
// ═══════════════════════════════════════════════════════════════════

#[test]
fn package_lock_json_malformed_returns_zero() {
    let tmp = TempDir::new().unwrap();
    let f = write_file(tmp.path(), "package-lock.json", b"not json");
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.lockfiles[0].dependencies, 0);
}

// ═══════════════════════════════════════════════════════════════════
// § 11. yarn.lock counting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn yarn_lock_counts_entries() {
    let tmp = TempDir::new().unwrap();
    let content = "# yarn lockfile\n\nlodash@^4.17.0:\n  version \"4.17.21\"\n\nexpress@^4.0.0:\n  version \"4.18.2\"\n";
    let f = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.lockfiles[0].kind, "yarn");
    assert_eq!(r.lockfiles[0].dependencies, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 12. go.sum counting with deduplication
// ═══════════════════════════════════════════════════════════════════

#[test]
fn go_sum_deduplicates_go_mod_entries() {
    let tmp = TempDir::new().unwrap();
    let content = "github.com/pkg/errors v0.9.1 h1:abc\ngithub.com/pkg/errors v0.9.1/go.mod h1:xyz\ngithub.com/stretchr/testify v1.8.0 h1:def\n";
    let f = write_file(tmp.path(), "go.sum", content.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.lockfiles[0].kind, "go");
    // /go.mod lines are skipped; two unique module@version pairs
    assert_eq!(r.lockfiles[0].dependencies, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 13. Gemfile.lock counting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gemfile_lock_counts_specs() {
    let tmp = TempDir::new().unwrap();
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rake (13.0.6)\n    rspec (3.12.0)\n\nPLATFORMS\n  ruby\n";
    let f = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.lockfiles[0].kind, "bundler");
    assert_eq!(r.lockfiles[0].dependencies, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 14. pnpm-lock.yaml counting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn pnpm_lock_counts_packages() {
    let tmp = TempDir::new().unwrap();
    let content = "lockfileVersion: 5.4\npackages:\n  /lodash/4.17.21:\n    resolution: {}\n  /express/4.18.2:\n    resolution: {}\n";
    let f = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.lockfiles[0].kind, "pnpm");
    assert_eq!(r.lockfiles[0].dependencies, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 15. Unknown lockfile names are skipped
// ═══════════════════════════════════════════════════════════════════

#[test]
fn unknown_lockfile_names_skipped() {
    let tmp = TempDir::new().unwrap();
    let f = write_file(tmp.path(), "random.lock", b"data");
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.total, 0);
    assert!(r.lockfiles.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// § 16. Empty lockfile content → zero dependencies
// ═══════════════════════════════════════════════════════════════════

#[test]
fn empty_cargo_lock_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let f = write_file(tmp.path(), "Cargo.lock", b"");
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.lockfiles[0].dependencies, 0);
}

// ═══════════════════════════════════════════════════════════════════
// § 17. Multiple lockfiles in one report
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multiple_lockfiles_aggregated() {
    let tmp = TempDir::new().unwrap();
    let cargo = "[[package]]\nname = \"a\"\nversion = \"1.0.0\"\n";
    let yarn = "# yarn lockfile\n\nlodash@^4.0.0:\n  version \"4.17.21\"\n\nexpress@^4.0.0:\n  version \"4.18.2\"\n";
    let f1 = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let f2 = write_file(tmp.path(), "yarn.lock", yarn.as_bytes());
    let r = build_dependency_report(tmp.path(), &[f1, f2]).unwrap();
    assert_eq!(r.lockfiles.len(), 2);
    assert_eq!(r.total, 3); // 1 cargo + 2 yarn
}

// ═══════════════════════════════════════════════════════════════════
// § 18. Dependency report determinism
// ═══════════════════════════════════════════════════════════════════

#[test]
fn dependency_report_deterministic() {
    let tmp = TempDir::new().unwrap();
    let cargo = "[[package]]\nname = \"x\"\nversion = \"1.0.0\"\n\n[[package]]\nname = \"y\"\nversion = \"2.0.0\"\n";
    let f = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
    let files = vec![f];
    let r1 = build_dependency_report(tmp.path(), &files).unwrap();
    let r2 = build_dependency_report(tmp.path(), &files).unwrap();
    assert_eq!(r1.total, r2.total);
    assert_eq!(r1.lockfiles.len(), r2.lockfiles.len());
    assert_eq!(r1.lockfiles[0].dependencies, r2.lockfiles[0].dependencies);
}

// ═══════════════════════════════════════════════════════════════════
// § 19. Nested directory paths in asset report
// ═══════════════════════════════════════════════════════════════════

#[test]
fn nested_directory_paths_preserved() {
    let tmp = TempDir::new().unwrap();
    let f = write_file(tmp.path(), "assets/images/deep/logo.svg", &[0u8; 128]);
    let r = build_assets_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.top_files[0].path, "assets/images/deep/logo.svg");
}

// ═══════════════════════════════════════════════════════════════════
// § 20. Lockfile path uses forward slashes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn lockfile_path_normalized() {
    let tmp = TempDir::new().unwrap();
    let f = write_file(
        tmp.path(),
        "sub/Cargo.lock",
        b"[[package]]\nname = \"a\"\nversion = \"1.0.0\"\n",
    );
    let r = build_dependency_report(tmp.path(), &[f]).unwrap();
    assert_eq!(r.lockfiles[0].path, "sub/Cargo.lock");
    assert!(!r.lockfiles[0].path.contains('\\'));
}
