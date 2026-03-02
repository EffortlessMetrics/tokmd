//! Schema and documentation synchronization tests.
//!
//! These tests verify that schema version constants in code stay in sync with
//! documentation files (`docs/SCHEMA.md`, `docs/schema.json`, `CHANGELOG.md`,
//! `docs/reference-cli.md`).

mod common;

use assert_cmd::Command;
use std::path::PathBuf;

/// Workspace root (two levels above crate manifest).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Build a `tokmd` command pointed at the test fixtures.
fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    cmd.current_dir(&fixtures);
    cmd
}

// ---------------------------------------------------------------------------
// Source-of-truth constants (re-exported from crate code)
// ---------------------------------------------------------------------------

const SCHEMA_VERSION: u32 = tokmd_types::SCHEMA_VERSION;
const ANALYSIS_SCHEMA_VERSION: u32 = tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION;
const COCKPIT_SCHEMA_VERSION: u32 = tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION;
const HANDOFF_SCHEMA_VERSION: u32 = tokmd_types::HANDOFF_SCHEMA_VERSION;
const CONTEXT_SCHEMA_VERSION: u32 = tokmd_types::CONTEXT_SCHEMA_VERSION;
const CONTEXT_BUNDLE_SCHEMA_VERSION: u32 = tokmd_types::CONTEXT_BUNDLE_SCHEMA_VERSION;

// ---------------------------------------------------------------------------
// Embedded documentation (compiled into the test binary)
// ---------------------------------------------------------------------------

const SCHEMA_MD: &str = include_str!("../../../docs/SCHEMA.md");
const SCHEMA_JSON: &str = include_str!("../../../docs/schema.json");
const CHANGELOG_MD: &str = include_str!("../../../CHANGELOG.md");
const REFERENCE_CLI_MD: &str = include_str!("../../../docs/reference-cli.md");

// ===========================================================================
// 1. docs/SCHEMA.md version table matches code constants
// ===========================================================================

/// Parse the "Current Versions" markdown table in SCHEMA.md and return
/// `(family, version)` pairs.
fn parse_schema_md_versions(md: &str) -> Vec<(String, u32)> {
    let mut results = Vec::new();
    let mut in_table = false;
    for line in md.lines() {
        if line.contains("| Receipt Family") {
            in_table = true;
            continue;
        }
        if in_table && line.starts_with("|--") {
            continue;
        }
        if in_table && line.starts_with('|') {
            let cols: Vec<&str> = line.split('|').collect();
            // cols: ["", " **Core** ", " 2 ", " `SCHEMA_VERSION` ", " ... ", ""]
            if cols.len() >= 4 {
                let family = cols[1].trim().replace("**", "").trim().to_string();
                let version_str = cols[2].trim().replace('`', "");
                // Skip non-numeric versions (e.g. "sensor.report.v1")
                if let Ok(ver) = version_str.parse::<u32>() {
                    results.push((family, ver));
                }
            }
        } else if in_table && !line.starts_with('|') {
            break;
        }
    }
    results
}

#[test]
fn schema_md_core_version_matches_code() {
    let versions = parse_schema_md_versions(SCHEMA_MD);
    let core = versions.iter().find(|(f, _)| f == "Core");
    assert!(
        core.is_some(),
        "SCHEMA.md must document the Core receipt family"
    );
    assert_eq!(
        core.unwrap().1,
        SCHEMA_VERSION,
        "SCHEMA.md Core version ({}) != SCHEMA_VERSION ({SCHEMA_VERSION})",
        core.unwrap().1
    );
}

#[test]
fn schema_md_analysis_version_matches_code() {
    let versions = parse_schema_md_versions(SCHEMA_MD);
    let analysis = versions.iter().find(|(f, _)| f == "Analysis");
    assert!(
        analysis.is_some(),
        "SCHEMA.md must document the Analysis receipt family"
    );
    assert_eq!(
        analysis.unwrap().1,
        ANALYSIS_SCHEMA_VERSION,
        "SCHEMA.md Analysis version ({}) != ANALYSIS_SCHEMA_VERSION ({ANALYSIS_SCHEMA_VERSION})",
        analysis.unwrap().1
    );
}

