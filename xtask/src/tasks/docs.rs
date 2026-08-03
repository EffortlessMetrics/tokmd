use crate::{cli::DocsArgs, tasks::doc_artifacts};
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

const HELP_MARKERS: &[(&str, &str)] = &[
    ("lang", "lang"), // Explicitly use lang subcommand help
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
    ("packet", "packet"),
    // `syntax` is gated behind the `ast` feature, which is on by default, so
    // the default-feature build this generator shells out to always has it.
    ("syntax", "syntax"),
    ("evidence-packet", "evidence-packet"),
    ("render", "render"),
    ("completions", "completions"),
];

pub fn run(args: DocsArgs) -> Result<()> {
    let repo_root = std::env::current_dir()?;
    let ref_md_path = repo_root.join("docs/reference-cli.md");

    if !ref_md_path.exists() {
        bail!("Reference docs not found at {}", ref_md_path.display());
    }

    let content = std::fs::read_to_string(&ref_md_path)?;
    validate_help_markers(&content)?;
    let mut new_content = content.clone();
    let mut drift = false;

    // We look for patterns like <!-- HELP: lang --> ... <!-- /HELP: lang -->
    // and replace the content with the output of `tokmd <command> --help`

    // Every `<!-- HELP: x -->` / `<!-- /HELP: x -->` pair in reference-cli.md must
    // appear here. A pair with no entry is never regenerated, so its block silently
    // freezes at whatever the help output was when it was last written by hand --
    // which is how `syntax`, `evidence-packet`, and `render` fell behind. The
    // missing-marker branch below catches the opposite mistake (an entry with no
    // pair), so the two lists only stay in sync if additions land on both sides.
    for &(cmd_name, marker_id) in HELP_MARKERS {
        let start_marker = format!("<!-- HELP: {} -->", marker_id);
        let end_marker = format!("<!-- /HELP: {} -->", marker_id);

        if let Some(start_idx) = new_content.find(&start_marker)
            && let Some(end_idx) = new_content.find(&end_marker)
        {
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
        } else {
            drift = true;
            if args.check {
                bail!(
                    "Documentation drift detected: Missing marker pair for `{}` in {}. Run `cargo xtask docs --update` to fix.",
                    marker_id,
                    ref_md_path.display()
                );
            } else if args.update {
                println!(
                    "Warning: Missing marker pair for `{}` in {}. You must manually add `<!-- HELP: {} -->` and `<!-- /HELP: {} -->` to the file.",
                    marker_id,
                    ref_md_path.display(),
                    marker_id,
                    marker_id
                );
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

    if args.check {
        let summary = doc_artifacts::check_current_repo(Path::new("policy/doc-artifacts.toml"))?;
        println!("{summary}");
    }

    Ok(())
}

fn validate_help_markers(content: &str) -> Result<()> {
    let configured: BTreeSet<&str> = HELP_MARKERS.iter().map(|(_, marker)| *marker).collect();
    let mut starts = BTreeSet::new();
    let mut ends = BTreeSet::new();
    // Openings still awaiting their close, innermost last.
    //
    // Set equality alone accepts a document whose markers are reversed or
    // crossed. `run` then locates each pair with two independent `find` calls
    // and slices `range_start..end_idx`: for a reversed pair that range runs
    // backwards and panics, and for a crossed pair it spans a neighbouring
    // marker and replaces the wrong help block. Marker blocks never nest, so a
    // close must match the most recent open.
    let mut open: Vec<&str> = Vec::new();

    for line in content.lines().map(str::trim) {
        if let Some(marker) = line
            .strip_prefix("<!-- HELP: ")
            .and_then(|value| value.strip_suffix(" -->"))
        {
            if !starts.insert(marker) {
                bail!("Duplicate help marker pair for `{marker}` in docs/reference-cli.md");
            }
            open.push(marker);
        }
        if let Some(marker) = line
            .strip_prefix("<!-- /HELP: ")
            .and_then(|value| value.strip_suffix(" -->"))
        {
            if !ends.insert(marker) {
                bail!("Duplicate closing help marker for `{marker}` in docs/reference-cli.md");
            }
            match open.pop() {
                Some(top) if top == marker => {}
                Some(top) => bail!(
                    "Help markers cross in docs/reference-cli.md: `{marker}` closes while `{top}` is still open"
                ),
                None => {
                    bail!("Help marker `{marker}` closes before it opens in docs/reference-cli.md")
                }
            }
        }
    }

    if let Some(unclosed) = open.last() {
        bail!("Help marker `{unclosed}` is never closed in docs/reference-cli.md");
    }

    if starts != ends {
        bail!(
            "Help marker pairs are unbalanced in docs/reference-cli.md: starts={starts:?}, ends={ends:?}"
        );
    }
    if starts != configured {
        bail!(
            "Help marker inventory does not match HELP_MARKERS in docs/reference-cli.md: configured={configured:?}, document={starts:?}"
        );
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

    let mut s = String::from_utf8_lossy(&output.stdout).to_string();

    // Normalize cross-platform drift:
    // - Windows prints `tokmd.exe` in Usage lines; Unix prints `tokmd`
    // - CRLF vs LF line endings
    // - clap may indent otherwise blank description spacer lines
    s = s.replace("\r\n", "\n");
    s = s.replace("tokmd.exe", "tokmd");
    s = s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::{HELP_MARKERS, validate_help_markers};
    use anyhow::{Result, bail};
    use clap::CommandFactory;
    use std::collections::BTreeSet;

    #[test]
    fn help_markers_match_cli_commands() {
        let marker_names: BTreeSet<&str> =
            HELP_MARKERS.iter().map(|(command, _)| *command).collect();
        let cli_command = tokmd::cli::Cli::command();
        let cli_names: BTreeSet<&str> = cli_command
            .get_subcommands()
            .map(|command| command.get_name())
            .collect();

        assert_eq!(
            HELP_MARKERS.len(),
            marker_names.len(),
            "HELP_MARKERS contains duplicate command names"
        );
        assert_eq!(marker_names, cli_names);
    }

    #[test]
    fn reference_help_markers_match_configured_inventory() -> Result<()> {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/reference-cli.md");
        let content = std::fs::read_to_string(path)?;
        validate_help_markers(&content)?;
        Ok(())
    }

    #[test]
    fn marker_validation_rejects_reversed_pairs() -> Result<()> {
        // `run` slices `range_start..end_idx`; reversed, that range runs
        // backwards and panics rather than reporting drift.
        let content = "<!-- /HELP: lang -->\nstale\n<!-- HELP: lang -->";
        if validate_help_markers(content).is_ok() {
            bail!("a close before its open must fail");
        }
        Ok(())
    }

    #[test]
    fn marker_validation_rejects_crossed_pairs() -> Result<()> {
        // Each pair is individually ordered, so a per-marker check would pass.
        // The `lang` range still spans `module`'s opening marker, so
        // regenerating `lang` would overwrite it.
        let content = "<!-- HELP: lang -->\n<!-- HELP: module -->\n\
                       <!-- /HELP: lang -->\n<!-- /HELP: module -->";
        if validate_help_markers(content).is_ok() {
            bail!("crossed marker pairs must fail");
        }
        Ok(())
    }

    #[test]
    fn marker_validation_rejects_duplicate_document_ids() -> Result<()> {
        let content =
            "<!-- HELP: lang -->\n<!-- /HELP: lang -->\n<!-- HELP: lang -->\n<!-- /HELP: lang -->";
        if validate_help_markers(content).is_ok() {
            bail!("duplicate marker must fail");
        }
        Ok(())
    }
}
