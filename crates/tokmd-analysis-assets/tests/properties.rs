use std::path::PathBuf;

use proptest::prelude::*;
use tempfile::TempDir;
use tokmd_analysis_assets::{build_assets_report, build_dependency_report};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_asset_ext() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("png".to_string()),
        Just("jpg".to_string()),
        Just("gif".to_string()),
        Just("svg".to_string()),
        Just("mp4".to_string()),
        Just("mp3".to_string()),
        Just("zip".to_string()),
        Just("exe".to_string()),
        Just("woff2".to_string()),
        Just("ttf".to_string()),
    ]
}

fn arb_non_asset_ext() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rs".to_string()),
        Just("py".to_string()),
        Just("js".to_string()),
        Just("toml".to_string()),
        Just("md".to_string()),
        Just("txt".to_string()),
        Just("html".to_string()),
        Just("css".to_string()),
    ]
}

fn arb_file_size() -> impl Strategy<Value = usize> {
    1usize..4096
}

// ---------------------------------------------------------------------------
// Asset report properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn total_bytes_equals_sum_of_category_bytes(
        sizes in prop::collection::vec(arb_file_size(), 1..20),
    ) {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = sizes
            .iter()
            .enumerate()
            .map(|(i, &sz)| {
                let name = format!("file_{i}.png");
                let full = tmp.path().join(&name);
                std::fs::write(&full, vec![0u8; sz]).unwrap();
                PathBuf::from(name)
            })
            .collect();

        let report = build_assets_report(tmp.path(), &files).unwrap();
        let cat_sum: u64 = report.categories.iter().map(|c| c.bytes).sum();
        prop_assert_eq!(report.total_bytes, cat_sum);
    }

    #[test]
    fn total_files_equals_sum_of_category_files(
        sizes in prop::collection::vec(arb_file_size(), 1..20),
    ) {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = sizes
            .iter()
            .enumerate()
            .map(|(i, &sz)| {
                let name = format!("file_{i}.jpg");
                let full = tmp.path().join(&name);
                std::fs::write(&full, vec![0u8; sz]).unwrap();
                PathBuf::from(name)
            })
            .collect();

        let report = build_assets_report(tmp.path(), &files).unwrap();
        let cat_file_sum: usize = report.categories.iter().map(|c| c.files).sum();
        prop_assert_eq!(report.total_files, cat_file_sum);
    }

    #[test]
    fn top_files_never_exceeds_ten(
        count in 0usize..30,
    ) {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = (0..count)
            .map(|i| {
                let name = format!("f_{i}.png");
                let full = tmp.path().join(&name);
                std::fs::write(&full, vec![0u8; 10]).unwrap();
                PathBuf::from(name)
            })
            .collect();

        let report = build_assets_report(tmp.path(), &files).unwrap();
        prop_assert!(report.top_files.len() <= 10);
    }

    #[test]
    fn top_files_sorted_descending_by_bytes(
        sizes in prop::collection::vec(arb_file_size(), 1..15),
    ) {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = sizes
            .iter()
            .enumerate()
            .map(|(i, &sz)| {
                let name = format!("asset_{i}.mp4");
                let full = tmp.path().join(&name);
                std::fs::write(&full, vec![0u8; sz]).unwrap();
                PathBuf::from(name)
            })
            .collect();

        let report = build_assets_report(tmp.path(), &files).unwrap();
        for pair in report.top_files.windows(2) {
            prop_assert!(pair[0].bytes >= pair[1].bytes);
        }
    }

    #[test]
    fn categories_sorted_descending_by_bytes(
        img_size in arb_file_size(),
        vid_size in arb_file_size(),
        aud_size in arb_file_size(),
    ) {
        let tmp = TempDir::new().unwrap();
        let files = vec![
            {
                let name = "photo.png";
                let full = tmp.path().join(name);
                std::fs::write(&full, vec![0u8; img_size]).unwrap();
                PathBuf::from(name)
            },
            {
                let name = "movie.mp4";
                let full = tmp.path().join(name);
                std::fs::write(&full, vec![0u8; vid_size]).unwrap();
                PathBuf::from(name)
            },
            {
                let name = "track.mp3";
                let full = tmp.path().join(name);
                std::fs::write(&full, vec![0u8; aud_size]).unwrap();
                PathBuf::from(name)
            },
        ];

        let report = build_assets_report(tmp.path(), &files).unwrap();
        for pair in report.categories.windows(2) {
            prop_assert!(
                pair[0].bytes >= pair[1].bytes,
                "categories not sorted: {} ({}) >= {} ({})",
                pair[0].category, pair[0].bytes,
                pair[1].category, pair[1].bytes,
            );
        }
    }

    #[test]
    fn non_asset_files_never_counted(
        exts in prop::collection::vec(arb_non_asset_ext(), 1..10),
    ) {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = exts
            .iter()
            .enumerate()
            .map(|(i, ext)| {
                let name = format!("file_{i}.{ext}");
                let full = tmp.path().join(&name);
                std::fs::write(&full, b"content").unwrap();
                PathBuf::from(name)
            })
            .collect();

        let report = build_assets_report(tmp.path(), &files).unwrap();
        prop_assert_eq!(report.total_files, 0);
        prop_assert_eq!(report.total_bytes, 0);
    }

    #[test]
    fn asset_files_always_counted(
        exts in prop::collection::vec(arb_asset_ext(), 1..10),
    ) {
        let tmp = TempDir::new().unwrap();
        let files: Vec<PathBuf> = exts
            .iter()
            .enumerate()
            .map(|(i, ext)| {
                let name = format!("file_{i}.{ext}");
                let full = tmp.path().join(&name);
                std::fs::write(&full, vec![0u8; 16]).unwrap();
                PathBuf::from(name)
            })
            .collect();

        let report = build_assets_report(tmp.path(), &files).unwrap();
        prop_assert_eq!(report.total_files, files.len());
    }

    #[test]
    fn paths_never_contain_backslashes(
        count in 1usize..8,
    ) {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub").join("dir");
        std::fs::create_dir_all(&sub).unwrap();
        let files: Vec<PathBuf> = (0..count)
            .map(|i| {
                let rel = format!("sub/dir/img_{i}.png");
                let full = tmp.path().join(&rel);
                std::fs::write(&full, vec![0u8; 8]).unwrap();
                PathBuf::from(rel)
            })
            .collect();

        let report = build_assets_report(tmp.path(), &files).unwrap();
        for f in &report.top_files {
            prop_assert!(!f.path.contains('\\'), "backslash in path: {}", f.path);
        }
    }
}

