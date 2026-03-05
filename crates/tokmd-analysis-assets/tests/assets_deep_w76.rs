//! W76 deep tests for `tokmd-analysis-assets`.
//!
//! Exercises asset categorisation edge cases, lockfile parsing boundary
//! conditions, dependency counting accuracy, top-file truncation,
//! multi-lockfile aggregation, and determinism invariants.

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

// ═══════════════════════════════════════════════════════════════════
// § 1. Asset classification coverage
// ═══════════════════════════════════════════════════════════════════

mod asset_categories_w76 {
    use super::*;

    #[test]
    fn all_image_extensions_classified() {
        let tmp = TempDir::new().unwrap();
        let exts = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "tiff", "ico"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|ext| write_file(tmp.path(), &format!("img.{ext}"), &[0u8; 8]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories.len(), 1);
        assert_eq!(r.categories[0].category, "image");
        assert_eq!(r.categories[0].extensions.len(), exts.len());
    }

    #[test]
    fn all_binary_extensions_classified() {
        let tmp = TempDir::new().unwrap();
        let exts = ["exe", "dll", "so", "dylib", "bin", "class", "jar"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|ext| write_file(tmp.path(), &format!("file.{ext}"), &[0u8; 16]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories[0].category, "binary");
    }

    #[test]
    fn font_extensions_classified() {
        let tmp = TempDir::new().unwrap();
        let exts = ["ttf", "otf", "woff", "woff2"];
        let files: Vec<PathBuf> = exts
            .iter()
            .map(|ext| write_file(tmp.path(), &format!("font.{ext}"), &[0u8; 32]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.total_files, exts.len());
        assert_eq!(r.categories[0].category, "font");
    }

    #[test]
    fn files_without_extension_skipped() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "Makefile", b"all: build\n");
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.total_files, 0);
        assert!(r.categories.is_empty());
    }

    #[test]
    fn unknown_extensions_skipped() {
        let tmp = TempDir::new().unwrap();
        let f = write_file(tmp.path(), "code.rs", b"fn main() {}\n");
        let r = build_assets_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.total_files, 0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Top-files truncation and sorting
// ═══════════════════════════════════════════════════════════════════

mod top_files_w76 {
    use super::*;

    #[test]
    fn top_files_truncated_to_ten() {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = (0..15)
            .map(|i| write_file(tmp.path(), &format!("img{i:02}.png"), &vec![0u8; (i + 1) * 10]))
            .collect();
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.top_files.len(), 10);
        // Largest files should be first
        assert!(r.top_files[0].bytes >= r.top_files[9].bytes);
    }

    #[test]
    fn top_files_sorted_by_bytes_descending() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "small.png", &[0u8; 10]),
            write_file(tmp.path(), "medium.png", &[0u8; 100]),
            write_file(tmp.path(), "large.png", &[0u8; 1000]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        assert_eq!(r.top_files[0].path, "large.png");
        assert_eq!(r.top_files[2].path, "small.png");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Lockfile dependency counting edge cases
// ═══════════════════════════════════════════════════════════════════

mod lockfile_w76 {
    use super::*;

    #[test]
    fn cargo_lock_single_package() {
        let tmp = TempDir::new().unwrap();
        let content = "[[package]]\nname = \"tokmd\"\nversion = \"0.1.0\"\n";
        let f = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.total, 1);
    }

    #[test]
    fn package_lock_json_empty_packages_object() {
        let tmp = TempDir::new().unwrap();
        let json = r#"{"packages": {"": {}}}"#;
        let f = write_file(tmp.path(), "package-lock.json", json.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        // Only root "" entry which gets subtracted
        assert_eq!(r.lockfiles[0].dependencies, 0);
    }

    #[test]
    fn go_sum_identical_module_different_versions_counted() {
        let tmp = TempDir::new().unwrap();
        let content = "github.com/pkg/errors v0.9.1 h1:abc\ngithub.com/pkg/errors v0.8.0 h1:def\ngithub.com/stretchr/testify v1.8.0 h1:ghi\n";
        let f = write_file(tmp.path(), "go.sum", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 3);
    }

    #[test]
    fn gemfile_lock_counts_indented_specs_with_parens() {
        let tmp = TempDir::new().unwrap();
        let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rake (13.0.6)\n    rspec (3.12.0)\n      rspec-core (~> 3.12.0)\n    rspec-core (3.12.1)\n\nPLATFORMS\n  ruby\n";
        let f = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        // All 4+ space indented lines with parens: rake, rspec, nested rspec-core, rspec-core = 4
        assert_eq!(r.lockfiles[0].dependencies, 4);
    }

    #[test]
    fn yarn_lock_comment_lines_excluded() {
        let tmp = TempDir::new().unwrap();
        let content = "# THIS IS AN AUTOGENERATED FILE.\n# yarn lockfile v1\n\nlodash@^4.17.0:\n  version \"4.17.21\"\n  resolved \"https://registry.yarnpkg.com/lodash\"\n";
        let f = write_file(tmp.path(), "yarn.lock", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 1);
    }

    #[test]
    fn pnpm_lock_lines_without_colon_excluded() {
        let tmp = TempDir::new().unwrap();
        let content = "lockfileVersion: 5.4\npackages:\n  /lodash/4.17.21:\n    resolution: {}\n  some-line-without-slash\n";
        let f = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f]).unwrap();
        assert_eq!(r.lockfiles[0].dependencies, 1);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Mixed reports and structural invariants
// ═══════════════════════════════════════════════════════════════════

mod structural_w76 {
    use super::*;

    #[test]
    fn total_bytes_equals_sum_of_category_bytes() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "a.png", &[0u8; 100]),
            write_file(tmp.path(), "b.mp3", &[0u8; 200]),
            write_file(tmp.path(), "c.zip", &[0u8; 300]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        let sum: u64 = r.categories.iter().map(|c| c.bytes).sum();
        assert_eq!(r.total_bytes, sum);
    }

    #[test]
    fn total_files_equals_sum_of_category_files() {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            write_file(tmp.path(), "a.png", &[0u8; 10]),
            write_file(tmp.path(), "b.jpg", &[0u8; 10]),
            write_file(tmp.path(), "c.mp4", &[0u8; 10]),
            write_file(tmp.path(), "d.wav", &[0u8; 10]),
        ];
        let r = build_assets_report(tmp.path(), &files).unwrap();
        let sum: usize = r.categories.iter().map(|c| c.files).sum();
        assert_eq!(r.total_files, sum);
    }

    #[test]
    fn dependency_total_equals_sum_of_lockfile_deps() {
        let tmp = TempDir::new().unwrap();
        let cargo = "[[package]]\nname = \"a\"\nversion = \"1\"\n\n[[package]]\nname = \"b\"\nversion = \"2\"\n";
        let yarn = "# yarn lockfile\n\nlodash@^4.0.0:\n  version \"4.17.21\"\n";
        let f1 = write_file(tmp.path(), "Cargo.lock", cargo.as_bytes());
        let f2 = write_file(tmp.path(), "yarn.lock", yarn.as_bytes());
        let r = build_dependency_report(tmp.path(), &[f1, f2]).unwrap();
        let sum: usize = r.lockfiles.iter().map(|l| l.dependencies).sum();
        assert_eq!(r.total, sum);
    }

    #[test]
    fn empty_file_list_produces_empty_reports() {
        let tmp = TempDir::new().unwrap();
        let assets = build_assets_report(tmp.path(), &[]).unwrap();
        let deps = build_dependency_report(tmp.path(), &[]).unwrap();
        assert_eq!(assets.total_files, 0);
        assert_eq!(assets.total_bytes, 0);
        assert!(assets.categories.is_empty());
        assert_eq!(deps.total, 0);
        assert!(deps.lockfiles.is_empty());
    }
}
