use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_invalid_config_errors_out() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("tokmd.toml");
    std::fs::write(&config_path, "invalid = toml").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("lang")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse configuration"));
}
