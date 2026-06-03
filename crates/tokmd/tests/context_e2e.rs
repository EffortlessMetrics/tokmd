mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// Mode: list (default)
// ---------------------------------------------------------------------------

#[test]
fn test_context_list_mode() {
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_context_default_mode_is_list() {
    // Without --mode, default should behave like list
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_context_default_list_reconciles_head_tail_charged_tokens() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();

    let large_file = (1..=300)
        .map(|i| format!("pub fn f{i}() -> i32 {{ {i} }}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(root.join("big.rs"), large_file).unwrap();

    let json_output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .args(["context", "--mode", "json", "--budget", "1000"])
        .args(["--max-file-tokens", "50"])
        .output()
        .unwrap();
    assert!(json_output.status.success());

    let parsed: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let files = parsed["files"].as_array().unwrap();
    assert_eq!(files.len(), 1, "fixture should select exactly one file");
    let file = &files[0];
    let full_tokens = file["tokens"].as_u64().unwrap();
    let effective_tokens = file["effective_tokens"].as_u64().unwrap();
    let reason = file["policy_reason"].as_str().unwrap();

    assert!(
        full_tokens > effective_tokens,
        "fixture should trigger head+tail truncation"
    );
    assert_eq!(parsed["used_tokens"].as_u64(), Some(effective_tokens));
    assert_eq!(file["policy"].as_str(), Some("head_tail"));

    let list_output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .args(["context", "--budget", "1000"])
        .args(["--max-file-tokens", "50"])
        .output()
        .unwrap();
    assert!(list_output.status.success());

    let list = String::from_utf8_lossy(&list_output.stdout);
    assert!(list.contains("|Path|Module|Lang|Used|Tokens|Policy|Code|"));
    assert!(list.contains(&format!("Used: {effective_tokens} tokens")));
    assert!(list.contains(&format!("|{effective_tokens}|{full_tokens}|")));
    assert!(list.contains(reason));
}

#[test]
fn test_context_bun_ub_handoff_recipe_scopes_paths_and_reports_policy() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join(".git")).unwrap();

    let runtime_api = root.join("src").join("runtime").join("api");
    let bindings = root.join("src").join("bun.js").join("bindings");
    let bun_api = root.join("src").join("bun.js").join("api");
    let unrelated = root
        .join("test")
        .join("cli")
        .join("install")
        .join("fixtures");
    fs::create_dir_all(&runtime_api).unwrap();
    fs::create_dir_all(&bindings).unwrap();
    fs::create_dir_all(&bun_api).unwrap();
    fs::create_dir_all(&unrelated).unwrap();

    let runtime_source = (1..=200)
        .map(|i| {
            format!(
                "pub unsafe extern \"C\" fn markdown_object_{i}(ptr: *mut std::ffi::c_void) -> bool {{ !ptr.is_null() }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(runtime_api.join("MarkdownObject.rs"), runtime_source).unwrap();
    fs::write(
        bindings.join("native.rs"),
        "use std::ffi::c_void;\npub unsafe extern \"C\" fn bun_binding(ptr: *mut c_void) { let _ = ptr; }\n",
    )
    .unwrap();
    fs::write(
        bun_api.join("api.rs"),
        "pub fn native_boundary_name() -> &'static str { \"bun-api\" }\n",
    )
    .unwrap();
    let generated_nodes = format!(
        "{{\"nodes\":[{}]}}",
        (1..=300)
            .map(|i| format!("{{\"type\":\"GeneratedNode{i}\",\"fields\":[\"a\",\"b\",\"c\"]}}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    fs::write(bun_api.join("node-types.json"), generated_nodes).unwrap();
    fs::write(
        unrelated.join("outside.rs"),
        "pub fn outside_requested_scope() {}\n",
    )
    .unwrap();

    let scope_args = ["src/runtime/api", "src/bun.js/bindings", "src/bun.js/api"];

    let list_output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .args(["context", "--budget", "64000"])
        .args(scope_args)
        .output()
        .unwrap();
    assert!(
        list_output.status.success(),
        "bun-ub context list should succeed: {:?}\nstderr: {}",
        list_output.status,
        String::from_utf8_lossy(&list_output.stderr)
    );

    let list = String::from_utf8_lossy(&list_output.stdout);
    assert!(list.contains("|Path|Module|Lang|Used|Tokens|Policy|Code|"));
    assert!(list.contains("src/runtime/api/MarkdownObject.rs"));
    assert!(list.contains("src/bun.js/bindings/native.rs"));
    assert!(list.contains("src/bun.js/api/api.rs"));
    assert!(
        !list.contains("test/cli/install"),
        "context list should stay inside requested Bun UB handoff paths:\n{list}"
    );

    let capped_json_output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .args(["context", "--mode", "json", "--budget", "64000"])
        .args(["--max-file-tokens", "80"])
        .args(scope_args)
        .output()
        .unwrap();
    assert!(
        capped_json_output.status.success(),
        "bun-ub capped context JSON should succeed: {:?}\nstderr: {}",
        capped_json_output.status,
        String::from_utf8_lossy(&capped_json_output.stderr)
    );

    let parsed: serde_json::Value = serde_json::from_slice(&capped_json_output.stdout).unwrap();
    assert_eq!(parsed["budget_tokens"].as_u64(), Some(64_000));
    assert!(
        parsed["used_tokens"].as_u64().unwrap_or(64_001) <= 64_000,
        "used tokens should fit budget: {parsed}"
    );

    let files = parsed["files"].as_array().unwrap();
    let runtime_row = files
        .iter()
        .find(|file| file["path"].as_str() == Some("src/runtime/api/MarkdownObject.rs"))
        .expect("runtime API row should be selected");
    let effective_tokens = runtime_row["effective_tokens"].as_u64().unwrap();
    let full_tokens = runtime_row["tokens"].as_u64().unwrap();
    let policy_reason = runtime_row["policy_reason"].as_str().unwrap();
    assert!(
        full_tokens > effective_tokens,
        "runtime row should be head+tail truncated: {runtime_row}"
    );
    assert!(policy_reason.contains("head+tail"));

    let excluded = parsed["excluded_by_policy"].as_array().unwrap();
    let generated_skip = excluded
        .iter()
        .find(|file| file["path"].as_str() == Some("src/bun.js/api/node-types.json"))
        .expect("generated node types should be visible as policy-skipped");
    assert_eq!(generated_skip["policy"].as_str(), Some("skip"));
    assert!(
        generated_skip["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("generated"),
        "skip reason should explain generated policy: {generated_skip}"
    );

    let capped_list_output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(root)
        .args(["context", "--budget", "64000"])
        .args(["--max-file-tokens", "80"])
        .args(scope_args)
        .output()
        .unwrap();
    assert!(
        capped_list_output.status.success(),
        "bun-ub capped context list should succeed: {:?}\nstderr: {}",
        capped_list_output.status,
        String::from_utf8_lossy(&capped_list_output.stderr)
    );

    let capped_list = String::from_utf8_lossy(&capped_list_output.stdout);
    assert!(capped_list.contains(&format!("|{effective_tokens}|{full_tokens}|")));
    assert!(capped_list.contains("head+tail"));
    assert!(
        !capped_list.contains("node-types.json"),
        "policy-skipped generated files belong in JSON excluded_by_policy, not selected rows:\n{capped_list}"
    );
}

// ---------------------------------------------------------------------------
// Mode: json
// ---------------------------------------------------------------------------

#[test]
fn test_context_json_mode() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .expect("failed to run tokmd context --mode json");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("context JSON output should be valid JSON");

    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
    assert_eq!(parsed["mode"].as_str(), Some("context"));
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["utilization_pct"].is_number());
    assert!(parsed["file_count"].is_number());
    assert!(parsed["files"].is_array());
    assert!(parsed["tool"]["name"].as_str() == Some("tokmd"));
    assert!(parsed["generated_at_ms"].is_number());

    let files = parsed["files"].as_array().unwrap();
    assert!(!files.is_empty(), "should include at least one file");
}

// ---------------------------------------------------------------------------
// Budget limiting
// ---------------------------------------------------------------------------

#[test]
fn test_context_budget_limiting() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--budget")
        .arg("1000")
        .output()
        .expect("failed to run tokmd context --budget 1000");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let budget = parsed["budget_tokens"].as_u64().unwrap();
    let used = parsed["used_tokens"].as_u64().unwrap();

    assert_eq!(budget, 1000);
    assert!(
        used <= budget,
        "used_tokens ({}) should not exceed budget_tokens ({})",
        used,
        budget
    );
}

// ---------------------------------------------------------------------------
// Mode: bundle (directory output)
// ---------------------------------------------------------------------------

#[test]
fn test_context_bundle_mode() {
    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("bundle")
        .assert()
        .success()
        // Bundle mode writes concatenated file contents to stdout
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// --output flag (write to file)
// ---------------------------------------------------------------------------

#[test]
fn test_context_output_to_file_json() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("context_output.json");

    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--output")
        .arg(&out_file)
        .assert()
        .success();

    assert!(out_file.exists(), "output file should be created");

    let content = fs::read_to_string(&out_file).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("output file should contain valid JSON");

    assert_eq!(parsed["schema_version"].as_u64(), Some(4));
    assert_eq!(parsed["mode"].as_str(), Some("context"));
    assert!(parsed["files"].is_array());
}

#[test]
fn test_context_output_to_file_list() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("context_list.md");

    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--mode")
        .arg("list")
        .arg("--output")
        .arg(&out_file)
        .assert()
        .success();

    assert!(out_file.exists(), "output file should be created");

    let content = fs::read_to_string(&out_file).unwrap();
    assert!(
        content.contains("src/main.rs"),
        "list output should contain fixture file"
    );
}

// ---------------------------------------------------------------------------
// Strategy flag
// ---------------------------------------------------------------------------

#[test]
fn test_context_strategy_greedy() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--strategy")
        .arg("greedy")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["strategy"].as_str(), Some("greedy"));
}

#[test]
fn test_context_strategy_spread() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .arg("--strategy")
        .arg("spread")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["strategy"].as_str(), Some("spread"));
}

