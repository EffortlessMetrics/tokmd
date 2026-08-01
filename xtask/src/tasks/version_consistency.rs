use std::borrow::Cow;

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
    check_msrv_pins(&workspace_root)?;
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

fn check_msrv_pins(workspace_root: &Path) -> Result<()> {
    let msrv = load_workspace_rust_version(workspace_root)?;
    let mut mismatches = Vec::new();
    let mut checked = 0usize;

    for path in msrv_pin_sources(workspace_root)? {
        let contents = fs::read_to_string(workspace_root.join(&path))
            .with_context(|| format!("Failed to read {path}"))?;

        for pin in extract_toolchain_pins(&contents) {
            checked += 1;
            if !toolchain_matches_msrv(&pin.version, &msrv) {
                mismatches.push(format!(
                    "{path}:{} pins Rust {} but workspace MSRV is {msrv}",
                    pin.line, pin.version
                ));
            }
        }
    }

    if !mismatches.is_empty() {
        bail!(
            "Rust toolchain pins are out of sync with the workspace MSRV:\n  {}\n\
             Update the pins above (or bump [workspace.package].rust-version).",
            mismatches.join("\n  ")
        );
    }

    if checked == 0 {
        bail!(
            "No concrete Rust toolchain pin found in the Dockerfile or any workflow. \
             The MSRV is then unproven; restore a pin or remove this check."
        );
    }

    println!("  ✓ {checked} Rust toolchain pin(s) match MSRV {msrv}.");
    Ok(())
}

/// Every file that could carry a concrete Rust toolchain pin.
///
/// Discovered rather than hard-coded: a fixed list silently stops covering a
/// workflow added later, which is the same drift this check exists to catch.
/// Files with no concrete pin simply contribute nothing.
fn msrv_pin_sources(workspace_root: &Path) -> Result<Vec<String>> {
    let mut sources = vec!["Dockerfile".to_string()];

    let workflows = workspace_root.join(".github/workflows");
    let entries = fs::read_dir(&workflows)
        .with_context(|| format!("Failed to read {}", workflows.display()))?;

    let mut discovered = Vec::new();
    for entry in entries {
        let path = entry?.path();
        let is_workflow = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "yml" || ext == "yaml");
        if !is_workflow {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            discovered.push(format!(".github/workflows/{name}"));
        }
    }

    // Sorted so error output and the checked count are stable across platforms.
    discovered.sort();
    sources.extend(discovered);
    Ok(sources)
}

/// A concrete Rust toolchain version pinned by a build or CI file.
struct ToolchainPin {
    /// 1-based line number, for error messages that point at the edit site.
    line: usize,
    version: String,
}

/// Extracts concrete toolchain pins from a Dockerfile or GitHub workflow.
///
/// Recognizes `FROM rust:<version>...` and `toolchain: <version>`. Channel names
/// (`stable`, `nightly`, `beta`) and `${{ ... }}` expressions are intentionally
/// skipped: they float by design and cannot drift against a fixed MSRV.
///
/// A trailing YAML comment is stripped before the value is unquoted, so the
/// idiomatic `toolchain: "1.95" # MSRV` reads as `1.95`. Order matters here:
/// unquoting first would leave `1.95" # MSRV`, because `trim_matches` only
/// strips from the ends and the line no longer ends in a quote.
fn extract_toolchain_pins(contents: &str) -> Vec<ToolchainPin> {
    let mut pins = Vec::new();

    for (index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.starts_with('#') {
            continue;
        }

        let candidate = line
            // `rust:1.95-alpine` / `rust:1.95.0-slim` -> `1.95` / `1.95.0`
            .strip_prefix("FROM rust:")
            .and_then(|rest| rest.split(['-', ' ']).next())
            .or_else(|| {
                line.strip_prefix("toolchain:").map(|rest| {
                    let uncommented = rest.split('#').next().unwrap_or(rest);
                    uncommented.trim().trim_matches(['"', '\''])
                })
            });

        let Some(candidate) = candidate.map(str::trim) else {
            continue;
        };

        if !candidate.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }

        pins.push(ToolchainPin {
            line: index + 1,
            version: candidate.to_string(),
        });
    }

    pins
}

