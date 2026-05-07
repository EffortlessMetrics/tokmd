//! Non-Rust file policy checker.
//!
//! Walks the repo and reports any non-Rust file that does not match a
//! `[[allow]]` glob in `policy/non-rust-allowlist.toml`. Rust files are
//! governed by the workspace lints + proof policy and are skipped here.
//!
//! Advisory by default: returns non-zero only with `--strict` or on a
//! hard parse / schema error.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use walkdir::WalkDir;

use crate::cli::FilePolicyArgs;

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", "run-artifacts", "plans"];

#[derive(Debug, Deserialize)]
struct AllowlistFile {
    schema_version: String,
    #[serde(default)]
    policy: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    allow: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    glob: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    surface: String,
    #[serde(default)]
    classification: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    covered_by: Vec<String>,
}

pub fn run(args: FilePolicyArgs) -> Result<()> {
    let root = workspace_root()?;
    let allowlist_path = root.join(&args.allowlist);
    let allowlist = parse(&allowlist_path)?;

    let mut hard_errors = Vec::new();
    if allowlist.schema_version != "1.0" {
        hard_errors.push(format!(
            "{}: unsupported schema_version {:?}",
            allowlist_path.display(),
            allowlist.schema_version
        ));
    }

    let mut findings: Vec<String> = Vec::new();
    validate_entries(&allowlist.allow, &mut findings);

    let mut builder = GlobSetBuilder::new();
    for entry in &allowlist.allow {
        builder
            .add(Glob::new(&entry.glob).with_context(|| format!("compile glob {:?}", entry.glob))?);
    }
    let set: GlobSet = builder.build()?;

    let mut unmatched = Vec::new();
    let mut covered = 0usize;
    let mut rust_skipped = 0usize;

    for entry in WalkDir::new(&root)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !is_skipped(e.path(), &root))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                hard_errors.push(format!("walk: {err}"));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = match entry.path().strip_prefix(&root) {
            Ok(p) => p.to_path_buf(),
            Err(_) => continue,
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.ends_with(".rs") {
            rust_skipped += 1;
            continue;
        }
        if set.is_match(&rel_str) {
            covered += 1;
        } else {
            unmatched.push(rel_str);
        }
    }

    unmatched.sort();
    for path in &unmatched {
        findings.push(format!(
            "file {path} does not match any non-Rust allowlist glob"
        ));
    }

    if let Some(report_dir) = &args.report_dir {
        let dir = root.join(report_dir);
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let out = dir.join("file-policy-report.txt");
        let body = render_report(&allowlist, covered, rust_skipped, &unmatched, &findings);
        fs::write(&out, &body).with_context(|| format!("write {}", out.display()))?;
        println!("file-policy report written to {}", out.display());
    }

    if !hard_errors.is_empty() {
        for err in &hard_errors {
            eprintln!("error: {err}");
        }
        bail!("file-policy: {} hard error(s)", hard_errors.len());
    }

    if findings.is_empty() {
        println!(
            "file-policy OK: {} entries, {} non-Rust files covered, {} Rust files skipped",
            allowlist.allow.len(),
            covered,
            rust_skipped
        );
        return Ok(());
    }

    println!("file-policy findings ({}):", findings.len());
    for finding in findings.iter().take(50) {
        println!("  - {finding}");
    }
    if findings.len() > 50 {
        println!("  ... ({} more, see report)", findings.len() - 50);
    }

    if args.strict {
        bail!("file-policy: {} finding(s) (strict)", findings.len());
    }
    println!("(advisory mode; rerun with --strict to fail on findings)");
    Ok(())
}

fn validate_entries(entries: &[Entry], findings: &mut Vec<String>) {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for entry in entries {
        if !seen.insert(entry.glob.as_str()) {
            findings.push(format!("duplicate glob {:?}", entry.glob));
        }
        if entry.owner.is_empty() {
            findings.push(format!("entry {:?}: missing owner", entry.glob));
        }
        if entry.kind.is_empty() {
            findings.push(format!("entry {:?}: missing kind", entry.glob));
        }
        if entry.classification.is_empty() {
            findings.push(format!("entry {:?}: missing classification", entry.glob));
        }
        if entry.reason.is_empty() {
            findings.push(format!("entry {:?}: missing reason", entry.glob));
        }
        if entry.surface.is_empty() {
            findings.push(format!("entry {:?}: missing surface", entry.glob));
        }
        if entry.classification == "production" && entry.covered_by.is_empty() {
            findings.push(format!(
                "entry {:?}: production classification needs at least one covered_by",
                entry.glob
            ));
        }
    }
}

fn is_skipped(path: &Path, root: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };
    let Some(first) = rel.iter().next().and_then(|s| s.to_str()) else {
        return false;
    };
    SKIP_DIRS.contains(&first)
}

fn parse(path: &Path) -> Result<AllowlistFile> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&body).with_context(|| format!("parse {}", path.display()))
}

fn render_report(
    allowlist: &AllowlistFile,
    covered: usize,
    rust_skipped: usize,
    unmatched: &[String],
    findings: &[String],
) -> String {
    let mut out = String::new();
    out.push_str("# Non-Rust file policy report\n\n");
    if let Some(name) = &allowlist.policy {
        out.push_str(&format!("- policy: {name}\n"));
    }
    if let Some(owner) = &allowlist.owner {
        out.push_str(&format!("- owner: {owner}\n"));
    }
    if let Some(status) = &allowlist.status {
        out.push_str(&format!("- status: {status}\n"));
    }
    if let Some(updated) = &allowlist.updated {
        out.push_str(&format!("- updated: {updated}\n"));
    }
    out.push_str(&format!("- allow entries: {}\n", allowlist.allow.len()));
    out.push_str(&format!("- non-Rust files covered: {covered}\n"));
    out.push_str(&format!("- Rust files skipped: {rust_skipped}\n"));
    out.push_str(&format!("- unmatched: {}\n", unmatched.len()));
    out.push_str(&format!("- findings: {}\n\n", findings.len()));

    if !unmatched.is_empty() {
        out.push_str("## Unmatched files\n\n");
        for path in unmatched {
            out.push_str(&format!("- {path}\n"));
        }
        out.push('\n');
    }

    if !findings.is_empty() {
        out.push_str("## Findings\n\n");
        for finding in findings {
            out.push_str(&format!("- {finding}\n"));
        }
    }

    out
}

fn workspace_root() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("locate workspace root")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(glob: &str) -> Entry {
        Entry {
            glob: glob.into(),
            kind: "documentation".into(),
            owner: "docs".into(),
            surface: "docs".into(),
            classification: "documentation".into(),
            reason: "test".into(),
            covered_by: vec![],
        }
    }

    #[test]
    fn duplicate_glob_is_finding() {
        let entries = vec![entry("docs/**"), entry("docs/**")];
        let mut findings = Vec::new();
        validate_entries(&entries, &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("duplicate glob")),
            "{findings:?}"
        );
    }

    #[test]
    fn missing_owner_is_finding() {
        let mut e = entry("docs/**");
        e.owner.clear();
        let entries = vec![e];
        let mut findings = Vec::new();
        validate_entries(&entries, &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("missing owner")),
            "{findings:?}"
        );
    }

    #[test]
    fn production_without_covered_by_is_finding() {
        let mut e = entry("Formula/**");
        e.classification = "production".into();
        let entries = vec![e];
        let mut findings = Vec::new();
        validate_entries(&entries, &mut findings);
        assert!(
            findings
                .iter()
                .any(|f| f.contains("needs at least one covered_by")),
            "{findings:?}"
        );
    }
}
