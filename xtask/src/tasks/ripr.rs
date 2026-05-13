use crate::cli::{RiprPrArgs, RiprReviewCommentsArgs};
use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RIPR_PR_JSON: &str = "target/ripr/pr/repo-exposure.json";
const RIPR_PR_MD: &str = "target/ripr/pr/repo-exposure.md";
const RIPR_REVIEW_JSON: &str = "target/ripr/review/comments.json";
const RIPR_REVIEW_MD: &str = "target/ripr/review/comments.md";

pub fn run_pr(args: RiprPrArgs) -> Result<()> {
    if args.check {
        return check_pr_contract();
    }

    let workspace_root = workspace_root_path()?;
    let ripr_bin = ripr_bin();
    let output_json = workspace_root.join(RIPR_PR_JSON);
    let output_md = workspace_root.join(RIPR_PR_MD);
    ensure_parent(&output_json)?;

    let status = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&args.base)
        .arg("--head")
        .arg(&args.head)
        .arg("--format")
        .arg("repo-exposure-json")
        .arg("--out")
        .arg(&output_json)
        .current_dir(&workspace_root)
        .status()
        .with_context(|| format!("failed to run {ripr_bin} for PR evidence"))?;

    if !status.success() {
        bail!("{ripr_bin} PR evidence failed with status {status}");
    }

    let status = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&args.base)
        .arg("--head")
        .arg(&args.head)
        .arg("--format")
        .arg("repo-exposure-md")
        .arg("--out")
        .arg(&output_md)
        .current_dir(&workspace_root)
        .status()
        .with_context(|| format!("failed to run {ripr_bin} for PR Markdown evidence"))?;

    if !status.success() {
        bail!("{ripr_bin} PR Markdown evidence failed with status {status}");
    }

    check_pr_contract()
}

pub fn run_review_comments(args: RiprReviewCommentsArgs) -> Result<()> {
    if args.check {
        return check_review_contract();
    }

    let workspace_root = workspace_root_path()?;
    let ripr_bin = ripr_bin();
    let output_json = workspace_root.join(RIPR_REVIEW_JSON);
    ensure_parent(&output_json)?;

    let status = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&args.base)
        .arg("--head")
        .arg(&args.head)
        .arg("--out")
        .arg(&output_json)
        .current_dir(&workspace_root)
        .status()
        .with_context(|| format!("failed to run {ripr_bin} review-comments"))?;

    if !status.success() {
        bail!("{ripr_bin} review-comments failed with status {status}");
    }

    check_review_contract()
}

fn check_pr_contract() -> Result<()> {
    let root = workspace_root_path()?;
    require_json(&root.join(RIPR_PR_JSON))?;
    require_nonempty_file(&root.join(RIPR_PR_MD))?;
    println!("ripr-pr: output contract is intact");
    Ok(())
}

fn check_review_contract() -> Result<()> {
    let root = workspace_root_path()?;
    require_json(&root.join(RIPR_REVIEW_JSON))?;
    require_nonempty_file(&root.join(RIPR_REVIEW_MD))?;
    println!("ripr-review-comments: output contract is intact");
    Ok(())
}

fn require_json(path: &Path) -> Result<Value> {
    require_nonempty_file(path)?;
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("parse JSON {}", path.display()))
}

fn require_nonempty_file(path: &Path) -> Result<()> {
    let metadata =
        fs::metadata(path).with_context(|| format!("missing required file {}", path.display()))?;
    if !metadata.is_file() || metadata.len() == 0 {
        bail!("required file is empty or not a file: {}", path.display());
    }
    Ok(())
}

fn workspace_root_path() -> Result<PathBuf> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to load cargo metadata")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent directory {}", parent.display()))?;
    }
    Ok(())
}
