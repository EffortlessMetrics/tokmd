use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use cargo_metadata::{Metadata, MetadataCommand, PackageId};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use crate::cli::VersionConsistencyArgs;

const NODE_PACKAGE_MANIFESTS: &[&str] = &[
    "crates/tokmd-node/package.json",
    "crates/tokmd-node/npm/package.json",
];

pub fn run(_args: VersionConsistencyArgs) -> Result<()> {
    let workspace_root = find_workspace_root()?;
    let workspace_version = load_workspace_version(&workspace_root)?;

    println!("Checking version consistency against workspace version {workspace_version}\n");

    let metadata = load_workspace_metadata(&workspace_root)?;
    check_cargo_versions(&metadata, &workspace_version)?;
    check_workspace_dependency_versions(&workspace_root, &workspace_version)?;
    check_node_manifest_versions(&workspace_root, &workspace_version)?;
    check_case_insensitive_path_collisions(&workspace_root)?;

    println!("Version consistency checks passed.");
    Ok(())
}

fn check_cargo_versions(metadata: &Metadata, expected: &str) -> Result<()> {
    let workspace_member_ids: HashSet<&PackageId> = metadata.workspace_members.iter().collect();
    let mut mismatches = Vec::new();

    for package in &metadata.packages {
        if !workspace_member_ids.contains(&package.id) {
            continue;
        }

        // xtask and tokmd-fuzz are intentionally excluded from release-aligned release metadata checks.
        if matches!(package.name.as_str(), "xtask" | "tokmd-fuzz" | "fuzz") {
            continue;
        }

        let package_version = package.version.to_string();
        if package_version != expected {
            mismatches.push(format!("{} ({})", package.name, package_version));
        }
    }

    if !mismatches.is_empty() {
        bail!(
            "Cargo crate versions are out of sync with workspace {}:\n  {}",
            expected,
            mismatches.join("\n  ")
        );
    }

    println!("  ✓ Cargo crate versions match {}.", expected);
    Ok(())
}

fn check_workspace_dependency_versions(workspace_root: &Path, expected: &str) -> Result<()> {
    let manifest = read_toml(&workspace_root.join("Cargo.toml"))?;
    let workspace = manifest
        .get("workspace")
        .and_then(TomlValue::as_table)
        .context("Missing [workspace] table in root Cargo.toml")?;

    let mut mismatches = Vec::new();

    if let Some(deps) = workspace.get("dependencies").and_then(TomlValue::as_table) {
        for (name, dependency) in deps {
            let Some(dep_table) = dependency.as_table() else {
                continue;
            };
            if !dep_table.contains_key("path") {
                continue;
            }

            let Some(dep_version) = dep_table.get("version").and_then(TomlValue::as_str) else {
                continue;
            };

            if dep_version != expected {
                mismatches.push(format!("{} dependency version {}", name, dep_version));
            }
        }
    }

    if !mismatches.is_empty() {
        bail!(
            "Cargo workspace dependency versions are out of sync with workspace {}:\n  {}",
            expected,
            mismatches.join("\n  ")
        );
    }

    println!(
        "  ✓ Cargo workspace dependency versions match {}.",
        expected
    );
    Ok(())
}

fn check_node_manifest_versions(workspace_root: &Path, expected: &str) -> Result<()> {
    let mut mismatches = Vec::new();

    for path in NODE_PACKAGE_MANIFESTS {
        let manifest = read_package_manifest(workspace_root, path)
            .with_context(|| format!("Reading {path}"))?;
        let actual = manifest_package_version(&manifest, path)?;
        if actual != expected {
            mismatches.push(format!("{path} ({actual})"));
        }

        mismatches.extend(find_internal_node_dependency_mismatches(
            path, &manifest, expected,
        ));
    }

    if !mismatches.is_empty() {
        bail!(
            "Node package manifest release metadata is out of sync with workspace {}:\n  {}",
            expected,
            mismatches.join("\n  ")
        );
    }

    println!("  ✓ Node package manifest versions match {}.", expected);
    Ok(())
}

fn check_case_insensitive_path_collisions(workspace_root: &Path) -> Result<()> {
    let tracked_paths = read_tracked_paths(workspace_root)?;
    let collisions = detect_case_insensitive_collisions(tracked_paths);

    if !collisions.is_empty() {
        let details = collisions
            .into_iter()
            .map(|paths| format!("{} -> {}", paths[0].to_lowercase(), paths.join(", ")))
            .collect::<Vec<_>>()
            .join("\n  ");

        bail!(
            "Tracked paths collide on case-insensitive filesystems:\n  {}\nRename one side of each collision before release.",
            details
        );
    }

    println!("  ✓ No case-insensitive tracked-path collisions detected.");
    Ok(())
}

fn find_workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml)
                .with_context(|| format!("Failed to read {}", cargo_toml.display()))?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!("Could not find workspace root (Cargo.toml with [workspace])");
        }
    }
}

fn load_workspace_version(workspace_root: &Path) -> Result<String> {
    let manifest = read_toml(&workspace_root.join("Cargo.toml"))?;
    let workspace = manifest
        .get("workspace")
        .and_then(TomlValue::as_table)
        .context("Missing [workspace] table in root Cargo.toml")?;
    let package = workspace
        .get("package")
        .and_then(TomlValue::as_table)
        .context("Missing [workspace.package] table in root Cargo.toml")?;
    let version = package
        .get("version")
        .and_then(TomlValue::as_str)
        .context("Missing [workspace.package].version in root Cargo.toml")?
        .to_string();

    Ok(version)
}