/// A pin satisfies the MSRV when it names the same release, at any precision the
/// MSRV does not constrain.
///
/// The comparison is a component-wise prefix match, so an MSRV of `1.95` accepts
/// `1.95`, `1.95.0`, and `1.95.1` — all of them *are* Rust 1.95, and the patch
/// component is not something the MSRV pins down.
///
/// A different minor is rejected in both directions. `1.94` cannot build an
/// edition-2024 workspace that requires 1.95; `1.96` builds fine but means CI has
/// quietly stopped proving the MSRV, which is how an MSRV claim rots without
/// anything going red.
///
/// A pin less precise than the MSRV (`1` against `1.95`) is rejected: it does not
/// name the release the MSRV requires.
fn toolchain_matches_msrv(pin: &str, msrv: &str) -> bool {
    fn components(value: &str) -> Vec<&str> {
        value.split('.').take(3).collect()
    }

    let pin_parts = components(pin);
    let msrv_parts = components(msrv);

    // Compare only the components the MSRV actually specifies, so an MSRV of
    // `1.95` accepts a `1.95.0` pin but rejects `1.96.0`.
    if pin_parts.len() < msrv_parts.len() {
        return false;
    }

    pin_parts
        .iter()
        .zip(msrv_parts.iter())
        .all(|(left, right)| left == right)
}

