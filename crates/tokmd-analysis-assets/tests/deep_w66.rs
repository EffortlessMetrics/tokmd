//! W66 deep tests for `tokmd-analysis-assets`.
//!
//! Exercises asset detection, lockfile parsing, edge cases, sorting
//! guarantees, and structural invariants.

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tokmd_analysis_assets::{build_assets_report, build_dependency_report};

// ── Helper ──────────────────────────────────────────────────────

fn write_file(dir: &Path, rel: &str, content: &[u8]) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ── Asset detection ─────────────────────────────────────────────

mod asset_detection_w66 {
    use super::*;

    #[test]
    fn all_image_extensions_detected() {
        let tmp = TempDir::new().unwrap();
        let exts = [
            "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "tiff", "ico",
        ];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|e| write_file(tmp.path(), &format!("img.{e}"), &[0u8; 16]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories.len(), 1);
        assert_eq!(r.categories[0].category, "image");
        assert_eq!(r.categories[0].extensions.len(), exts.len());
    }

    #[test]
    fn all_video_extensions_detected() {
        let tmp = TempDir::new().unwrap();
        let exts = ["mp4", "mov", "avi", "mkv", "webm", "mpeg", "mpg"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|e| write_file(tmp.path(), &format!("vid.{e}"), &[0u8; 16]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories[0].category, "video");
    }

    #[test]
    fn all_audio_extensions_detected() {
        let tmp = TempDir::new().unwrap();
        let exts = ["mp3", "wav", "flac", "ogg", "aac", "m4a"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|e| write_file(tmp.path(), &format!("snd.{e}"), &[0u8; 16]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories[0].category, "audio");
    }

    #[test]
    fn all_archive_extensions_detected() {
        let tmp = TempDir::new().unwrap();
        let exts = ["zip", "tar", "gz", "bz2", "xz", "7z", "rar"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|e| write_file(tmp.path(), &format!("arc.{e}"), &[0u8; 16]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories[0].category, "archive");
    }

    #[test]
    fn all_binary_extensions_detected() {
        let tmp = TempDir::new().unwrap();
        let exts = ["exe", "dll", "so", "dylib", "bin", "class", "jar"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|e| write_file(tmp.path(), &format!("b.{e}"), &[0u8; 16]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories[0].category, "binary");
    }

    #[test]
    fn all_font_extensions_detected() {
        let tmp = TempDir::new().unwrap();
        let exts = ["ttf", "otf", "woff", "woff2"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|e| write_file(tmp.path(), &format!("f.{e}"), &[0u8; 16]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories[0].category, "font");
    }

    #[test]
    fn files_without_extension_skipped() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "Makefile", b"all:");
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.total_files, 0);
    }
}

// ── Lockfile patterns ───────────────────────────────────────────

mod lockfile_patterns_w66 {
    use super::*;

    #[test]
    fn npm_lock_with_only_root_entry() {
        let tmp = TempDir::new().unwrap();
        let content = r#"{"packages": {"": {"name": "root"}}}"#;
        let f = write_file(tmp.path(), "package-lock.json", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 0);
    }

    #[test]
    fn npm_lock_with_dependencies_key_fallback() {
        let tmp = TempDir::new().unwrap();
        let content = r#"{"dependencies": {"a": {"version": "1.0"}, "b": {"version": "2.0"}}}"#;
        let f = write_file(tmp.path(), "package-lock.json", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 2);
    }

    #[test]
    fn go_sum_filters_go_mod_entries() {
        let tmp = TempDir::new().unwrap();
        let content = "mod/a v1.0.0 h1:abc=\nmod/a v1.0.0/go.mod h1:def=\n";
        let f = write_file(tmp.path(), "go.sum", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 1);
    }

    #[test]
    fn yarn_lock_ignores_comments_and_indented_lines() {
        let tmp = TempDir::new().unwrap();
        let content = "# comment\n\npkg@^1.0:\n  version \"1.0\"\n  resolved \"...\"\n";
        let f = write_file(tmp.path(), "yarn.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 1);
    }

    #[test]
    fn unrecognized_lockfile_name_skipped() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "poetry.lock", b"[metadata]");
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.total, 0);
        assert!(r.lockfiles.is_empty());
    }

