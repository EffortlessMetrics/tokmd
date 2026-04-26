//! Deep W68 tests for archetype inference.
//!
//! Covers: all archetype kinds, priority ordering, evidence content,
//! edge cases (empty, single file, mixed kinds), multi-language repos,
//! deterministic detection, and negative cases.

use crate::archetype::detect_archetype;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

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

fn export_with(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ── 1. Empty repo returns None ──────────────────────────────────

#[test]
fn empty_repo_returns_none() {
    let export = export_with(vec![]);
    assert!(detect_archetype(&export).is_none());
}

// ── 2. Single generic file returns None ─────────────────────────

#[test]
fn single_generic_file_returns_none() {
    let export = export_with(vec![parent_row("README.md")]);
    assert!(detect_archetype(&export).is_none());
}

// ── 3. Rust workspace library ───────────────────────────────────

#[test]
fn rust_workspace_library() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("crates/core/src/lib.rs"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(!a.kind.contains("CLI"));
}

// ── 4. Rust workspace CLI with main.rs ──────────────────────────

#[test]
fn rust_workspace_cli_main_rs() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("crates/core/src/lib.rs"),
        parent_row("src/main.rs"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

// ── 5. Rust workspace CLI with bin dir ──────────────────────────

#[test]
fn rust_workspace_cli_bin_dir() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("crates/app/src/bin/run.rs"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.contains("CLI"));
}

// ── 6. Rust workspace with packages/ dir ────────────────────────

#[test]
fn rust_workspace_packages_dir() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("packages/utils/src/lib.rs"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.contains("Rust workspace"));
    assert!(a.evidence.iter().any(|e| e.starts_with("packages/")));
}

// ── 7. Rust workspace evidence includes Cargo.toml ──────────────

#[test]
fn rust_workspace_evidence_includes_cargo_toml() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("crates/foo/src/lib.rs"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.evidence.contains(&"Cargo.toml".to_string()));
}

// ── 8. Next.js app detected ─────────────────────────────────────

#[test]
fn nextjs_app_detected() {
    let export = export_with(vec![
        parent_row("package.json"),
        parent_row("next.config.js"),
        parent_row("pages/index.tsx"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ── 9. Next.js with .mjs config ────────────────────────────────

#[test]
fn nextjs_mjs_config() {
    let export = export_with(vec![
        parent_row("package.json"),
        parent_row("next.config.mjs"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ── 10. Containerized service ───────────────────────────────────

#[test]
fn containerized_service_detected() {
    let export = export_with(vec![
        parent_row("Dockerfile"),
        parent_row("k8s/deployment.yaml"),
        parent_row("src/main.go"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
    assert!(a.evidence.contains(&"Dockerfile".to_string()));
}

// ── 11. Containerized service with kubernetes/ dir ──────────────

#[test]
fn containerized_service_kubernetes_dir() {
    let export = export_with(vec![
        parent_row("Dockerfile"),
        parent_row("kubernetes/service.yaml"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

// ── 12. IaC with .tf file ───────────────────────────────────────

#[test]
fn iac_with_tf_file() {
    let export = export_with(vec![parent_row("main.tf"), parent_row("variables.tf")]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ── 13. IaC with terraform/ dir ─────────────────────────────────

#[test]
fn iac_with_terraform_dir() {
    let export = export_with(vec![parent_row("terraform/main.tf")]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ── 14. Python package ──────────────────────────────────────────

#[test]
fn python_package_detected() {
    let export = export_with(vec![
        parent_row("pyproject.toml"),
        parent_row("src/mylib/__init__.py"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
    assert!(a.evidence.contains(&"pyproject.toml".to_string()));
}

// ── 15. Node package ────────────────────────────────────────────

#[test]
fn node_package_detected() {
    let export = export_with(vec![parent_row("package.json"), parent_row("src/index.js")]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
}

// ── 16. Priority: Rust workspace > Node ─────────────────────────

#[test]
fn rust_workspace_priority_over_node() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("crates/lib/src/lib.rs"),
        parent_row("package.json"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.contains("Rust workspace"));
}

// ── 17. Priority: Next.js > Node ────────────────────────────────

#[test]
fn nextjs_priority_over_node() {
    let export = export_with(vec![
        parent_row("package.json"),
        parent_row("next.config.ts"),
        parent_row("src/index.ts"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ── 18. Child rows ignored in detection ─────────────────────────

#[test]
fn child_rows_ignored() {
    // Only child rows with Cargo.toml and crates/ — should not detect workspace
    let export = export_with(vec![
        child_row("Cargo.toml"),
        child_row("crates/core/src/lib.rs"),
    ]);
    assert!(detect_archetype(&export).is_none());
}

// ── 19. Detection is deterministic ──────────────────────────────

#[test]
fn detection_is_deterministic() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("crates/a/src/lib.rs"),
        parent_row("package.json"),
        parent_row("Dockerfile"),
        parent_row("k8s/deploy.yaml"),
    ]);
    let a1 = detect_archetype(&export);
    let a2 = detect_archetype(&export);
    assert_eq!(a1.as_ref().map(|a| &a.kind), a2.as_ref().map(|a| &a.kind));
}

// ── 20. Backslash paths normalized in detection ─────────────────

#[test]
fn backslash_paths_normalized() {
    let export = export_with(vec![
        parent_row("Cargo.toml"),
        parent_row("crates\\core\\src\\lib.rs"),
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.contains("Rust workspace"));
}
