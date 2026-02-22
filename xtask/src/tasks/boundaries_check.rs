use crate::cli::BoundariesCheckArgs;
use anyhow::{Result, bail};

/// Dependency names that analysis microcrates must never depend on.
const FORBIDDEN: &[&str] = &["tokmd-config"];

/// Cargo.toml tables that declare dependencies.
const DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

pub fn run(_args: BoundariesCheckArgs) -> Result<()> {
    let repo_root = std::env::current_dir()?;
    let crates_dir = repo_root.join("crates");

    let mut manifests: Vec<_> = std::fs::read_dir(&crates_dir)?
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

    let mut violations = 0u32;

    for manifest in &manifests {
        let content = std::fs::read_to_string(manifest)?;
        let table: toml::Table = toml::from_str(&content)?;

        let crate_name = table
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        let rel_path = manifest
            .strip_prefix(&repo_root)
            .unwrap_or(manifest)
            .display();

        for &dep_table in DEP_TABLES {
            if let Some(toml::Value::Table(deps)) = table.get(dep_table) {
                for dep in deps.keys() {
                    if FORBIDDEN.contains(&dep.as_str()) {
                        eprintln!(
                            "::error file={rel_path}::{crate_name} has {dep} in [{dep_table}]"
                        );
                        violations += 1;
                    }
                }
            }
        }
    }

    if violations > 0 {
        bail!(
            "Found {} analysis crate(s) with forbidden dependencies",
            violations
        );
    }

    println!("All analysis microcrate boundaries OK");
    Ok(())
}
