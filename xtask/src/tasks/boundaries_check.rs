use crate::cli::BoundariesCheckArgs;
use crate::proof::policy::load_policy;
use crate::proof::policy_ast::{DependencyBoundary, ProofPolicy};
use crate::proof::validate::validate_policy;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Repo-owned proof policy path. Dependency boundary rules live here so
/// retired packages such as tokmd-config stay guarded by policy, not code.
const PROOF_POLICY_PATH: &str = "ci/proof.toml";

/// Cargo.toml tables that declare dependencies.
const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundaryViolation {
    rel_path: String,
    crate_name: String,
    dep_table: String,
    dep: String,
    boundary: String,
}

pub fn run(_args: BoundariesCheckArgs) -> Result<()> {
    let repo_root = std::env::current_dir()?;
    let policy = load_checked_policy(&repo_root.join(PROOF_POLICY_PATH))?;
    let manifests = collect_analysis_manifests(&repo_root)?;
    let violations = collect_boundary_violations(&repo_root, &manifests, &policy)?;

    for violation in &violations {
        eprintln!(
            "::error file={}::{} has {} in [{}] (policy boundary `{}`)",
            violation.rel_path,
            violation.crate_name,
            violation.dep,
            violation.dep_table,
            violation.boundary
        );
    }

    if !violations.is_empty() {
        bail!(
            "Found {} analysis crate dependency boundary violation(s)",
            violations.len()
        );
    }

    println!(
        "All analysis microcrate boundaries OK ({} policy rule(s))",
        policy.dependency_boundary.len()
    );
    Ok(())
}

fn load_checked_policy(path: &Path) -> Result<ProofPolicy> {
    let policy = load_policy(path)?;
    let violations = validate_policy(&policy);
    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("::error file={}::{}", path.display(), violation.message);
        }
        bail!(
            "proof policy is invalid; run `cargo xtask proof-policy --check` before boundaries-check"
        );
    }
    Ok(policy)
}

fn collect_analysis_manifests(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let crates_dir = repo_root.join("crates");
    let mut manifests: Vec<_> = std::fs::read_dir(&crates_dir)
        .with_context(|| format!("failed to read {}", crates_dir.display()))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name();
            let name = name.to_str()?;
            if name.starts_with("tokmd-analysis") {
                let cargo_toml = entry.path().join("Cargo.toml");
                if cargo_toml.exists() {
                    return Some(cargo_toml);
                }
            }
            None
        })
        .collect();
    manifests.sort();
    Ok(manifests)
}

