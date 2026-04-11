//! Specsmith generated integration test for locking in the exact error behavior
//! of an unknown subcommand in tokmd.

use assert_cmd::Command;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

#[test]
fn test_unknown_subcommand_exact_error_message() {
    let output = tokmd_cmd()
        .arg("frobnicate")
        .output()
        .expect("Failed to execute tokmd");

    assert!(!output.status.success(), "Should exit with non-zero code");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.is_empty(), "Stdout should be empty on error");

    assert!(stderr.contains("Error: Path not found: frobnicate"));
    assert!(stderr.contains("Hints:"));
    assert!(stderr.contains("Verify the input path exists and is readable."));
    assert!(stderr.contains("Use an absolute path to avoid working-directory confusion."));
    assert!(stderr.contains(
        "If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`."
    ));
}
