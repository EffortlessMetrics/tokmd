use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::cli::DocArtifactsArgs;

const README: &str = "README.md";
const CHECK_SCHEMA: &str = "tokmd.doc_artifacts_check.v1";

#[derive(Debug, Deserialize)]
struct Policy {
    schema_version: String,
    policy: String,
    #[serde(default)]
    active_goal: Option<ActiveGoalPolicy>,
    #[serde(default)]
    required_doc: Vec<RequiredDoc>,
    #[serde(default)]
    family: Vec<ArtifactFamily>,
}

#[derive(Debug, Deserialize)]
struct ActiveGoalPolicy {
    path: String,
    schema: String,
    #[serde(default)]
    allowed_statuses: Vec<String>,
    #[serde(default)]
    required_top_level: Vec<String>,
    #[serde(default)]
    required_tables: Vec<String>,
    #[serde(default)]
    required_links: Vec<String>,
    #[serde(default)]
    forbid_absolute_links: bool,
    #[serde(default)]
    require_existing_links: bool,
}

#[derive(Debug, Deserialize)]
struct RequiredDoc {
    path: String,
    #[serde(default)]
    required_sections: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ArtifactFamily {
    id: String,
    root: String,
    readme: String,
    #[serde(default)]
    allow_readme: bool,
    #[serde(default)]
    filename_pattern: Option<String>,
    #[serde(default)]
    allowed_statuses: Vec<String>,
    #[serde(default)]
    required_sections: Vec<String>,
    #[serde(default)]
    draft_may_omit_sections: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ActiveGoal {
    schema: String,
    status: String,
    program: String,
    lane: String,
    #[serde(default)]
    links: toml::value::Table,
    #[serde(default)]
    rules: toml::value::Table,
    #[serde(default)]
    stop_conditions: toml::value::Table,
}

pub fn run(args: DocArtifactsArgs) -> Result<()> {
    let _check = args.check;
    let root = workspace_root()?;
    let report = check(&root, &args.policy)?;
    if let Some(path) = &args.json {
        write_receipt(&root, path, &report)?;
    }
    let summary = report_result(report)?;
    println!("{summary}");
    Ok(())
}

pub fn check_current_repo(policy: &Path) -> Result<String> {
    let root = workspace_root()?;
    let report = check(&root, policy)?;
    report_result(report)
}

#[derive(Default)]
struct CheckReport {
    required_docs: usize,
    family_files: usize,
    active_goals: usize,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CheckReceipt {
    schema: &'static str,
    ok: bool,
    checked: CheckedCounts,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CheckedCounts {
    required_docs: usize,
    family_files: usize,
    active_goals: usize,
}

impl From<&CheckReport> for CheckReceipt {
    fn from(report: &CheckReport) -> Self {
        Self {
            schema: CHECK_SCHEMA,
            ok: report.errors.is_empty(),
            checked: CheckedCounts {
                required_docs: report.required_docs,
                family_files: report.family_files,
                active_goals: report.active_goals,
            },
            errors: report.errors.clone(),
        }
    }
}

fn check(root: &Path, policy_path: &Path) -> Result<CheckReport> {
    let policy_path = root.join(policy_path);
    let policy = read_policy(&policy_path)?;
    let mut report = CheckReport::default();

    validate_policy_header(&policy, &policy_path, &mut report.errors);

    for required_doc in &policy.required_doc {
        report.required_docs += 1;
        validate_required_doc(root, required_doc, &mut report.errors);
    }

    if let Some(active_goal) = &policy.active_goal {
        report.active_goals += 1;
        validate_active_goal(root, active_goal, &mut report.errors);
    }

    for family in &policy.family {
        validate_family(root, family, &mut report);
    }

    Ok(report)
}

fn report_result(report: CheckReport) -> Result<String> {
    if !report.errors.is_empty() {
        for error in &report.errors {
            eprintln!("doc artifact error: {error}");
        }
        bail!(
            "doc artifact check failed with {} error(s)",
            report.errors.len()
        );
    }

    Ok(format!(
        "doc artifacts ok: {} required doc(s), {} family file(s), {} active goal(s)",
        report.required_docs, report.family_files, report.active_goals
    ))
}

fn write_receipt(root: &Path, path: &Path, report: &CheckReport) -> Result<()> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create doc artifacts receipt dir {}", parent.display()))?;
    }
    let receipt = CheckReceipt::from(report);
    let json = serde_json::to_string_pretty(&receipt).context("serialize doc artifacts receipt")?;
    fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("write doc artifacts receipt {}", path.display()))
}

fn workspace_root() -> Result<PathBuf> {
    let mut command = MetadataCommand::new();
    command.no_deps();
    let metadata = command.exec().context("load cargo metadata")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn read_policy(path: &Path) -> Result<Policy> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("read doc artifacts policy {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("parse doc artifacts policy {}", path.display()))
}

fn validate_policy_header(policy: &Policy, path: &Path, errors: &mut Vec<String>) {
    if policy.schema_version != "0.1" {
        errors.push(format!(
            "{}: unsupported schema_version {:?}",
            path.display(),
            policy.schema_version
        ));
    }
    if policy.policy != "doc-artifacts" {
        errors.push(format!(
            "{}: expected policy \"doc-artifacts\", found {:?}",
            path.display(),
            policy.policy
        ));
    }
}

fn validate_required_doc(root: &Path, required_doc: &RequiredDoc, errors: &mut Vec<String>) {
    let path = root.join(&required_doc.path);
    let content = match read_markdown(&path, errors) {
        Some(content) => content,
        None => return,
    };
    validate_top_heading(&required_doc.path, &content, errors);
    for section in &required_doc.required_sections {
        validate_section(&required_doc.path, &content, section, errors);
    }
}

fn validate_active_goal(root: &Path, policy: &ActiveGoalPolicy, errors: &mut Vec<String>) {
    let path = root.join(&policy.path);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            errors.push(format!("{} is unreadable: {err}", policy.path));
            return;
        }
    };
    let value: toml::Value = match toml::from_str(&content) {
        Ok(value) => value,
        Err(err) => {
            errors.push(format!("{} is invalid TOML: {err}", policy.path));
            return;
        }
    };
    let goal: ActiveGoal = match value.clone().try_into() {
        Ok(goal) => goal,
        Err(err) => {
            errors.push(format!(
                "{} does not match active-goal shape: {err}",
                policy.path
            ));
            return;
        }
    };

