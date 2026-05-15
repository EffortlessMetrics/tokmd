use crate::cli::{AstShadowCheckArgs, AstShadowCompareArgs};
use crate::tasks::ast_shadow_compare;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tokmd_analysis::ast::{AST_SHADOW_SCHEMA_VERSION, default_shadow_artifacts};

const AST_SHADOW_CHECK_SCHEMA: &str = "tokmd.ast_shadow_check.v1";

pub fn run(args: AstShadowCheckArgs) -> Result<()> {
    if !args.paths.is_empty() || args.manifest.is_some() {
        ast_shadow_compare::run(AstShadowCompareArgs {
            paths: args.paths.clone(),
            manifest: args.manifest.clone(),
            out: args.dir.clone(),
            summary_md: None,
        })?;
    }

    let report = validate_ast_shadow_dir(&args.dir)?;
    if let Some(path) = &args.json {
        write_check_receipt(path, &report)?;
    }

    println!(
        "AST shadow artifacts OK: {} artifact(s), {} file(s), {} matched landmark(s), {} parse-degraded file(s) in `{}`",
        report.artifact_count,
        report.summary.files,
        report.summary.matched,
        report.summary.parse_degraded,
        args.dir.display()
    );
    Ok(())
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct AstShadowCheckReport {
    schema: &'static str,
    ok: bool,
    artifact_count: usize,
    artifacts: Vec<VerifiedAstShadowArtifact>,
    summary: AstShadowDiffSummary,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct VerifiedAstShadowArtifact {
    path: String,
    kind: String,
    schema: String,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
struct AstShadowDiffSummary {
    files: usize,
    matched: usize,
    heuristic_only: usize,
    ast_only: usize,
    parse_degraded: usize,
    unsupported: usize,
}

fn validate_ast_shadow_dir(dir: &Path) -> Result<AstShadowCheckReport> {
    if !dir.is_dir() {
        bail!(
            "AST shadow artifact directory does not exist: {}",
            dir.display()
        );
    }

    let artifacts = default_shadow_artifacts();
    let expected = [
        ("heuristic", artifacts.heuristic),
        ("ast", artifacts.ast),
        ("diff", artifacts.diff),
    ];

    let heuristic = read_json(&dir.join(artifacts.heuristic), artifacts.heuristic)?;
    let ast = read_json(&dir.join(artifacts.ast), artifacts.ast)?;
    let diff = read_json(&dir.join(artifacts.diff), artifacts.diff)?;

    let mut errors = Vec::new();
    let mut verified = Vec::with_capacity(expected.len());

    for (kind, path) in expected {
        let value = match kind {
            "heuristic" => &heuristic,
            "ast" => &ast,
            "diff" => &diff,
            _ => unreachable!("known AST shadow artifact kind"),
        };
        validate_schema_and_kind(value, path, kind, &mut errors);
        validate_no_environment_leakage(value, path, &mut errors);
        verified.push(VerifiedAstShadowArtifact {
            path: path.to_owned(),
            kind: kind.to_owned(),
            schema: schema_value(value).unwrap_or_default().to_owned(),
        });
    }

    let heuristic_paths = validate_files_array(&heuristic, artifacts.heuristic, &mut errors);
    let ast_paths = validate_files_array(&ast, artifacts.ast, &mut errors);
    let diff_paths = validate_files_array(&diff, artifacts.diff, &mut errors);
    validate_path_sets_match(&heuristic_paths, &ast_paths, &diff_paths, &mut errors);

    let summary = validate_diff_summary(&diff, artifacts.diff, &mut errors);

    if !errors.is_empty() {
        bail!(
            "AST shadow artifact check failed:\n- {}",
            errors.join("\n- ")
        );
    }

    Ok(AstShadowCheckReport {
        schema: AST_SHADOW_CHECK_SCHEMA,
        ok: true,
        artifact_count: expected.len(),
        artifacts: verified,
        summary,
        errors,
    })
}

fn write_check_receipt(path: &Path, report: &AstShadowCheckReport) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let json =
        serde_json::to_string_pretty(report).context("serialize AST shadow check receipt")?;
    fs::write(path, format!("{json}\n")).with_context(|| format!("write {}", path.display()))
}

fn read_json(path: &Path, label: &str) -> Result<Value> {
    let content = fs::read_to_string(path).with_context(|| format!("failed to read {label}"))?;
    serde_json::from_str(&content).with_context(|| format!("failed to parse {label}"))
}

fn validate_schema_and_kind(
    value: &Value,
    label: &str,
    expected_kind: &str,
    errors: &mut Vec<String>,
) {
    match schema_value(value) {
        Some(schema) if schema == AST_SHADOW_SCHEMA_VERSION => {}
        Some(schema) => errors.push(format!(
            "{label} schema `{schema}` does not match `{AST_SHADOW_SCHEMA_VERSION}`"
        )),
        None => errors.push(format!("{label} is missing string field `schema`")),
    }

    match value.get("kind").and_then(Value::as_str) {
        Some(kind) if kind == expected_kind => {}
        Some(kind) => errors.push(format!(
            "{label} kind `{kind}` does not match `{expected_kind}`"
        )),
        None => errors.push(format!("{label} is missing string field `kind`")),
    }
}

fn schema_value(value: &Value) -> Option<&str> {
    value.get("schema").and_then(Value::as_str)
}

fn validate_files_array(value: &Value, label: &str, errors: &mut Vec<String>) -> Vec<String> {
    let Some(files) = value.get("files").and_then(Value::as_array) else {
        errors.push(format!("{label} is missing array field `files`"));
        return Vec::new();
    };

    let mut paths = Vec::with_capacity(files.len());
    let mut previous: Option<String> = None;
    for (index, file) in files.iter().enumerate() {
        let Some(path) = file.get("path").and_then(Value::as_str) else {
            errors.push(format!(
                "{label} files[{index}] is missing string field `path`"
            ));
            continue;
        };

        validate_relative_artifact_path(path, &format!("{label} files[{index}].path"), errors);
        if let Some(previous) = &previous
            && previous.as_str() > path
        {
            errors.push(format!(
                "{label} files are not sorted by path: `{previous}` appears before `{path}`"
            ));
        }
        previous = Some(path.to_owned());
        paths.push(path.to_owned());

        if label == "diff.json" {
            validate_diff_file_entry(file, label, index, errors);
        }
    }

    paths
}

fn validate_path_sets_match(
    heuristic_paths: &[String],
    ast_paths: &[String],
    diff_paths: &[String],
    errors: &mut Vec<String>,
) {
    if heuristic_paths != ast_paths {
        errors.push("heuristic.json and ast.json file paths differ".to_owned());
    }
    if heuristic_paths != diff_paths {
        errors.push("heuristic.json and diff.json file paths differ".to_owned());
    }
}

fn validate_diff_file_entry(file: &Value, label: &str, index: usize, errors: &mut Vec<String>) {
    let status = file.get("status").and_then(Value::as_str);
    let parse_degraded = file.get("parse_degraded").and_then(Value::as_bool);
    let unsupported = file.get("unsupported").and_then(Value::as_bool);

    match status {
        Some("compared" | "parse_degraded" | "unsupported") => {}
        Some(other) => errors.push(format!(
            "{label} files[{index}].status `{other}` is unknown"
        )),
        None => errors.push(format!(
            "{label} files[{index}] is missing string field `status`"
        )),
    }

    let Some(parse_degraded) = parse_degraded else {
        errors.push(format!(
            "{label} files[{index}] is missing bool field `parse_degraded`"
        ));
        return;
    };
    let Some(unsupported) = unsupported else {
        errors.push(format!(
            "{label} files[{index}] is missing bool field `unsupported`"
        ));
        return;
    };

    if parse_degraded && status != Some("parse_degraded") {
        errors.push(format!(
            "{label} files[{index}] has parse_degraded=true but status is not `parse_degraded`"
        ));
    }
    if unsupported && status != Some("unsupported") {
        errors.push(format!(
            "{label} files[{index}] has unsupported=true but status is not `unsupported`"
        ));
    }

    for field in ["matches", "heuristic_only", "ast_only"] {
        if !file.get(field).is_some_and(Value::is_array) {
            errors.push(format!(
                "{label} files[{index}] is missing array field `{field}`"
            ));
        }
    }
}

fn validate_diff_summary(
    diff: &Value,
    label: &str,
    errors: &mut Vec<String>,
) -> AstShadowDiffSummary {
    let observed = observed_diff_summary(diff, label, errors);
    let declared = declared_diff_summary(diff, label, errors);

    if let Some(declared) = declared {
        if declared != observed {
            errors.push(format!(
                "{label} summary does not match file entries: declared {:?}, observed {:?}",
                declared, observed
            ));
        }
    } else {
        errors.push(format!("{label} is missing object field `summary`"));
    }

    observed
}

fn observed_diff_summary(
    diff: &Value,
    label: &str,
    errors: &mut Vec<String>,
) -> AstShadowDiffSummary {
    let Some(files) = diff.get("files").and_then(Value::as_array) else {
        return AstShadowDiffSummary::default();
    };

    let mut summary = AstShadowDiffSummary {
        files: files.len(),
        ..AstShadowDiffSummary::default()
    };

    for (index, file) in files.iter().enumerate() {
        summary.matched += array_len(file, "matches", label, index, errors);
        summary.heuristic_only += array_len(file, "heuristic_only", label, index, errors);
        summary.ast_only += array_len(file, "ast_only", label, index, errors);
        summary.parse_degraded += usize::from(
            file.get("parse_degraded")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        summary.unsupported += usize::from(
            file.get("unsupported")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
    }

    summary
}

fn declared_diff_summary(
    diff: &Value,
    label: &str,
    errors: &mut Vec<String>,
) -> Option<AstShadowDiffSummary> {
    let summary = diff.get("summary")?.as_object()?;
    Some(AstShadowDiffSummary {
        files: unsigned_field(summary.get("files"), label, "summary.files", errors),
        matched: unsigned_field(summary.get("matched"), label, "summary.matched", errors),
        heuristic_only: unsigned_field(
            summary.get("heuristic_only"),
            label,
            "summary.heuristic_only",
            errors,
        ),
        ast_only: unsigned_field(summary.get("ast_only"), label, "summary.ast_only", errors),
        parse_degraded: unsigned_field(
            summary.get("parse_degraded"),
            label,
            "summary.parse_degraded",
            errors,
        ),
        unsupported: unsigned_field(
            summary.get("unsupported"),
            label,
            "summary.unsupported",
            errors,
        ),
    })
}

fn unsigned_field(
    value: Option<&Value>,
    label: &str,
    field: &str,
    errors: &mut Vec<String>,
) -> usize {
    match value.and_then(Value::as_u64) {
        Some(value) => usize::try_from(value).unwrap_or(usize::MAX),
        None => {
            errors.push(format!(
                "{label} is missing unsigned integer field `{field}`"
            ));
            0
        }
    }
}

fn array_len(
    value: &Value,
    field: &str,
    label: &str,
    index: usize,
    errors: &mut Vec<String>,
) -> usize {
    match value.get(field).and_then(Value::as_array) {
        Some(values) => values.len(),
        None => {
            errors.push(format!(
                "{label} files[{index}] is missing array field `{field}`"
            ));
            0
        }
    }
}

fn validate_no_environment_leakage(value: &Value, label: &str, errors: &mut Vec<String>) {
    validate_no_environment_leakage_at(value, label, "$", errors);
}

fn validate_no_environment_leakage_at(
    value: &Value,
    label: &str,
    pointer: &str,
    errors: &mut Vec<String>,
) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if matches!(key.as_str(), "generated_at" | "created_at" | "timestamp") {
                    errors.push(format!(
                        "{label} contains forbidden timestamp field `{pointer}.{key}`"
                    ));
                }
                validate_no_environment_leakage_at(
                    value,
                    label,
                    &format!("{pointer}.{key}"),
                    errors,
                );
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                validate_no_environment_leakage_at(
                    value,
                    label,
                    &format!("{pointer}[{index}]"),
                    errors,
                );
            }
        }
        Value::String(value) if is_absolute_like(value) || looks_like_temp_dir(value) => {
            errors.push(format!(
                "{label} contains environment-specific path-like string at `{pointer}`: `{value}`"
            ));
        }
        Value::String(_) => {}
        _ => {}
    }
}

