use crate::cli::MutationSummaryArgs;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SUMMARY_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct MutationSummary {
    schema_version: u8,
    commit: String,
    base_ref: String,
    status: String,
    scope: Vec<String>,
    survivors: Vec<Survivor>,
    killed: usize,
    timeout: usize,
    unviable: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Survivor {
    file: String,
    line: u64,
    mutation: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MutationCounts {
    killed: usize,
    timeout: usize,
    unviable: usize,
    survivors: Vec<Survivor>,
}

pub fn run(args: MutationSummaryArgs) -> Result<()> {
    let root = workspace_root()?;
    let all_changed_files = read_lines(&root.join(&args.all_changed_files))?;
    let changed_files = read_lines(&root.join(&args.changed_files))?;
    let summary = mutation_summary(
        args.commit,
        args.base_ref,
        args.scope_exceeded,
        args.mutants_ran,
        all_changed_files,
        changed_files,
        &root.join(&args.mutants_dir),
    )?;

    write_json(&root.join(&args.json_output), &summary)?;

    if let Some(path) = &args.github_output {
        write_text(&root.join(path), &render_github_outputs(&summary))?;
    }

    println!(
        "mutation-summary: status={}, scope={}, survivors={}, killed={}, timeout={}, unviable={}",
        summary.status,
        summary.scope.len(),
        summary.survivors.len(),
        summary.killed,
        summary.timeout,
        summary.unviable
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
    let value: Value = serde_json::from_slice(&output.stdout).context("parse cargo metadata")?;
    let root = value
        .get("workspace_root")
        .and_then(Value::as_str)
        .context("workspace_root missing from cargo metadata")?;
    Ok(PathBuf::from(root))
}

fn mutation_summary(
    commit: String,
    base_ref: String,
    scope_exceeded: bool,
    mutants_ran: bool,
    all_changed_files: Vec<String>,
    changed_files: Vec<String>,
    mutants_dir: &Path,
) -> Result<MutationSummary> {
    if scope_exceeded {
        return Ok(MutationSummary {
            schema_version: SUMMARY_SCHEMA_VERSION,
            commit,
            base_ref,
            status: "fail".to_string(),
            scope: all_changed_files,
            survivors: Vec::new(),
            killed: 0,
            timeout: 0,
            unviable: 0,
        });
    }

    if all_changed_files.is_empty() && changed_files.is_empty() {
        return Ok(MutationSummary {
            schema_version: SUMMARY_SCHEMA_VERSION,
            commit,
            base_ref,
            status: "skipped".to_string(),
            scope: Vec::new(),
            survivors: Vec::new(),
            killed: 0,
            timeout: 0,
            unviable: 0,
        });
    }

    if !mutants_ran {
        return Ok(MutationSummary {
            schema_version: SUMMARY_SCHEMA_VERSION,
            commit,
            base_ref,
            status: "skipped".to_string(),
            scope: Vec::new(),
            survivors: Vec::new(),
            killed: 0,
            timeout: 0,
            unviable: 0,
        });
    }

    let counts = collect_mutation_counts(mutants_dir)?;
    let status = if counts.survivors.is_empty() {
        "pass"
    } else {
        "fail"
    };

    Ok(MutationSummary {
        schema_version: SUMMARY_SCHEMA_VERSION,
        commit,
        base_ref,
        status: status.to_string(),
        scope: changed_files,
        survivors: counts.survivors,
        killed: counts.killed,
        timeout: counts.timeout,
        unviable: counts.unviable,
    })
}

fn collect_mutation_counts(mutants_dir: &Path) -> Result<MutationCounts> {
    let mut counts = MutationCounts::default();

    for dir in mutants_output_dirs(mutants_dir)? {
        let mut parsed_outcomes = false;
        let outcomes_path = dir.join("outcomes.json");
        if outcomes_path.is_file()
            && let Some(outcomes) = parse_outcomes_json(&outcomes_path)?
        {
            counts.killed += outcomes.killed;
            counts.timeout += outcomes.timeout;
            counts.unviable += outcomes.unviable;
            counts.survivors.extend(outcomes.survivors);
            parsed_outcomes = true;
        }

        counts.killed += count_lines_if_exists(&dir.join("caught.txt"))?;
        counts.timeout += count_lines_if_exists(&dir.join("timeout.txt"))?;
        counts.unviable += count_lines_if_exists(&dir.join("unviable.txt"))?;

        if !parsed_outcomes && counts.survivors.is_empty() {
            counts
                .survivors
                .extend(parse_missed_txt(&dir.join("missed.txt"))?);
        }
    }

    Ok(counts)
}

fn mutants_output_dirs(mutants_dir: &Path) -> Result<Vec<PathBuf>> {
    if !mutants_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut dirs = Vec::new();
    for entry in
        fs::read_dir(mutants_dir).with_context(|| format!("read {}", mutants_dir.display()))?
    {
        let entry = entry.with_context(|| format!("read entry in {}", mutants_dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort_by_key(|path| path.to_string_lossy().to_string());
    Ok(dirs)
}

fn parse_outcomes_json(path: &Path) -> Result<Option<MutationCounts>> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let Ok(value) = serde_json::from_str::<Value>(&body) else {
        return Ok(None);
    };
    let Some(outcomes) = value.get("outcomes").and_then(Value::as_array) else {
        return Ok(None);
    };

    let mut counts = MutationCounts::default();
    for outcome in outcomes {
        match outcome.get("outcome").and_then(Value::as_str) {
            Some("Killed") => counts.killed += 1,
            Some("Timeout") => counts.timeout += 1,
            Some("Unviable") => counts.unviable += 1,
            Some("Missed") => {
                if let Some(survivor) = survivor_from_outcome(outcome) {
                    counts.survivors.push(survivor);
                }
            }
            _ => {}
        }
    }

    Ok(Some(counts))
}

fn survivor_from_outcome(outcome: &Value) -> Option<Survivor> {
    let scenario = outcome.get("scenario")?;
    Some(Survivor {
        file: scenario.get("source_file")?.as_str()?.replace('\\', "/"),
        line: scenario.get("line")?.as_u64()?,
        mutation: scenario.get("genre")?.as_str()?.to_string(),
    })
}

fn parse_missed_txt(path: &Path) -> Result<Vec<Survivor>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }

    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut survivors = Vec::new();
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        survivors.push(parse_missed_line(line)?);
    }
    Ok(survivors)
}

fn parse_missed_line(line: &str) -> Result<Survivor> {
    let mut parts = line.splitn(3, ':');
    let file = parts
        .next()
        .filter(|part| !part.is_empty())
        .with_context(|| format!("missed mutation line is missing file: {line}"))?;
    let line_number = parts
        .next()
        .with_context(|| format!("missed mutation line is missing line number: {line}"))?
        .parse::<u64>()
        .with_context(|| format!("missed mutation line has invalid line number: {line}"))?;
    let mutation = parts
        .next()
        .with_context(|| format!("missed mutation line is missing mutation: {line}"))?;
    let mutation = mutation.strip_prefix(' ').unwrap_or(mutation);

    Ok(Survivor {
        file: file.replace('\\', "/"),
        line: line_number,
        mutation: mutation.to_string(),
    })
}

fn read_lines(path: &Path) -> Result<Vec<String>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }

    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = body
        .lines()
        .map(normalize_line)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    lines.sort();
    lines.dedup();
    Ok(lines)
}

