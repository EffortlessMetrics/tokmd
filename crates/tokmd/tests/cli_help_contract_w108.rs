use std::process::Command;

fn help(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "help command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn analyze_help_explains_representative_presets() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = help(&["analyze", "--help"])?;
    for needle in [
        "receipt",
        "Core derived metrics",
        "bun-ub",
        "Manual-candidate UB review",
        "health",
        "TODO density",
        "risk",
        "git hotspots",
    ] {
        if !stdout.contains(needle) {
            return Err(format!("analyze help is missing {needle:?}: {stdout}").into());
        }
    }
    Ok(())
}

#[test]
fn directory_output_help_uses_canonical_name_and_aliases() -> Result<(), Box<dyn std::error::Error>>
{
    let packet = help(&["packet", "generate", "--help"])?;
    let handoff = help(&["handoff", "--help"])?;
    let context = help(&["context", "--help"])?;
    for (name, text) in [
        ("packet", &packet),
        ("handoff", &handoff),
        ("context", &context),
    ] {
        if !text.contains("--output-dir") {
            return Err(format!("{name} help is missing --output-dir: {text}").into());
        }
    }
    if !packet.contains("[alias: --out]")
        || !handoff.contains("--out-dir")
        || !context.contains("--bundle-dir")
    {
        return Err("directory output compatibility aliases are missing from help".into());
    }
    Ok(())
}

#[test]
fn handoff_accepts_legacy_output_directory_alias() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = tempfile::tempdir()?;
    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .args(["handoff", "--no-git", "--budget", "1k", ".", "--out-dir"])
        .arg(output_dir.path())
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "handoff --out-dir failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}
