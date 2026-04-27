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

    if haystack.contains("parent traversal")
        || haystack.contains("must be relative")
        || haystack.contains("escapes scan root")
        || haystack.contains("scan root must not be empty")
        || haystack.contains("bounded path must not be empty")
    {
        push_hint(
            &mut out,
            "Pass paths inside the selected scan root; parent traversal (`..`) is rejected.",
        );
        push_hint(
            &mut out,
            "Use root-relative paths for scanned entries, or choose the containing directory as the root.",
        );

        if haystack.contains("escapes scan root") {
            push_hint(
                &mut out,
                "Avoid symlinked or redirected paths that resolve outside the scan root.",
            );
        }
    }

    if haystack.contains("path not found")
        || haystack.contains("input path does not exist")
        || haystack.contains("no such file or directory")
    {
        let mut did_you_mean = false;

        let mut extracted_bad_path = None;

        // Check for common typoed subcommands in "Path not found: <bad>"
        if haystack.contains("path not found") {
            // Find the original path string from the chain
            for e in err.chain() {
                let e_str = e.to_string();
                if e_str.starts_with("Path not found: ") {
                    let bad_path = e_str.trim_start_matches("Path not found: ").trim();
                    extracted_bad_path = Some(bad_path.to_string());
                    if !bad_path.contains('/') && !bad_path.contains('.') && !bad_path.is_empty() {
                        let known = [
                            "lang",
                            "module",
                            "export",
                            "analyze",
                            "badge",
                            "init",
                            "completions",
                            "run",
                            "diff",
                            "context",
                            "check-ignore",
                            "tools",
                            "gate",
                            "cockpit",
                            "baseline",
                            "handoff",
                            "sensor",
                        ];

                        let mut best_match = None;
                        let mut best_dist = usize::MAX;

                        for k in known.iter() {
                            let d = levenshtein(bad_path, k);
                            if d < best_dist {
                                best_dist = d;
                                best_match = Some(*k);
                            }
                        }

                        if let Some(m) = best_match {
                            // Max distance 2 for a typo, or proportional to length
                            let threshold = std::cmp::max(2, m.len() / 3);
                            if best_dist <= threshold && best_dist > 0 {
                                push_hint(&mut out, &format!("Did you mean the subcommand `{m}`?"));
                                did_you_mean = true;
                            }
                        }
                    }
                    break;
                }
            }
        }

        if !did_you_mean {
            if let Some(bp) = extracted_bad_path {
                if !bp.contains('/') && !bp.contains('.') && !bp.contains('\\') {
                    push_hint(
                        &mut out,
                        &format!(
                            "If `{bp}` was intended as a subcommand, it is not recognized. Use `tokmd --help`."
                        ),
                    );
                }
            } else {
                push_hint(
                    &mut out,
                    "If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.",
                );
            }
        }
        push_hint(&mut out, "Verify the input path exists and is readable.");
        push_hint(
            &mut out,
            "Use an absolute path to avoid working-directory confusion.",
        );
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

    if haystack.contains("unknown metric/finding key") {
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

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.is_empty() {
        return b_chars.len();
    }
    if b_chars.is_empty() {
        return a_chars.len();
    }

    let mut d = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

    for (i, row) in d.iter_mut().enumerate().take(a_chars.len() + 1) {
        row[0] = i;
    }
    for (j, item) in d[0].iter_mut().enumerate().take(b_chars.len() + 1) {
        *item = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            d[i][j] = std::cmp::min(
                std::cmp::min(d[i - 1][j] + 1, d[i][j - 1] + 1),
                d[i - 1][j - 1] + cost,
            );
        }
    }

    d[a_chars.len()][b_chars.len()]
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
    fn suggests_for_typo_subcommand() {
        let err = anyhow!("Path not found: anolyze");
        let hints = suggestions(&err);
        assert!(
            hints
                .iter()
                .any(|h| h.contains("Did you mean the subcommand `analyze`?"))
        );
        assert!(
            !hints
                .iter()
                .any(|h| h.contains("subcommand, it is not recognized"))
        );
    }

    #[test]
    fn suggests_for_missing_path() {
        let err = anyhow!("Path not found: does-not-exist");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("input path exists")));
        assert!(hints.iter().any(|h| {
            h.contains("If `does-not-exist` was intended as a subcommand, it is not recognized")
        }));
    }

    #[test]
    fn suggests_for_parent_traversal() {
        let err = anyhow!("Bounded path must not contain parent traversal: ../secret.txt");
        let hints = suggestions(&err);
        assert!(
            hints
                .iter()
                .any(|h| h.contains("inside the selected scan root"))
        );
        assert!(hints.iter().any(|h| h.contains("root-relative paths")));
    }

    #[test]
    fn suggests_for_root_escape() {
        let err = anyhow!("Bounded path escapes scan root C:/repo: C:/secret.txt");
        let rendered = format(&err);
        assert!(rendered.contains("Error:"));
        assert!(rendered.contains("Hints:"));
        assert!(rendered.contains("inside the selected scan root"));
        assert!(rendered.contains("resolve outside the scan root"));
    }

    #[test]
    fn resolve_failures_do_not_get_bounded_path_hints() {
        let err = anyhow!("Failed to resolve scan root C:/repo: permission denied");
        let hints = suggestions(&err);
        assert!(
            !hints
                .iter()
                .any(|h| h.contains("parent traversal") || h.contains("root-relative"))
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
