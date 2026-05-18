mod edit_distance;
mod path;
mod suggestions;

use anyhow::Error;

use self::{path::missing_path_as_unrecognized_subcommand, suggestions::suggestions};

pub(crate) fn format(err: &Error) -> String {
    let mut out = if let Some(token) = missing_path_as_unrecognized_subcommand(err) {
        format!("Error: Unrecognized subcommand '{token}'")
    } else {
        format!("Error: {err:#}")
    };
    let mut hints = suggestions(err);
    if out.starts_with("Error: Unrecognized subcommand ") {
        hints.retain(|h| {
            !h.contains("was intended as a subcommand")
                && !h.contains("was meant to be a subcommand")
        });
    }
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

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::{format, suggestions::suggestions};

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
        assert!(!hints.iter().any(|h| h.contains("Run `tokmd --help`")));
        assert!(!hints.iter().any(|h| h.contains("input path exists")));
        assert!(
            !hints
                .iter()
                .any(|h| h.contains("subcommand, it is not recognized"))
        );
    }

    #[test]
    fn format_rewrites_bare_missing_path_as_unrecognized_subcommand() {
        let err = anyhow!("Path not found: frobnicate");
        let rendered = format(&err);
        assert!(rendered.contains("Error: Unrecognized subcommand 'frobnicate'"));
        assert!(!rendered.contains("Error: Path not found: frobnicate"));
        assert!(!rendered.contains("was intended as a subcommand"));
        assert!(rendered.contains("Run `tokmd --help` to see a list of available subcommands."));
        assert!(!rendered.contains("Verify the input path exists and is readable."));
    }

    #[test]
    fn format_preserves_path_shaped_missing_path_errors() {
        let err = anyhow!("Path not found: missing/file.rs");
        let rendered = format(&err);
        assert!(rendered.contains("Error: Path not found: missing/file.rs"));
        assert!(!rendered.contains("Unrecognized subcommand"));
    }

    #[test]
    fn suggests_for_missing_path() {
        let err = anyhow!("Path not found: missing/file.rs");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("input path exists")));
        assert!(hints.iter().any(|h| h.contains("absolute path")));
        assert!(!hints.iter().any(|h| h.contains("Run `tokmd --help`")));
    }

    #[test]
    fn suggests_help_for_unrecognized_bare_subcommand() {
        let err = anyhow!("Path not found: frobnicate");
        let hints = suggestions(&err);
        assert!(
            hints
                .iter()
                .any(|h| h.contains("Run `tokmd --help` to see a list of available subcommands."))
        );
        assert!(!hints.iter().any(|h| h.contains("input path exists")));
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

    #[test]
    fn suggests_for_rate_limit_errors() {
        let err = anyhow!("GitHub returned HTTP 429 Too Many Requests");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("limiting requests")));
        assert!(hints.iter().any(|h| h.contains("Retry-After")));
        assert!(hints.iter().any(|h| h.contains("smaller input scope")));
    }

    #[test]
    fn suggests_for_transient_network_errors() {
        let err = anyhow!("request timed out while contacting remote service");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("looks transient")));
        assert!(hints.iter().any(|h| h.contains("VPN, or proxy")));
    }
}
