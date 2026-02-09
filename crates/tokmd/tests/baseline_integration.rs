mod common;

use assert_cmd::Command;

#[test]
fn baseline_generates_output_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let out_file = dir.path().join("baseline.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root())
        .arg("--no-progress")
        .arg("baseline")
        .arg("--output")
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = std::fs::read_to_string(&out_file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(json["baseline_version"].as_u64(), Some(1));
    assert!(json.get("metrics").is_some());
    Ok(())
}
