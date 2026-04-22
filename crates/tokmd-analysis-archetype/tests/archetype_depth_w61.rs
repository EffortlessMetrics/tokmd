//! Wave-61 depth tests for `tokmd-analysis-archetype`.
//!
//! Covers: BDD edge cases for every archetype detector, priority chain
//! exhaustive combinations, evidence invariants, path normalization,
//! FileKind filtering, ChildIncludeMode, determinism, serde roundtrip,
//! large inputs, and proptest properties.

use proptest::prelude::*;
use tokmd_analysis_archetype::detect_archetype;
use tokmd_analysis_types::Archetype;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────────────

fn parent_row(path: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "(root)".to_string(),
        lang: "Unknown".to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }
}

fn child_row(path: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "(root)".to_string(),
        lang: "Unknown".to_string(),
        kind: FileKind::Child,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }
}

fn export_with_paths(paths: &[&str]) -> ExportData {
    ExportData {
        rows: paths.iter().map(|p| parent_row(p)).collect(),
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn export_from_rows(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

const KNOWN_KINDS: &[&str] = &[
    "Rust workspace",
    "Rust workspace (CLI)",
    "Next.js app",
    "Containerized service",
    "Infrastructure as code",
    "Python package",
    "Node package",
];

// =============================================================================
// 1. Rust workspace – deep edge cases
// =============================================================================

#[test]
fn rust_workspace_with_both_crates_and_packages() {
    let export = export_with_paths(&["Cargo.toml", "crates/a/src/lib.rs", "packages/b/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
    // Evidence should pick first matching workspace dir
    assert!(
        a.evidence
            .iter()
            .any(|e| e.starts_with("crates/") || e.starts_with("packages/")),
    );
}

#[test]
fn rust_workspace_cli_via_bin_in_deep_path() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/tool/src/lib.rs",
        "crates/tool/src/bin/main.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn rust_workspace_cli_via_root_main_rs() {
    let export = export_with_paths(&["Cargo.toml", "crates/lib/src/lib.rs", "src/main.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn rust_workspace_evidence_always_starts_with_cargo_toml() {
    let export = export_with_paths(&["Cargo.toml", "crates/z/src/lib.rs", "crates/a/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence[0], "Cargo.toml");
}

#[test]
fn rust_workspace_evidence_has_exactly_two_entries() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/utils/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 2);
}

#[test]
fn rust_workspace_only_packages_no_crates() {
    let export = export_with_paths(&["Cargo.toml", "packages/core/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(a.evidence[1].starts_with("packages/"));
}

// =============================================================================
// 2. Next.js – deep edge cases
// =============================================================================

#[test]
fn nextjs_with_starts_with_next_config_dot() {
    // next.config.json should NOT trigger Next.js detection
    let export = export_with_paths(&["package.json", "next.config.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
}

#[test]
fn nextjs_evidence_always_starts_with_package_json() {
    let export = export_with_paths(&["package.json", "next.config.mjs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence[0], "package.json");
}

#[test]
fn nextjs_with_multiple_next_configs_picks_first_match() {
    let export = export_with_paths(&[
        "package.json",
        "next.config.js",
        "next.config.mjs",
        "next.config.ts",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
    assert_eq!(a.evidence.len(), 2);
}

#[test]
fn nextjs_deeply_nested_config() {
    let export = export_with_paths(&["package.json", "apps/web/frontend/next.config.js"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
    assert!(a.evidence.iter().any(|e| e.contains("next.config")));
}

// =============================================================================
// 3. Containerized service – deep edge cases
// =============================================================================

#[test]
fn containerized_with_kubernetes_deep_path() {
    let export = export_with_paths(&["Dockerfile", "kubernetes/charts/app/deploy.yaml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn containerized_evidence_only_dockerfile() {
    let export = export_with_paths(&["Dockerfile", "k8s/deploy.yaml", "k8s/service.yaml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 1);
    assert_eq!(a.evidence[0], "Dockerfile");
}

#[test]
fn not_containerized_without_dockerfile() {
    let export = export_with_paths(&["k8s/deploy.yaml"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn not_containerized_without_k8s_dir() {
    let export = export_with_paths(&["Dockerfile"]);
    assert!(detect_archetype(&export).is_none());
}

// =============================================================================
// 4. IaC – deep edge cases
// =============================================================================

#[test]
fn iac_with_dot_tf_in_subdirectory() {
    let export = export_with_paths(&["infra/modules/vpc/main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn iac_evidence_always_terraform_slash() {
    let export = export_with_paths(&["terraform/main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence, vec!["terraform/"]);
}

#[test]
fn iac_with_only_tf_extension_no_terraform_dir() {
    let export = export_with_paths(&["variables.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn not_iac_without_tf_files() {
    let export = export_with_paths(&["infra/main.yaml", "deploy/helm/chart.yaml"]);
    assert!(detect_archetype(&export).is_none());
}

// =============================================================================
// 5. Python package – deep edge cases
// =============================================================================

#[test]
fn python_with_pyproject_only() {
    let export = export_with_paths(&["pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
    assert_eq!(a.evidence, vec!["pyproject.toml"]);
}

#[test]
fn python_not_detected_with_setup_py_only() {
    let export = export_with_paths(&["setup.py"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn python_not_detected_with_requirements_only() {
    let export = export_with_paths(&["requirements.txt"]);
    assert!(detect_archetype(&export).is_none());
}

// =============================================================================
// 6. Node package – deep edge cases
// =============================================================================

#[test]
fn node_package_minimal() {
    let export = export_with_paths(&["package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
    assert_eq!(a.evidence, vec!["package.json"]);
}

#[test]
fn node_package_with_many_files() {
    let export = export_with_paths(&[
        "package.json",
        "src/index.ts",
        "src/utils.ts",
        "tests/index.test.ts",
        "dist/bundle.js",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
}

// =============================================================================
// 7. Priority chain – exhaustive pairwise verification
// =============================================================================

#[test]
fn priority_rust_over_nextjs() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/a/src/lib.rs",
        "package.json",
        "next.config.js",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn priority_rust_over_containerized() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/a/src/lib.rs",
        "Dockerfile",
        "k8s/deploy.yaml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn priority_rust_over_iac() {
    let export = export_with_paths(&["Cargo.toml", "crates/a/src/lib.rs", "main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn priority_rust_over_python() {
    let export = export_with_paths(&["Cargo.toml", "crates/a/src/lib.rs", "pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn priority_nextjs_over_containerized() {
    let export = export_with_paths(&[
        "package.json",
        "next.config.js",
        "Dockerfile",
        "k8s/deploy.yaml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn priority_nextjs_over_iac() {
    let export = export_with_paths(&["package.json", "next.config.js", "main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn priority_nextjs_over_python() {
    let export = export_with_paths(&["package.json", "next.config.js", "pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn priority_containerized_over_iac() {
    let export = export_with_paths(&["Dockerfile", "k8s/deploy.yaml", "main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn priority_containerized_over_python() {
    let export = export_with_paths(&["Dockerfile", "k8s/deploy.yaml", "pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn priority_containerized_over_node() {
    let export = export_with_paths(&["Dockerfile", "k8s/deploy.yaml", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn priority_iac_over_python() {
    let export = export_with_paths(&["main.tf", "pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn priority_iac_over_node() {
    let export = export_with_paths(&["main.tf", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn priority_python_over_node() {
    let export = export_with_paths(&["pyproject.toml", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

// =============================================================================
// 8. Empty and unrecognized inputs
// =============================================================================

#[test]
fn empty_export_returns_none() {
    let export = export_with_paths(&[]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn single_unknown_file_returns_none() {
    let export = export_with_paths(&["random.xyz"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn many_source_files_no_markers_returns_none() {
    let paths: Vec<String> = (0..50).map(|i| format!("src/file_{i}.rs")).collect();
    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    let export = export_with_paths(&refs);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn cargo_toml_alone_no_workspace() {
    let export = export_with_paths(&["Cargo.toml"]);
    assert!(detect_archetype(&export).is_none());
}

// =============================================================================
// 9. FileKind::Child ignored by detector
// =============================================================================

#[test]
fn all_child_rows_never_trigger_detection() {
    let rows = vec![
        child_row("Cargo.toml"),
        child_row("crates/core/src/lib.rs"),
        child_row("package.json"),
        child_row("next.config.js"),
        child_row("Dockerfile"),
        child_row("k8s/deploy.yaml"),
        child_row("pyproject.toml"),
        child_row("main.tf"),
    ];
    let export = export_from_rows(rows);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn mixed_parent_child_only_parents_matter() {
    let rows = vec![
        parent_row("pyproject.toml"),
        child_row("Cargo.toml"),
        child_row("crates/a/src/lib.rs"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

// =============================================================================
// 10. Backslash normalization
// =============================================================================

#[test]
fn backslash_terraform_path_detected() {
    let rows = vec![parent_row("terraform\\main.tf")];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn backslash_packages_path_detected() {
    let rows = vec![
        parent_row("Cargo.toml"),
        parent_row("packages\\core\\src\\lib.rs"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn backslash_kubernetes_path_detected() {
    let rows = vec![
        parent_row("Dockerfile"),
        parent_row("kubernetes\\deploy.yaml"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

// =============================================================================
// 11. ChildIncludeMode variations
// =============================================================================

#[test]
fn separate_mode_detects_rust_workspace() {
    let export = ExportData {
        rows: vec![parent_row("Cargo.toml"), parent_row("crates/a/src/lib.rs")],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
}

#[test]
fn parents_only_mode_detects_archetype() {
    let export = ExportData {
        rows: vec![parent_row("package.json"), parent_row("next.config.ts")],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    };
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// =============================================================================
// 12. Determinism
// =============================================================================

#[test]
fn deterministic_over_100_iterations_all_archetypes() {
    let cases: Vec<(&str, Vec<&str>)> = vec![
        ("Rust workspace", vec!["Cargo.toml", "crates/a/src/lib.rs"]),
        (
            "Rust workspace (CLI)",
            vec!["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"],
        ),
        ("Next.js app", vec!["package.json", "next.config.js"]),
        (
            "Containerized service",
            vec!["Dockerfile", "k8s/deploy.yaml"],
        ),
        ("Infrastructure as code", vec!["main.tf"]),
        ("Python package", vec!["pyproject.toml"]),
        ("Node package", vec!["package.json"]),
    ];
    for (expected_kind, paths) in &cases {
        let export = export_with_paths(paths);
        for _ in 0..100 {
            let result = detect_archetype(&export).unwrap();
            assert_eq!(
                result.kind, *expected_kind,
                "non-deterministic for {expected_kind}"
            );
        }
    }
}

// =============================================================================
// 13. Serde roundtrip
// =============================================================================

#[test]
fn serde_roundtrip_every_archetype() {
    let cases = vec![
        export_with_paths(&["Cargo.toml", "crates/a/src/lib.rs"]),
        export_with_paths(&["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"]),
        export_with_paths(&["package.json", "next.config.js"]),
        export_with_paths(&["Dockerfile", "k8s/deploy.yaml"]),
        export_with_paths(&["main.tf"]),
        export_with_paths(&["pyproject.toml"]),
        export_with_paths(&["package.json"]),
    ];
    for export in &cases {
        let a = detect_archetype(export).unwrap();
        let json = serde_json::to_string(&a).unwrap();
        let b: Archetype = serde_json::from_str(&json).unwrap();
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.evidence, b.evidence);
    }
}

#[test]
fn archetype_json_shape_always_two_keys() {
    let a = Archetype {
        kind: "Test".to_string(),
        evidence: vec!["a".to_string(), "b".to_string()],
    };
    let v: serde_json::Value = serde_json::to_value(&a).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert!(obj.contains_key("kind"));
    assert!(obj.contains_key("evidence"));
}

#[test]
fn archetype_evidence_serializes_as_array_of_strings() {
    let a = Archetype {
        kind: "X".to_string(),
        evidence: vec!["one".to_string(), "two".to_string()],
    };
    let v: serde_json::Value = serde_json::to_value(&a).unwrap();
    let arr = v["evidence"].as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert!(arr.iter().all(|v| v.is_string()));
}

// =============================================================================
// 14. Large inputs
// =============================================================================

#[test]
fn large_repo_1000_files_with_markers() {
    let mut paths: Vec<String> = (0..1000).map(|i| format!("src/gen/file_{i}.rs")).collect();
    paths.push("Cargo.toml".to_string());
    paths.push("crates/core/src/lib.rs".to_string());
    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    let export = export_with_paths(&refs);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn large_repo_1000_files_without_markers() {
    let paths: Vec<String> = (0..1000).map(|i| format!("src/gen/file_{i}.go")).collect();
    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    let export = export_with_paths(&refs);
    assert!(detect_archetype(&export).is_none());
}

// =============================================================================
// 15. Evidence never contains backslashes
// =============================================================================

#[test]
fn evidence_paths_always_forward_slashes() {
    let cases: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/a/src/lib.rs"],
        vec!["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"],
        vec!["package.json", "next.config.js"],
        vec!["Dockerfile", "k8s/deploy.yaml"],
        vec!["terraform/main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
    ];
    for paths in &cases {
        let export = export_with_paths(paths);
        if let Some(a) = detect_archetype(&export) {
            for ev in &a.evidence {
                assert!(
                    !ev.contains('\\'),
                    "backslash in evidence '{ev}' for kind={}",
                    a.kind
                );
            }
        }
    }
}

// =============================================================================
// 16. Proptest: never panics with random paths
// =============================================================================

fn path_segment() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,8}").unwrap()
}

fn file_path() -> impl Strategy<Value = String> {
    let extensions = prop_oneof![
        Just(".rs"),
        Just(".ts"),
        Just(".py"),
        Just(".js"),
        Just(".toml"),
        Just(".json"),
        Just(".tf"),
        Just(".yaml"),
    ];
    (prop::collection::vec(path_segment(), 1..=3), extensions)
        .prop_map(|(segs, ext)| format!("{}{}", segs.join("/"), ext))
}

fn file_paths_with_markers() -> impl Strategy<Value = Vec<String>> {
    let markers = prop::collection::vec(
        prop_oneof![
            Just("Cargo.toml".to_string()),
            Just("package.json".to_string()),
            Just("Dockerfile".to_string()),
            Just("pyproject.toml".to_string()),
            Just("next.config.js".to_string()),
            Just("main.tf".to_string()),
            Just("crates/x/src/lib.rs".to_string()),
            Just("k8s/deploy.yaml".to_string()),
            Just("kubernetes/pod.yaml".to_string()),
        ],
        0..=4,
    );
    let random = prop::collection::vec(file_path(), 0..=6);
    (markers, random).prop_map(|(mut m, r)| {
        m.extend(r);
        m.sort();
        m.dedup();
        m
    })
}

proptest! {
    #[test]
    fn prop_never_panics(paths in file_paths_with_markers()) {
        let export = ExportData {
            rows: paths.into_iter().map(|p| FileRow {
                path: p,
                module: "(root)".to_string(),
                lang: "Unknown".to_string(),
                kind: FileKind::Parent,
                code: 0, comments: 0, blanks: 0, lines: 0, bytes: 0, tokens: 0,
            }).collect(),
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let _ = detect_archetype(&export);
    }
}

// =============================================================================
// 17. Proptest: detected kind is always in known set
// =============================================================================

proptest! {
    #[test]
    fn prop_kind_always_known(paths in file_paths_with_markers()) {
        let export = ExportData {
            rows: paths.into_iter().map(|p| FileRow {
                path: p,
                module: "(root)".to_string(),
                lang: "Unknown".to_string(),
                kind: FileKind::Parent,
                code: 0, comments: 0, blanks: 0, lines: 0, bytes: 0, tokens: 0,
            }).collect(),
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        if let Some(a) = detect_archetype(&export) {
            prop_assert!(
                KNOWN_KINDS.contains(&a.kind.as_str()),
                "unexpected kind: {}", a.kind
            );
        }
    }
}

// =============================================================================
// 18. Proptest: evidence is non-empty when detected
// =============================================================================

proptest! {
    #[test]
    fn prop_evidence_non_empty(paths in file_paths_with_markers()) {
        let export = ExportData {
            rows: paths.into_iter().map(|p| FileRow {
                path: p,
                module: "(root)".to_string(),
                lang: "Unknown".to_string(),
                kind: FileKind::Parent,
                code: 0, comments: 0, blanks: 0, lines: 0, bytes: 0, tokens: 0,
            }).collect(),
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        if let Some(a) = detect_archetype(&export) {
            prop_assert!(!a.evidence.is_empty());
        }
    }
}

// =============================================================================
// 19. Proptest: deterministic
// =============================================================================

proptest! {
    #[test]
    fn prop_deterministic(paths in file_paths_with_markers()) {
        let mk = |p: &[String]| ExportData {
            rows: p.iter().map(|p| FileRow {
                path: p.clone(),
                module: "(root)".to_string(),
                lang: "Unknown".to_string(),
                kind: FileKind::Parent,
                code: 0, comments: 0, blanks: 0, lines: 0, bytes: 0, tokens: 0,
            }).collect(),
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let r1 = detect_archetype(&mk(&paths));
        let r2 = detect_archetype(&mk(&paths));
        match (&r1, &r2) {
            (None, None) => {}
            (Some(a), Some(b)) => {
                prop_assert_eq!(&a.kind, &b.kind);
                prop_assert_eq!(&a.evidence, &b.evidence);
            }
            _ => prop_assert!(false, "determinism violated"),
        }
    }
}

// =============================================================================
// 20. Proptest: child-only rows never produce detection
// =============================================================================

proptest! {
    #[test]
    fn prop_child_only_never_detected(paths in file_paths_with_markers()) {
        let export = ExportData {
            rows: paths.into_iter().map(|p| FileRow {
                path: p,
                module: "(root)".to_string(),
                lang: "Unknown".to_string(),
                kind: FileKind::Child,
                code: 0, comments: 0, blanks: 0, lines: 0, bytes: 0, tokens: 0,
            }).collect(),
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        prop_assert!(detect_archetype(&export).is_none());
    }
}
