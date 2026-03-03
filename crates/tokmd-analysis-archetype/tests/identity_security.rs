//! Extended identity & security tests for archetype inference.
//!
//! Covers gaps not exercised by existing unit/bdd/property suites:
//! - Evidence content verification for IaC, Python, containerized archetypes
//! - Deeply nested workspace directory detection
//! - Multiple next.config variants in same repo
//! - Rust workspace with both crates/ and packages/
//! - Confidence in priority chain with more complex layouts
//! - Archetype stability across repeated detections with varied inputs

use tokmd_analysis_archetype::detect_archetype;
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

fn export_with_paths(paths: &[&str]) -> ExportData {
    ExportData {
        rows: paths.iter().map(|p| parent_row(p)).collect(),
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ===========================================================================
// 1. Evidence content for Infrastructure as Code
// ===========================================================================

#[test]
fn iac_evidence_contains_terraform_marker() {
    let export = export_with_paths(&["terraform/main.tf", "terraform/variables.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
    assert!(!a.evidence.is_empty(), "IaC evidence should not be empty");
}

#[test]
fn iac_with_root_tf_file_detected() {
    let export = export_with_paths(&["variables.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ===========================================================================
// 2. Evidence content for Python package
// ===========================================================================

#[test]
fn python_package_evidence_always_contains_pyproject() {
    let export = export_with_paths(&["pyproject.toml", "src/main.py", "tests/test_foo.py"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
    assert!(
        a.evidence.contains(&"pyproject.toml".to_string()),
        "Python package evidence must include pyproject.toml: {:?}",
        a.evidence
    );
}

// ===========================================================================
// 3. Evidence for containerized service includes Dockerfile
// ===========================================================================

#[test]
fn containerized_service_evidence_always_has_dockerfile() {
    let export = export_with_paths(&["Dockerfile", "kubernetes/deployment.yaml", "src/main.go"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
    assert!(
        a.evidence.contains(&"Dockerfile".to_string()),
        "Containerized service evidence must include Dockerfile: {:?}",
        a.evidence
    );
}

// ===========================================================================
// 4. Deeply nested workspace directories
// ===========================================================================

#[test]
fn deeply_nested_crates_dir_detects_rust_workspace() {
    let export = export_with_paths(&["Cargo.toml", "crates/nested/deep/inner/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.kind.starts_with("Rust workspace"),
        "deeply nested crates/ should still trigger Rust workspace: {}",
        a.kind
    );
}

#[test]
fn deeply_nested_packages_dir_detects_rust_workspace() {
    let export = export_with_paths(&["Cargo.toml", "packages/deep/nested/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.kind.starts_with("Rust workspace"),
        "deeply nested packages/ should trigger Rust workspace: {}",
        a.kind
    );
}

// ===========================================================================
// 5. Rust workspace with both crates/ AND packages/
// ===========================================================================

#[test]
fn rust_workspace_with_both_crates_and_packages() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "packages/ui/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
    // Evidence should include at least one workspace directory path
    assert!(
        a.evidence
            .iter()
            .any(|e| e.starts_with("crates/") || e.starts_with("packages/")),
        "evidence should include workspace dir: {:?}",
        a.evidence
    );
}

// ===========================================================================
// 6. Multiple next.config variants — still detects Next.js
// ===========================================================================

#[test]
fn multiple_next_config_files_still_detects_nextjs() {
    let export = export_with_paths(&[
        "package.json",
        "next.config.js",
        "next.config.mjs",
        "next.config.ts",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// 7. Priority: containerized + IaC + Python + Node — containerized wins
// ===========================================================================

#[test]
fn containerized_beats_python_and_node() {
    let export = export_with_paths(&[
        "Dockerfile",
        "k8s/deploy.yaml",
        "pyproject.toml",
        "package.json",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(
        a.kind, "Containerized service",
        "Containerized service should beat Python and Node"
    );
}

// ===========================================================================
// 8. Single-file repos
// ===========================================================================

#[test]
fn single_readme_file_no_archetype() {
    let export = export_with_paths(&["README.md"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn single_source_file_no_archetype() {
    let export = export_with_paths(&["main.rs"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn single_cargo_toml_no_archetype() {
    // Cargo.toml alone without workspace dir is not a Rust workspace
    let export = export_with_paths(&["Cargo.toml"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn single_dockerfile_no_archetype() {
    let export = export_with_paths(&["Dockerfile"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 9. Deterministic detection across 10 repeated calls
// ===========================================================================

#[test]
fn detection_is_deterministic_across_many_calls() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/cli/src/main.rs",
        "package.json",
        "Dockerfile",
    ]);

    let baseline = detect_archetype(&export).unwrap();
    for _ in 0..10 {
        let result = detect_archetype(&export).unwrap();
        assert_eq!(result.kind, baseline.kind);
        assert_eq!(result.evidence, baseline.evidence);
    }
}

// ===========================================================================
// 10. Evidence paths are always forward-slash normalized
// ===========================================================================

#[test]
fn evidence_paths_never_contain_backslashes_for_all_archetypes() {
    let test_cases: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/a/src/lib.rs"],
        vec!["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"],
        vec!["package.json", "next.config.js"],
        vec!["package.json", "next.config.mjs"],
        vec!["package.json", "next.config.ts"],
        vec!["Dockerfile", "k8s/pod.yaml"],
        vec!["Dockerfile", "kubernetes/deploy.yaml"],
        vec!["main.tf"],
        vec!["terraform/main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
    ];

    for paths in &test_cases {
        let export = export_with_paths(paths);
        if let Some(a) = detect_archetype(&export) {
            for ev in &a.evidence {
                assert!(
                    !ev.contains('\\'),
                    "evidence '{}' contains backslash for kind='{}' with paths={:?}",
                    ev,
                    a.kind,
                    paths
                );
            }
        }
    }
}

// ===========================================================================
// 11. Archetype JSON serialization round-trip
// ===========================================================================

#[test]
fn archetype_json_round_trip_preserves_all_fields() {
    let export = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs", "src/main.rs"]);
    let original = detect_archetype(&export).unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: tokmd_analysis_types::Archetype = serde_json::from_str(&json).unwrap();

    assert_eq!(original.kind, deserialized.kind);
    assert_eq!(original.evidence, deserialized.evidence);
}

// ===========================================================================
// 12. Next.js with nested config in monorepo
// ===========================================================================

#[test]
fn nextjs_monorepo_with_nested_configs() {
    let export = export_with_paths(&[
        "package.json",
        "apps/web/next.config.js",
        "apps/docs/next.config.mjs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// 13. Rust workspace CLI detection via deeply nested bin directory
// ===========================================================================

#[test]
fn rust_workspace_cli_via_nested_bin_directory() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/tool/src/lib.rs",
        "crates/tool/src/bin/my-tool.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

// ===========================================================================
// 14. Any .tf file triggers IaC detection
// ===========================================================================

#[test]
fn tf_extension_always_triggers_iac() {
    let export = export_with_paths(&["modules/network.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ===========================================================================
// 15. Mixed FileKind: Parent markers with Child noise
// ===========================================================================

#[test]
fn parent_markers_with_child_noise_still_detected() {
    let rows = vec![
        parent_row("Cargo.toml"),
        parent_row("crates/core/src/lib.rs"),
        FileRow {
            path: "random/noise.txt".to_string(),
            module: "(root)".to_string(),
            lang: "Text".to_string(),
            kind: FileKind::Child,
            code: 0,
            comments: 0,
            blanks: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        },
        FileRow {
            path: "more/child/noise.py".to_string(),
            module: "(root)".to_string(),
            lang: "Python".to_string(),
            kind: FileKind::Child,
            code: 0,
            comments: 0,
            blanks: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        },
    ];
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.kind.starts_with("Rust workspace"),
        "Child noise should not affect archetype detection: {}",
        a.kind
    );
}

// ===========================================================================
// 16. Empty export yields None
// ===========================================================================

#[test]
fn empty_export_yields_no_archetype() {
    let export = export_with_paths(&[]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 17. Rust workspace takes priority over containerized
// ===========================================================================

#[test]
fn rust_workspace_takes_priority_over_containerized() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "Dockerfile",
        "k8s/deploy.yaml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.kind.starts_with("Rust workspace"),
        "Rust workspace should beat containerized: {}",
        a.kind
    );
}

// ===========================================================================
// 18. Next.js takes priority over containerized
// ===========================================================================

#[test]
fn nextjs_takes_priority_over_containerized() {
    let export = export_with_paths(&[
        "package.json",
        "next.config.js",
        "Dockerfile",
        "k8s/deploy.yaml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// 19. IaC takes priority over Python
// ===========================================================================

#[test]
fn iac_takes_priority_over_python() {
    let export = export_with_paths(&["terraform/main.tf", "pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ===========================================================================
// 20. Node package evidence contains package.json
// ===========================================================================

#[test]
fn node_package_evidence_contains_package_json() {
    let export = export_with_paths(&["package.json", "src/index.js"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
    assert!(
        a.evidence.contains(&"package.json".to_string()),
        "Node package evidence must include package.json: {:?}",
        a.evidence
    );
}
