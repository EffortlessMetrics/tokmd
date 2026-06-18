//! Jules provenance index generator.
//!
//! This replaces the previous Python helper with a Rust-native xtask so the
//! Rust-first repo policy can keep non-Rust files only where platform contracts
//! require them.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cli::JulesIndexArgs;

const GENERATED_BY: &str = "`cargo xtask jules-index`";
const GENERATED_DIR: &str = ".jules/index/generated";
const RUNS_ROLLUP: &str = ".jules/index/generated/RUNS_ROLLUP.md";
const FRICTION_ROLLUP: &str = ".jules/index/generated/FRICTION_ROLLUP.md";

#[derive(Debug, Clone, Eq, PartialEq)]
struct RunRow {
    id: String,
    persona: String,
    style: String,
    shard: String,
    status: String,
    gates_run: usize,
    source: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct FrictionRow {
    id: String,
    persona: String,
    style: String,
    shard: String,
    status: String,
    summary: String,
}

pub fn run(args: JulesIndexArgs) -> Result<()> {
    let root = workspace_root()?;
    write_or_check_runs_rollup(&root, args.check)?;
    write_or_check_friction_rollup(&root, args.check)?;

    if args.check {
        println!("Jules indexes are up to date.");
    } else {
        println!(
            "Jules indexes written under {}",
            root.join(GENERATED_DIR).display()
        );
    }
    Ok(())
}

fn write_or_check_runs_rollup(root: &Path, check: bool) -> Result<()> {
    let output_file = root.join(RUNS_ROLLUP);
    let runs = collect_runs(root)?;
    write_or_check_file(&output_file, &render_runs_rollup(&runs), check)
}

fn write_or_check_friction_rollup(root: &Path, check: bool) -> Result<()> {
    let output_file = root.join(FRICTION_ROLLUP);
    let open_dir = root.join(".jules/friction/open");
    let done_dir = root.join(".jules/friction/done");

    let mut items = collect_friction(&open_dir)?;
    items.extend(collect_friction(&done_dir)?);
    items.sort_by(|a, b| a.id.cmp(&b.id));

    write_or_check_file(&output_file, &render_friction_rollup(&items), check)
}

fn write_or_check_file(path: &Path, expected: &str, check: bool) -> Result<()> {
    if check {
        let actual =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        if actual != expected {
            bail!(
                "Jules index drift detected in {}. Run `cargo xtask jules-index` to update.",
                path.display()
            );
        }
        return Ok(());
    }

    ensure_parent(path)?;
    fs::write(path, expected).with_context(|| format!("write {}", path.display()))
}

fn collect_runs(root: &Path) -> Result<Vec<RunRow>> {
    let mut runs = Vec::new();
    collect_live_runs(root, &mut runs)?;
    collect_docs_ledger(root, &mut runs)?;
    collect_quality_ledger(root, &mut runs)?;
    Ok(runs)
}

fn collect_live_runs(root: &Path, runs: &mut Vec<RunRow>) -> Result<()> {
    let runs_dir = root.join(".jules/runs");
    if !runs_dir.exists() {
        return Ok(());
    }

    for run_path in sorted_dir_entries(&runs_dir)? {
        if !run_path.is_dir() {
            continue;
        }
        let envelope = read_json_object(&run_path.join("envelope.json"))?;
        let result = read_json_object(&run_path.join("result.json"))?;
        let id = file_name_string(&run_path).unwrap_or_else(|| "Unknown".to_string());
        runs.push(RunRow {
            id,
            persona: json_string(&envelope, "persona", "Unknown"),
            style: json_string(&envelope, "style", "Unknown"),
            shard: json_string(&envelope, "primary_shard", "Unknown"),
            status: json_string(&result, "status", "in-progress"),
            gates_run: array_len(&result, "gates_run"),
            source: "live".to_string(),
        });
    }
    Ok(())
}

fn collect_docs_ledger(root: &Path, runs: &mut Vec<RunRow>) -> Result<()> {
    let ledger = read_json_value(&root.join(".jules/docs/ledger.json"))?;
    let Some(entries) = ledger.as_array() else {
        return Ok(());
    };

    for entry in entries.iter().filter_map(Value::as_object) {
        let run_id = json_string_from_map(entry, "run_id", "Unknown");
        let env_path = root
            .join(".jules/docs/envelopes")
            .join(format!("{run_id}.json"));
        let env_data = read_json_object(&env_path)?;

        runs.push(RunRow {
            id: run_id,
            persona: first_non_empty([
                map_string(&env_data, "persona"),
                map_string(entry, "persona"),
                map_string(entry, "lane"),
            ])
            .unwrap_or_else(|| "Unknown".to_string()),
            style: first_non_empty([map_string(&env_data, "style"), map_string(entry, "style")])
                .unwrap_or_else(|| "Unknown".to_string()),
            shard: first_non_empty([
                map_string(&env_data, "primary_shard"),
                map_string(entry, "target"),
            ])
            .unwrap_or_else(|| "Unknown".to_string()),
            status: "historical".to_string(),
            gates_run: env_data
                .get("receipts")
                .or_else(|| entry.get("receipts"))
                .and_then(Value::as_array)
                .map_or(0, Vec::len),
            source: "docs".to_string(),
        });
    }
    Ok(())
}

fn collect_quality_ledger(root: &Path, runs: &mut Vec<RunRow>) -> Result<()> {
    let ledger = read_json_value(&root.join(".jules/quality/ledger.json"))?;
    let Some(entries) = ledger.get("runs").and_then(Value::as_array) else {
        return Ok(());
    };

    for entry in entries.iter().filter_map(Value::as_object) {
        let run_id = json_string_from_map(entry, "run_id", "Unknown");
        let env_path = root
            .join(".jules/quality/envelopes")
            .join(format!("{run_id}.json"));
        let env_data = read_json_object(&env_path)?;
        let description = map_string(entry, "description").unwrap_or_default();
        let persona = map_string(&env_data, "persona")
            .or_else(|| {
                description
                    .contains("Gatekeeper")
                    .then(|| "Gatekeeper".to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        runs.push(RunRow {
            id: run_id,
            persona,
            style: json_string(&env_data, "style", "Unknown"),
            shard: first_non_empty([
                map_string(&env_data, "primary_shard"),
                map_string(entry, "target"),
            ])
            .unwrap_or_else(|| "Unknown".to_string()),
            status: "historical".to_string(),
            gates_run: env_data
                .get("receipts")
                .or_else(|| entry.get("receipts"))
                .and_then(Value::as_array)
                .map_or(0, Vec::len),
            source: "quality".to_string(),
        });
    }
    Ok(())
}

fn collect_friction(friction_dir: &Path) -> Result<Vec<FrictionRow>> {
    if !friction_dir.exists() {
        return Ok(Vec::new());
    }

    sorted_dir_entries(friction_dir)?
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .map(|path| friction_metadata(&path))
        .collect()
}

fn friction_metadata(path: &Path) -> Result<FrictionRow> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let lines = content.lines().collect::<Vec<_>>();

    let mut row = FrictionRow {
        id: path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Unknown")
            .to_string(),
        persona: "Unknown".to_string(),
        style: "Unknown".to_string(),
        shard: "Unknown".to_string(),
        status: "open".to_string(),
        summary: String::new(),
    };

    for line in lines.iter().take(24) {
        let stripped = line.trim().trim_matches('*');
        let Some((key, value)) = stripped.split_once(':') else {
            continue;
        };
        let key = key.trim().to_lowercase().replace(' ', "_");
        let value = value.trim().trim_matches('*').trim();
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            "id" => row.id = value.to_string(),
            "persona" => row.persona = value.to_string(),
            "style" => row.style = value.to_string(),
            "shard" | "surface" | "component" => row.shard = value.to_string(),
            "status" => row.status = value.to_string(),
            "summary" => row.summary = value.to_string(),
            _ => {}
        }
    }

    if row.summary.is_empty() {
        row.summary = heading_summary(&content).unwrap_or_else(|| fallback_summary(&lines));
    }

    Ok(row)
}

fn heading_summary(content: &str) -> Option<String> {
    ["## Problem", "**Problem:**", "**Description:**"]
        .into_iter()
        .find_map(|heading| {
            let (_, after) = content.split_once(heading)?;
            after.lines().find_map(|line| {
                let summary = line.trim().trim_start_matches('-').trim();
                (!summary.is_empty()).then(|| summary.to_string())
            })
        })
}

fn fallback_summary(lines: &[&str]) -> String {
    lines
        .iter()
        .find_map(|line| {
            let stripped = line.trim();
            (!stripped.is_empty() && !stripped.starts_with('#') && !stripped.contains(':'))
                .then(|| stripped.to_string())
        })
        .unwrap_or_default()
}

fn render_runs_rollup(runs: &[RunRow]) -> String {
    let mut out = String::new();
    out.push_str("# Generated Run Index\n\n");
    out.push_str(&format!(
        "This file is automatically generated by {GENERATED_BY}.\n"
    ));
    out.push_str(
        "It rolls up metadata from all run packets in `.jules/runs/` and historical ledgers.\n\n",
    );
    out.push_str("| Run ID | Persona | Style | Shard | Status | Gates Run | Source |\n");
    out.push_str("|---|---|---|---|---|---|---|\n");
    for run in runs {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |\n",
            run.id,
            markdown_cell(&run.persona),
            markdown_cell(&run.style),
            markdown_cell(&run.shard),
            markdown_cell(&run.status),
            run.gates_run,
            markdown_cell(&run.source)
        ));
    }
    out
}

