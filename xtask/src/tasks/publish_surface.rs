use crate::cli::PublishSurfaceArgs;
use anyhow::{Context, Result, bail};
use cargo_metadata::{DependencyKind, MetadataCommand, Package};
use serde::Serialize;
use serde_json::to_string_pretty;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::process::Command;

const PUBLISHED_PUBLIC_CRATES: &[&str] = &[
    "tokmd",
    "tokmd-cockpit",
    "tokmd-core",
    "tokmd-envelope",
    "tokmd-ffi-envelope",
    "tokmd-gate",
    "tokmd-io-port",
    "tokmd-sensor",
    "tokmd-settings",
    "tokmd-substrate",
    "tokmd-types",
    "tokmd-wasm",
    "tokmd-analysis-types",
];

const PUBLISHED_SUPPORT_CRATES: &[&str] = &[
    "tokmd-analysis",
    "tokmd-analysis-api-surface",
    "tokmd-analysis-complexity",
    "tokmd-analysis-content",
    "tokmd-analysis-effort",
    "tokmd-analysis-entropy",
    "tokmd-analysis-explain",
    "tokmd-analysis-format",
    "tokmd-analysis-git",
    "tokmd-analysis-halstead",
    "tokmd-analysis-html",
    "tokmd-analysis-imports",
    "tokmd-analysis-license",
    "tokmd-analysis-maintainability",
    "tokmd-analysis-near-dup",
    "tokmd-analysis-util",
    "tokmd-content",
    "tokmd-format",
    "tokmd-fun",
    "tokmd-git",
    "tokmd-model",
    "tokmd-scan",
    "tokmd-test-support",
    "tokmd-walk",
];

const TARGET_SUPPORT_CRATES: &[&str] = &[
    "tokmd-analysis",
    "tokmd-analysis-api-surface",
    "tokmd-analysis-complexity",
    "tokmd-analysis-content",
    "tokmd-analysis-effort",
    "tokmd-analysis-entropy",
    "tokmd-analysis-explain",
    "tokmd-analysis-format",
    "tokmd-analysis-git",
    "tokmd-analysis-halstead",
    "tokmd-analysis-html",
    "tokmd-analysis-imports",
    "tokmd-analysis-license",
    "tokmd-analysis-maintainability",
    "tokmd-analysis-near-dup",
    "tokmd-content",
    "tokmd-format",
    "tokmd-fun",
    "tokmd-git",
    "tokmd-model",
    "tokmd-scan",
    "tokmd-test-support",
    "tokmd-walk",
];

const TARGET_SUPPORT_GAP_CRATES: &[&str] = &["tokmd-analysis-util"];

const NON_CRATES_IO_PACKAGES: &[&str] = &["tokmd-fuzz", "tokmd-node", "tokmd-python", "xtask"];

#[derive(Debug, Serialize)]
struct PublishSurface {
    workspace_version: String,
    summary: PublishSurfaceSummary,
    crates: Vec<PublishCrateReport>,
    packaging_checks: Vec<PackagingCheck>,
    violations: Vec<PublishViolation>,
}

