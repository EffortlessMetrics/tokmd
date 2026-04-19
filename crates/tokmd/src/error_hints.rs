use anyhow::Error;

pub(crate) fn format(err: &Error) -> String {
    let mut out = format!("Error: {err:#}");
    let hints = suggestions(err);
    if !hints.is_empty() {
        out.push_str("\n\nHints:\n");
        for hint in hints {
            out.push_str("- ");
            out.push_str(&hint);
            out.push('\n');
        }
    }
    out
}

fn suggestions(err: &Error) -> Vec<String> {
    let chain: Vec<String> = err.chain().map(|e| e.to_string()).collect();
    let haystack = chain.join(" | ").to_ascii_lowercase();
    let mut out: Vec<String> = Vec::new();

    if haystack.contains("git is not available on path")
        || haystack.contains("requires the 'git' feature")
    {
        push_hint(&mut out, "Install git and verify it with `git --version`.");
        push_hint(
            &mut out,
            "If git metrics are optional, disable them with `--no-git`.",
        );
    }

    if haystack.contains("not inside a git repository") {
        push_hint(
            &mut out,
            "Run the command from a git repository, or disable git-dependent behavior.",
        );
        push_hint(&mut out, "Initialize git first if needed: `git init`.");
    }

    if haystack.contains("path not found")
        || haystack.contains("input path does not exist")
        || haystack.contains("no such file or directory")
    {
        // Check if the missing path looks like a typo of a subcommand
        let mut did_you_mean = None;

        // Extract the path name from the error string if possible
        if let Some(path_start) = err.to_string().rfind("Path not found: ") {
            let path = err.to_string()[path_start + 16..].trim().to_string();
            // Subcommands usually do not contain slashes
            if !path.contains('/') && !path.contains('\\') {
                let subcommands = [
                    "lang",
                    "module",
                    "export",
                    "analyze",
                    "badge",
                    "diff",
                    "context",
                    "gate",
                    "handoff",
                    "completions",
                    "run",
                    "init",
                    "check-ignore",
                    "cockpit",
                    "sensor",
                    "baseline",
                    "tools",
                ];
                let mut best_match = None;
                let mut best_score = 0.0;

                for cmd in subcommands {
                    let score = strsim::jaro_winkler(&path, cmd);
                    if score > best_score {
                        best_score = score;
                        best_match = Some(cmd);
                    }
                }

                if best_score > 0.8 {
                    did_you_mean = Some(format!("Did you mean `{}`?", best_match.unwrap()));
                }
            }
        }

        if let Some(suggestion) = did_you_mean {
            push_hint(&mut out, &suggestion);
        } else {
            push_hint(&mut out, "Verify the input path exists and is readable.");
            push_hint(
                &mut out,
                "Use an absolute path to avoid working-directory confusion.",
            );
            push_hint(
                &mut out,
                "If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.",
            );
        }
    }

    if haystack.contains("base ref") && haystack.contains("not found") {
        push_hint(
            &mut out,
            "Fetch refs (`git fetch --tags --prune`) and retry with `--base <ref>`.",
        );
        push_hint(
            &mut out,
            "You can also set `TOKMD_GIT_BASE_REF` to a valid default base ref.",
        );
    }

    if haystack.contains("failed to load diff source") || haystack.contains("invalid reference") {
        push_hint(
            &mut out,
            "If you meant to compare files, ensure they both exist locally.",
        );
        push_hint(
            &mut out,
            "If you meant to compare git refs, ensure the branch, tag, or commit exists.",
        );
    }

    if haystack.contains("unknown metric/finding key") && !haystack.contains("use --explain list") {
        push_hint(
            &mut out,
            "Run `tokmd analyze --explain list` to see supported keys.",
        );
    }

    if haystack.contains("toml") && (haystack.contains("parse") || haystack.contains("invalid")) {
        push_hint(
            &mut out,
            "Check `tokmd.toml` syntax and key names, or regenerate with `tokmd init --force`.",
        );
    }

    out
}

fn push_hint(out: &mut Vec<String>, hint: &str) {
    if !out.iter().any(|h| h == hint) {
        out.push(hint.to_string());
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::{format, suggestions};

    #[test]
    fn suggests_for_missing_git() {
        let err = anyhow!("git is not available on PATH");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("git --version")));
        assert!(hints.iter().any(|h| h.contains("--no-git")));
    }

    #[test]
    fn suggests_for_missing_path() {
        let err = anyhow!("Path not found: does-not-exist");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("input path exists")));
        assert!(
            hints
                .iter()
                .any(|h| h.contains("subcommand, it is not recognized"))
        );
    }

    #[test]
    fn suggests_for_unknown_explain_key() {
        let err = anyhow!("Unknown metric/finding key 'foo'.");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("--explain list")));
    }

    #[test]
    fn suggests_for_missing_diff_source() {
        let err = anyhow!(
            "Failed to load diff source 'missing_file.json': Failed to create worktree for 'missing_file.json': git worktree add failed for 'missing_file.json'"
        );
        let hints = suggestions(&err);
        assert!(
            hints
                .iter()
                .any(|h| h.contains("ensure they both exist locally"))
        );
        assert!(
            hints
                .iter()
                .any(|h| h.contains("ensure the branch, tag, or commit exists"))
        );
    }

    #[test]
    fn format_includes_hints_section() {
        let err = anyhow!("Path not found: no-file");
        let rendered = format(&err);
        assert!(rendered.contains("Error:"));
        assert!(rendered.contains("Hints:"));
    }
}
