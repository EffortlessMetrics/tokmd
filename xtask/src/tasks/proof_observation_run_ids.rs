use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cli::ProofObservationRunIdsArgs;

pub fn run(args: ProofObservationRunIdsArgs) -> Result<()> {
    let run_ids = read_run_ids(&args.runs_json)?;
    write_run_ids(&args.output, &run_ids)?;

    println!(
        "proof observation run ids: wrote {} id(s) to {}",
        run_ids.len(),
        args.output.display()
    );
    Ok(())
}

fn read_run_ids(path: &Path) -> Result<Vec<String>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let runs: Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    let runs = runs
        .as_array()
        .with_context(|| format!("{} must contain a JSON array", path.display()))?;

    runs.iter()
        .enumerate()
        .map(|(index, run)| run_id_at(run, index))
        .collect()
}

fn run_id_at(run: &Value, index: usize) -> Result<String> {
    let id = run
        .get("databaseId")
        .with_context(|| format!("run at index {index} is missing databaseId"))?;

    if let Some(id) = id.as_u64() {
        return Ok(id.to_string());
    }

    if let Some(id) = id.as_str() {
        let trimmed = id.trim();
        if !trimmed.is_empty() && trimmed.chars().all(|ch| ch.is_ascii_digit()) {
            return Ok(trimmed.to_owned());
        }
    }

    bail!("run at index {index} has non-numeric databaseId")
}

fn write_run_ids(path: &Path, run_ids: &[String]) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }

    let mut body = String::new();
    for run_id in run_ids {
        body.push_str(run_id);
        body.push('\n');
    }
    fs::write(path, body).with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_database_ids_in_source_order() {
        let runs = json!([
            {"databaseId": 333, "headSha": "c"},
            {"databaseId": "222", "headSha": "b"},
            {"databaseId": 111, "headSha": "a"}
        ]);

        let ids: Vec<_> = runs
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(index, run)| run_id_at(run, index).unwrap())
            .collect();

        assert_eq!(ids, ["333", "222", "111"]);
    }

    #[test]
    fn rejects_missing_database_id() {
        let err = run_id_at(&json!({"headSha": "abc"}), 2)
            .unwrap_err()
            .to_string();

        assert!(
            err.contains("run at index 2 is missing databaseId"),
            "{err}"
        );
    }

    #[test]
    fn rejects_non_numeric_database_id() {
        let err = run_id_at(&json!({"databaseId": "abc"}), 0)
            .unwrap_err()
            .to_string();

        assert!(
            err.contains("run at index 0 has non-numeric databaseId"),
            "{err}"
        );
    }
}
