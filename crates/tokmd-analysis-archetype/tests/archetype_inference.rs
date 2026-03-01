//! Tests for archetype inference across known project patterns.
//!
//! Covers: web app, CLI tool, library, data pipeline, mixed signals,
//! determinism, and edge-case file layouts.

use tokmd_analysis_archetype::detect_archetype;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn row(path: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: "(root)".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 1,
        comments: 0,
        blanks: 0,
        lines: 1,
        bytes: 10,
        tokens: 2,
    }
}

fn export(paths: &[&str]) -> ExportData {
    ExportData {
        rows: paths.iter().map(|p| row(p)).collect(),
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ===========================================================================
// Known pattern: web app (Next.js)
// ===========================================================================

#[test]
fn web_app_nextjs_with_pages_dir() {
    let e = export(&[
        "package.json",
        "next.config.js",
        "pages/index.tsx",
        "pages/api/hello.ts",
        "styles/globals.css",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn web_app_nextjs_with_app_router() {
    let e = export(&[
        "package.json",
        "next.config.mjs",
        "app/layout.tsx",
        "app/page.tsx",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// Known pattern: CLI tool (Rust workspace with binary)
// ===========================================================================

#[test]
fn cli_tool_rust_workspace_with_main() {
    let e = export(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/cli/src/main.rs",
        "src/main.rs",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn cli_tool_rust_workspace_with_bin_dir() {
    let e = export(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/core/src/bin/runner.rs",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

// ===========================================================================
// Known pattern: library (Rust workspace, no binary)
// ===========================================================================

#[test]
fn library_rust_workspace_no_binary() {
    let e = export(&[
        "Cargo.toml",
        "crates/types/src/lib.rs",
        "crates/utils/src/lib.rs",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(!a.kind.contains("CLI"));
}

// ===========================================================================
// Known pattern: data pipeline / IaC
// ===========================================================================

#[test]
fn data_pipeline_terraform() {
    let e = export(&["terraform/main.tf", "terraform/variables.tf", "terraform/outputs.tf"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn iac_standalone_tf_files() {
    let e = export(&["main.tf", "provider.tf", "outputs.tf"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ===========================================================================
// Known pattern: containerized microservice
// ===========================================================================

#[test]
fn containerized_service_k8s() {
    let e = export(&[
        "Dockerfile",
        "k8s/deployment.yaml",
        "k8s/service.yaml",
        "src/main.go",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn containerized_service_kubernetes_dir() {
    let e = export(&[
        "Dockerfile",
        "kubernetes/deployment.yaml",
        "cmd/server/main.go",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

// ===========================================================================
// Known pattern: Python package
// ===========================================================================

#[test]
fn python_package_with_src_layout() {
    let e = export(&[
        "pyproject.toml",
        "src/mypackage/__init__.py",
        "src/mypackage/core.py",
        "tests/test_core.py",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Python package");
}

// ===========================================================================
// Known pattern: Node package (plain)
// ===========================================================================

#[test]
fn node_package_express_app() {
    let e = export(&[
        "package.json",
        "src/index.js",
        "src/routes/api.js",
        "package-lock.json",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Node package");
}

// ===========================================================================
// Determinism: same input always produces same classification
// ===========================================================================

#[test]
fn determinism_across_100_runs() {
    let scenarios: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"],
        vec!["package.json", "next.config.ts"],
        vec!["Dockerfile", "k8s/pod.yaml"],
        vec!["terraform/main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
        vec!["README.md"],
        vec![],
    ];

    for paths in &scenarios {
        let reference = detect_archetype(&export(paths));
        for _ in 0..100 {
            let result = detect_archetype(&export(paths));
            match (&reference, &result) {
                (None, None) => {}
                (Some(a), Some(b)) => {
                    assert_eq!(a.kind, b.kind, "kind mismatch for {paths:?}");
                    assert_eq!(a.evidence, b.evidence, "evidence mismatch for {paths:?}");
                }
                _ => panic!("determinism violated for {paths:?}: {reference:?} vs {result:?}"),
            }
        }
    }
}

// ===========================================================================
// Edge case: duplicate paths should not cause issues
// ===========================================================================

#[test]
fn duplicate_paths_are_deduplicated() {
    let e = export(&["Cargo.toml", "Cargo.toml", "crates/a/src/lib.rs"]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

// ===========================================================================
// Edge case: very deep nesting
// ===========================================================================

#[test]
fn deeply_nested_paths_still_match() {
    let e = export(&[
        "Cargo.toml",
        "crates/deeply/nested/workspace/member/src/lib.rs",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

// ===========================================================================
// Edge case: only non-matching files
// ===========================================================================

#[test]
fn unrecognized_project_returns_none() {
    let e = export(&[
        "Makefile",
        "CMakeLists.txt",
        "src/main.c",
        "include/header.h",
    ]);
    assert!(detect_archetype(&e).is_none());
}

#[test]
fn single_readme_returns_none() {
    let e = export(&["README.md"]);
    assert!(detect_archetype(&e).is_none());
}

// ===========================================================================
// Evidence contains real file paths from input
// ===========================================================================

#[test]
fn evidence_paths_are_from_input() {
    let paths = &["Cargo.toml", "crates/core/src/lib.rs", "src/main.rs"];
    let e = export(paths);
    let a = detect_archetype(&e).unwrap();
    for ev in &a.evidence {
        assert!(
            paths.contains(&ev.as_str()),
            "evidence '{ev}' not found in input paths {paths:?}"
        );
    }
}

#[test]
fn nextjs_evidence_includes_config_file() {
    let e = export(&["package.json", "next.config.ts", "app/page.tsx"]);
    let a = detect_archetype(&e).unwrap();
    assert!(
        a.evidence.iter().any(|e| e.contains("next.config")),
        "evidence should include next.config file: {:?}",
        a.evidence
    );
}

// ===========================================================================
// Priority: full chain with one missing signal per level
// ===========================================================================

#[test]
fn without_rust_signals_nextjs_wins() {
    let e = export(&[
        "package.json",
        "next.config.js",
        "Dockerfile",
        "k8s/deploy.yaml",
        "main.tf",
        "pyproject.toml",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn without_rust_or_nextjs_container_wins() {
    let e = export(&[
        "package.json",
        "Dockerfile",
        "k8s/deploy.yaml",
        "main.tf",
        "pyproject.toml",
    ]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn without_rust_nextjs_container_iac_wins() {
    let e = export(&["package.json", "main.tf", "pyproject.toml"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn without_rust_nextjs_container_iac_python_wins() {
    let e = export(&["package.json", "pyproject.toml"]);
    let a = detect_archetype(&e).unwrap();
    assert_eq!(a.kind, "Python package");
}
