use std::process::Command;

fn run_tokmd(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
}

#[test]
fn ignored_root_format_is_rejected_without_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_tokmd(&["--format", "json", "module"])?;
    if output.status.success() {
        return Err("root format unexpectedly succeeded before module".into());
    }
    if output.status.code() != Some(2) {
        return Err(format!(
            "root format should be a usage error (exit 2), got {:?}",
            output.status.code()
        )
        .into());
    }
    if !output.stdout.is_empty() {
        return Err("rejected root format wrote to stdout".into());
    }
    let stderr = String::from_utf8(output.stderr)?;
    if !stderr.contains("default `lang` options cannot appear before a subcommand") {
        return Err(format!("missing diagnostic: {stderr}").into());
    }
    Ok(())
}

#[test]
fn module_format_after_subcommand_writes_json() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_tokmd(&["module", "--format", "json", "--no-progress"])?;
    if !output.status.success() {
        return Err(format!(
            "module command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let stdout = String::from_utf8(output.stdout)?;
    let _: serde_json::Value = serde_json::from_str(&stdout)?;
    Ok(())
}
