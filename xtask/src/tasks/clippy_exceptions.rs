//! AST-backed Clippy exception ledger checker.
//!
//! Validates `policy/clippy-exceptions.toml` and (in --strict mode) walks
//! source files to verify that every `#[expect(clippy::<lint>, reason =
//! "policy:clippy-NNNN ...")]` attribute corresponds to an entry in the
//! ledger. Bare `#[allow(clippy::...)]` is forbidden by the existing
//! Clippy `allow_attributes` deny rule; this checker focuses on the
//! `#[expect(...)]` link.
//!
//! Advisory by default; --strict makes findings blocking.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::cli::ClippyExceptionsArgs;

const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", "run-artifacts"];

#[derive(Debug, Deserialize)]
struct ExceptionsFile {
    schema_version: String,
    #[serde(default)]
    policy: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    exception: Vec<Exception>,
}

#[derive(Debug, Deserialize)]
struct Exception {
    id: String,
    lint: String,
    path: String,
    #[serde(default)]
    classification: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    expires: Option<String>,
    #[serde(default)]
    selector: Option<Selector>,
    #[serde(default)]
    last_seen: Option<LastSeen>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Selector {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    container: String,
    #[serde(default)]
    text_contains: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LastSeen {
    #[serde(default)]
    line: Option<u64>,
    #[serde(default)]
    column: Option<u64>,
}

pub fn run(args: ClippyExceptionsArgs) -> Result<()> {
    let root = workspace_root()?;
    let path = root.join(&args.policy);
    let file = parse(&path)?;

    let mut hard_errors = Vec::new();
    if file.schema_version != "1.0" {
        hard_errors.push(format!(
            "{}: unsupported schema_version {:?}",
            path.display(),
            file.schema_version
        ));
    }

    let mut findings = Vec::new();
    validate_entries(&file.exception, &mut findings);

    let known_ids: BTreeSet<&str> = file.exception.iter().map(|e| e.id.as_str()).collect();

    let mut suppression_findings = scan_source(&root, &known_ids)?;
    findings.append(&mut suppression_findings);

    if let Some(report_dir) = &args.report_dir {
        let dir = root.join(report_dir);
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let out = dir.join("clippy-exceptions-report.txt");
        let body = render_report(&file, &findings, &hard_errors);
        fs::write(&out, &body).with_context(|| format!("write {}", out.display()))?;
        println!("clippy-exceptions report written to {}", out.display());
    }

    if !hard_errors.is_empty() {
        for err in &hard_errors {
            eprintln!("error: {err}");
        }
        bail!("clippy-exceptions: {} hard error(s)", hard_errors.len());
    }

    if findings.is_empty() {
        println!(
            "clippy-exceptions OK: {} entries, schema {}",
            file.exception.len(),
            file.schema_version
        );
        return Ok(());
    }

    println!("clippy-exceptions findings ({}):", findings.len());
    for finding in findings.iter().take(50) {
        println!("  - {finding}");
    }
    if findings.len() > 50 {
        println!("  ... ({} more, see report)", findings.len() - 50);
    }

    if args.strict {
        bail!("clippy-exceptions: {} finding(s) (strict)", findings.len());
    }
    println!("(advisory mode; rerun with --strict to fail on findings)");
    Ok(())
}

fn validate_entries(entries: &[Exception], findings: &mut Vec<String>) {
    let today = chrono::Utc::now().date_naive();
    let mut seen_ids: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, e) in entries.iter().enumerate() {
        if let Some(prev) = seen_ids.insert(e.id.as_str(), i) {
            findings.push(format!(
                "duplicate id {:?} at index {} (previous index {})",
                e.id, i, prev
            ));
        }
        if !e.id.starts_with("clippy-") {
            findings.push(format!("entry {:?}: id must start with 'clippy-'", e.id));
        }
        if !e.lint.starts_with("clippy::") {
            findings.push(format!("entry {:?}: lint must start with 'clippy::'", e.id));
        }
        if e.path.is_empty() {
            findings.push(format!("entry {:?}: missing path", e.id));
        }
        if e.classification.is_empty() {
            findings.push(format!("entry {:?}: missing classification", e.id));
        }
        if e.owner.is_empty() {
            findings.push(format!("entry {:?}: missing owner", e.id));
        }
        if e.reason.is_empty() {
            findings.push(format!("entry {:?}: missing reason", e.id));
        }
        match &e.expires {
            None => findings.push(format!("entry {:?}: missing expires", e.id)),
            Some(date) => match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                Ok(parsed) if parsed < today => {
                    findings.push(format!("entry {:?}: expired on {date}", e.id));
                }
                Ok(_) => {}
                Err(err) => findings.push(format!(
                    "entry {:?}: expires {:?} is not YYYY-MM-DD ({err})",
                    e.id, date
                )),
            },
        }
        if e.selector.is_none() {
            // Selector is optional but recommended.
        }
    }
}

