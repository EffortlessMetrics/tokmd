use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

fn run_xtask(args: &[&str]) -> (String, String, bool) {
    let root = workspace_root();
    let output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("xtask")
        .arg("--")
        .args(args)
        .current_dir(&root)
        .output()
        .expect("failed to run cargo xtask");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

#[test]
fn check_no_panic_family_advisory_mode_passes() {
    // Advisory mode (the default) reports findings without failing on
    // unallowlisted ones; only schema/shape, expired, and stale entries block.
    let (stdout, stderr, success) = run_xtask(&["check-no-panic-family"]);

    assert!(
        success,
        "advisory check-no-panic-family failed.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("no-panic policy:"),
        "stdout missing summary line: {stdout}"
    );
    assert!(
        stdout.contains("finding(s)"),
        "stdout missing finding count: {stdout}"
    );
}

#[test]
fn check_no_panic_family_emits_json_report() {
    let root = workspace_root();
    let path = root
        .join("target")
        .join("no-panic-w94")
        .join("no-panic-report.json");
    if path.exists() {
        fs::remove_file(&path).expect("stale no-panic report should be removable");
    }

    let path_arg = path.to_string_lossy().to_string();
    let (stdout, stderr, success) = run_xtask(&[
        "check-no-panic-family",
        "--json",
        "--json-output",
        &path_arg,
    ]);

    assert!(
        success,
        "json check-no-panic-family failed.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.trim_start().starts_with('{'),
        "stdout is not JSON: {stdout}"
    );
    assert!(
        stdout.contains("\"finding_count\""),
        "json report missing finding_count: {stdout}"
    );
    assert!(
        stdout.contains("\"unallowlisted_findings\""),
        "json report missing unallowlisted_findings: {stdout}"
    );
    assert!(path.exists(), "json-output report should be written");

    let written = fs::read_to_string(&path).expect("json-output report should be readable");
    let stdout_json: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout no-panic report should be JSON");
    let written_json: serde_json::Value =
        serde_json::from_str(&written).expect("written no-panic report should be JSON");
    assert_eq!(written_json, stdout_json);
}

#[test]
fn check_no_panic_family_help_mentions_json_output() {
    let (stdout, stderr, success) = run_xtask(&["check-no-panic-family", "--help"]);

    assert!(
        success,
        "check-no-panic-family --help failed. stderr: {stderr}"
    );
    assert!(stdout.contains("--json"), "stdout: {stdout}");
    assert!(stdout.contains("--json-output"), "stdout: {stdout}");
    assert!(stdout.contains("--strict"), "stdout: {stdout}");
}

#[test]
fn no_panic_workflow_uses_json_output_artifact() {
    let root = workspace_root();
    let workflow = fs::read_to_string(root.join(".github/workflows/no-panic-policy.yml"))
        .expect("no-panic workflow should be readable");

    assert!(
        workflow.contains("--json-output target/tokmd/reports/no-panic-report.json"),
        "workflow should write the no-panic JSON artifact through xtask"
    );
    assert!(
        !workflow.contains("--json \\\n            > target/tokmd/reports/no-panic-report.json"),
        "workflow should not capture no-panic JSON with shell redirection"
    );
}

#[test]
fn xtask_help_mentions_no_panic_gate() {
    let (stdout, stderr, success) = run_xtask(&["--help"]);

    assert!(success, "xtask --help failed. stderr: {stderr}");
    assert!(stdout.contains("check-no-panic-family"), "stdout: {stdout}");
    assert!(stdout.contains("no-panic-propose"), "stdout: {stdout}");
}
