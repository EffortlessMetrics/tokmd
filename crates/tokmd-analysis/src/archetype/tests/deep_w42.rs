//! Wave-42 deep tests for archetype inference.
//!
//! Tests archetype detection for various repo shapes, scoring correctness,
//! priority ordering, serde roundtrips, and edge cases.

use crate::archetype::detect_archetype;
use tokmd_analysis_types::Archetype;
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

fn export_with_paths(paths: &[&str]) -> ExportData {
    ExportData {
        rows: paths.iter().map(|p| parent_row(p)).collect(),
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ── 1. Rust workspace library (no CLI) ──────────────────────────

#[test]
fn rust_workspace_library_no_main() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/utils/src/lib.rs",
    ]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Rust workspace");
    assert!(!arch.kind.contains("CLI"));
}

// ── 2. Rust workspace CLI detected via bin dir ──────────────────

#[test]
fn rust_workspace_cli_via_bin() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/app/src/lib.rs",
        "crates/app/src/bin/main.rs",
    ]);
    let arch = detect_archetype(&export).unwrap();
    assert!(arch.kind.contains("CLI"), "expected CLI: {}", arch.kind);
}

// ── 3. Next.js app with .mjs config ────────────────────────────

#[test]
fn nextjs_mjs_config() {
    let export = export_with_paths(&["package.json", "next.config.mjs", "app/page.tsx"]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Next.js app");
    assert!(arch.evidence.iter().any(|e| e.contains("next.config.mjs")));
}

// ── 4. Containerized service with kubernetes/ directory ──────────

#[test]
fn containerized_service_kubernetes_dir() {
    let export = export_with_paths(&["Dockerfile", "kubernetes/deployment.yaml", "src/main.go"]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Containerized service");
    assert!(arch.evidence.contains(&"Dockerfile".to_string()));
}

// ── 5. Infrastructure-as-code via .tf files ─────────────────────

#[test]
fn iac_detected_from_tf_extension() {
    let export = export_with_paths(&["main.tf", "variables.tf", "outputs.tf"]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Infrastructure as code");
}

// ── 6. Python package via pyproject.toml ────────────────────────

#[test]
fn python_package_pyproject() {
    let export = export_with_paths(&["pyproject.toml", "src/mylib/__init__.py"]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Python package");
    assert!(arch.evidence.contains(&"pyproject.toml".to_string()));
}

// ── 7. Node package fallback ────────────────────────────────────

#[test]
fn node_package_fallback_when_no_nextjs() {
    let export = export_with_paths(&["package.json", "src/index.ts"]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Node package");
}

// ── 8. Priority: Rust workspace > Node package ──────────────────

#[test]
fn priority_rust_over_node() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/wasm/src/lib.rs",
        "package.json",
        "src/index.js",
    ]);
    let arch = detect_archetype(&export).unwrap();
    assert!(
        arch.kind.contains("Rust workspace"),
        "Rust workspace should take priority over Node package: {}",
        arch.kind
    );
}

// ── 9. Serde roundtrip for Archetype ────────────────────────────

#[test]
fn archetype_serde_roundtrip() {
    let arch = Archetype {
        kind: "Rust workspace (CLI)".to_string(),
        evidence: vec![
            "Cargo.toml".to_string(),
            "crates/foo/src/lib.rs".to_string(),
        ],
    };
    let json = serde_json::to_string(&arch).unwrap();
    let deser: Archetype = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.kind, arch.kind);
    assert_eq!(deser.evidence, arch.evidence);
}

// ── 10. No archetype for unrecognised repo ──────────────────────

#[test]
fn no_archetype_for_unrecognised_repo() {
    let export = export_with_paths(&["README.md", "docs/guide.md", "Makefile"]);
    assert!(detect_archetype(&export).is_none());
}

// ── 11. Backslash paths are normalised ──────────────────────────

#[test]
fn backslash_paths_normalised_to_forward_slash() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates\\cli\\src\\main.rs",
        "crates\\cli\\src\\lib.rs",
    ]);
    let arch = detect_archetype(&export).unwrap();
    assert!(
        arch.kind.contains("Rust workspace"),
        "backslash paths should be normalised: {}",
        arch.kind
    );
}

// ── 12. Evidence always contains at least one entry ─────────────

#[test]
fn evidence_always_non_empty() {
    let shapes: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/x/src/lib.rs"],
        vec!["package.json", "next.config.js"],
        vec!["Dockerfile", "k8s/deploy.yaml"],
        vec!["main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
    ];
    for paths in &shapes {
        let export = export_with_paths(paths);
        if let Some(arch) = detect_archetype(&export) {
            assert!(
                !arch.evidence.is_empty(),
                "evidence should not be empty for {:?} → {}",
                paths,
                arch.kind
            );
        }
    }
}

// ── 13. IaC inside terraform/ subdir detected ───────────────────

#[test]
fn iac_terraform_subdir() {
    let export = export_with_paths(&["terraform/modules/vpc/main.tf"]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Infrastructure as code");
}

// ── 14. Next.js with .ts config ─────────────────────────────────

#[test]
fn nextjs_ts_config() {
    let export = export_with_paths(&["package.json", "next.config.ts"]);
    let arch = detect_archetype(&export).unwrap();
    assert_eq!(arch.kind, "Next.js app");
}
