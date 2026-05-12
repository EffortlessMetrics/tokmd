//! Commit intent classification for git history subjects.

use tokmd_types::CommitIntentKind;

/// Classify a commit subject line into an intent kind.
///
/// Uses a two-stage pipeline:
/// 1. **Conventional Commits**: Parse `type(scope)!: description` prefix
/// 2. **Keyword heuristic**: Match known keywords in the subject
pub fn classify_intent(subject: &str) -> CommitIntentKind {
    let trimmed = subject.trim();
    if trimmed.is_empty() {
        return CommitIntentKind::Other;
    }

    // Check for revert pattern first
    if trimmed.starts_with("Revert \"") || trimmed.starts_with("revert:") {
        return CommitIntentKind::Revert;
    }

    // Try conventional commit parsing
    if let Some(kind) = parse_conventional_prefix(trimmed) {
        return kind;
    }

    // Fall back to keyword heuristic
    keyword_heuristic(trimmed)
}

/// Parse a conventional commit prefix like `feat(scope)!: description`.
fn parse_conventional_prefix(subject: &str) -> Option<CommitIntentKind> {
    let colon_pos = subject.find(':')?;
    let prefix = &subject[..colon_pos];

    // Strip optional (scope) and trailing !
    let prefix = if let Some(paren_pos) = prefix.find('(') {
        &prefix[..paren_pos]
    } else {
        prefix
    };
    let prefix = prefix.trim_end_matches('!');

    match prefix.to_ascii_lowercase().as_str() {
        "feat" | "feature" => Some(CommitIntentKind::Feat),
        "fix" | "bugfix" | "hotfix" => Some(CommitIntentKind::Fix),
        "refactor" => Some(CommitIntentKind::Refactor),
        "docs" | "doc" => Some(CommitIntentKind::Docs),
        "test" | "tests" => Some(CommitIntentKind::Test),
        "chore" => Some(CommitIntentKind::Chore),
        "ci" => Some(CommitIntentKind::Ci),
        "build" => Some(CommitIntentKind::Build),
        "perf" => Some(CommitIntentKind::Perf),
        "style" => Some(CommitIntentKind::Style),
        "revert" => Some(CommitIntentKind::Revert),
        _ => None,
    }
}

/// Keyword-based heuristic for commit intent classification.
fn keyword_heuristic(subject: &str) -> CommitIntentKind {
    let lower = subject.to_ascii_lowercase();

    // Ordered by priority: more specific matches first
    if contains_word(&lower, "revert") {
        CommitIntentKind::Revert
    } else if contains_word(&lower, "fix")
        || contains_word(&lower, "bug")
        || contains_word(&lower, "patch")
        || contains_word(&lower, "hotfix")
    {
        CommitIntentKind::Fix
    } else if contains_word(&lower, "feat")
        || contains_word(&lower, "feature")
        || lower.starts_with("add ")
        || lower.starts_with("implement ")
        || lower.starts_with("introduce ")
    {
        CommitIntentKind::Feat
    } else if contains_word(&lower, "refactor") || contains_word(&lower, "restructure") {
        CommitIntentKind::Refactor
    } else if contains_word(&lower, "doc") || contains_word(&lower, "readme") {
        CommitIntentKind::Docs
    } else if contains_word(&lower, "test") {
        CommitIntentKind::Test
    } else if contains_word(&lower, "perf")
        || contains_word(&lower, "performance")
        || contains_word(&lower, "optimize")
    {
        CommitIntentKind::Perf
    } else if contains_word(&lower, "style")
        || contains_word(&lower, "format")
        || contains_word(&lower, "lint")
    {
        CommitIntentKind::Style
    } else if contains_word(&lower, "ci") || contains_word(&lower, "pipeline") {
        CommitIntentKind::Ci
    } else if contains_word(&lower, "build") || contains_word(&lower, "deps") {
        CommitIntentKind::Build
    } else if contains_word(&lower, "chore") || contains_word(&lower, "cleanup") {
        CommitIntentKind::Chore
    } else {
        CommitIntentKind::Other
    }
}

/// Check if a word appears as a word boundary match in the subject.
fn contains_word(haystack: &str, word: &str) -> bool {
    for (idx, _) in haystack.match_indices(word) {
        let before_ok = idx == 0 || !haystack.as_bytes()[idx - 1].is_ascii_alphanumeric();
        let after_idx = idx + word.len();
        let after_ok =
            after_idx >= haystack.len() || !haystack.as_bytes()[after_idx].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_intent_prefers_conventional_commit_prefix() {
        assert_eq!(
            classify_intent("feat(parser): add support"),
            CommitIntentKind::Feat
        );
        assert_eq!(
            classify_intent("fix!: breaking hotfix"),
            CommitIntentKind::Fix
        );
        assert_eq!(
            classify_intent("docs(readme): update usage"),
            CommitIntentKind::Docs
        );
        assert_eq!(
            classify_intent("test: add regression"),
            CommitIntentKind::Test
        );
    }

    #[test]
    fn classify_intent_uses_keyword_heuristics() {
        assert_eq!(classify_intent("Add caching layer"), CommitIntentKind::Feat);
        assert_eq!(
            classify_intent("optimize parser allocations"),
            CommitIntentKind::Perf
        );
        assert_eq!(classify_intent("lint workspace"), CommitIntentKind::Style);
        assert_eq!(
            classify_intent("pipeline: update checks"),
            CommitIntentKind::Ci
        );
    }

    #[test]
    fn classify_intent_handles_revert_and_empty_subjects() {
        assert_eq!(
            classify_intent("Revert \"bad commit\""),
            CommitIntentKind::Revert
        );
        assert_eq!(
            classify_intent("revert: undo change"),
            CommitIntentKind::Revert
        );
        assert_eq!(classify_intent("   \t"), CommitIntentKind::Other);
    }

    #[test]
    fn contains_word_respects_word_boundaries() {
        assert!(contains_word("fix parser", "fix"));
        assert!(contains_word("fix-parser", "fix"));
        assert!(!contains_word("prefix parser", "fix"));
        assert!(!contains_word("fixture", "fix"));
    }
}
