use crate::cli::ReviewPacketCheckArgs;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

const MANIFEST_SCHEMA_JSON: &str =
    include_str!("../../../crates/tokmd/schemas/review-packet-manifest.schema.json");
const EVIDENCE_SCHEMA_JSON: &str =
    include_str!("../../../crates/tokmd/schemas/review-packet-evidence.schema.json");
const REVIEW_MAP_SCHEMA_JSON: &str =
    include_str!("../../../crates/tokmd/schemas/review-map.schema.json");

const REQUIRED_PACKET_ARTIFACTS: &[&str] = &[
    "cockpit.json",
    "evidence.json",
    "review-map.json",
    "review-map.md",
    "comment.md",
];
const HOSTED_COMMENT_COPY: &str = "tokmd-review-packet-comment.md";

pub fn run(args: ReviewPacketCheckArgs) -> Result<()> {
    let report = validate_review_packet_dir(&args.dir)?;
    println!(
        "Review packet OK: {} artifact(s) in `{}`",
        report.artifact_count,
        args.dir.display()
    );
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ReviewPacketCheckReport {
    artifact_count: usize,
}

#[derive(Debug, Deserialize)]
struct ReviewPacketManifest {
    artifacts: Vec<ReviewPacketArtifact>,
}

#[derive(Debug, Deserialize)]
struct ReviewPacketArtifact {
    id: String,
    path: String,
    hash: ReviewPacketArtifactHash,
}

#[derive(Debug, Deserialize)]
struct ReviewPacketArtifactHash {
    hash: String,
}

fn validate_review_packet_dir(dir: &Path) -> Result<ReviewPacketCheckReport> {
    if !dir.is_dir() {
        bail!("review packet directory does not exist: {}", dir.display());
    }

    let manifest_path = dir.join("manifest.json");
    let evidence_path = dir.join("evidence.json");
    let review_map_path = dir.join("review-map.json");

    let manifest_value = read_json(&manifest_path, "manifest.json")?;
    let evidence_value = read_json(&evidence_path, "evidence.json")?;
    let review_map_value = read_json(&review_map_path, "review-map.json")?;

    let mut errors = Vec::new();
    errors.extend(validate_json_schema(
        &manifest_value,
        MANIFEST_SCHEMA_JSON,
        "manifest.json",
    )?);
    errors.extend(validate_json_schema(
        &evidence_value,
        EVIDENCE_SCHEMA_JSON,
        "evidence.json",
    )?);
    errors.extend(validate_json_schema(
        &review_map_value,
        REVIEW_MAP_SCHEMA_JSON,
        "review-map.json",
    )?);

    let manifest = serde_json::from_value::<ReviewPacketManifest>(manifest_value)
        .context("manifest.json should match review packet manifest shape")?;

    errors.extend(validate_manifest_artifacts(dir, &manifest.artifacts));

    if !errors.is_empty() {
        bail!("review packet check failed:\n- {}", errors.join("\n- "));
    }

    Ok(ReviewPacketCheckReport {
        artifact_count: manifest.artifacts.len(),
    })
}

fn read_json(path: &Path, label: &str) -> Result<Value> {
    let content = fs::read_to_string(path).with_context(|| format!("failed to read {label}"))?;
    serde_json::from_str(&content).with_context(|| format!("failed to parse {label}"))
}

fn validate_json_schema(document: &Value, schema_json: &str, label: &str) -> Result<Vec<String>> {
    let schema: Value = serde_json::from_str(schema_json)
        .with_context(|| format!("failed to parse embedded schema for {label}"))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| anyhow::anyhow!("failed to compile embedded schema for {label}: {e}"))?;

    Ok(validator
        .iter_errors(document)
        .map(|err| {
            format!(
                "{label} schema validation failed: {} at {}",
                err,
                err.instance_path()
            )
        })
        .collect())
}

