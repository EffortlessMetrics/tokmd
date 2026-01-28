use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{LicenseFinding, LicenseReport, LicenseSourceKind};

use crate::analysis::AnalysisLimits;

const DEFAULT_MAX_LICENSE_BYTES: u64 = 256 * 1024;

pub(crate) fn build_license_report(
    root: &Path,
    files: &[PathBuf],
    limits: &AnalysisLimits,
) -> Result<LicenseReport> {
    let candidates = tokmd_walk::license_candidates(files);
    let max_bytes = limits
        .max_file_bytes
        .unwrap_or(DEFAULT_MAX_LICENSE_BYTES) as usize;

    let mut findings: Vec<LicenseFinding> = Vec::new();
    let mut extra_license_files: BTreeSet<PathBuf> = BTreeSet::new();

    for rel in &candidates.metadata_files {
        let path = root.join(rel);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if let Some(spdx) = parse_metadata_license(&path)? {
            findings.push(LicenseFinding {
                spdx,
                confidence: 0.95,
                source_path: rel_str.clone(),
                source_kind: LicenseSourceKind::Metadata,
            });
        }
        if let Some(license_file) = parse_metadata_license_file(&path)? {
            extra_license_files.insert(PathBuf::from(license_file));
        }
    }

    let mut text_files: BTreeSet<PathBuf> = candidates.license_files.into_iter().collect();
    text_files.extend(extra_license_files);

    for rel in text_files {
        let path = root.join(&rel);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let text = tokmd_content::read_text_capped(&path, max_bytes)?;
        if text.is_empty() {
            continue;
        }
        if let Some((spdx, confidence)) = match_license_text(&text) {
            findings.push(LicenseFinding {
                spdx,
                confidence,
                source_path: rel_str,
                source_kind: LicenseSourceKind::Text,
            });
        }
    }

    findings.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.spdx.cmp(&b.spdx))
            .then_with(|| a.source_path.cmp(&b.source_path))
    });

    let effective = findings.first().map(|f| f.spdx.clone());

    Ok(LicenseReport { findings, effective })
}

fn parse_metadata_license(path: &Path) -> Result<Option<String>> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    if file_name == "cargo.toml" {
        return Ok(parse_toml_license(path, "package"));
    }
    if file_name == "pyproject.toml" {
        return Ok(parse_toml_license(path, "project")
            .or_else(|| parse_toml_license(path, "tool.poetry")));
    }
    if file_name == "package.json" {
        return Ok(parse_package_json_license(path));
    }
    Ok(None)
}

fn parse_metadata_license_file(path: &Path) -> Result<Option<String>> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    if file_name != "cargo.toml" {
        return Ok(None);
    }
    let text = tokmd_content::read_text_capped(path, DEFAULT_MAX_LICENSE_BYTES as usize)?;
    Ok(parse_toml_key(&text, "package", "license-file"))
}

fn parse_toml_license(path: &Path, section: &str) -> Option<String> {
    let text = tokmd_content::read_text_capped(path, DEFAULT_MAX_LICENSE_BYTES as usize).ok()?;
    parse_toml_key(&text, section, "license")
}

fn parse_toml_key(text: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let name = line.trim_matches(&['[', ']'][..]).trim();
            in_section = name == section;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some(value) = parse_key_value(line, key) {
            return Some(value);
        }
    }
    None
}

fn parse_key_value(line: &str, key: &str) -> Option<String> {
    let mut parts = line.splitn(2, '=');
    let left = parts.next()?.trim();
    let right = parts.next()?.trim();
    if left != key {
        return None;
    }
    extract_quoted(right)
}

fn extract_quoted(text: &str) -> Option<String> {
    let mut chars = text.chars();
    let mut quote = None;
    while let Some(c) = chars.next() {
        if c == '"' || c == '\'' {
            quote = Some(c);
            break;
        }
    }
    let quote = quote?;
    let mut out = String::new();
    for c in chars {
        if c == quote {
            break;
        }
        out.push(c);
    }
    if out.is_empty() { None } else { Some(out) }
}