fn collect_boundary_violations(
    repo_root: &Path,
    manifests: &[PathBuf],
    policy: &ProofPolicy,
) -> Result<Vec<BoundaryViolation>> {
    let mut violations = Vec::new();

    for manifest in manifests {
        let content = std::fs::read_to_string(manifest)?;
        let table: toml::Table = toml::from_str(&content)?;

        let crate_name = table
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        let rel_path = manifest
            .strip_prefix(repo_root)
            .unwrap_or(manifest)
            .display();

        for &dep_table in DEP_TABLES {
            if let Some(toml::Value::Table(deps)) = table.get(dep_table) {
                for boundary in matching_boundaries(&policy.dependency_boundary, crate_name) {
                    for dep in deps.keys() {
                        if boundary.forbid.iter().any(|forbidden| forbidden == dep) {
                            violations.push(BoundaryViolation {
                                rel_path: rel_path.to_string(),
                                crate_name: crate_name.to_string(),
                                dep_table: dep_table.to_string(),
                                dep: dep.to_string(),
                                boundary: boundary.name.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(violations)
}

fn matching_boundaries<'a>(
    boundaries: &'a [DependencyBoundary],
    crate_name: &str,
) -> Vec<&'a DependencyBoundary> {
    boundaries
        .iter()
        .filter(|boundary| {
            boundary.packages.iter().any(|selector| {
                selector == "*"
                    || selector == crate_name
                    || (selector.ends_with('*')
                        && crate_name.starts_with(selector.trim_end_matches('*')))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        BoundaryViolation, collect_analysis_manifests, collect_boundary_violations,
        load_checked_policy, matching_boundaries,
    };
    use crate::proof::policy::parse_policy_str;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf()
    }

    fn minimal_policy(forbid: &[&str]) -> crate::proof::policy_ast::ProofPolicy {
        let forbid = forbid
            .iter()
            .map(|dep| format!("\"{dep}\""))
            .collect::<Vec<_>>()
            .join(", ");
        parse_policy_str(&format!(
            r#"
schema = "tokmd.proof_policy.v1"

[[dependency_boundary]]
name = "retired_tokmd_config_must_not_return"
packages = ["*"]
forbid = [{forbid}]
reason = "tokmd-config is retired."
"#
        ))
        .expect("policy should parse")
    }

    fn write_manifest(root: &std::path::Path, crate_name: &str, dependency: &str) -> PathBuf {
        let crate_dir = root.join("crates").join(crate_name);
        fs::create_dir_all(&crate_dir).expect("create crate dir");
        let manifest = crate_dir.join("Cargo.toml");
        fs::write(
            &manifest,
            format!(
                r#"
[package]
name = "{crate_name}"
version = "0.0.0"
edition = "2024"

[dependencies]
{dependency} = {{ path = "../{dependency}" }}
"#
            ),
        )
        .expect("write manifest");
        manifest
    }

    #[test]
    fn repo_policy_loads_dependency_boundaries() {
        let policy = load_checked_policy(&workspace_root().join("ci/proof.toml"))
            .expect("repo proof policy should load");

        assert!(policy.dependency_boundary.iter().any(|boundary| {
            boundary.name == "retired_tokmd_config_must_not_return"
                && boundary.forbid.iter().any(|dep| dep == "tokmd-config")
        }));
    }

    #[test]
    fn missing_policy_is_reported() {
        let dir = tempdir().expect("tempdir");
        let err = load_checked_policy(&dir.path().join("missing.toml"))
            .expect_err("missing policy should fail");

        assert!(err.to_string().contains("failed to read"));
    }

    #[test]
    fn malformed_policy_is_reported() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("proof.toml");
        fs::write(&path, "schema = [").expect("write malformed policy");

        let err = load_checked_policy(&path).expect_err("malformed policy should fail");

        assert!(err.to_string().contains("failed to parse"));
    }

    #[test]
    fn matching_boundaries_supports_star_and_prefix_selectors() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[[dependency_boundary]]
name = "all"
packages = ["*"]
forbid = ["tokmd-config"]
reason = "all packages"

[[dependency_boundary]]
name = "analysis"
packages = ["tokmd-analysis*"]
forbid = ["tokmd-core"]
reason = "analysis packages"
"#,
        )
        .expect("policy should parse");

        let matches = matching_boundaries(&policy.dependency_boundary, "tokmd-analysis");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].name, "all");
        assert_eq!(matches[1].name, "analysis");
    }

    #[test]
    fn policy_rule_detects_forbidden_dependency() {
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest(dir.path(), "tokmd-analysis-demo", "tokmd-config");
        let policy = minimal_policy(&["tokmd-config"]);

        let violations = collect_boundary_violations(dir.path(), &[manifest], &policy)
            .expect("collect violations");

        assert_eq!(
            violations,
            vec![BoundaryViolation {
                rel_path: "crates\\tokmd-analysis-demo\\Cargo.toml"
                    .replace('\\', std::path::MAIN_SEPARATOR_STR),
                crate_name: "tokmd-analysis-demo".to_string(),
                dep_table: "dependencies".to_string(),
                dep: "tokmd-config".to_string(),
                boundary: "retired_tokmd_config_must_not_return".to_string(),
            }]
        );
    }

    #[test]
    fn analysis_manifest_collection_remains_sorted() {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), "tokmd-analysis-zed", "tokmd-types");
        write_manifest(dir.path(), "tokmd-analysis-alpha", "tokmd-types");
        write_manifest(dir.path(), "tokmd-model", "tokmd-types");

        let manifests = collect_analysis_manifests(dir.path()).expect("collect manifests");
        let names = manifests
            .iter()
            .map(|path| {
                path.parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
            })
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["tokmd-analysis-alpha", "tokmd-analysis-zed"]);
    }
}
