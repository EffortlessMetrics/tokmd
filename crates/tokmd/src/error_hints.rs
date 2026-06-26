use anyhow::Error;

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

fn missing_path_as_unrecognized_subcommand(err: &Error) -> Option<String> {
    for entry in err.chain() {
        let message = entry.to_string();
        let token = message
            .strip_prefix("Path not found: ")
            .or_else(|| message.strip_prefix("Input path does not exist: "));

        if let Some(token) = token {
            let token = token.trim();
            if looks_like_bare_subcommand_token(token) {
                return Some(token.to_string());
            }
        }
    }

    None
}

fn looks_like_bare_subcommand_token(token: &str) -> bool {
    !token.is_empty()
        && !token.starts_with('-')
        && !token.contains('/')
        && !token.contains('\\')
        && !token.contains('.')
        && !token.contains(':')
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

    if haystack.contains("rate limit")
        || haystack.contains("rate_limit")
        || haystack.contains("too many requests")
        || haystack.contains("http 429")
        || haystack.contains("status 429")
    {
        push_hint(
            &mut out,
            "The upstream service is limiting requests. Wait briefly, then retry.",
        );
        push_hint(
            &mut out,
            "Honor provider retry windows such as `Retry-After` when available.",
        );
        push_hint(
            &mut out,
            "Use a smaller input scope if this command contacts a remote service.",
        );
    }

    if haystack.contains("timed out")
        || haystack.contains("timeout")
        || haystack.contains("temporary")
        || haystack.contains("temporarily")
        || haystack.contains("connection reset")
        || haystack.contains("connection refused")
        || haystack.contains("broken pipe")
        || haystack.contains("dns")
        || haystack.contains("network error")
        || haystack.contains("service unavailable")
        || haystack.contains("http 503")
        || haystack.contains("status 503")
    {
        push_hint(
            &mut out,
            "This looks transient. Retry with backoff after network or service health recovers.",
        );
        push_hint(
            &mut out,
            "Check network, VPN, or proxy settings if retries keep failing.",
        );
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
                    if looks_like_bare_subcommand_token(bad_path) {
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
                if looks_like_bare_subcommand_token(&bp) {
                    push_hint(
                        &mut out,
                        "Run `tokmd --help` to see a list of available subcommands.",
                    );
                    return out;
                }
            } else {
                push_hint(
                    &mut out,
                    "Run `tokmd --help` to see a list of available subcommands.",
                );
                return out;
            }
        }

        if did_you_mean {
            return out;
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

    // JSON receipt / baseline parse failures (gate, diff, analyze, badge,
    // baseline) surface a bare serde error. Point the user at a valid source
    // instead of leaving them to guess what shape the file should have. Scoped
    // away from TOML so the two parse hints do not both fire.
    if haystack.contains("failed to parse")
        && !haystack.contains("toml")
        && (haystack.contains("receipt")
            || haystack.contains("json")
            || haystack.contains("export rows"))
    {
        push_hint(
            &mut out,
            "Ensure the file is a tokmd JSON receipt (produced by `tokmd run`, `tokmd export`, or `tokmd analyze`).",
        );
        push_hint(
            &mut out,
            "If it was hand-edited or truncated, regenerate the receipt and retry.",
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
    fn suggests_for_json_receipt_parse_failure() {
        let err = anyhow!("Failed to parse JSON from receipt.json: expected value at line 1");
        let hints = suggestions(&err);
        assert!(
            hints.iter().any(|h| h.contains("tokmd JSON receipt")),
            "expected receipt-source guidance, got: {hints:?}"
        );
        assert!(
            hints.iter().any(|h| h.contains("regenerate the receipt")),
            "expected regenerate guidance, got: {hints:?}"
        );
    }

    #[test]
    fn suggests_for_lang_receipt_parse_failure() {
        let err = anyhow!("Failed to parse lang receipt");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("tokmd JSON receipt")));
    }

    #[test]
    fn suggests_for_baseline_json_parse_failure() {
        let err = anyhow!("Failed to parse baseline JSON from baseline.json");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("tokmd JSON receipt")));
    }

    #[test]
    fn toml_parse_failure_does_not_get_receipt_hint() {
        let err = anyhow!("invalid TOML: failed to parse key at line 3");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("tokmd.toml")));
        assert!(
            !hints.iter().any(|h| h.contains("tokmd JSON receipt")),
            "TOML parse errors must not get the JSON receipt hint, got: {hints:?}"
        );
    }

    #[test]
    fn suggests_for_transient_network_errors() {
        let err = anyhow!("request timed out while contacting remote service");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("looks transient")));
        assert!(hints.iter().any(|h| h.contains("VPN, or proxy")));
    }
}
