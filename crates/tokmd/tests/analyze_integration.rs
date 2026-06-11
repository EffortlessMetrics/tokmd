#![cfg(feature = "analysis")]

mod common;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    // Point to hermetic copy of test fixtures with .git/ marker
    cmd.current_dir(common::fixture_root());
    cmd
}

#[test]
fn analyze_receipt_preset_json_smoke() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON output");

    assert_eq!(json["mode"], "analysis");
    assert_eq!(json["schema_version"], 9);
    assert!(json["generated_at_ms"].is_number());

    // A couple of stable "shape" checks
    assert!(json.get("source").is_some());
    assert!(json.get("args").is_some());
}

#[test]
fn analyze_help_lists_bun_ub_preset() {
    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("analyze")
        .arg("--help")
        .output()
        .expect("failed to execute tokmd analyze --help");

    assert!(
        output.status.success(),
        "tokmd analyze --help failed: {:?}",
        output.status
    );
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(
        stdout.contains("bun-ub"),
        "help should list bun-ub as an analyze preset:\n{stdout}"
    );
}

#[test]
fn analyze_marks_default_capped_complexity_as_partial() {
    let dir = tempdir().expect("should create temp dir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");

    let mut source =
        String::from("pub fn early_branch(x: i32) -> i32 { if x > 0 { x } else { 0 } }\n");
    source
        .push_str(&"// filler keeps the file over the default complexity byte cap\n".repeat(3000));
    std::fs::write(src_dir.join("large.rs"), source).expect("write large Rust source");

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg("src/large.rs")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .arg("--no-git")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert_eq!(json["status"], "partial");
    let warnings = json["warnings"].as_array().expect("warnings array");
    assert!(
        warnings.iter().any(|warning| {
            warning
                .as_str()
                .is_some_and(|text| text.contains("complexity scan bounded"))
        }),
        "{warnings:?}"
    );
}

#[test]
fn analyze_health_scoped_directory_does_not_scan_unrelated_todos() {
    let dir = tempdir().expect("should create temp dir");
    let src_dir = dir.path().join("src");
    let test_dir = dir.path().join("test");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::create_dir_all(&test_dir).expect("create test dir");
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");
    std::fs::write(src_dir.join("main.rs"), "pub const X: i32 = 1;\n").expect("write src file");
    std::fs::write(
        test_dir.join("leak.rs"),
        "// TODO unrelated one\n// TODO unrelated two\npub const Y: i32 = 1;\n",
    )
    .expect("write unrelated file");

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg("src")
        .arg("--preset")
        .arg("health")
        .arg("--format")
        .arg("json")
        .arg("--no-git")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON output");

    assert_eq!(json["status"], "complete");
    assert_eq!(json["derived"]["todo"]["total"].as_u64(), Some(0));
    assert_eq!(json["derived"]["totals"]["files"].as_u64(), Some(1));
}

#[test]
fn analyze_health_skips_dangling_symlink_without_partial_status() {
    let dir = tempdir().expect("should create temp dir");
    let src_dir = dir.path().join("src");
    let fixture_dir = dir.path().join("test").join("fixtures");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::create_dir_all(&fixture_dir).expect("create fixture dir");
    std::fs::create_dir_all(dir.path().join(".git")).expect("create .git marker");
    std::fs::write(src_dir.join("main.rs"), "pub const X: i32 = 1;\n").expect("write src file");

    let missing_target = fixture_dir.join("missing-target.rs");
    let dangling_link = fixture_dir.join("broken-link.rs");
    if create_file_symlink(&missing_target, &dangling_link).is_err() {
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("health")
        .arg("--format")
        .arg("json")
        .arg("--no-git")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON output");

    assert_eq!(json["status"], "complete");
    assert!(
        json["warnings"].as_array().is_some_and(Vec::is_empty),
        "dangling symlink should not create analysis warnings: {:?}",
        json["warnings"]
    );
}

#[test]
fn analyze_effort_bad_base_ref_fails_with_ref_name() {
    if !common::git_available() {
        return;
    }
    let dir = tempdir().expect("should create temp dir");
    if !common::init_git_repo(dir.path()) {
        return;
    }
    std::fs::create_dir_all(dir.path().join("src")).expect("create src dir");
    std::fs::write(dir.path().join("src/main.rs"), "pub const X: i32 = 1;\n")
        .expect("write src file");
    assert!(common::git_add_commit(dir.path(), "initial"));

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("estimate")
        .arg("--format")
        .arg("json")
        .arg("--effort-base-ref")
        .arg("nope-xyz-123")
        .arg("--effort-head-ref")
        .arg("HEAD")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        !output.status.success(),
        "bad effort ref should fail instead of writing a generic receipt"
    );
    let stderr = String::from_utf8(output.stderr).expect("invalid UTF-8");
    assert!(
        stderr.contains("could not resolve ref 'nope-xyz-123'"),
        "stderr should name the unresolved ref, got: {stderr}"
    );
}

