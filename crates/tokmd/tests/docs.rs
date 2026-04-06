use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use std::path::PathBuf;

/// "Docs as tests" - verify that the commands we recommend in README/Recipes actually work.
/// These run against `tests/data` to ensure stability.
fn tokmd() -> Command {
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    cmd.current_dir(&fixtures);
    cmd
}

#[test]
fn recipe_badge_generation() {
    let tmp = tempfile::tempdir().unwrap();

    // "tokmd badge --metric lines --output badge-lines.svg"
    let badge_lines_path = tmp.path().join("badge-lines.svg");
    tokmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .arg("--output")
        .arg(&badge_lines_path)
        .assert()
        .success();
    assert!(badge_lines_path.exists());

    // "tokmd badge --metric hotspot --preset risk --output badge-hotspot.svg"
    let badge_hotspot_path = tmp.path().join("badge-hotspot.svg");
    tokmd()
        .arg("badge")
        .arg("--metric")
        .arg("hotspot")
        .arg("--preset")
        .arg("risk")
        .arg("--output")
        .arg(&badge_hotspot_path)
        .assert()
        .success();
    assert!(badge_hotspot_path.exists());
}

#[test]
fn recipe_analyze_presets() {
    // "tokmd analyze --preset receipt --format md"
    tokmd()
        .arg("analyze")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("md")
        .assert()
        .success();

    // "tokmd analyze --preset risk --format md"
    tokmd()
        .arg("analyze")
        .arg("--preset")
        .arg("risk")
        .arg("--format")
        .arg("md")
        .assert()
        .success();

    // "tokmd analyze --preset estimate --effort-layer headline --format md"
    tokmd()
        .arg("analyze")
        .arg("--preset")
        .arg("estimate")
        .arg("--effort-layer")
        .arg("headline")
        .arg("--format")
        .arg("md")
        .assert()
        .success();
}

#[test]
fn recipe_context_list() {
    // "tokmd context --budget 128k"
    tokmd()
        .arg("context")
        .arg("--budget")
        .arg("128k")
        .assert()
        .success();
}

#[test]
fn recipe_context_bundle() {
    // "tokmd context --budget 128k --mode bundle --output context.txt"
    let tmp = tempfile::tempdir().unwrap();
    let bundle_path = tmp.path().join("context.txt");
    tokmd()
        .arg("context")
        .arg("--budget")
        .arg("128k")
        .arg("--mode")
        .arg("bundle")
        .arg("--output")
        .arg(&bundle_path)
        .assert()
        .success();
    assert!(bundle_path.exists());
}

#[test]
fn recipe_context_spread() {
    // "tokmd context --budget 200k --strategy spread"
    tokmd()
        .arg("context")
        .arg("--budget")
        .arg("200k")
        .arg("--strategy")
        .arg("spread")
        .assert()
        .success();
}

#[test]
fn recipe_context_bundle_compress() {
    // "tokmd context --budget 100k --mode bundle --compress --output bundle.txt"
    let tmp = tempfile::tempdir().unwrap();
    let bundle_path = tmp.path().join("bundle.txt");
    tokmd()
        .arg("context")
        .arg("--budget")
        .arg("100k")
        .arg("--mode")
        .arg("bundle")
        .arg("--compress")
        .arg("--output")
        .arg(&bundle_path)
        .assert()
        .success();
    assert!(bundle_path.exists());
}

#[test]
fn recipe_context_json() {
    // "tokmd context --budget 128k --mode json --output selection.json"
    let tmp = tempfile::tempdir().unwrap();
    let json_path = tmp.path().join("selection.json");
    tokmd()
        .arg("context")
        .arg("--budget")
        .arg("128k")
        .arg("--mode")
        .arg("json")
        .arg("--output")
        .arg(&json_path)
        .assert()
        .success();
    assert!(json_path.exists());
}