fn parse_package_json_license(path: &Path) -> Option<String> {
    let text = tokmd_content::read_text_capped(path, DEFAULT_MAX_LICENSE_BYTES as usize).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    match value.get("license") {
        Some(serde_json::Value::String(s)) => Some(s.trim().to_string()),
        Some(serde_json::Value::Object(obj)) => obj
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string()),
        _ => None,
    }
}

fn match_license_text(text: &str) -> Option<(String, f32)> {
    let lower = text.to_lowercase();
    let patterns = license_patterns();
    let mut best: Option<(String, f32)> = None;

    for pattern in patterns {
        let hits = pattern
            .phrases
            .iter()
            .filter(|phrase| lower.contains(*phrase))
            .count();
        if hits < pattern.min_hits {
            continue;
        }
        let confidence = 0.6 + 0.4 * (hits as f32 / pattern.phrases.len() as f32);
        let candidate = (pattern.spdx.to_string(), confidence);
        if best
            .as_ref()
            .map(|(_, c)| confidence > *c)
            .unwrap_or(true)
        {
            best = Some(candidate);
        }
    }

    best
}

struct LicensePattern {
    spdx: &'static str,
    phrases: &'static [&'static str],
    min_hits: usize,
}

fn license_patterns() -> Vec<LicensePattern> {
    vec![
        LicensePattern {
            spdx: "MIT",
            phrases: &[
                "permission is hereby granted, free of charge",
                "the software is provided \"as is\"",
            ],
            min_hits: 1,
        },
        LicensePattern {
            spdx: "Apache-2.0",
            phrases: &[
                "apache license",
                "version 2.0",
                "http://www.apache.org/licenses/",
                "limitations under the license",
            ],
            min_hits: 2,
        },
        LicensePattern {
            spdx: "GPL-3.0-or-later",
            phrases: &[
                "gnu general public license",
                "version 3",
                "any later version",
            ],
            min_hits: 2,
        },
        LicensePattern {
            spdx: "AGPL-3.0-or-later",
            phrases: &[
                "gnu affero general public license",
                "version 3",
                "any later version",
            ],
            min_hits: 2,
        },
        LicensePattern {
            spdx: "BSD-3-Clause",
            phrases: &[
                "redistribution and use in source and binary forms",
                "neither the name of",
                "contributors may be used",
            ],
            min_hits: 2,
        },
        LicensePattern {
            spdx: "BSD-2-Clause",
            phrases: &[
                "redistribution and use in source and binary forms",
                "this software is provided by the copyright holders and contributors \"as is\"",
            ],
            min_hits: 1,
        },
        LicensePattern {
            spdx: "MPL-2.0",
            phrases: &[
                "mozilla public license",
                "version 2.0",
                "http://mozilla.org/MPL/2.0/",
            ],
            min_hits: 2,
        },
    ]
}

#[cfg(all(test, feature = "content", feature = "walk"))]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_metadata_license() {
        let dir = tempdir().unwrap();
        let cargo = dir.path().join("Cargo.toml");
        fs::write(
            &cargo,
            r#"[package]
name = "demo"
license = "MIT"
"#,
        )
        .unwrap();

        let files = vec![PathBuf::from("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
        assert!(report
            .findings
            .iter()
            .any(|f| f.spdx == "MIT" && f.source_kind == LicenseSourceKind::Metadata));
    }

    #[test]
    fn detects_text_license() {
        let dir = tempdir().unwrap();
        let license = dir.path().join("LICENSE");
        fs::write(
            &license,
            "Permission is hereby granted, free of charge, to any person obtaining a copy of this software. The software is provided \"as is\".",
        )
        .unwrap();

        let files = vec![PathBuf::from("LICENSE")];
        let report = build_license_report(dir.path(), &files, &AnalysisLimits::default()).unwrap();
        assert!(report
            .findings
            .iter()
            .any(|f| f.spdx == "MIT" && f.source_kind == LicenseSourceKind::Text));
    }
}
