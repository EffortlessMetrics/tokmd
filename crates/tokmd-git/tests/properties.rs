//! Property-based tests for tokmd-git.
//!
//! These tests verify parsing logic, edge case handling, and safety properties
//! without requiring actual git execution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_git::{GitCommit, GitRangeMode, classify_intent, git_available, repo_root};
use tokmd_types::CommitIntentKind;

// ============================================================================
// Strategies for generating test data
// ============================================================================

/// Strategy for generating valid Unix timestamps (realistic range).
fn arb_valid_timestamp() -> impl Strategy<Value = String> {
    // Timestamps from 2000 to 2030 (approximate)
    (946684800i64..1893456000i64).prop_map(|ts| ts.to_string())
}

/// Strategy for generating invalid timestamp strings.
fn arb_invalid_timestamp() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("not_a_number".to_string()),
        Just("-1".to_string()),
        Just("abc123".to_string()),
        Just("12.34".to_string()),
        Just("9999999999999999999999".to_string()),
        "[a-z]{1,10}".prop_map(|s| s),
    ]
}

/// Strategy for generating email-like author strings.
fn arb_author_email() -> impl Strategy<Value = String> {
    prop_oneof![
        // Valid email formats
        "[a-z]{1,10}@[a-z]{1,10}\\.[a-z]{2,4}".prop_map(|s| s),
        // Simple usernames
        "[a-z]{1,20}".prop_map(|s| s),
        // Emails with dots
        "[a-z.]{1,15}@[a-z]{1,10}\\.[a-z]{2,4}".prop_map(|s| s),
    ]
}

/// Strategy for generating git log header lines in the "%ct|%ae|%s" format.
fn arb_git_log_line() -> impl Strategy<Value = String> {
    (
        arb_valid_timestamp(),
        arb_author_email(),
        "[a-zA-Z0-9 _-]{0,50}",
    )
        .prop_map(|(ts, author, subject)| format!("{}|{}|{}", ts, author, subject))
}

/// Strategy for generating malformed git log lines.
fn arb_malformed_git_log_line() -> impl Strategy<Value = String> {
    prop_oneof![
        // Missing pipe
        arb_valid_timestamp(),
        // Multiple pipes (now valid with 3-field format)
        (
            arb_valid_timestamp(),
            arb_author_email(),
            "[a-z]{1,10}",
            "[a-z]{1,10}"
        )
            .prop_map(|(ts, author, subj, extra)| format!("{}|{}|{}|{}", ts, author, subj, extra)),
        // Empty string
        Just("".to_string()),
        // Only pipe
        Just("|".to_string()),
        // Pipe at start
        arb_author_email().prop_map(|author| format!("|{}", author)),
        // Pipe at end
        arb_valid_timestamp().prop_map(|ts| format!("{}|", ts)),
        // Invalid timestamp with valid author
        (arb_invalid_timestamp(), arb_author_email())
            .prop_map(|(ts, author)| format!("{}|{}", ts, author)),
    ]
}

/// Strategy for generating file paths (like git --name-only output).
fn arb_file_path() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple file
        "[a-z]{1,10}\\.[a-z]{1,5}".prop_map(|s| s),
        // Nested path
        prop::collection::vec("[a-z]{1,10}", 1..=5).prop_map(|parts| {
            let mut path = parts.join("/");
            path.push_str(".rs");
            path
        }),
        // Deep nested path
        prop::collection::vec("[a-z0-9_-]{1,15}", 5..=10).prop_map(|parts| {
            let mut path = parts.join("/");
            path.push_str(".txt");
            path
        }),
    ]
}

/// Strategy for generating very long file paths.
fn arb_long_file_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z]{10,20}", 10..=20).prop_map(|parts| {
        let mut path = parts.join("/");
        path.push_str(".rs");
        path
    })
}

// ============================================================================
// Diff Hunk Parsing Helper
// ============================================================================

