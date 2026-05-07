use crate::cli::CoverageReceiptArgs;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::path::Path;

const COVERAGE_RECEIPT_SCHEMA: &str = "tokmd.coverage_receipt.v1";

#[derive(Debug, Serialize)]
struct CoverageReceipt {
    schema: String,
    schema_version: u32,
    repo: String,
    lane: String,
    flag: String,
    workflow: String,
    sha: String,
    github: GithubContext,
    artifacts: Vec<CoverageArtifact>,
    status: CoverageStatus,
}

#[derive(Debug, Serialize)]
struct GithubContext {
    run_id: Option<String>,
    run_attempt: Option<String>,
    event_name: Option<String>,
    ref_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct CoverageArtifact {
    path: String,
    kind: String,
    bytes: u64,
    non_empty: bool,
}

#[derive(Debug, Serialize)]
struct CoverageStatus {
    ok: bool,
    missing: Vec<String>,
    empty: Vec<String>,
}

struct ArtifactInput<'a> {
    path: &'a Path,
    kind: &'static str,
}

pub fn run(args: CoverageReceiptArgs) -> Result<()> {
    let receipt = coverage_receipt(&args)?;
    if !receipt.status.ok {
        bail!(
            "coverage receipt refused missing/empty artifacts: missing={:?}, empty={:?}",
            receipt.status.missing,
            receipt.status.empty
        );
    }

    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&receipt).context("serialize coverage receipt")?;
    fs::write(&args.output, format!("{json}\n"))
        .with_context(|| format!("write {}", args.output.display()))?;
    println!(
        "coverage receipt written to {} ({} artifact(s))",
        args.output.display(),
        receipt.artifacts.len()
    );
    Ok(())
}