#[test]
fn analyze_effort_valid_refs_emit_delta() {
    if !common::git_available() {
        return;
    }
    let dir = tempdir().expect("should create temp dir");
    if !common::init_git_repo(dir.path()) {
        return;
    }
    std::fs::create_dir_all(dir.path().join("src")).expect("create src dir");
    let file = dir.path().join("src/main.rs");
    std::fs::write(&file, "pub const X: i32 = 1;\n").expect("write initial file");
    assert!(common::git_add_commit(dir.path(), "initial"));
    std::fs::write(&file, "pub const X: i32 = 1;\npub const Y: i32 = 2;\n")
        .expect("write changed file");
    assert!(common::git_add_commit(dir.path(), "change"));

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("estimate")
        .arg("--format")
        .arg("json")
        .arg("--effort-base-ref")
        .arg("HEAD~1")
        .arg("--effort-head-ref")
        .arg("HEAD")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "valid effort refs should succeed: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    let delta = json["effort"]["delta"]
        .as_object()
        .expect("effort delta should be present");

    assert_eq!(delta["base"].as_str(), Some("HEAD~1"));
    assert_eq!(delta["head"].as_str(), Some("HEAD"));
    assert!(
        delta["files_changed"].as_u64().unwrap_or(0) >= 1,
        "expected changed files in delta: {delta:?}"
    );
}

#[test]
fn analyze_bun_ub_valid_refs_emit_scoped_review_packet() {
    if !common::git_available() {
        return;
    }
    let dir = tempdir().expect("should create temp dir");
    if !common::init_git_repo(dir.path()) {
        return;
    }
    let src_dir = dir.path().join("src");
    let test_dir = dir.path().join("test");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::create_dir_all(&test_dir).expect("create test dir");
    let file = src_dir.join("main.rs");
    std::fs::write(
        &file,
        "use std::ffi::c_void;\npub unsafe fn native(value: *mut c_void) -> bool { !value.is_null() }\n",
    )
    .expect("write initial file");
    std::fs::write(test_dir.join("leak.rs"), "pub const UNRELATED: i32 = 1;\n")
        .expect("write unrelated file");
    assert!(common::git_add_commit(dir.path(), "initial"));
    std::fs::write(
        &file,
        "use std::ffi::c_void;\npub unsafe fn native(value: *mut c_void) -> bool { !value.is_null() }\npub fn boundary_name() -> &'static str { \"bun\" }\n",
    )
    .expect("write changed file");
    assert!(common::git_add_commit(dir.path(), "change"));

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg("src")
        .arg("--preset")
        .arg("bun-ub")
        .arg("--format")
        .arg("json")
        .arg("--effort-base-ref")
        .arg("HEAD~1")
        .arg("--effort-head-ref")
        .arg("HEAD")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "bun-ub should accept valid refs: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON output");
    assert_eq!(json["args"]["preset"].as_str(), Some("bun-ub"));
    assert_eq!(json["source"]["inputs"][0].as_str(), Some("src"));
    assert_eq!(json["derived"]["totals"]["files"].as_u64(), Some(1));
    assert!(json["effort"]["delta"].is_object(), "effort delta missing");
    assert!(json["imports"].is_object(), "imports signal missing");
    assert!(json["dup"].is_object(), "duplication signal missing");
    assert!(json["complexity"].is_object(), "complexity signal missing");
    assert!(
        json["api_surface"].is_object(),
        "api surface signal missing"
    );
    assert!(json["assets"].is_null(), "bun-ub should not enable assets");
    assert!(json["deps"].is_null(), "bun-ub should not enable deps");
    assert!(
        json["license"].is_null(),
        "bun-ub should not enable license"
    );
    assert!(json["fun"].is_null(), "bun-ub should not enable fun");
}

