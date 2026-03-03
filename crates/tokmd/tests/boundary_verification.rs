//! Architectural boundary verification tests.
//!
//! Enforce the tiered dependency rules documented in CLAUDE.md by parsing
//! Cargo.toml files at test time. Every assertion is deterministic and
//! requires no network access.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

// ── Helpers ────────────────────────────────────────────────────────────────

/// Returns the workspace root directory.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Parse a crate's Cargo.toml and return the parsed TOML value.
fn parse_crate_toml(crate_name: &str) -> toml::Value {
    let path = workspace_root()
        .join("crates")
        .join(crate_name)
        .join("Cargo.toml");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    toml::from_str::<toml::Value>(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

/// True when the named dependency is marked `optional = true`.
fn is_optional_dep(deps_table: &toml::value::Table, dep_name: &str) -> bool {
    match deps_table.get(dep_name) {
        Some(toml::Value::Table(t)) => t.get("optional").and_then(|v| v.as_bool()).unwrap_or(false),
        _ => false,
    }
}

/// Returns the set of **required** (non-optional) dependency names.
fn get_required_deps(toml_val: &toml::Value) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    if let Some(table) = toml_val.get("dependencies").and_then(|v| v.as_table()) {
        for (key, val) in table {
            let optional = match val {
                toml::Value::Table(t) => {
                    t.get("optional").and_then(|v| v.as_bool()).unwrap_or(false)
                }
                _ => false,
            };
            if !optional {
                deps.insert(key.clone());
            }
        }
    }
    deps
}

/// Returns the set of **all** dependency names (including optional).
fn get_all_deps(toml_val: &toml::Value) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    if let Some(table) = toml_val.get("dependencies").and_then(|v| v.as_table()) {
        for key in table.keys() {
            deps.insert(key.clone());
        }
    }
    deps
}

/// Returns the set of feature names defined by a crate.
fn get_features(toml_val: &toml::Value) -> BTreeSet<String> {
    let mut features = BTreeSet::new();
    if let Some(table) = toml_val.get("features").and_then(|v| v.as_table()) {
        for key in table.keys() {
            features.insert(key.clone());
        }
    }
    features
}

/// Tier mapping as documented in CLAUDE.md.
///
/// Returns `None` for utility crates that are not explicitly assigned a tier;
/// those are skipped in upward-dependency checks rather than producing false
/// positives.
fn tier_for_crate(name: &str) -> Option<u8> {
    match name {
        // Tier 0: Contracts
        "tokmd-types"
        | "tokmd-analysis-types"
        | "tokmd-settings"
        | "tokmd-envelope"
        | "tokmd-substrate" => Some(0),

        // Tier 1: Core
        "tokmd-scan"
        | "tokmd-model"
        | "tokmd-redact"
        | "tokmd-context-policy"
        | "tokmd-scan-args"
        | "tokmd-sensor" => Some(1),

        // Tier 2: Adapters
        "tokmd-format" | "tokmd-walk" | "tokmd-content" | "tokmd-git" | "tokmd-badge"
        | "tokmd-progress" => Some(2),

        // Tier 3: Orchestration (all analysis-* crates, fun, gate, cockpit)
        n if n.starts_with("tokmd-analysis") => Some(3),
        "tokmd-fun" | "tokmd-gate" | "tokmd-cockpit" => Some(3),

        // Tier 4: Facade
        "tokmd-config" | "tokmd-core" | "tokmd-tool-schema" => Some(4),

        // Tier 5: Products
        "tokmd" | "tokmd-python" | "tokmd-node" => Some(5),

        _ => None,
    }
}

/// Get workspace members from root Cargo.toml (only `crates/` entries).
fn workspace_crate_members() -> Vec<String> {
    let root_toml_path = workspace_root().join("Cargo.toml");
    let content = fs::read_to_string(&root_toml_path).expect("Failed to read workspace Cargo.toml");
    let val: toml::Value = toml::from_str(&content).expect("Failed to parse workspace Cargo.toml");
    val.get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .expect("No workspace.members found")
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .filter(|m| m.starts_with("crates/"))
        .collect()
}

/// Recursively visit all `.rs` files under a directory.
fn visit_rs_files(dir: &Path, callback: &mut dyn FnMut(&Path)) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_rs_files(&path, callback);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                callback(&path);
            }
        }
    }
}

// ── Test 1: Tier 0 contracts have no required clap dependency ──────────────

const TIER0_CRATES: &[&str] = &[
    "tokmd-types",
    "tokmd-analysis-types",
    "tokmd-settings",
    "tokmd-envelope",
    "tokmd-substrate",
];

