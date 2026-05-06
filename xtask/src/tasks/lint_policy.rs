use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use toml::Value;

use crate::cli::LintPolicyArgs;

const ROOT_MANIFEST: &str = "Cargo.toml";
const POLICY_FILE: &str = "policy/clippy-lints.toml";
const DEBT_FILE: &str = "policy/clippy-debt.toml";
const CLIPPY_FILE: &str = "clippy.toml";
const FORBIDDEN_TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

#[derive(Debug, Deserialize)]
struct LintPolicy {
    schema: u64,
    msrv: String,
    policy: PolicySettings,
    #[serde(default)]
    lint: Vec<LintEntry>,
    #[serde(default)]
    planned: Vec<PlannedLint>,
}

#[derive(Debug, Deserialize)]
struct PolicySettings {
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
    #[serde(rename = "class")]
    lint_class: String,
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
struct DebtLedger {
    schema: u64,
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
    let root = read_toml(ROOT_MANIFEST)?;
    let policy: LintPolicy = toml::from_str(&fs::read_to_string(POLICY_FILE)?)
        .with_context(|| format!("failed to parse {POLICY_FILE}"))?;

    let mut failures = Vec::new();
    check_policy_shape(&policy, &mut failures);
    check_msrv(&root, &policy, &mut failures);
    check_active_lints(&root, &policy, &mut failures);
    check_member_inheritance(&root, &mut failures)?;
    check_clippy_toml(&mut failures)?;
    check_debt(&mut failures)?;

    if failures.is_empty() {
        println!(
            "lint policy ok: {} active lints, {} planned flips",
            policy.lint.len(),
            policy.planned.len()
        );
        return Ok(());
    }

    for failure in &failures {
        eprintln!("lint policy violation: {failure}");
    }
    bail!(
        "lint policy check failed with {} violation(s)",
        failures.len()
    )
}

fn read_toml(path: impl AsRef<Path>) -> Result<Value> {
    let path = path.as_ref();
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn check_policy_shape(policy: &LintPolicy, failures: &mut Vec<String>) {
    if policy.schema != 1 {
        failures.push(format!("{POLICY_FILE} schema must be 1"));
    }
    if !policy.policy.panic_free_tests {
        failures.push("policy.panic_free_tests must be true".to_string());
    }
    if policy.policy.allow_test_carveouts {
        failures.push("policy.allow_test_carveouts must be false".to_string());
    }
    if policy.policy.suppression_style != "expect-with-reason" {
        failures.push("policy.suppression_style must be expect-with-reason".to_string());
    }
    if policy.policy.blanket_categories {
        failures.push("policy.blanket_categories must be false".to_string());
    }

    for lint in &policy.lint {
        if lint.name.trim().is_empty()
            || lint.level.trim().is_empty()
            || lint.status.trim().is_empty()
            || lint.lint_class.trim().is_empty()
            || lint.reason.trim().is_empty()
        {
            failures.push(format!(
                "active lint entry for {:?} has an empty required field",
                lint.name
            ));
        }
        if lint.status != "active" {
            failures.push(format!("{} must have status = active", lint.name));
        }
    }
    for planned in &policy.planned {
        if planned.name.trim().is_empty()
            || planned.level.trim().is_empty()
            || planned.activate_when_msrv.trim().is_empty()
            || planned.reason.trim().is_empty()
        {
            failures.push(format!(
                "planned lint entry for {:?} has an empty required field",
                planned.name
            ));
        }
    }
}

fn check_msrv(root: &Value, policy: &LintPolicy, failures: &mut Vec<String>) {
    let Some(msrv) = root
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("rust-version"))
        .and_then(Value::as_str)
    else {
        failures.push("workspace.package.rust-version is missing".to_string());
        return;
    };
    if msrv != policy.msrv {
        failures.push(format!(
            "workspace.package.rust-version ({msrv}) must match {POLICY_FILE} msrv ({})",
            policy.msrv
        ));
    }
}

fn check_active_lints(root: &Value, policy: &LintPolicy, failures: &mut Vec<String>) {
    let Some(workspace_lints) = root
        .get("workspace")
        .and_then(|workspace| workspace.get("lints"))
        .and_then(Value::as_table)
    else {
        failures.push("root Cargo.toml must define [workspace.lints]".to_string());
        return;
    };

    for lint in &policy.lint {
        let Some((tool, name)) = lint.name.split_once("::") else {
            failures.push(format!("{} must use tool::lint_name syntax", lint.name));
            continue;
        };
        let configured = workspace_lints
            .get(tool)
            .and_then(Value::as_table)
            .and_then(|table| table.get(name))
            .and_then(Value::as_str);
        if configured != Some(lint.level.as_str()) {
            failures.push(format!(
                "{} is {} in {POLICY_FILE} but {:?} in root Cargo.toml",
                lint.name, lint.level, configured
            ));
        }
    }

    for planned in &policy.planned {
        let Some((tool, name)) = planned.name.split_once("::") else {
            failures.push(format!("{} must use tool::lint_name syntax", planned.name));
            continue;
        };
        let configured = workspace_lints
            .get(tool)
            .and_then(Value::as_table)
            .and_then(|table| table.get(name));
        if configured.is_some() && policy.msrv.as_str() < planned.activate_when_msrv.as_str() {
            failures.push(format!(
                "{} is planned for MSRV {} but is already active at MSRV {}",
                planned.name, planned.activate_when_msrv, policy.msrv
            ));
        }
    }
}

fn check_member_inheritance(root: &Value, failures: &mut Vec<String>) -> Result<()> {
    let members = root
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("workspace.members missing from {ROOT_MANIFEST}"))?;