fn validate_manifest_artifacts(dir: &Path, artifacts: &[ReviewPacketArtifact]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_paths = BTreeSet::new();

    for artifact in artifacts {
        let relative_path = match packet_relative_path(&artifact.path) {
            Ok(relative_path) => relative_path,
            Err(reason) => {
                errors.push(format!(
                    "artifact `{}` path `{}` is not packet-local: {reason}",
                    artifact.id, artifact.path
                ));
                continue;
            }
        };

        let display_path = portable_path(&relative_path);
        seen_paths.insert(display_path.clone());

        if display_path == HOSTED_COMMENT_COPY {
            errors.push(format!(
                "hosted comment copy `{HOSTED_COMMENT_COPY}` must not be listed in manifest"
            ));
        }

        let artifact_path = dir.join(&relative_path);
        match fs::symlink_metadata(&artifact_path) {
            Ok(metadata) if metadata.file_type().is_file() => {
                verify_artifact_hash(&artifact_path, &display_path, artifact, &mut errors);
            }
            Ok(_) => errors.push(format!(
                "artifact `{}` path `{display_path}` is not a regular file",
                artifact.id
            )),
            Err(err) => errors.push(format!(
                "artifact `{}` path `{display_path}` is missing: {err}",
                artifact.id
            )),
        }
    }

    for required_path in REQUIRED_PACKET_ARTIFACTS {
        if !seen_paths.contains(*required_path) {
            errors.push(format!(
                "manifest is missing required packet artifact `{required_path}`"
            ));
        }
    }

    errors
}

fn verify_artifact_hash(
    artifact_path: &Path,
    display_path: &str,
    artifact: &ReviewPacketArtifact,
    errors: &mut Vec<String>,
) {
    match fs::read(artifact_path) {
        Ok(bytes) => {
            let actual = blake3::hash(&bytes).to_hex().to_string();
            if actual != artifact.hash.hash {
                errors.push(format!(
                    "artifact `{}` path `{display_path}` hash mismatch: expected {}, actual {actual}",
                    artifact.id, artifact.hash.hash
                ));
            }
        }
        Err(err) => errors.push(format!(
            "artifact `{}` path `{display_path}` could not be read: {err}",
            artifact.id
        )),
    }
}

