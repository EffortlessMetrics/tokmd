//! RIPR pull-request evidence wrappers.
//!
//! These commands keep PR evidence diff-scoped and write artifacts under
//! `target/ripr/` for CI summaries and uploads.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cli::{RiprAnnotationsArgs, RiprPrArgs, RiprReviewCommentsArgs};

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";

pub fn run_pr(args: RiprPrArgs) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let out_dir = workspace_root.join(RIPR_PR_DIR);

    if args.check {
        check_pr_contract(&out_dir)?;
        println!("ripr-pr: output contract is intact");
        return Ok(());
    }

    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    run_ripr_check_format(
        &workspace_root,
        &args.base,
        "repo-exposure-json",
        &out_dir.join("repo-exposure.json"),
    )?;
    run_ripr_check_format(
        &workspace_root,
        &args.base,
        "repo-exposure-md",
        &out_dir.join("repo-exposure.md"),
    )?;
    check_pr_contract(&out_dir)?;
    println!("ripr-pr: wrote {}", out_dir.display());
    Ok(())
}

pub fn run_annotations(args: RiprAnnotationsArgs) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let comments_path = workspace_root.join(args.comments);

    if !comments_path.exists() {
        eprintln!("::warning::No RIPR review comments JSON found; skipping annotations.");
        return Ok(());
    }

    let body = match fs::read_to_string(&comments_path) {
        Ok(body) => body,
        Err(error) => {
            eprintln!(
                "::warning::{}",
                escape_data(&format!(
                    "Unable to read RIPR review comments JSON in {}: {error}",
                    comments_path.display()
                ))
            );
            return Ok(());
        }
    };
    let value = match serde_json::from_str::<Value>(&body) {
        Ok(value) => value,
        Err(error) => {
            eprintln!(
                "::warning::{}",
                escape_data(&format!(
                    "Invalid RIPR review comments JSON in {}: {error}",
                    comments_path.display()
                ))
            );
            return Ok(());
        }
    };

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    emit_annotations(&value, &mut handle)?;
    Ok(())
}

pub fn run_review_comments(args: RiprReviewCommentsArgs) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let out_dir = workspace_root.join(RIPR_REVIEW_DIR);
    let json_path = out_dir.join("comments.json");
    let md_path = out_dir.join("comments.md");

    if args.check {
        check_review_contract(&json_path, &md_path)?;
        println!("ripr-review-comments: output contract is intact");
        return Ok(());
    }

    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let ripr_bin = ripr_bin();
    let output = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&args.base)
        .arg("--head")
        .arg(&args.head)
        .arg("--out")
        .arg(&json_path)
        .current_dir(&workspace_root)
        .output()
        .with_context(|| format!("run {ripr_bin} review-comments"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} review-comments failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    ensure_review_markdown(&json_path, &md_path)?;
    check_review_contract(&json_path, &md_path)?;
    println!("ripr-review-comments: wrote {}", out_dir.display());
    Ok(())
}

fn emit_annotations<W: Write>(value: &Value, writer: &mut W) -> Result<()> {
    let Some(comments) = value.get("comments").and_then(Value::as_array) else {
        return Ok(());
    };

    for item in comments {
        let file = item
            .get("path")
            .or_else(|| item.get("file"))
            .and_then(command_value);
        let line = item.get("line").and_then(command_value);
        let title = item
            .get("title")
            .and_then(command_value)
            .unwrap_or_else(|| "RIPR".to_string());
        let body = item
            .get("body")
            .or_else(|| item.get("message"))
            .and_then(command_value)
            .unwrap_or_default();

        let (Some(file), Some(line)) = (file, line) else {
            continue;
        };

        writeln!(
            writer,
            "::warning file={},line={},title={}::{}",
            escape_property(&file),
            escape_property(&line),
            escape_property(&title),
            escape_data(&body)
        )
        .context("write RIPR annotation")?;
    }

    Ok(())
}

fn command_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) if value.is_empty() => None,
        Value::String(value) => Some(value.clone()),
        Value::Bool(false) => None,
        Value::Bool(true) => Some("true".to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Array(_) | Value::Object(_) => Some(value.to_string()),
    }
}

fn escape_data(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
}

fn escape_property(value: &str) -> String {
    escape_data(value).replace('=', "%3D").replace(',', "%2C")
}

fn run_ripr_check_format(
    workspace_root: &Path,
    base: &str,
    format: &str,
    output_path: &Path,
) -> Result<()> {
    let ripr_bin = ripr_bin();
    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(workspace_root)
        .arg("--base")
        .arg(base)
        .arg("--format")
        .arg(format)
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("run {ripr_bin} check --format {format}"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} check --format {format} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fs::write(output_path, output.stdout)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(())
}

