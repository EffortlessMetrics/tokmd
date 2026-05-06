use crate::cli::FixtureBlobsCheckArgs;
use crate::proof::policy::load_policy;
use crate::proof::policy_ast::{FixtureBlobRule, ProofPolicy};
use crate::proof::validate::validate_policy;
use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Repo-owned proof policy path. Fixture blob rules live here so the crypto
/// guardrail stays aligned with the broader proof control plane.
const PROOF_POLICY_PATH: &str = "ci/proof.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    path: String,
    reason: String,
}

#[derive(Debug)]
struct FixtureBlobPolicy {
    rules: Vec<FixtureBlobRule>,
    allow: GlobSet,
}

fn load_checked_policy(path: &Path) -> Result<ProofPolicy> {
    let policy = load_policy(path)?;
    let violations = validate_policy(&policy);
    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("::error file={}::{}", path.display(), violation.message);
        }
        bail!(
            "proof policy is invalid; run `cargo xtask proof-policy --check` before fixture-blobs-check"
        );
    }
    Ok(policy)
}

fn fixture_blob_policy(policy: &ProofPolicy) -> Result<FixtureBlobPolicy> {
    if policy.forbid.fixture_blob.is_empty() {
        bail!("proof policy must include at least one forbid.fixture_blob rule");
    }

    let mut allow_builder = GlobSetBuilder::new();
    for rule in &policy.forbid.fixture_blob {
        for pattern in &rule.allow {
            allow_builder.add(
                Glob::new(pattern)
                    .with_context(|| format!("invalid fixture blob allow glob `{pattern}`"))?,
            );
        }
    }

    Ok(FixtureBlobPolicy {
        rules: policy.forbid.fixture_blob.clone(),
        allow: allow_builder
            .build()
            .context("failed to build fixture blob allowlist")?,
    })
}

fn is_allowlisted(policy: &FixtureBlobPolicy, path: &str) -> bool {
    policy.allow.is_match(path)
}

fn forbidden_extension(policy: &FixtureBlobPolicy, path: &str) -> Option<String> {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())?;

    policy
        .rules
        .iter()
        .any(|rule| {
            rule.extensions
                .iter()
                .any(|forbidden| forbidden.eq_ignore_ascii_case(&extension))
        })
        .then_some(extension)
}

fn contains_forbidden_marker(policy: &FixtureBlobPolicy, path: &Path) -> Result<Option<String>> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(policy
        .rules
        .iter()
        .flat_map(|rule| &rule.markers)
        .find(|marker| text.contains(marker.as_str()))
        .cloned())
}

