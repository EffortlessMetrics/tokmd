use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use toml::Value as TomlValue;

use crate::cli::LintPolicyArgs;

const CLIPPY_POLICY: &str = "policy/clippy-lints.toml";
const CLIPPY_DEBT: &str = "policy/clippy-debt.toml";
const CLIPPY_CONFIG: &str = "clippy.toml";
const NO_PANIC_ALLOWLIST: &str = "policy/no-panic-allowlist.toml";
const NON_RUST_ALLOWLIST: &str = "policy/non-rust-allowlist.toml";
const FORBIDDEN_TEST_CARVEOUTS: &[&str] = &[
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
    policy: LedgerPolicy,
    #[serde(default)]
    lint: Vec<LintEntry>,
}

#[derive(Debug, Deserialize)]
struct LedgerPolicy {
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
    #[serde(default)]
    class: String,
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

#[derive(Debug, Deserialize)]
struct NoPanicAllowlist {
    schema_version: String,
    #[serde(default)]
    allow: Vec<NoPanicAllow>,
}

#[derive(Debug, Deserialize)]
struct NoPanicAllow {
    path: String,
    family: String,
    classification: String,
    owner: String,
    explanation: String,
    #[serde(default)]
    expires: Option<String>,
    selector: NoPanicSelector,
}

#[derive(Debug, Deserialize)]
struct NoPanicSelector {
    kind: String,
    #[serde(default)]
    container: Option<String>,
    #[serde(default)]
    callee: Option<String>,
    #[serde(default)]
    receiver_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NonRustAllowlist {
    schema_version: String,
    #[serde(default)]
    allow: Vec<NonRustAllow>,
}

#[derive(Debug, Deserialize)]
struct NonRustAllow {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    glob: Option<String>,
    kind: String,
    owner: String,
    reason: String,
    surface: String,
    classification: String,
    #[serde(default)]
    covered_by: Vec<String>,
    #[serde(default)]
    expires: Option<String>,
}

pub fn run(_args: LintPolicyArgs) -> Result<()> {
    let root = find_workspace_root()?;
    let root_manifest_path = root.join("Cargo.toml");
    let root_manifest = read_toml(&root_manifest_path)?;
    let ledger = read_ledger(&root.join(CLIPPY_POLICY))?;

    let mut violations = Vec::new();

    check_ledger_policy(&ledger, &mut violations);
    check_msrv(&root_manifest, &ledger, &mut violations);
    check_active_lints(&root_manifest, &ledger, &mut violations);
    check_planned_lints(&root_manifest, &ledger, &mut violations);
    check_workspace_inheritance(&root, &mut violations)?;
    check_clippy_config(&root.join(CLIPPY_CONFIG), &mut violations)?;
    check_debt(&root.join(CLIPPY_DEBT), &mut violations)?;
    check_no_panic_allowlist(&root.join(NO_PANIC_ALLOWLIST), &mut violations)?;
    check_non_rust_allowlist(&root.join(NON_RUST_ALLOWLIST), &mut violations)?;

    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("::error ::{violation}");
        }
        bail!(
            "lint policy check failed with {} violation(s)",
            violations.len()
        );
    }

    println!("Lint policy checks passed.");
    Ok(())
}

fn check_ledger_policy(ledger: &LintLedger, violations: &mut Vec<String>) {
    if ledger.schema != 1 {
        violations.push(format!("{CLIPPY_POLICY}: schema must be 1"));
    }
    if !ledger.policy.panic_free_tests {
        violations.push(format!("{CLIPPY_POLICY}: panic_free_tests must be true"));
    }
    if ledger.policy.allow_test_carveouts {
        violations.push(format!(
            "{CLIPPY_POLICY}: allow_test_carveouts must be false"
        ));
    }
    if ledger.policy.suppression_style != "expect-with-reason" {
        violations.push(format!(
            "{CLIPPY_POLICY}: suppression_style must be expect-with-reason"
        ));
    }
    if ledger.policy.blanket_categories {
        violations.push(format!("{CLIPPY_POLICY}: blanket_categories must be false"));
    }
}

