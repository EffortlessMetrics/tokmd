//! Deep tests (wave 38) for `analysis assets module`.
//!
//! Exercises build_assets_report and build_dependency_report with various
//! file types, lockfile formats, edge cases, and serialization.

use std::path::{Path, PathBuf};

use crate::assets::{build_assets_report, build_dependency_report};
use tempfile::TempDir;

// ── Helpers ─────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, content: &[u8]) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ── Asset discovery and classification ──────────────────────────

mod asset_classification_w38 {
    use super::*;

    #[test]
    fn single_image_classified() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "logo.png", &[0u8; 256]);
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.total_files, 1);
        assert_eq!(r.categories[0].category, "image");
    }

    #[test]
    fn mixed_categories_counted_separately() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "logo.png", &[0u8; 100]),
            write_file(tmp.path(), "icon.svg", &[0u8; 50]),
            write_file(tmp.path(), "clip.mp4", &[0u8; 500]),
            write_file(tmp.path(), "data.zip", &[0u8; 200]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, 4);
        assert_eq!(r.total_bytes, 850);
        assert_eq!(r.categories.len(), 3); // image, video, archive
    }

    #[test]
    fn case_insensitive_extension() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "logo.PNG", &[0u8; 64]);
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.total_files, 1);
        assert_eq!(r.categories[0].category, "image");
    }

    #[test]
    fn non_asset_extensions_skipped() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "main.rs", b"fn main() {}"),
            write_file(tmp.path(), "data.txt", b"hello"),
            write_file(tmp.path(), "style.css", b"body {}"),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, 0);
    }

    #[test]
    fn nested_paths_normalized() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "assets/images/bg.jpg", &[0u8; 128]);
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.top_files.len(), 1);
        assert!(r.top_files[0].path.contains('/'));
        assert!(!r.top_files[0].path.contains('\\'));
    }
}

// ── Dependency lockfile detection ───────────────────────────────

mod lockfile_detection_w38 {
    use super::*;

    #[test]
    fn cargo_lock_counts_packages() {
        let tmp = TempDir::new().unwrap();
        let content =
            "[[package]]\nname=\"a\"\n\n[[package]]\nname=\"b\"\n\n[[package]]\nname=\"c\"\n";
        let f = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].kind, "cargo");
        assert_eq!(r.lockfiles[0].dependencies, 3);
        assert_eq!(r.total, 3);
    }

    #[test]
    fn npm_packages_minus_root() {
        let tmp = TempDir::new().unwrap();
        let content = r#"{"packages": {"": {}, "node_modules/a": {}, "node_modules/b": {}, "node_modules/c": {}}}"#;
        let f = write_file(tmp.path(), "package-lock.json", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].kind, "npm");
        assert_eq!(r.lockfiles[0].dependencies, 3);
    }

    #[test]
    fn yarn_lock_counts_entries() {
        let tmp = TempDir::new().unwrap();
        let content = "# yarn lockfile v1\n\na@^1.0:\n  version \"1.0\"\n\nb@^2.0:\n  version \"2.0\"\n\nc@^3.0:\n  version \"3.0\"\n";
        let f = write_file(tmp.path(), "yarn.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].kind, "yarn");
        assert_eq!(r.lockfiles[0].dependencies, 3);
    }

    #[test]
    fn go_sum_deduplicates() {
        let tmp = TempDir::new().unwrap();
        let content = "mod/a v1.0.0 h1:abc=\nmod/a v1.0.0/go.mod h1:def=\nmod/b v2.0.0 h1:ghi=\nmod/c v1.0.0 h1:jkl=\n";
        let f = write_file(tmp.path(), "go.sum", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].kind, "go");
        assert_eq!(r.lockfiles[0].dependencies, 3);
    }

    #[test]
    fn gemfile_lock_counts_specs() {
        let tmp = TempDir::new().unwrap();
        let content =
            "GEM\n  specs:\n    rails (7.0)\n    rack (2.2)\n    puma (5.6)\n\nPLATFORMS\n  ruby\n";
        let f = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].kind, "bundler");
        assert_eq!(r.lockfiles[0].dependencies, 3);
    }

    #[test]
    fn pnpm_lock_counts_packages() {
        let tmp = TempDir::new().unwrap();
        let content = "lockfileVersion: 5\n\npackages:\n  /a/1.0.0:\n    res: {}\n  /b/2.0.0:\n    res: {}\n  /c/3.0.0:\n    res: {}\n";
        let f = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].kind, "pnpm");
        assert_eq!(r.lockfiles[0].dependencies, 3);
    }
}

