use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::MetadataCommand;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use toml::Value;

use crate::cli::LintPolicyArgs;

const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

#[derive(Debug, Deserialize)]
struct LintPolicyFile {
    schema: u32,
    msrv: String,
    policy: PolicyPosture,
    #[serde(default)]
    lint: Vec<LintEntry>,
    #[serde(default)]
    planned: Vec<PlannedLint>,
}

#[derive(Debug, Deserialize)]
struct PolicyPosture {
    panic_free_tests: bool,
    allow_test_carveouts: bool,
    suppression_style: String,
    blanket_categories: bool,
}

#[derive(Debug, Deserialize)]
struct LintEntry {
    name: String,
    level: String,
    status: String,
    class: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct PlannedLint {
    name: String,
    level: String,
    activate_when_msrv: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct DebtFile {
    schema: u32,
    #[serde(default)]
    debt: Vec<DebtEntry>,
}

#[derive(Debug, Deserialize)]
struct DebtEntry {
    lint: String,
    path: String,
    owner: String,
    reason: String,
    expires: String,
}

pub fn run(_args: LintPolicyArgs) -> Result<()> {
    let root = workspace_root()?;
    let mut errors = Vec::new();

    let cargo_toml = read_toml(&root.join("Cargo.toml"), &mut errors)?;
    let policy = read_policy(&root.join("policy/clippy-lints.toml"), &mut errors)?;

    validate_policy_shape(&policy, &mut errors);
    validate_msrv(&cargo_toml, &policy, &mut errors);
    validate_workspace_lints(&cargo_toml, &policy, &mut errors);
    validate_workspace_members_do_not_override_lints(&root, &mut errors);
    validate_clippy_toml(&root.join("clippy.toml"), &mut errors);
    validate_planned_lints(&cargo_toml, &policy, &mut errors);
    validate_debt(&root.join("policy/clippy-debt.toml"), &mut errors);

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("lint policy error: {error}");
        }
        bail!("lint policy check failed with {} error(s)", errors.len());
    }

    println!(
        "lint policy ok: {} active lints, {} planned lints, MSRV {}",
        policy.lint.len(),
        policy.planned.len(),
        policy.msrv
    );
    Ok(())
}

fn metadata_command() -> MetadataCommand {
    let mut command = MetadataCommand::new();
    command.no_deps();
    command
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = metadata_command().exec().context("load cargo metadata")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn read_toml(path: &Path, errors: &mut Vec<String>) -> Result<Value> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            errors.push(format!("{} is unreadable: {err}", path.display()));
            String::new()
        }
    };
    if content.is_empty() {
        return Ok(Value::Table(Default::default()));
    }
    match toml::from_str::<Value>(&content) {
        Ok(value) => Ok(value),
        Err(err) => {
            errors.push(format!("{} is invalid TOML: {err}", path.display()));
            Ok(Value::Table(Default::default()))
        }
    }
}

fn read_policy(path: &Path, errors: &mut Vec<String>) -> Result<LintPolicyFile> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&content).map_err(|err| {
        errors.push(format!("{} is invalid policy TOML: {err}", path.display()));
        anyhow!("invalid lint policy")
    })
}

fn validate_policy_shape(policy: &LintPolicyFile, errors: &mut Vec<String>) {
    if policy.schema != 1 {
        errors.push(format!("policy schema must be 1, got {}", policy.schema));
    }
    if !policy.policy.panic_free_tests {
        errors.push("policy.panic_free_tests must be true".to_string());
    }
    if policy.policy.allow_test_carveouts {
        errors.push("policy.allow_test_carveouts must be false".to_string());
    }
    if policy.policy.suppression_style != "expect-with-reason" {
        errors.push("policy.suppression_style must be expect-with-reason".to_string());
    }
    if policy.policy.blanket_categories {
        errors.push("policy.blanket_categories must be false".to_string());
    }

    let mut seen = BTreeSet::new();
    for lint in &policy.lint {
        if !seen.insert(&lint.name) {
            errors.push(format!("duplicate active lint {}", lint.name));
        }
        require_non_empty(&lint.name, "lint.name", errors);
        require_non_empty(&lint.level, "lint.level", errors);
        require_non_empty(&lint.status, "lint.status", errors);
        require_non_empty(&lint.class, "lint.class", errors);
        require_non_empty(&lint.reason, "lint.reason", errors);
        if lint.status != "active" {
            errors.push(format!("lint {} must have status = active", lint.name));
        }
    }

    let mut planned_seen = BTreeSet::new();
    for planned in &policy.planned {
        if !planned_seen.insert(&planned.name) {
            errors.push(format!("duplicate planned lint {}", planned.name));
        }
        require_non_empty(&planned.name, "planned.name", errors);
        require_non_empty(&planned.level, "planned.level", errors);
        require_non_empty(
            &planned.activate_when_msrv,
            "planned.activate_when_msrv",
            errors,
        );
        require_non_empty(&planned.reason, "planned.reason", errors);
    }
}

fn validate_msrv(cargo_toml: &Value, policy: &LintPolicyFile, errors: &mut Vec<String>) {
    let rust_version = cargo_toml
        .get("workspace")
        .and_then(|v| v.get("package"))
        .and_then(|v| v.get("rust-version"))
        .and_then(Value::as_str);
    if rust_version != Some(policy.msrv.as_str()) {
        errors.push(format!(
            "workspace.package.rust-version ({rust_version:?}) must equal policy msrv ({})",
            policy.msrv
        ));
    }
}

