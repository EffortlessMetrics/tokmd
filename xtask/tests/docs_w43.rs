//! Documentation validation tests.
//!
//! Validates that docs/*.md internal links are valid, JSON schemas parse,
//! and SCHEMA.md version references match actual source constants.

use std::path::PathBuf;

/// Find the workspace root.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

// ── Internal link validation ────────────────────────────────────────────

/// Collect all markdown files in docs/.
fn docs_md_files() -> Vec<PathBuf> {
    let docs_dir = workspace_root().join("docs");
    std::fs::read_dir(&docs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "md").unwrap_or(false))
        .collect()
}

/// Extensions for generated/binary artifacts that we don't validate.
const SKIP_EXTENSIONS: &[&str] = &["svg", "png", "jpg", "jpeg", "gif", "ico", "pdf"];

/// Extract relative link targets from markdown content (excludes http(s), anchors, and images).
fn extract_relative_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    // Match [text](target) patterns
    let mut rest = content;
    while let Some(start) = rest.find("](") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find(')') {
            let target = &after[..end];
            // Skip external URLs and pure anchors
            if !target.starts_with("http://")
                && !target.starts_with("https://")
                && !target.starts_with('#')
                && !target.is_empty()
            {
                // Strip anchor fragment for file existence check
                let file_part = target.split('#').next().unwrap_or(target);
                if !file_part.is_empty() {
                    // Skip generated artifacts (badges, images)
                    let skip = SKIP_EXTENSIONS
                        .iter()
                        .any(|ext| file_part.ends_with(&format!(".{ext}")));
                    if !skip {
                        links.push(file_part.to_string());
                    }
                }
            }
            rest = &after[end..];
        } else {
            break;
        }
    }
    links
}

#[test]
fn docs_internal_links_are_valid() {
    let docs_dir = workspace_root().join("docs");
    let mut broken = Vec::new();

    for md_file in docs_md_files() {
        let content = std::fs::read_to_string(&md_file).unwrap();
        let links = extract_relative_links(&content);
        let file_name = md_file.file_name().unwrap().to_string_lossy().to_string();

        for link in links {
            let target = docs_dir.join(&link);
            if !target.exists() {
                broken.push(format!("{file_name} -> {link}"));
            }
        }
    }

    assert!(
        broken.is_empty(),
        "Broken internal links in docs/:\n  {}",
        broken.join("\n  ")
    );
}

#[test]
fn docs_directory_has_required_files() {
    let docs_dir = workspace_root().join("docs");
    let required = [
        "SCHEMA.md",
        "schema.json",
        "architecture.md",
        "reference-cli.md",
    ];
    for name in &required {
        assert!(
            docs_dir.join(name).exists(),
            "Required docs file missing: {name}"
        );
    }
}

#[test]
fn docs_all_md_files_are_nonempty() {
    for md_file in docs_md_files() {
        let content = std::fs::read_to_string(&md_file).unwrap();
        let name = md_file.file_name().unwrap().to_string_lossy().to_string();
        assert!(
            !content.trim().is_empty(),
            "docs/{name} should not be empty"
        );
    }
}

// ── JSON schema validation ──────────────────────────────────────────────

#[test]
fn schema_json_is_valid_json() {
    let path = workspace_root().join("docs").join("schema.json");
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "docs/schema.json is not valid JSON: {:?}",
        parsed.err()
    );
}

#[test]
fn schema_json_has_schema_field() {
    let path = workspace_root().join("docs").join("schema.json");
    let content = std::fs::read_to_string(&path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        val.get("$schema").is_some() || val.get("type").is_some(),
        "schema.json should have $schema or type field"
    );
}

#[test]
fn handoff_schema_json_is_valid() {
    let path = workspace_root().join("docs").join("handoff.schema.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(
            parsed.is_ok(),
            "docs/handoff.schema.json is not valid JSON: {:?}",
            parsed.err()
        );
    }
}

#[test]
fn baseline_schema_json_is_valid() {
    let path = workspace_root().join("docs").join("baseline.schema.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(
            parsed.is_ok(),
            "docs/baseline.schema.json is not valid JSON: {:?}",
            parsed.err()
        );
    }
}

// ── SCHEMA.md version consistency ───────────────────────────────────────

/// Read a schema version constant from a Rust source file.
fn read_schema_constant(relative_path: &str, constant_name: &str) -> Option<u32> {
    let path = workspace_root().join(relative_path);
    let content = std::fs::read_to_string(&path).ok()?;
    let pattern = format!("pub const {constant_name}: u32 = ");
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            let after = &trimmed[pattern.len()..];
            let num_str = after.trim_end_matches(';').trim();
            return num_str.parse().ok();
        }
    }
    None
}

/// Parse a version number from a SCHEMA.md table row like "| **Core** | 2 | ..."
fn extract_schema_md_version(schema_md: &str, constant_name: &str) -> Option<u32> {
    for line in schema_md.lines() {
        if line.contains(constant_name) {
            // Parse "|  ... | N | ..." - find the version column (second pipe-delimited field)
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let version_str = parts[2].trim();
                return version_str.parse().ok();
            }
        }
    }
    None
}

