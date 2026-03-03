//! Schema-documentation synchronization tests for tokmd-types constants.
//!
//! Reads CLAUDE.md and docs/SCHEMA.md and verifies that the schema version
//! numbers documented there match the constants exported by this crate.
//! This prevents drift between code and documentation.

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Parse a markdown table row containing `constant_name` and return the version
/// number from the adjacent column. Handles both CLAUDE.md style
/// (`` `CONST = N` ``) and SCHEMA.md style (``| N | `CONST` |``).
fn extract_version_for(content: &str, constant_name: &str) -> Option<u32> {
    for line in content.lines() {
        if !line.contains(constant_name) {
            continue;
        }
        // CLAUDE.md style: `SCHEMA_VERSION = 2`
        let eq_pattern = format!("{constant_name} = ");
        if let Some(pos) = line.find(&eq_pattern) {
            let rest = &line[pos + eq_pattern.len()..];
            let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(v) = num.parse::<u32>() {
                return Some(v);
            }
        }
        // SCHEMA.md table style: | 2 | `SCHEMA_VERSION` |
        if line.contains('|') {
            let cols: Vec<&str> = line.split('|').map(str::trim).collect();
            for (i, col) in cols.iter().enumerate() {
                if col.contains(constant_name) {
                    // Version is typically in the column before or after
                    for &offset in &[1i32, -1] {
                        let idx = i as i32 + offset;
                        if idx >= 0 && (idx as usize) < cols.len() {
                            let candidate = cols[idx as usize].trim().replace('`', "");
                            if let Ok(v) = candidate.parse::<u32>() {
                                return Some(v);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// CLAUDE.md synchronization
// ---------------------------------------------------------------------------

#[test]
fn claude_md_schema_version_matches_code() {
    let claude_md = std::fs::read_to_string(workspace_root().join("CLAUDE.md"))
        .expect("CLAUDE.md should exist at workspace root");

    let cases: &[(&str, u32)] = &[
        ("SCHEMA_VERSION", tokmd_types::SCHEMA_VERSION),
        (
            "COCKPIT_SCHEMA_VERSION",
            tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION,
        ),
        (
            "HANDOFF_SCHEMA_VERSION",
            tokmd_types::HANDOFF_SCHEMA_VERSION,
        ),
        (
            "CONTEXT_SCHEMA_VERSION",
            tokmd_types::CONTEXT_SCHEMA_VERSION,
        ),
        (
            "CONTEXT_BUNDLE_SCHEMA_VERSION",
            tokmd_types::CONTEXT_BUNDLE_SCHEMA_VERSION,
        ),
    ];

    for (name, expected) in cases {
        let doc_val = extract_version_for(&claude_md, name)
            .unwrap_or_else(|| panic!("CLAUDE.md should mention {name}"));
        assert_eq!(
            doc_val, *expected,
            "CLAUDE.md says {name} = {doc_val}, but code says {expected}"
        );
    }
}

// ---------------------------------------------------------------------------
// docs/SCHEMA.md synchronization
// ---------------------------------------------------------------------------

#[test]
fn schema_md_version_table_matches_code() {
    let schema_md = std::fs::read_to_string(workspace_root().join("docs").join("SCHEMA.md"))
        .expect("docs/SCHEMA.md should exist");

    let cases: &[(&str, u32)] = &[
        ("SCHEMA_VERSION", tokmd_types::SCHEMA_VERSION),
        (
            "COCKPIT_SCHEMA_VERSION",
            tokmd_types::cockpit::COCKPIT_SCHEMA_VERSION,
        ),
        (
            "HANDOFF_SCHEMA_VERSION",
            tokmd_types::HANDOFF_SCHEMA_VERSION,
        ),
        (
            "CONTEXT_SCHEMA_VERSION",
            tokmd_types::CONTEXT_SCHEMA_VERSION,
        ),
        (
            "CONTEXT_BUNDLE_SCHEMA_VERSION",
            tokmd_types::CONTEXT_BUNDLE_SCHEMA_VERSION,
        ),
    ];

    for (name, expected) in cases {
        let doc_val = extract_version_for(&schema_md, name)
            .unwrap_or_else(|| panic!("docs/SCHEMA.md should mention {name}"));
        assert_eq!(
            doc_val, *expected,
            "docs/SCHEMA.md says {name} = {doc_val}, but code says {expected}"
        );
    }
}
