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

#[cfg(feature = "git")]
#[test]
fn baseline_with_determinism_flag() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let out_file = dir.path().join("baseline.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root())
        .arg("--no-progress")
        .arg("baseline")
        .arg("--determinism")
        .arg("--output")
        .arg(&out_file)
        .arg("--force")
        .assert()
        .success();

    let content = std::fs::read_to_string(&out_file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(json["baseline_version"].as_u64(), Some(1));

    // Determinism section should be present
    let det = json
        .get("determinism")
        .expect("determinism section should be present");

    assert_eq!(det["baseline_version"].as_u64(), Some(1));
    assert!(det.get("generated_at").is_some());
    assert!(det.get("source_hash").is_some());

    // Source hash should be a 64-char hex string
    let hash = det["source_hash"].as_str().unwrap();
    assert_eq!(hash.len(), 64, "BLAKE3 hash should be 64 hex chars");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "hash should be hex"
    );

    Ok(())
}

#[test]
fn baseline_without_determinism_flag_has_no_determinism() -> Result<(), Box<dyn std::error::Error>>
{
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

    // Without --determinism, the field should be absent (skip_serializing_if)
    assert!(
        json.get("determinism").is_none(),
        "determinism should not be present without --determinism flag"
    );

    Ok(())
}