    if goal.schema != policy.schema {
        errors.push(format!(
            "{}: unsupported schema {:?}; expected {:?}",
            policy.path, goal.schema, policy.schema
        ));
    }
    if !policy.allowed_statuses.is_empty()
        && !policy
            .allowed_statuses
            .iter()
            .any(|status| status == &goal.status)
    {
        errors.push(format!(
            "{}: status {:?} is not one of {}",
            policy.path,
            goal.status,
            policy.allowed_statuses.join(", ")
        ));
    }
    if goal.program.trim().is_empty() {
        errors.push(format!("{}: program must not be empty", policy.path));
    }
    if goal.lane.trim().is_empty() {
        errors.push(format!("{}: lane must not be empty", policy.path));
    }

    for key in &policy.required_top_level {
        if value.get(key).is_none() {
            errors.push(format!("{}: missing top-level key {key}", policy.path));
        }
    }
    for table in &policy.required_tables {
        if !matches!(value.get(table), Some(toml::Value::Table(_))) {
            errors.push(format!("{}: missing table [{table}]", policy.path));
        }
    }
    for link in &policy.required_links {
        if !goal.links.contains_key(link) {
            errors.push(format!("{}: missing [links].{link}", policy.path));
        }
    }
    for (link, value) in &goal.links {
        let Some(link_value) = value.as_str() else {
            errors.push(format!("{}: [links].{link} must be a string", policy.path));
            continue;
        };
        validate_active_goal_link(root, &policy.path, link, link_value, policy, errors);
    }
    if goal.rules.is_empty() {
        errors.push(format!("{}: [rules] must not be empty", policy.path));
    }
    if goal.stop_conditions.is_empty() {
        errors.push(format!(
            "{}: [stop_conditions] must not be empty",
            policy.path
        ));
    }
}

fn validate_active_goal_link(
    root: &Path,
    active_goal_path: &str,
    link: &str,
    value: &str,
    policy: &ActiveGoalPolicy,
    errors: &mut Vec<String>,
) {
    let link_path = Path::new(value);
    if policy.forbid_absolute_links && link_path.is_absolute() {
        errors.push(format!(
            "{active_goal_path}: [links].{link} must be repo-relative, found {value:?}"
        ));
        return;
    }
    if value.contains("..") {
        errors.push(format!(
            "{active_goal_path}: [links].{link} must not traverse parents, found {value:?}"
        ));
        return;
    }
    if policy.require_existing_links && !root.join(link_path).exists() {
        errors.push(format!(
            "{active_goal_path}: [links].{link} points at missing path {value:?}"
        ));
    }
}

