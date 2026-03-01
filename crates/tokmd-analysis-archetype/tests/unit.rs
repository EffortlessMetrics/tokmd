//! Unit tests for `tokmd-analysis-archetype`.
//!
//! Covers trait implementations, serde round-trips, edge cases, and
//! archetype detection nuances not exercised by the BDD or property suites.

use tokmd_analysis_archetype::detect_archetype;
use tokmd_analysis_types::Archetype;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn export_from_rows(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn export_with_paths(paths: &[&str]) -> ExportData {
    export_from_rows(paths.iter().map(|p| parent_row(p)).collect())
}

// ===========================================================================
// 1. Archetype trait implementations: Debug, Clone, Serialize, Deserialize
// ===========================================================================

#[test]
fn archetype_debug_impl() {
    let a = Archetype {
        kind: "Test".to_string(),
        evidence: vec!["file.rs".to_string()],
    };
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("Test"), "Debug output must contain kind");
    assert!(
        dbg.contains("file.rs"),
        "Debug output must contain evidence"
    );
}

#[test]
fn archetype_clone_impl() {
    let a = Archetype {
        kind: "Rust workspace".to_string(),
        evidence: vec!["Cargo.toml".to_string()],
    };
    let b = a.clone();
    assert_eq!(a.kind, b.kind);
    assert_eq!(a.evidence, b.evidence);
}

#[test]
fn archetype_serde_round_trip() {
    let a = Archetype {
        kind: "Python package".to_string(),
        evidence: vec!["pyproject.toml".to_string()],
    };
    let json = serde_json::to_string(&a).expect("serialize");
    let b: Archetype = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(a.kind, b.kind);
    assert_eq!(a.evidence, b.evidence);
}

#[test]
fn archetype_serde_preserves_empty_evidence() {
    let a = Archetype {
        kind: "Custom".to_string(),
        evidence: vec![],
    };
    let json = serde_json::to_string(&a).unwrap();
    let b: Archetype = serde_json::from_str(&json).unwrap();
    assert!(b.evidence.is_empty());
}

#[test]
fn archetype_serde_json_shape() {
    let a = Archetype {
        kind: "Node package".to_string(),
        evidence: vec!["package.json".to_string()],
    };
    let v: serde_json::Value = serde_json::to_value(&a).unwrap();
    assert!(v.is_object());
    assert_eq!(v["kind"], "Node package");
    assert!(v["evidence"].is_array());
    assert_eq!(v["evidence"][0], "package.json");
}

// ===========================================================================
// 2. detect_archetype: single-file marker archetypes
// ===========================================================================

#[test]
fn single_tf_file_detects_iac() {
    let export = export_with_paths(&["main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn single_pyproject_detects_python() {
    let export = export_with_paths(&["pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

#[test]
fn single_package_json_detects_node() {
    let export = export_with_paths(&["package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
}

// ===========================================================================
// 3. Edge cases: duplicate paths, large inputs, unusual structures
// ===========================================================================

#[test]
fn duplicate_paths_do_not_affect_detection() {
    let rows = vec![
        parent_row("Cargo.toml"),
        parent_row("Cargo.toml"),
        parent_row("crates/a/src/lib.rs"),
        parent_row("crates/a/src/lib.rs"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
}

#[test]
fn many_random_files_without_markers_returns_none() {
    let paths: Vec<&str> = vec![
        "src/foo.rs",
        "src/bar.rs",
        "lib/baz.py",
        "docs/README.md",
        "Makefile",
        "build.sh",
        "test/test_a.rs",
        "test/test_b.rs",
        "assets/logo.png",
        "config/dev.yaml",
    ];
    let export = export_with_paths(&paths);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn cargo_toml_alone_is_not_rust_workspace() {
    let export = export_with_paths(&["Cargo.toml"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 4. Child-row filtering edge cases
// ===========================================================================

#[test]
fn all_markers_as_child_rows_returns_none() {
    let rows = vec![
        child_row("Cargo.toml"),
        child_row("crates/a/src/lib.rs"),
        child_row("package.json"),
        child_row("next.config.js"),
        child_row("Dockerfile"),
        child_row("k8s/deploy.yaml"),
        child_row("main.tf"),
        child_row("pyproject.toml"),
    ];
    let export = export_from_rows(rows);
    assert!(
        detect_archetype(&export).is_none(),
        "Child rows must never trigger any archetype"
    );
}

#[test]
fn parent_marker_with_child_workspace_dir_no_rust_workspace() {
    // Cargo.toml as Parent but crates/ only as Child → not a workspace
    let rows = vec![
        parent_row("Cargo.toml"),
        child_row("crates/core/src/lib.rs"),
    ];
    let export = export_from_rows(rows);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 5. Backslash normalization
// ===========================================================================

#[test]
fn deeply_nested_backslash_paths_normalized() {
    let rows = vec![
        parent_row("Cargo.toml"),
        parent_row("crates\\deep\\nested\\src\\lib.rs"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn backslash_main_rs_detected_as_cli() {
    let rows = vec![
        parent_row("Cargo.toml"),
        parent_row("crates\\foo\\src\\lib.rs"),
        parent_row("src\\main.rs"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn backslash_k8s_path_detected() {
    let rows = vec![parent_row("Dockerfile"), parent_row("k8s\\deployment.yaml")];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

// ===========================================================================
// 6. next.config. prefix edge case
// ===========================================================================

#[test]
fn next_config_dot_prefix_at_root_detected() {
    // "next.config." as prefix matches `starts_with("next.config.")`
    let export = export_with_paths(&["package.json", "next.config.cjs"]);
    let a = detect_archetype(&export).unwrap();
    // "next.config.cjs" matches starts_with("next.config.") but not the
    // evidence finder's ends_with checks, so evidence only has package.json
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// 7. Evidence content validation
// ===========================================================================

#[test]
fn rust_workspace_evidence_always_includes_cargo_toml() {
    let export = export_with_paths(&["Cargo.toml", "crates/x/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.evidence.contains(&"Cargo.toml".to_string()),
        "evidence: {:?}",
        a.evidence
    );
}

#[test]
fn node_package_evidence_includes_package_json() {
    let export = export_with_paths(&["package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence, vec!["package.json".to_string()]);
}

#[test]
fn containerized_service_evidence_includes_dockerfile() {
    let export = export_with_paths(&["Dockerfile", "k8s/pod.yaml"]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.evidence.contains(&"Dockerfile".to_string()));
}

// ===========================================================================
// 8. module_depth / module_roots do not affect archetype detection
// ===========================================================================

#[test]
fn module_depth_does_not_affect_detection() {
    for depth in [0, 1, 5, 100] {
        let export = ExportData {
            rows: vec![parent_row("pyproject.toml")],
            module_roots: vec![],
            module_depth: depth,
            children: ChildIncludeMode::Separate,
        };
        let a = detect_archetype(&export).unwrap();
        assert_eq!(a.kind, "Python package", "depth={depth}");
    }
}

#[test]
fn children_mode_does_not_affect_detection() {
    for mode in [ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly] {
        let export = ExportData {
            rows: vec![parent_row("package.json")],
            module_roots: vec!["src".to_string()],
            module_depth: 2,
            children: mode,
        };
        let a = detect_archetype(&export).unwrap();
        assert_eq!(a.kind, "Node package", "mode={mode:?}");
    }
}