    for member in members {
        let Some(member) = member.as_str() else {
            failures.push("workspace member must be a string".to_string());
            continue;
        };
        for manifest in manifests_for_member(member)? {
            let value = read_toml(&manifest)?;
            let inherits = value
                .get("lints")
                .and_then(|lints| lints.get("workspace"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !inherits {
                failures.push(format!(
                    "{} must contain [lints] workspace = true",
                    manifest.display()
                ));
            }
        }
    }
    Ok(())
}

fn manifests_for_member(member: &str) -> Result<Vec<PathBuf>> {
    if member.contains('*') {
        let pattern = format!("{member}/Cargo.toml");
        let paths = globset::Glob::new(&pattern)?.compile_matcher();
        let mut manifests = Vec::new();
        for path in fs::read_dir(".")? {
            let path = path?.path();
            if paths.is_match(&path) {
                manifests.push(path);
            }
        }
        return Ok(manifests);
    }
    Ok(vec![Path::new(member).join("Cargo.toml")])
}

fn check_clippy_toml(failures: &mut Vec<String>) -> Result<()> {
    let path = Path::new(CLIPPY_FILE);
    if !path.exists() {
        failures.push(format!("{CLIPPY_FILE} must exist"));
        return Ok(());
    }
    let value = read_toml(path)?;
    for key in FORBIDDEN_TEST_CARVEOUTS {
        if value.get(*key).is_some() {
            failures.push(format!("{CLIPPY_FILE} must not set {key}"));
        }
    }
    Ok(())
}

fn check_debt(failures: &mut Vec<String>) -> Result<()> {
    let raw =
        fs::read_to_string(DEBT_FILE).with_context(|| format!("failed to read {DEBT_FILE}"))?;
    let debt: DebtLedger =
        toml::from_str(&raw).with_context(|| format!("failed to parse {DEBT_FILE}"))?;
    if debt.schema != 1 {
        failures.push(format!("{DEBT_FILE} schema must be 1"));
    }

    let today = Utc::now().date_naive();
    for entry in debt.debt {
        if entry.lint.trim().is_empty()
            || entry.path.trim().is_empty()
            || entry.owner.trim().is_empty()
            || entry.reason.trim().is_empty()
            || entry.expires.trim().is_empty()
        {
            failures.push("clippy debt entry has an empty required field".to_string());
        }
        match NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d") {
            Ok(expires) if expires < today => failures.push(format!(
                "clippy debt {} at {} expired on {}",
                entry.lint, entry.path, entry.expires
            )),
            Ok(_) => {}
            Err(_) => failures.push(format!(
                "clippy debt {} at {} has invalid expires date {}",
                entry.lint, entry.path, entry.expires
            )),
        }
    }
    Ok(())
}
