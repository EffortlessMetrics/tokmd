use std::collections::BTreeSet;

use tokmd_analysis_types::Archetype;
use tokmd_types::{ExportData, FileKind, FileRow};

pub(crate) fn detect_archetype(export: &ExportData) -> Option<Archetype> {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let mut files: BTreeSet<String> = BTreeSet::new();
    for row in parents {
        files.insert(row.path.replace('\\', "/"));
    }

    if let Some(archetype) = rust_workspace(&files) {
        return Some(archetype);
    }
    if let Some(archetype) = nextjs_app(&files) {
        return Some(archetype);
    }
    if let Some(archetype) = containerized_service(&files) {
        return Some(archetype);
    }
    if let Some(archetype) = iac_project(&files) {
        return Some(archetype);
    }
    if let Some(archetype) = python_package(&files) {
        return Some(archetype);
    }
    if files.contains("package.json") {
        return Some(Archetype {
            kind: "Node package".to_string(),
            evidence: vec!["package.json".to_string()],
        });
    }

    None
}

fn rust_workspace(files: &BTreeSet<String>) -> Option<Archetype> {
    let has_manifest = files.contains("Cargo.toml");
    let has_workspace_dir = files
        .iter()
        .any(|p| p.starts_with("crates/") || p.starts_with("packages/"));
    if !has_manifest || !has_workspace_dir {
        return None;
    }

    let mut evidence = vec!["Cargo.toml".to_string()];
    if let Some(path) = files
        .iter()
        .find(|p| p.starts_with("crates/") || p.starts_with("packages/"))
    {
        evidence.push(path.clone());
    }

    let is_cli = files
        .iter()
        .any(|p| p.ends_with("src/main.rs") || p.contains("/src/bin/"));
    let kind = if is_cli {
        "Rust workspace (CLI)"
    } else {
        "Rust workspace"
    };

    Some(Archetype {
        kind: kind.to_string(),
        evidence,
    })
}

fn nextjs_app(files: &BTreeSet<String>) -> Option<Archetype> {
    let has_package = files.contains("package.json");
    let has_next_config = files.iter().any(|p| {
        p.starts_with("next.config.")
            || p.ends_with("/next.config.js")
            || p.ends_with("/next.config.mjs")
            || p.ends_with("/next.config.ts")
    });
    if has_package && has_next_config {
        let mut evidence = vec!["package.json".to_string()];
        if let Some(cfg) = files.iter().find(|p| {
            p.ends_with("next.config.js")
                || p.ends_with("next.config.mjs")
                || p.ends_with("next.config.ts")
        }) {
            evidence.push(cfg.clone());
        }
        return Some(Archetype {
            kind: "Next.js app".to_string(),
            evidence,
        });
    }
    None
}

fn containerized_service(files: &BTreeSet<String>) -> Option<Archetype> {
    let has_docker = files.contains("Dockerfile");
    let has_k8s = files
        .iter()
        .any(|p| p.starts_with("k8s/") || p.starts_with("kubernetes/"));
    if has_docker && has_k8s {
        return Some(Archetype {
            kind: "Containerized service".to_string(),
            evidence: vec!["Dockerfile".to_string()],
        });
    }
    None
}

fn iac_project(files: &BTreeSet<String>) -> Option<Archetype> {
    let has_tf = files
        .iter()
        .any(|p| p.ends_with(".tf") || p.starts_with("terraform/"));
    if has_tf {
        return Some(Archetype {
            kind: "Infrastructure as code".to_string(),
            evidence: vec!["terraform/".to_string()],
        });
    }
    None
}

