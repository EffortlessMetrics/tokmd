use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, Utc};
use semver::Version;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::CheckLintPolicyArgs;

const ROOT_MANIFEST: &str = "Cargo.toml";
const LINT_LEDGER: &str = "policy/clippy-lints.toml";
const LINT_DEBT: &str = "policy/clippy-debt.toml";
const CLIPPY_CONFIG: &str = "clippy.toml";

const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

#[derive(Debug, Deserialize)]
struct LintLedger {
    schema: u64,
    msrv: String,
    policy: LintPolicy,
    #[serde(default)]
    lint: Vec<LintEntry>,
}

#[derive(Debug, Deserialize)]
struct LintPolicy {
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
    #[serde(default)]
    activate_when_msrv: Option<String>,
    #[serde(rename = "class")]
    class_name: String,
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

pub fn run(_args: CheckLintPolicyArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("failed to determine repository root")?;
    let root_manifest = load_toml_value(&repo_root.join(ROOT_MANIFEST))?;
    let ledger: LintLedger = load_toml(&repo_root.join(LINT_LEDGER))?;
    let debt: DebtLedger = load_toml(&repo_root.join(LINT_DEBT))?;

    let mut violations = Vec::new();

    validate_ledger_shape(&ledger, &mut violations);
    validate_msrv(&root_manifest, &ledger, &mut violations);
    validate_workspace_lints(&root_manifest, &ledger, &mut violations);
    validate_lint_inheritance(&repo_root, &root_manifest, &mut violations)?;
    validate_clippy_config(&repo_root.join(CLIPPY_CONFIG), &mut violations)?;
    validate_debt(&debt, &mut violations);

    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("::error::{}", violation);
        }
        bail!(
            "lint policy check failed with {} violation(s)",
            violations.len()
        );
    }

    println!(
        "Lint policy OK: {} active lint(s), {} staged lint(s), {} planned lint(s), {} debt entry/entries",
        ledger
            .lint
            .iter()
            .filter(|entry| entry.status == "active")
            .count(),
        ledger
            .lint
            .iter()
            .filter(|entry| entry.status == "staged")
            .count(),
        ledger
            .lint
            .iter()
            .filter(|entry| entry.status == "planned")
            .count(),
        debt.debt.len()
    );
    Ok(())
}

fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

fn load_toml_value(path: &Path) -> Result<toml::Value> {
    load_toml(path)
}

fn validate_ledger_shape(ledger: &LintLedger, violations: &mut Vec<String>) {
    if ledger.schema != 1 {
        violations.push(format!("{LINT_LEDGER}: schema must be 1"));
    }
    if !ledger.policy.panic_free_tests {
        violations.push(format!(
            "{LINT_LEDGER}: policy.panic_free_tests must be true"
        ));
    }
    if ledger.policy.allow_test_carveouts {
        violations.push(format!(
            "{LINT_LEDGER}: policy.allow_test_carveouts must be false"
        ));
    }
    if ledger.policy.suppression_style != "expect-with-reason" {
        violations.push(format!(
            "{LINT_LEDGER}: policy.suppression_style must be expect-with-reason"
        ));
    }
    if ledger.policy.blanket_categories {
        violations.push(format!(
            "{LINT_LEDGER}: policy.blanket_categories must be false"
        ));
    }

    let mut seen = BTreeSet::new();
    for entry in &ledger.lint {
        if !seen.insert(entry.name.clone()) {
            violations.push(format!(
                "{LINT_LEDGER}: duplicate lint entry {}",
                entry.name
            ));
        }
        if entry.name.trim().is_empty()
            || entry.level.trim().is_empty()
            || entry.status.trim().is_empty()
            || entry.class_name.trim().is_empty()
            || entry.reason.trim().is_empty()
        {
            violations.push(format!(
                "{LINT_LEDGER}: lint {} must include name, level, status, class, and reason",
                entry.name
            ));
        }
        if entry.status == "planned" && entry.activate_when_msrv.is_none() {
            violations.push(format!(
                "{LINT_LEDGER}: planned lint {} must include activate_when_msrv",
                entry.name
            ));
        }
        if entry.status != "active" && entry.status != "staged" && entry.status != "planned" {
            violations.push(format!(
                "{LINT_LEDGER}: lint {} has unsupported status {}",
                entry.name, entry.status
            ));
        }
    }
}

fn validate_msrv(root_manifest: &toml::Value, ledger: &LintLedger, violations: &mut Vec<String>) {
    let workspace_msrv = root_manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("rust-version"))
        .and_then(toml::Value::as_str);

    if workspace_msrv != Some(ledger.msrv.as_str()) {
        violations.push(format!(
            "workspace.package.rust-version ({:?}) must match {LINT_LEDGER} msrv ({})",
            workspace_msrv, ledger.msrv
        ));
    }
}