fn check_msrv(root_manifest: &TomlValue, ledger: &LintLedger, violations: &mut Vec<String>) {
    let actual = root_manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("rust-version"))
        .and_then(TomlValue::as_str);

    if actual != Some(ledger.msrv.as_str()) {
        violations.push(format!(
            "workspace.package.rust-version ({}) must match {CLIPPY_POLICY} msrv ({})",
            actual.unwrap_or("missing"),
            ledger.msrv
        ));
    }
}

fn check_active_lints(
    root_manifest: &TomlValue,
    ledger: &LintLedger,
    violations: &mut Vec<String>,
) {
    let manifest_lints = workspace_lints(root_manifest);
    let active_lints: BTreeMap<String, String> = ledger
        .lint
        .iter()
        .filter(|entry| entry.status == "active")
        .map(|entry| (entry.name.clone(), entry.level.clone()))
        .collect();

    for (name, level) in &manifest_lints {
        match active_lints.get(name) {
            Some(ledger_level) if ledger_level == level => {}
            Some(ledger_level) => violations.push(format!(
                "{name} level mismatch: Cargo.toml has {level}, {CLIPPY_POLICY} has {ledger_level}"
            )),
            None => violations.push(format!(
                "{name} is active in Cargo.toml but missing from {CLIPPY_POLICY}"
            )),
        }
    }

    for (name, level) in active_lints {
        match manifest_lints.get(&name) {
            Some(manifest_level) if manifest_level == &level => {}
            Some(manifest_level) => violations.push(format!(
                "{name} level mismatch: {CLIPPY_POLICY} has {level}, Cargo.toml has {manifest_level}"
            )),
            None => violations.push(format!(
                "{name} is active in {CLIPPY_POLICY} but missing from Cargo.toml"
            )),
        }
    }

    let mut seen = BTreeSet::new();
    for entry in ledger.lint.iter().filter(|entry| entry.status == "active") {
        validate_lint_entry(entry, &mut seen, violations);
    }
}

fn check_planned_lints(
    root_manifest: &TomlValue,
    ledger: &LintLedger,
    violations: &mut Vec<String>,
) {
    let manifest_lints = workspace_lints(root_manifest);
    let mut planned = BTreeSet::new();

    for entry in ledger.lint.iter().filter(|entry| entry.status == "planned") {
        validate_lint_entry(entry, &mut planned, violations);
        if entry.activate_when_msrv.as_deref().is_none() {
            violations.push(format!(
                "{} planned lint must set activate_when_msrv",
                entry.name
            ));
        }
        if manifest_lints.contains_key(&entry.name) {
            violations.push(format!(
                "{} is planned for MSRV {} but is already active in Cargo.toml",
                entry.name,
                entry.activate_when_msrv.as_deref().unwrap_or("unknown")
            ));
        }
    }

    for (msrv, expected) in [
        (
            "1.94",
            [
                "clippy::same_length_and_capacity",
                "clippy::manual_ilog2",
                "clippy::decimal_bitwise_operands",
                "clippy::needless_type_cast",
            ]
            .as_slice(),
        ),
        (
            "1.95",
            [
                "clippy::disallowed_fields",
                "clippy::manual_checked_ops",
                "clippy::manual_take",
                "clippy::manual_pop_if",
                "clippy::duration_suboptimal_units",
                "clippy::unnecessary_trailing_comma",
            ]
            .as_slice(),
        ),
    ] {
        for lint in expected {
            let tracked = ledger.lint.iter().any(|entry| {
                entry.status == "planned"
                    && entry.name == *lint
                    && entry.activate_when_msrv.as_deref() == Some(msrv)
            });
            if !tracked {
                violations.push(format!("missing planned {msrv} lint {lint}"));
            }
        }
    }
}

fn validate_lint_entry(
    entry: &LintEntry,
    seen: &mut BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    if !seen.insert(entry.name.clone()) {
        violations.push(format!(
            "{} appears more than once for status {}",
            entry.name, entry.status
        ));
    }
    if !matches!(entry.level.as_str(), "allow" | "warn" | "deny" | "forbid") {
        violations.push(format!("{} has invalid level {}", entry.name, entry.level));
    }
    if entry.class.trim().is_empty() {
        violations.push(format!("{} must set class", entry.name));
    }
    if entry.reason.trim().is_empty() {
        violations.push(format!("{} must set reason", entry.name));
    }
}