#[test]
fn tier0_contracts_have_no_required_clap() {
    for crate_name in TIER0_CRATES {
        let toml_val = parse_crate_toml(crate_name);
        let required_deps = get_required_deps(&toml_val);
        assert!(
            !required_deps.contains("clap"),
            "Tier 0 crate `{crate_name}` has `clap` as a required (non-optional) dependency. \
             Contract crates must be clap-free to allow embedding without CLI concerns.",
        );
    }
}

#[test]
fn tier0_contracts_clap_only_behind_feature_flag() {
    for crate_name in TIER0_CRATES {
        let toml_val = parse_crate_toml(crate_name);
        let all_deps = get_all_deps(&toml_val);
        if all_deps.contains("clap") {
            if let Some(deps_table) = toml_val.get("dependencies").and_then(|v| v.as_table()) {
                assert!(
                    is_optional_dep(deps_table, "clap"),
                    "Tier 0 crate `{crate_name}` lists `clap` but it is not optional. \
                     If clap is present it must be behind an optional feature flag.",
                );

                // Verify clap is NOT in default features
                let features = get_features(&toml_val);
                if features.contains("default") {
                    if let Some(defaults) = toml_val
                        .get("features")
                        .and_then(|f| f.get("default"))
                        .and_then(|d| d.as_array())
                    {
                        let default_strs: Vec<&str> =
                            defaults.iter().filter_map(|v| v.as_str()).collect();
                        assert!(
                            !default_strs.contains(&"clap"),
                            "Tier 0 crate `{crate_name}` has `clap` in default features. \
                             It must be opt-in only.",
                        );
                    }
                }
            }
        }
    }
}

// ── Test 2: No upward dependency violations ────────────────────────────────