fn packet_relative_path(path: &str) -> std::result::Result<PathBuf, &'static str> {
    let path = Path::new(path);
    if path.as_os_str().is_empty() {
        return Err("path is empty");
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir => return Err("uses parent directory component"),
            Component::RootDir | Component::Prefix(_) => {
                return Err("uses an absolute or rooted path component");
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err("path resolves to the packet directory");
    }

    Ok(normalized)
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{HOSTED_COMMENT_COPY, validate_review_packet_dir};
    use serde_json::{Value, json};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn valid_packet_passes_with_hosted_comment_copy_outside_manifest() {
        let dir = tempdir().expect("tempdir");
        write_valid_packet(dir.path());
        fs::write(dir.path().join(HOSTED_COMMENT_COPY), "hosted comment copy")
            .expect("write hosted copy");

        let report = validate_review_packet_dir(dir.path()).expect("valid packet should pass");

        assert_eq!(report.artifact_count, 5);
    }

    #[test]
    fn hash_drift_is_reported() {
        let dir = tempdir().expect("tempdir");
        write_valid_packet(dir.path());
        fs::write(dir.path().join("comment.md"), "mutated hosted summary").expect("mutate comment");

        let err = validate_review_packet_dir(dir.path()).expect_err("hash drift should fail check");
        let msg = err.to_string();

        assert!(msg.contains("hash mismatch"), "{msg}");
        assert!(msg.contains("comment.md"), "{msg}");
    }

    #[test]
    fn review_map_schema_drift_is_reported() {
        let dir = tempdir().expect("tempdir");
        write_valid_packet(dir.path());
        let mut review_map = read_json(&dir.path().join("review-map.json"));
        review_map["schema"] = json!("tokmd.review_map.v0");
        fs::write(
            dir.path().join("review-map.json"),
            serde_json::to_string_pretty(&review_map).expect("serialize review map"),
        )
        .expect("write review map");

        let err =
            validate_review_packet_dir(dir.path()).expect_err("schema drift should fail check");
        let msg = err.to_string();

        assert!(
            msg.contains("review-map.json schema validation failed"),
            "{msg}"
        );
    }

    #[test]
    fn manifest_parent_dir_artifact_path_is_rejected() {
        let dir = tempdir().expect("tempdir");
        write_valid_packet(dir.path());
        let mut manifest = read_json(&dir.path().join("manifest.json"));
        manifest["artifacts"][0]["path"] = json!("../cockpit.json");
        write_json(&dir.path().join("manifest.json"), &manifest);

        let err = validate_review_packet_dir(dir.path())
            .expect_err("parent-dir artifact path should fail check");
        let msg = err.to_string();

        assert!(msg.contains("parent directory component"), "{msg}");
        assert!(msg.contains("cockpit"), "{msg}");
    }

    #[test]
    fn hosted_comment_copy_must_not_be_listed_in_manifest() {
        let dir = tempdir().expect("tempdir");
        write_valid_packet(dir.path());
        fs::write(dir.path().join(HOSTED_COMMENT_COPY), "hosted comment copy")
            .expect("write hosted copy");

        let mut manifest = read_json(&dir.path().join("manifest.json"));
        let hosted_artifact = artifact(
            "hosted-comment",
            HOSTED_COMMENT_COPY,
            "markdown",
            "text/markdown",
            "hosted comment copy",
        );
        manifest["artifacts"]
            .as_array_mut()
            .expect("artifacts array")
            .push(hosted_artifact);
        write_json(&dir.path().join("manifest.json"), &manifest);

        let err = validate_review_packet_dir(dir.path())
            .expect_err("hosted comment copy in manifest should fail check");
        let msg = err.to_string();

        assert!(msg.contains("must not be listed in manifest"), "{msg}");
    }

    fn write_valid_packet(dir: &Path) {
        fs::create_dir_all(dir).expect("create packet dir");

        let cockpit_json = r#"{"schema_version":3,"mode":"cockpit"}"#;
        let evidence_json = serde_json::to_string_pretty(&json!({
            "schema": "tokmd.review_packet_evidence.v1",
            "overall_status": "pass",
            "base_ref": "main",
            "head_ref": "feature",
            "gates": [],
        }))
        .expect("serialize evidence");
        let review_map_json = serde_json::to_string_pretty(&json!({
            "schema": "tokmd.review_map.v1",
            "base_ref": "main",
            "head_ref": "feature",
            "source": "cockpit.review_plan",
            "item_count": 0,
            "items": [],
        }))
        .expect("serialize review map");
        let review_map_md = "# Review Map\n\nNo prioritized files were identified.\n";
        let comment_md = "## tokmd cockpit\n\nReview packet artifacts.\n";

        fs::write(dir.join("cockpit.json"), cockpit_json).expect("write cockpit");
        fs::write(dir.join("evidence.json"), &evidence_json).expect("write evidence");
        fs::write(dir.join("review-map.json"), &review_map_json).expect("write review map json");
        fs::write(dir.join("review-map.md"), review_map_md).expect("write review map md");
        fs::write(dir.join("comment.md"), comment_md).expect("write comment");

        let manifest = json!({
            "schema": "tokmd.review_packet_manifest.v1",
            "generated_by": {
                "name": "tokmd",
                "version": "1.10.0",
                "mode": "cockpit",
                "arguments": ["cockpit", "--review-packet-dir"],
            },
            "generated_at_ms": 0,
            "base_ref": "main",
            "head_ref": "feature",
            "verdict": {
                "status": "pass",
                "blocking": false,
                "reason": "cockpit review packets are advisory by default",
                "evidence": evidence_summary(),
            },
            "capabilities": {
                "evidence": evidence_capabilities(),
            },
            "artifacts": [
                artifact(
                    "cockpit",
                    "cockpit.json",
                    "tokmd.cockpit_receipt.v3",
                    "application/json",
                    cockpit_json,
                ),
                artifact(
                    "evidence",
                    "evidence.json",
                    "tokmd.review_packet_evidence.v1",
                    "application/json",
                    &evidence_json,
                ),
                artifact(
                    "review-map",
                    "review-map.json",
                    "tokmd.review_map.v1",
                    "application/json",
                    &review_map_json,
                ),
                artifact(
                    "review-map-md",
                    "review-map.md",
                    "markdown",
                    "text/markdown",
                    review_map_md,
                ),
                artifact(
                    "comment",
                    "comment.md",
                    "markdown",
                    "text/markdown",
                    comment_md,
                ),
            ],
        });
        write_json(&dir.join("manifest.json"), &manifest);
    }

    fn evidence_summary() -> Value {
        json!({
            "details": "evidence.json#/gates",
            "total_gates": 0,
            "available": 0,
            "degraded": 0,
            "stale": 0,
            "skipped": 0,
            "unavailable": 0,
            "missing": 0,
        })
    }

    fn evidence_capabilities() -> Value {
        json!({
            "details": "evidence.json#/gates",
            "available": [],
            "degraded": [],
            "stale": [],
            "skipped": [],
            "unavailable": [],
            "missing": [],
        })
    }

    fn artifact(id: &str, path: &str, schema: &str, media_type: &str, content: &str) -> Value {
        json!({
            "id": id,
            "path": path,
            "schema": schema,
            "media_type": media_type,
            "hash": {
                "algo": "blake3",
                "hash": blake3::hash(content.as_bytes()).to_hex().to_string(),
            },
        })
    }

    fn read_json(path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
    }

    fn write_json(path: &Path, value: &Value) {
        fs::write(
            path,
            serde_json::to_string_pretty(value).expect("serialize json"),
        )
        .expect("write json");
    }
}