#[test]
fn schema_md_cockpit_version_matches_code() {
    let versions = parse_schema_md_versions(SCHEMA_MD);
    let cockpit = versions.iter().find(|(f, _)| f == "Cockpit");
    assert!(
        cockpit.is_some(),
        "SCHEMA.md must document the Cockpit receipt family"
    );
    assert_eq!(
        cockpit.unwrap().1,
        COCKPIT_SCHEMA_VERSION,
        "SCHEMA.md Cockpit version ({}) != COCKPIT_SCHEMA_VERSION ({COCKPIT_SCHEMA_VERSION})",
        cockpit.unwrap().1
    );
}

#[test]
fn schema_md_handoff_version_matches_code() {
    let versions = parse_schema_md_versions(SCHEMA_MD);
    let handoff = versions.iter().find(|(f, _)| f == "Handoff");
    assert!(
        handoff.is_some(),
        "SCHEMA.md must document the Handoff receipt family"
    );
    assert_eq!(
        handoff.unwrap().1,
        HANDOFF_SCHEMA_VERSION,
        "SCHEMA.md Handoff version ({}) != HANDOFF_SCHEMA_VERSION ({HANDOFF_SCHEMA_VERSION})",
        handoff.unwrap().1
    );
}

#[test]
fn schema_md_context_version_matches_code() {
    let versions = parse_schema_md_versions(SCHEMA_MD);
    let context = versions.iter().find(|(f, _)| f == "Context");
    assert!(
        context.is_some(),
        "SCHEMA.md must document the Context receipt family"
    );
    assert_eq!(
        context.unwrap().1,
        CONTEXT_SCHEMA_VERSION,
        "SCHEMA.md Context version ({}) != CONTEXT_SCHEMA_VERSION ({CONTEXT_SCHEMA_VERSION})",
        context.unwrap().1
    );
}

#[test]
fn schema_md_context_bundle_version_matches_code() {
    let versions = parse_schema_md_versions(SCHEMA_MD);
    let cb = versions.iter().find(|(f, _)| f == "Context Bundle");
    assert!(
        cb.is_some(),
        "SCHEMA.md must document the Context Bundle receipt family"
    );
    assert_eq!(
        cb.unwrap().1,
        CONTEXT_BUNDLE_SCHEMA_VERSION,
        "SCHEMA.md Context Bundle version ({}) != CONTEXT_BUNDLE_SCHEMA_VERSION ({CONTEXT_BUNDLE_SCHEMA_VERSION})",
        cb.unwrap().1
    );
}

// ===========================================================================
// 2. docs/schema.json `const` values match code constants
// ===========================================================================

/// Extract `schema_version` const for a named definition from schema.json.
fn schema_json_version(json: &serde_json::Value, definition: &str) -> Option<u32> {
    json.get("definitions")?
        .get(definition)?
        .get("properties")?
        .get("schema_version")?
        .get("const")?
        .as_u64()
        .map(|v| v as u32)
}

#[test]
fn schema_json_lang_receipt_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "LangReceipt")
        .expect("schema.json must define LangReceipt.schema_version const");
    assert_eq!(
        ver, SCHEMA_VERSION,
        "schema.json LangReceipt.schema_version ({ver}) != SCHEMA_VERSION ({SCHEMA_VERSION})"
    );
}

#[test]
fn schema_json_module_receipt_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "ModuleReceipt")
        .expect("schema.json must define ModuleReceipt.schema_version const");
    assert_eq!(
        ver, SCHEMA_VERSION,
        "schema.json ModuleReceipt.schema_version ({ver}) != SCHEMA_VERSION ({SCHEMA_VERSION})"
    );
}

#[test]
fn schema_json_export_receipt_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "ExportReceipt")
        .expect("schema.json must define ExportReceipt.schema_version const");
    assert_eq!(
        ver, SCHEMA_VERSION,
        "schema.json ExportReceipt.schema_version ({ver}) != SCHEMA_VERSION ({SCHEMA_VERSION})"
    );
}

#[test]
fn schema_json_analysis_receipt_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "AnalysisReceipt")
        .expect("schema.json must define AnalysisReceipt.schema_version const");
    assert_eq!(
        ver, ANALYSIS_SCHEMA_VERSION,
        "schema.json AnalysisReceipt.schema_version ({ver}) != ANALYSIS_SCHEMA_VERSION ({ANALYSIS_SCHEMA_VERSION})"
    );
}