#[test]
fn analyze_bun_ub_smoke_emits_scoped_json_and_markdown_artifacts() {
    if !common::git_available() {
        return;
    }
    let dir = tempdir().expect("should create temp dir");
    if !common::init_git_repo(dir.path()) {
        return;
    }

    let runtime_api_dir = dir.path().join("src").join("runtime").join("api");
    let bindings_dir = dir.path().join("src").join("bun.js").join("bindings");
    let bun_api_dir = dir.path().join("src").join("bun.js").join("api");
    let fixture_dir = dir
        .path()
        .join("test")
        .join("cli")
        .join("install")
        .join("fixtures");
    std::fs::create_dir_all(&runtime_api_dir).expect("create runtime api dir");
    std::fs::create_dir_all(&bindings_dir).expect("create bindings dir");
    std::fs::create_dir_all(&bun_api_dir).expect("create bun api dir");
    std::fs::create_dir_all(&fixture_dir).expect("create fixture dir");

    let scoped_file = runtime_api_dir.join("MarkdownObject.rs");
    std::fs::write(
        &scoped_file,
        "use std::ffi::c_void;\n\npub unsafe extern \"C\" fn markdown_object(ptr: *mut c_void) -> bool {\n    !ptr.is_null()\n}\n",
    )
    .expect("write scoped initial file");
    std::fs::write(
        bindings_dir.join("native.rs"),
        "pub unsafe extern \"C\" fn unrelated_binding() {}\n",
    )
    .expect("write unrelated binding file");
    std::fs::write(
        bun_api_dir.join("api.rs"),
        "pub fn unrelated_api() -> &'static str { \"outside-scope\" }\n",
    )
    .expect("write unrelated api file");
    assert!(common::git_add_commit(dir.path(), "initial"));

    std::fs::write(
        &scoped_file,
        "use std::ffi::c_void;\nuse std::ptr::NonNull;\n\npub unsafe extern \"C\" fn markdown_object(ptr: *mut c_void) -> bool {\n    NonNull::new(ptr).is_some()\n}\n\npub fn markdown_boundary_name() -> &'static str { \"bun-runtime-api\" }\n",
    )
    .expect("write scoped changed file");
    assert!(common::git_add_commit(dir.path(), "runtime api change"));

    let missing_target = fixture_dir.join("missing-target.rs");
    let dangling_link = fixture_dir.join("broken-bun-fixture.rs");
    let dangling_link_created = create_file_symlink(&missing_target, &dangling_link).is_ok();

    let json_output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg("src/runtime/api")
        .arg("--preset")
        .arg("bun-ub")
        .arg("--format")
        .arg("json")
        .arg("--effort-base-ref")
        .arg("HEAD~1")
        .arg("--effort-head-ref")
        .arg("HEAD")
        .output()
        .expect("failed to execute tokmd analyze json");

    assert!(
        json_output.status.success(),
        "bun-ub json smoke should succeed: {:?}\nstderr: {}",
        json_output.status,
        String::from_utf8_lossy(&json_output.stderr)
    );

    let json_stdout = String::from_utf8(json_output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&json_stdout).expect("invalid JSON output");
    assert_eq!(json["status"], "complete");
    assert_eq!(json["args"]["preset"].as_str(), Some("bun-ub"));
    assert_eq!(
        json["source"]["inputs"][0].as_str(),
        Some("src/runtime/api")
    );
    assert_eq!(json["derived"]["totals"]["files"].as_u64(), Some(1));
    assert!(json["effort"]["delta"].is_object(), "effort delta missing");
    assert!(json["git"].is_object(), "git signal missing");
    assert!(json["predictive_churn"].is_object(), "churn signal missing");
    assert!(json["imports"].is_object(), "imports signal missing");
    assert!(json["complexity"].is_object(), "complexity signal missing");
    assert!(
        json["api_surface"].is_object(),
        "api surface signal missing"
    );

    let warnings_text = json["warnings"]
        .as_array()
        .expect("warnings should be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !warnings_text.contains("test/cli/install")
            && !warnings_text.contains("broken-bun-fixture"),
        "unrelated dangling fixture leaked into warnings: {warnings_text}"
    );

    let json_text = serde_json::to_string(&json).expect("serialize JSON");
    assert!(
        !json_text.contains("test/cli/install"),
        "json evidence should not mention unrelated fixture paths: {json_text}"
    );
    assert!(
        !json_text.contains("src/bun.js"),
        "json evidence should not mention sibling Bun paths outside requested scope: {json_text}"
    );
    assert!(
        json_text.contains("src/runtime/api"),
        "json evidence should name the scoped runtime API path: {json_text}"
    );

    let md_output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("--no-progress")
        .arg("analyze")
        .arg("src/runtime/api")
        .arg("--preset")
        .arg("bun-ub")
        .arg("--format")
        .arg("md")
        .arg("--effort-base-ref")
        .arg("HEAD~1")
        .arg("--effort-head-ref")
        .arg("HEAD")
        .output()
        .expect("failed to execute tokmd analyze md");

    assert!(
        md_output.status.success(),
        "bun-ub markdown smoke should succeed: {:?}\nstderr: {}",
        md_output.status,
        String::from_utf8_lossy(&md_output.stderr)
    );

    let md = String::from_utf8(md_output.stdout).expect("invalid UTF-8");
    for section in [
        "Preset: `bun-ub`",
        "## Effort estimate",
        "### Delta",
        "## Top offenders",
        "### Doc density by language",
        "## Integrity",
    ] {
        assert!(
            md.contains(section),
            "markdown smoke missing section {section:?}:\n{md}"
        );
    }
    assert!(
        md.contains("src/runtime/api"),
        "markdown should name the scoped input path:\n{md}"
    );
    assert!(
        !md.contains("test/cli/install") && !md.contains("src/bun.js"),
        "markdown evidence should stay scoped; dangling link created: {dangling_link_created}\n{md}"
    );
}

