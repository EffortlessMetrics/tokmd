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

    /// Whether a failed `git ls-files` means "there is no repository here"
    /// rather than "this repository is broken".
    ///
    /// Anchored to the start of git's fatal line rather than searched across
    /// the whole of stderr: the dubious-ownership message interpolates the
    /// repository path, so a bare substring test reads a checkout living under
    /// a directory named "not a git repository" as having no repository at all
    /// -- silently skipping the case most worth asserting on.
    fn stderr_means_no_repository(stderr: &str) -> bool {
        stderr
            .lines()
            .any(|line| line.starts_with("fatal: not a git repository"))
    }

    /// The repo's tracked files, or `None` when the invariant cannot be
    /// evaluated here.
    ///
    /// Unevaluable environments and real errors are deliberately treated
    /// differently, and the line between them is not simply git's exit code.
    /// A failing `git ls-files` lands in one of two buckets:
    ///
    /// - **Skipped**, because the invariant cannot be evaluated: git is
    ///   absent, or the tree is not a git repository at all. An exported
    ///   source archive has no `.git`, and asking `git ls-files` there fails
    ///   -- treating that as a fault would fail `cargo test` on a perfectly
    ///   healthy archive tree.
    /// - **Asserted**, because the tree *is* a repository and something is
    ///   wrong with it: a container tripping `safe.directory` ownership
    ///   checks, or a corrupt index. Swallowing these would let a repository
    ///   that violates the invariant report green.
    ///
    /// All three exit 128, so the exit code cannot carry the distinction, and
    /// neither can a `rev-parse --is-inside-work-tree` probe: repository
    /// discovery runs the same ownership check, so a `safe.directory` trip
    /// fails the probe exactly as it fails `ls-files` -- the very case most
    /// worth asserting on would have been silently skipped. (A corrupt index
    /// does *not* fail the probe, since `rev-parse` never reads the index, so
    /// a probe would have split these two apart for no good reason.) What
    /// actually separates them is git's own message, so that is what is
    /// matched, under `LC_ALL=C` to keep the marker stable when a translation
    /// catalog is installed.
    fn tracked_files_for_policy(root: &Path) -> Result<Option<String>> {
        // `NotFound` from `Command::output` is ambiguous: a missing `git`
        // executable and a missing `current_dir` both surface as ENOENT from
        // the same call, so the spawn error alone cannot tell them apart.
        // Rule the bad root out here, or it would take the git-is-absent arm
        // below and skip the invariant instead of reporting the fault.
        if !root.is_dir() {
            anyhow::bail!(
                "cannot check the tracked-file invariant: {} is not a directory",
                root.to_string_lossy().replace('\\', "/")
            );
        }
        let output = match std::process::Command::new("git")
            .arg("ls-files")
            .current_dir(root)
            .env("LC_ALL", "C")
            .output()
        {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => anyhow::bail!(
                "failed to spawn `git ls-files` while checking tracked files: {error}"
            ),
        };
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr_means_no_repository(&stderr) {
                return Ok(None);
            }
            let root_display = root.to_string_lossy().replace('\\', "/");
            anyhow::bail!(
                "`git ls-files` failed with {} in the repository at {root_display}; the \
                 tracked-file invariant could not be checked. stderr: {}",
                output.status,
                stderr.trim()
            );
        }
        Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
    }

    /// Whether a `workspace_root` failure means "cargo is not installed here"
    /// rather than "this workspace is broken".
    ///
    /// `cargo_metadata` surfaces a missing executable as `Error::Io` with
    /// `NotFound`; a manifest or workspace fault arrives as a different
    /// variant. `anyhow` keeps the concrete error in the source chain, so the
    /// distinction survives the `context` wrapper in `workspace_root`.
    fn cargo_metadata_is_unavailable(error: &anyhow::Error) -> bool {
        matches!(
            error.downcast_ref::<cargo_metadata::Error>(),
            Some(cargo_metadata::Error::Io(io)) if io.kind() == std::io::ErrorKind::NotFound
        )
    }

    /// Tracked paths that live under a component the policy walk skips.
    ///
    /// Split out so the detection has a red state: run against the real
    /// repository it is always empty, which cannot distinguish "no offenders"
    /// from "the filter never matches anything".
    fn tracked_offenders(listing: &str) -> Vec<&str> {
        listing
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| line.split('/').any(|c| SKIP_DIRS_ANY_DEPTH.contains(&c)))
            .collect()
    }

    #[test]
    fn tracked_offenders_finds_files_under_a_skipped_component() {
        // The positive case the repo itself cannot provide. Without this, a
        // filter that matched nothing would pass the invariant test forever.
        let listing = "src/main.rs\nfixtures/target/blob.json\ndocs/guide.md\ntarget/out.bin\n";
        assert_eq!(
            tracked_offenders(listing),
            vec!["fixtures/target/blob.json", "target/out.bin"]
        );
        // A clean listing must stay clean -- including paths that merely
        // contain the component name as a substring.
        assert!(tracked_offenders("src/targeting.rs\ndocs/on-target.md\n").is_empty());
    }

    #[test]
    fn a_real_git_ls_files_error_is_not_skipped() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let initialized = match std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(temp.path())
            .env("LC_ALL", "C")
            .output()
        {
            Ok(output) => output,
            // Only a missing git makes this witness impossible. Swallowing
            // every spawn error here would quietly turn the one test that
            // proves failures are not skipped into a no-op -- the same
            // conflation it exists to catch.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                anyhow::bail!("failed to spawn `git init` for the corrupt-index case: {error}")
            }
        };
        if !initialized.status.success() {
            anyhow::bail!(
                "git init failed: {}",
                String::from_utf8_lossy(&initialized.stderr).trim()
            );
        }
        fs::write(temp.path().join(".git/index"), b"corrupt")?;

        let error = tracked_files_for_policy(temp.path())
            .expect_err("a corrupt index is a real git failure, not an unevaluable source archive");
        assert!(error.to_string().contains("git ls-files"));
        Ok(())
    }

    #[test]
    fn only_a_missing_repository_counts_as_unevaluable() {
        // Verbatim `git ls-files` stderr, measured under `LC_ALL=C`. All three
        // real cases exit 128, which is why the exit code cannot carry this
        // distinction and the message has to.
        assert!(stderr_means_no_repository(
            "fatal: not a git repository (or any of the parent directories): .git\n"
        ));
        assert!(!stderr_means_no_repository(
            "fatal: detected dubious ownership in repository at '/repo'\n"
        ));
        assert!(!stderr_means_no_repository(
            "fatal: .git/index: index file smaller than expected\n"
        ));
        // The repository path is interpolated into the ownership message, so a
        // checkout that happens to live under this name is still a real fault.
        assert!(!stderr_means_no_repository(
            "fatal: detected dubious ownership in repository at '/tmp/not a git repository/wt'\n"
        ));
        assert!(!stderr_means_no_repository(
            "fatal: bad config line 1 in file .git/config\n"
        ));
        // `str::lines` strips the trailing `\r`, so the anchor still matches
        // when git writes CRLF; pinned because the prefix test would otherwise
        // be sensitive to a line ending nobody thinks about.
        assert!(stderr_means_no_repository(
            "fatal: not a git repository (or any of the parent directories): .git\r\n"
        ));
        // The marker is honoured on any line, not just the first.
        assert!(stderr_means_no_repository(
            "warning: unable to access config\nfatal: not a git repository (or any of the parent directories): .git\n"
        ));
    }

    #[test]
    fn source_archive_without_git_is_unevaluable() -> Result<()> {
        let archive = tempfile::tempdir()?;
        assert!(tracked_files_for_policy(archive.path())?.is_none());
        Ok(())
    }

    #[test]
    fn a_missing_root_is_not_skipped() -> Result<()> {
        // A root that does not exist makes `Command::output` fail with the same
        // `NotFound` that a missing `git` produces, so without the explicit
        // directory check this would report unevaluable rather than a fault.
        // Bind the `TempDir`: the parent has to outlive the call so that only
        // `gone` is missing. Joining onto a temporary would drop the directory
        // at the end of the statement and pass for the wrong reason.
        let parent = tempfile::tempdir()?;
        let absent = parent.path().join("gone");
        assert!(parent.path().is_dir(), "the parent must exist");
        assert!(!absent.exists(), "only the joined name must be missing");
        let error = tracked_files_for_policy(&absent)
            .expect_err("a nonexistent root is a caller error, not an unevaluable environment");
        assert!(error.to_string().contains("is not a directory"));
        Ok(())
    }

    /// Skipping `target` at any depth is only safe while no tracked file lives
    /// under such a path. That held when the rule was introduced, but it is an
    /// assumption about repository contents rather than about this module, so
    /// enforce it instead of trusting a one-time check: a fixture added under
    /// e.g. `fixtures/target/` would otherwise vanish from the policy walk
    /// silently.
    #[test]
    fn no_tracked_file_lives_under_a_target_component() -> Result<()> {
        let root = match workspace_root() {
            Ok(root) => root,
            // Only an absent `cargo` makes the workspace root unknowable. Any
            // other metadata failure -- a malformed manifest, an unreadable
            // workspace -- is a real fault, and swallowing it here would skip
            // the invariant for the same reason this test exists to reject.
            Err(error) if cargo_metadata_is_unavailable(&error) => return Ok(()),
            Err(error) => return Err(error),
        };
        let Some(listing) = tracked_files_for_policy(&root)? else {
            return Ok(());
        };
        let offenders = tracked_offenders(&listing);
        assert!(
            offenders.is_empty(),
            "tracked files live under a skipped build-output component and would be \
             invisible to the file-policy walk: {offenders:?}"
        );
        Ok(())
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