#[test]
fn recipe_gate_default() {
    // "tokmd gate" (Requires a policy to succeed in tests, simulating inline config)
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(&policy_path, r#"
[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 5000000
"#).unwrap();

    tokmd()
        .arg("gate")
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn recipe_gate_json() {
    // "tokmd gate --format json"
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(&policy_path, r#"
[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 5000000
"#).unwrap();

    tokmd()
        .arg("gate")
        .arg("--format")
        .arg("json")
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn recipe_gate_fail_fast() {
    // "tokmd gate --fail-fast"
    let tmp = tempfile::tempdir().unwrap();
    let policy_path = tmp.path().join("policy.toml");
    std::fs::write(&policy_path, r#"
[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 5000000
"#).unwrap();

    tokmd()
        .arg("gate")
        .arg("--fail-fast")
        .arg("--policy")
        .arg(&policy_path)
        .assert()
        .success();
}

#[test]
fn recipe_tools_export_schemas() {
    // "tokmd tools --format openai --pretty"
    tokmd()
        .arg("tools")
        .arg("--format")
        .arg("openai")
        .arg("--pretty")
        .assert()
        .success();

    // "tokmd tools --format anthropic --pretty"
    tokmd()
        .arg("tools")
        .arg("--format")
        .arg("anthropic")
        .arg("--pretty")
        .assert()
        .success();

    // "tokmd tools --format jsonschema --pretty"
    tokmd()
        .arg("tools")
        .arg("--format")
        .arg("jsonschema")
        .arg("--pretty")
        .assert()
        .success();
}

#[test]
fn recipe_default_map() {
    // "tokmd module --top 20"
    tokmd()
        .arg("module")
        .arg("--top")
        .arg("20")
        .assert()
        .success();
}

#[test]
fn recipe_export_map_jsonl() {
    // "tokmd export --min-code 20 --max-rows 300 --redact paths > map.jsonl"
    tokmd()
        .arg("export")
        .arg("--min-code")
        .arg("0") // adjusted for small test data
        .arg("--max-rows")
        .arg("300")
        .arg("--redact")
        .arg("paths")
        .assert()
        .success();
}

#[test]
fn recipe_simple_lang_summary() {
    // "tokmd lang"
    tokmd().arg("lang").assert().success();
}

#[test]
fn recipe_module_summary_markdown() {
    // "tokmd module --format md"
    tokmd()
        .arg("module")
        .arg("--format")
        .arg("md")
        .assert()
        .success();
}

#[test]
fn recipe_export_full_json() {
    // "tokmd export --format json"
    tokmd()
        .arg("export")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn recipe_ci_workflow_snippet() {
    // From README: "tokmd module --format json > tokmd.module.json"
    // We don't redirect here, just check exit code.
    tokmd()
        .arg("module")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn recipe_generate_baseline() {
    // "tokmd baseline --output baseline.json"
    let tmp = tempfile::tempdir().unwrap();
    let baseline_path = tmp.path().join("baseline.json");
    tokmd()
        .arg("baseline")
        .arg("--output")
        .arg(&baseline_path)
        .assert()
        .success();
    assert!(baseline_path.exists());
}

#[test]
fn recipe_handoff_bundle() {
    // "tokmd handoff --out-dir .handoff"
    let tmp = tempfile::tempdir().unwrap();
    let handoff_dir = tmp.path().join(".handoff");
    tokmd()
        .arg("handoff")
        .arg("--out-dir")
        .arg(&handoff_dir)
        .assert()
        .success();
    assert!(handoff_dir.exists());
    assert!(handoff_dir.join("manifest.json").exists());
}

#[cfg(feature = "git")]
#[test]
fn recipe_sensor_json() {
    // Skip if the fixtures directory is not inside a git repository
    // (e.g., in the Nix build sandbox where .git is absent)
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    let in_git = std::process::Command::new("git")
        .arg("-C")
        .arg(&fixtures)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !in_git {
        eprintln!("Skipping recipe_sensor_json: not inside a git repository");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let report_path = tmp.path().join("report.json");
    tokmd()
        .env("TOKMD_GIT_BASE_REF", "HEAD")
        .arg("sensor")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&report_path)
        .assert()
        .success();
    assert!(report_path.exists());
}

#[test]
#[cfg(feature = "content")]
fn recipe_gate_with_baseline() {
    // "tokmd gate --baseline baseline.json"
    let tmp = tempfile::tempdir().unwrap();
    let baseline_path = tmp.path().join("baseline.json");

    // First generate a baseline
    tokmd()
        .arg("baseline")
        .arg("--output")
        .arg(&baseline_path)
        .assert()
        .success();

    // Then gate against it (should pass since it's the same state)
    let ratchet_path = tmp.path().join("ratchet.toml");
    std::fs::write(
        &ratchet_path,
        r#"
[[rules]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 10.0
"#,
    )
    .unwrap();

    tokmd()
        .arg("gate")
        .arg("--baseline")
        .arg(&baseline_path)
        .arg("--ratchet-config")
        .arg(&ratchet_path)
        .assert()
        .success();
}

#[test]
fn recipe_init_non_interactive() {
    let tmp = tempfile::tempdir().unwrap();
    tokmd()
        .arg("init")
        .arg("--non-interactive")
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .success();
}