fn validate_workspace_lints(
    root_manifest: &toml::Value,
    ledger: &LintLedger,
    violations: &mut Vec<String>,
) {
    let active_from_manifest = active_lints_from_manifest(root_manifest);
    let active_from_ledger: BTreeMap<_, _> = ledger
        .lint
        .iter()
        .filter(|entry| entry.status == "active")
        .map(|entry| (entry.name.clone(), entry.level.clone()))
        .collect();

    for (name, level) in &active_from_ledger {
        match active_from_manifest.get(name) {
            Some(manifest_level) if manifest_level == level => {}
            Some(manifest_level) => violations.push(format!(
                "{name} level mismatch: Cargo.toml has {manifest_level}, {LINT_LEDGER} has {level}"
            )),
            None => violations.push(format!(
                "{name} is active in {LINT_LEDGER} but missing from root Cargo.toml"
            )),
        }
    }

    for (name, level) in &active_from_manifest {
        if !active_from_ledger.contains_key(name) {
            violations.push(format!(
                "{name} = {level} is active in root Cargo.toml but missing from {LINT_LEDGER}"
            ));
        }
    }

    let current_msrv = parse_version(&ledger.msrv);
    for entry in ledger.lint.iter().filter(|entry| entry.status == "planned") {
        if active_from_manifest.contains_key(&entry.name) {
            violations.push(format!(
                "planned lint {} is already active before its planned MSRV {:?}",
                entry.name, entry.activate_when_msrv
            ));
        }
        if let (Some(current), Some(activate)) = (
            current_msrv.as_ref(),
            entry.activate_when_msrv.as_deref().and_then(parse_version),
        ) && &activate <= current
        {
            violations.push(format!(
                "planned lint {} activates at {}, which is not after current MSRV {}",
                entry.name, activate, current
            ));
        }
    }
}

fn active_lints_from_manifest(root_manifest: &toml::Value) -> BTreeMap<String, String> {
    let mut lints = BTreeMap::new();
    let Some(workspace_lints) = root_manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("lints"))
        .and_then(toml::Value::as_table)
    else {
        return lints;
    };

    for group in ["rust", "clippy"] {
        if let Some(table) = workspace_lints.get(group).and_then(toml::Value::as_table) {
            for (name, level) in table {
                if let Some(level) = level.as_str() {
                    lints.insert(format!("{group}::{name}"), level.to_string());
                }
            }
        }
    }

    lints
}

fn validate_lint_inheritance(
    repo_root: &Path,
    root_manifest: &toml::Value,
    violations: &mut Vec<String>,
) -> Result<()> {
    for member in workspace_members(root_manifest) {
        let manifest_path = repo_root.join(member).join("Cargo.toml");
        let manifest = load_toml_value(&manifest_path)?;
        let inherits = manifest
            .get("lints")
            .and_then(|lints| lints.get("workspace"))
            .and_then(toml::Value::as_bool)
            == Some(true);
        if !inherits {
            violations.push(format!(
                "{} must include [lints] workspace = true",
                manifest_path
                    .strip_prefix(repo_root)
                    .unwrap_or(&manifest_path)
                    .display()
            ));
        }
    }
    Ok(())
}

fn workspace_members(root_manifest: &toml::Value) -> Vec<PathBuf> {
    root_manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(toml::Value::as_str)
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

fn validate_clippy_config(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    for carveout in TEST_CARVEOUTS {
        if content.contains(carveout) {
            violations.push(format!(
                "{} must not contain test carveout `{}`",
                path.display(),
                carveout
            ));
        }
    }
    Ok(())
}

fn validate_debt(debt: &DebtLedger, violations: &mut Vec<String>) {
    if debt.schema != 1 {
        violations.push(format!("{LINT_DEBT}: schema must be 1"));
    }

    let today = Utc::now().date_naive();
    for (index, entry) in debt.debt.iter().enumerate() {
        let label = format!("{LINT_DEBT}: debt entry {}", index + 1);
        if entry.lint.trim().is_empty()
            || entry.path.trim().is_empty()
            || entry.owner.trim().is_empty()
            || entry.reason.trim().is_empty()
            || entry.expires.trim().is_empty()
        {
            violations.push(format!(
                "{label} must include lint, path, owner, reason, and expires"
            ));
            continue;
        }

        match NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d") {
            Ok(expires) if expires < today => {
                violations.push(format!("{label} for {} expired on {expires}", entry.lint));
            }
            Ok(_) => {}
            Err(_) => violations.push(format!(
                "{label} expires value `{}` must use YYYY-MM-DD",
                entry.expires
            )),
        }
    }
}

fn parse_version(version: &str) -> Option<Version> {
    let mut parts = version.split('.');
    let major = parts.next()?;
    let minor = parts.next().unwrap_or("0");
    let patch = parts.next().unwrap_or("0");
    Version::parse(&format!("{major}.{minor}.{patch}")).ok()
}
