//! W72 – Documentation & schema alignment tests (xtask side).
//!
//! These "meta-tests" verify that documentation files stay in sync with the
//! code: schema versions, CLI command tables, changelog references, and
//! docs/schema.json structure.

use std::path::PathBuf;

/// Workspace root (one level above the xtask crate).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Read a `pub const NAME: u32 = N;` value from a Rust source file.
fn read_const_u32(relative_path: &str, constant_name: &str) -> Option<u32> {
    let path = workspace_root().join(relative_path);
    let content = std::fs::read_to_string(&path).ok()?;
    let pattern = format!("pub const {constant_name}: u32 = ");
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            let after = &trimmed[pattern.len()..];
            return after.trim_end_matches(';').trim().parse().ok();
        }
    }
    None
}

/// Parse the version column for a given constant name from docs/SCHEMA.md table.
fn schema_md_version(md: &str, constant_name: &str) -> Option<u32> {
    for line in md.lines() {
        if line.contains(constant_name) {
            let cols: Vec<&str> = line.split('|').collect();
            if cols.len() >= 3 {
                return cols[2].trim().parse().ok();
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Cached doc content
// ---------------------------------------------------------------------------

fn schema_md() -> String {
    std::fs::read_to_string(workspace_root().join("docs/SCHEMA.md"))
        .expect("docs/SCHEMA.md must exist")
}

fn schema_json() -> serde_json::Value {
    let raw = std::fs::read_to_string(workspace_root().join("docs/schema.json"))
        .expect("docs/schema.json must exist");
    serde_json::from_str(&raw).expect("docs/schema.json must be valid JSON")
}

fn readme_md() -> String {
    std::fs::read_to_string(workspace_root().join("README.md")).expect("README.md must exist")
}

fn reference_cli_md() -> String {
    std::fs::read_to_string(workspace_root().join("docs/reference-cli.md"))
        .expect("docs/reference-cli.md must exist")
}

fn changelog_md() -> String {
    std::fs::read_to_string(workspace_root().join("CHANGELOG.md")).expect("CHANGELOG.md must exist")
}

fn cargo_toml() -> String {
    std::fs::read_to_string(workspace_root().join("Cargo.toml")).expect("Cargo.toml must exist")
}

// ===========================================================================
// 1. docs/SCHEMA.md mentions all current schema version constants
// ===========================================================================

#[test]
fn schema_md_mentions_schema_version() {
    let md = schema_md();
    assert!(
        md.contains("`SCHEMA_VERSION`"),
        "docs/SCHEMA.md must mention SCHEMA_VERSION"
    );
}

#[test]
fn schema_md_mentions_analysis_schema_version() {
    let md = schema_md();
    assert!(
        md.contains("`ANALYSIS_SCHEMA_VERSION`"),
        "docs/SCHEMA.md must mention ANALYSIS_SCHEMA_VERSION"
    );
}

#[test]
fn schema_md_mentions_cockpit_schema_version() {
    let md = schema_md();
    assert!(
        md.contains("`COCKPIT_SCHEMA_VERSION`"),
        "docs/SCHEMA.md must mention COCKPIT_SCHEMA_VERSION"
    );
}

#[test]
fn schema_md_mentions_handoff_schema_version() {
    let md = schema_md();
    assert!(
        md.contains("`HANDOFF_SCHEMA_VERSION`"),
        "docs/SCHEMA.md must mention HANDOFF_SCHEMA_VERSION"
    );
}

#[test]
fn schema_md_mentions_context_schema_version() {
    let md = schema_md();
    assert!(
        md.contains("`CONTEXT_SCHEMA_VERSION`"),
        "docs/SCHEMA.md must mention CONTEXT_SCHEMA_VERSION"
    );
}

#[test]
fn schema_md_mentions_context_bundle_schema_version() {
    let md = schema_md();
    assert!(
        md.contains("`CONTEXT_BUNDLE_SCHEMA_VERSION`"),
        "docs/SCHEMA.md must mention CONTEXT_BUNDLE_SCHEMA_VERSION"
    );
}

// ===========================================================================
// 2. Schema version constants in code match what docs say
// ===========================================================================

#[test]
fn schema_md_core_version_matches_source() {
    let src = read_const_u32("crates/tokmd-types/src/lib.rs", "SCHEMA_VERSION")
        .expect("SCHEMA_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`SCHEMA_VERSION`")
        .expect("SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(src, doc, "SCHEMA_VERSION: source={src} != SCHEMA.md={doc}");
}

#[test]
fn schema_md_analysis_version_matches_source() {
    let src = read_const_u32(
        "crates/tokmd-analysis-types/src/lib.rs",
        "ANALYSIS_SCHEMA_VERSION",
    )
    .expect("ANALYSIS_SCHEMA_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`ANALYSIS_SCHEMA_VERSION`")
        .expect("ANALYSIS_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        src, doc,
        "ANALYSIS_SCHEMA_VERSION: source={src} != SCHEMA.md={doc}"
    );
}

#[test]
fn schema_md_cockpit_version_matches_source() {
    let src = read_const_u32(
        "crates/tokmd-types/src/cockpit.rs",
        "COCKPIT_SCHEMA_VERSION",
    )
    .expect("COCKPIT_SCHEMA_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`COCKPIT_SCHEMA_VERSION`")
        .expect("COCKPIT_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        src, doc,
        "COCKPIT_SCHEMA_VERSION: source={src} != SCHEMA.md={doc}"
    );
}

#[test]
fn schema_md_handoff_version_matches_source() {
    let src = read_const_u32("crates/tokmd-types/src/lib.rs", "HANDOFF_SCHEMA_VERSION")
        .expect("HANDOFF_SCHEMA_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`HANDOFF_SCHEMA_VERSION`")
        .expect("HANDOFF_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        src, doc,
        "HANDOFF_SCHEMA_VERSION: source={src} != SCHEMA.md={doc}"
    );
}

#[test]
fn schema_md_context_version_matches_source() {
    let src = read_const_u32("crates/tokmd-types/src/lib.rs", "CONTEXT_SCHEMA_VERSION")
        .expect("CONTEXT_SCHEMA_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`CONTEXT_SCHEMA_VERSION`")
        .expect("CONTEXT_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        src, doc,
        "CONTEXT_SCHEMA_VERSION: source={src} != SCHEMA.md={doc}"
    );
}

#[test]
fn schema_md_context_bundle_version_matches_source() {
    let src = read_const_u32(
        "crates/tokmd-types/src/lib.rs",
        "CONTEXT_BUNDLE_SCHEMA_VERSION",
    )
    .expect("CONTEXT_BUNDLE_SCHEMA_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`CONTEXT_BUNDLE_SCHEMA_VERSION`")
        .expect("CONTEXT_BUNDLE_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        src, doc,
        "CONTEXT_BUNDLE_SCHEMA_VERSION: source={src} != SCHEMA.md={doc}"
    );
}

// ===========================================================================
// 3. schema.json structure alignment
// ===========================================================================

#[test]
fn schema_json_is_draft7() {
    let json = schema_json();
    assert_eq!(
        json["$schema"].as_str().unwrap_or(""),
        "http://json-schema.org/draft-07/schema#",
        "docs/schema.json must declare JSON Schema Draft 7"
    );
}

#[test]
fn schema_json_has_required_receipt_definitions() {
    let json = schema_json();
    let defs = json["definitions"].as_object().expect("definitions object");
    let required = [
        "LangReceipt",
        "ModuleReceipt",
        "ExportReceipt",
        "AnalysisReceipt",
        "CockpitReceipt",
    ];
    for name in &required {
        assert!(
            defs.contains_key(*name),
            "docs/schema.json missing definition for {name}"
        );
    }
}

#[test]
fn schema_json_receipt_versions_match_source() {
    let json = schema_json();
    let cases: &[(&str, &str, &str)] = &[
        (
            "LangReceipt",
            "crates/tokmd-types/src/lib.rs",
            "SCHEMA_VERSION",
        ),
        (
            "ModuleReceipt",
            "crates/tokmd-types/src/lib.rs",
            "SCHEMA_VERSION",
        ),
        (
            "ExportReceipt",
            "crates/tokmd-types/src/lib.rs",
            "SCHEMA_VERSION",
        ),
        (
            "AnalysisReceipt",
            "crates/tokmd-analysis-types/src/lib.rs",
            "ANALYSIS_SCHEMA_VERSION",
        ),
        (
            "CockpitReceipt",
            "crates/tokmd-types/src/cockpit.rs",
            "COCKPIT_SCHEMA_VERSION",
        ),
    ];
    for (def_name, file, const_name) in cases {
        let src = read_const_u32(file, const_name)
            .unwrap_or_else(|| panic!("{const_name} not found in {file}"));
        let json_ver = json["definitions"][def_name]["properties"]["schema_version"]["const"]
            .as_u64()
            .unwrap_or(0) as u32;
        assert_eq!(
            json_ver, src,
            "schema.json {def_name}.schema_version ({json_ver}) != {const_name} ({src})"
        );
    }
}

// ===========================================================================
// 4. Every CLI command in README.md Commands table actually exists
// ===========================================================================

/// Extract subcommand names from the README Commands table.
fn readme_command_names(readme: &str) -> Vec<String> {
    let mut cmds = Vec::new();
    let mut in_table = false;
    for line in readme.lines() {
        if line.contains("| Command") && line.contains("| Purpose") {
            in_table = true;
            continue;
        }
        if in_table && line.starts_with("| :") {
            continue;
        }
        if in_table && line.starts_with('|') {
            let cols: Vec<&str> = line.split('|').collect();
            if cols.len() >= 2 {
                let cmd_cell = cols[1].trim().replace('`', "");
                // e.g. "tokmd module" -> "module", "tokmd" -> "lang"
                let name = cmd_cell
                    .strip_prefix("tokmd ")
                    .unwrap_or("lang")
                    .to_string();
                cmds.push(name);
            }
        } else if in_table && !line.starts_with('|') {
            break;
        }
    }
    cmds
}

#[test]
fn readme_commands_table_matches_reference_cli() {
    let readme = readme_md();
    let ref_cli = reference_cli_md();
    let cmds = readme_command_names(&readme);
    assert!(!cmds.is_empty(), "Failed to parse commands from README.md");

    for cmd in &cmds {
        let pattern = format!("tokmd {cmd}");
        let default_pattern = "tokmd` (Default";
        let found =
            ref_cli.contains(&pattern) || (cmd == "lang" && ref_cli.contains(default_pattern));
        assert!(
            found,
            "README.md lists `tokmd {cmd}` but docs/reference-cli.md has no section for it"
        );
    }
}

// ===========================================================================
// 5. docs/reference-cli.md consistency with subcommands
// ===========================================================================

#[test]
fn reference_cli_global_args_header_exists() {
    let content = reference_cli_md();
    assert!(
        content.contains("## Global Arguments"),
        "docs/reference-cli.md must have a Global Arguments section"
    );
}

#[test]
fn reference_cli_commands_section_exists() {
    let content = reference_cli_md();
    assert!(
        content.contains("## Commands"),
        "docs/reference-cli.md must have a Commands section"
    );
}

// ===========================================================================
// 6. CHANGELOG.md mentions the latest workspace version
// ===========================================================================

fn workspace_version() -> String {
    let content = cargo_toml();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version = \"") && trimmed.ends_with('"') {
            return trimmed
                .strip_prefix("version = \"")
                .unwrap()
                .trim_end_matches('"')
                .to_string();
        }
    }
    panic!("Could not find version in workspace Cargo.toml");
}

#[test]
fn changelog_mentions_latest_version() {
    let cl = changelog_md();
    let ver = workspace_version();
    assert!(
        cl.contains(&ver),
        "CHANGELOG.md should mention the latest workspace version ({ver})"
    );
}

#[test]
fn changelog_has_unreleased_section() {
    let cl = changelog_md();
    assert!(
        cl.contains("## [Unreleased]"),
        "CHANGELOG.md should have an [Unreleased] section"
    );
}

#[test]
fn changelog_follows_keepachangelog() {
    let cl = changelog_md();
    assert!(
        cl.contains("keepachangelog.com"),
        "CHANGELOG.md should reference keepachangelog.com"
    );
}

// ===========================================================================
// 7. Cross-doc consistency
// ===========================================================================

#[test]
fn readme_and_reference_cli_list_same_subcommands() {
    let readme = readme_md();
    let readme_cmds = readme_command_names(&readme);

    let ref_cli = reference_cli_md();
    // Extract subcommand names from reference-cli.md section headers
    let mut ref_cmds: Vec<String> = Vec::new();
    for line in ref_cli.lines() {
        let trimmed = line.trim();
        // Match ### `tokmd <cmd>` patterns
        if trimmed.starts_with("### `tokmd ")
            && let Some(rest) = trimmed.strip_prefix("### `tokmd ")
        {
            let name = rest.trim_end_matches('`').split('`').next().unwrap_or("");
            if !name.is_empty() {
                ref_cmds.push(name.to_string());
            }
        }
        // Match ### `tokmd` (Default / `lang`)
        if trimmed.contains("`tokmd` (Default") {
            ref_cmds.push("lang".to_string());
        }
    }

    // Every README command should appear in reference-cli
    for cmd in &readme_cmds {
        assert!(
            ref_cmds.contains(cmd),
            "README.md lists `{cmd}` but reference-cli.md has no section for it"
        );
    }
}