fn validate_family(root: &Path, family: &ArtifactFamily, report: &mut CheckReport) {
    let root_path = root.join(&family.root);
    if !root_path.is_dir() {
        report
            .errors
            .push(format!("{}: family root is missing", family.id));
        return;
    }
    if !root.join(&family.readme).is_file() {
        report
            .errors
            .push(format!("{}: family README is missing", family.id));
    }

    for entry in WalkDir::new(&root_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let relative = match path.strip_prefix(root) {
            Ok(path) => normalize_path(path),
            Err(_) => path.display().to_string(),
        };
        if family.allow_readme && path.file_name().and_then(|name| name.to_str()) == Some(README) {
            continue;
        }
        report.family_files += 1;
        validate_family_file(path, &relative, family, &mut report.errors);
    }
}

fn validate_family_file(
    path: &Path,
    relative: &str,
    family: &ArtifactFamily,
    errors: &mut Vec<String>,
) {
    if family.filename_pattern.is_some() && !valid_numbered_doc_filename(path) {
        errors.push(format!(
            "{}: filename must match numbered ADR form NNNN-name.md",
            relative
        ));
    }

    let content = match read_markdown(path, errors) {
        Some(content) => content,
        None => return,
    };
    validate_top_heading(relative, &content, errors);

    let status = match status_line(&content) {
        Some(status) => status,
        None => {
            errors.push(format!("{relative}: missing '- Status:' line"));
            String::new()
        }
    };
    if !status.is_empty()
        && !family.allowed_statuses.is_empty()
        && !family
            .allowed_statuses
            .iter()
            .any(|allowed| allowed == &status)
    {
        errors.push(format!(
            "{}: status {:?} is not one of {}",
            relative,
            status,
            family.allowed_statuses.join(", ")
        ));
    }

    for section in &family.required_sections {
        if status == "draft" && family.draft_may_omit_sections.iter().any(|s| s == section) {
            continue;
        }
        validate_section(relative, &content, section, errors);
    }
}

fn read_markdown(path: &Path, errors: &mut Vec<String>) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(content) => Some(content),
        Err(err) => {
            errors.push(format!("{} is unreadable: {err}", path.display()));
            None
        }
    }
}

fn validate_top_heading(relative: &str, content: &str, errors: &mut Vec<String>) {
    if !content.lines().any(|line| line.starts_with("# ")) {
        errors.push(format!("{relative}: missing top-level Markdown heading"));
    }
}

fn validate_section(relative: &str, content: &str, section: &str, errors: &mut Vec<String>) {
    if !content.lines().any(|line| line.trim_end() == section) {
        errors.push(format!("{relative}: missing required section {section:?}"));
    }
}

fn status_line(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let value = line.strip_prefix("- Status:")?.trim();
        Some(value.trim_matches('`').to_string())
    })
}

