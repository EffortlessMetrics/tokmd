//! W72 – Documentation & schema alignment tests (xtask side).
//!
//! These "meta-tests" verify that documentation files stay in sync with the
//! code: schema versions, CLI command tables, changelog references, and
//! docs/schema.json structure.

use std::collections::BTreeSet;
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

fn wasm_capability_matrix() -> serde_json::Value {
    let raw = std::fs::read_to_string(workspace_root().join("docs/capabilities/wasm.json"))
        .expect("docs/capabilities/wasm.json must exist");
    serde_json::from_str(&raw).expect("docs/capabilities/wasm.json must be valid JSON")
}

fn web_runner_messages_js() -> String {
    std::fs::read_to_string(workspace_root().join("web/runner/messages.js"))
        .expect("web/runner/messages.js must exist")
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

#[test]
fn schema_md_mentions_tool_schema_version() {
    let md = schema_md();
    assert!(
        md.contains("`TOOL_SCHEMA_VERSION`"),
        "docs/SCHEMA.md must mention TOOL_SCHEMA_VERSION"
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
    let src = read_const_u32(
        "crates/tokmd-types/src/context.rs",
        "HANDOFF_SCHEMA_VERSION",
    )
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
    let src = read_const_u32(
        "crates/tokmd-types/src/context.rs",
        "CONTEXT_SCHEMA_VERSION",
    )
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
        "crates/tokmd-types/src/context.rs",
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

#[test]
fn schema_md_baseline_version_matches_source() {
    let src = read_const_u32(
        "crates/tokmd-analysis-types/src/baseline.rs",
        "BASELINE_VERSION",
    )
    .expect("BASELINE_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`BASELINE_VERSION`")
        .expect("BASELINE_VERSION not found in SCHEMA.md");
    assert_eq!(
        src, doc,
        "BASELINE_VERSION: source={src} != SCHEMA.md={doc}"
    );
}

#[test]
fn schema_md_tool_version_matches_source() {
    let src = read_const_u32("crates/tokmd/src/tool_schema.rs", "TOOL_SCHEMA_VERSION")
        .expect("TOOL_SCHEMA_VERSION not found in source");
    let doc = schema_md_version(&schema_md(), "`TOOL_SCHEMA_VERSION`")
        .expect("TOOL_SCHEMA_VERSION not found in SCHEMA.md");
    assert_eq!(
        src, doc,
        "TOOL_SCHEMA_VERSION: source={src} != SCHEMA.md={doc}"
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

fn baseline_schema_json() -> serde_json::Value {
    let raw = std::fs::read_to_string(workspace_root().join("docs/baseline.schema.json"))
        .expect("docs/baseline.schema.json must exist");
    serde_json::from_str(&raw).expect("docs/baseline.schema.json must be valid JSON")
}

#[test]
fn baseline_schema_json_version_matches_source() {
    let json = baseline_schema_json();
    let src = read_const_u32(
        "crates/tokmd-analysis-types/src/baseline.rs",
        "BASELINE_VERSION",
    )
    .expect("BASELINE_VERSION not found in source");
    let json_ver = json["properties"]["baseline_version"]["const"]
        .as_u64()
        .expect("baseline_version const must be a number");

    assert_eq!(
        json_ver as u32, src,
        "baseline.schema.json baseline_version ({json_ver}) != BASELINE_VERSION ({src})"
    );
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
// 4. Browser/WASM capability matrix structure
// ===========================================================================

#[test]
fn wasm_capability_matrix_declares_required_commands_and_fields() {
    let matrix = wasm_capability_matrix();
    assert_eq!(
        matrix["version"].as_u64(),
        Some(1),
        "WASM capability matrix version should be 1"
    );

    let commands = matrix["commands"]
        .as_object()
        .expect("docs/capabilities/wasm.json commands must be an object");
    let required_commands = readme_command_names(&readme_md());
    let required_fields = [
        "browser_safe",
        "rootless_safe",
        "native_only",
        "requires_filesystem",
        "requires_git_history",
        "requires_host_clock",
        "requires_validated_root",
    ];

    for command in &required_commands {
        let entry = commands
            .get(command.as_str())
            .unwrap_or_else(|| panic!("WASM capability matrix missing command {command}"));
        let object = entry
            .as_object()
            .unwrap_or_else(|| panic!("WASM capability entry {command} must be an object"));
        for field in required_fields {
            assert!(
                object.contains_key(field),
                "WASM capability entry {command} missing field {field}"
            );
        }
    }
}

#[test]
fn wasm_capability_matrix_matches_readme_command_surface() {
    let matrix = wasm_capability_matrix();
    let commands = matrix["commands"]
        .as_object()
        .expect("docs/capabilities/wasm.json commands must be an object");
    let documented: BTreeSet<String> = readme_command_names(&readme_md()).into_iter().collect();
    let matrix_commands: BTreeSet<String> = commands.keys().cloned().collect();

    assert!(
        !documented.is_empty(),
        "README.md command table must define the command surface"
    );

    for command in &documented {
        assert!(
            matrix_commands.contains(command),
            "WASM capability matrix missing README command {command}"
        );
    }

    for command in &matrix_commands {
        assert!(
            documented.contains(command),
            "WASM capability matrix lists undocumented command {command}"
        );
    }
}

#[test]
fn wasm_capability_matrix_uses_allowed_values() {
    let matrix = wasm_capability_matrix();
    let commands = matrix["commands"]
        .as_object()
        .expect("docs/capabilities/wasm.json commands must be an object");
    let capability_fields = [
        "browser_safe",
        "rootless_safe",
        "native_only",
        "requires_filesystem",
        "requires_git_history",
        "requires_host_clock",
        "requires_validated_root",
    ];

    for (command, entry) in commands {
        let object = entry
            .as_object()
            .unwrap_or_else(|| panic!("WASM capability entry {command} must be an object"));
        for field in capability_fields {
            let value = object
                .get(field)
                .unwrap_or_else(|| panic!("WASM capability entry {command} missing {field}"));
            let allowed =
                value.is_boolean() || matches!(value.as_str(), Some("partial" | "native_only"));
            assert!(
                allowed,
                "WASM capability entry {command}.{field} must be true, false, partial, or native_only; got {value}"
            );
        }
    }
}

fn browser_runner_supported_modes() -> BTreeSet<String> {
    let js = web_runner_messages_js();
    let mut modes = BTreeSet::new();
    let mut in_supported_modes = false;

    for line in js.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("export const SUPPORTED_MODES") {
            in_supported_modes = true;
            continue;
        }

        if !in_supported_modes {
            continue;
        }

        if trimmed.starts_with("]);") {
            break;
        }

        let candidate = trimmed.trim_end_matches(',').trim();
        if let Some(mode) = candidate
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        {
            modes.insert(mode.to_string());
        }
    }

    assert!(
        !modes.is_empty(),
        "web/runner/messages.js SUPPORTED_MODES must not be empty"
    );
    modes
}

#[test]
fn wasm_capability_matrix_browser_safe_matches_current_runner_modes() {
    let matrix = wasm_capability_matrix();
    let commands = matrix["commands"]
        .as_object()
        .expect("docs/capabilities/wasm.json commands must be an object");
    let supported_modes = browser_runner_supported_modes();

    for mode in &supported_modes {
        assert!(
            commands.contains_key(mode),
            "WASM capability matrix missing browser runner mode {mode}"
        );
    }

    for (command, entry) in commands {
        let object = entry
            .as_object()
            .unwrap_or_else(|| panic!("WASM capability entry {command} must be an object"));
        let browser_safe = object
            .get("browser_safe")
            .unwrap_or_else(|| panic!("WASM capability entry {command} missing browser_safe"));
        let native_only = object
            .get("native_only")
            .unwrap_or_else(|| panic!("WASM capability entry {command} missing native_only"));

        if supported_modes.contains(command) {
            let runnable =
                browser_safe.as_bool() == Some(true) || browser_safe.as_str() == Some("partial");
            assert!(
                runnable,
                "browser runner mode {command} must be marked browser-safe or partial"
            );
            assert_eq!(
                native_only.as_bool(),
                Some(false),
                "browser runner mode {command} must not be native_only"
            );
        } else {
            assert_eq!(
                browser_safe.as_bool(),
                Some(false),
                "non-runner command {command} must not claim browser safety"
            );
            assert_eq!(
                native_only.as_bool(),
                Some(true),
                "non-runner command {command} must be marked native_only"
            );
        }
    }

    for mode in ["lang", "module", "export"] {
        assert_eq!(
            commands[mode]["browser_safe"].as_bool(),
            Some(true),
            "{mode} should be fully browser-safe in the current runner"
        );
    }
    assert_eq!(
        commands["analyze"]["browser_safe"].as_str(),
        Some("partial"),
        "analyze is browser-safe only for supported runner presets"
    );
}

/// Parse a `const NAME: &[&str] = &["a", "b"];` string-slice literal from a Rust
/// source file, matching the source-parse approach the other meta-tests use so
/// this check needs no cross-crate dependency on the wasm binding.
///
/// The declaration may span multiple lines; collection starts at the `= &[`
/// opener and stops at the closing `];`.
fn read_str_slice_const(relative_path: &str, constant_name: &str) -> Option<Vec<String>> {
    let path = workspace_root().join(relative_path);
    let content = std::fs::read_to_string(&path).ok()?;
    let opener = format!("const {constant_name}: &[&str] = &[");

    let start = content.find(&opener)? + opener.len();
    let rest = content.get(start..)?;
    let end = rest.find("];")?;
    let body = rest.get(..end)?;

    let mut values = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            continue;
        }
        let mut token = String::new();
        for inner in chars.by_ref() {
            if inner == '"' {
                break;
            }
            token.push(inner);
        }
        values.push(token);
    }
    Some(values)
}

/// Resolve a JSON pointer in the capability matrix to a `Vec<String>`, failing
/// with a descriptive error instead of panicking so the meta-tests stay free of
/// panic-family calls (`expect` / unchecked indexing).
fn matrix_string_array(matrix: &serde_json::Value, pointer: &str) -> Result<Vec<String>, String> {
    matrix
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("wasm.json {pointer} must be an array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("wasm.json {pointer} entries must be strings"))
        })
        .collect()
}

/// The browser rootless analyze preset surface in `wasm.json` must mirror the
/// `ROOTLESS_ANALYZE_PRESETS` constant the wasm binding advertises through
/// `capabilities()`. Together with the wasm crate's own
/// `capabilities_reports_rootless_surface` and
/// `validate_analyze_args_accepts_rootless_receipt_and_estimate` tests, this
/// closes the chain from the `supports_rootless_in_memory_analyze_preset`
/// authority through the advertised constant to the documented capability
/// matrix, so widening or narrowing rootless presets cannot silently leave the
/// docs stale.
#[test]
fn wasm_capability_matrix_analyze_presets_match_wasm_rootless_constant() -> Result<(), String> {
    let source_presets =
        read_str_slice_const("crates/tokmd-wasm/src/lib.rs", "ROOTLESS_ANALYZE_PRESETS")
            .ok_or("ROOTLESS_ANALYZE_PRESETS not found in crates/tokmd-wasm/src/lib.rs")?;
    assert!(
        !source_presets.is_empty(),
        "ROOTLESS_ANALYZE_PRESETS must declare at least one preset"
    );

    let matrix = wasm_capability_matrix();
    let doc_presets = matrix_string_array(&matrix, "/commands/analyze/browser_analyze_presets")?;

    assert_eq!(
        doc_presets, source_presets,
        "wasm.json commands.analyze.browser_analyze_presets ({doc_presets:?}) must match \
         ROOTLESS_ANALYZE_PRESETS in crates/tokmd-wasm/src/lib.rs ({source_presets:?})"
    );
    Ok(())
}

/// The `runJsonBytes` binding row in `wasm.json` must declare the archive
/// byte-mode shape the spec fixes: browser-safe, `archive-zip` gated,
/// `zip_bytes` input, and a `supported_modes` array.
#[test]
fn wasm_capability_matrix_declares_bytes_binding_shape() -> Result<(), String> {
    let matrix = wasm_capability_matrix();
    let binding = matrix
        .pointer("/bindings/runJsonBytes")
        .and_then(serde_json::Value::as_object)
        .ok_or("wasm.json bindings.runJsonBytes must be an object")?;

    assert_eq!(
        binding
            .get("browser_safe")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "runJsonBytes must be marked browser_safe"
    );
    assert_eq!(
        binding
            .get("native_only")
            .and_then(serde_json::Value::as_bool),
        Some(false),
        "runJsonBytes must not be native_only"
    );
    assert_eq!(
        binding
            .get("feature_gate")
            .and_then(serde_json::Value::as_str),
        Some("archive-zip"),
        "runJsonBytes must record the archive-zip feature gate"
    );
    assert_eq!(
        binding
            .get("input_format")
            .and_then(serde_json::Value::as_str),
        Some("zip_bytes"),
        "runJsonBytes must record the zip_bytes input format"
    );
    assert!(
        binding
            .get("supported_modes")
            .and_then(serde_json::Value::as_array)
            .is_some(),
        "runJsonBytes must declare a supported_modes array"
    );
    Ok(())
}

/// The byte-mode `supported_modes` must equal the browser-safe scan-mode command
/// surface (`native_only == false` commands). This binds the archive byte
/// binding to the same set of input-consuming scan modes the matrix advertises
/// as non-native, with no external hardcoding: adding a browser command or
/// flipping `analyze` to native would fail this check until `supported_modes`
/// is reconciled.
#[test]
fn wasm_capability_matrix_bytes_binding_modes_match_browser_scan_modes() -> Result<(), String> {
    let matrix = wasm_capability_matrix();

    let supported_modes: BTreeSet<String> =
        matrix_string_array(&matrix, "/bindings/runJsonBytes/supported_modes")?
            .into_iter()
            .collect();

    let commands = matrix
        .pointer("/commands")
        .and_then(serde_json::Value::as_object)
        .ok_or("docs/capabilities/wasm.json commands must be an object")?;
    let browser_scan_modes: BTreeSet<String> = commands
        .iter()
        .filter(|(_, entry)| {
            entry
                .get("native_only")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
        })
        .map(|(command, _)| command.clone())
        .collect();

    assert_eq!(
        supported_modes, browser_scan_modes,
        "runJsonBytes supported_modes ({supported_modes:?}) must equal the non-native_only \
         command surface ({browser_scan_modes:?})"
    );
    Ok(())
}

// ===========================================================================
// 5. Every CLI command in README.md Commands table actually exists
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
// 6. docs/reference-cli.md consistency with subcommands
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
// 7. CHANGELOG.md mentions the latest workspace version
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
// 8. Cross-doc consistency
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

#[test]
fn gate_user_docs_use_current_input_shape() {
    let start_here = std::fs::read_to_string(workspace_root().join("docs/start-here.md"))
        .expect("docs/start-here.md must exist");
    let troubleshooting = std::fs::read_to_string(workspace_root().join("docs/troubleshooting.md"))
        .expect("docs/troubleshooting.md must exist");

    assert!(
        start_here.contains("tokmd gate .runs/current/receipt.json --policy tokmd-gate.toml"),
        "start-here should show gate INPUT as a positional receipt path"
    );
    assert!(
        !start_here.contains("tokmd gate --receipt"),
        "start-here should not mention retired gate --receipt flag"
    );
    assert!(
        troubleshooting.contains("tokmd gate . --policy .tokmd-gates.toml --format json"),
        "troubleshooting should validate policies with current gate flags"
    );
    assert!(
        !troubleshooting.contains("--validate"),
        "troubleshooting should not mention nonexistent gate --validate flag"
    );
    assert!(
        !troubleshooting.contains("tokmd gate --policy .tokmd-gates.toml -v"),
        "troubleshooting should not put global -v after the gate subcommand"
    );
}