fn python_package(files: &BTreeSet<String>) -> Option<Archetype> {
    if files.contains("pyproject.toml") {
        return Some(Archetype {
            kind: "Python package".to_string(),
            evidence: vec!["pyproject.toml".to_string()],
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_config::ChildIncludeMode;
    use tokmd_types::{ExportData, FileKind, FileRow};

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

    fn files_set(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    // =============================================================================
    // rust_workspace tests
    // =============================================================================

    #[test]
    fn detects_rust_workspace_cli() {
        let export = export_with_paths(&[
            "Cargo.toml",
            "crates/core/Cargo.toml",
            "crates/core/src/lib.rs",
            "src/main.rs",
        ]);
        let archetype = detect_archetype(&export).unwrap();
        assert!(archetype.kind.contains("Rust workspace"));
        assert!(archetype.kind.contains("CLI"));
    }

    #[test]
    fn rust_workspace_needs_cargo_toml() {
        // Missing Cargo.toml should return None
        let files = files_set(&["crates/core/src/lib.rs"]);
        assert!(rust_workspace(&files).is_none());
    }

    #[test]
    fn rust_workspace_needs_workspace_dir() {
        // Has Cargo.toml but no crates/ or packages/
        let files = files_set(&["Cargo.toml", "src/lib.rs"]);
        assert!(rust_workspace(&files).is_none());
    }

    #[test]
    fn rust_workspace_with_packages_dir() {
        // Should work with packages/ instead of crates/
        let files = files_set(&["Cargo.toml", "packages/foo/src/lib.rs"]);
        let archetype = rust_workspace(&files).unwrap();
        assert_eq!(archetype.kind, "Rust workspace");
    }

    #[test]
    fn rust_workspace_detects_cli_with_main_rs() {
        let files = files_set(&["Cargo.toml", "crates/foo/src/lib.rs", "src/main.rs"]);
        let archetype = rust_workspace(&files).unwrap();
        assert!(archetype.kind.contains("CLI"));
    }

    #[test]
    fn rust_workspace_detects_cli_with_bin_dir() {
        let files = files_set(&[
            "Cargo.toml",
            "crates/foo/src/lib.rs",
            "crates/foo/src/bin/cli.rs",
        ]);
        let archetype = rust_workspace(&files).unwrap();
        assert!(archetype.kind.contains("CLI"));
    }

    #[test]
    fn rust_workspace_library_only() {
        // No main.rs or bin/, should be plain workspace
        let files = files_set(&["Cargo.toml", "crates/foo/src/lib.rs"]);
        let archetype = rust_workspace(&files).unwrap();
        assert_eq!(archetype.kind, "Rust workspace");
        assert!(!archetype.kind.contains("CLI"));
    }

    // =============================================================================
    // nextjs_app tests
    // =============================================================================

    #[test]
    fn detects_nextjs() {
        let export = export_with_paths(&["package.json", "next.config.js", "pages/index.tsx"]);
        let archetype = detect_archetype(&export).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
    }

    #[test]
    fn nextjs_needs_package_json() {
        // Has next.config.js but no package.json
        let files = files_set(&["next.config.js", "pages/index.tsx"]);
        assert!(nextjs_app(&files).is_none());
    }

    #[test]
    fn nextjs_needs_next_config() {
        // Has package.json but no next config
        let files = files_set(&["package.json", "pages/index.tsx"]);
        assert!(nextjs_app(&files).is_none());
    }

    #[test]
    fn nextjs_with_mjs_config() {
        let files = files_set(&["package.json", "next.config.mjs"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
    }

    #[test]
    fn nextjs_with_ts_config() {
        let files = files_set(&["package.json", "next.config.ts"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
    }

    #[test]
    fn nextjs_with_nested_config() {
        // Config in subdirectory
        let files = files_set(&["package.json", "app/next.config.js"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
    }

    // =============================================================================
    // containerized_service tests
    // =============================================================================

    #[test]
    fn containerized_service_needs_dockerfile() {
        // Has k8s/ but no Dockerfile
        let files = files_set(&["k8s/deployment.yaml"]);
        assert!(containerized_service(&files).is_none());
    }

    #[test]
    fn containerized_service_needs_k8s() {
        // Has Dockerfile but no k8s/
        let files = files_set(&["Dockerfile", "src/main.rs"]);
        assert!(containerized_service(&files).is_none());
    }

    #[test]
    fn containerized_service_detected() {
        let files = files_set(&["Dockerfile", "k8s/deployment.yaml"]);
        let archetype = containerized_service(&files).unwrap();
        assert_eq!(archetype.kind, "Containerized service");
    }

    #[test]
    fn containerized_service_with_kubernetes_dir() {
        let files = files_set(&["Dockerfile", "kubernetes/deployment.yaml"]);
        let archetype = containerized_service(&files).unwrap();
        assert_eq!(archetype.kind, "Containerized service");
    }

    // =============================================================================
    // iac_project tests
    // =============================================================================

    #[test]
    fn iac_project_with_tf_file() {
        let files = files_set(&["main.tf"]);
        let archetype = iac_project(&files).unwrap();
        assert_eq!(archetype.kind, "Infrastructure as code");
    }

    #[test]
    fn iac_project_with_terraform_dir() {
        let files = files_set(&["terraform/main.tf"]);
        let archetype = iac_project(&files).unwrap();
        assert_eq!(archetype.kind, "Infrastructure as code");
    }

    #[test]
    fn iac_project_not_detected_without_tf() {
        let files = files_set(&["src/main.rs", "Cargo.toml"]);
        assert!(iac_project(&files).is_none());
    }

    // =============================================================================
    // python_package tests
    // =============================================================================

    #[test]
    fn python_package_detected() {
        let files = files_set(&["pyproject.toml", "src/main.py"]);
        let archetype = python_package(&files).unwrap();
        assert_eq!(archetype.kind, "Python package");
    }

    #[test]
    fn python_package_not_detected_without_pyproject() {
        let files = files_set(&["setup.py", "src/main.py"]);
        assert!(python_package(&files).is_none());
    }

    // =============================================================================
    // Node package tests
    // =============================================================================

    #[test]
    fn node_package_detected() {
        let export = export_with_paths(&["package.json", "src/index.js"]);
        let archetype = detect_archetype(&export).unwrap();
        assert_eq!(archetype.kind, "Node package");
    }

    // =============================================================================
    // Priority tests
    // =============================================================================

    #[test]
    fn rust_workspace_takes_priority_over_node() {
        // Has both Cargo.toml/crates and package.json
        let export = export_with_paths(&["Cargo.toml", "crates/foo/src/lib.rs", "package.json"]);
        let archetype = detect_archetype(&export).unwrap();
        assert!(archetype.kind.contains("Rust workspace"));
    }

    #[test]
    fn nextjs_takes_priority_over_node() {
        let export = export_with_paths(&["package.json", "next.config.js"]);
        let archetype = detect_archetype(&export).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
    }

    #[test]
    fn no_archetype_for_empty() {
        let export = export_with_paths(&[]);
        assert!(detect_archetype(&export).is_none());
    }

    #[test]
    fn no_archetype_for_generic_files() {
        let export = export_with_paths(&["README.md", "src/lib.rs"]);
        assert!(detect_archetype(&export).is_none());
    }
}
