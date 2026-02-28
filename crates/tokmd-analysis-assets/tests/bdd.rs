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
// build_assets_report – empty / basic
// ===========================================================================

#[test]
fn given_no_files_when_assets_report_built_then_totals_are_zero() {
    let tmp = TempDir::new().unwrap();
    let report = build_assets_report(tmp.path(), &[]).unwrap();

    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
    assert!(report.top_files.is_empty());
}

#[test]
fn given_single_png_when_assets_report_built_then_image_category_detected() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "logo.png", &[0u8; 128]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.total_files, 1);
    assert_eq!(report.total_bytes, 128);
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "image");
    assert_eq!(report.categories[0].files, 1);
    assert_eq!(report.categories[0].bytes, 128);
    assert_eq!(report.categories[0].extensions, vec!["png"]);
}

// ===========================================================================
// build_assets_report – category classification
// ===========================================================================

#[test]
fn given_files_across_categories_when_report_built_then_all_categories_present() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "photo.jpg", &[0u8; 100]),
        write_file(tmp.path(), "movie.mp4", &[0u8; 200]),
        write_file(tmp.path(), "song.mp3", &[0u8; 50]),
        write_file(tmp.path(), "archive.zip", &[0u8; 300]),
        write_file(tmp.path(), "app.exe", &[0u8; 150]),
        write_file(tmp.path(), "font.woff2", &[0u8; 75]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 6);
    assert_eq!(report.total_bytes, 875);

    let cat_names: Vec<&str> = report
        .categories
        .iter()
        .map(|c| c.category.as_str())
        .collect();
    assert!(cat_names.contains(&"image"));
    assert!(cat_names.contains(&"video"));
    assert!(cat_names.contains(&"audio"));
    assert!(cat_names.contains(&"archive"));
    assert!(cat_names.contains(&"binary"));
    assert!(cat_names.contains(&"font"));
}

#[test]
fn given_multiple_image_extensions_when_report_built_then_extensions_merged() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 10]),
        write_file(tmp.path(), "b.jpg", &[0u8; 20]),
        write_file(tmp.path(), "c.svg", &[0u8; 30]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    let cat = &report.categories[0];
    assert_eq!(cat.category, "image");
    assert_eq!(cat.files, 3);
    assert_eq!(cat.bytes, 60);
    // extensions sorted alphabetically (BTreeSet)
    assert_eq!(cat.extensions, vec!["jpg", "png", "svg"]);
}

// ===========================================================================
// build_assets_report – unknown extensions skipped
// ===========================================================================

#[test]
fn given_files_with_unknown_extensions_when_report_built_then_skipped() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "readme.md", b"# Hello"),
        write_file(tmp.path(), "main.rs", b"fn main(){}"),
        write_file(tmp.path(), "style.css", b"body{}"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
}

#[test]
fn given_files_without_extension_when_report_built_then_skipped() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "Makefile", b"all:"),
        write_file(tmp.path(), "LICENSE", b"MIT"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
}

// ===========================================================================
// build_assets_report – sorting and top-N truncation
// ===========================================================================

#[test]
fn given_categories_when_report_built_then_sorted_by_bytes_descending() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "small.png", &[0u8; 10]), // image: 10
        write_file(tmp.path(), "big.mp4", &[0u8; 1000]), // video: 1000
        write_file(tmp.path(), "medium.zip", &[0u8; 500]), // archive: 500
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    let cats: Vec<&str> = report
        .categories
        .iter()
        .map(|c| c.category.as_str())
        .collect();
    assert_eq!(cats, vec!["video", "archive", "image"]);
}

#[test]
fn given_top_files_when_report_built_then_sorted_by_bytes_desc_and_truncated_to_ten() {
    let tmp = TempDir::new().unwrap();
    // Create 15 image files with different sizes
    let mut files = Vec::new();
    for i in 0..15 {
        let name = format!("img_{:02}.png", i);
        let size = (i + 1) * 100;
        files.push(write_file(tmp.path(), &name, &vec![0u8; size]));
    }
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.top_files.len(), 10);
    // First file should be the largest (img_14.png = 1500 bytes)
    assert_eq!(report.top_files[0].path, "img_14.png");
    assert_eq!(report.top_files[0].bytes, 1500);
    // Last of top 10 should be img_05.png = 600 bytes
    assert_eq!(report.top_files[9].path, "img_05.png");
    assert_eq!(report.top_files[9].bytes, 600);
}

