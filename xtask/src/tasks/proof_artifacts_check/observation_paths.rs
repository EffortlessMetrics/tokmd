use crate::cli::{ProofExecutionObservationsSummaryArgs, ProofRunObservationsSummaryArgs};
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) fn collect_observation_paths(
    args: &ProofExecutionObservationsSummaryArgs,
) -> Result<Vec<PathBuf>> {
    let mut paths = BTreeSet::new();

    if args.observations.is_empty() && args.observation_dirs.is_empty() {
        paths.insert(PathBuf::from(
            "target/proof/proof-executor-observation.json",
        ));
    }

    paths.extend(args.observations.iter().cloned());
    for dir in &args.observation_dirs {
        collect_observation_paths_from_dir(dir, &mut paths)?;
    }

    if paths.is_empty() {
        bail!("no proof executor observation artifacts found");
    }

    Ok(paths.into_iter().collect())
}

pub(super) fn collect_observation_paths_from_dir(
    dir: &Path,
    paths: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if !dir.is_dir() {
        bail!(
            "observation directory `{}` is not a directory",
            dir.display()
        );
    }

    for entry in WalkDir::new(dir) {
        let entry = entry
            .with_context(|| format!("failed to scan observation directory `{}`", dir.display()))?;
        if entry.file_type().is_file()
            && entry.file_name().to_string_lossy() == "proof-executor-observation.json"
        {
            paths.insert(entry.path().to_path_buf());
        }
    }

    Ok(())
}

pub(super) fn collect_proof_run_observation_paths(
    args: &ProofRunObservationsSummaryArgs,
) -> Result<Vec<PathBuf>> {
    let mut paths = BTreeSet::new();

    if args.observations.is_empty() && args.observation_dirs.is_empty() {
        paths.insert(PathBuf::from("target/proof-run/proof-run-observation.json"));
    }

    paths.extend(args.observations.iter().cloned());
    for dir in &args.observation_dirs {
        collect_proof_run_observation_paths_from_dir(dir, &mut paths)?;
    }

    if paths.is_empty() {
        bail!("no proof run observation artifacts found");
    }

    Ok(paths.into_iter().collect())
}

pub(super) fn collect_proof_run_observation_paths_from_dir(
    dir: &Path,
    paths: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if !dir.is_dir() {
        bail!(
            "observation directory `{}` is not a directory",
            dir.display()
        );
    }

    for entry in WalkDir::new(dir) {
        let entry = entry
            .with_context(|| format!("failed to scan observation directory `{}`", dir.display()))?;
        if entry.file_type().is_file()
            && entry.file_name().to_string_lossy() == "proof-run-observation.json"
        {
            paths.insert(entry.path().to_path_buf());
        }
    }

    Ok(())
}