fn coverage_receipt(args: &CoverageReceiptArgs) -> Result<CoverageReceipt> {
    let inputs = [
        ArtifactInput {
            path: &args.coverage_json,
            kind: "json",
        },
        ArtifactInput {
            path: &args.coverage_text,
            kind: "text",
        },
        ArtifactInput {
            path: &args.lcov,
            kind: "lcov",
        },
    ];

    let mut artifacts = Vec::new();
    let mut missing = Vec::new();
    let mut empty = Vec::new();

    for input in inputs {
        match fs::metadata(input.path) {
            Ok(metadata) => {
                let bytes = metadata.len();
                let path = display_path(input.path);
                let non_empty = bytes > 0;
                if !non_empty {
                    empty.push(path.clone());
                }
                artifacts.push(CoverageArtifact {
                    path,
                    kind: input.kind.to_string(),
                    bytes,
                    non_empty,
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                missing.push(display_path(input.path));
            }
            Err(err) => {
                return Err(err).with_context(|| format!("inspect {}", input.path.display()));
            }
        }
    }

    Ok(CoverageReceipt {
        schema: COVERAGE_RECEIPT_SCHEMA.to_string(),
        schema_version: 1,
        repo: args.repo.clone(),
        lane: args.lane.clone(),
        flag: args.flag.clone(),
        workflow: args.workflow.clone(),
        sha: receipt_sha(args),
        github: GithubContext {
            run_id: env_non_empty("GITHUB_RUN_ID"),
            run_attempt: env_non_empty("GITHUB_RUN_ATTEMPT"),
            event_name: env_non_empty("GITHUB_EVENT_NAME"),
            ref_name: env_non_empty("GITHUB_REF_NAME").or_else(|| env_non_empty("GITHUB_REF")),
        },
        artifacts,
        status: CoverageStatus {
            ok: missing.is_empty() && empty.is_empty(),
            missing,
            empty,
        },
    })
}

fn receipt_sha(args: &CoverageReceiptArgs) -> String {
    args.sha
        .clone()
        .or_else(|| env_non_empty("GITHUB_SHA"))
        .unwrap_or_else(|| "HEAD".to_string())
}

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn display_path(path: &Path) -> String {
    normalize_slashes(path.to_string_lossy().as_ref())
}

fn normalize_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn receipt_records_artifact_sizes_and_status() {
        let temp = tempfile::tempdir().expect("tempdir");
        let coverage_json = temp.path().join("coverage.json");
        let coverage_text = temp.path().join("coverage.txt");
        let lcov = temp.path().join("lcov.info");
        fs::write(&coverage_json, "{}\n").expect("coverage json");
        fs::write(&coverage_text, "coverage\n").expect("coverage text");
        fs::write(&lcov, "TN:\nSF:src/lib.rs\nend_of_record\n").expect("lcov");

        let args = CoverageReceiptArgs {
            coverage_json,
            coverage_text,
            lcov,
            output: temp.path().join("receipt.json"),
            sha: Some("abc123".to_string()),
            ..CoverageReceiptArgs::default()
        };

        let receipt = coverage_receipt(&args).expect("receipt");
        assert_eq!(receipt.schema, COVERAGE_RECEIPT_SCHEMA);
        assert_eq!(receipt.schema_version, 1);
        assert_eq!(receipt.sha, "abc123");
        assert!(receipt.status.ok);
        assert_eq!(receipt.artifacts.len(), 3);
        assert!(receipt.artifacts.iter().all(|artifact| artifact.non_empty));
        assert!(receipt.artifacts.iter().all(|artifact| artifact.bytes > 0));
    }

    #[test]
    fn receipt_reports_missing_and_empty_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let coverage_json = temp.path().join("coverage.json");
        let coverage_text = temp.path().join("coverage.txt");
        let lcov = temp.path().join("lcov.info");
        fs::write(&coverage_json, "{}\n").expect("coverage json");
        fs::File::create(&coverage_text).expect("coverage text");

        let args = CoverageReceiptArgs {
            coverage_json,
            coverage_text,
            lcov,
            output: temp.path().join("receipt.json"),
            ..CoverageReceiptArgs::default()
        };

        let receipt = coverage_receipt(&args).expect("receipt");
        assert!(!receipt.status.ok);
        assert_eq!(receipt.status.empty.len(), 1);
        assert_eq!(receipt.status.missing.len(), 1);
        assert!(receipt.status.empty[0].ends_with("coverage.txt"));
        assert!(receipt.status.missing[0].ends_with("lcov.info"));
    }

    #[test]
    fn run_writes_pretty_json_receipt() {
        let temp = tempfile::tempdir().expect("tempdir");
        let coverage_json = temp.path().join("coverage.json");
        let coverage_text = temp.path().join("coverage.txt");
        let lcov = temp.path().join("lcov.info");
        let output = temp.path().join("out").join("coverage-receipt.json");
        fs::write(&coverage_json, "{}\n").expect("coverage json");
        fs::write(&coverage_text, "coverage\n").expect("coverage text");
        fs::write(&lcov, "TN:\nSF:src/lib.rs\nend_of_record\n").expect("lcov");

        let args = CoverageReceiptArgs {
            coverage_json,
            coverage_text,
            lcov,
            output: output.clone(),
            sha: Some("abc123".to_string()),
            ..CoverageReceiptArgs::default()
        };

        run(args).expect("run");
        let body = fs::read_to_string(output).expect("receipt body");
        let json: serde_json::Value = serde_json::from_str(&body).expect("receipt json");
        assert_eq!(json["schema"], COVERAGE_RECEIPT_SCHEMA);
        assert_eq!(json["status"]["ok"], true);
    }

    #[test]
    fn display_paths_use_forward_slashes() {
        let path = PathBuf::from("target\\coverage\\coverage-receipt.json");
        assert_eq!(display_path(&path), "target/coverage/coverage-receipt.json");
    }

    #[test]
    fn file_create_is_not_enough_for_non_empty_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("empty.txt");
        let mut file = fs::File::create(&path).expect("create empty");
        file.flush().expect("flush");
        let args = CoverageReceiptArgs {
            coverage_json: path.clone(),
            coverage_text: path.clone(),
            lcov: path,
            output: temp.path().join("receipt.json"),
            ..CoverageReceiptArgs::default()
        };

        let receipt = coverage_receipt(&args).expect("receipt");
        assert_eq!(receipt.status.empty.len(), 3);
    }
}