fn valid_numbered_doc_filename(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some(stem) = name.strip_suffix(".md") else {
        return false;
    };
    let Some((number, slug)) = stem.split_once('-') else {
        return false;
    };
    number.len() == 4
        && number.chars().all(|ch| ch.is_ascii_digit())
        && !slug.is_empty()
        && slug
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn valid_fixture_passes() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());

        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();

        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(report.active_goals, 1);
        assert_eq!(report.required_docs, 1);
        assert_eq!(report.family_files, 4);
    }

    #[test]
    fn json_receipt_reports_success_counts() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();
        let output = temp.path().join("target/doc-artifacts-check.json");

        write_receipt(temp.path(), &output, &report).unwrap();

        let receipt = read_json(&output);
        assert_eq!(receipt["schema"], CHECK_SCHEMA);
        assert_eq!(receipt["ok"], true);
        assert_eq!(receipt["checked"]["required_docs"], 1);
        assert_eq!(receipt["checked"]["family_files"], 4);
        assert_eq!(receipt["checked"]["active_goals"], 1);
        assert_eq!(receipt["errors"].as_array().expect("errors array").len(), 0);
    }

    #[test]
    fn json_receipt_reports_failure_errors() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        fs::write(
            temp.path().join(".jules/goals/active.toml"),
            active_goal("missing.md", "tokmd.jules.active_goal.v1"),
        )
        .unwrap();
        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();
        let output = Path::new("target/doc-artifacts-check.json");

        write_receipt(temp.path(), output, &report).unwrap();

        let receipt = read_json(&temp.path().join(output));
        assert_eq!(receipt["schema"], CHECK_SCHEMA);
        assert_eq!(receipt["ok"], false);
        assert!(
            receipt["errors"]
                .as_array()
                .expect("errors array")
                .iter()
                .any(|error| error
                    .as_str()
                    .expect("error string")
                    .contains("points at missing path")),
            "{receipt:#}"
        );
    }

    #[test]
    fn broken_active_goal_link_fails() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        fs::write(
            temp.path().join(".jules/goals/active.toml"),
            active_goal("missing.md", "tokmd.jules.active_goal.v1"),
        )
        .unwrap();

        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("points at missing path")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn extra_active_goal_links_are_validated() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        let goal = active_goal("docs/source-of-truth.md", "tokmd.jules.active_goal.v1").replace(
            "adr_readme = \"docs/adr/README.md\"",
            "adr_readme = \"docs/adr/README.md\"\nreview_packet_contract = \"missing.md\"",
        );
        fs::write(temp.path().join(".jules/goals/active.toml"), goal).unwrap();

        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();

        assert!(
            report.errors.iter().any(|error| {
                error.contains("[links].review_packet_contract")
                    && error.contains("points at missing path")
            }),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn missing_required_section_fails() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        fs::write(
            temp.path().join("docs/plans/doc-artifacts-check.md"),
            "# Plan: Documentation Artifact Checker\n\n- Status: active\n\n## Goal\n",
        )
        .unwrap();

        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("missing required section \"## Work Packets\"")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn invalid_adr_filename_fails() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        fs::rename(
            temp.path().join("docs/adr/0000-adr-process.md"),
            temp.path().join("docs/adr/process.md"),
        )
        .unwrap();

        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("filename must match numbered ADR form")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn unknown_active_goal_schema_fails() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        fs::write(
            temp.path().join(".jules/goals/active.toml"),
            active_goal("docs/source-of-truth.md", "tokmd.jules.active_goal.v9"),
        )
        .unwrap();

        let report = check(temp.path(), Path::new("policy/doc-artifacts.toml")).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("unsupported schema")),
            "{:?}",
            report.errors
        );
    }

    fn write_valid_fixture(root: &Path) {
        for dir in [
            "docs/proposals",
            "docs/specs",
            "docs/adr",
            "docs/plans",
            ".jules/goals",
            "policy",
        ] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        fs::write(root.join("ci-proof-placeholder"), "").unwrap();
        fs::write(
            root.join("policy/doc-artifacts.toml"),
            include_str!("../../../policy/doc-artifacts.toml"),
        )
        .unwrap();
        fs::write(
            root.join("docs/source-of-truth.md"),
            "# Source of Truth Model\n\n## Goal\n\n## Artifact Roles\n\n## Conflict Resolution\n\n## Lifecycle\n\n## Review Expectations\n",
        )
        .unwrap();
        fs::write(root.join("docs/proposals/README.md"), "# Proposals\n").unwrap();
        fs::write(root.join("docs/specs/README.md"), "# Specs\n").unwrap();
        fs::write(root.join("docs/adr/README.md"), "# ADRs\n").unwrap();
        fs::write(root.join("docs/plans/README.md"), "# Plans\n").unwrap();
        fs::write(
            root.join("docs/proposals/source-of-truth.md"),
            "# Proposal: Source of Truth\n\n- Status: proposed\n\n## Problem\n\n## Goals\n\n## Non-goals\n\n## Options\n\n## Recommendation\n\n## Open Questions\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/specs/doc-artifacts.md"),
            "# Spec: Documentation Artifacts\n\n- Status: active\n\n## Contract\n\n## Inputs\n\n## Outputs\n\n## Compatibility\n\n## Proof Requirements\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/adr/0000-adr-process.md"),
            "# ADR-0000: ADR Process\n\n- Status: accepted\n\n## Context\n\n## Decision\n\n## Consequences\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/plans/doc-artifacts-check.md"),
            "# Plan: Documentation Artifact Checker\n\n- Status: active\n\n## Goal\n\n## Non-goals\n\n## Work Packets\n\n## Validation\n\n## Stop Conditions\n",
        )
        .unwrap();
        fs::write(
            root.join(".jules/goals/active.toml"),
            active_goal("docs/source-of-truth.md", "tokmd.jules.active_goal.v1"),
        )
        .unwrap();
    }

    fn active_goal(source_of_truth: &str, schema: &str) -> String {
        format!(
            r#"schema = "{schema}"
status = "active"
program = "source_of_truth_docs"
lane = "documentation_control_surface"

[links]
source_of_truth = "{source_of_truth}"
doc_artifacts_spec = "docs/specs/doc-artifacts.md"
doc_artifacts_plan = "docs/plans/doc-artifacts-check.md"
plan_readme = "docs/plans/README.md"
proposal_readme = "docs/proposals/README.md"
spec_readme = "docs/specs/README.md"
adr_readme = "docs/adr/README.md"

[rules]
proof_promotion = "do_not_promote"

[stop_conditions]
docs_check = "cargo xtask docs --check"
"#
        )
    }

    fn read_json(path: &Path) -> serde_json::Value {
        let body = fs::read_to_string(path).expect("read json receipt");
        serde_json::from_str(&body).expect("parse json receipt")
    }
}
