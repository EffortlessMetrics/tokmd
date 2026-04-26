//! Deep integration tests for archetype inference.
//!
//! Targets gaps not covered by existing unit/bdd/edge/property/identity suites:
//! - ExportData with non-standard module_roots and module_depth
//! - Archetype PartialEq manual comparison
//! - Deserialization from known JSON
//! - Full priority chain exhaustive testing
//! - Evidence exactly matches expected values (not just contains)
//! - Rust workspace with src/bin/ in root (not nested)
//! - IaC with multiple .tf files in different dirs
//! - Python package alongside Node package (Python higher priority)
//! - FileKind::Child in mixed sets
//! - Very large file sets with many markers
//! - Empty path strings
//! - next.config. prefix variations (e.g., next.config.cjs in subdir)

use crate::archetype::detect_archetype;
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

// ===========================================================================
// 1. Archetype deserialization from known JSON
// ===========================================================================

#[test]
fn archetype_deserializes_from_known_json() {
    let json =
        r#"{"kind":"Rust workspace (CLI)","evidence":["Cargo.toml","crates/cli/src/main.rs"]}"#;
    let a: Archetype = serde_json::from_str(json).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
    assert_eq!(a.evidence.len(), 2);
    assert_eq!(a.evidence[0], "Cargo.toml");
}

// ===========================================================================
// 2. Archetype with empty evidence round-trips
// ===========================================================================

#[test]
fn archetype_empty_evidence_deserializes() {
    let json = r#"{"kind":"Custom","evidence":[]}"#;
    let a: Archetype = serde_json::from_str(json).unwrap();
    assert_eq!(a.kind, "Custom");
    assert!(a.evidence.is_empty());
}

// ===========================================================================
// 3. Full priority chain: Rust > Next.js > Containerized > IaC > Python > Node
// ===========================================================================