#[derive(Debug, Serialize)]
struct PublishSurfaceSummary {
    /// Compatibility alias for `current_public_surface`.
    public_surface: Vec<String>,
    /// Compatibility alias for `current_support_surface`.
    support_surface: Vec<String>,
    /// Compatibility alias for `current_non_crates_io_surface`.
    non_crates_io_packages: Vec<String>,
    current_public_surface: Vec<String>,
    current_support_surface: Vec<String>,
    current_non_crates_io_surface: Vec<String>,
    target_public_surface: Vec<String>,
    target_support_surface: Vec<String>,
    target_gap: Vec<String>,
    new_unapproved_support_crates: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PublishCrateReport {
    crate_name: String,
    package_exists: bool,
    publish_false: bool,
    non_dev_workspace_closure: Vec<String>,
    required_public: Vec<String>,
    required_support: Vec<String>,
    required_internal: Vec<String>,
    required_non_crates_io: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PackagingCheck {
    crate_name: String,
    package_list_ok: bool,
    package_list_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct PublishViolation {
    crate_name: String,
    reason: String,
    details: Vec<String>,
}

pub fn run(args: PublishSurfaceArgs) -> Result<()> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Failed to load cargo metadata")?;

    let workspace_member_ids: HashSet<_> = metadata.workspace_members.iter().collect();
    let workspace_packages: Vec<&Package> = metadata
        .packages
        .iter()
        .filter(|pkg| workspace_member_ids.contains(&pkg.id))
        .collect();

    let package_by_name: BTreeMap<String, &Package> = workspace_packages
        .iter()
        .map(|package| (package.name.to_string(), *package))
        .collect();

    let workspace_version = workspace_packages
        .iter()
        .find(|pkg| pkg.name == "tokmd")
        .map(|pkg| pkg.version.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let public_surface = sort_unique(PUBLISHED_PUBLIC_CRATES);
    let support_surface = sort_unique(PUBLISHED_SUPPORT_CRATES);
    let non_crates_io_packages = sort_unique(NON_CRATES_IO_PACKAGES);
    let target_public_surface = public_surface.clone();
    let target_support_surface = sort_unique(TARGET_SUPPORT_CRATES);
    let target_gap = sort_unique(TARGET_SUPPORT_GAP_CRATES);

    let publish_surface: BTreeSet<String> = public_surface
        .iter()
        .chain(support_surface.iter())
        .cloned()
        .collect();

    let public_surface_set: BTreeSet<String> = public_surface.iter().cloned().collect();
    let support_surface_set: BTreeSet<String> = support_surface.iter().cloned().collect();
    let non_crates_io_set: BTreeSet<String> = non_crates_io_packages.iter().cloned().collect();
    let target_support_set: BTreeSet<String> = target_support_surface.iter().cloned().collect();
    let target_gap_set: BTreeSet<String> = target_gap.iter().cloned().collect();
    let new_unapproved_support_crates =
        new_unapproved_support_crates(&support_surface_set, &target_support_set, &target_gap_set);

    let mut violations = Vec::new();
    classify_target_surface_violations(
        &mut violations,
        &support_surface_set,
        &target_support_set,
        &target_gap_set,
        &new_unapproved_support_crates,
    );

    let mut crate_reports = Vec::new();

    for crate_name in publish_surface.iter() {
        let Some(package) = package_by_name.get(crate_name).copied() else {
            violations.push(PublishViolation {
                crate_name: crate_name.clone(),
                reason: "Missing from workspace".to_string(),
                details: vec![
                    "No package found for this expected public/support crate".to_string(),
                ],
            });

            crate_reports.push(PublishCrateReport {
                crate_name: crate_name.clone(),
                package_exists: false,
                publish_false: false,
                non_dev_workspace_closure: Vec::new(),
                required_public: Vec::new(),
                required_support: Vec::new(),
                required_internal: Vec::new(),
                required_non_crates_io: Vec::new(),
            });

            continue;
        };

        let closure = dependency_closure(&package.name, &package_by_name)?;
        let mut required_public = BTreeSet::new();
        let mut required_support = BTreeSet::new();
        let mut required_internal = BTreeSet::new();
        let mut required_non_crates_io = BTreeSet::new();

        for dep in &closure {
            if public_surface_set.contains(dep) {
                required_public.insert(dep.clone());
            } else if support_surface_set.contains(dep) {
                required_support.insert(dep.clone());
            } else if non_crates_io_set.contains(dep) {
                required_non_crates_io.insert(dep.clone());
            } else {
                required_internal.insert(dep.clone());
            }
        }

        if package
            .publish
            .as_ref()
            .is_some_and(|publish| publish.is_empty())
        {
            violations.push(PublishViolation {
                crate_name: crate_name.clone(),
                reason: "Public target crate is publish = false".to_string(),
                details: vec![format!(
                    "{} is currently in the public/support surface",
                    package.name
                )],
            });
        }

        if !required_internal.is_empty() {
            violations.push(PublishViolation {
                crate_name: crate_name.clone(),
                reason: "Closure contains internal crates not in publish support surface"
                    .to_string(),
                details: required_internal.iter().cloned().collect(),
            });
        }

        if !required_non_crates_io.is_empty() {
            violations.push(PublishViolation {
                crate_name: crate_name.clone(),
                reason: "Closure contains non-crates.io package layer".to_string(),
                details: required_non_crates_io.iter().cloned().collect(),
            });
        }

        let report = PublishCrateReport {
            crate_name: package.name.to_string(),
            package_exists: true,
            publish_false: package
                .publish
                .as_ref()
                .is_some_and(|publish| publish.is_empty()),
            non_dev_workspace_closure: closure.into_iter().collect(),
            required_public: required_public.into_iter().collect(),
            required_support: required_support.into_iter().collect(),
            required_internal: required_internal.into_iter().collect(),
            required_non_crates_io: required_non_crates_io.into_iter().collect(),
        };

        crate_reports.push(report);
    }

    let mut packaging_checks = Vec::new();
    if args.verify_publish {
        for crate_name in &publish_surface {
            let check = verify_packaging(crate_name)?;
            if !check.package_list_ok {
                violations.push(PublishViolation {
                    crate_name: crate_name.clone(),
                    reason: "Cargo packaging validation failed".to_string(),
                    details: check
                        .package_list_error
                        .clone()
                        .map(|error| vec![format!("package --list: {error}")])
                        .unwrap_or_default(),
                });
            }
            packaging_checks.push(check);
        }
    }

    let report = PublishSurface {
        workspace_version,
        summary: PublishSurfaceSummary {
            public_surface: public_surface.clone(),
            support_surface: support_surface.clone(),
            non_crates_io_packages: non_crates_io_packages.clone(),
            current_public_surface: public_surface,
            current_support_surface: support_surface,
            current_non_crates_io_surface: non_crates_io_packages,
            target_public_surface,
            target_support_surface,
            target_gap,
            new_unapproved_support_crates,
        },
        crates: crate_reports,
        packaging_checks,
        violations,
    };

    if args.json {
        println!("{}", to_string_pretty(&report)?);
    } else {
        print_human_report(&report);
    }

    if !report.violations.is_empty() {
        bail!(
            "publish surface validation failed with {} violation(s)",
            report.violations.len()
        );
    }

    Ok(())
}

fn dependency_closure(
    crate_name: &str,
    package_by_name: &BTreeMap<String, &Package>,
) -> Result<BTreeSet<String>> {
    let mut closure = BTreeSet::new();
    let mut frontier = vec![crate_name.to_string()];

    while let Some(current) = frontier.pop() {
        if !closure.insert(current.clone()) {
            continue;
        }

        let package = package_by_name
            .get(&current)
            .copied()
            .context(format!("Package {current} missing from workspace"))?;

        for dep in &package.dependencies {
            if !is_non_dev_dependency(&dep.kind) {
                continue;
            }

            if package_by_name.contains_key(&dep.name) {
                frontier.push(dep.name.to_string());
            }
        }
    }

    Ok(closure)
}

fn is_non_dev_dependency(kind: &DependencyKind) -> bool {
    matches!(
        kind,
        DependencyKind::Normal | DependencyKind::Build | DependencyKind::Unknown
    )
}

fn verify_packaging(crate_name: &str) -> Result<PackagingCheck> {
    let package_list = run_cargo_command(
        crate_name,
        &["package", "-p", crate_name, "--list", "--locked"],
        "cargo package --list",
    )?;

    Ok(PackagingCheck {
        crate_name: crate_name.to_string(),
        package_list_ok: package_list.0,
        package_list_error: package_list.1,
    })
}

fn run_cargo_command(
    crate_name: &str,
    args: &[&str],
    command_label: &str,
) -> Result<(bool, Option<String>)> {
    let output = Command::new("cargo")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run {command_label} for {crate_name}"))?;

    if output.status.success() {
        return Ok((true, None));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stdout.is_empty() {
        stderr
    } else {
        format!("{stderr}\n{stdout}")
    };

    Ok((
        false,
        Some(detail.lines().take(10).collect::<Vec<_>>().join("\n")),
    ))
}

fn print_human_report(report: &PublishSurface) {
    println!("Publish surface v{}", report.workspace_version);
    println!(
        "Current public crate surface ({}):",
        report.summary.current_public_surface.len()
    );
    for item in &report.summary.current_public_surface {
        println!("  - {item}");
    }

    println!(
        "Current published support crates ({}):",
        report.summary.current_support_surface.len()
    );
    for item in &report.summary.current_support_surface {
        println!("  - {item}");
    }

    println!(
        "Target public crate surface ({}):",
        report.summary.target_public_surface.len()
    );
    for item in &report.summary.target_public_surface {
        println!("  - {item}");
    }

    println!(
        "Target support crates ({}):",
        report.summary.target_support_surface.len()
    );
    for item in &report.summary.target_support_surface {
        println!("  - {item}");
    }

    println!("Target support gap ({}):", report.summary.target_gap.len());
    for item in &report.summary.target_gap {
        println!("  - {item}");
    }

    if !report.summary.new_unapproved_support_crates.is_empty() {
        println!(
            "New unapproved support crates ({}):",
            report.summary.new_unapproved_support_crates.len()
        );
        for item in &report.summary.new_unapproved_support_crates {
            println!("  - {item}");
        }
    }

    println!(
        "Non-crates.io packages: {}",
        report.summary.current_non_crates_io_surface.len()
    );
    for item in &report.summary.current_non_crates_io_surface {
        println!("  - {item}");
    }

    println!("Closure checks:");
    for crate_report in &report.crates {
        println!(
            "  - {} (publish = {}): closure {} crates",
            crate_report.crate_name,
            if crate_report.publish_false {
                "false"
            } else {
                "true"
            },
            crate_report.non_dev_workspace_closure.len()
        );

        if !crate_report.required_internal.is_empty() {
            println!(
                "      internal dependency leakage: {:?}",
                crate_report.required_internal
            );
        }

        if !crate_report.required_non_crates_io.is_empty() {
            println!(
                "      non-crates.io leakage: {:?}",
                crate_report.required_non_crates_io
            );
        }
    }

    if report.packaging_checks.is_empty() {
        return;
    }

    println!("Packaging checks:");
    for check in &report.packaging_checks {
        println!(
            "  - {}: package_list={}",
            check.crate_name, check.package_list_ok
        );
    }
}

fn sort_unique(values: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = values.iter().map(|v| (*v).to_string()).collect();
    out.sort();
    out.dedup();
    out
}

fn new_unapproved_support_crates(
    support_surface: &BTreeSet<String>,
    target_support_surface: &BTreeSet<String>,
    target_gap: &BTreeSet<String>,
) -> Vec<String> {
    let approved_support: BTreeSet<String> = target_support_surface
        .iter()
        .chain(target_gap.iter())
        .cloned()
        .collect();

    support_surface
        .difference(&approved_support)
        .cloned()
        .collect()
}

fn classify_target_surface_violations(
    violations: &mut Vec<PublishViolation>,
    support_surface: &BTreeSet<String>,
    target_support_surface: &BTreeSet<String>,
    target_gap: &BTreeSet<String>,
    new_unapproved_support_crates: &[String],
) {
    let overlap: Vec<String> = target_support_surface
        .intersection(target_gap)
        .cloned()
        .collect();
    if !overlap.is_empty() {
        violations.push(PublishViolation {
            crate_name: "publish-surface".to_string(),
            reason: "Target support classification overlaps target gap".to_string(),
            details: overlap,
        });
    }

    let missing_target_support: Vec<String> = target_support_surface
        .difference(support_surface)
        .cloned()
        .collect();
    if !missing_target_support.is_empty() {
        violations.push(PublishViolation {
            crate_name: "publish-surface".to_string(),
            reason: "Target support crate is not in current support surface".to_string(),
            details: missing_target_support,
        });
    }

    let missing_target_gap: Vec<String> = target_gap.difference(support_surface).cloned().collect();
    if !missing_target_gap.is_empty() {
        violations.push(PublishViolation {
            crate_name: "publish-surface".to_string(),
            reason: "Target-gap crate is no longer in current support surface".to_string(),
            details: missing_target_gap,
        });
    }

    if !new_unapproved_support_crates.is_empty() {
        violations.push(PublishViolation {
            crate_name: "publish-surface".to_string(),
            reason: "Current support crate is not classified for target policy".to_string(),
            details: new_unapproved_support_crates.to_vec(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn publish_surface_target_gap_is_explicit_and_complete() {
        let current_support = set(PUBLISHED_SUPPORT_CRATES);
        let target_support = set(TARGET_SUPPORT_CRATES);
        let target_gap = set(TARGET_SUPPORT_GAP_CRATES);
        let new_unapproved =
            new_unapproved_support_crates(&current_support, &target_support, &target_gap);

        assert_eq!(target_gap.len(), 1);
        assert!(target_gap.contains("tokmd-analysis-util"));
        assert!(current_support.is_superset(&target_support));
        assert!(current_support.is_superset(&target_gap));
        assert!(target_support.is_disjoint(&target_gap));
        assert!(
            new_unapproved.is_empty(),
            "all current support crates must be target support or target gap: {new_unapproved:?}"
        );
    }

    #[test]
    fn publish_surface_keeps_test_support_classified_until_policy_changes() {
        let current_support = set(PUBLISHED_SUPPORT_CRATES);
        let target_support = set(TARGET_SUPPORT_CRATES);
        let target_gap = set(TARGET_SUPPORT_GAP_CRATES);

        assert!(current_support.contains("tokmd-test-support"));
        assert!(target_support.contains("tokmd-test-support"));
        assert!(!target_gap.contains("tokmd-test-support"));
    }

    #[test]
    fn publish_surface_ignores_dev_dependencies_for_closure() {
        assert!(!is_non_dev_dependency(&DependencyKind::Development));
    }
}