// ── Asset report structure ──────────────────────────────────────

mod report_structure_w38 {
    use super::*;

    #[test]
    fn categories_sorted_by_bytes_desc() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "small.png", &[0u8; 10]),
            write_file(tmp.path(), "big.mp4", &[0u8; 1000]),
            write_file(tmp.path(), "med.zip", &[0u8; 500]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.categories[0].category, "video");
        assert_eq!(r.categories[1].category, "archive");
        assert_eq!(r.categories[2].category, "image");
    }

    #[test]
    fn top_files_sorted_by_bytes_desc() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "a.png", &[0u8; 50]),
            write_file(tmp.path(), "b.png", &[0u8; 200]),
            write_file(tmp.path(), "c.png", &[0u8; 100]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.top_files[0].path, "b.png");
        assert_eq!(r.top_files[1].path, "c.png");
        assert_eq!(r.top_files[2].path, "a.png");
    }

    #[test]
    fn top_files_capped_at_ten() {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = (0..20)
            .map(|i| write_file(tmp.path(), &format!("img{i}.png"), &vec![0u8; (i + 1) * 10]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.top_files.len(), 10);
    }

    #[test]
    fn extensions_list_complete() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "a.png", &[0u8; 10]),
            write_file(tmp.path(), "b.jpg", &[0u8; 20]),
            write_file(tmp.path(), "c.svg", &[0u8; 30]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        let exts = &r.categories[0].extensions;
        assert!(exts.contains(&"png".to_string()));
        assert!(exts.contains(&"jpg".to_string()));
        assert!(exts.contains(&"svg".to_string()));
    }
}

// ── Edge cases ──────────────────────────────────────────────────

mod edge_cases_w38 {
    use super::*;

    #[test]
    fn empty_assets_report() {
        let tmp = TempDir::new().unwrap();
        let r = build_assets_report(tmp.path(), &[]).unwrap();
        assert_eq!(r.total_files, 0);
        assert_eq!(r.total_bytes, 0);
        assert!(r.categories.is_empty());
        assert!(r.top_files.is_empty());
    }

    #[test]
    fn empty_dependency_report() {
        let tmp = TempDir::new().unwrap();
        let r = build_dependency_report(tmp.path(), &[]).unwrap();
        assert_eq!(r.total, 0);
        assert!(r.lockfiles.is_empty());
    }

    #[test]
    fn many_assets_all_same_category() {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = (0..50)
            .map(|i| write_file(tmp.path(), &format!("img{i}.png"), &[0u8; 32]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, 50);
        assert_eq!(r.categories.len(), 1);
        assert_eq!(r.categories[0].files, 50);
    }

    #[test]
    fn cargo_lock_empty_content() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "Cargo.lock", b"# empty\n");
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 0);
    }

    #[test]
    fn npm_lock_invalid_json() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "package-lock.json", b"not json");
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 0);
    }

    #[test]
    fn multiple_lockfile_total_aggregation() {
        let tmp = TempDir::new().unwrap();
        let cargo = "[[package]]\nname=\"a\"\n\n[[package]]\nname=\"b\"\n";
        let npm = r#"{"packages": {"": {}, "node_modules/x": {}, "node_modules/y": {}}}"#;
        let f1 = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
        let f2 = write_file(tmp.path(), "package-lock.json", npm.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f1, f2]).unwrap();
        assert_eq!(r.total, 4); // 2 cargo + 2 npm
    }

    #[test]
    fn dependency_report_serialization() {
        let tmp = TempDir::new().unwrap();
        let content = "[[package]]\nname=\"serde\"\n";
        let f = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let parsed: tokmd_analysis_types::DependencyReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total, r.total);
    }

    #[test]
    fn asset_report_serialization() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "logo.png", &[0u8; 128]);
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let parsed: tokmd_analysis_types::AssetReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total_files, 1);
        assert_eq!(parsed.total_bytes, r.total_bytes);
    }
}
