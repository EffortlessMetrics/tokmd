use crate::cli::DocsArgs;
use anyhow::{Context, Result, bail};
use std::process::Command;

pub fn run(args: DocsArgs) -> Result<()> {
    let repo_root = std::env::current_dir()?;
    let ref_md_path = repo_root.join("docs/reference-cli.md");

    if !ref_md_path.exists() {
        bail!("Reference docs not found at {}", ref_md_path.display());
    }

    let content = std::fs::read_to_string(&ref_md_path)?;
    let mut new_content = content.clone();
    let mut drift = false;

    // We look for patterns like <!-- HELP: lang --> ... <!-- /HELP: lang -->
    // and replace the content with the output of `tokmd <command> --help`

    let markers = [
        ("", "lang"), // Default is lang
        ("module", "module"),
        ("export", "export"),
        ("run", "run"),
        ("analyze", "analyze"),
        ("baseline", "baseline"),
        ("badge", "badge"),
        ("diff", "diff"),
        ("init", "init"),
        ("context", "context"),
        ("handoff", "handoff"),
        ("check-ignore", "check-ignore"),
        ("tools", "tools"),
        ("cockpit", "cockpit"),
        ("sensor", "sensor"),
        ("gate", "gate"),
        ("completions", "completions"),
    ];

    for (cmd_name, marker_id) in markers {
        let start_marker = format!("<!-- HELP: {} -->", marker_id);
        let end_marker = format!("<!-- /HELP: {} -->", marker_id);

        if let Some(start_idx) = new_content.find(&start_marker) {
            if let Some(end_idx) = new_content.find(&end_marker) {
                let help_output = get_tokmd_help(cmd_name)?;
                let wrapped_help = format!("```text\n{}\n```", help_output.trim());

                let range_start = start_idx + start_marker.len();
                let old_help = new_content[range_start..end_idx].trim();

                if old_help != wrapped_help.trim() {
                    drift = true;
                    if args.update {
                        let mut replacement = String::new();
                        replacement.push('\n');
                        replacement.push_str(&wrapped_help);
                        replacement.push('\n');
                        new_content.replace_range(range_start..end_idx, &replacement);
                    }
                }
            }
        }
    }

    if drift {
        if args.update {
            std::fs::write(&ref_md_path, new_content)?;
            println!("Updated {}", ref_md_path.display());
        } else if args.check {
            bail!(
                "Documentation drift detected in {}. Run `cargo xtask docs --update` to fix.",
                ref_md_path.display()
            );
        }
    } else {
        println!("Documentation is up to date.");
    }

    Ok(())
}

fn get_tokmd_help(cmd: &str) -> Result<String> {
    let mut command = Command::new("cargo");
    command
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("tokmd")
        .arg("--");
    if !cmd.is_empty() {
        command.arg(cmd);
    }
    command.arg("--help");

    let output = command.output().context("Failed to run tokmd --help")?;
    if !output.status.success() {
        bail!(
            "tokmd --help failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
