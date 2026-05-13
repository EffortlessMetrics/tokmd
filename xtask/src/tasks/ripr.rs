use crate::cli::{RiprPrArgs, RiprReviewCommentsArgs};
use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use std::path::{Path, PathBuf};
use std::process::Command;

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";

pub fn run_pr(args: RiprPrArgs) -> Result<()> {
    if args.check {
        return check_pr_contract(&workspace_root_path()?);
    }

    let root = workspace_root_path()?;
    let out_dir = root.join(RIPR_PR_DIR);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let json = out_dir.join("repo-exposure.json");
    let md = out_dir.join("repo-exposure.md");
    run_ripr(
        &root,
        &[
            "check".to_string(),
            "--root".to_string(),
            root.display().to_string(),
            "--base".to_string(),
            args.base.clone(),
            "--head".to_string(),
            args.head.clone(),
            "--format".to_string(),
            "repo-exposure-json".to_string(),
            "--out".to_string(),
            json.display().to_string(),
        ],
    )?;
    run_ripr(
        &root,
        &[
            "check".to_string(),
            "--root".to_string(),
            root.display().to_string(),
            "--base".to_string(),
            args.base.clone(),
            "--head".to_string(),
            args.head.clone(),
            "--format".to_string(),
            "repo-exposure-md".to_string(),
            "--out".to_string(),
            md.display().to_string(),
        ],
    )?;

    check_pr_contract(&root)
}

pub fn run_review_comments(args: RiprReviewCommentsArgs) -> Result<()> {
    if args.check {
        return check_review_contract(&workspace_root_path()?);
    }

    let root = workspace_root_path()?;
    let out_dir = root.join(RIPR_REVIEW_DIR);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;
    let out = out_dir.join("comments.json");

    run_ripr(
        &root,
        &[
            "review-comments".to_string(),
            "--root".to_string(),
            root.display().to_string(),
            "--base".to_string(),
            args.base.clone(),
            "--head".to_string(),
            args.head.clone(),
            "--out".to_string(),
            out.display().to_string(),
        ],
    )?;

    check_review_contract(&root)
}

fn run_ripr(root: &Path, args: &[String]) -> Result<()> {
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());
    let output = Command::new(&ripr_bin)
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin}; set RIPR_BIN to override"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn check_pr_contract(root: &Path) -> Result<()> {
    let json = root.join(RIPR_PR_DIR).join("repo-exposure.json");
    let md = root.join(RIPR_PR_DIR).join("repo-exposure.md");
    read_json(&json)?;
    require_nonempty(&md)?;
    println!("ripr-pr: output contract is intact");
    Ok(())
}

fn check_review_contract(root: &Path) -> Result<()> {
    let json = root.join(RIPR_REVIEW_DIR).join("comments.json");
    let md = root.join(RIPR_REVIEW_DIR).join("comments.md");
    read_json(&json)?;
    require_nonempty(&md)?;
    println!("ripr-review-comments: output contract is intact");
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("missing required RIPR JSON artifact {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("invalid RIPR JSON artifact {}", path.display()))
}

fn require_nonempty(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("missing required RIPR Markdown artifact {}", path.display()))?;
    if metadata.len() == 0 {
        bail!("RIPR Markdown artifact is empty: {}", path.display());
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
