use anyhow::Error;

use super::catalog::closest_known_subcommand;
use super::path::{extract_missing_path, looks_like_bare_subcommand_token};

pub(super) fn suggestions(err: &Error) -> Vec<String> {
    let haystack = err
        .chain()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(" | ")
        .to_ascii_lowercase();
    let mut out = Vec::new();

    add_git_hints(&haystack, &mut out);
    add_service_hints(&haystack, &mut out);
    add_path_safety_hints(&haystack, &mut out);
    if add_missing_path_hints(err, &haystack, &mut out) == MissingPathAction::ReturnEarly {
        return out;
    }
    add_ref_hints(&haystack, &mut out);
    add_explain_hints(&haystack, &mut out);
    add_config_hints(&haystack, &mut out);

    out
}

fn add_git_hints(haystack: &str, out: &mut Vec<String>) {
    if haystack.contains("git is not available on path")
        || haystack.contains("requires the 'git' feature")
    {
        push_hint(out, "Install git and verify it with `git --version`.");
        push_hint(
            out,
            "If git metrics are optional, disable them with `--no-git`.",
        );
    }

    if haystack.contains("not inside a git repository") {
        push_hint(
            out,
            "Run the command from a git repository, or disable git-dependent behavior.",
        );
        push_hint(out, "Initialize git first if needed: `git init`.");
    }
}

fn add_service_hints(haystack: &str, out: &mut Vec<String>) {
    if contains_any(
        haystack,
        &[
            "rate limit",
            "rate_limit",
            "too many requests",
            "http 429",
            "status 429",
        ],
    ) {
        push_hint(
            out,
            "The upstream service is limiting requests. Wait briefly, then retry.",
        );
        push_hint(
            out,
            "Honor provider retry windows such as `Retry-After` when available.",
        );
        push_hint(
            out,
            "Use a smaller input scope if this command contacts a remote service.",
        );
    }

    if contains_any(
        haystack,
        &[
            "timed out",
            "timeout",
            "temporary",
            "temporarily",
            "connection reset",
            "connection refused",
            "broken pipe",
            "dns",
            "network error",
            "service unavailable",
            "http 503",
            "status 503",
        ],
    ) {
        push_hint(
            out,
            "This looks transient. Retry with backoff after network or service health recovers.",
        );
        push_hint(
            out,
            "Check network, VPN, or proxy settings if retries keep failing.",
        );
    }
}

fn add_path_safety_hints(haystack: &str, out: &mut Vec<String>) {
    if contains_any(
        haystack,
        &[
            "parent traversal",
            "must be relative",
            "escapes scan root",
            "scan root must not be empty",
            "bounded path must not be empty",
        ],
    ) {
        push_hint(
            out,
            "Pass paths inside the selected scan root; parent traversal (`..`) is rejected.",
        );
        push_hint(
            out,
            "Use root-relative paths for scanned entries, or choose the containing directory as the root.",
        );

        if haystack.contains("escapes scan root") {
            push_hint(
                out,
                "Avoid symlinked or redirected paths that resolve outside the scan root.",
            );
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum MissingPathAction {
    Continue,
    ReturnEarly,
}

fn add_missing_path_hints(err: &Error, haystack: &str, out: &mut Vec<String>) -> MissingPathAction {
    if !contains_any(
        haystack,
        &[
            "path not found",
            "input path does not exist",
            "no such file or directory",
        ],
    ) {
        return MissingPathAction::Continue;
    }

    let extracted_bad_path = extract_missing_path(err);
    if let Some(bad_path) = extracted_bad_path.as_deref() {
        if looks_like_bare_subcommand_token(bad_path) {
            if let Some(candidate) = closest_known_subcommand(bad_path) {
                push_hint(out, &format!("Did you mean the subcommand `{candidate}`?"));
            } else {
                push_hint(
                    out,
                    "Run `tokmd --help` to see a list of available subcommands.",
                );
            }
            return MissingPathAction::ReturnEarly;
        }
    } else {
        push_hint(
            out,
            "Run `tokmd --help` to see a list of available subcommands.",
        );
        return MissingPathAction::ReturnEarly;
    }

    push_hint(out, "Verify the input path exists and is readable.");
    push_hint(
        out,
        "Use an absolute path to avoid working-directory confusion.",
    );
    MissingPathAction::Continue
}

fn add_ref_hints(haystack: &str, out: &mut Vec<String>) {
    if haystack.contains("base ref") && haystack.contains("not found") {
        push_hint(
            out,
            "Fetch refs (`git fetch --tags --prune`) and retry with `--base <ref>`.",
        );
        push_hint(
            out,
            "You can also set `TOKMD_GIT_BASE_REF` to a valid default base ref.",
        );
    }

    if haystack.contains("failed to load diff source") || haystack.contains("invalid reference") {
        push_hint(
            out,
            "If you meant to compare files, ensure they both exist locally.",
        );
        push_hint(
            out,
            "If you meant to compare git refs, ensure the branch, tag, or commit exists.",
        );
    }
}

fn add_explain_hints(haystack: &str, out: &mut Vec<String>) {
    if haystack.contains("unknown metric/finding key") {
        push_hint(
            out,
            "Run `tokmd analyze --explain list` to see supported keys.",
        );
    }
}

fn add_config_hints(haystack: &str, out: &mut Vec<String>) {
    if haystack.contains("toml") && (haystack.contains("parse") || haystack.contains("invalid")) {
        push_hint(
            out,
            "Check `tokmd.toml` syntax and key names, or regenerate with `tokmd init --force`.",
        );
    }
}

fn push_hint(out: &mut Vec<String>, hint: &str) {
    if !out.iter().any(|h| h == hint) {
        out.push(hint.to_string());
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