fn validate_relative_artifact_path(path: &str, label: &str, errors: &mut Vec<String>) {
    if path.is_empty() {
        errors.push(format!("{label} is empty"));
    }
    if path.contains('\\') {
        errors.push(format!("{label} is not normalized to `/`: `{path}`"));
    }
    if is_absolute_like(path) {
        errors.push(format!("{label} is absolute: `{path}`"));
    }
    if path.split('/').any(|component| component == "..") {
        errors.push(format!("{label} escapes the artifact root: `{path}`"));
    }
}

fn is_absolute_like(value: &str) -> bool {
    let normalized = value.replace('\\', "/");
    let bytes = normalized.as_bytes();
    normalized.starts_with('/')
        || bytes
            .get(0..2)
            .is_some_and(|prefix| prefix[0].is_ascii_alphabetic() && prefix[1] == b':')
}

fn looks_like_temp_dir(value: &str) -> bool {
    let normalized = value.replace('\\', "/").to_ascii_lowercase();
    normalized.contains("/appdata/local/temp/")
        || normalized.contains("/temp/")
        || normalized.contains("/tmp/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn valid_artifacts_pass_and_emit_receipt() -> Result<()> {
        let temp = valid_artifact_dir()?;
        let report = validate_ast_shadow_dir(temp.path())?;

        assert_eq!(report.schema, AST_SHADOW_CHECK_SCHEMA);
        assert!(report.ok);
        assert_eq!(report.artifact_count, 3);
        assert_eq!(report.summary.files, 2);
        assert_eq!(report.summary.matched, 2);
        assert_eq!(report.summary.heuristic_only, 1);
        assert_eq!(report.summary.ast_only, 1);
        assert_eq!(report.summary.parse_degraded, 1);

        let receipt = temp.path().join("check.json");
        write_check_receipt(&receipt, &report)?;
        let written = fs::read_to_string(receipt)?;
        assert!(written.contains("\"schema\": \"tokmd.ast_shadow_check.v1\""));
        assert!(written.ends_with('\n'));
        Ok(())
    }

    #[test]
    fn check_can_generate_fixture_artifacts_before_verifying() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let current_dir = std::env::current_dir()?;
        let scratch_root = current_dir.join("target");
        fs::create_dir_all(&scratch_root)?;
        let source_root = tempfile::tempdir_in(scratch_root)?;
        let source = source_root.path().join("input.rs");
        fs::write(
            &source,
            "use std::path::Path;\npub fn fixture(value: usize) -> usize {\n    if value > 0 { value } else { 0 }\n}\n",
        )?;
        let source = source.strip_prefix(current_dir)?.to_path_buf();
        let out = temp.path().join("ast-shadow");
        let receipt = temp.path().join("check.json");

        run(AstShadowCheckArgs {
            paths: vec![source],
            manifest: None,
            dir: out.clone(),
            json: Some(receipt.clone()),
        })?;

        assert!(out.join("heuristic.json").is_file());
        assert!(out.join("ast.json").is_file());
        assert!(out.join("diff.json").is_file());
        let report = read_json(&receipt, "check.json")?;
        assert_eq!(
            report.get("schema").and_then(Value::as_str),
            Some(AST_SHADOW_CHECK_SCHEMA)
        );
        assert_eq!(report["summary"]["files"], json!(1));
        Ok(())
    }

    #[test]
    fn check_can_generate_manifest_artifacts_before_verifying() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let current_dir = std::env::current_dir()?;
        let scratch_root = current_dir.join("target");
        fs::create_dir_all(&scratch_root)?;
        let source_root = tempfile::tempdir_in(scratch_root)?;
        let source_dir = source_root.path().join("fixtures");
        fs::create_dir_all(&source_dir)?;
        fs::write(
            source_dir.join("input.rs"),
            "use std::fs;\npub fn fixture() {}\n",
        )?;
        let source = source_dir
            .join("input.rs")
            .strip_prefix(&current_dir)?
            .to_path_buf();
        let manifest = source_root.path().join("ast-shadow-corpus.toml");
        fs::write(
            &manifest,
            format!(
                "schema = \"tokmd.ast_shadow_corpus.v1\"\nlanguage = \"rust\"\n\n[[file]]\npath = \"{}\"\n",
                source.to_string_lossy().replace('\\', "/")
            ),
        )?;
        let manifest = manifest.strip_prefix(&current_dir)?.to_path_buf();
        let out = temp.path().join("ast-shadow");

        run(AstShadowCheckArgs {
            paths: Vec::new(),
            manifest: Some(manifest),
            dir: out.clone(),
            json: None,
        })?;

        assert!(out.join("heuristic.json").is_file());
        assert!(out.join("ast.json").is_file());
        assert!(out.join("diff.json").is_file());
        Ok(())
    }

    #[test]
    fn missing_artifact_fails() -> Result<()> {
        let temp = valid_artifact_dir()?;
        fs::remove_file(temp.path().join("diff.json"))?;

        let error = validate_ast_shadow_dir(temp.path()).expect_err("missing diff should fail");

        assert!(error.to_string().contains("failed to read diff.json"));
        Ok(())
    }

    #[test]
    fn summary_drift_fails() -> Result<()> {
        let temp = valid_artifact_dir()?;
        let diff_path = temp.path().join("diff.json");
        let mut diff = read_json(&diff_path, "diff.json")?;
        diff["summary"]["matched"] = json!(99);
        write_json(&diff_path, &diff)?;

        let error = validate_ast_shadow_dir(temp.path()).expect_err("summary drift should fail");

        assert!(
            error
                .to_string()
                .contains("summary does not match file entries")
        );
        Ok(())
    }

    #[test]
    fn absolute_paths_fail() -> Result<()> {
        let temp = valid_artifact_dir()?;
        let heuristic_path = temp.path().join("heuristic.json");
        let mut heuristic = read_json(&heuristic_path, "heuristic.json")?;
        heuristic["files"][0]["path"] = json!("C:\\repo\\src\\lib.rs");
        write_json(&heuristic_path, &heuristic)?;

        let error = validate_ast_shadow_dir(temp.path()).expect_err("absolute paths should fail");

        assert!(error.to_string().contains("absolute"));
        Ok(())
    }

    #[test]
    fn unsorted_paths_fail() -> Result<()> {
        let temp = valid_artifact_dir()?;
        let ast_path = temp.path().join("ast.json");
        let mut ast = read_json(&ast_path, "ast.json")?;
        ast["files"].as_array_mut().unwrap().reverse();
        write_json(&ast_path, &ast)?;

        let error = validate_ast_shadow_dir(temp.path()).expect_err("unsorted paths should fail");

        assert!(error.to_string().contains("not sorted by path"));
        Ok(())
    }

    #[test]
    fn timestamp_fields_fail() -> Result<()> {
        let temp = valid_artifact_dir()?;
        let diff_path = temp.path().join("diff.json");
        let mut diff = read_json(&diff_path, "diff.json")?;
        diff["generated_at"] = json!("2026-05-14T00:00:00Z");
        write_json(&diff_path, &diff)?;

        let error = validate_ast_shadow_dir(temp.path()).expect_err("timestamps should fail");

        assert!(error.to_string().contains("forbidden timestamp field"));
        Ok(())
    }

    fn valid_artifact_dir() -> Result<TempDir> {
        let temp = tempfile::tempdir()?;
        write_json(
            &temp.path().join("heuristic.json"),
            &json!({
                "schema": "tokmd.ast_shadow.v1",
                "kind": "heuristic",
                "files": [
                    {
                        "path": "fixtures/ast-shadow/rust/basic.rs",
                        "language": "rust",
                        "source": "caller_supplied",
                        "landmarks": [
                            landmark("function", "compute"),
                            landmark("control_flow", "if"),
                            landmark("function", "heuristic_only")
                        ]
                    },
                    {
                        "path": "fixtures/ast-shadow/rust/parse-degraded.rs",
                        "language": "rust",
                        "source": "caller_supplied",
                        "landmarks": []
                    }
                ]
            }),
        )?;
        write_json(
            &temp.path().join("ast.json"),
            &json!({
                "schema": "tokmd.ast_shadow.v1",
                "kind": "ast",
                "capabilities": [],
                "files": [
                    {
                        "path": "fixtures/ast-shadow/rust/basic.rs",
                        "language": "rust",
                        "parser_status": "parser_backed_shadow",
                        "has_error": false,
                        "landmarks": [
                            landmark("function", "compute"),
                            landmark("control_flow", "if"),
                            landmark("function", "ast_only")
                        ]
                    },
                    {
                        "path": "fixtures/ast-shadow/rust/parse-degraded.rs",
                        "language": "rust",
                        "parser_status": "parser_backed_shadow",
                        "has_error": true,
                        "landmarks": []
                    }
                ]
            }),
        )?;
        write_json(
            &temp.path().join("diff.json"),
            &json!({
                "schema": "tokmd.ast_shadow.v1",
                "kind": "diff",
                "summary": {
                    "files": 2,
                    "matched": 2,
                    "heuristic_only": 1,
                    "ast_only": 1,
                    "parse_degraded": 1,
                    "unsupported": 0
                },
                "files": [
                    {
                        "path": "fixtures/ast-shadow/rust/basic.rs",
                        "language": "rust",
                        "status": "compared",
                        "parse_degraded": false,
                        "unsupported": false,
                        "matches": [
                            landmark("function", "compute"),
                            landmark("control_flow", "if")
                        ],
                        "heuristic_only": [
                            landmark("function", "heuristic_only")
                        ],
                        "ast_only": [
                            landmark("function", "ast_only")
                        ]
                    },
                    {
                        "path": "fixtures/ast-shadow/rust/parse-degraded.rs",
                        "language": "rust",
                        "status": "parse_degraded",
                        "parse_degraded": true,
                        "unsupported": false,
                        "matches": [],
                        "heuristic_only": [],
                        "ast_only": []
                    }
                ]
            }),
        )?;
        Ok(temp)
    }

    fn landmark(kind: &str, name: &str) -> Value {
        json!({
            "kind": kind,
            "name": name,
            "start_line": 1,
            "end_line": 1,
        })
    }

    fn write_json(path: &Path, value: &Value) -> Result<()> {
        let mut bytes = serde_json::to_vec_pretty(value)?;
        bytes.push(b'\n');
        fs::write(path, bytes)?;
        Ok(())
    }
}