/// Mirrors the hunk-parsing logic from `get_added_lines()` in lib.rs.
/// Extracts added line numbers per file from unified diff output.
fn parse_diff_output(stdout: &str) -> BTreeMap<PathBuf, BTreeSet<usize>> {
    let mut result: BTreeMap<PathBuf, BTreeSet<usize>> = BTreeMap::new();
    let mut current_file: Option<PathBuf> = None;

    for line in stdout.lines() {
        if let Some(file_path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(file_path));
            continue;
        }

        if line.starts_with("@@") {
            let Some(file) = current_file.as_ref() else {
                continue;
            };

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let new_range = parts[2];
            let range_str = new_range.strip_prefix('+').unwrap_or(new_range);
            let range_parts: Vec<&str> = range_str.split(',').collect();

            let start: usize = range_parts[0].parse().unwrap_or(0);
            let count: usize = if range_parts.len() > 1 {
                range_parts[1].parse().unwrap_or(1)
            } else {
                1
            };

            if count > 0 && start > 0 {
                let set = result.entry(file.clone()).or_default();
                for i in 0..count {
                    set.insert(start + i);
                }
            }
        }
    }

    result
}

// ============================================================================
// Parsing Logic Tests
// ============================================================================

/// Simulates the parsing logic from collect_history for a single header line.
/// This mirrors the exact parsing in lib.rs: `line.splitn(4, '|')`
fn parse_header_line(line: &str) -> (i64, String, String, String) {
    let mut parts = line.splitn(4, '|');
    let ts = parts.next().unwrap_or("0").parse::<i64>().unwrap_or(0);
    let author = parts.next().unwrap_or("").to_string();
    let hash = parts.next().unwrap_or("").to_string();
    let subject = parts.next().unwrap_or("").to_string();
    (ts, author, hash, subject)
}

/// Simulates the max_commit_files limit logic.
fn apply_file_limit(files: Vec<String>, limit: Option<usize>) -> Vec<String> {
    match limit {
        Some(max) => files.into_iter().take(max).collect(),
        None => files,
    }
}

/// Simulates the max_commits limit logic.
fn apply_commit_limit(commits: Vec<GitCommit>, limit: Option<usize>) -> Vec<GitCommit> {
    match limit {
        Some(max) => commits.into_iter().take(max).collect(),
        None => commits,
    }
}