#[test]
fn given_fewer_than_ten_files_when_report_built_then_all_in_top_files() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "b.jpg", &[0u8; 200]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.top_files.len(), 2);
    assert_eq!(report.top_files[0].path, "b.jpg");
    assert_eq!(report.top_files[1].path, "a.png");
}

// ===========================================================================
// build_assets_report – path normalization
// ===========================================================================

#[test]
fn given_path_with_backslashes_when_report_built_then_normalized_to_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "assets/images/logo.png", &[0u8; 64]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.top_files.len(), 1);
    assert!(
        !report.top_files[0].path.contains('\\'),
        "path should use forward slashes: {}",
        report.top_files[0].path
    );
    assert_eq!(report.top_files[0].path, "assets/images/logo.png");
}

// ===========================================================================
// build_assets_report – case insensitivity of extensions
// ===========================================================================

#[test]
fn given_uppercase_extension_when_report_built_then_lowercased() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "PHOTO.JPG", &[0u8; 50]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.total_files, 1);
    assert_eq!(report.categories[0].category, "image");
    assert_eq!(report.top_files[0].extension, "jpg");
}

// ===========================================================================
// build_assets_report – file row fields
// ===========================================================================

#[test]
fn given_asset_file_when_report_built_then_row_has_correct_fields() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "music/track.flac", &[0u8; 999]);
    let report = build_assets_report(tmp.path(), &[rel]).unwrap();

    let row = &report.top_files[0];
    assert_eq!(row.path, "music/track.flac");
    assert_eq!(row.bytes, 999);
    assert_eq!(row.category, "audio");
    assert_eq!(row.extension, "flac");
}

// ===========================================================================
// build_dependency_report – empty
// ===========================================================================

#[test]
fn given_no_lockfiles_when_dependency_report_built_then_empty() {
    let tmp = TempDir::new().unwrap();
    let report = build_dependency_report(tmp.path(), &[]).unwrap();

    assert_eq!(report.total, 0);
    assert!(report.lockfiles.is_empty());
}

// ===========================================================================
// build_dependency_report – Cargo.lock
// ===========================================================================

#[test]
fn given_cargo_lock_when_dependency_report_built_then_packages_counted() {
    let tmp = TempDir::new().unwrap();
    let content = r#"
[[package]]
name = "serde"
version = "1.0.0"

[[package]]
name = "anyhow"
version = "1.0.0"

[[package]]
name = "tokei"
version = "12.0.0"
"#;
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.total, 3);
    assert_eq!(report.lockfiles.len(), 1);
    assert_eq!(report.lockfiles[0].kind, "cargo");
    assert_eq!(report.lockfiles[0].dependencies, 3);
    assert_eq!(report.lockfiles[0].path, "Cargo.lock");
}

// ===========================================================================
// build_dependency_report – package-lock.json (v2 with packages)
// ===========================================================================

#[test]
fn given_npm_v2_lockfile_when_dependency_report_built_then_packages_counted_minus_root() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{
  "name": "my-app",
  "lockfileVersion": 2,
  "packages": {
    "": { "name": "my-app" },
    "node_modules/react": { "version": "18.0.0" },
    "node_modules/lodash": { "version": "4.17.0" }
  }
}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].kind, "npm");
    assert_eq!(report.lockfiles[0].dependencies, 2); // root "" excluded
}

#[test]
fn given_npm_v1_lockfile_when_dependency_report_built_then_dependencies_counted() {
    let tmp = TempDir::new().unwrap();
    let content = r#"{
  "name": "my-app",
  "lockfileVersion": 1,
  "dependencies": {
    "react": { "version": "17.0.0" },
    "react-dom": { "version": "17.0.0" }
  }
}"#;
    let rel = write_file(tmp.path(), "package-lock.json", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].kind, "npm");
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