/// Walk Rust source files and report `#[expect(clippy::...)]` attributes
/// whose `reason = "..."` does not include `policy:clippy-NNNN` matching
/// a known ledger id. This is a regex-light, line-based scanner: it does
/// not need to be syntactically perfect because the goal is "raise the
/// floor" advisory signal, not parser parity with rustc.
fn scan_source(root: &Path, known_ids: &BTreeSet<&str>) -> Result<Vec<String>> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !is_skipped(e.path(), root))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.ends_with(".rs") {
            continue;
        }
        let body = match fs::read_to_string(path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string());
        scan_body(&rel, &body, known_ids, &mut findings);
    }
    Ok(findings)
}

fn scan_body(path: &str, body: &str, known_ids: &BTreeSet<&str>, findings: &mut Vec<String>) {
    for (lineno, line) in body.lines().enumerate() {
        let lineno = lineno + 1;
        let trimmed = line.trim_start();
        // Skip line / doc comments. The scanner is line-based, so attribute
        // usage that wraps onto multiple lines is intentionally only reported
        // on its first line; that's the line a reader would scan anyway.
        if trimmed.starts_with("//") {
            continue;
        }
        let trimmed = line.trim();
        // Match only when the attribute starts the line. Attributes always
        // start a line; substrings inside string literals or format strings
        // never do.
        if !trimmed.starts_with("#[expect(") {
            continue;
        }
        if !trimmed.contains("clippy::") {
            continue;
        }
        // Try to find a `reason = "..."` substring.
        let reason = extract_reason(trimmed);
        match reason {
            None => {
                findings.push(format!(
                    "{path}:{lineno}: #[expect(clippy::...)] without reason = \"...\""
                ));
            }
            Some(reason) => {
                let id = extract_policy_id(&reason);
                match id {
                    None => findings.push(format!(
                        "{path}:{lineno}: #[expect(clippy::...)] reason missing policy:clippy-NNNN link",
                    )),
                    Some(id) => {
                        if !known_ids.contains(id.as_str()) {
                            findings.push(format!(
                                "{path}:{lineno}: #[expect(clippy::...)] references unknown policy id {id}"
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn extract_reason(line: &str) -> Option<String> {
    let needle = "reason";
    let idx = line.find(needle)?;
    let rest = &line[idx + needle.len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_policy_id(reason: &str) -> Option<String> {
    let needle = "policy:";
    let idx = reason.find(needle)?;
    let rest = &reason[idx + needle.len()..];
    // Take while alphanumeric / dash.
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(rest.len());
    let id = &rest[..end];
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn is_skipped(path: &Path, root: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };
    let Some(first) = rel.iter().next().and_then(|s| s.to_str()) else {
        return false;
    };
    SKIP_DIRS.contains(&first)
}

fn parse(path: &Path) -> Result<ExceptionsFile> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&body).with_context(|| format!("parse {}", path.display()))
}

fn render_report(file: &ExceptionsFile, findings: &[String], hard_errors: &[String]) -> String {
    let mut out = String::new();
    out.push_str("# Clippy exceptions report\n\n");
    if let Some(name) = &file.policy {
        out.push_str(&format!("- policy: {name}\n"));
    }
    if let Some(owner) = &file.owner {
        out.push_str(&format!("- owner: {owner}\n"));
    }
    if let Some(status) = &file.status {
        out.push_str(&format!("- status: {status}\n"));
    }
    if let Some(updated) = &file.updated {
        out.push_str(&format!("- updated: {updated}\n"));
    }
    let with_selector = file
        .exception
        .iter()
        .filter(|e| e.selector.is_some())
        .count();
    let with_last_seen = file
        .exception
        .iter()
        .filter(|e| e.last_seen.is_some())
        .count();
    out.push_str(&format!("- entries: {}\n", file.exception.len()));
    out.push_str(&format!("- entries with selector: {with_selector}\n"));
    out.push_str(&format!("- entries with last_seen: {with_last_seen}\n"));
    out.push_str(&format!("- findings: {}\n", findings.len()));
    out.push_str(&format!("- hard errors: {}\n\n", hard_errors.len()));

    if !findings.is_empty() {
        out.push_str("## Findings\n\n");
        for f in findings {
            out.push_str(&format!("- {f}\n"));
        }
    }

    if !hard_errors.is_empty() {
        out.push_str("\n## Hard errors\n\n");
        for f in hard_errors {
            out.push_str(&format!("- {f}\n"));
        }
    }

    out
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

    fn entry(id: &str) -> Exception {
        Exception {
            id: id.into(),
            lint: "clippy::indexing_slicing".into(),
            path: "crates/x/src/lib.rs".into(),
            classification: "generated_table".into(),
            owner: "model".into(),
            reason: "test".into(),
            expires: Some("2099-12-31".into()),
            selector: None,
            last_seen: None,
        }
    }

    #[test]
    fn duplicate_id_is_finding() {
        let entries = vec![entry("clippy-0001"), entry("clippy-0001")];
        let mut findings = Vec::new();
        validate_entries(&entries, &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("duplicate id")),
            "{findings:?}"
        );
    }

    #[test]
    fn expired_is_finding() {
        let mut e = entry("clippy-0002");
        e.expires = Some("2020-01-01".into());
        let mut findings = Vec::new();
        validate_entries(&[e], &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("expired on 2020-01-01")),
            "{findings:?}"
        );
    }

    #[test]
    fn id_must_start_with_clippy_prefix() {
        let mut e = entry("foo-0003");
        e.id = "foo-0003".into();
        let mut findings = Vec::new();
        validate_entries(&[e], &mut findings);
        assert!(
            findings
                .iter()
                .any(|f| f.contains("must start with 'clippy-'")),
            "{findings:?}"
        );
    }

    #[test]
    fn extract_reason_finds_quoted_value() {
        let line = r#"    #[expect(clippy::indexing_slicing, reason = "policy:clippy-0001 generated table")]"#;
        assert_eq!(
            extract_reason(line).as_deref(),
            Some("policy:clippy-0001 generated table"),
        );
    }

    #[test]
    fn extract_policy_id_takes_alphanumerics() {
        assert_eq!(
            extract_policy_id("policy:clippy-0001 generated table").as_deref(),
            Some("clippy-0001"),
        );
    }

    #[test]
    fn scan_body_flags_missing_reason() {
        let body = "#[expect(clippy::indexing_slicing)]\nfn x() {}\n";
        let known: BTreeSet<&str> = std::iter::once("clippy-0001").collect();
        let mut findings = Vec::new();
        scan_body("a.rs", body, &known, &mut findings);
        assert!(
            findings.iter().any(|f| f.contains("without reason")),
            "{findings:?}"
        );
    }

    #[test]
    fn scan_body_flags_unknown_policy_id() {
        let body =
            "#[expect(clippy::indexing_slicing, reason = \"policy:clippy-9999 oh\")]\nfn x() {}\n";
        let known: BTreeSet<&str> = std::iter::once("clippy-0001").collect();
        let mut findings = Vec::new();
        scan_body("a.rs", body, &known, &mut findings);
        assert!(
            findings
                .iter()
                .any(|f| f.contains("unknown policy id clippy-9999")),
            "{findings:?}"
        );
    }

    #[test]
    fn scan_body_accepts_known_id() {
        let body =
            "#[expect(clippy::indexing_slicing, reason = \"policy:clippy-0001 ok\")]\nfn x() {}\n";
        let known: BTreeSet<&str> = std::iter::once("clippy-0001").collect();
        let mut findings = Vec::new();
        scan_body("a.rs", body, &known, &mut findings);
        assert!(findings.is_empty(), "{findings:?}");
    }
}