fn check_workspace_inheritance(root: &Path, violations: &mut Vec<String>) -> Result<()> {
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .context("loading cargo metadata")?;
    let workspace_members: BTreeSet<_> = metadata.workspace_members.iter().collect();

    for package in metadata
        .packages
        .iter()
        .filter(|package| workspace_members.contains(&package.id))
    {
        let manifest_path = package.manifest_path.as_std_path();
        let manifest = read_toml(manifest_path)?;
        let inherits = manifest
            .get("lints")
            .and_then(|lints| lints.get("workspace"))
            .and_then(TomlValue::as_bool)
            == Some(true);
        if !inherits && manifest.get("lints").is_some() {
            violations.push(format!(
                "{} has a [lints] table but does not set workspace = true",
                rel_path(root, manifest_path)
            ));
        }
    }

    Ok(())
}

fn check_clippy_config(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    if !path.exists() {
        violations.push(format!("{} must exist", CLIPPY_CONFIG));
        return Ok(());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    for carveout in FORBIDDEN_TEST_CARVEOUTS {
        if content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && trimmed.starts_with(carveout)
        }) {
            violations.push(format!(
                "{CLIPPY_CONFIG}: test carveout {carveout} is forbidden"
            ));
        }
    }
    Ok(())
}

fn check_debt(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    if !path.exists() {
        violations.push(format!("{CLIPPY_DEBT} must exist"));
        return Ok(());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let debt: DebtLedger =
        toml::from_str(&content).with_context(|| format!("parsing {CLIPPY_DEBT}"))?;
    if debt.schema != 1 {
        violations.push(format!("{CLIPPY_DEBT}: schema must be 1"));
    }

    let today = Utc::now().date_naive();
    for entry in debt.debt {
        if entry.lint.trim().is_empty() {
            violations.push(format!("{CLIPPY_DEBT}: debt entry missing lint"));
        }
        if entry.path.trim().is_empty() {
            violations.push(format!(
                "{CLIPPY_DEBT}: debt entry for {} missing path",
                entry.lint
            ));
        }
        if entry.owner.trim().is_empty() {
            violations.push(format!(
                "{CLIPPY_DEBT}: debt entry for {} missing owner",
                entry.lint
            ));
        }
        if entry.reason.trim().is_empty() {
            violations.push(format!(
                "{CLIPPY_DEBT}: debt entry for {} missing reason",
                entry.lint
            ));
        }
        match NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d") {
            Ok(expires) if expires < today => violations.push(format!(
                "{CLIPPY_DEBT}: debt entry for {} at {} expired on {}",
                entry.lint, entry.path, entry.expires
            )),
            Ok(_) => {}
            Err(_) => violations.push(format!(
                "{CLIPPY_DEBT}: debt entry for {} must use YYYY-MM-DD expires",
                entry.lint
            )),
        }
    }

    Ok(())
}

fn workspace_lints(root_manifest: &TomlValue) -> BTreeMap<String, String> {
    let mut lints = BTreeMap::new();
    let Some(workspace_lints) = root_manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("lints"))
        .and_then(TomlValue::as_table)
    else {
        return lints;
    };

    for namespace in ["rust", "clippy"] {
        if let Some(table) = workspace_lints.get(namespace).and_then(TomlValue::as_table) {
            for (name, value) in table {
                if let Some(level) = value.as_str() {
                    lints.insert(format!("{namespace}::{name}"), level.to_string());
                }
            }
        }
    }

    lints
}

