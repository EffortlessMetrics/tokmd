use anyhow::{Context, Result, bail};
use tokmd_config as cli;
use tokmd_format::{compute_diff_rows, compute_diff_totals, create_diff_receipt, render_diff_md};
#[cfg(feature = "git")]
use tokmd_model as model;
#[cfg(feature = "git")]
use tokmd_scan as scan;
use tokmd_types::LangReport;

use std::path::{Path, PathBuf};
#[cfg(feature = "git")]
use std::process::Command;
#[cfg(feature = "git")]
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn handle(args: cli::DiffArgs, global: &cli::GlobalArgs) -> Result<()> {
    let (from, to) = resolve_targets(&args)?;
    let from_report = resolve_lang_report(&from, global)
        .with_context(|| format!("Failed to load diff source '{}'", from))?;
    let to_report = resolve_lang_report(&to, global)
        .with_context(|| format!("Failed to load diff source '{}'", to))?;

    let diff_rows = compute_diff_rows(&from_report, &to_report);
    let totals = compute_diff_totals(&diff_rows);

    match args.format {
        cli::DiffFormat::Md => {
            print!("{}", render_diff_md(&from, &to, &diff_rows, &totals));
        }
        cli::DiffFormat::Json => {
            let receipt = create_diff_receipt(&from, &to, diff_rows, totals);
            println!("{}", serde_json::to_string(&receipt)?);
        }
    }

    Ok(())
}

fn resolve_targets(args: &cli::DiffArgs) -> Result<(String, String)> {
    if !args.refs.is_empty() {
        if args.from.is_some() || args.to.is_some() {
            bail!("Use either two positional refs/paths or --from/--to, not both.");
        }
        if args.refs.len() != 2 {
            bail!("Diff expects exactly two refs/paths.");
        }
        return Ok((args.refs[0].clone(), args.refs[1].clone()));
    }

    match (&args.from, &args.to) {
        (Some(from), Some(to)) => Ok((from.clone(), to.clone())),
        _ => bail!("Provide either two positional refs/paths or both --from and --to."),
    }
}

fn resolve_lang_report(input: &str, global: &cli::GlobalArgs) -> Result<LangReport> {
    let path = PathBuf::from(input);
    if path.exists() {
        return load_lang_report_from_path(&path);
    }

    lang_report_from_git_ref(input, global)
}

fn load_lang_report_from_path(path: &Path) -> Result<LangReport> {
    let lang_path = if path.is_dir() {
        path.join("lang.json")
    } else if path
        .file_name()
        .map(|name| name == "receipt.json")
        .unwrap_or(false)
    {
        path.parent().unwrap_or(path).join("lang.json")
    } else {
        path.to_path_buf()
    };

    let content = std::fs::read_to_string(&lang_path)
        .with_context(|| format!("Failed to read {}", lang_path.display()))?;
    let receipt: tokmd_types::LangReceipt =
        serde_json::from_str(&content).context("Failed to parse lang receipt")?;
    Ok(receipt.report)
}

#[cfg(not(feature = "git"))]
fn lang_report_from_git_ref(_revision: &str, _global: &cli::GlobalArgs) -> Result<LangReport> {
    bail!("Git support is disabled in this build. Cannot diff git refs.");
}

#[cfg(feature = "git")]
fn lang_report_from_git_ref(revision: &str, global: &cli::GlobalArgs) -> Result<LangReport> {
    if !tokmd_git::git_available() {
        bail!("git is not available on PATH");
    }
    let cwd = std::env::current_dir().context("Failed to resolve current directory")?;
    let repo_root =
        tokmd_git::repo_root(&cwd).ok_or_else(|| anyhow::anyhow!("not inside a git repository"))?;

    let worktree = GitWorktree::new(&repo_root, revision)
        .with_context(|| format!("Failed to create worktree for '{}'", revision))?;
    let _cwd = ScopedCwd::new(&worktree.path)
        .with_context(|| format!("Failed to enter worktree for '{}'", revision))?;

    let scan_opts = tokmd_settings::ScanOptions::from(global);
    let languages = scan::scan(std::slice::from_ref(&worktree.path), &scan_opts)?;
    Ok(model::create_lang_report(
        &languages,
        0,
        false,
        cli::ChildrenMode::Collapse,
    ))
}

#[cfg(feature = "git")]
struct ScopedCwd {
    previous: PathBuf,
}

#[cfg(feature = "git")]
impl ScopedCwd {
    fn new(path: &Path) -> Result<Self> {
        let previous = std::env::current_dir().context("Failed to capture current directory")?;
        std::env::set_current_dir(path)
            .with_context(|| format!("Failed to set current directory to {}", path.display()))?;
        Ok(Self { previous })
    }
}

#[cfg(feature = "git")]
impl Drop for ScopedCwd {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

#[cfg(feature = "git")]
struct GitWorktree {
    repo_root: PathBuf,
    path: PathBuf,
}

#[cfg(feature = "git")]
impl GitWorktree {
    fn new(repo_root: &Path, revision: &str) -> Result<Self> {
        let path = make_temp_dir("diff-worktree")?;

        let status = Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .arg("worktree")
            .arg("add")
            .arg("--detach")
            .arg(&path)
            .arg(revision)
            .status()
            .with_context(|| format!("Failed to spawn git worktree for {}", revision))?;

        if !status.success() {
            let _ = std::fs::remove_dir_all(&path);
            bail!("git worktree add failed for '{}'", revision);
        }

        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            path,
        })
    }
}

#[cfg(feature = "git")]
impl Drop for GitWorktree {
    fn drop(&mut self) {
        let _ = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(&self.path)
            .status();
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(feature = "git")]
fn make_temp_dir(prefix: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let pid = std::process::id();

    for attempt in 0..1000 {
        let path = base.join(format!("tokmd-{}-{}-{}-{}", prefix, now, pid, attempt));
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create temp dir {}", path.display()))?;
            return Ok(path);
        }
    }

    bail!("Failed to create a unique temp directory for diff")
}
