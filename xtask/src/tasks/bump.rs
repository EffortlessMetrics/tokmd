//! Version bump task for the tokmd workspace.
//!
//! Updates all version references across the workspace:
//! - [workspace.package].version in root Cargo.toml
//! - [workspace.dependencies] version fields for internal crates
//! - Optionally updates schema version constants

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value as JsonValue;

use crate::cli::BumpArgs;

/// Schema version location for updating.
#[derive(Debug, Clone)]
struct SchemaVersionLocation {
    /// Path relative to workspace root.
    path: &'static str,
    /// Constant name to match.
    constant: &'static str,
    /// Current version for display.
    current: u32,
}

/// Known schema version locations in the codebase.
const SCHEMA_LOCATIONS: &[SchemaVersionLocation] = &[
    SchemaVersionLocation {
        path: "crates/tokmd-types/src/lib.rs",
        constant: "SCHEMA_VERSION",
        current: 2,
    },
    SchemaVersionLocation {
        path: "crates/tokmd-analysis-types/src/lib.rs",
        constant: "ANALYSIS_SCHEMA_VERSION",
        current: 9,
    },
    SchemaVersionLocation {
        path: "crates/tokmd-types/src/cockpit.rs",
        constant: "COCKPIT_SCHEMA_VERSION",
        current: 3,
    },
    SchemaVersionLocation {
        path: "crates/tokmd-tool-schema/src/lib.rs",
        constant: "TOOL_SCHEMA_VERSION",
        current: 1,
    },
    SchemaVersionLocation {
        path: "crates/tokmd-types/src/lib.rs",
        constant: "CONTEXT_SCHEMA_VERSION",
        current: 4,
    },
    SchemaVersionLocation {
        path: "crates/tokmd-types/src/lib.rs",
        constant: "CONTEXT_BUNDLE_SCHEMA_VERSION",
        current: 2,
    },
    SchemaVersionLocation {
        path: "crates/tokmd-types/src/lib.rs",
        constant: "HANDOFF_SCHEMA_VERSION",
        current: 5,
    },
];

const NODE_PACKAGE_MANIFESTS: &[&str] = &[
    "crates/tokmd-node/package.json",
    "crates/tokmd-node/npm/package.json",
    "web/runner/package.json",
];