fn check_no_panic_allowlist(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    if !path.exists() {
        violations.push(format!("{NO_PANIC_ALLOWLIST} must exist"));
        return Ok(());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let allowlist: NoPanicAllowlist =
        toml::from_str(&content).with_context(|| format!("parsing {NO_PANIC_ALLOWLIST}"))?;
    if allowlist.schema_version != "0.3" {
        violations.push(format!("{NO_PANIC_ALLOWLIST}: schema_version must be 0.3"));
    }

    for entry in allowlist.allow {
        require_field(NO_PANIC_ALLOWLIST, "path", &entry.path, violations);
        require_field(NO_PANIC_ALLOWLIST, "family", &entry.family, violations);
        require_field(
            NO_PANIC_ALLOWLIST,
            "classification",
            &entry.classification,
            violations,
        );
        require_field(NO_PANIC_ALLOWLIST, "owner", &entry.owner, violations);
        require_field(
            NO_PANIC_ALLOWLIST,
            "explanation",
            &entry.explanation,
            violations,
        );
        require_field(
            NO_PANIC_ALLOWLIST,
            "selector.kind",
            &entry.selector.kind,
            violations,
        );
        if entry
            .selector
            .container
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
            && entry
                .selector
                .callee
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            && entry
                .selector
                .receiver_fingerprint
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            violations.push(format!(
                "{NO_PANIC_ALLOWLIST}: selector must include container, callee, or receiver_fingerprint"
            ));
        }
        check_optional_expiry(NO_PANIC_ALLOWLIST, &entry.expires, violations);
    }

    Ok(())
}

fn check_non_rust_allowlist(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    if !path.exists() {
        violations.push(format!("{NON_RUST_ALLOWLIST} must exist"));
        return Ok(());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let allowlist: NonRustAllowlist =
        toml::from_str(&content).with_context(|| format!("parsing {NON_RUST_ALLOWLIST}"))?;
    if allowlist.schema_version != "1.0" {
        violations.push(format!("{NON_RUST_ALLOWLIST}: schema_version must be 1.0"));
    }

    for entry in allowlist.allow {
        let path_missing = entry.path.as_deref().unwrap_or("").trim().is_empty();
        let glob_missing = entry.glob.as_deref().unwrap_or("").trim().is_empty();
        if path_missing == glob_missing {
            violations.push(format!(
                "{NON_RUST_ALLOWLIST}: each entry must set exactly one of path or glob"
            ));
        }
        require_field(NON_RUST_ALLOWLIST, "kind", &entry.kind, violations);
        require_field(NON_RUST_ALLOWLIST, "owner", &entry.owner, violations);
        require_field(NON_RUST_ALLOWLIST, "reason", &entry.reason, violations);
        require_field(NON_RUST_ALLOWLIST, "surface", &entry.surface, violations);
        require_field(
            NON_RUST_ALLOWLIST,
            "classification",
            &entry.classification,
            violations,
        );
        if matches!(
            entry.classification.as_str(),
            "production" | "test" | "tooling"
        ) && entry.covered_by.is_empty()
        {
            violations.push(format!(
                "{NON_RUST_ALLOWLIST}: production/test/tooling entries must set covered_by"
            ));
        }
        check_optional_expiry(NON_RUST_ALLOWLIST, &entry.expires, violations);
    }

    Ok(())
}

fn require_field(file: &str, field: &str, value: &str, violations: &mut Vec<String>) {
    if value.trim().is_empty() {
        violations.push(format!("{file}: allow entry missing {field}"));
    }
}

fn check_optional_expiry(file: &str, expires: &Option<String>, violations: &mut Vec<String>) {
    let Some(expires) = expires else {
        return;
    };
    let today = Utc::now().date_naive();
    match NaiveDate::parse_from_str(expires, "%Y-%m-%d") {
        Ok(date) if date < today => {
            violations.push(format!("{file}: allow entry expired on {expires}"));
        }
        Ok(_) => {}
        Err(_) => violations.push(format!("{file}: allow entry expiry must use YYYY-MM-DD")),
    }
}

fn read_ledger(path: &Path) -> Result<LintLedger> {
    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))
}

fn read_toml(path: &Path) -> Result<TomlValue> {
    let content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))
}

fn find_workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.exists() {
            let content = fs::read_to_string(&manifest)
                .with_context(|| format!("reading {}", manifest.display()))?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!("could not find workspace root");
        }
    }
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