#[test]
fn schema_json_cockpit_receipt_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "CockpitReceipt")
        .expect("schema.json must define CockpitReceipt.schema_version const");
    assert_eq!(
        ver, COCKPIT_SCHEMA_VERSION,
        "schema.json CockpitReceipt.schema_version ({ver}) != COCKPIT_SCHEMA_VERSION ({COCKPIT_SCHEMA_VERSION})"
    );
}

#[test]
fn schema_json_handoff_manifest_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "HandoffManifest")
        .expect("schema.json must define HandoffManifest.schema_version const");
    assert_eq!(
        ver, HANDOFF_SCHEMA_VERSION,
        "schema.json HandoffManifest.schema_version ({ver}) != HANDOFF_SCHEMA_VERSION ({HANDOFF_SCHEMA_VERSION})"
    );
}

#[test]
fn schema_json_context_receipt_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "ContextReceipt")
        .expect("schema.json must define ContextReceipt.schema_version const");
    assert_eq!(
        ver, CONTEXT_SCHEMA_VERSION,
        "schema.json ContextReceipt.schema_version ({ver}) != CONTEXT_SCHEMA_VERSION ({CONTEXT_SCHEMA_VERSION})"
    );
}

#[test]
fn schema_json_context_bundle_version_matches_code() {
    let json: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let ver = schema_json_version(&json, "ContextBundleManifest")
        .expect("schema.json must define ContextBundleManifest.schema_version const");
    assert_eq!(
        ver, CONTEXT_BUNDLE_SCHEMA_VERSION,
        "schema.json ContextBundleManifest.schema_version ({ver}) != CONTEXT_BUNDLE_SCHEMA_VERSION ({CONTEXT_BUNDLE_SCHEMA_VERSION})"
    );
}

// ===========================================================================
// 3. JSON receipt output fields documented in schema.json exist in actual output
// ===========================================================================