fn load_workspace_metadata(workspace_root: &Path) -> Result<Metadata> {
    MetadataCommand::new()
        .manifest_path(workspace_root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .context("Failed to load cargo metadata")
}

fn read_tracked_paths(workspace_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(workspace_root)
        .output()
        .context("Failed to run `git ls-files -z`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`git ls-files -z` failed: {}", stderr.trim());
    }

    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8(entry.to_vec()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("`git ls-files -z` produced non-UTF-8 output")
}

use std::borrow::Cow;

fn to_lowercase_cow(s: &str) -> Cow<'_, str> {
    if s.chars()
        .any(|c| c.is_ascii_uppercase() || c.is_uppercase())
    {
        Cow::Owned(s.to_lowercase())
    } else {
        Cow::Borrowed(s)
    }
}

fn detect_case_insensitive_collisions(paths: Vec<String>) -> Vec<Vec<String>> {
    let mut by_lowercase = BTreeMap::<String, Vec<String>>::new();

    for path in paths {
        let lower = to_lowercase_cow(&path);
        if let Some(entries) = by_lowercase.get_mut(lower.as_ref()) {
            entries.push(path);
        } else {
            by_lowercase.insert(lower.into_owned(), vec![path]);
        }
    }

    by_lowercase
        .into_values()
        .filter_map(|mut entries| {
            entries.sort();
            entries.dedup();
            (entries.len() > 1).then_some(entries)
        })
        .collect()
}

fn read_package_manifest(workspace_root: &Path, path: &str) -> Result<JsonValue> {
    let package_path = workspace_root.join(path);
    if !package_path.exists() {
        bail!("Missing package manifest: {path}");
    }

    let raw =
        fs::read_to_string(&package_path).with_context(|| format!("Failed to read {path}"))?;
    serde_json::from_str(&raw).with_context(|| format!("Failed to parse JSON in {path}"))
}

fn manifest_package_version(json: &JsonValue, path: &str) -> Result<String> {
    let version = json
        .get("version")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| anyhow::anyhow!("Missing `version` in {path}"))?;

    Ok(version.to_string())
}

fn find_internal_node_dependency_mismatches(
    path: &str,
    json: &JsonValue,
    expected: &str,
) -> Vec<String> {
    let mut mismatches = Vec::new();

    for section in ["dependencies", "optionalDependencies", "peerDependencies"] {
        let Some(entries) = json.get(section).and_then(JsonValue::as_object) else {
            continue;
        };

        for (name, version) in entries {
            if !name.starts_with("@tokmd/") {
                continue;
            }

            let Some(actual) = version.as_str() else {
                mismatches.push(format!("{path} {section}.{name} (non-string version)"));
                continue;
            };

            if actual != expected {
                mismatches.push(format!("{path} {section}.{name} ({actual})"));
            }
        }
    }

    mismatches
}

fn read_toml(path: &Path) -> Result<TomlValue> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("Failed to parse TOML in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workspace_version() {
        let workspace_root = find_workspace_root().expect("workspace root should parse");
        let version =
            load_workspace_version(&workspace_root).expect("workspace version should parse");
        assert!(!version.is_empty());
    }

    #[test]
    fn test_read_package_manifest_errors() {
        let workspace_root = find_workspace_root().expect("workspace root should parse");
        assert!(read_package_manifest(&workspace_root, "no-such-file.json").is_err());
    }

    #[test]
    fn detects_case_insensitive_collisions() {
        let collisions = detect_case_insensitive_collisions(vec![
            "docs/PR_BODY.md".to_string(),
            "docs/pr_body.md".to_string(),
            "README.md".to_string(),
        ]);

        assert_eq!(collisions.len(), 1);
        assert_eq!(
            collisions[0],
            vec!["docs/PR_BODY.md".to_string(), "docs/pr_body.md".to_string()]
        );
    }

    #[test]
    fn ignores_unique_paths_when_checking_case_collisions() {
        let collisions = detect_case_insensitive_collisions(vec![
            "docs/README.md".to_string(),
            "src/lib.rs".to_string(),
            "web/runner/runtime.js".to_string(),
        ]);

        assert!(collisions.is_empty());
    }

    #[test]
    fn detects_internal_node_dependency_version_mismatches() {
        let manifest = serde_json::json!({
            "version": "1.9.0",
            "optionalDependencies": {
                "@tokmd/core-linux-x64-gnu": "1.8.1",
                "@tokmd/core-win32-x64-msvc": "1.9.0"
            },
            "dependencies": {
                "@tokmd/helper": "workspace:*",
                "chalk": "^5.0.0"
            }
        });

        let mismatches = find_internal_node_dependency_mismatches(
            "crates/tokmd-node/package.json",
            &manifest,
            "1.9.0",
        );

        assert_eq!(
            mismatches,
            vec![
                "crates/tokmd-node/package.json dependencies.@tokmd/helper (workspace:*)"
                    .to_string(),
                "crates/tokmd-node/package.json optionalDependencies.@tokmd/core-linux-x64-gnu (1.8.1)"
                    .to_string(),
            ]
        );
    }
}