#[test]
fn analyze_writes_json_to_output_dir() {
    let dir = tempdir().expect("should create temp dir");
    let out = dir.path();

    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .arg("--output-dir")
        .arg(out)
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}",
        output.status
    );

    let path = out.join("analysis.json");
    assert!(path.exists(), "expected analysis.json at {:?}", path);

    let content = std::fs::read_to_string(&path).expect("failed to read analysis.json");
    let json: Value = serde_json::from_str(&content).expect("analysis.json is not valid JSON");
    assert_eq!(json["mode"], "analysis");
}

#[test]
fn analyze_explain_known_metric() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--explain")
        .arg("avg_cyclomatic")
        .output()
        .expect("failed to execute tokmd analyze --explain");

    assert!(
        output.status.success(),
        "tokmd analyze --explain failed: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(stdout.contains("avg_cyclomatic"));
    assert!(stdout.contains("complexity"));
}

#[test]
fn analyze_explain_list() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--explain")
        .arg("list")
        .output()
        .expect("failed to execute tokmd analyze --explain list");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(stdout.contains("Available metric/finding keys:"));
    assert!(stdout.contains("maintainability_index"));
}

#[test]
fn analyze_explain_unknown_metric_fails() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--explain")
        .arg("not_a_metric")
        .output()
        .expect("failed to execute tokmd analyze --explain unknown");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid UTF-8");
    assert!(stderr.contains("Unknown metric/finding key"));
    assert!(stderr.contains("--explain list"));
}

#[test]
fn analyze_fun_preset_returns_eco_label() {
    // Given: a fixture repository with a small baseline code footprint
    // When: analyze is run with --preset fun and json output
    // Then: eco label metadata is present in the fun section
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("fun")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("analysis JSON output is invalid");

    let eco_label = json["fun"]["eco_label"]
        .as_object()
        .expect("eco_label should be object");
    assert!(eco_label.get("label").is_some());
    assert!(eco_label.get("score").is_some());
    assert!(eco_label.get("notes").is_some());
}

#[test]
fn analyze_topics_preset_returns_topic_cloud() {
    // Given: the same fixture repository used by other analysis tests
    // When: analyze is run with --preset topics and json output
    // Then: topic-cloud payload is present and non-empty
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("topics")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute tokmd analyze");

    assert!(
        output.status.success(),
        "tokmd analyze failed: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("analysis JSON output is invalid");

    let topics = json["topics"].as_object().expect("topics should be object");
    let per_module = topics
        .get("per_module")
        .and_then(Value::as_object)
        .expect("topics.per_module should be object");
    assert!(!per_module.is_empty());

    let overall = topics
        .get("overall")
        .and_then(Value::as_array)
        .expect("topics.overall should be array");
    assert!(!overall.is_empty());
}

#[cfg(unix)]
fn create_file_symlink(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn create_file_symlink(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(src, dst)
}
