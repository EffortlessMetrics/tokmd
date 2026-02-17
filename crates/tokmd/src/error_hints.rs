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
    }

    #[test]
    fn suggests_for_unknown_explain_key() {
        let err = anyhow!("Unknown metric/finding key 'foo'.");
        let hints = suggestions(&err);
        assert!(hints.iter().any(|h| h.contains("--explain list")));
    }

    #[test]
    fn format_includes_hints_section() {
        let err = anyhow!("Path not found: no-file");
        let rendered = format(&err);
        assert!(rendered.contains("Error:"));
        assert!(rendered.contains("Hints:"));
    }
}
