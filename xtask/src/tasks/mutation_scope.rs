use crate::cli::MutationScopeArgs;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MUTATION_SCOPE_SCHEMA: &str = "tokmd.mutation_scope.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct MutationScopeReport {
    schema: &'static str,
    ok: bool,
    base_ref: String,
    base: String,
    head: String,
    max_files: usize,
    total_count: usize,
    count: usize,
    scope_exceeded: bool,
    all_changed_files: Vec<String>,
    changed_files: Vec<String>,
}

pub fn run(args: MutationScopeArgs) -> Result<()> {
    let root = workspace_root()?;
    let all_changed = git_diff_rust_files(&root, &args.base, &args.head)?;
    let report = mutation_scope_report(
        args.base_ref,
        args.base,
        args.head,
        args.max_files,
        all_changed,
    );

    write_lines(
        &root.join(&args.all_changed_files),
        &report.all_changed_files,
    )?;
    write_lines(&root.join(&args.changed_files), &report.changed_files)?;

    if let Some(path) = &args.json_output {
        write_json(&root.join(path), &report)?;
    }

    if let Some(path) = &args.github_output {
        write_text(&root.join(path), &render_github_outputs(&report))?;
    }

    if report.scope_exceeded {
        println!(
            "::error::Too many changed files ({} > {}). Please split your PR into smaller chunks or run crate-scoped mutation testing manually.",
            report.total_count, report.max_files
        );
    }

    println!(
        "mutation-scope: {} candidate file(s), {} selected, scope_exceeded={}",
        report.total_count, report.count, report.scope_exceeded
    );

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .context("cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("parse cargo metadata")?;
    let root = value
        .get("workspace_root")
        .and_then(|v| v.as_str())
        .context("workspace_root missing from cargo metadata")?;
    Ok(PathBuf::from(root))
}

fn git_diff_rust_files(root: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{base}...{head}"))
        .arg("--")
        .arg("*.rs")
        .current_dir(root)
        .output()
        .with_context(|| format!("git diff --name-only {base}...{head} -- *.rs"))?;
    if !output.status.success() {
        bail!(
            "git diff exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8(output.stdout).context("git diff output was not UTF-8")?;
    Ok(select_mutation_candidates(stdout.lines()))
}

fn mutation_scope_report(
    base_ref: String,
    base: String,
    head: String,
    max_files: usize,
    all_changed_files: Vec<String>,
) -> MutationScopeReport {
    let total_count = all_changed_files.len();
    let scope_exceeded = total_count > max_files;
    let changed_files = if scope_exceeded {
        Vec::new()
    } else {
        all_changed_files.clone()
    };

    MutationScopeReport {
        schema: MUTATION_SCOPE_SCHEMA,
        ok: !scope_exceeded,
        base_ref,
        base,
        head,
        max_files,
        total_count,
        count: changed_files.len(),
        scope_exceeded,
        all_changed_files,
        changed_files,
    }
}

fn select_mutation_candidates<'a>(files: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut selected = files
        .into_iter()
        .map(normalize_path)
        .filter(|path| is_mutation_candidate(path))
        .collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    selected
}

fn is_mutation_candidate(path: &str) -> bool {
    path.ends_with(".rs")
        && !path.contains("/tests/")
        && !path.ends_with("_test.rs")
        && !path.starts_with("fuzz/")
        && !path.contains("/fuzz/")
}

fn normalize_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

fn write_lines(path: &Path, lines: &[String]) -> Result<()> {
    let body = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    write_text(path, &body)
}

fn write_json(path: &Path, report: &MutationScopeReport) -> Result<()> {
    write_text(path, &serde_json::to_string_pretty(report)?)
}

fn write_text(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, body).with_context(|| format!("write {}", path.display()))
}

fn render_github_outputs(report: &MutationScopeReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("base_ref={}\n", report.base_ref));
    out.push_str(&format!("total_count={}\n", report.total_count));
    out.push_str(&format!("scope_exceeded={}\n", report.scope_exceeded));
    out.push_str(&format!("count={}\n", report.count));
    out.push_str("files<<EOF\n");
    for file in &report.changed_files {
        out.push_str(file);
        out.push('\n');
    }
    out.push_str("EOF\n");
    out
}

#[cfg(test)]
mod tests {
    use super::{mutation_scope_report, render_github_outputs, select_mutation_candidates};

    #[test]
    fn mutation_scope_filters_to_production_rust_files() {
        let files = select_mutation_candidates([
            "crates/tokmd/src/main.rs",
            "crates/tokmd/tests/cockpit.rs",
            "crates/tokmd/src/foo_test.rs",
            "fuzz/fuzz_targets/fuzz_lang.rs",
            "crates/tokmd/src/fuzz/helper.rs",
            "README.md",
            "crates\\tokmd-model\\src\\lib.rs",
        ]);

        assert_eq!(
            files,
            vec!["crates/tokmd-model/src/lib.rs", "crates/tokmd/src/main.rs",]
        );
    }

    #[test]
    fn mutation_scope_selects_all_files_under_limit() {
        let report = mutation_scope_report(
            "main".to_string(),
            "origin/main".to_string(),
            "HEAD".to_string(),
            2,
            vec!["a.rs".to_string(), "b.rs".to_string()],
        );

        assert!(report.ok);
        assert!(!report.scope_exceeded);
        assert_eq!(report.total_count, 2);
        assert_eq!(report.count, 2);
        assert_eq!(report.changed_files, report.all_changed_files);
    }

    #[test]
    fn mutation_scope_clears_selected_files_when_scope_exceeds_limit() {
        let report = mutation_scope_report(
            "main".to_string(),
            "origin/main".to_string(),
            "HEAD".to_string(),
            1,
            vec!["a.rs".to_string(), "b.rs".to_string()],
        );

        assert!(!report.ok);
        assert!(report.scope_exceeded);
        assert_eq!(report.total_count, 2);
        assert_eq!(report.count, 0);
        assert_eq!(report.changed_files, Vec::<String>::new());
        assert_eq!(report.all_changed_files, vec!["a.rs", "b.rs"]);
    }

    #[test]
    fn github_outputs_match_mutants_workflow_contract() {
        let report = mutation_scope_report(
            "main".to_string(),
            "origin/main".to_string(),
            "HEAD".to_string(),
            20,
            vec!["crates/tokmd/src/main.rs".to_string()],
        );

        let outputs = render_github_outputs(&report);
        assert!(outputs.contains("base_ref=main\n"));
        assert!(outputs.contains("total_count=1\n"));
        assert!(outputs.contains("scope_exceeded=false\n"));
        assert!(outputs.contains("count=1\n"));
        assert!(outputs.contains("files<<EOF\ncrates/tokmd/src/main.rs\nEOF\n"));
    }
}
