//! BDD-style scenario tests for archetype detection.
//!
//! Each test follows: **Given** a file layout → **When** `detect_archetype` runs → **Then** assert kind / evidence.

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
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ===========================================================================
// Scenario: Empty / unrecognized repositories
// ===========================================================================

#[test]
fn given_empty_repo_then_no_archetype() {
    let export = export_with_paths(&[]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn given_only_readme_then_no_archetype() {
    let export = export_with_paths(&["README.md"]);
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn given_generic_source_files_then_no_archetype() {
    let export = export_with_paths(&["src/lib.rs", "src/utils.rs", "Makefile"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// Scenario: Rust workspace detection
// ===========================================================================

#[test]
fn given_cargo_toml_and_crates_dir_then_rust_workspace() {
    let export = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(a.evidence.contains(&"Cargo.toml".to_string()));
}

#[test]
fn given_cargo_toml_and_packages_dir_then_rust_workspace() {
    let export = export_with_paths(&["Cargo.toml", "packages/util/src/lib.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace");
    assert!(a.evidence.iter().any(|e| e.starts_with("packages/")));
}

#[test]
fn given_rust_workspace_with_main_rs_then_cli_variant() {
    let export = export_with_paths(&["Cargo.toml", "crates/foo/src/lib.rs", "src/main.rs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn given_rust_workspace_with_bin_dir_then_cli_variant() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/foo/src/lib.rs",
        "crates/foo/src/bin/runner.rs",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn given_cargo_toml_without_workspace_dir_then_no_rust_workspace() {
    let export = export_with_paths(&["Cargo.toml", "src/lib.rs"]);
    assert!(
        detect_archetype(&export).is_none(),
        "plain Cargo.toml without crates/ or packages/ must not match Rust workspace"
    );
}

// ===========================================================================
// Scenario: Next.js detection
// ===========================================================================

#[test]
fn given_package_json_and_next_config_js_then_nextjs() {
    let export = export_with_paths(&["package.json", "next.config.js"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
    assert!(a.evidence.contains(&"package.json".to_string()));
}

#[test]
fn given_package_json_and_next_config_mjs_then_nextjs() {
    let export = export_with_paths(&["package.json", "next.config.mjs"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn given_package_json_and_next_config_ts_then_nextjs() {
    let export = export_with_paths(&["package.json", "next.config.ts"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn given_nested_next_config_then_nextjs() {
    let export = export_with_paths(&["package.json", "apps/web/next.config.js"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
    assert!(a.evidence.iter().any(|e| e == "apps/web/next.config.js"));
}

#[test]
fn given_next_config_without_package_json_then_no_nextjs() {
    let export = export_with_paths(&["next.config.js"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// Scenario: Containerized service
// ===========================================================================

#[test]
fn given_dockerfile_and_k8s_dir_then_containerized_service() {
    let export = export_with_paths(&["Dockerfile", "k8s/deployment.yaml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
    assert!(a.evidence.contains(&"Dockerfile".to_string()));
}

#[test]
fn given_dockerfile_and_kubernetes_dir_then_containerized_service() {
    let export = export_with_paths(&["Dockerfile", "kubernetes/pod.yaml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn given_dockerfile_without_k8s_then_no_containerized_service() {
    let export = export_with_paths(&["Dockerfile", "src/main.rs"]);
    // Dockerfile alone matches nothing (falls through all checks)
    assert!(detect_archetype(&export).is_none());
}

#[test]
fn given_k8s_without_dockerfile_then_no_containerized_service() {
    let export = export_with_paths(&["k8s/deployment.yaml"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// Scenario: Infrastructure as code
// ===========================================================================

#[test]
fn given_tf_file_then_iac() {
    let export = export_with_paths(&["main.tf", "variables.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn given_terraform_dir_then_iac() {
    let export = export_with_paths(&["terraform/main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn given_no_tf_files_then_no_iac() {
    let export = export_with_paths(&["cloudformation.yaml"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// Scenario: Python package
// ===========================================================================

#[test]
fn given_pyproject_toml_then_python_package() {
    let export = export_with_paths(&["pyproject.toml", "src/main.py"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
    assert!(a.evidence.contains(&"pyproject.toml".to_string()));
}

#[test]
fn given_setup_py_without_pyproject_then_no_python_package() {
    let export = export_with_paths(&["setup.py", "src/main.py"]);
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// Scenario: Node package (fallback)
// ===========================================================================

#[test]
fn given_only_package_json_then_node_package() {
    let export = export_with_paths(&["package.json", "src/index.js"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Node package");
    assert!(a.evidence.contains(&"package.json".to_string()));
}

// ===========================================================================
// Scenario: Priority chain — earlier detectors win
// ===========================================================================

#[test]
fn rust_workspace_beats_node_package() {
    let export = export_with_paths(&["Cargo.toml", "crates/core/src/lib.rs", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.kind.starts_with("Rust workspace"),
        "Rust workspace should take priority over Node package, got: {}",
        a.kind
    );
}

#[test]
fn rust_workspace_beats_containerized_service() {
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/core/src/lib.rs",
        "Dockerfile",
        "k8s/deploy.yaml",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert!(a.kind.starts_with("Rust workspace"));
}

#[test]
fn nextjs_beats_plain_node_package() {
    let export = export_with_paths(&["package.json", "next.config.js", "src/index.ts"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Next.js app");
}

#[test]
fn containerized_beats_iac_when_both_present() {
    let export = export_with_paths(&["Dockerfile", "k8s/pod.yaml", "main.tf"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn iac_beats_python_when_both_present() {
    let export = export_with_paths(&["main.tf", "pyproject.toml"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Infrastructure as code");
}

#[test]
fn python_beats_node_when_both_present() {
    let export = export_with_paths(&["pyproject.toml", "package.json"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
}

// ===========================================================================
// Scenario: Child rows are ignored — only Parent rows matter
// ===========================================================================

#[test]
fn given_child_rows_only_then_no_archetype() {
    let rows = vec![FileRow {
        path: "Cargo.toml".to_string(),
        module: "(root)".to_string(),
        lang: "TOML".to_string(),
        kind: FileKind::Child,
        code: 10,
        comments: 0,
        blanks: 0,
        lines: 10,
        bytes: 100,
        tokens: 20,
    }];
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    assert!(
        detect_archetype(&export).is_none(),
        "Child FileKind rows should be filtered out"
    );
}

#[test]
fn given_mix_of_parent_and_child_rows_then_only_parents_considered() {
    let rows = vec![
        FileRow {
            path: "Cargo.toml".to_string(),
            module: "(root)".to_string(),
            lang: "TOML".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 100,
            tokens: 20,
        },
        // Child row for crates/ — should NOT contribute to workspace detection
        FileRow {
            path: "crates/core/src/lib.rs".to_string(),
            module: "core".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 50,
            comments: 5,
            blanks: 3,
            lines: 58,
            bytes: 500,
            tokens: 100,
        },
    ];
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };
    // Only "Cargo.toml" as Parent, no crates/ Parent → no Rust workspace
    assert!(detect_archetype(&export).is_none());
}

// ===========================================================================
// Scenario: Backslash path normalization
// ===========================================================================

#[test]
fn given_backslash_paths_then_normalized_to_forward_slash() {
    let rows = vec![
        FileRow {
            path: "Cargo.toml".to_string(),
            module: "(root)".to_string(),
            lang: "TOML".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        },
        FileRow {
            path: "crates\\core\\src\\lib.rs".to_string(),
            module: "core".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
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
    let a = detect_archetype(&export).unwrap();
    assert!(
        a.kind.starts_with("Rust workspace"),
        "backslash paths should be normalized; got: {:?}",
        a.kind
    );
}

// ===========================================================================
// Scenario: Evidence invariants
// ===========================================================================

#[test]
fn evidence_is_never_empty_when_archetype_detected() {
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
            assert!(
                !a.evidence.is_empty(),
                "evidence must not be empty for kind={:?} with paths={:?}",
                a.kind,
                paths
            );
        }
    }
}

#[test]
fn kind_is_always_a_known_archetype() {
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
        vec!["Dockerfile", "k8s/pod.yaml"],
        vec!["main.tf"],
        vec!["pyproject.toml"],
        vec!["package.json"],
    ];
    for paths in &cases {
        let export = export_with_paths(paths);
        if let Some(a) = detect_archetype(&export) {
            assert!(
                known.contains(&a.kind.as_str()),
                "unexpected kind {:?} for paths={:?}",
                a.kind,
                paths
            );
        }
    }
}

// ===========================================================================
// Scenario: Determinism
// ===========================================================================

#[test]
fn detect_archetype_is_deterministic() {
    let cases: Vec<Vec<&str>> = vec![
        vec!["Cargo.toml", "crates/a/src/lib.rs"],
        vec!["package.json", "next.config.js"],
        vec!["Dockerfile", "k8s/pod.yaml"],
        vec!["main.tf"],
        vec!["pyproject.toml", "src/main.py"],
        vec!["package.json", "src/index.js"],
        vec![],
    ];
    for paths in &cases {
        let r1 = detect_archetype(&export_with_paths(paths));
        let r2 = detect_archetype(&export_with_paths(paths));
        match (&r1, &r2) {
            (None, None) => {}
            (Some(a1), Some(a2)) => {
                assert_eq!(a1.kind, a2.kind, "kind mismatch for paths={:?}", paths);
                assert_eq!(
                    a1.evidence, a2.evidence,
                    "evidence mismatch for paths={:?}",
                    paths
                );
            }
            _ => panic!(
                "determinism violated for paths={:?}: {:?} vs {:?}",
                paths, r1, r2
            ),
        }
    }
}

// ===========================================================================
// Scenario: Python with setup.py and/or pyproject.toml
// ===========================================================================

#[test]
fn given_setup_py_alone_then_no_python_archetype() {
    let export = export_with_paths(&["setup.py", "mypackage/__init__.py"]);
    assert!(
        detect_archetype(&export).is_none(),
        "setup.py alone should not trigger Python package detection"
    );
}

#[test]
fn given_pyproject_toml_and_setup_py_then_python_package() {
    let export = export_with_paths(&["pyproject.toml", "setup.py", "src/main.py"]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Python package");
    assert!(a.evidence.contains(&"pyproject.toml".to_string()));
}

// ===========================================================================
// Scenario: Multiple archetype signals coexist — single winner
// ===========================================================================

#[test]
fn given_all_archetype_signals_then_highest_priority_wins() {
    // All possible signals present at once
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
    // Rust workspace has the highest priority
    assert_eq!(a.kind, "Rust workspace (CLI)");
}

#[test]
fn given_everything_except_rust_then_nextjs_wins() {
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
fn given_container_iac_python_node_then_container_wins() {
    let export = export_with_paths(&[
        "Dockerfile",
        "k8s/deploy.yaml",
        "main.tf",
        "pyproject.toml",
        "package.json",
    ]);
    let a = detect_archetype(&export).unwrap();
    assert_eq!(a.kind, "Containerized service");
}

#[test]
fn exactly_one_archetype_returned_despite_multiple_signals() {
    // Verify the function returns a single archetype, not multiple
    let export = export_with_paths(&[
        "Cargo.toml",
        "crates/a/src/lib.rs",
        "package.json",
        "pyproject.toml",
        "main.tf",
    ]);
    let result = detect_archetype(&export);
    assert!(result.is_some());
    // It's Option<Archetype>, so by type system it's always 0 or 1;
    // verify the winner is correct
    assert!(result.unwrap().kind.starts_with("Rust workspace"));
}