    #[test]
    fn gemfile_lock_counts_only_indented_specs() {
        let tmp = TempDir::new().unwrap();
        let content = "GEM\n  specs:\n    rails (7.0)\n      actionpack (7.0)\n    puma (5.6)\n\nPLATFORMS\n  ruby\n";
        let f = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].kind, "bundler");
        assert_eq!(r.lockfiles[0].dependencies, 3);
    }
}

// ── Edge cases ──────────────────────────────────────────────────

mod edge_cases_w66 {
    use super::*;

    #[test]
    fn empty_repo_produces_empty_reports() {
        let tmp = TempDir::new().unwrap();
        let asset_r = build_assets_report(tmp.path(), &[]).unwrap();
        let dep_r = build_dependency_report(tmp.path(), &[]).unwrap();
        assert_eq!(asset_r.total_files, 0);
        assert_eq!(dep_r.total, 0);
    }

    #[test]
    fn repo_with_only_assets_no_code() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "a.png", &[0u8; 100]),
            write_file(tmp.path(), "b.mp4", &[0u8; 200]),
            write_file(tmp.path(), "c.woff2", &[0u8; 50]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, 3);
        assert_eq!(r.total_bytes, 350);
    }

    #[test]
    fn categories_sorted_by_bytes_desc_then_name() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "a.mp4", &[0u8; 100]),
            write_file(tmp.path(), "b.png", &[0u8; 100]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.categories.len(), 2);
        assert_eq!(r.categories[0].category, "image");
        assert_eq!(r.categories[1].category, "video");
    }

    #[test]
    fn top_files_path_uses_forward_slashes() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "assets\\deep\\img.png", &[0u8; 64]);
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        assert!(!r.top_files[0].path.contains('\\'));
    }

    #[test]
    fn multiple_lockfiles_total_sums_correctly() {
        let tmp = TempDir::new().unwrap();
        let cargo =
            "[[package]]\nname=\"a\"\n\n[[package]]\nname=\"b\"\n\n[[package]]\nname=\"c\"\n";
        let go = "mod/x v1.0 h1:a=\nmod/y v2.0 h1:b=\n";
        let f1 = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
        let f2 = write_file(tmp.path(), "go.sum", go.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f1, f2]).unwrap();
        assert_eq!(r.total, 5);
        assert_eq!(r.lockfiles.len(), 2);
    }

    #[test]
    fn lockfile_path_normalized_to_forward_slashes() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "sub/Cargo.lock", b"[[package]]\nname=\"x\"\n");
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert!(
            r.lockfiles[0].path.contains('/'),
            "path should use forward slashes: {}",
            r.lockfiles[0].path
        );
        assert!(
            !r.lockfiles[0].path.contains('\\'),
            "path should not contain backslashes: {}",
            r.lockfiles[0].path
        );
    }
}

// ── Determinism ─────────────────────────────────────────────────

mod determinism_w66 {
    use super::*;

    #[test]
    fn asset_report_deterministic() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "z.png", &[0u8; 200]),
            write_file(tmp.path(), "a.jpg", &[0u8; 300]),
            write_file(tmp.path(), "m.mp4", &[0u8; 100]),
        ];
        let r1 = build_assets_report(tmp.path(), &files).unwrap();
        let r2 = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }

    #[test]
    fn dependency_report_deterministic() {
        let tmp = TempDir::new().unwrap();
        let content = "[[package]]\nname=\"a\"\n\n[[package]]\nname=\"b\"\n";
        let f = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
        let r1 = build_dependency_report(tmp.path(), std::slice::from_ref(&f)).unwrap();
        let r2 = build_dependency_report(tmp.path(), std::slice::from_ref(&f)).unwrap();
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap(),
        );
    }
}
