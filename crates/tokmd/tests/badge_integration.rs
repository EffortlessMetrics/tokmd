use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_badge_uses_config_metric() {
    // Given: A temp dir with tokmd.toml setting badge metric to lines
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("tokmd.toml"),
        r#"
[badge]
metric = "lines"
"#,
    )
    .unwrap();

    // When: We run tokmd badge without --metric
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("badge")
        .assert()
        .success()
        // Then: It should succeed and produce SVG output with "lines" label
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("lines"));
}

#[test]
fn test_badge_cli_overrides_config() {
    // Given: A temp dir with tokmd.toml setting badge metric to lines
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("tokmd.toml"),
        r#"
[badge]
metric = "lines"
"#,
    )
    .unwrap();

    // When: We run tokmd badge with --metric tokens
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("badge")
        .arg("--metric")
        .arg("tokens")
        .assert()
        .success()
        // Then: It should produce SVG output with "tokens" label
        .stdout(predicate::str::contains("tokens"));
}

#[test]
fn test_badge_missing_arg_and_config() {
    // Given: Empty temp dir (no config)
    let dir = tempdir().unwrap();

    // When: We run tokmd badge without args
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("badge")
        .assert()
        .failure()
        // Then: It should fail with helpful error message
        .stderr(predicate::str::contains("Badge metric is required"));
}