fn check_pr_contract(out_dir: &Path) -> Result<()> {
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    let json_body =
        fs::read_to_string(&json_path).with_context(|| format!("read {}", json_path.display()))?;
    serde_json::from_str::<Value>(&json_body)
        .with_context(|| format!("parse {}", json_path.display()))?;

    let md_body =
        fs::read_to_string(&md_path).with_context(|| format!("read {}", md_path.display()))?;
    if md_body.trim().is_empty() {
        bail!("{} is empty", md_path.display());
    }

    Ok(())
}

fn check_review_contract(json_path: &Path, md_path: &Path) -> Result<()> {
    let json_body =
        fs::read_to_string(json_path).with_context(|| format!("read {}", json_path.display()))?;
    let value = serde_json::from_str::<Value>(&json_body)
        .with_context(|| format!("parse {}", json_path.display()))?;
    for key in ["comments", "summary_only", "suppressed", "warnings"] {
        if value.get(key).is_none() {
            bail!("{} is missing `{key}`", json_path.display());
        }
    }

    let md_body =
        fs::read_to_string(md_path).with_context(|| format!("read {}", md_path.display()))?;
    if md_body.trim().is_empty() {
        bail!("{} is empty", md_path.display());
    }

    Ok(())
}

fn ensure_review_markdown(json_path: &Path, md_path: &Path) -> Result<()> {
    if md_path.exists() {
        return Ok(());
    }

    let json_body =
        fs::read_to_string(json_path).with_context(|| format!("read {}", json_path.display()))?;
    let value = serde_json::from_str::<Value>(&json_body)
        .with_context(|| format!("parse {}", json_path.display()))?;

    let comments = value
        .get("comments")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let summary_only = value
        .get("summary_only")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let suppressed = value
        .get("suppressed")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let warnings = value
        .get("warnings")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);

    let fallback = format!(
        "## RIPR Review Guidance\n\n\
         Generated from `{}` because the sibling Markdown report was not present.\n\n\
         - Inline comments: {comments}\n\
         - Summary-only items: {summary_only}\n\
         - Suppressed items: {suppressed}\n\
         - Warnings: {warnings}\n",
        json_path.display()
    );
    fs::write(md_path, fallback).with_context(|| format!("write {}", md_path.display()))?;
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
    use super::*;

    #[test]
    fn annotations_escape_github_command_fields() {
        let value = serde_json::json!({
            "comments": [
                {
                    "path": "src/a,b=1.rs",
                    "line": 12,
                    "title": "RIPR: needs=test",
                    "body": "first%line\nsecond:line"
                }
            ]
        });
        let mut output = Vec::new();

        emit_annotations(&value, &mut output).unwrap();

        let body = String::from_utf8(output).unwrap();
        assert_eq!(
            body,
            "::warning file=src/a%2Cb%3D1.rs,line=12,title=RIPR%3A needs%3Dtest::first%25line%0Asecond%3Aline\n"
        );
    }

    #[test]
    fn annotations_skip_comments_without_line_place() {
        let value = serde_json::json!({
            "comments": [
                {"path": "src/lib.rs", "body": "missing line"},
                {"line": 7, "body": "missing file"},
                {"file": "src/main.rs", "line": 3, "message": "fallback fields"}
            ]
        });
        let mut output = Vec::new();

        emit_annotations(&value, &mut output).unwrap();

        let body = String::from_utf8(output).unwrap();
        assert_eq!(
            body,
            "::warning file=src/main.rs,line=3,title=RIPR::fallback fields\n"
        );
    }

    #[test]
    fn pr_contract_rejects_missing_output() {
        let temp = tempfile::tempdir().unwrap();
        assert!(check_pr_contract(temp.path()).is_err());
    }

    #[test]
    fn review_contract_accepts_empty_arrays() {
        let temp = tempfile::tempdir().unwrap();
        let json = temp.path().join("comments.json");
        let md = temp.path().join("comments.md");
        fs::write(
            &json,
            r#"{"comments":[],"summary_only":[],"suppressed":[],"warnings":[]}"#,
        )
        .unwrap();
        fs::write(&md, "## RIPR\n").unwrap();

        check_review_contract(&json, &md).unwrap();
    }

    #[test]
    fn review_markdown_fallback_is_written_when_missing() {
        let temp = tempfile::tempdir().unwrap();
        let json = temp.path().join("comments.json");
        let md = temp.path().join("comments.md");
        fs::write(
            &json,
            r#"{"comments":[],"summary_only":[{}],"suppressed":[],"warnings":[{}]}"#,
        )
        .unwrap();

        ensure_review_markdown(&json, &md).unwrap();

        let body = fs::read_to_string(&md).unwrap();
        assert!(body.contains("RIPR Review Guidance"));
        assert!(body.contains("Summary-only items: 1"));
        check_review_contract(&json, &md).unwrap();
    }
}
