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

/// Directories skipped when they appear as the first path component under the
/// workspace root.
const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", "run-artifacts", "plans"];

/// Directories skipped at any depth.
///
/// Cargo writes build output to a `target/` directory next to every manifest it
/// builds, not only at the workspace root: this repo has `xtask/target/`,
/// `crates/*/target/`, and `fuzz/target/`, all of which `.gitignore` already
/// excludes. Matching only the first path component would let whatever the last
/// `cargo test`/`cargo build` happened to leave behind decide the outcome of
/// this check, so the walk skips these names wherever they occur.
const SKIP_DIRS_ANY_DEPTH: &[&str] = &["target"];

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
    let mut components = rel.iter().filter_map(|s| s.to_str());
    let Some(first) = components.next() else {
        return false;
    };
    if SKIP_DIRS.contains(&first) {
        return true;
    }
    // Includes `first` deliberately. `SKIP_DIRS_ANY_DEPTH` is currently a
    // subset of `SKIP_DIRS`, so a separate first-component test would be dead
    // code today; folding it into the any-depth scan keeps the first component
    // covered if the two lists ever diverge.
    std::iter::once(first)
        .chain(components)
        .any(|c| SKIP_DIRS_ANY_DEPTH.contains(&c))
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
    fn skips_root_level_skip_dirs() {
        let root = Path::new("/repo");
        for dir in SKIP_DIRS {
            let path = Path::new("/repo").join(dir).join("some-file.json");
            assert!(is_skipped(&path, root), "expected {dir}/ to be skipped");
        }
    }

    /// Skipping `target` at any depth is only safe while no tracked file lives
    /// under such a path. That held when the rule was introduced, but it is an
    /// assumption about repository contents rather than about this module, so
    /// enforce it instead of trusting a one-time check: a fixture added under
    /// e.g. `fixtures/target/` would otherwise vanish from the policy walk
    /// silently.
    #[test]
    fn no_tracked_file_lives_under_a_target_component() {
        let Ok(root) = workspace_root() else {
            return;
        };
        let Ok(output) = std::process::Command::new("git")
            .arg("ls-files")
            .current_dir(&root)
            .output()
        else {
            // git unavailable: nothing to assert against.
            return;
        };
        if !output.status.success() {
            return;
        }
        let listing = String::from_utf8_lossy(&output.stdout);
        let offenders: Vec<&str> = listing
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| line.split('/').any(|c| SKIP_DIRS_ANY_DEPTH.contains(&c)))
            .collect();
        assert!(
            offenders.is_empty(),
            "tracked files live under a skipped build-output component and would be \
             invisible to the file-policy walk: {offenders:?}"
        );
    }

    #[test]
    fn skips_nested_cargo_target_dirs() {
        let root = Path::new("/repo");
        // Cargo writes a target/ next to every manifest it builds. Before this
        // was handled, a stale xtask/target/ from `cargo test -p xtask` turned
        // an otherwise-clean `check-file-policy --strict` into 11 findings.
        for rel in [
            "xtask/target/test-proof-observation-status/aggregate/affected.json",
            "crates/tokmd-format/target/debug/build-output.json",
            "fuzz/target/release/artifact.json",
        ] {
            let path = root.join(rel);
            assert!(is_skipped(&path, root), "expected {rel} to be skipped");
        }
    }

    #[test]
    fn does_not_skip_tracked_files_outside_build_dirs() {
        let root = Path::new("/repo");
        for rel in [
            "docs/plans/ast-productization.md",
            "fixtures/ast-shadow/python/basic.py",
            "policy/non-rust-allowlist.toml",
            ".gitignore",
        ] {
            let path = root.join(rel);
            assert!(!is_skipped(&path, root), "expected {rel} to be walked");
        }
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
