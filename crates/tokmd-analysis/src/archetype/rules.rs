//! Archetype-specific detection rules.
//!
//! The parent module owns export-row filtering and path normalization. This
//! module owns the ordered rules that turn a normalized path set into a
//! repository archetype receipt fragment.

use std::collections::BTreeSet;

use tokmd_analysis_types::Archetype;

pub(crate) fn detect(files: &BTreeSet<String>) -> Option<Archetype> {
    if let Some(archetype) = rust_workspace(files) {
        return Some(archetype);
    }
    if let Some(archetype) = nextjs_app(files) {
        return Some(archetype);
    }
    if let Some(archetype) = containerized_service(files) {
        return Some(archetype);
    }
    if let Some(archetype) = iac_project(files) {
        return Some(archetype);
    }
    if let Some(archetype) = python_package(files) {
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

    fn files_set(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn detects_rust_workspace_cli() {
        let files = files_set(&[
            "Cargo.toml",
            "crates/core/Cargo.toml",
            "crates/core/src/lib.rs",
            "src/main.rs",
        ]);
        let archetype = detect(&files).unwrap();
        assert!(archetype.kind.contains("Rust workspace"));
        assert!(archetype.kind.contains("CLI"));
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e.starts_with("crates/") || e.starts_with("packages/")),
            "evidence must contain workspace dir path: {:?}",
            archetype.evidence
        );
    }

    #[test]
    fn rust_workspace_needs_cargo_toml() {
        let files = files_set(&["crates/core/src/lib.rs"]);
        assert!(rust_workspace(&files).is_none());
    }

    #[test]
    fn rust_workspace_needs_workspace_dir() {
        let files = files_set(&["Cargo.toml", "src/lib.rs"]);
        assert!(rust_workspace(&files).is_none());
    }

    #[test]
    fn rust_workspace_with_packages_dir() {
        let files = files_set(&["Cargo.toml", "packages/foo/src/lib.rs"]);
        let archetype = rust_workspace(&files).unwrap();
        assert_eq!(archetype.kind, "Rust workspace");
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e.starts_with("packages/")),
            "evidence must contain packages/ path: {:?}",
            archetype.evidence
        );
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
        let files = files_set(&["Cargo.toml", "crates/foo/src/lib.rs"]);
        let archetype = rust_workspace(&files).unwrap();
        assert_eq!(archetype.kind, "Rust workspace");
        assert!(!archetype.kind.contains("CLI"));
    }

    #[test]
    fn detects_nextjs() {
        let files = files_set(&["package.json", "next.config.js", "pages/index.tsx"]);
        let archetype = detect(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e.ends_with("next.config.js")),
            "evidence must contain next.config.js: {:?}",
            archetype.evidence
        );
    }

    #[test]
    fn nextjs_needs_package_json() {
        let files = files_set(&["next.config.js", "pages/index.tsx"]);
        assert!(nextjs_app(&files).is_none());
    }

    #[test]
    fn nextjs_needs_next_config() {
        let files = files_set(&["package.json", "pages/index.tsx"]);
        assert!(nextjs_app(&files).is_none());
    }

    #[test]
    fn nextjs_with_mjs_config() {
        let files = files_set(&["package.json", "next.config.mjs"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e.ends_with("next.config.mjs")),
            "evidence must contain next.config.mjs: {:?}",
            archetype.evidence
        );
    }

    #[test]
    fn nextjs_with_ts_config() {
        let files = files_set(&["package.json", "next.config.ts"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e.ends_with("next.config.ts")),
            "evidence must contain next.config.ts: {:?}",
            archetype.evidence
        );
    }

    #[test]
    fn nextjs_with_subdir_next_config_mjs() {
        let files = files_set(&["package.json", "apps/web/next.config.mjs"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e == "apps/web/next.config.mjs"),
            "evidence must contain apps/web/next.config.mjs: {:?}",
            archetype.evidence
        );
    }

    #[test]
    fn nextjs_with_nested_config() {
        let files = files_set(&["package.json", "app/next.config.js"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
        assert!(
            archetype.evidence.iter().any(|e| e == "app/next.config.js"),
            "evidence must contain app/next.config.js: {:?}",
            archetype.evidence
        );
    }

    #[test]
    fn nextjs_with_subdir_next_config_ts() {
        let files = files_set(&["package.json", "apps/web/next.config.ts"]);
        let archetype = nextjs_app(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
        assert!(
            archetype
                .evidence
                .iter()
                .any(|e| e == "apps/web/next.config.ts"),
            "evidence must contain apps/web/next.config.ts: {:?}",
            archetype.evidence
        );
    }

    #[test]
    fn containerized_service_needs_dockerfile() {
        let files = files_set(&["k8s/deployment.yaml"]);
        assert!(containerized_service(&files).is_none());
    }

    #[test]
    fn containerized_service_needs_k8s() {
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

    #[test]
    fn node_package_detected() {
        let files = files_set(&["package.json", "src/index.js"]);
        let archetype = detect(&files).unwrap();
        assert_eq!(archetype.kind, "Node package");
    }

    #[test]
    fn rust_workspace_takes_priority_over_node() {
        let files = files_set(&["Cargo.toml", "crates/foo/src/lib.rs", "package.json"]);
        let archetype = detect(&files).unwrap();
        assert!(archetype.kind.contains("Rust workspace"));
    }

    #[test]
    fn nextjs_takes_priority_over_node() {
        let files = files_set(&["package.json", "next.config.js"]);
        let archetype = detect(&files).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
    }

    #[test]
    fn no_archetype_for_empty() {
        let files = files_set(&[]);
        assert!(detect(&files).is_none());
    }

    #[test]
    fn no_archetype_for_generic_files() {
        let files = files_set(&["README.md", "src/lib.rs"]);
        assert!(detect(&files).is_none());
    }
}
