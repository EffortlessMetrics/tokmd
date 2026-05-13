use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::cli::{RiprPrArgs, RiprReviewCommentsArgs};

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";

pub fn run_pr(args: RiprPrArgs) -> Result<()> {
    let root = workspace_root_path()?;
    if args.check {
        check_pr_contract(&root)
    } else {
        produce_pr_evidence(&root, &args.base, &args.head)
    }
}

pub fn run_review_comments(args: RiprReviewCommentsArgs) -> Result<()> {
    let root = workspace_root_path()?;
    if args.check {
        check_review_contract(&root)
    } else {
        produce_review_comments(&root, &args.base, &args.head)
    }
}

fn produce_pr_evidence(root: &Path, base: &str, head: &str) -> Result<()> {
    ensure_git_ref(root, base)?;
    ensure_git_ref(root, head)?;

    let out_dir = root.join(RIPR_PR_DIR);
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let ripr_bin = ripr_bin();

    run_ripr_output(
        root,
        &ripr_bin,
        &[
            "check",
            "--root",
            ".",
            "--base",
            base,
            "--head",
            head,
            "--format",
            "repo-exposure-json",
        ],
        &out_dir.join("repo-exposure.json"),
    )?;
    run_ripr_output(
        root,
        &ripr_bin,
        &[
            "check",
            "--root",
            ".",
            "--base",
            base,
            "--head",
            head,
            "--format",
            "repo-exposure-md",
        ],
        &out_dir.join("repo-exposure.md"),
    )?;

    check_pr_contract(root)
}

fn produce_review_comments(root: &Path, base: &str, head: &str) -> Result<()> {
    ensure_git_ref(root, base)?;
    ensure_git_ref(root, head)?;

    let out_dir = root.join(RIPR_REVIEW_DIR);
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let json_out = out_dir.join("comments.json");
    let ripr_bin = ripr_bin();

    let status = Command::new(&ripr_bin)
        .args([
            "review-comments",
            "--root",
            ".",
            "--base",
            base,
            "--head",
            head,
            "--out",
        ])
        .arg(&json_out)
        .current_dir(root)
        .status()
        .with_context(|| format!("failed to run {ripr_bin} review-comments"))?;

    if !status.success() {
        bail!("{ripr_bin} review-comments failed with {status}");
    }

    check_review_contract(root)
}

fn run_ripr_output(root: &Path, ripr_bin: &str, args: &[&str], out: &Path) -> Result<()> {
    let output = Command::new(ripr_bin)
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin} {}", args.join(" ")))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fs::write(out, output.stdout).with_context(|| format!("write {}", out.display()))
}

fn check_pr_contract(root: &Path) -> Result<()> {
    let json = root.join(RIPR_PR_DIR).join("repo-exposure.json");
    let md = root.join(RIPR_PR_DIR).join("repo-exposure.md");
    require_json(&json)?;
    require_non_empty(&md)?;
    println!("ripr-pr: output contract is intact");
    Ok(())
}

fn check_review_contract(root: &Path) -> Result<()> {
    let json = root.join(RIPR_REVIEW_DIR).join("comments.json");
    let md = root.join(RIPR_REVIEW_DIR).join("comments.md");
    require_json(&json)?;
    require_non_empty(&md)?;
    println!("ripr-review-comments: output contract is intact");
    Ok(())
}

fn require_json(path: &Path) -> Result<Value> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("parse JSON {}", path.display()))
}

fn require_non_empty(path: &Path) -> Result<()> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if body.trim().is_empty() {
        bail!("{} is empty", path.display());
    }
    Ok(())
}

fn ensure_git_ref(root: &Path, revision: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", revision])
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("resolve git revision {revision}"))?;
    if !status.success() {
        bail!("git revision `{revision}` could not be resolved");
    }
    Ok(())
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}

fn workspace_root_path() -> Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("locate workspace root")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

#[cfg(test)]
mod tests {
    use super::{require_json, require_non_empty};

    #[test]
    fn ripr_json_contract_accepts_objects() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("comments.json");
        std::fs::write(&path, "{\"comments\":[]}\n").unwrap();
        require_json(&path).unwrap();
    }

    #[test]
    fn ripr_markdown_contract_rejects_empty_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("comments.md");
        std::fs::write(&path, "\n").unwrap();
        let err = require_non_empty(&path).unwrap_err();
        assert!(err.to_string().contains("is empty"));
    }
}