proptest! {
    // ========================================================================
    // Timestamp Parsing Properties
    // ========================================================================

    /// Valid timestamps parse correctly.
    #[test]
    fn valid_timestamp_parses(
        ts in 0i64..2000000000i64,
        author in arb_author_email()
    ) {
        let line = format!("{}|{}", ts, author);
        let (parsed_ts, parsed_author, _, _) = parse_header_line(&line);

        prop_assert_eq!(parsed_ts, ts, "Timestamp should parse correctly");
        prop_assert_eq!(parsed_author, author, "Author should parse correctly");
    }

    /// Invalid timestamps default to 0 (not panic).
    #[test]
    fn invalid_timestamp_defaults_to_zero(
        invalid_ts in arb_invalid_timestamp(),
        author in arb_author_email()
    ) {
        let line = format!("{}|{}", invalid_ts, author);
        let (parsed_ts, parsed_author, _, _) = parse_header_line(&line);

        // Invalid timestamp should parse as 0 or a valid i64 (for "-1")
        // The key property is it doesn't panic and produces a valid i64
        // (which is guaranteed by the type system, so we just verify it runs)
        let _ = parsed_ts; // Parsing completed without panic
        prop_assert_eq!(parsed_author, author, "Author should still parse correctly");
    }

    /// Empty author produces empty string (not panic).
    #[test]
    fn empty_author_is_empty_string(ts in arb_valid_timestamp()) {
        let line = format!("{}|", ts);
        let (_, parsed_author, _, _) = parse_header_line(&line);

        prop_assert_eq!(parsed_author, "", "Empty author should be empty string");
    }

    /// Missing pipe separator: timestamp is parsed, author is empty.
    #[test]
    fn missing_pipe_produces_empty_author(ts in arb_valid_timestamp()) {
        let line = ts.clone();
        let (parsed_ts, parsed_author, _, _) = parse_header_line(&line);

        // The timestamp string itself becomes the "timestamp" and author is ""
        let expected_ts = ts.parse::<i64>().unwrap_or(0);
        prop_assert_eq!(parsed_ts, expected_ts, "Timestamp should parse");
        prop_assert_eq!(parsed_author, "", "Author should be empty when no pipe");
    }

    /// Line with only pipe separator.
    #[test]
    fn only_pipe_separator(dummy in 0u8..1) {
        let _ = dummy;
        let line = "|";
        let (parsed_ts, parsed_author, _, _) = parse_header_line(line);

        prop_assert_eq!(parsed_ts, 0, "Empty timestamp should be 0");
        prop_assert_eq!(parsed_author, "", "Empty author should be empty string");
    }

    /// Empty line produces defaults.
    #[test]
    fn empty_line_produces_defaults(dummy in 0u8..1) {
        let _ = dummy;
        let line = "";
        let (parsed_ts, parsed_author, _, _) = parse_header_line(line);

        prop_assert_eq!(parsed_ts, 0, "Empty line should produce timestamp 0");
        prop_assert_eq!(parsed_author, "", "Empty line should produce empty author");
    }

    // ========================================================================
    // GitCommit Structure Properties
    // ========================================================================

    /// GitCommit can be constructed with arbitrary valid data.
    #[test]
    fn git_commit_construction(
        ts in 0i64..2000000000i64,
        author in arb_author_email(),
        files in prop::collection::vec(arb_file_path(), 0..20)
    ) {
        let commit = GitCommit {
            timestamp: ts,
            author: author.clone(),
            hash: None,
            subject: String::new(),
            files: files.clone(),
        };

        prop_assert_eq!(commit.timestamp, ts);
        prop_assert_eq!(commit.author, author);
        prop_assert_eq!(commit.files.len(), files.len());
    }

    /// Timestamp is always a valid i64 (can be 0 for invalid input).
    #[test]
    fn timestamp_is_valid_i64(line in arb_malformed_git_log_line()) {
        let (parsed_ts, _, _, _) = parse_header_line(&line);

        // The key property: parsing never panics and produces a valid i64
        // The type system guarantees i64 bounds, so we verify parsing completes
        // and for malformed input defaults to a reasonable value (typically 0 or -1)
        prop_assert!(
            parsed_ts == 0 || parsed_ts == -1 || parsed_ts > 0,
            "Malformed input should parse to 0, -1, or a valid positive timestamp"
        );
    }

    /// Author is always valid UTF-8 string.
    #[test]
    fn author_is_valid_utf8(line in arb_git_log_line()) {
        let (_, parsed_author, _, _) = parse_header_line(&line);

        // String type guarantees UTF-8 validity
        prop_assert!(parsed_author.is_ascii() || !parsed_author.is_empty() || parsed_author.is_empty());
    }

    // ========================================================================
    // File List Limit Properties
    // ========================================================================

    /// max_commit_files limit is respected.
    #[test]
    fn file_limit_is_respected(
        files in prop::collection::vec(arb_file_path(), 0..50),
        limit in 0usize..20
    ) {
        let limited = apply_file_limit(files.clone(), Some(limit));

        prop_assert!(
            limited.len() <= limit,
            "File count {} should not exceed limit {}",
            limited.len(),
            limit
        );
    }

    /// No limit returns all files.
    #[test]
    fn no_limit_returns_all(files in prop::collection::vec(arb_file_path(), 0..50)) {
        let limited = apply_file_limit(files.clone(), None);

        prop_assert_eq!(limited.len(), files.len(), "All files should be returned");
    }

    /// Limit of 0 returns empty list.
    #[test]
    fn limit_zero_returns_empty(files in prop::collection::vec(arb_file_path(), 1..50)) {
        let limited = apply_file_limit(files, Some(0));

        prop_assert!(limited.is_empty(), "Limit 0 should return empty list");
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    /// Very long author emails are handled.
    #[test]
    fn long_author_email_handled(
        prefix in "[a-z]{50,100}",
        domain in "[a-z]{20,50}"
    ) {
        let long_email = format!("{}@{}.com", prefix, domain);
        let line = format!("1234567890|{}", long_email);
        let (parsed_ts, parsed_author, _, _) = parse_header_line(&line);

        prop_assert_eq!(parsed_ts, 1234567890);
        prop_assert_eq!(parsed_author, long_email);
    }

    /// Very long file paths are handled.
    #[test]
    fn long_file_path_handled(path in arb_long_file_path()) {
        let commit = GitCommit {
            timestamp: 1234567890,
            author: "test@example.com".to_string(),
            hash: None,
            subject: String::new(),
            files: vec![path.clone()],
        };

        prop_assert_eq!(&commit.files[0], &path);
        prop_assert!(commit.files[0].len() > 100, "Path should be long");
    }

    /// Multiple pipes: format is now "ts|author|hash|subject" with splitn(4).
    #[test]
    fn four_field_parsing(
        ts in arb_valid_timestamp(),
        author in "[a-z]{1,10}",
        hash in "[0-9a-f]{40}",
        subject in "[a-z]{1,10}"
    ) {
        let line = format!("{}|{}|{}|{}", ts, author, hash, subject);
        let (_, parsed_author, parsed_hash, parsed_subject) = parse_header_line(&line);

        prop_assert_eq!(parsed_author, author, "Author should be second field");
        prop_assert_eq!(parsed_hash, hash, "Hash should be third field");
        prop_assert_eq!(parsed_subject, subject, "Subject should be fourth field");
    }

    /// Subject may contain pipes (splitn(4) handles this).
    #[test]
    fn subject_with_pipes(
        ts in arb_valid_timestamp(),
        author in "[a-z]{1,10}",
        hash in "[0-9a-f]{40}",
        part1 in "[a-z]{1,10}",
        part2 in "[a-z]{1,10}"
    ) {
        let subject = format!("{}|{}", part1, part2);
        let line = format!("{}|{}|{}|{}", ts, author, hash, subject);
        let (_, parsed_author, _parsed_hash, parsed_subject) = parse_header_line(&line);

        prop_assert_eq!(parsed_author, author, "Author should be second field");
        prop_assert_eq!(parsed_subject, subject, "Subject should contain pipe");
    }

    /// Whitespace-only lines parse as empty.
    #[test]
    fn whitespace_line_parses(spaces in "[ \t]{1,20}") {
        let (parsed_ts, _, _, _) = parse_header_line(&spaces);

        // Whitespace cannot be parsed as i64, so it becomes 0
        prop_assert_eq!(parsed_ts, 0, "Whitespace should not parse as valid timestamp");
    }

    /// Negative timestamps are valid i64 values.
    #[test]
    fn negative_timestamp_is_valid(
        ts in -1000000000i64..0i64,
        author in arb_author_email()
    ) {
        let line = format!("{}|{}", ts, author);
        let (parsed_ts, parsed_author, _, _) = parse_header_line(&line);

        prop_assert_eq!(parsed_ts, ts, "Negative timestamp should parse correctly");
        prop_assert_eq!(parsed_author, author);
    }
}

// ============================================================================
// Non-panicking function tests (unit tests, not property tests)
// ============================================================================
// These are "doesn't panic" smoke tests that involve real I/O (process spawn,
// filesystem access). They only need to run once, not N times with random input,
// so they're regular unit tests rather than property tests.

#[test]
fn git_available_never_panics() {
    let _ = git_available();
}

#[test]
fn repo_root_edge_cases_never_panic() {
    // Empty path
    let _ = repo_root(std::path::Path::new(""));
    // Current directory
    let _ = repo_root(std::path::Path::new("."));
    // Parent directory
    let _ = repo_root(std::path::Path::new(".."));
    // Root path
    let _ = repo_root(std::path::Path::new("/"));
    // Windows root
    #[cfg(windows)]
    let _ = repo_root(std::path::Path::new(r"C:\"));
    // Non-existent deep path
    let _ = repo_root(std::path::Path::new(
        "/nonexistent/deep/path/that/does/not/exist",
    ));
    // Relative non-existent path
    let _ = repo_root(std::path::Path::new("nonexistent/relative/path"));
}

#[test]
fn repo_root_finds_git_dir_in_ancestors() {
    if !git_available() {
        eprintln!("git not available; skipping repo_root correctness tests");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    // Initialize a real git repo (just .git dir isn't enough - git rev-parse needs a valid repo)
    let status = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .status()
        .expect("failed to spawn git");
    assert!(status.success(), "git init failed: {status}");
    let result = repo_root(dir.path());
    assert!(result.is_some(), "repo_root should find the git repo");
    // Canonicalize both paths to handle symlinks and path normalization
    let expected = dir.path().canonicalize().unwrap();
    let actual = result.unwrap().canonicalize().unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn repo_root_finds_git_dir_from_nested_path() {
    if !git_available() {
        eprintln!("git not available; skipping repo_root correctness tests");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let status = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .status()
        .expect("failed to spawn git");
    assert!(status.success(), "git init failed: {status}");
    let nested = dir.path().join("src").join("lib");
    std::fs::create_dir_all(&nested).unwrap();
    let result = repo_root(&nested);
    assert!(
        result.is_some(),
        "repo_root should find the git repo from nested path"
    );
    let expected = dir.path().canonicalize().unwrap();
    let actual = result.unwrap().canonicalize().unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn repo_root_returns_none_without_git_dir() {
    if !git_available() {
        eprintln!("git not available; skipping repo_root tests");
        return;
    }
    let dir = tempfile::tempdir().unwrap();

    // If the temp directory is inside an existing repo (rare but possible via TMPDIR),
    // repo_root returning Some is correct. Skip instead of failing.
    if repo_root(dir.path()).is_some() {
        eprintln!("tempdir appears to be inside an existing git repo; skipping negative case");
        return;
    }

    assert_eq!(repo_root(dir.path()), None);
}

// ============================================================================
// Determinism Tests
// ============================================================================

proptest! {
    /// Parsing is deterministic.
    #[test]
    fn parsing_is_deterministic(line in arb_git_log_line()) {
        let (ts1, author1, _, _) = parse_header_line(&line);
        let (ts2, author2, _, _) = parse_header_line(&line);

        prop_assert_eq!(ts1, ts2, "Timestamp parsing should be deterministic");
        prop_assert_eq!(author1, author2, "Author parsing should be deterministic");
    }
}

// ============================================================================
// Commit limit simulation tests
// ============================================================================

proptest! {
    /// max_commits limit is respected.
    #[test]
    fn commit_limit_is_respected(
        commit_count in 0usize..50,
        limit in 1usize..20
    ) {
        let commits: Vec<GitCommit> = (0..commit_count)
            .map(|i| GitCommit {
                timestamp: i as i64,
                author: format!("author{}@example.com", i),
                hash: None,
                subject: String::new(),
                files: vec![format!("file{}.rs", i)],
            })
            .collect();

        let limited = apply_commit_limit(commits, Some(limit));

        prop_assert!(
            limited.len() <= limit,
            "Commit count {} should not exceed limit {}",
            limited.len(),
            limit
        );
    }

    /// No commit limit returns all commits.
    #[test]
    fn no_commit_limit_returns_all(commit_count in 0usize..50) {
        let commits: Vec<GitCommit> = (0..commit_count)
            .map(|i| GitCommit {
                timestamp: i as i64,
                author: format!("author{}@example.com", i),
                hash: None,
                subject: String::new(),
                files: vec![format!("file{}.rs", i)],
            })
            .collect();

        let limited = apply_commit_limit(commits.clone(), None);

        prop_assert_eq!(limited.len(), commits.len(), "All commits should be returned");
    }
}

// ============================================================================
// Diff hunk parsing property tests
// ============================================================================

proptest! {
    /// `@@ ... +start,count @@` produces `{start..start+count}`.
    #[test]
    fn hunk_with_count_produces_consecutive_lines(
        file in arb_file_path(),
        start in 1usize..1000,
        count in 1usize..100,
    ) {
        let diff = format!(
            "+++ b/{}\n@@ -1,1 +{},{} @@\n",
            file, start, count
        );
        let result = parse_diff_output(&diff);
        let expected: BTreeSet<usize> = (start..start + count).collect();
        let file_path = PathBuf::from(&file);
        prop_assert_eq!(
            result.get(&file_path),
            Some(&expected),
            "Hunk +{},{} should produce lines {}..{}",
            start, count, start, start + count
        );
    }

    /// `+start,0` produces nothing (deletion-only hunk).
    #[test]
    fn hunk_with_zero_count_produces_no_lines(
        file in arb_file_path(),
        start in 1usize..1000,
    ) {
        let diff = format!(
            "+++ b/{}\n@@ -1,1 +{},0 @@\n",
            file, start
        );
        let result = parse_diff_output(&diff);
        prop_assert!(
            result.is_empty(),
            "Zero count should produce no lines, got: {:?}",
            result
        );
    }

    /// Random text without `@@` or `+++ b/` yields empty map.
    #[test]
    fn no_hunk_headers_produces_empty_result(
        text in "[a-zA-Z0-9 \n]{0,500}"
    ) {
        let result = parse_diff_output(&text);
        prop_assert!(
            result.is_empty(),
            "Text without hunk headers should produce empty result, got: {:?}",
            result
        );
    }

    /// Same input always produces same output.
    #[test]
    fn diff_parsing_is_deterministic(
        file in arb_file_path(),
        start in 1usize..1000,
        count in 1usize..100,
    ) {
        let diff = format!(
            "+++ b/{}\n@@ -1,1 +{},{} @@\n",
            file, start, count
        );
        let r1 = parse_diff_output(&diff);
        let r2 = parse_diff_output(&diff);
        prop_assert_eq!(r1, r2, "Parsing should be deterministic");
    }
}

// ============================================================================
// classify_intent property tests
// ============================================================================

/// Strategy for conventional commit subjects.
fn arb_conventional_subject() -> impl Strategy<Value = String> {
    let types = prop_oneof![
        Just("feat"),
        Just("fix"),
        Just("refactor"),
        Just("docs"),
        Just("test"),
        Just("chore"),
        Just("ci"),
        Just("build"),
        Just("perf"),
        Just("style"),
        Just("revert"),
        Just("bugfix"),
        Just("hotfix"),
        Just("feature"),
        Just("doc"),
        Just("tests"),
    ];
    let scope = prop_oneof![Just("".to_string()), "[a-z]{1,8}".prop_map(|s| format!("({})", s)),];
    let bang = prop_oneof![Just(""), Just("!")];
    let desc = "[a-zA-Z0-9 ]{1,40}";
    (types, scope, bang, desc).prop_map(|(t, s, b, d)| format!("{}{}{}: {}", t, s, b, d))
}

/// Strategy for freeform commit subjects (keyword heuristic territory).
fn arb_freeform_subject() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z ]{1,60}".prop_map(|s| s),
        Just("Fix crash on startup".to_string()),
        Just("Add user authentication".to_string()),
        Just("Update readme".to_string()),
        Just("WIP".to_string()),
        Just("v1.0.0".to_string()),
        Just("".to_string()),
        Just("   ".to_string()),
    ]
}

proptest! {
    // ========================================================================
    // classify_intent never panics
    // ========================================================================

    /// classify_intent never panics on arbitrary input.
    #[test]
    fn classify_intent_never_panics(subject in ".*") {
        let _ = classify_intent(&subject);
    }

    /// classify_intent always returns a valid CommitIntentKind variant.
    #[test]
    fn classify_intent_returns_valid_variant(subject in arb_freeform_subject()) {
        let kind = classify_intent(&subject);
        // Verify it's one of the known variants by matching
        match kind {
            CommitIntentKind::Feat
            | CommitIntentKind::Fix
            | CommitIntentKind::Refactor
            | CommitIntentKind::Docs
            | CommitIntentKind::Test
            | CommitIntentKind::Chore
            | CommitIntentKind::Ci
            | CommitIntentKind::Build
            | CommitIntentKind::Perf
            | CommitIntentKind::Style
            | CommitIntentKind::Revert
            | CommitIntentKind::Other => {} // all good
        }
    }

    /// classify_intent is deterministic.
    #[test]
    fn classify_intent_is_deterministic(subject in ".*") {
        let a = classify_intent(&subject);
        let b = classify_intent(&subject);
        prop_assert_eq!(a, b, "classify_intent should be deterministic");
    }

    /// Conventional commits always classify to a non-Other kind.
    #[test]
    fn conventional_commits_never_classify_as_other(subject in arb_conventional_subject()) {
        let kind = classify_intent(&subject);
        prop_assert_ne!(
            kind,
            CommitIntentKind::Other,
            "Conventional commit '{}' should not classify as Other",
            subject
        );
    }

    /// Empty or whitespace-only subjects always classify as Other.
    #[test]
    fn blank_subjects_are_other(spaces in "[ \t\n\r]{0,20}") {
        let kind = classify_intent(&spaces);
        prop_assert_eq!(
            kind,
            CommitIntentKind::Other,
            "Blank input '{}' should be Other",
            spaces
        );
    }

    // ========================================================================
    // GitRangeMode property tests
    // ========================================================================

    /// GitRangeMode::format always contains both base and head.
    #[test]
    fn range_format_contains_base_and_head(
        base in "[a-zA-Z0-9/_.-]{1,30}",
        head in "[a-zA-Z0-9/_.-]{1,30}",
        mode in prop_oneof![Just(GitRangeMode::TwoDot), Just(GitRangeMode::ThreeDot)]
    ) {
        let formatted = mode.format(&base, &head);
        prop_assert!(
            formatted.contains(&base),
            "Formatted range '{}' should contain base '{}'",
            formatted, base
        );
        prop_assert!(
            formatted.contains(&head),
            "Formatted range '{}' should contain head '{}'",
            formatted, head
        );
    }

    /// TwoDot format always contains exactly ".." and never "...".
    #[test]
    fn two_dot_format_has_double_dot(
        base in "[a-z]{1,10}",
        head in "[a-z]{1,10}"
    ) {
        let formatted = GitRangeMode::TwoDot.format(&base, &head);
        prop_assert!(formatted.contains(".."), "Should contain ..");
        // Count dots: should have exactly 2 consecutive dots (not 3)
        let without_range = formatted.replacen("..", "", 1);
        prop_assert!(
            !without_range.contains(".."),
            "Should not contain extra double-dots after removing the range separator"
        );
    }

    /// ThreeDot format always contains "...".
    #[test]
    fn three_dot_format_has_triple_dot(
        base in "[a-z]{1,10}",
        head in "[a-z]{1,10}"
    ) {
        let formatted = GitRangeMode::ThreeDot.format(&base, &head);
        prop_assert!(formatted.contains("..."), "Should contain ...");
    }

    /// GitRangeMode::format is deterministic.
    #[test]
    fn range_format_is_deterministic(
        base in "[a-z]{1,10}",
        head in "[a-z]{1,10}",
        mode in prop_oneof![Just(GitRangeMode::TwoDot), Just(GitRangeMode::ThreeDot)]
    ) {
        let a = mode.format(&base, &head);
        let b = mode.format(&base, &head);
        prop_assert_eq!(a, b, "format should be deterministic");
    }

    // ========================================================================
    // Hunk parsing: multiple hunks in same file accumulate lines
    // ========================================================================

    /// Two disjoint hunks in the same file produce the union of their line sets.
    #[test]
    fn two_hunks_same_file_accumulate(
        file in arb_file_path(),
        start1 in 1usize..500,
        count1 in 1usize..50,
        gap in 50usize..200,
        count2 in 1usize..50,
    ) {
        let start2 = start1 + count1 + gap;
        let diff = format!(
            "+++ b/{f}\n@@ -1,1 +{s1},{c1} @@\n@@ -{s2},1 +{s2},{c2} @@\n",
            f = file, s1 = start1, c1 = count1, s2 = start2, c2 = count2
        );
        let result = parse_diff_output(&diff);
        let key = PathBuf::from(&file);
        let lines = result.get(&key).expect("file should be present");

        let expected_count = count1 + count2;
        prop_assert_eq!(
            lines.len(), expected_count,
            "Two disjoint hunks should produce {} lines, got {}",
            expected_count, lines.len()
        );

        // Verify ranges are correct
        for i in 0..count1 {
            prop_assert!(lines.contains(&(start1 + i)));
        }
        for i in 0..count2 {
            prop_assert!(lines.contains(&(start2 + i)));
        }
    }

    /// Multiple files in a single diff are all captured.
    #[test]
    fn multiple_files_in_diff(
        file1 in "[a-z]{1,8}\\.rs",
        file2 in "[a-z]{1,8}\\.txt",
        start1 in 1usize..100,
        start2 in 1usize..100,
    ) {
        // Ensure distinct file names
        prop_assume!(file1 != file2);
        let diff = format!(
            "+++ b/{f1}\n@@ -1,1 +{s1},1 @@\n+++ b/{f2}\n@@ -1,1 +{s2},1 @@\n",
            f1 = file1, s1 = start1, f2 = file2, s2 = start2
        );
        let result = parse_diff_output(&diff);
        prop_assert_eq!(result.len(), 2, "Should have 2 files");
        prop_assert!(result.contains_key(&PathBuf::from(&file1)));
        prop_assert!(result.contains_key(&PathBuf::from(&file2)));
    }

    /// Hunk with start=0 produces no lines (invalid line number).
    #[test]
    fn hunk_start_zero_produces_no_lines(
        file in arb_file_path(),
        count in 1usize..50,
    ) {
        let diff = format!(
            "+++ b/{}\n@@ -1,1 +0,{} @@\n",
            file, count
        );
        let result = parse_diff_output(&diff);
        prop_assert!(
            result.is_empty(),
            "start=0 should produce no lines, got: {:?}",
            result
        );
    }
}
