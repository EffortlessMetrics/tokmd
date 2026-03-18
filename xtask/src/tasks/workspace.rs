use anyhow::{Context, Result, bail};
use cargo_metadata::{MetadataCommand, PackageId};
use std::collections::HashSet;
use std::process::Command;

fn workspace_package_names() -> Result<Vec<String>> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Failed to load cargo metadata")?;

    let workspace_members: HashSet<PackageId> = metadata.workspace_members.into_iter().collect();
    let mut packages: Vec<String> = metadata
        .packages
        .into_iter()
        .filter(|package| workspace_members.contains(&package.id))
        .map(|package| package.name.to_string())
        .collect();

    packages.sort();
    packages.dedup();
    Ok(packages)
}

pub fn run_workspace_fmt(check: bool) -> Result<()> {
    if !cfg!(windows) {
        let mut command = Command::new("cargo");
        command.arg("fmt").arg("--all");
        if check {
            command.arg("--check");
        }

        let status = command.status().context("failed to run cargo fmt")?;
        if !status.success() {
            bail!("fmt failed");
        }

        return Ok(());
    }

    for package in workspace_package_names()? {
        let mut command = Command::new("cargo");
        command.arg("fmt").arg("-p").arg(&package);
        if check {
            command.arg("--check");
        }

        let status = command
            .status()
            .with_context(|| format!("failed to run cargo fmt for package {package}"))?;
        if !status.success() {
            bail!("fmt failed for package {package}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::workspace_package_names;

    #[test]
    fn workspace_package_names_are_sorted_and_include_known_members() {
        let packages = workspace_package_names().expect("workspace metadata should load");
        let mut sorted = packages.clone();
        sorted.sort();
        sorted.dedup();

        assert_eq!(packages, sorted);
        assert!(packages.iter().any(|name| name == "tokmd"));
        assert!(packages.iter().any(|name| name == "xtask"));
    }
}