#[test]
fn schema_md_core_version_matches_source() {
    let source_version = read_schema_constant("crates/tokmd-types/src/lib.rs", "SCHEMA_VERSION")
        .expect("SCHEMA_VERSION not found in source");
    let schema_md = std::fs::read_to_string(workspace_root().join("docs/SCHEMA.md")).unwrap();
    let doc_version = extract_schema_md_version(&schema_md, "`SCHEMA_VERSION`")
        .expect("SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        source_version, doc_version,
        "SCHEMA_VERSION mismatch: source={source_version}, SCHEMA.md={doc_version}"
    );
}

#[test]
fn schema_md_analysis_version_matches_source() {
    let source_version = read_schema_constant(
        "crates/tokmd-analysis-types/src/lib.rs",
        "ANALYSIS_SCHEMA_VERSION",
    )
    .expect("ANALYSIS_SCHEMA_VERSION not found in source");
    let schema_md = std::fs::read_to_string(workspace_root().join("docs/SCHEMA.md")).unwrap();
    let doc_version = extract_schema_md_version(&schema_md, "`ANALYSIS_SCHEMA_VERSION`")
        .expect("ANALYSIS_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        source_version, doc_version,
        "ANALYSIS_SCHEMA_VERSION mismatch: source={source_version}, SCHEMA.md={doc_version}"
    );
}

#[test]
fn schema_md_cockpit_version_matches_source() {
    let source_version = read_schema_constant(
        "crates/tokmd-types/src/cockpit.rs",
        "COCKPIT_SCHEMA_VERSION",
    )
    .expect("COCKPIT_SCHEMA_VERSION not found in source");
    let schema_md = std::fs::read_to_string(workspace_root().join("docs/SCHEMA.md")).unwrap();
    let doc_version = extract_schema_md_version(&schema_md, "`COCKPIT_SCHEMA_VERSION`")
        .expect("COCKPIT_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        source_version, doc_version,
        "COCKPIT_SCHEMA_VERSION mismatch: source={source_version}, SCHEMA.md={doc_version}"
    );
}

#[test]
fn schema_md_context_version_matches_source() {
    let source_version =
        read_schema_constant("crates/tokmd-types/src/lib.rs", "CONTEXT_SCHEMA_VERSION")
            .expect("CONTEXT_SCHEMA_VERSION not found in source");
    let schema_md = std::fs::read_to_string(workspace_root().join("docs/SCHEMA.md")).unwrap();
    let doc_version = extract_schema_md_version(&schema_md, "`CONTEXT_SCHEMA_VERSION`")
        .expect("CONTEXT_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        source_version, doc_version,
        "CONTEXT_SCHEMA_VERSION mismatch: source={source_version}, SCHEMA.md={doc_version}"
    );
}

#[test]
fn schema_md_handoff_version_matches_source() {
    let source_version =
        read_schema_constant("crates/tokmd-types/src/lib.rs", "HANDOFF_SCHEMA_VERSION")
            .expect("HANDOFF_SCHEMA_VERSION not found in source");
    let schema_md = std::fs::read_to_string(workspace_root().join("docs/SCHEMA.md")).unwrap();
    let doc_version = extract_schema_md_version(&schema_md, "`HANDOFF_SCHEMA_VERSION`")
        .expect("HANDOFF_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        source_version, doc_version,
        "HANDOFF_SCHEMA_VERSION mismatch: source={source_version}, SCHEMA.md={doc_version}"
    );
}

// ── Reference-CLI doc markers ───────────────────────────────────────────

#[test]
fn reference_cli_has_help_markers() {
    let path = workspace_root().join("docs").join("reference-cli.md");
    let content = std::fs::read_to_string(&path).unwrap();
    // At minimum, the lang command should have markers
    assert!(
        content.contains("<!-- HELP: lang -->"),
        "reference-cli.md should have HELP markers for lang command"
    );
    assert!(
        content.contains("<!-- /HELP: lang -->"),
        "reference-cli.md should have closing HELP markers"
    );
}

#[test]
fn reference_cli_markers_are_paired() {
    let path = workspace_root().join("docs").join("reference-cli.md");
    let content = std::fs::read_to_string(&path).unwrap();
    let commands = [
        "lang",
        "module",
        "export",
        "run",
        "analyze",
        "badge",
        "diff",
        "context",
        "tools",
        "cockpit",
        "gate",
        "completions",
    ];
    for cmd in &commands {
        let open = format!("<!-- HELP: {cmd} -->");
        let close = format!("<!-- /HELP: {cmd} -->");
        let has_open = content.contains(&open);
        let has_close = content.contains(&close);
        assert_eq!(
            has_open, has_close,
            "Unpaired HELP marker for {cmd}: open={has_open}, close={has_close}"
        );
    }
}