fn validate_workspace_lints(cargo_toml: &Value, policy: &LintPolicyFile, errors: &mut Vec<String>) {
    let rust_lints = cargo_toml
        .get("workspace")
        .and_then(|v| v.get("lints"))
        .and_then(|v| v.get("rust"));
    let clippy_lints = cargo_toml
        .get("workspace")
        .and_then(|v| v.get("lints"))
        .and_then(|v| v.get("clippy"));

    let active: BTreeMap<&str, &str> = policy
        .lint
        .iter()
        .map(|lint| (lint.name.as_str(), lint.level.as_str()))
        .collect();

    for (name, level) in active {
        let (table, key) = if let Some(key) = name.strip_prefix("rust::") {
            (rust_lints, key)
        } else if let Some(key) = name.strip_prefix("clippy::") {
            (clippy_lints, key)
        } else {
            errors.push(format!("lint {name} must start with rust:: or clippy::"));
            continue;
        };
        let actual = table.and_then(|v| v.get(key)).and_then(Value::as_str);
        if actual != Some(level) {
            errors.push(format!(
                "workspace lint {name} must be {level:?}, got {actual:?}"
            ));
        }
    }
}

fn validate_workspace_members_do_not_override_lints(root: &Path, errors: &mut Vec<String>) {
    let metadata = match metadata_command().current_dir(root).exec() {
        Ok(metadata) => metadata,
        Err(err) => {
            errors.push(format!("could not load cargo metadata: {err}"));
            return;
        }
    };
    let root_manifest = root.join("Cargo.toml");
    for package in metadata.packages {
        let manifest = package.manifest_path.into_std_path_buf();
        if manifest == root_manifest {
            continue;
        }
        let Ok(content) = fs::read_to_string(&manifest) else {
            errors.push(format!("{} is unreadable", manifest.display()));
            continue;
        };
        let Ok(value) = toml::from_str::<Value>(&content) else {
            errors.push(format!("{} is invalid TOML", manifest.display()));
            continue;
        };

        if let Some(lints) = value.get("lints") {
            let inherits = lints.get("workspace").and_then(Value::as_bool) == Some(true);
            if !inherits {
                errors.push(format!(
                    "{} must not define repo-local lint overrides; use [lints] workspace = true when enabling inheritance",
                    manifest.display()
                ));
            }
        }
    }
}

fn validate_clippy_toml(path: &Path, errors: &mut Vec<String>) {
    if !path.exists() {
        errors.push(format!("{} must exist", path.display()));
        return;
    }
    let value = match read_toml(path, errors) {
        Ok(value) => value,
        Err(err) => {
            errors.push(format!("{} could not be read: {err}", path.display()));
            return;
        }
    };
    for key in TEST_CARVEOUTS {
        if value.get(*key).and_then(Value::as_bool) == Some(true) {
            errors.push(format!("clippy.toml must not enable {key}"));
        }
    }
}

fn validate_planned_lints(cargo_toml: &Value, policy: &LintPolicyFile, errors: &mut Vec<String>) {
    let rust_version = cargo_toml
        .get("workspace")
        .and_then(|v| v.get("package"))
        .and_then(|v| v.get("rust-version"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let active = cargo_toml
        .get("workspace")
        .and_then(|v| v.get("lints"))
        .and_then(|v| v.get("clippy"));
    for planned in &policy.planned {
        if planned.activate_when_msrv.as_str() > rust_version {
            let key = planned.name.trim_start_matches("clippy::");
            if active.and_then(|v| v.get(key)).is_some() {
                errors.push(format!(
                    "planned lint {} is active before MSRV {}",
                    planned.name, planned.activate_when_msrv
                ));
            }
        }
    }
}

fn validate_debt(path: &Path, errors: &mut Vec<String>) {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            errors.push(format!("{} is unreadable: {err}", path.display()));
            return;
        }
    };
    let debt_file = match toml::from_str::<DebtFile>(&content) {
        Ok(debt_file) => debt_file,
        Err(err) => {
            errors.push(format!("{} is invalid TOML: {err}", path.display()));
            return;
        }
    };
    if debt_file.schema != 1 {
        errors.push(format!("debt schema must be 1, got {}", debt_file.schema));
    }
    let today = Utc::now().date_naive();
    for debt in debt_file.debt {
        require_non_empty(&debt.lint, "debt.lint", errors);
        require_non_empty(&debt.path, "debt.path", errors);
        require_non_empty(&debt.owner, "debt.owner", errors);
        require_non_empty(&debt.reason, "debt.reason", errors);
        require_non_empty(&debt.expires, "debt.expires", errors);
        match NaiveDate::parse_from_str(&debt.expires, "%Y-%m-%d") {
            Ok(expires) if expires < today => errors.push(format!(
                "debt {} at {} expired on {}",
                debt.lint, debt.path, debt.expires
            )),
            Ok(_) => {}
            Err(err) => errors.push(format!(
                "debt {} at {} has invalid expires date {}: {err}",
                debt.lint, debt.path, debt.expires
            )),
        }
    }
}

fn require_non_empty(value: &str, field: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field} must not be empty"));
    }
}
