#[cfg(feature = "git")]
mod git_tests {
    use std::process::Command;

    use tempfile::tempdir;
    use tokmd_analysis::{
        AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
        NearDupScope, analyze,
    };
    use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
    use tokmd_types::{ExportData, FileKind, FileRow};

    fn git_cmd(dir: &std::path::Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .expect("git command failed");
        assert!(status.success());
    }

    #[test]
    fn git_metrics_basic() {
        if !tokmd_git::git_available() {
            return;
        }

        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        git_cmd(root, &["init"]);

        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "fn a() {}\n").unwrap();

        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(root)
            .args(["add", "."])
            .status()
            .expect("git add");

        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(root)
            .args(["commit", "-m", "first"])
            .env("GIT_AUTHOR_NAME", "Alice")
            .env("GIT_AUTHOR_EMAIL", "alice@example.com")
            .env("GIT_AUTHOR_DATE", "2020-01-01T00:00:00Z")
            .env("GIT_COMMITTER_NAME", "Alice")
            .env("GIT_COMMITTER_EMAIL", "alice@example.com")
            .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00Z")
            .status()
            .expect("git commit");

        std::fs::write(root.join("src/lib.rs"), "fn a() {}\nfn b() {}\n").unwrap();
        std::fs::write(root.join("README.md"), "# test\n").unwrap();

        git_cmd(root, &["add", "."]);

        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(root)
            .args(["commit", "-m", "second"])
            .env("GIT_AUTHOR_NAME", "Bob")
            .env("GIT_AUTHOR_EMAIL", "bob@example.com")
            .env("GIT_AUTHOR_DATE", "2020-01-02T00:00:00Z")
            .env("GIT_COMMITTER_NAME", "Bob")
            .env("GIT_COMMITTER_EMAIL", "bob@example.com")
            .env("GIT_COMMITTER_DATE", "2020-01-02T00:00:00Z")
            .status()
            .expect("git commit");

        let export = ExportData {
            rows: vec![
                FileRow {
                    path: "src/lib.rs".to_string(),
                    module: "src".to_string(),
                    lang: "Rust".to_string(),
                    kind: FileKind::Parent,
                    code: 2,
                    comments: 0,
                    blanks: 0,
                    lines: 2,
                    bytes: 20,
                    tokens: 5,
                },
                FileRow {
                    path: "README.md".to_string(),
                    module: "(root)".to_string(),
                    lang: "Markdown".to_string(),
                    kind: FileKind::Parent,
                    code: 1,
                    comments: 0,
                    blanks: 0,
                    lines: 1,
                    bytes: 8,
                    tokens: 2,
                },
            ],
            module_roots: vec!["crates".to_string(), "packages".to_string()],
            module_depth: 2,
            children: tokmd_config::ChildIncludeMode::Separate,
        };

        let ctx = AnalysisContext {
            export,
            root: root.to_path_buf(),
            source: AnalysisSource {
                inputs: vec![root.display().to_string()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: vec!["crates".to_string(), "packages".to_string()],
                module_depth: 2,
                children: "separate".to_string(),
            },
        };

        let request = AnalysisRequest {
            preset: AnalysisPreset::Risk,
            args: AnalysisArgsMeta {
                preset: "risk".to_string(),
                format: "md".to_string(),
                window_tokens: None,
                git: Some(true),
                max_files: None,
                max_bytes: None,
                max_file_bytes: None,
                max_commits: None,
                max_commit_files: None,
                import_granularity: "module".to_string(),
            },
            limits: AnalysisLimits::default(),
            window_tokens: None,
            git: Some(true),
            import_granularity: ImportGranularity::Module,
            detail_functions: false,
            near_dup: false,
            near_dup_threshold: 0.80,
            near_dup_max_files: 2000,
            near_dup_scope: NearDupScope::Module,
        };

        let receipt = analyze(ctx, request).expect("analysis");
        let git = receipt.git.expect("git report");
        assert_eq!(git.commits_scanned, 2);
        assert!(!git.hotspots.is_empty());
        assert!(!git.bus_factor.is_empty());
        let age = git.age_distribution.expect("age distribution");
        assert_eq!(age.recent_refreshes, 2);
        assert_eq!(age.prior_refreshes, 0);
        assert_eq!(age.refresh_trend, tokmd_analysis_types::TrendClass::Rising);
        assert!(!age.buckets.is_empty());
    }
}