#[test]
fn given_malformed_package_lock_when_dependency_report_built_then_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "package-lock.json", b"not valid json{{{");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].kind, "npm");
    assert_eq!(report.lockfiles[0].dependencies, 0);
}

// ===========================================================================
// build_dependency_report – yarn.lock
// ===========================================================================

#[test]
fn given_yarn_lock_when_dependency_report_built_then_entries_counted() {
    let tmp = TempDir::new().unwrap();
    let content = r#"# yarn lockfile v1

react@^18.0.0:
  version "18.0.0"
  resolved "https://registry.yarnpkg.com/react/-/react-18.0.0.tgz"

lodash@^4.17.0:
  version "4.17.21"
  resolved "https://registry.yarnpkg.com/lodash/-/lodash-4.17.21.tgz"
"#;
    let rel = write_file(tmp.path(), "yarn.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].kind, "yarn");
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// build_dependency_report – go.sum
// ===========================================================================

#[test]
fn given_go_sum_when_dependency_report_built_then_unique_modules_counted() {
    let tmp = TempDir::new().unwrap();
    // go.sum has two lines per module (source + go.mod), dedup expected
    let content = "\
github.com/pkg/errors v0.9.1 h1:abc=\n\
github.com/pkg/errors v0.9.1/go.mod h1:def=\n\
golang.org/x/sys v0.0.0-20210615035016 h1:ghi=\n\
golang.org/x/sys v0.0.0-20210615035016/go.mod h1:jkl=\n";
    let rel = write_file(tmp.path(), "go.sum", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].kind, "go");
    assert_eq!(report.lockfiles[0].dependencies, 2);
}

// ===========================================================================
// build_dependency_report – Gemfile.lock
// ===========================================================================

#[test]
fn given_gemfile_lock_when_dependency_report_built_then_specs_counted() {
    let tmp = TempDir::new().unwrap();
    let content = "GEM\n  remote: https://rubygems.org/\n  specs:\n    rails (7.0.0)\n    nokogiri (1.13.0)\n    rake (13.0.0)\n\nPLATFORMS\n  ruby\n";
    let rel = write_file(tmp.path(), "Gemfile.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].kind, "bundler");
    assert_eq!(report.lockfiles[0].dependencies, 3);
}

// ===========================================================================
// build_dependency_report – pnpm-lock.yaml
// ===========================================================================

#[test]
fn given_pnpm_lock_when_dependency_report_built_then_slash_lines_counted() {
    let tmp = TempDir::new().unwrap();
    let content = "\
lockfileVersion: 5\n\
packages:\n\
  /react/18.0.0:\n\
    resolution: {integrity: sha512-abc}\n\
  /lodash/4.17.21:\n\
    resolution: {integrity: sha512-def}\n\
  /chalk/5.0.0:\n\
    resolution: {integrity: sha512-ghi}\n";
    let rel = write_file(tmp.path(), "pnpm-lock.yaml", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].kind, "pnpm");
    assert_eq!(report.lockfiles[0].dependencies, 3);
}

// ===========================================================================
// build_dependency_report – multiple lockfiles
// ===========================================================================

#[test]
fn given_multiple_lockfiles_when_dependency_report_built_then_total_is_sum() {
    let tmp = TempDir::new().unwrap();
    let cargo = write_file(
        tmp.path(),
        "Cargo.lock",
        b"[[package]]\nname = \"a\"\n\n[[package]]\nname = \"b\"\n",
    );
    let yarn = write_file(
        tmp.path(),
        "yarn.lock",
        b"# yarn lockfile v1\n\nreact@^18:\n  version \"18\"\n",
    );
    let report = build_dependency_report(tmp.path(), &[cargo, yarn]).unwrap();

    assert_eq!(report.lockfiles.len(), 2);
    assert_eq!(report.total, 3); // 2 cargo + 1 yarn
}

// ===========================================================================
// build_dependency_report – unknown filenames skipped
// ===========================================================================

#[test]
fn given_non_lockfile_when_dependency_report_built_then_skipped() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "README.md", b"# Hello");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.total, 0);
    assert!(report.lockfiles.is_empty());
}

