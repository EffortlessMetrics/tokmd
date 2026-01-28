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
    fn detects_nextjs() {
        let export = export_with_paths(&["package.json", "next.config.js", "pages/index.tsx"]);
        let archetype = detect_archetype(&export).unwrap();
        assert_eq!(archetype.kind, "Next.js app");
    }
}
