use anyhow::{bail, Context, Result};
use tokmd_config as cli;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_types::LangReport;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn handle(args: cli::DiffArgs, global: &cli::GlobalArgs) -> Result<()> {
    let (from, to) = resolve_targets(&args)?;
    let from_report = resolve_lang_report(&from, global)
        .with_context(|| format!("Failed to load diff source '{}'", from))?;
    let to_report =
        resolve_lang_report(&to, global).with_context(|| format!("Failed to load diff source '{}'", to))?;

    println!("Diffing Language Summaries: {} -> {}", from, to);
    println!(
        "{:<20} {:>10} {:>10} {:>10}",
        "Language", "Old LOC", "New LOC", "Delta"
    );
    println!("{:-<55}", "");

    // Simple map-based diff
    let mut all_langs: Vec<String> = from_report
        .rows
        .iter()
        .chain(to_report.rows.iter())
        .map(|r| r.lang.clone())
        .collect();
    all_langs.sort();
    all_langs.dedup();

    for lang_name in all_langs {
        let old_row = from_report.rows.iter().find(|r| r.lang == lang_name);
        let new_row = to_report.rows.iter().find(|r| r.lang == lang_name);

        let old_code = old_row.map(|r| r.code).unwrap_or(0);
        let new_code = new_row.map(|r| r.code).unwrap_or(0);

        if old_code == new_code {
            continue;
        }

        let delta = new_code as i64 - old_code as i64;
        let sign = if delta > 0 { "+" } else { "" };

        println!(
            "{:<20} {:>10} {:>10} {:>10}{}",
            lang_name, old_code, new_code, sign, delta
        );
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

    let languages = scan::scan(&[worktree.path.clone()], global)?;
    Ok(model::create_lang_report(
        &languages,
        0,
        false,
        cli::ChildrenMode::Collapse,
    ))
}

struct ScopedCwd {
    previous: PathBuf,
}

impl ScopedCwd {
    fn new(path: &Path) -> Result<Self> {
        let previous = std::env::current_dir().context("Failed to capture current directory")?;
        std::env::set_current_dir(path)
            .with_context(|| format!("Failed to set current directory to {}", path.display()))?;
        Ok(Self { previous })
    }
}

impl Drop for ScopedCwd {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

struct GitWorktree {
    repo_root: PathBuf,
    path: PathBuf,
}

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