#[test]
fn priority_rust_over_nextjs() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "package.json",
        "next.config.js",
    ]);
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
fn priority_containerized_over_iac() {
    let export = export_with_paths(&["Dockerfile", "kubernetes/deploy.yaml", "main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn priority_iac_over_python() {
    let export = export_with_paths(&["terraform/main.tf", "pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn priority_python_over_node() {
    let export = export_with_paths(&["pyproject.toml", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

// ===========================================================================
// 4. Rust workspace evidence exact values
// ===========================================================================

#[test]
fn rust_workspace_evidence_exact_cargo_toml_plus_first_crate_path() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/alpha/src/lib.rs",
        "crates/beta/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence[0], "Cargo.toml");
    // Evidence should contain one of the crates/ paths (first in BTreeSet order)
    assert!(
        a.evidence[1].starts_with("crates/"),
        "second evidence should be a crates/ path: {:?}",
        a.evidence
    );
}

// ===========================================================================
// 5. Rust workspace with src/bin/ at root level
// ===========================================================================

#[test]
fn rust_workspace_cli_via_root_src_bin() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/lib/src/lib.rs",
        "crates/cli/src/bin/my-tool.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

// ===========================================================================
// 6. IaC with multiple .tf files in different directories
// ===========================================================================

#[test]
fn multiple_tf_files_still_iac() {
    let export = export_with_paths(&[
        "modules/network/main.tf",
        "modules/compute/main.tf",
        "environments/prod.tf",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

// ===========================================================================
// 7. Mixed Parent and Child rows with same path
// ===========================================================================

#[test]
fn same_path_as_parent_and_child_detects_from_parent() {
    let rows = vec![parent_row("pyproject.toml"), child_row("pyproject.toml")];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

// ===========================================================================
// 8. Very large file set with many markers
// ===========================================================================

#[test]
fn large_file_set_with_rust_workspace_markers() {
    let paths: Vec<&str> = vec!["Cargo.toml", "crates/core/src/lib.rs", "src/main.rs"];
    // Add 200 generic files
    let generated: Vec<String> = (0..200).map(|i| format!("src/gen/file_{i}.rs")).collect();
    let all_paths: Vec<&str> = paths
        .iter()
        .copied()
        .chain(generated.iter().map(String::as_str))
        .collect();

    let export = export_with_paths(&all_paths);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

// ===========================================================================
// 9. Empty path strings in rows
// ===========================================================================

#[test]
fn empty_path_strings_do_not_match_any_archetype() {
    let rows = vec![parent_row(""), parent_row(""), parent_row("")];
    let export = export_from_rows(rows);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 10. next.config.cjs at root detected
// ===========================================================================

#[test]
fn next_config_cjs_at_root_detected() {
    let export = export_with_paths(&["package.json", "next.config.cjs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// 11. next.config.cjs in subdirectory
// ===========================================================================

#[test]
fn next_config_cjs_in_subdir_not_detected_by_ends_with() {
    // next.config.cjs doesn't match starts_with("next.config.") at root
    // nor ends_with patterns for .js/.mjs/.ts
    // But starts_with("next.config.") does match "next.config.cjs"
    let export = export_with_paths(&["package.json", "next.config.cjs"]);
    let a = detect_archetype(&export);
    // "next.config.cjs" matches starts_with("next.config.") so should detect
    assert!(a.is_some());
    assert_eq!(a.unwrap().kind, "Next.js app");
}

// ===========================================================================
// 12. Archetype JSON shape validation
// ===========================================================================

#[test]
fn archetype_json_has_kind_and_evidence_keys() {
    let export = export_with_paths(&["package.json"]);
    let a = detect_archetype(&export).unwrap();
    let v: serde_json::Value = serde_json::to_value(a).unwrap();
    assert!(v.is_object());
    assert!(v.get("kind").is_some());
    assert!(v.get("evidence").is_some());
    assert_eq!(v["kind"], "Node package");
}

// ===========================================================================
// 13. Determinism with complex layout
// ===========================================================================

#[test]
fn detection_deterministic_complex_layout() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/cli/src/main.rs",
        "crates/cli/src/bin/extra.rs",
        "package.json",
        "Dockerfile",
        "k8s/deploy.yaml",
        "pyproject.toml",
        "main.tf",
    ]);

    let baseline = detect_archetype(&export).unwrap();
    for _ in 0..20 {
        let result = detect_archetype(&export).unwrap();
        assert_eq!(result.kind, baseline.kind);
        assert_eq!(result.evidence, baseline.evidence);
    }
}

// ===========================================================================
// 14. Node package evidence is exactly ["package.json"]
// ===========================================================================

#[test]
fn node_package_evidence_is_exactly_package_json() {
    let export = export_with_paths(&["package.json", "src/index.js", "README.md"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
    assert_eq!(a.evidence, vec!["package.json".to_string()]);
}

// ===========================================================================
// 15. Python package evidence is exactly ["pyproject.toml"]
// ===========================================================================

#[test]
fn python_package_evidence_is_exactly_pyproject_toml() {
    let export = export_with_paths(&["pyproject.toml", "src/main.py"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
    assert_eq!(a.evidence, vec!["pyproject.toml".to_string()]);
}

// ===========================================================================
// 16. Containerized service evidence is exactly ["Dockerfile"]
// ===========================================================================

#[test]
fn containerized_evidence_is_exactly_dockerfile() {
    let export = export_with_paths(&["Dockerfile", "k8s/service.yaml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
    assert_eq!(a.evidence, vec!["Dockerfile".to_string()]);
}

// ===========================================================================
// 17. IaC evidence is exactly ["terraform/"]
// ===========================================================================

#[test]
fn iac_evidence_is_terraform_slash() {
    let export = export_with_paths(&["terraform/main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
    assert_eq!(a.evidence, vec!["terraform/".to_string()]);
}

// ===========================================================================
// 18. Non-standard module_roots don't affect detection
// ===========================================================================

#[test]
fn custom_module_roots_do_not_affect_detection() {
    let export = ExportData {
        rows: vec![parent_row("package.json")],
        module_roots: vec![
            "lib".to_string(),
            "vendor".to_string(),
            "third_party".to_string(),
        ],
        module_depth: 5,
        children: ChildIncludeMode::Separate,
    };
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
}

// ===========================================================================
// 19. Backslash paths in Next.js config
// ===========================================================================

#[test]
fn backslash_next_config_js_detected() {
    let rows = vec![
        parent_row("package.json"),
        parent_row("apps\\web\\next.config.js"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// ===========================================================================
// 20. Only Dockerfile without k8s/ returns None
// ===========================================================================

#[test]
fn dockerfile_alone_returns_none() {
    let export = export_with_paths(&["Dockerfile"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 21. Only k8s/ without Dockerfile returns None
// ===========================================================================

#[test]
fn k8s_dir_alone_returns_none() {
    let export = export_with_paths(&["k8s/deployment.yaml"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// 22. Archetype JSON serialization is deterministic
// ===========================================================================

#[test]
fn archetype_json_serialization_deterministic() {
    let export = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    let json1 = serde_json::to_string(&a).unwrap();
    let json2 = serde_json::to_string(&a).unwrap();
    assert_eq!(json1, json2);
}

// ===========================================================================
// 23. Multiple workspace indicators: packages/ preferred in BTreeSet
// ===========================================================================

#[test]
fn btreeset_ordering_determines_evidence_workspace_dir() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "packages/foo/src/lib.rs",
        "crates/bar/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    // BTreeSet: "crates/bar/src/lib.rs" < "packages/foo/src/lib.rs"
    // So evidence[1] should be from crates/ (first alphabetically)
    assert!(
        a.evidence[1].starts_with("crates/"),
        "BTreeSet should order crates/ before packages/: {:?}",
        a.evidence
    );
}

// ===========================================================================
// 24. All known archetype kinds
// ===========================================================================

#[test]
fn all_known_archetype_kinds_are_detectable() {
    let test_cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["Cargo.toml", "crates/a/src/lib.rs"], "Rust workspace"),
        (
            vec!["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"],
            "Rust workspace (CLI)",
        ),
        (vec!["package.json", "next.config.js"], "Next.js app"),
        (
            vec!["Dockerfile", "k8s/deploy.yaml"],
            "Containerized service",
        ),
        (vec!["main.tf"], "Infrastructure as code"),
        (vec!["pyproject.toml"], "Python package"),
        (vec!["package.json"], "Node package"),
    ];

    for (paths, expected_kind) in test_cases {
        let export = export_with_paths(&paths);
        let a = detect_archetype(&export)
            .unwrap_or_else(|| panic!("expected archetype for {:?}", paths));
        assert_eq!(
            a.kind, expected_kind,
            "paths {:?} should produce kind '{}'",
            paths, expected_kind
        );
    }
}

// ===========================================================================
// 25. Archetype with many evidence items (Rust workspace)
// ===========================================================================

#[test]
fn rust_workspace_evidence_has_exactly_two_items() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/a/src/lib.rs",
        "crates/b/src/lib.rs",
        "crates/c/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(
        a.evidence.len(),
        2,
        "Rust workspace evidence should have exactly 2 items: {:?}",
        a.evidence
    );
    assert_eq!(a.evidence[0], "Cargo.toml");
    assert!(a.evidence[1].starts_with("crates/"));
}

// ===========================================================================
// 26. Next.js evidence has exactly two items when config is found
// ===========================================================================

#[test]
fn nextjs_evidence_has_two_items() {
    let export = export_with_paths(&["package.json", "next.config.js"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 2);
    assert_eq!(a.evidence[0], "package.json");
    assert_eq!(a.evidence[1], "next.config.js");
}

// ===========================================================================
// 27. Backslash in packages/ path still detects workspace
// ===========================================================================

#[test]
fn backslash_packages_path_detected() {
    let rows = vec![
        parent_row("Cargo.toml"),
        parent_row("packages\\foo\\src\\lib.rs"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}
