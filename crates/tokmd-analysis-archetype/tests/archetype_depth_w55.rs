//! Wave-55 depth tests for `tokmd-analysis-archetype`.
//!
//! Targets gaps not covered by existing suites:
//! - Archetype detection for ambiguous repo layouts
//! - Monorepo patterns (multi-framework)
//! - Confidence in evidence correctness
//! - Empty / minimal repos
//! - Deterministic classification across many iterations
//! - Priority chain exhaustive combinatorics
//! - Mixed FileKind scenarios
//! - ChildIncludeMode variations

use tokmd_analysis_archetype::detect_archetype;
use tokmd_analysis_types::Archetype;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────────

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

// =============================================================================
// 1. Archetype detection for different repo shapes
// =============================================================================

#[test]
fn rust_workspace_library_with_multiple_crates() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "crates/utils/src/lib.rs",
        "crates/macros/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(!a.kind.contains("CLI"));
}

#[test]
fn rust_workspace_cli_with_main_in_nested_crate() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/cli/src/main.rs",
        "crates/lib/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn nextjs_app_with_app_directory() {
    let export = export_with_paths(&[
        "package.json",
        "next.config.js",
        "app/page.tsx",
        "app/layout.tsx",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn containerized_service_with_k8s_manifests() {
    let export = export_with_paths(&[
        "Dockerfile",
        "k8s/deployment.yaml",
        "k8s/service.yaml",
        "k8s/ingress.yaml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn iac_with_terraform_modules() {
    let export = export_with_paths(&[
        "terraform/main.tf",
        "terraform/variables.tf",
        "terraform/outputs.tf",
        "terraform/modules/vpc/main.tf",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn python_package_with_tests_and_docs() {
    let export = export_with_paths(&[
        "pyproject.toml",
        "src/mylib/__init__.py",
        "tests/test_main.py",
        "docs/conf.py",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

#[test]
fn node_package_with_src_and_tests() {
    let export = export_with_paths(&[
        "package.json",
        "src/index.ts",
        "src/utils.ts",
        "tests/index.test.ts",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
}

// =============================================================================
// 2. Ambiguous repos — priority determines winner
// =============================================================================

#[test]
fn all_six_archetypes_present_rust_wins() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "src/main.rs",
        "package.json",
        "next.config.js",
        "Dockerfile",
        "k8s/deploy.yaml",
        "main.tf",
        "pyproject.toml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn five_archetypes_no_rust_nextjs_wins() {
    let export = export_with_paths(&[
        "package.json",
        "next.config.js",
        "Dockerfile",
        "k8s/deploy.yaml",
        "main.tf",
        "pyproject.toml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn four_archetypes_no_rust_no_nextjs_containerized_wins() {
    let export = export_with_paths(&[
        "Dockerfile",
        "kubernetes/deploy.yaml",
        "main.tf",
        "pyproject.toml",
        "package.json",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn three_archetypes_iac_python_node_iac_wins() {
    let export = export_with_paths(&["main.tf", "pyproject.toml", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn two_archetypes_python_node_python_wins() {
    let export = export_with_paths(&["pyproject.toml", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

// =============================================================================
// 3. Confidence scoring (evidence correctness)
// =============================================================================

#[test]
fn rust_workspace_evidence_always_has_cargo_toml_first() {
    let export = export_with_paths(&["Cargo.toml", "crates/a/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence[0], "Cargo.toml");
}

#[test]
fn rust_workspace_evidence_second_item_is_workspace_dir() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/alpha/src/lib.rs",
        "crates/beta/src/lib.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 2);
    assert!(a.evidence[1].starts_with("crates/") || a.evidence[1].starts_with("packages/"));
}

#[test]
fn nextjs_evidence_has_package_json_and_config() {
    let export = export_with_paths(&["package.json", "next.config.mjs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 2);
    assert_eq!(a.evidence[0], "package.json");
    assert!(a.evidence[1].contains("next.config"));
}

#[test]
fn containerized_evidence_is_singleton_dockerfile() {
    let export = export_with_paths(&["Dockerfile", "k8s/deploy.yaml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 1);
    assert_eq!(a.evidence[0], "Dockerfile");
}

#[test]
fn iac_evidence_is_singleton_terraform_slash() {
    let export = export_with_paths(&["terraform/main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 1);
    assert_eq!(a.evidence[0], "terraform/");
}

#[test]
fn python_evidence_is_singleton_pyproject_toml() {
    let export = export_with_paths(&["pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 1);
    assert_eq!(a.evidence[0], "pyproject.toml");
}

#[test]
fn node_evidence_is_singleton_package_json() {
    let export = export_with_paths(&["package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.evidence.len(), 1);
    assert_eq!(a.evidence[0], "package.json");
}

// =============================================================================
// 4. Edge cases: empty repos, minimal repos
// =============================================================================

#[test]
fn empty_rows_returns_none() {
    let export = export_with_paths(&[]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn single_readme_returns_none() {
    let export = export_with_paths(&["README.md"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn single_makefile_returns_none() {
    let export = export_with_paths(&["Makefile"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn only_source_files_without_markers_returns_none() {
    let export = export_with_paths(&["src/main.rs", "src/lib.rs", "src/utils.rs"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn cargo_toml_without_workspace_dir_returns_none() {
    let export = export_with_paths(&["Cargo.toml", "src/lib.rs"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn dockerfile_without_k8s_returns_none() {
    let export = export_with_paths(&["Dockerfile", "src/main.go"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn next_config_without_package_json_returns_none() {
    let export = export_with_paths(&["next.config.js", "pages/index.tsx"]);
    assert!(detect_archetype(&export).is_none());
}

// =============================================================================
// 5. Monorepo patterns
// =============================================================================

#[test]
fn rust_monorepo_with_many_crates() {
    let mut paths = vec!["Cargo.toml"];
    let generated: Vec<String> = (0..30)
        .map(|i| format!("crates/crate_{i}/src/lib.rs"))
        .collect();
    let refs: Vec<&str> = generated.iter().map(String::as_str).collect();
    paths.extend(refs);
    let export = export_with_paths(&paths);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
}

#[test]
fn rust_monorepo_with_one_cli_crate_among_many() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/lib_a/src/lib.rs",
        "crates/lib_b/src/lib.rs",
        "crates/cli/src/bin/tool.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

// =============================================================================
// 6. Deterministic classification
// =============================================================================

#[test]
fn detection_deterministic_over_50_iterations() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "src/main.rs",
        "package.json",
    ]);
    let baseline = detect_archetype(&export).unwrap();
    for _ in 0..50 {
        let result = detect_archetype(&export).unwrap();
        assert_eq!(result.kind, baseline.kind);
        assert_eq!(result.evidence, baseline.evidence);
    }
}

#[test]
fn detection_deterministic_for_each_archetype_kind() {
    let cases: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/a/src/lib.rs"],
        vec!["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"],
        vec!["package.json", "next.config.js"],
        vec!["Dockerfile", "k8s/deploy.yaml"],
        vec!["main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
    ];
    for paths in &cases {
        let export = export_with_paths(paths);
        let baseline = detect_archetype(&export);
        for _ in 0..10 {
            let result = detect_archetype(&export);
            assert_eq!(
                baseline.as_ref().map(|a| &a.kind),
                result.as_ref().map(|a| &a.kind),
                "non-deterministic for {:?}",
                paths
            );
        }
    }
}

#[test]
fn archetype_serde_roundtrip_all_kinds() {
    let kinds = [
        "Rust workspace",
        "Rust workspace (CLI)",
        "Next.js app",
        "Containerized service",
        "Infrastructure as code",
        "Python package",
        "Node package",
    ];
    for kind in &kinds {
        let a = Archetype {
            kind: kind.to_string(),
            evidence: vec!["test.txt".to_string()],
        };
        let json = serde_json::to_string(&a).unwrap();
        let b: Archetype = serde_json::from_str(&json).unwrap();
        assert_eq!(a.kind, b.kind);
        assert_eq!(a.evidence, b.evidence);
    }
}

// =============================================================================
// 7. Mixed FileKind scenarios
// =============================================================================

#[test]
fn child_rows_are_ignored_for_detection() {
    let rows = vec![
        child_row("Cargo.toml"),
        child_row("crates/core/src/lib.rs"),
        child_row("package.json"),
    ];
    let export = export_from_rows(rows);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn parent_cargo_with_child_crates_no_workspace() {
    let rows = vec![
        parent_row("Cargo.toml"),
        child_row("crates/core/src/lib.rs"),
    ];
    let export = export_from_rows(rows);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn parent_markers_with_interleaved_child_noise() {
    let rows = vec![
        parent_row("Cargo.toml"),
        child_row("noise1.txt"),
        parent_row("crates/core/src/lib.rs"),
        child_row("noise2.txt"),
        parent_row("src/main.rs"),
        child_row("noise3.txt"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn parent_dockerfile_with_child_k8s_no_containerized() {
    let rows = vec![parent_row("Dockerfile"), child_row("k8s/deployment.yaml")];
    let export = export_from_rows(rows);
    assert!(detect_archetype(&export).is_none());
}

// =============================================================================
// 8. Backslash path normalization
// =============================================================================

#[test]
fn backslash_crates_path_detected_as_workspace() {
    let rows = vec![
        parent_row("Cargo.toml"),
        parent_row("crates\\core\\src\\lib.rs"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn backslash_k8s_path_detected() {
    let rows = vec![parent_row("Dockerfile"), parent_row("k8s\\deployment.yaml")];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn backslash_next_config_path_detected() {
    let rows = vec![
        parent_row("package.json"),
        parent_row("apps\\web\\next.config.js"),
    ];
    let export = export_from_rows(rows);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

// =============================================================================
// 9. ChildIncludeMode variations
// =============================================================================

#[test]
fn parents_only_mode_still_detects_archetype() {
    let export = ExportData {
        rows: vec![parent_row("pyproject.toml"), parent_row("src/main.py")],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    };
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

#[test]
fn separate_mode_still_detects_archetype() {
    let export = ExportData {
        rows: vec![parent_row("package.json")],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
}

// =============================================================================
// 10. Evidence paths never contain backslashes
// =============================================================================

#[test]
fn evidence_never_has_backslashes_for_all_archetypes() {
    let cases: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/a/src/lib.rs"],
        vec!["package.json", "next.config.js"],
        vec!["Dockerfile", "k8s/pod.yaml"],
        vec!["main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
    ];
    for paths in &cases {
        let export = export_with_paths(paths);
        if let Some(a) = detect_archetype(&export) {
            for ev in &a.evidence {
                assert!(
                    !ev.contains('\\'),
                    "evidence '{}' has backslash for kind={}, paths={:?}",
                    ev,
                    a.kind,
                    paths
                );
            }
        }
    }
}

// =============================================================================
// 11. Known archetype kinds exhaustive check
// =============================================================================

#[test]
fn every_detected_kind_is_in_known_set() {
    let known = [
        "Rust workspace",
        "Rust workspace (CLI)",
        "Next.js app",
        "Containerized service",
        "Infrastructure as code",
        "Python package",
        "Node package",
    ];
    let cases: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/a/src/lib.rs"],
        vec!["Cargo.toml", "crates/a/src/lib.rs", "src/main.rs"],
        vec!["package.json", "next.config.js"],
        vec!["package.json", "next.config.mjs"],
        vec!["package.json", "next.config.ts"],
        vec!["Dockerfile", "k8s/deploy.yaml"],
        vec!["Dockerfile", "kubernetes/deploy.yaml"],
        vec!["main.tf"],
        vec!["terraform/main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
    ];
    for paths in &cases {
        let export = export_with_paths(paths);
        let a = detect_archetype(&export).unwrap();
        assert!(
            known.contains(&a.kind.as_str()),
            "unexpected kind '{}' for paths {:?}",
            a.kind,
            paths
        );
    }
}

// =============================================================================
// 12. Archetype JSON shape
// =============================================================================

#[test]
fn archetype_json_has_exactly_two_keys() {
    let a = Archetype {
        kind: "Test".to_string(),
        evidence: vec!["file.txt".to_string()],
    };
    let v: serde_json::Value = serde_json::to_value(&a).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert!(obj.contains_key("kind"));
    assert!(obj.contains_key("evidence"));
}

#[test]
fn archetype_evidence_serializes_as_array() {
    let a = Archetype {
        kind: "Test".to_string(),
        evidence: vec!["a.txt".to_string(), "b.txt".to_string()],
    };
    let v: serde_json::Value = serde_json::to_value(&a).unwrap();
    assert!(v["evidence"].is_array());
    assert_eq!(v["evidence"].as_array().unwrap().len(), 2);
}

#[test]
fn archetype_empty_evidence_serializes_as_empty_array() {
    let a = Archetype {
        kind: "Custom".to_string(),
        evidence: vec![],
    };
    let v: serde_json::Value = serde_json::to_value(&a).unwrap();
    assert!(v["evidence"].as_array().unwrap().is_empty());
}

// =============================================================================
// 13. Large file sets
// =============================================================================

#[test]
fn large_repo_with_rust_markers_detected() {
    let mut paths: Vec<String> = (0..500).map(|i| format!("src/gen/file_{i}.rs")).collect();
    paths.push("Cargo.toml".to_string());
    paths.push("crates/core/src/lib.rs".to_string());

    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    let export = export_with_paths(&refs);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn large_repo_without_markers_returns_none() {
    let paths: Vec<String> = (0..500).map(|i| format!("src/gen/file_{i}.rs")).collect();
    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    let export = export_with_paths(&refs);
    assert!(detect_archetype(&export).is_none());
}