// ---------------------------------------------------------------------------
// JSON receipt file rows have expected fields
// ---------------------------------------------------------------------------

#[test]
fn test_context_json_file_rows_have_required_fields() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("context")
        .arg("--mode")
        .arg("json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let files = parsed["files"].as_array().unwrap();
    assert!(!files.is_empty());

    let first = &files[0];
    assert!(first["path"].is_string(), "file row should have path");
    assert!(
        first["tokens"].is_number(),
        "file row should have token count"
    );
}

// ---------------------------------------------------------------------------
// Bundle directory output (--bundle-dir)
// ---------------------------------------------------------------------------

#[test]
fn test_context_bundle_dir_creates_artifacts() {
    let dir = tempdir().unwrap();
    let bundle_dir = dir.path().join("context_bundle");

    let mut cmd = tokmd_cmd();
    cmd.arg("context")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .assert()
        .success();

    // Bundle dir should contain manifest.json and bundle.txt
    assert!(
        bundle_dir.join("manifest.json").exists(),
        "bundle dir should contain manifest.json"
    );
    assert!(
        bundle_dir.join("bundle.txt").exists(),
        "bundle dir should contain bundle.txt"
    );

    // manifest.json should be valid JSON
    let manifest_content = fs::read_to_string(bundle_dir.join("manifest.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&manifest_content).expect("manifest.json should be valid JSON");

    assert_eq!(parsed["schema_version"].as_u64(), Some(2));
    assert!(parsed["budget_tokens"].is_number());
    assert!(parsed["used_tokens"].is_number());
    assert!(parsed["included_files"].is_array());
}