fn count_lines_if_exists(path: &Path) -> Result<usize> {
    if !path.is_file() {
        return Ok(0);
    }
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(body.lines().filter(|line| !line.trim().is_empty()).count())
}

fn normalize_line(line: &str) -> String {
    line.trim().replace('\\', "/")
}

fn write_json(path: &Path, summary: &MutationSummary) -> Result<()> {
    let body = format!("{}\n", serde_json::to_string_pretty(summary)?);
    write_text(path, &body)
}

fn write_text(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, body).with_context(|| format!("write {}", path.display()))
}

fn render_github_outputs(summary: &MutationSummary) -> String {
    format!(
        "status={}\nsurvivor_count={}\n",
        summary.status,
        summary.survivors.len()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        collect_mutation_counts, mutation_summary, parse_missed_line, render_github_outputs,
    };
    use anyhow::{Result, bail};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn mutation_summary_reports_scope_exceeded_as_failure() -> Result<()> {
        let dir = tempdir()?;
        let summary = mutation_summary(
            "abc123".to_string(),
            "main".to_string(),
            true,
            false,
            vec!["crates/tokmd/src/main.rs".to_string()],
            Vec::new(),
            dir.path(),
        )?;

        assert_eq!(summary.schema_version, 1);
        assert_eq!(summary.status, "fail");
        assert_eq!(summary.scope, vec!["crates/tokmd/src/main.rs"]);
        assert_eq!(summary.survivors, Vec::new());
        Ok(())
    }

    #[test]
    fn mutation_summary_reports_no_candidate_files_as_skipped() -> Result<()> {
        let dir = tempdir()?;
        let summary = mutation_summary(
            "abc123".to_string(),
            "main".to_string(),
            false,
            false,
            Vec::new(),
            Vec::new(),
            dir.path(),
        )?;

        assert_eq!(summary.status, "skipped");
        assert_eq!(summary.scope, Vec::<String>::new());
        Ok(())
    }

    #[test]
    fn mutation_summary_parses_outcomes_json() -> Result<()> {
        let dir = tempdir()?;
        let mutants_all = dir.path().join("mutants-all");
        let out_dir = mutants_all.join("mutants-0");
        fs::create_dir_all(&out_dir)?;
        fs::write(
            out_dir.join("outcomes.json"),
            r#"{
  "outcomes": [
    { "outcome": "Killed", "scenario": { "source_file": "src/lib.rs", "line": 1, "genre": "replace +" } },
    { "outcome": "Timeout", "scenario": { "source_file": "src/lib.rs", "line": 2, "genre": "loop" } },
    { "outcome": "Unviable", "scenario": { "source_file": "src/lib.rs", "line": 3, "genre": "delete" } },
    { "outcome": "Missed", "scenario": { "source_file": "src\\lib.rs", "line": 4, "genre": "replace ==" } }
  ]
}"#,
        )?;

        let summary = mutation_summary(
            "abc123".to_string(),
            "main".to_string(),
            false,
            true,
            vec!["src/lib.rs".to_string()],
            vec!["src/lib.rs".to_string()],
            &mutants_all,
        )?;

        assert_eq!(summary.status, "fail");
        assert_eq!(summary.killed, 1);
        assert_eq!(summary.timeout, 1);
        assert_eq!(summary.unviable, 1);
        let survivor = summary
            .survivors
            .first()
            .ok_or_else(|| anyhow::anyhow!("expected survivor"))?;
        assert_eq!(survivor.file, "src/lib.rs");
        assert_eq!(survivor.line, 4);
        assert_eq!(survivor.mutation, "replace ==");
        Ok(())
    }

    #[test]
    fn mutation_summary_uses_text_fallbacks_without_outcomes_json() -> Result<()> {
        let dir = tempdir()?;
        let mutants_all = dir.path().join("mutants-all");
        let out_dir = mutants_all.join("mutants-0");
        fs::create_dir_all(&out_dir)?;
        fs::write(out_dir.join("caught.txt"), "a\nb\n")?;
        fs::write(out_dir.join("timeout.txt"), "a\n")?;
        fs::write(out_dir.join("unviable.txt"), "a\n")?;
        fs::write(
            out_dir.join("missed.txt"),
            "src/lib.rs:12: replaced == with !=\n",
        )?;

        let counts = collect_mutation_counts(&mutants_all)?;
        assert_eq!(counts.killed, 2);
        assert_eq!(counts.timeout, 1);
        assert_eq!(counts.unviable, 1);
        let survivor = counts
            .survivors
            .first()
            .ok_or_else(|| anyhow::anyhow!("expected fallback survivor"))?;
        assert_eq!(survivor.file, "src/lib.rs");
        assert_eq!(survivor.line, 12);
        assert_eq!(survivor.mutation, "replaced == with !=");
        Ok(())
    }

    #[test]
    fn mutation_summary_renders_workflow_outputs() -> Result<()> {
        let dir = tempdir()?;
        let summary = mutation_summary(
            "abc123".to_string(),
            "main".to_string(),
            false,
            true,
            vec!["src/lib.rs".to_string()],
            vec!["src/lib.rs".to_string()],
            dir.path(),
        )?;

        assert_eq!(
            render_github_outputs(&summary),
            "status=pass\nsurvivor_count=0\n"
        );
        Ok(())
    }

    #[test]
    fn missed_line_requires_three_fields() {
        let error = parse_missed_line("src/lib.rs:not-a-number: mutant")
            .err()
            .map(|error| error.to_string());
        assert!(
            error
                .as_deref()
                .is_some_and(|message| message.contains("invalid line number"))
        );
    }

    #[test]
    fn mutation_summary_reports_unrun_mutants_as_skipped() -> Result<()> {
        let dir = tempdir()?;
        let summary = mutation_summary(
            "abc123".to_string(),
            "main".to_string(),
            false,
            false,
            vec!["src/lib.rs".to_string()],
            vec!["src/lib.rs".to_string()],
            dir.path(),
        )?;

        if summary.status != "skipped" {
            bail!("expected skipped, got {}", summary.status);
        }
        assert_eq!(summary.scope, Vec::<String>::new());
        Ok(())
    }
}
