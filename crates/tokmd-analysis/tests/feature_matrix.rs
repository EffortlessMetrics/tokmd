#[cfg(feature = "git")]
use std::process::Command;
use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn make_context(export: ExportData) -> AnalysisContext {
    AnalysisContext {
        export,
        root: std::path::PathBuf::from("."),
        source: AnalysisSource {
            inputs: vec![".".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: "separate".to_string(),
        },
    }
}

#[cfg(feature = "git")]
fn make_git_context(export: ExportData) -> (AnalysisContext, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_root = temp_dir.path();

    std::fs::create_dir_all(repo_root.join("src")).unwrap();
    std::fs::write(
        repo_root.join("Cargo.toml"),
        "[package]\nname = \"mini\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(repo_root.join("src/lib.rs"), "pub fn foo() {}\n").unwrap();

    assert!(
        Command::new("git")
            .args(["init"])
            .current_dir(repo_root)
            .status()
            .expect("git init")
            .success()
    );

    assert!(
        Command::new("git")
            .args(["config", "user.email", "builder@example.internal"])
            .current_dir(repo_root)
            .status()
            .expect("git config user.email")
            .success()
    );

    assert!(
        Command::new("git")
            .args(["config", "user.name", "Builder"])
            .current_dir(repo_root)
            .status()
            .expect("git config user.name")
            .success()
    );

    assert!(
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_root)
            .status()
            .expect("git add")
            .success()
    );

    assert!(
        Command::new("git")
            .args(["commit", "-m", "fixture seed"])
            .current_dir(repo_root)
            .status()
            .expect("git commit")
            .success()
    );

    let context = AnalysisContext {
        export,
        root: repo_root.to_path_buf(),
        source: AnalysisSource {
            inputs: vec![repo_root.to_string_lossy().to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["(root)".to_string()],
            module_depth: 1,
            children: "separate".to_string(),
        },
    };

    (context, temp_dir)
}

fn make_request(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: format!("{:?}", preset).to_lowercase(),
            format: "json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        window_tokens: None,
        git: None,
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 1_000,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    }
}

fn export_with_paths(rows: Vec<(&str, &str, &str, usize)>) -> ExportData {
    let rows = rows
        .into_iter()
        .map(|(path, module, lang, lines)| FileRow {
            path: path.to_string(),
            module: module.to_string(),
            lang: lang.to_string(),
            kind: FileKind::Parent,
            code: lines,
            comments: 0,
            blanks: 0,
            lines,
            bytes: 1024 + lines,
            tokens: lines * 2,
        })
        .collect();

    ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

#[cfg(feature = "archetype")]
#[test]
fn archetype_feature_is_included() {
    // Given: an export that looks like a Rust workspace
    // When: Identity analysis runs with archetype detection enabled
    // Then: archetype should resolve to Rust workspace
    let export = export_with_paths(vec![
        ("Cargo.toml", "(root)", "TOML", 1),
        ("crates/core/Cargo.toml", "crates/core", "TOML", 1),
        ("src/main.rs", "src", "Rust", 10),
    ]);

    let receipt =
        analyze(make_context(export), make_request(AnalysisPreset::Identity)).expect("analyze");
    let archetype = receipt.archetype.expect("archetype should be present");

    assert!(archetype.kind.contains("Rust workspace"));
}

#[cfg(not(feature = "archetype"))]
#[test]
fn archetype_feature_is_reported_when_disabled() {
    // Given: an export that could support archetype detection
    // When: Identity analysis runs without archetype feature
    // Then: receipt should not include archetype and should report a warning
    let export = export_with_paths(vec![
        ("Cargo.toml", "(root)", "TOML", 1),
        ("crates/core/Cargo.toml", "crates/core", "TOML", 1),
        ("src/main.rs", "src", "Rust", 10),
    ]);

    let receipt =
        analyze(make_context(export), make_request(AnalysisPreset::Identity)).expect("analyze");

    assert!(receipt.archetype.is_none());
    assert!(receipt.warnings.iter().any(|warning| {
        warning.contains(tokmd_analysis_grid::DisabledFeature::Archetype.warning())
    }),);
}

#[cfg(feature = "topics")]
#[test]
fn topics_feature_is_included() {
    // Given: an export with multiple module file patterns
    // When: Topics preset runs with topic extractor enabled
    // Then: per-module topics should include representative term tokens
    let export = export_with_paths(vec![
        ("crates/auth/src/login.rs", "crates/auth", "Rust", 20),
        (
            "crates/payments/src/stripe_api.rs",
            "crates/payments",
            "Rust",
            20,
        ),
    ]);

    let receipt =
        analyze(make_context(export), make_request(AnalysisPreset::Topics)).expect("analyze");
    let topics = receipt.topics.expect("topics should be present");

    let auth = topics.per_module.get("crates/auth").unwrap();
    assert!(auth.iter().any(|term| term.term == "login"));
}

#[cfg(feature = "fun")]
#[test]
fn fun_feature_is_included() {
    // Given: a repo-like export with a few non-zero file bytes
    // When: Fun preset runs with the fun feature enabled
    // Then: fun data and eco-label should be present
    let export = export_with_paths(vec![("src/lib.rs", "src", "Rust", 50)]);
    let mut request = make_request(AnalysisPreset::Fun);
    request.args.preset = "fun".to_string();

    let receipt = analyze(make_context(export), request).expect("analyze");
    let fun = receipt.fun.expect("fun should be present");
    let eco_label = fun.eco_label.expect("eco_label should be present");

    assert!(eco_label.score > 0.0);
    assert!(!eco_label.label.is_empty());
    assert!(eco_label.notes.contains("MB"));
}

#[cfg(not(feature = "fun"))]
#[test]
fn fun_feature_is_reported_when_disabled() {
    // Given: an export where fun output would be meaningful
    // When: Fun preset runs without fun feature
    // Then: fun section should be absent and warning should explain the feature gate
    let export = export_with_paths(vec![("src/lib.rs", "src", "Rust", 50)]);
    let mut request = make_request(AnalysisPreset::Fun);
    request.args.preset = "fun".to_string();

    let receipt = analyze(make_context(export), request).expect("analyze");

    assert!(receipt.fun.is_none());
    assert!(
        receipt
            .warnings
            .iter()
            .any(|warning| warning.contains(tokmd_analysis_grid::DisabledFeature::Fun.warning())),
    );
}

#[cfg(feature = "git")]
#[test]
fn fingerprint_feature_is_included() {
    // Given: analysis on a git-backed export
    // When: Identity preset runs with git enabled by default
    // Then: fingerprint enrichment should be computed and surfaced
    let export = export_with_paths(vec![
        ("Cargo.toml", "(root)", "TOML", 1),
        ("src/lib.rs", "src", "Rust", 10),
    ]);

    let mut request = make_request(AnalysisPreset::Identity);
    request.args.preset = "identity".to_string();

    let (ctx, _repo) = make_git_context(export);
    let receipt = analyze(ctx, request).expect("analyze");
    let fingerprint = receipt
        .corporate_fingerprint
        .expect("corporate_fingerprint should be present");

    assert!(
        !fingerprint.domains.is_empty(),
        "fingerprint should contain domain statistics"
    );
}

#[cfg(not(feature = "git"))]
#[test]
fn fingerprint_feature_is_reported_when_disabled() {
    // Given: identity mode that would request fingerprint output
    // When: git feature is disabled
    // Then: fingerprint section is absent and warning explains missing git feature
    let export = export_with_paths(vec![("Cargo.toml", "(root)", "TOML", 1)]);

    let mut request = make_request(AnalysisPreset::Identity);
    request.git = Some(true);
    let receipt = analyze(make_context(export), request).expect("analyze");
    assert!(receipt.corporate_fingerprint.is_none());
    assert!(receipt.warnings.iter().any(|warning| {
        warning.contains(tokmd_analysis_grid::DisabledFeature::GitMetrics.warning())
    }),);
}

#[cfg(not(feature = "topics"))]
#[test]
fn topics_feature_is_reported_when_disabled() {
    // Given: an export with multiple module file patterns
    // When: Topics preset runs without topics feature
    // Then: topics section should be absent and warning should explain missing feature
    let export = export_with_paths(vec![
        ("crates/auth/src/login.rs", "crates/auth", "Rust", 20),
        (
            "crates/payments/src/stripe_api.rs",
            "crates/payments",
            "Rust",
            20,
        ),
    ]);

    let receipt =
        analyze(make_context(export), make_request(AnalysisPreset::Topics)).expect("analyze");

    assert!(receipt.topics.is_none());
    assert!(receipt
        .warnings
        .iter()
        .any(|warning| warning.contains(tokmd_analysis_grid::DisabledFeature::Topics.warning())),);
}