#[test]
fn no_upward_dependency_violations() {
    let members = workspace_crate_members();
    let mut violations = Vec::new();

    for member_path in &members {
        let crate_name = member_path.strip_prefix("crates/").unwrap_or(member_path);
        let source_tier = match tier_for_crate(crate_name) {
            Some(t) => t,
            None => continue,
        };

        let toml_path = workspace_root().join(member_path).join("Cargo.toml");
        let content = match fs::read_to_string(&toml_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let toml_val: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Only check required (non-optional) dependencies.
        // Optional deps behind feature flags are an intentional design pattern.
        let required_deps = get_required_deps(&toml_val);
        for dep in &required_deps {
            if let Some(dep_tier) = tier_for_crate(dep) {
                if dep_tier > source_tier {
                    violations.push(format!(
                        "{crate_name} (tier {source_tier}) depends on {dep} (tier {dep_tier})"
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Upward dependency violations found (lower tier depending on higher tier):\n  {}",
        violations.join("\n  "),
    );
}

#[test]
fn tier0_crates_never_depend_on_scan_or_higher() {
    let higher_tier_crates: BTreeSet<&str> = [
        "tokmd-scan",
        "tokmd-model",
        "tokmd-format",
        "tokmd-walk",
        "tokmd-content",
        "tokmd-git",
        "tokmd-analysis",
        "tokmd-config",
        "tokmd-core",
    ]
    .into_iter()
    .collect();

    for crate_name in TIER0_CRATES {
        let toml_val = parse_crate_toml(crate_name);
        let deps = get_all_deps(&toml_val);
        let bad: Vec<&String> = deps
            .iter()
            .filter(|d| higher_tier_crates.contains(d.as_str()))
            .collect();
        assert!(
            bad.is_empty(),
            "Tier 0 crate `{crate_name}` depends on higher-tier crates: {bad:?}",
        );
    }
}

// ── Test 3: Feature flags are properly gated ───────────────────────────────

#[test]
fn git_feature_only_in_expected_crates() {
    let members = workspace_crate_members();
    let expected: BTreeSet<&str> = [
        "tokmd",
        "tokmd-analysis",
        "tokmd-analysis-grid",
        "tokmd-cockpit",
        "tokmd-context-git",
        "tokmd-sensor",
    ]
    .into_iter()
    .collect();

    let mut unexpected = Vec::new();
    for member_path in &members {
        let crate_name = member_path.strip_prefix("crates/").unwrap_or(member_path);
        let toml_path = workspace_root().join(member_path).join("Cargo.toml");
        let content = match fs::read_to_string(&toml_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let toml_val: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let features = get_features(&toml_val);
        if features.contains("git") && !expected.contains(crate_name) {
            unexpected.push(crate_name.to_string());
        }
    }

    assert!(
        unexpected.is_empty(),
        "Unexpected crates with `git` feature flag: {unexpected:?}\n\
         If intentional, update the expected set in this test.",
    );
}

#[test]
fn content_feature_only_in_expected_crates() {
    let members = workspace_crate_members();
    let expected: BTreeSet<&str> = ["tokmd", "tokmd-analysis", "tokmd-analysis-grid"]
        .into_iter()
        .collect();

    let mut unexpected = Vec::new();
    for member_path in &members {
        let crate_name = member_path.strip_prefix("crates/").unwrap_or(member_path);
        let toml_path = workspace_root().join(member_path).join("Cargo.toml");
        let content = match fs::read_to_string(&toml_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let toml_val: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let features = get_features(&toml_val);
        if features.contains("content") && !expected.contains(crate_name) {
            unexpected.push(crate_name.to_string());
        }
    }

    assert!(
        unexpected.is_empty(),
        "Unexpected crates with `content` feature flag: {unexpected:?}\n\
         If intentional, update the expected set in this test.",
    );
}

#[test]
fn walk_feature_only_in_expected_crates() {
    let members = workspace_crate_members();
    let expected: BTreeSet<&str> = ["tokmd", "tokmd-analysis", "tokmd-analysis-grid"]
        .into_iter()
        .collect();

    let mut unexpected = Vec::new();
    for member_path in &members {
        let crate_name = member_path.strip_prefix("crates/").unwrap_or(member_path);
        let toml_path = workspace_root().join(member_path).join("Cargo.toml");
        let content = match fs::read_to_string(&toml_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let toml_val: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let features = get_features(&toml_val);
        if features.contains("walk") && !expected.contains(crate_name) {
            unexpected.push(crate_name.to_string());
        }
    }

    assert!(
        unexpected.is_empty(),
        "Unexpected crates with `walk` feature flag: {unexpected:?}\n\
         If intentional, update the expected set in this test.",
    );
}

// ── Test 4: Schema version constants are uniquely defined ──────────────────

#[test]
fn schema_version_constants_are_unique() {
    let version_constants = [
        "SCHEMA_VERSION",
        "ANALYSIS_SCHEMA_VERSION",
        "COCKPIT_SCHEMA_VERSION",
        "HANDOFF_SCHEMA_VERSION",
        "CONTEXT_SCHEMA_VERSION",
        "CONTEXT_BUNDLE_SCHEMA_VERSION",
    ];

    let root = workspace_root();
    let crates_dir = root.join("crates");

    for constant in &version_constants {
        let pattern = format!("pub const {constant}");
        let mut found_in: Vec<String> = Vec::new();

        visit_rs_files(&crates_dir, &mut |path| {
            let content = fs::read_to_string(path).unwrap_or_default();
            if content.contains(&pattern) {
                found_in.push(
                    path.strip_prefix(&root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        });

        assert!(
            !found_in.is_empty(),
            "Schema constant `{constant}` is not defined anywhere in crates/. \
             Expected exactly one canonical definition.",
        );
        assert!(
            found_in.len() == 1,
            "Schema constant `{constant}` is defined in multiple locations:\n  {}\n\
             Each schema version family must have exactly one canonical definition.",
            found_in.join("\n  "),
        );
    }
}

#[test]
fn schema_version_constants_are_public() {
    let version_constants = [
        "SCHEMA_VERSION",
        "ANALYSIS_SCHEMA_VERSION",
        "COCKPIT_SCHEMA_VERSION",
        "HANDOFF_SCHEMA_VERSION",
        "CONTEXT_SCHEMA_VERSION",
        "CONTEXT_BUNDLE_SCHEMA_VERSION",
    ];

    let root = workspace_root();
    let crates_dir = root.join("crates");

    for constant in &version_constants {
        let def_pattern = format!("const {constant}:");
        let mut non_pub_defs: Vec<String> = Vec::new();

        visit_rs_files(&crates_dir, &mut |path| {
            // Skip test files — they may define local constants for assertions
            let path_str = path.to_string_lossy();
            if path_str.contains("tests") || path_str.contains("test") {
                return;
            }
            let content = fs::read_to_string(path).unwrap_or_default();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains(&def_pattern) && !trimmed.contains("pub") {
                    non_pub_defs.push(
                        path.strip_prefix(&root)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
            }
        });

        assert!(
            non_pub_defs.is_empty(),
            "Schema constant `{constant}` has non-public definitions in:\n  {}\n\
             All schema versions must be `pub` (or `pub(crate)`) `const`.",
            non_pub_defs.join("\n  "),
        );
    }
}