// ===========================================================================
// build_dependency_report – path normalization
// ===========================================================================

#[test]
fn given_nested_lockfile_when_report_built_then_path_uses_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(
        tmp.path(),
        "sub/project/Cargo.lock",
        b"[[package]]\nname = \"x\"\n",
    );
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].path, "sub/project/Cargo.lock");
}

// ===========================================================================
// build_assets_report – mixed known and unknown
// ===========================================================================

#[test]
fn given_mix_of_known_and_unknown_extensions_when_report_built_then_only_known_counted() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "logo.png", &[0u8; 100]),
        write_file(tmp.path(), "main.rs", b"fn main(){}"),
        write_file(tmp.path(), "video.mkv", &[0u8; 500]),
        write_file(tmp.path(), "data.csv", b"a,b,c"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 2); // png + mkv
    assert_eq!(report.total_bytes, 600);
}

// ===========================================================================
// build_assets_report – every recognized extension
// ===========================================================================

#[test]
fn given_every_image_extension_when_report_built_then_all_classified_as_image() {
    let tmp = TempDir::new().unwrap();
    let exts = [
        "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "tiff", "ico",
    ];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("file.{ext}"), &[0u8; 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "image");
    assert_eq!(report.categories[0].files, exts.len());
}

#[test]
fn given_every_video_extension_when_report_built_then_all_classified_as_video() {
    let tmp = TempDir::new().unwrap();
    let exts = ["mp4", "mov", "avi", "mkv", "webm", "mpeg", "mpg"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("file.{ext}"), &[0u8; 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "video");
    assert_eq!(report.categories[0].files, exts.len());
}

#[test]
fn given_every_audio_extension_when_report_built_then_all_classified_as_audio() {
    let tmp = TempDir::new().unwrap();
    let exts = ["mp3", "wav", "flac", "ogg", "aac", "m4a"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("file.{ext}"), &[0u8; 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "audio");
    assert_eq!(report.categories[0].files, exts.len());
}

#[test]
fn given_every_archive_extension_when_report_built_then_all_classified_as_archive() {
    let tmp = TempDir::new().unwrap();
    let exts = ["zip", "tar", "gz", "bz2", "xz", "7z", "rar"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("file.{ext}"), &[0u8; 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "archive");
    assert_eq!(report.categories[0].files, exts.len());
}

#[test]
fn given_every_binary_extension_when_report_built_then_all_classified_as_binary() {
    let tmp = TempDir::new().unwrap();
    let exts = ["exe", "dll", "so", "dylib", "bin", "class", "jar"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("file.{ext}"), &[0u8; 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "binary");
    assert_eq!(report.categories[0].files, exts.len());
}

#[test]
fn given_every_font_extension_when_report_built_then_all_classified_as_font() {
    let tmp = TempDir::new().unwrap();
    let exts = ["ttf", "otf", "woff", "woff2"];
    let files: Vec<PathBuf> = exts
        .iter()
        .map(|ext| write_file(tmp.path(), &format!("file.{ext}"), &[0u8; 10]))
        .collect();
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].category, "font");
    assert_eq!(report.categories[0].files, exts.len());
}

// ===========================================================================
// build_dependency_report – edge: empty lockfile content
// ===========================================================================

#[test]
fn given_empty_cargo_lock_when_dependency_report_built_then_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "Cargo.lock", b"");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn given_empty_go_sum_when_dependency_report_built_then_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "go.sum", b"");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn given_empty_gemfile_lock_when_dependency_report_built_then_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "Gemfile.lock", b"");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn given_empty_yarn_lock_when_dependency_report_built_then_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "yarn.lock", b"");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn given_empty_pnpm_lock_when_dependency_report_built_then_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "pnpm-lock.yaml", b"");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].dependencies, 0);
}

#[test]
fn given_empty_package_lock_when_dependency_report_built_then_zero_deps() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "package-lock.json", b"{}");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    assert_eq!(report.lockfiles[0].dependencies, 0);
}