fn load_workspace_rust_version(workspace_root: &Path) -> Result<String> {
    let manifest = read_toml(&workspace_root.join("Cargo.toml"))?;
    let version = manifest
        .get("workspace")
        .and_then(TomlValue::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(TomlValue::as_table)
        .and_then(|package| package.get("rust-version"))
        .and_then(TomlValue::as_str)
        .context("Missing [workspace.package].rust-version in root Cargo.toml")?;

    Ok(version.to_string())
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

fn to_lowercase_cow(s: &str) -> Cow<'_, str> {
    if s.chars().any(|c| c.is_uppercase()) {
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
    fn test_extract_toolchain_pins_reads_dockerfile_and_workflows() {
        let dockerfile = "# syntax=docker/dockerfile:1\nFROM rust:1.95-alpine AS builder\nFROM alpine:3.21 AS runtime\n";
        let pins = extract_toolchain_pins(dockerfile);
        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0].version, "1.95");
        assert_eq!(pins[0].line, 2);

        let workflow = "      - uses: dtolnay/rust-toolchain@stable\n        with:\n          toolchain: \"1.95\"\n          toolchain: 1.95.0\n";
        let pins = extract_toolchain_pins(workflow);
        assert_eq!(
            pins.iter().map(|p| p.version.as_str()).collect::<Vec<_>>(),
            vec!["1.95", "1.95.0"]
        );
    }

    #[test]
    fn test_extract_toolchain_pins_skips_floating_channels_and_comments() {
        let workflow = "          toolchain: stable\n          toolchain: nightly\n          toolchain: ${{ matrix.rust }}\n#          toolchain: 1.90\n";
        assert!(extract_toolchain_pins(workflow).is_empty());
    }

    #[test]
    // An explanatory comment after the pin is idiomatic. Misreading it as part
    // of the version would make the guard bail on a perfectly correct pin.
    fn test_extract_toolchain_pins_strips_inline_comments() {
        let quoted = "          toolchain: \"1.95\" # MSRV, see Cargo.toml\n";
        let bare = "          toolchain: 1.95 # MSRV\n";

        for workflow in [quoted, bare] {
            let pins = extract_toolchain_pins(workflow);
            assert_eq!(pins.len(), 1, "{workflow}");
            assert_eq!(pins[0].version, "1.95", "{workflow}");
        }
    }

    /// Builds a throwaway workspace so the drift path can be exercised without
    /// editing the real Dockerfile.
    fn write_msrv_fixture(root: &Path, msrv: &str, dockerfile_pin: &str) -> Result<()> {
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace]\n[workspace.package]\nrust-version = \"{msrv}\"\n"),
        )?;
        fs::write(
            root.join("Dockerfile"),
            format!("FROM rust:{dockerfile_pin}-alpine AS builder\n"),
        )?;
        fs::create_dir_all(root.join(".github/workflows"))?;
        fs::write(
            root.join(".github/workflows/ci.yml"),
            format!("        with:\n          toolchain: {msrv}\n"),
        )?;
        Ok(())
    }

    #[test]
    // The red half of red/green: prove the guard actually fails on drift, and
    // that the message names the file and line to edit.
    fn test_check_msrv_pins_bails_on_drifted_pin() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_msrv_fixture(temp.path(), "1.95", "1.92")?;

        let Err(error) = check_msrv_pins(temp.path()) else {
            bail!("a Dockerfile pinned below the MSRV should fail the check");
        };

        let message = error.to_string();
        assert!(
            message.contains("Dockerfile:1 pins Rust 1.92 but workspace MSRV is 1.95"),
            "{message}"
        );
        Ok(())
    }

    #[test]
    // Pinning above the MSRV is the quieter failure: it builds, but CI has
    // stopped proving the MSRV.
    fn test_check_msrv_pins_bails_on_pin_above_msrv() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_msrv_fixture(temp.path(), "1.95", "1.96")?;

        let Err(error) = check_msrv_pins(temp.path()) else {
            bail!("a Dockerfile pinned above the MSRV should fail the check");
        };
        assert!(error.to_string().contains("pins Rust 1.96"), "{error}");
        Ok(())
    }

    #[test]
    fn test_check_msrv_pins_accepts_matching_fixture() -> Result<()> {
        let temp = tempfile::tempdir()?;
        write_msrv_fixture(temp.path(), "1.95", "1.95")?;
        check_msrv_pins(temp.path())
    }

    #[test]
    // A repo with no concrete pin anywhere cannot be proving its MSRV, so
    // silently reporting success would be the wrong answer.
    fn test_check_msrv_pins_bails_when_nothing_is_pinned() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\n[workspace.package]\nrust-version = \"1.95\"\n",
        )?;
        fs::write(root.join("Dockerfile"), "FROM alpine:3.21\n")?;
        fs::create_dir_all(root.join(".github/workflows"))?;
        fs::write(
            root.join(".github/workflows/ci.yml"),
            "          toolchain: stable\n",
        )?;

        let Err(error) = check_msrv_pins(root) else {
            bail!("a repo with no concrete pin should fail the check");
        };
        assert!(
            error.to_string().contains("No concrete Rust toolchain pin"),
            "{error}"
        );
        Ok(())
    }

    #[test]
    fn test_toolchain_matches_msrv() {
        // Patch precision the MSRV does not constrain is still Rust 1.95.
        assert!(toolchain_matches_msrv("1.95", "1.95"));
        assert!(toolchain_matches_msrv("1.95.0", "1.95"));
        assert!(toolchain_matches_msrv("1.95.1", "1.95"));

        // Below the MSRV cannot build; above it stops proving the MSRV.
        assert!(!toolchain_matches_msrv("1.92", "1.95"));
        assert!(!toolchain_matches_msrv("1.96", "1.95"));
        // A less specific pin cannot satisfy a more specific MSRV.
        assert!(!toolchain_matches_msrv("1", "1.95"));
        // A more specific MSRV does constrain the patch component.
        assert!(toolchain_matches_msrv("1.95.1", "1.95.1"));
        assert!(!toolchain_matches_msrv("1.95.0", "1.95.1"));
    }

    #[test]
    // Errors propagate with `?` rather than `expect` so this test adds no
    // panic-family debt to policy/no-panic-allowlist.toml.
    fn test_msrv_pins_match_workspace_rust_version() -> Result<()> {
        let workspace_root = find_workspace_root()?;
        check_msrv_pins(&workspace_root)?;
        Ok(())
    }

    #[test]
    // Discovery, not a fixed list: a workflow added later must be covered
    // automatically, otherwise the check stops seeing exactly the drift it
    // exists to catch.
    fn test_msrv_pin_sources_cover_every_workflow() -> Result<()> {
        let workspace_root = find_workspace_root()?;
        let sources = msrv_pin_sources(&workspace_root)?;

        assert!(sources.contains(&"Dockerfile".to_string()));

        let mut workflow_count = 0usize;
        for entry in fs::read_dir(workspace_root.join(".github/workflows"))? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
                workflow_count += 1;
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .context("workflow file name should be UTF-8")?;
                let expected = format!(".github/workflows/{name}");
                assert!(
                    sources.contains(&expected),
                    "{expected} is not covered by msrv_pin_sources"
                );
            }
        }
        assert!(workflow_count > 1, "expected multiple workflows to exist");

        Ok(())
    }

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
