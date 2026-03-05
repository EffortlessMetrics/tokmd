//! W57 depth tests for tokmd-analysis-archetype.

use std::collections::BTreeSet;
use tokmd_analysis_archetype::detect_archetype;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn export_with_paths(paths: &[&str]) -> ExportData {
    let rows = paths
        .iter()
        .map(|p| FileRow {
            path: (*p).to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        })
        .collect();
    ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ===========================================================================
// 1. Web-app archetype patterns
// ===========================================================================

#[test]
fn nextjs_detected_with_js_config() {
    let e = export_with_paths(&["package.json", "next.config.js", "pages/index.tsx"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn nextjs_detected_with_mjs_config() {
    let e = export_with_paths(&["package.json", "next.config.mjs"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn nextjs_detected_with_ts_config() {
    let e = export_with_paths(&["package.json", "next.config.ts"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn nextjs_not_detected_without_package_json() {
    let e = export_with_paths(&["next.config.js"]);
    assert!(detect_archetype(&e).is_none());
}

// ===========================================================================
// 2. CLI tool patterns
// ===========================================================================

#[test]
fn rust_cli_workspace_with_main_rs() {
    let e = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs", "src/main.rs"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn rust_cli_workspace_with_bin_dir() {
    let e = export_with_paths(&["Cargo.toml", "crates/cli/src/bin/app.rs"]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.kind.contains("CLI"));
}

// ===========================================================================
// 3. Library patterns
// ===========================================================================

#[test]
fn rust_library_workspace_no_cli() {
    let e = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(!a.kind.contains("CLI"));
}

#[test]
fn python_package_detected() {
    let e = export_with_paths(&["pyproject.toml", "src/mylib/__init__.py"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Python package");
}

#[test]
fn node_package_detected() {
    let e = export_with_paths(&["package.json", "src/index.js"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Node package");
}

// ===========================================================================
// 4. Monorepo / special patterns
// ===========================================================================

#[test]
fn rust_workspace_uses_packages_dir() {
    let e = export_with_paths(&["Cargo.toml", "packages/core/src/lib.rs"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Rust workspace");
}

#[test]
fn containerized_service_detected() {
    let e = export_with_paths(&["Dockerfile", "k8s/deployment.yaml", "src/main.go"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn containerized_service_with_kubernetes_dir() {
    let e = export_with_paths(&["Dockerfile", "kubernetes/service.yaml"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn iac_project_with_tf_file() {
    let e = export_with_paths(&["main.tf", "variables.tf"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn iac_project_with_terraform_dir() {
    let e = export_with_paths(&["terraform/main.tf"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ===========================================================================
// 5. Evidence / confidence scoring
// ===========================================================================

#[test]
fn rust_workspace_evidence_includes_cargo_toml() {
    let e = export_with_paths(&["Cargo.toml", "crates/foo/src/lib.rs"]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.evidence.contains(&"Cargo.toml".to_string()));
}

#[test]
fn rust_workspace_evidence_includes_workspace_dir() {
    let e = export_with_paths(&["Cargo.toml", "crates/foo/src/lib.rs"]);
    let a = detect_archetype(&e).unwrap();
    assert!(
        a.evidence
            .iter()
            .any(|e| e.starts_with("crates/") || e.starts_with("packages/"))
    );
}

#[test]
fn nextjs_evidence_includes_config_file() {
    let e = export_with_paths(&["package.json", "next.config.js"]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.evidence.iter().any(|e| e.contains("next.config")));
}

#[test]
fn containerized_evidence_includes_dockerfile() {
    let e = export_with_paths(&["Dockerfile", "k8s/pod.yaml"]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.evidence.contains(&"Dockerfile".to_string()));
}

// ===========================================================================
// 6. Ambiguous repos that match multiple archetypes
// ===========================================================================

#[test]
fn rust_workspace_takes_priority_over_node() {
    let e = export_with_paths(&["Cargo.toml", "crates/foo/src/lib.rs", "package.json"]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.kind.contains("Rust workspace"));
}

#[test]
fn nextjs_takes_priority_over_plain_node() {
    let e = export_with_paths(&["package.json", "next.config.js", "src/index.js"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn rust_workspace_takes_priority_over_containerized() {
    let e = export_with_paths(&[
        "Cargo.toml",
        "crates/api/src/main.rs",
        "Dockerfile",
        "k8s/deployment.yaml",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.kind.contains("Rust workspace"));
}

#[test]
fn containerized_takes_priority_over_python() {
    let e = export_with_paths(&["Dockerfile", "k8s/deploy.yaml", "pyproject.toml"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

// ===========================================================================
// 7. Deterministic selection
// ===========================================================================

#[test]
fn detection_is_deterministic_across_runs() {
    let paths = &[
        "Cargo.toml",
        "crates/a/src/lib.rs",
        "crates/b/src/lib.rs",
        "package.json",
        "next.config.js",
        "Dockerfile",
        "k8s/pod.yaml",
    ];
    let e = export_with_paths(paths);
    let first = detect_archetype(&e).unwrap();
    for _ in 0..10 {
        let again = detect_archetype(&e).unwrap();
        assert_eq!(first.kind, again.kind);
        assert_eq!(first.evidence, again.evidence);
    }
}

#[test]
fn empty_export_returns_none() {
    let e = export_with_paths(&[]);
    assert!(detect_archetype(&e).is_none());
}

#[test]
fn generic_files_return_none() {
    let e = export_with_paths(&["README.md", "LICENSE", "src/lib.rs"]);
    assert!(detect_archetype(&e).is_none());
}

// ===========================================================================
// 8. Serde roundtrips
// ===========================================================================

#[test]
fn archetype_serde_roundtrip_json() {
    let e = export_with_paths(&["Cargo.toml", "crates/foo/src/lib.rs"]);
    let a = detect_archetype(&e).unwrap();
    let json = serde_json::to_string(&a).unwrap();
    let back: tokmd_analysis_types::Archetype = serde_json::from_str(&json).unwrap();
    assert_eq!(a.kind, back.kind);
    assert_eq!(a.evidence, back.evidence);
}

#[test]
fn archetype_serde_roundtrip_nextjs() {
    let e = export_with_paths(&["package.json", "next.config.mjs"]);
    let a = detect_archetype(&e).unwrap();
    let json = serde_json::to_string_pretty(&a).unwrap();
    let back: tokmd_analysis_types::Archetype = serde_json::from_str(&json).unwrap();
    assert_eq!(a.kind, back.kind);
    assert_eq!(a.evidence, back.evidence);
}

#[test]
fn archetype_serde_roundtrip_containerized() {
    let e = export_with_paths(&["Dockerfile", "k8s/svc.yaml"]);
    let a = detect_archetype(&e).unwrap();
    let json = serde_json::to_string(&a).unwrap();
    let back: tokmd_analysis_types::Archetype = serde_json::from_str(&json).unwrap();
    assert_eq!(a.kind, back.kind);
}

// ===========================================================================
// 9. Backslash normalisation in file paths
// ===========================================================================

#[test]
fn backslash_paths_are_normalised() {
    let mut e = export_with_paths(&["Cargo.toml"]);
    e.rows.push(FileRow {
        path: r"crates\core\src\lib.rs".to_string(),
        module: "(root)".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    });
    let a = detect_archetype(&e).unwrap();
    assert!(a.kind.contains("Rust workspace"));
}

#[test]
fn child_rows_are_ignored_during_detection() {
    let mut e = export_with_paths(&["README.md"]);
    e.rows.push(FileRow {
        path: "Cargo.toml".to_string(),
        module: "(root)".to_string(),
        lang: "TOML".to_string(),
        kind: FileKind::Child, // not Parent
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    });
    e.rows.push(FileRow {
        path: "crates/core/src/lib.rs".to_string(),
        module: "(root)".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Child, // not Parent
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    });
    // Only Child rows contain workspace evidence → should NOT detect
    assert!(detect_archetype(&e).is_none());
}