// ---------------------------------------------------------------------------
// Dependency report properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn dependency_total_equals_sum_of_lockfile_deps(
        cargo_count in 0usize..20,
        yarn_count in 0usize..10,
    ) {
        let tmp = TempDir::new().unwrap();
        let mut files = Vec::new();

        // Build a Cargo.lock with N [[package]] entries
        let mut cargo_content = String::new();
        for i in 0..cargo_count {
            cargo_content.push_str(&format!("[[package]]\nname = \"dep-{i}\"\n\n"));
        }
        let cargo_rel = tmp.path().join("Cargo.lock");
        std::fs::write(&cargo_rel, &cargo_content).unwrap();
        files.push(PathBuf::from("Cargo.lock"));

        // Build a yarn.lock with N entries
        let mut yarn_content = String::from("# yarn lockfile v1\n\n");
        for i in 0..yarn_count {
            yarn_content.push_str(&format!("dep-{i}@^1.0.0:\n  version \"1.0.{i}\"\n\n"));
        }
        let yarn_rel = tmp.path().join("yarn.lock");
        std::fs::write(&yarn_rel, &yarn_content).unwrap();
        files.push(PathBuf::from("yarn.lock"));

        let report = build_dependency_report(tmp.path(), &files).unwrap();
        let sum: usize = report.lockfiles.iter().map(|l| l.dependencies).sum();
        prop_assert_eq!(report.total, sum);
    }

    #[test]
    fn cargo_lock_count_equals_package_markers(
        n in 0usize..50,
    ) {
        let tmp = TempDir::new().unwrap();
        let mut content = String::new();
        for i in 0..n {
            content.push_str(&format!("[[package]]\nname = \"crate-{i}\"\n\n"));
        }
        let path = tmp.path().join("Cargo.lock");
        std::fs::write(&path, &content).unwrap();

        let report = build_dependency_report(tmp.path(), &[PathBuf::from("Cargo.lock")]).unwrap();
        prop_assert_eq!(report.lockfiles[0].dependencies, n);
    }

    #[test]
    fn go_sum_deduplicates_go_mod_lines(
        n in 1usize..20,
    ) {
        let tmp = TempDir::new().unwrap();
        let mut content = String::new();
        for i in 0..n {
            // Each module has a source line + go.mod line; only source should count
            content.push_str(&format!(
                "example.com/mod{i} v1.0.0 h1:abc=\nexample.com/mod{i} v1.0.0/go.mod h1:def=\n"
            ));
        }
        let path = tmp.path().join("go.sum");
        std::fs::write(&path, &content).unwrap();

        let report = build_dependency_report(tmp.path(), &[PathBuf::from("go.sum")]).unwrap();
        prop_assert_eq!(report.lockfiles[0].dependencies, n);
    }
}