fn render_friction_rollup(items: &[FrictionRow]) -> String {
    let mut out = String::new();
    out.push_str("# Generated Friction Index\n\n");
    out.push_str(&format!(
        "This file is automatically generated by {GENERATED_BY}.\n"
    ));
    out.push_str("It rolls up active and closed friction metadata from `.jules/friction/open/` and `.jules/friction/done/`.\n\n");
    out.push_str("| ID | Persona | Style | Shard | Status | Summary |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    for item in items {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            markdown_cell(&item.id),
            markdown_cell(&item.persona),
            markdown_cell(&item.style),
            markdown_cell(&item.shard),
            markdown_cell(&item.status),
            markdown_cell(&item.summary)
        ));
    }
    out
}

fn markdown_cell(value: &str) -> String {
    let text = if value.is_empty() { "Unknown" } else { value };
    text.replace('\n', " ").trim().replace('|', "\\|")
}

fn sorted_dir_entries(path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(path)
        .with_context(|| format!("read dir {}", path.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("read dir entry {}", path.display()))?;
    paths.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    Ok(paths)
}

fn read_json_value(path: &Path) -> Result<Value> {
    match fs::read_to_string(path) {
        Ok(body) => serde_json::from_str(&body).or_else(|_| Ok(Value::Null)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Value::Null),
        Err(err) => Err(err).with_context(|| format!("read {}", path.display())),
    }
}

fn read_json_object(path: &Path) -> Result<BTreeMap<String, Value>> {
    Ok(read_json_value(path)?
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect())
}

fn json_string(map: &BTreeMap<String, Value>, key: &str, default: &str) -> String {
    map_string(map, key).unwrap_or_else(|| default.to_string())
}

fn json_string_from_map(map: &serde_json::Map<String, Value>, key: &str, default: &str) -> String {
    map_string(map, key).unwrap_or_else(|| default.to_string())
}

fn map_string(map: &impl JsonLookup, key: &str) -> Option<String> {
    map.get_value(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

trait JsonLookup {
    fn get_value(&self, key: &str) -> Option<&Value>;
}

impl JsonLookup for BTreeMap<String, Value> {
    fn get_value(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }
}

impl JsonLookup for serde_json::Map<String, Value> {
    fn get_value(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }
}

fn first_non_empty(values: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    values.into_iter().flatten().find(|value| !value.is_empty())
}

fn array_len(map: &BTreeMap<String, Value>, key: &str) -> usize {
    map.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    Ok(())
}

fn file_name_string(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(ToString::to_string)
}

fn workspace_root() -> Result<PathBuf> {
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
    fn markdown_cell_escapes_pipes_and_newlines() {
        assert_eq!(markdown_cell("a|b\nc"), "a\\|b c");
    }

    #[test]
    fn friction_metadata_reads_problem_heading() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("item.md");
        fs::write(
            &path,
            "**Persona:** Builder\n**Component:** core\n\n## Problem\n- Something broke\n",
        )
        .expect("write fixture");

        let row = friction_metadata(&path).expect("metadata");
        assert_eq!(row.persona, "Builder");
        assert_eq!(row.shard, "core");
        assert_eq!(row.summary, "Something broke");
    }

    #[test]
    fn check_mode_reports_drift_without_writing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("index.md");
        fs::write(&path, "old").expect("write fixture");

        let err = write_or_check_file(&path, "new", true).expect_err("drift should fail");
        assert!(
            err.to_string().contains("Jules index drift detected"),
            "{err:?}"
        );
        assert_eq!(fs::read_to_string(&path).expect("read fixture"), "old");
    }

    #[test]
    fn write_mode_creates_parent_and_writes_expected_content() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("index.md");

        write_or_check_file(&path, "generated\n", false).expect("write index");
        assert_eq!(
            fs::read_to_string(&path).expect("read fixture"),
            "generated\n"
        );
    }
}
