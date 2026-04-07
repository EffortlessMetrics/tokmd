use crate::cli::FixtureBlobsCheckArgs;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

const ALLOWLIST_PREFIXES: &[&str] = &[".claude/", ".jules/", "vendor/"];
const ALLOWLIST_PATHS: &[&str] = &["xtask/src/tasks/fixture_blobs_check.rs"];
const FORBIDDEN_EXTENSIONS: &[&str] = &[
    "cer", "crt", "der", "jwk", "jwks", "key", "p12", "p8", "pem", "pfx", "pk8",
];
const FORBIDDEN_MARKERS: &[&str] = &[
    "BEGIN CERTIFICATE",
    "BEGIN EC PRIVATE KEY",
    "BEGIN OPENSSH PRIVATE KEY",
    "BEGIN PRIVATE KEY",
    "BEGIN RSA PRIVATE KEY",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    path: String,
    reason: String,
}

fn is_allowlisted(path: &str) -> bool {
    if ALLOWLIST_PATHS.contains(&path) {
        return true;
    }

    ALLOWLIST_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn forbidden_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| FORBIDDEN_EXTENSIONS.contains(&ext.as_str()))
}

fn contains_forbidden_marker(path: &Path) -> Result<Option<&'static str>> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(FORBIDDEN_MARKERS
        .iter()
        .copied()
        .find(|marker| text.contains(marker)))
}

fn evaluate_candidate(repo_root: &Path, rel_path: &str) -> Result<Option<Violation>> {
    if is_allowlisted(rel_path) {
        return Ok(None);
    }

    if let Some(ext) = forbidden_extension(rel_path) {
        return Ok(Some(Violation {
            path: rel_path.to_string(),
            reason: format!(
                "committed crypto fixture blob extension .{ext} is forbidden; generate fixtures at runtime or explicitly whitelist the path"
            ),
        }));
    }

    let abs_path = repo_root.join(rel_path);
    if let Some(marker) = contains_forbidden_marker(&abs_path)? {
        return Ok(Some(Violation {
            path: rel_path.to_string(),
            reason: format!(
                "committed crypto fixture marker '{marker}' is forbidden; generate fixtures at runtime or explicitly whitelist the path"
            ),
        }));
    }

    Ok(None)
}

fn tracked_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .output()
        .context("failed to list tracked files with git ls-files")?;

    if !output.status.success() {
        bail!("git ls-files did not succeed");
    }

    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).to_string())
        .collect())
}

fn collect_violations(repo_root: &Path, tracked: &[String]) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for rel_path in tracked {
        if let Some(violation) = evaluate_candidate(repo_root, rel_path)? {
            violations.push(violation);
        }
    }

    Ok(violations)
}

pub fn run(_args: FixtureBlobsCheckArgs) -> Result<()> {
    let repo_root = std::env::current_dir()?;
    let tracked = tracked_files()?;
    let violations = collect_violations(&repo_root, &tracked)?;

    if violations.is_empty() {
        println!("No committed crypto fixture blobs found");
        return Ok(());
    }

    eprintln!("Committed crypto fixture blobs detected:");
    for violation in &violations {
        eprintln!("  - {} ({})", violation.path, violation.reason);
        eprintln!("::error file={}::{}", violation.path, violation.reason);
    }

    bail!(
        "found {} committed crypto fixture blob(s); use deterministic runtime fixtures instead",
        violations.len()
    );
}

#[cfg(test)]
mod tests {
    use super::{collect_violations, evaluate_candidate, forbidden_extension};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_forbidden_extension() {
        assert_eq!(forbidden_extension("fixtures/key.pem"), Some("pem".into()));
        assert_eq!(forbidden_extension("fixtures/key.PK8"), Some("pk8".into()));
        assert_eq!(forbidden_extension("src/lib.rs"), None);
    }

    #[test]
    fn skips_allowlisted_vendor_paths() {
        let dir = tempdir().expect("tempdir");
        let vendor_file = dir.path().join("vendor").join("fixture.pem");
        fs::create_dir_all(vendor_file.parent().unwrap()).expect("vendor dir");
        fs::write(&vendor_file, "BEGIN PRIVATE KEY").expect("write");

        let violation = evaluate_candidate(dir.path(), "vendor/fixture.pem").expect("check");
        assert!(violation.is_none());
    }

    #[test]
    fn detects_forbidden_marker_in_text_file() {
        let dir = tempdir().expect("tempdir");
        let manifest = dir.path().join("docs").join("example.md");
        fs::create_dir_all(manifest.parent().unwrap()).expect("docs dir");
        fs::write(&manifest, "-----BEGIN PRIVATE KEY-----").expect("write");

        let violation = evaluate_candidate(dir.path(), "docs/example.md")
            .expect("check")
            .expect("violation");

        assert_eq!(violation.path, "docs/example.md");
        assert!(violation.reason.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn allows_checker_source_file() {
        let dir = tempdir().expect("tempdir");

        let violation = evaluate_candidate(dir.path(), "xtask/src/tasks/fixture_blobs_check.rs")
            .expect("check");

        assert!(violation.is_none());
    }

    #[test]
    fn collects_violations_across_multiple_paths() {
        let dir = tempdir().expect("tempdir");
        let pem = dir.path().join("fixtures").join("bad.pem");
        let readme = dir.path().join("README.md");
        fs::create_dir_all(pem.parent().unwrap()).expect("fixtures dir");
        fs::write(&pem, "ignored due to extension").expect("write pem");
        fs::write(&readme, "no secrets here").expect("write readme");

        let tracked = vec!["fixtures/bad.pem".to_string(), "README.md".to_string()];
        let violations = collect_violations(dir.path(), &tracked).expect("collect");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].path, "fixtures/bad.pem");
    }
}