/// Run the version bump task.
pub fn run(args: BumpArgs) -> Result<()> {
    // Validate version format
    validate_semver(&args.version)?;

    let workspace_root = find_workspace_root()?;
    let cargo_toml_path = workspace_root.join("Cargo.toml");

    // Read current Cargo.toml
    let content = fs::read_to_string(&cargo_toml_path).context("Failed to read root Cargo.toml")?;

    // Parse current version
    let current_version = extract_workspace_version(&content)?;

    println!("=== tokmd Version Bump ===\n");
    println!("Current version: {}", current_version);
    println!("New version:     {}", args.version);
    println!();

    if current_version == args.version {
        println!("Version is already {}. Nothing to do.", args.version);
        return Ok(());
    }

    // Plan changes
    let mut changes: Vec<String> = Vec::new();

    // 1. Update [workspace.package].version
    let new_content = update_workspace_package_version(&content, &args.version)?;
    changes.push(format!(
        "Cargo.toml: [workspace.package].version = \"{}\" -> \"{}\"",
        current_version, args.version
    ));

    // 2. Update [workspace.dependencies] internal crate versions
    let (final_content, dep_changes) = update_workspace_dependencies(&new_content, &args.version)?;
    changes.extend(dep_changes);

    // 3. Update Node package manifest versions and internal @tokmd/* dependency versions.
    let node_manifest_updates = plan_node_manifest_updates(&workspace_root, &args.version)?;
    for update in &node_manifest_updates {
        changes.extend(update.changes.iter().cloned());
    }

    // Print planned changes
    println!("Planned changes:");
    for change in &changes {
        println!("  - {}", change);
    }

    // Schema version updates - validate first
    if let Some(schema_bumps) = &args.schema {
        println!("\nSchema version updates:");
        for bump in schema_bumps {
            let (name, new_ver) = parse_schema_bump(bump)?;
            if let Some(loc) = SCHEMA_LOCATIONS.iter().find(|l| l.constant == name) {
                println!(
                    "  - {}: {} -> {} (in {})",
                    name, loc.current, new_ver, loc.path
                );
            } else {
                bail!(
                    "Unknown schema constant: {}. Known constants: {}",
                    name,
                    SCHEMA_LOCATIONS
                        .iter()
                        .map(|l| l.constant)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
    }

    if args.dry_run {
        println!("\n[DRY RUN] No changes written.");
        return Ok(());
    }

    // Write Cargo.toml changes
    fs::write(&cargo_toml_path, &final_content).context("Failed to write root Cargo.toml")?;
    println!("\nWrote: {}", cargo_toml_path.display());

    for update in &node_manifest_updates {
        let manifest_path = workspace_root.join(&update.path);
        fs::write(&manifest_path, &update.updated_content)
            .with_context(|| format!("Failed to write {}", update.path))?;
        println!("Wrote: {}", manifest_path.display());
    }

    // Apply schema version updates if specified
    if let Some(schema_bumps) = &args.schema {
        for bump in schema_bumps {
            let (name, new_ver) = parse_schema_bump(bump)?;
            update_schema_version(&workspace_root, &name, new_ver)?;
        }
    }

    // Summary
    println!("\n=== Summary ===");
    println!("Version bumped: {} -> {}", current_version, args.version);
    println!(
        "Files modified: {}",
        1 + node_manifest_updates.len() + args.schema.as_ref().map(|s| s.len()).unwrap_or(0)
    );

    println!("\nNext steps:");
    println!("  1. Review changes: git diff");
    println!("  2. Update CHANGELOG.md with [{}] entry", args.version);
    println!(
        "  3. Commit: git commit -am \"chore: bump version to {}\"",
        args.version
    );
    println!("  4. Publish: cargo xtask publish --plan");

    Ok(())
}

#[derive(Debug, Clone)]
struct NodeManifestUpdate {
    path: String,
    updated_content: String,
    changes: Vec<String>,
}

/// Find the workspace root by looking for Cargo.toml with [workspace].
fn find_workspace_root() -> Result<std::path::PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!("Could not find workspace root (Cargo.toml with [workspace])");
        }
    }
}

/// Validate that a version string is valid semver.
fn validate_semver(version: &str) -> Result<()> {
    let parsed = semver::Version::parse(version)
        .with_context(|| format!("Version must be valid semver, got: {version}"))?;

    if !parsed.build.is_empty() {
        bail!("Version must not include build metadata for release bumps: {version}");
    }

    Ok(())
}

/// Extract the current workspace version from Cargo.toml content.
fn extract_workspace_version(content: &str) -> Result<String> {
    // Look for version = "x.y.z" in [workspace.package] section
    let mut in_workspace_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }
        if in_workspace_package && trimmed.starts_with('[') {
            break;
        }
        if in_workspace_package
            && trimmed.starts_with("version")
            && let Some(version) = extract_quoted_value(trimmed)
        {
            return Ok(version.to_string());
        }
    }
    bail!("Could not find version in [workspace.package]")
}

/// Extract a quoted value from a line like `key = "value"`.
fn extract_quoted_value(line: &str) -> Option<&str> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}

