use anyhow::Error;

use super::{edit_distance::levenshtein, path::looks_like_bare_subcommand_token};

pub(super) fn suggestions(err: &Error) -> Vec<String> {
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

    out
}

fn push_hint(out: &mut Vec<String>, hint: &str) {
    if !out.iter().any(|h| h == hint) {
        out.push(hint.to_string());
    }
}