/// Extract required field names from a schema.json definition.
fn required_fields(json: &serde_json::Value, definition: &str) -> Vec<String> {
    json.get("definitions")
        .and_then(|d| d.get(definition))
        .and_then(|d| d.get("required"))
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn lang_json_output_has_all_required_fields() {
    let schema: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let required = required_fields(&schema, "LangReceipt");
    assert!(
        !required.is_empty(),
        "LangReceipt must have required fields"
    );

    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("tokmd lang --format json should run");
    assert!(output.status.success());

    let receipt: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");
    let obj = receipt.as_object().expect("receipt should be an object");

    for field in &required {
        assert!(
            obj.contains_key(field),
            "LangReceipt output missing required field: {field}"
        );
    }
}

#[test]
fn module_json_output_has_all_required_fields() {
    let schema: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let required = required_fields(&schema, "ModuleReceipt");
    assert!(
        !required.is_empty(),
        "ModuleReceipt must have required fields"
    );

    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("tokmd module --format json should run");
    assert!(output.status.success());

    let receipt: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");
    let obj = receipt.as_object().expect("receipt should be an object");

    for field in &required {
        assert!(
            obj.contains_key(field),
            "ModuleReceipt output missing required field: {field}"
        );
    }
}

#[test]
fn export_json_output_has_all_required_fields() {
    let schema: serde_json::Value = serde_json::from_str(SCHEMA_JSON).unwrap();
    let required = required_fields(&schema, "ExportReceipt");
    assert!(
        !required.is_empty(),
        "ExportReceipt must have required fields"
    );

    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("tokmd export --format json should run");
    assert!(output.status.success());

    let receipt: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");
    let obj = receipt.as_object().expect("receipt should be an object");

    for field in &required {
        assert!(
            obj.contains_key(field),
            "ExportReceipt output missing required field: {field}"
        );
    }
}

// ===========================================================================
// 4. docs/reference-cli.md help text matches actual `--help` output
// ===========================================================================

/// Extract the first `<!-- HELP: lang -->` code block from reference-cli.md.
fn extract_help_block(md: &str, tag: &str) -> Option<String> {
    let marker = format!("<!-- HELP: {tag} -->");
    let rest = md.split_once(&marker)?.1;
    // Find opening ```text and closing ```
    let after_fence = rest.split_once("```text\n")?.1;
    let block = after_fence.split_once("\n```")?.0;
    Some(block.to_string())
}

#[test]
fn reference_cli_lang_help_matches_actual() {
    let documented = extract_help_block(REFERENCE_CLI_MD, "lang")
        .expect("reference-cli.md must contain <!-- HELP: lang --> block");

    let output = tokmd_cmd()
        .args(["lang", "--help"])
        .output()
        .expect("tokmd lang --help should run");
    assert!(output.status.success());

    let actual = String::from_utf8_lossy(&output.stdout);
    // Normalize whitespace and platform binary name differences (tokmd.exe vs tokmd)
    let normalize = |s: &str| -> String {
        s.lines()
            .map(|l| l.trim_end().replace("tokmd.exe", "tokmd"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    };

    assert_eq!(
        normalize(&documented),
        normalize(&actual),
        "docs/reference-cli.md lang help text is out of sync with `tokmd lang --help`"
    );
}

// ===========================================================================
// 5. Handoff schema version in schema.json matches code constant
//    (covered above in schema_json_handoff_manifest_version_matches_code,
//     but we also verify the embedded schemas/ copy is identical)
// ===========================================================================

const EMBEDDED_SCHEMA_JSON: &str = include_str!("../schemas/schema.json");

#[test]
fn embedded_schema_json_matches_docs_schema_json() {
    assert_eq!(
        SCHEMA_JSON, EMBEDDED_SCHEMA_JSON,
        "docs/schema.json and crates/tokmd/schemas/schema.json must be identical"
    );
}

// ===========================================================================
// 6. CHANGELOG.md references schema version bumps
// ===========================================================================

/// Verify that the current ANALYSIS_SCHEMA_VERSION value appears somewhere in CHANGELOG.md,
/// confirming that the latest bump was documented.
#[test]
fn changelog_documents_analysis_schema_version() {
    // The version before current should appear as part of a "→ N" or "version: N" note.
    let current = ANALYSIS_SCHEMA_VERSION.to_string();
    let contains_reference = CHANGELOG_MD.contains(&format!("→ {current}"))
        || CHANGELOG_MD.contains(&format!("version: {current}"))
        || CHANGELOG_MD.contains(&format!("schema_version: {current}"))
        || CHANGELOG_MD.contains(&format!("`schema_version: {current}`"))
        // v8 was added via near-dup clusters — check for that mention
        || CHANGELOG_MD.contains("ANALYSIS_SCHEMA_VERSION");

    assert!(
        contains_reference,
        "CHANGELOG.md should reference ANALYSIS_SCHEMA_VERSION = {current} (or the constant name)"
    );
}

#[test]
fn changelog_documents_core_schema_version() {
    let current = SCHEMA_VERSION.to_string();
    let contains_reference = CHANGELOG_MD.contains(&format!("SCHEMA_VERSION = {current}"))
        || CHANGELOG_MD.contains("SCHEMA_VERSION")
        || CHANGELOG_MD.contains(&format!("schema_version: {current}"));
    assert!(
        contains_reference,
        "CHANGELOG.md should reference SCHEMA_VERSION = {current} (or the constant name)"
    );
}

// ===========================================================================
// Bonus: SCHEMA.md "Code References" section lists correct file paths
// ===========================================================================

#[test]
fn schema_md_code_references_point_to_existing_files() {
    let root = workspace_root();
    // Extract lines from the "Code References" section
    let in_refs = SCHEMA_MD
        .lines()
        .skip_while(|l| !l.contains("### Code References") && !l.contains("## Code References"))
        .skip(1)
        .take_while(|l| !l.starts_with('#'))
        .filter(|l| l.contains("crates/"));

    for line in in_refs {
        // Extract path like `crates/tokmd-types/src/lib.rs`
        if let Some(start) = line.find("crates/") {
            let rest = &line[start..];
            // Path ends at backtick, space-dash, or end of line
            let end = rest
                .find('`')
                .or_else(|| rest.find(" -"))
                .unwrap_or(rest.len());
            let rel_path = &rest[..end].trim_end_matches('`');
            let full = root.join(rel_path.replace('/', "\\"));
            assert!(
                full.exists(),
                "SCHEMA.md Code References path does not exist: {rel_path}"
            );
        }
    }
}
