//! W75 security & identity tests for archetype inference.
//!
//! Focuses on:
//! - Archetype detection for various project types
//! - Priority chain: Rust workspace > Next.js > Containerized > IaC > Python > Node
//! - Evidence content verification
//! - Edge cases: child rows, empty exports, backslash paths

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

fn export_with_paths(paths: &[&str]) -> ExportData {
    ExportData {
        rows: paths.iter().map(|p| parent_row(p)).collect(),
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ===========================================================================
// 1. Rust workspace (library) detected
// ===========================================================================

#[test]
fn rust_workspace_library_detected() {
    let export = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(!a.kind.contains("CLI"));
}

// ===========================================================================
// 2. Rust workspace (CLI) detected via src/main.rs
// ===========================================================================

#[test]
fn rust_workspace_cli_detected_via_main_rs() {
    let export = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs", "src/main.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

// ===========================================================================
// 3. Next.js app detected
// ===========================================================================

#[test]
fn nextjs_app_detected() {
    let export = export_with_paths(&["package.json", "next.config.js", "pages/index.tsx"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// 4. Containerized service detected
// ===========================================================================

#[test]
fn containerized_service_detected() {
    let export = export_with_paths(&["Dockerfile", "k8s/deployment.yaml", "src/main.go"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

// ===========================================================================
// 5. Infrastructure as code detected
// ===========================================================================

#[test]
fn iac_detected_from_tf_file() {
    let export = export_with_paths(&["main.tf", "variables.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ===========================================================================
// 6. Python package detected
// ===========================================================================

#[test]
fn python_package_detected() {
    let export = export_with_paths(&["pyproject.toml", "src/app.py"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

// ===========================================================================
// 7. Node package detected (fallback)
// ===========================================================================

#[test]
fn node_package_detected_as_fallback() {
    let export = export_with_paths(&["package.json", "src/index.js"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
    assert_eq!(a.evidence, vec!["package.json".to_string()]);
}

// ===========================================================================
// 8. No archetype for unrecognized project
// ===========================================================================

#[test]
fn no_archetype_for_unrecognized_structure() {
    let export = export_with_paths(&["README.md", "docs/guide.md", "Makefile"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 9. Priority: Rust workspace beats everything
// ===========================================================================

#[test]
fn rust_workspace_beats_all_others() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "package.json",
        "next.config.js",
        "Dockerfile",
        "k8s/pod.yaml",
        "pyproject.toml",
        "main.tf",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.kind.starts_with("Rust workspace"),
        "Rust workspace should take priority: {}",
        a.kind
    );
}

// ===========================================================================
// 10. Child rows do not trigger archetype
// ===========================================================================

#[test]
fn child_rows_ignored_for_detection() {
    let rows = vec![
        FileRow {
            path: "Cargo.toml".to_string(),
            module: "(root)".to_string(),
            lang: "TOML".to_string(),
            kind: FileKind::Child,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        },
        FileRow {
            path: "crates/core/src/lib.rs".to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        },
    ];
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    assert!(
        detect_archetype(&export).is_none(),
        "child rows should not trigger archetype detection"
    );
}