/// Update [workspace.package].version in Cargo.toml content.
fn update_workspace_package_version(content: &str, new_version: &str) -> Result<String> {
    let mut result = String::with_capacity(content.len());
    let mut in_workspace_package = false;
    let mut version_updated = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_workspace_package && trimmed.starts_with('[') {
            in_workspace_package = false;
        }

        if in_workspace_package && trimmed.starts_with("version") && !version_updated {
            // Replace the version line
            let indent = line.len() - line.trim_start().len();
            let spaces = &line[..indent];
            result.push_str(spaces);
            result.push_str(&format!("version = \"{}\"\n", new_version));
            version_updated = true;
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    if !version_updated {
        bail!("Failed to update [workspace.package].version");
    }

    Ok(result)
}

/// Update [workspace.dependencies] internal crate version fields.
fn update_workspace_dependencies(
    content: &str,
    new_version: &str,
) -> Result<(String, Vec<String>)> {
    let mut result = String::with_capacity(content.len());
    let mut changes = Vec::new();
    let mut in_workspace_deps = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[workspace.dependencies]" {
            in_workspace_deps = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_workspace_deps && trimmed.starts_with('[') && trimmed != "[workspace.dependencies]" {
            in_workspace_deps = false;
        }

        if in_workspace_deps && trimmed.starts_with("tokmd") {
            // Parse the crate name and check if it has a version field
            if let Some(crate_name) = trimmed.split(&['=', ' '][..]).next() {
                let crate_name = crate_name.trim();
                // Check if this line has a version field
                if let Some(old_version) = extract_version_from_dep_line(trimmed) {
                    // Replace the version
                    let new_line = replace_version_in_dep_line(line, &old_version, new_version);
                    changes.push(format!(
                        "Cargo.toml: {} version = \"{}\" -> \"{}\"",
                        crate_name, old_version, new_version
                    ));
                    result.push_str(&new_line);
                    result.push('\n');
                    continue;
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    Ok((result, changes))
}

/// Extract version from a dependency line like `foo = { path = "...", version = "1.0.0" }`.
fn extract_version_from_dep_line(line: &str) -> Option<String> {
    // Look for version = "x.y.z" pattern
    let version_start = line.find("version")?;
    let after_version = &line[version_start..];
    let quote_start = after_version.find('"')? + 1;
    let remaining = &after_version[quote_start..];
    let quote_end = remaining.find('"')?;
    Some(remaining[..quote_end].to_string())
}

/// Replace version in a dependency line.
fn replace_version_in_dep_line(line: &str, old_version: &str, new_version: &str) -> String {
    // Find and replace version = "old" with version = "new"
    let pattern = format!("version = \"{}\"", old_version);
    let replacement = format!("version = \"{}\"", new_version);
    line.replace(&pattern, &replacement)
}

fn plan_node_manifest_updates(
    workspace_root: &Path,
    new_version: &str,
) -> Result<Vec<NodeManifestUpdate>> {
    let mut updates = Vec::new();

    for path in NODE_PACKAGE_MANIFESTS {
        let manifest_path = workspace_root.join(path);
        let content =
            fs::read_to_string(&manifest_path).with_context(|| format!("Failed to read {path}"))?;
        let (updated_content, changes) = update_node_manifest_versions(&content, path, new_version)
            .with_context(|| format!("Failed to update {path}"))?;

        if !changes.is_empty() {
            updates.push(NodeManifestUpdate {
                path: path.to_string(),
                updated_content,
                changes,
            });
        }
    }

    Ok(updates)
}

fn update_node_manifest_versions(
    content: &str,
    path: &str,
    new_version: &str,
) -> Result<(String, Vec<String>)> {
    let mut result = String::with_capacity(content.len());
    let mut changes = Vec::new();
    let mut root_version_updated = false;
    let mut current_section: Option<&str> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(section) = current_section {
            if trimmed.starts_with('}') {
                current_section = None;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if trimmed.starts_with("\"@tokmd/") {
                let dependency_name = extract_json_key(trimmed)
                    .with_context(|| format!("Missing dependency name in {path}: {trimmed}"))?;
                let old_version = extract_json_string_value(trimmed)
                    .with_context(|| format!("Missing dependency version in {path}: {trimmed}"))?;

                if old_version != new_version {
                    changes.push(format!(
                        "{path}: {section}.{dependency_name} = \"{old_version}\" -> \"{new_version}\""
                    ));
                    result.push_str(&replace_json_string_value(line, new_version)?);
                    result.push('\n');
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
            continue;
        }

        if !root_version_updated && trimmed.starts_with("\"version\":") {
            let old_version = extract_json_string_value(trimmed)
                .with_context(|| format!("Missing top-level version in {path}: {trimmed}"))?;

            if old_version != new_version {
                changes.push(format!(
                    "{path}: version = \"{old_version}\" -> \"{new_version}\""
                ));
                result.push_str(&replace_json_string_value(line, new_version)?);
                result.push('\n');
                root_version_updated = true;
                continue;
            }

            root_version_updated = true;
        }

        if matches!(
            trimmed,
            "\"dependencies\": {" | "\"optionalDependencies\": {" | "\"peerDependencies\": {"
        ) {
            current_section = Some(extract_json_key(trimmed).with_context(|| {
                format!("Missing dependency section name in {path}: {trimmed}")
            })?);
        }

        result.push_str(line);
        result.push('\n');
    }

    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    if !root_version_updated {
        bail!("Failed to find top-level version in {path}");
    }

    let _: JsonValue = serde_json::from_str(&result)
        .with_context(|| format!("Updated {path} is not valid JSON"))?;

    Ok((result, changes))
}

fn extract_json_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix('"')?;
    let key_end = remainder.find('"')?;
    Some(&remainder[..key_end])
}

fn extract_json_string_value(line: &str) -> Option<String> {
    let colon_index = line.find(':')?;
    let value_part = &line[colon_index + 1..];
    let quote_start = value_part.find('"')? + 1;
    let remainder = &value_part[quote_start..];
    let quote_end = remainder.find('"')?;
    Some(remainder[..quote_end].to_string())
}

fn replace_json_string_value(line: &str, new_value: &str) -> Result<String> {
    let colon_index = line
        .find(':')
        .with_context(|| format!("Expected ':' in JSON line: {line}"))?;
    let value_start = line[colon_index + 1..]
        .find('"')
        .with_context(|| format!("Expected opening quote in JSON line: {line}"))?
        + colon_index
        + 2;
    let value_end = line[value_start..]
        .find('"')
        .with_context(|| format!("Expected closing quote in JSON line: {line}"))?
        + value_start;

    Ok(format!(
        "{}{}{}",
        &line[..value_start],
        new_value,
        &line[value_end..]
    ))
}

/// Parse a schema bump argument like "SCHEMA_VERSION=3" or "ANALYSIS_SCHEMA_VERSION=5".
fn parse_schema_bump(bump: &str) -> Result<(String, u32)> {
    let parts: Vec<&str> = bump.split('=').collect();
    if parts.len() != 2 {
        bail!("Schema bump must be in format NAME=VERSION, got: {}", bump);
    }
    let name = parts[0].trim().to_string();
    let version: u32 = parts[1]
        .trim()
        .parse()
        .with_context(|| format!("Invalid schema version number: {}", parts[1]))?;
    Ok((name, version))
}

/// Update a schema version constant in a source file.
fn update_schema_version(
    workspace_root: &Path,
    constant_name: &str,
    new_version: u32,
) -> Result<()> {
    // Find the location for this constant
    let location = SCHEMA_LOCATIONS
        .iter()
        .find(|l| l.constant == constant_name)
        .with_context(|| {
            format!(
                "Unknown schema constant: {}. Known constants: {}",
                constant_name,
                SCHEMA_LOCATIONS
                    .iter()
                    .map(|l| l.constant)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    let file_path = workspace_root.join(location.path);
    let content = fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read {}", location.path))?;

    // Find and replace the constant definition
    let pattern = format!("pub const {}: u32 = ", constant_name);
    let mut result = String::with_capacity(content.len());
    let mut found = false;

    for line in content.lines() {
        if line.trim_start().starts_with(&pattern) {
            // Replace this line
            let indent = line.len() - line.trim_start().len();
            let spaces = &line[..indent];
            result.push_str(spaces);
            result.push_str(&format!(
                "pub const {}: u32 = {};\n",
                constant_name, new_version
            ));
            found = true;
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    if !found {
        bail!(
            "Could not find 'pub const {}: u32 = ...' in {}",
            constant_name,
            location.path
        );
    }

    fs::write(&file_path, &result).with_context(|| format!("Failed to write {}", location.path))?;
    println!("Wrote: {}", file_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_semver_valid() {
        assert!(validate_semver("1.0.0").is_ok());
        assert!(validate_semver("0.1.0").is_ok());
        assert!(validate_semver("10.20.30").is_ok());
        assert!(validate_semver("1.8.0-rc.1").is_ok());
    }

    #[test]
    fn test_validate_semver_invalid() {
        assert!(validate_semver("1.0").is_err());
        assert!(validate_semver("1.0.0.0").is_err());
        assert!(validate_semver("1.0.a").is_err());
        assert!(validate_semver("").is_err());
        assert!(validate_semver("1.8.0-rc.1+build.5").is_err());
    }

    #[test]
    fn test_extract_workspace_version() {
        let content = r#"
[workspace]
members = ["crates/foo"]

[workspace.package]
version = "1.2.3"
edition = "2021"
"#;
        assert_eq!(
            extract_workspace_version(content).expect("Should extract valid workspace version"),
            "1.2.3"
        );
    }

    #[test]
    fn test_update_workspace_package_version() {
        let content = r#"[workspace.package]
version = "1.2.3"
edition = "2021"
"#;
        let result = update_workspace_package_version(content, "1.3.0")
            .expect("Should update valid workspace version");
        assert!(result.contains("version = \"1.3.0\""));
        assert!(!result.contains("version = \"1.2.3\""));
    }

    #[test]
    fn test_extract_version_from_dep_line() {
        let line = r#"tokmd-types = { path = "crates/tokmd-types", version = "1.2.3" }"#;
        assert_eq!(
            extract_version_from_dep_line(line),
            Some("1.2.3".to_string())
        );

        let line_no_version = r#"tokmd-types = { path = "crates/tokmd-types" }"#;
        assert_eq!(extract_version_from_dep_line(line_no_version), None);
    }

    #[test]
    fn test_replace_version_in_dep_line() {
        let line = r#"tokmd-types = { path = "crates/tokmd-types", version = "1.2.3" }"#;
        let result = replace_version_in_dep_line(line, "1.2.3", "1.3.0");
        assert_eq!(
            result,
            r#"tokmd-types = { path = "crates/tokmd-types", version = "1.3.0" }"#
        );
    }

    #[test]
    fn test_parse_schema_bump() {
        let (name, version) =
            parse_schema_bump("SCHEMA_VERSION=3").expect("Should parse valid schema bump");
        assert_eq!(name, "SCHEMA_VERSION");
        assert_eq!(version, 3);

        let (name, version) = parse_schema_bump("ANALYSIS_SCHEMA_VERSION = 5")
            .expect("Should parse valid schema bump with spaces");
        assert_eq!(name, "ANALYSIS_SCHEMA_VERSION");
        assert_eq!(version, 5);
    }

    #[test]
    fn test_parse_schema_bump_invalid() {
        assert!(parse_schema_bump("SCHEMA_VERSION").is_err());
        assert!(parse_schema_bump("SCHEMA_VERSION=abc").is_err());
    }

    #[test]
    fn updates_node_manifest_versions_in_place() {
        let content = r#"{
  "name": "@tokmd/core",
  "version": "1.8.1",
  "optionalDependencies": {
    "@tokmd/core-win32-x64-msvc": "1.8.1",
    "@tokmd/core-linux-x64-gnu": "1.8.1",
    "chalk": "^5.0.0"
  }
}"#;

        let (updated, changes) =
            update_node_manifest_versions(content, "crates/tokmd-node/package.json", "1.9.0")
                .expect("node manifest should update");

        assert!(updated.contains("\"version\": \"1.9.0\""));
        assert!(updated.contains("\"@tokmd/core-win32-x64-msvc\": \"1.9.0\""));
        assert!(updated.contains("\"@tokmd/core-linux-x64-gnu\": \"1.9.0\""));
        assert!(updated.contains("\"chalk\": \"^5.0.0\""));
        assert_eq!(changes.len(), 3);
    }

    #[test]
    fn replace_json_string_value_preserves_json_line_shape() {
        let line = r#"    "@tokmd/core-linux-x64-gnu": "1.8.1","#;
        let updated = replace_json_string_value(line, "1.9.0").expect("value should replace");
        assert_eq!(updated, r#"    "@tokmd/core-linux-x64-gnu": "1.9.0","#);
    }
}