fn evaluate_candidate(
    policy: &FixtureBlobPolicy,
    repo_root: &Path,
    rel_path: &str,
) -> Result<Option<Violation>> {
    if is_allowlisted(policy, rel_path) {
        return Ok(None);
    }

    let abs_path = repo_root.join(rel_path);
    if !abs_path.is_file() {
        return Ok(None);
    }

    if let Some(ext) = forbidden_extension(policy, rel_path) {
        return Ok(Some(Violation {
            path: rel_path.to_string(),
            reason: format!(
                "committed crypto fixture blob extension .{ext} is forbidden; generate fixtures at runtime or explicitly whitelist the path"
            ),
        }));
    }

    if let Some(marker) = contains_forbidden_marker(policy, &abs_path)? {
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

fn collect_violations(
    policy: &FixtureBlobPolicy,
    repo_root: &Path,
    tracked: &[String],
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for rel_path in tracked {
        if let Some(violation) = evaluate_candidate(policy, repo_root, rel_path)? {
            violations.push(violation);
        }
    }

    Ok(violations)
}

pub fn run(_args: FixtureBlobsCheckArgs) -> Result<()> {
    let repo_root = std::env::current_dir()?;
    let policy = load_checked_policy(&repo_root.join(PROOF_POLICY_PATH))?;
    let fixture_policy = fixture_blob_policy(&policy)?;
    let tracked = tracked_files()?;
    let violations = collect_violations(&fixture_policy, &repo_root, &tracked)?;

    if violations.is_empty() {
        println!(
            "No committed crypto fixture blobs found ({} policy rule(s))",
            fixture_policy.rules.len()
        );
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
    use super::{
        collect_violations, evaluate_candidate, fixture_blob_policy, forbidden_extension,
        load_checked_policy,
    };
    use crate::proof::policy::parse_policy_str;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf()
    }

    fn test_policy() -> super::FixtureBlobPolicy {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[[forbid.fixture_blob]]
name = "committed_crypto_fixture_blobs"
extensions = ["pem", "pk8"]
markers = ["BEGIN PRIVATE KEY"]
allow = [".claude/**", ".jules/**", "vendor/**", "xtask/src/tasks/fixture_blobs_check.rs"]
reason = "Crypto material should be generated at runtime or explicitly documented."
"#,
        )
        .expect("policy should parse");

        fixture_blob_policy(&policy).expect("fixture policy should build")
    }

    #[test]
    fn detects_forbidden_extension() {
        let policy = test_policy();

        assert_eq!(
            forbidden_extension(&policy, "fixtures/key.pem"),
            Some("pem".into())
        );
        assert_eq!(
            forbidden_extension(&policy, "fixtures/key.PK8"),
            Some("pk8".into())
        );
        assert_eq!(forbidden_extension(&policy, "src/lib.rs"), None);
    }

    #[test]
    fn skips_allowlisted_vendor_paths() {
        let policy = test_policy();
        let dir = tempdir().expect("tempdir");
        let vendor_file = dir.path().join("vendor").join("fixture.pem");
        fs::create_dir_all(vendor_file.parent().unwrap()).expect("vendor dir");
        fs::write(&vendor_file, "BEGIN PRIVATE KEY").expect("write");

        let violation =
            evaluate_candidate(&policy, dir.path(), "vendor/fixture.pem").expect("check");
        assert!(violation.is_none());
    }

    #[test]
    fn skips_allowlisted_jules_and_claude_paths() {
        let policy = test_policy();
        let dir = tempdir().expect("tempdir");

        for rel_path in [".jules/run/fixture.pem", ".claude/context/fixture.pem"] {
            let abs_path = dir.path().join(rel_path);
            fs::create_dir_all(abs_path.parent().unwrap()).expect("allowlist dir");
            fs::write(&abs_path, "BEGIN PRIVATE KEY").expect("write");

            let violation = evaluate_candidate(&policy, dir.path(), rel_path).expect("check");
            assert!(violation.is_none(), "{rel_path} should be allowlisted");
        }
    }

    #[test]
    fn detects_forbidden_marker_in_text_file() {
        let policy = test_policy();
        let dir = tempdir().expect("tempdir");
        let manifest = dir.path().join("docs").join("example.md");
        fs::create_dir_all(manifest.parent().unwrap()).expect("docs dir");
        fs::write(&manifest, "-----BEGIN PRIVATE KEY-----").expect("write");

        let violation = evaluate_candidate(&policy, dir.path(), "docs/example.md")
            .expect("check")
            .expect("violation");

        assert_eq!(violation.path, "docs/example.md");
        assert!(violation.reason.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn allows_checker_source_file() {
        let policy = test_policy();
        let dir = tempdir().expect("tempdir");

        let violation = evaluate_candidate(
            &policy,
            dir.path(),
            "xtask/src/tasks/fixture_blobs_check.rs",
        )
        .expect("check");

        assert!(violation.is_none());
    }

    #[test]
    fn collects_violations_across_multiple_paths() {
        let policy = test_policy();
        let dir = tempdir().expect("tempdir");
        let pem = dir.path().join("fixtures").join("bad.pem");
        let readme = dir.path().join("README.md");
        fs::create_dir_all(pem.parent().unwrap()).expect("fixtures dir");
        fs::write(&pem, "ignored due to extension").expect("write pem");
        fs::write(&readme, "no secrets here").expect("write readme");

        let tracked = vec!["fixtures/bad.pem".to_string(), "README.md".to_string()];
        let violations = collect_violations(&policy, dir.path(), &tracked).expect("collect");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].path, "fixtures/bad.pem");
    }

    #[test]
    fn repo_policy_loads_fixture_blob_rule() {
        let policy = load_checked_policy(&workspace_root().join("ci/proof.toml"))
            .expect("repo proof policy should load");
        let fixture_policy =
            fixture_blob_policy(&policy).expect("repo fixture policy should build");

        assert!(fixture_policy.rules.iter().any(|rule| {
            rule.name == "committed_crypto_fixture_blobs"
                && rule.extensions.iter().any(|extension| extension == "pem")
                && rule
                    .markers
                    .iter()
                    .any(|marker| marker == "BEGIN PRIVATE KEY")
        }));
    }

    #[test]
    fn missing_policy_is_reported() {
        let dir = tempdir().expect("tempdir");
        let err = load_checked_policy(&dir.path().join("missing.toml"))
            .expect_err("missing policy should fail");

        assert!(err.to_string().contains("failed to read"));
    }

    #[test]
    fn malformed_policy_is_reported() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("proof.toml");
        fs::write(&path, "schema = [").expect("write malformed policy");

        let err = load_checked_policy(&path).expect_err("malformed policy should fail");

        assert!(err.to_string().contains("failed to parse"));
    }
}
