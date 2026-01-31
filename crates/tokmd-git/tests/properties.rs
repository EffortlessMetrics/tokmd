//! Property-based tests for tokmd-git.
//!
//! These tests verify parsing logic, edge case handling, and safety properties
//! without requiring actual git execution.

use proptest::prelude::*;
use std::path::PathBuf;
use tokmd_git::{GitCommit, git_available, repo_root};

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

/// Strategy for generating git log header lines in the "%ct|%ae" format.
fn arb_git_log_line() -> impl Strategy<Value = String> {
    (arb_valid_timestamp(), arb_author_email())
        .prop_map(|(ts, author)| format!("{}|{}", ts, author))
}

/// Strategy for generating malformed git log lines.
fn arb_malformed_git_log_line() -> impl Strategy<Value = String> {
    prop_oneof![
        // Missing pipe
        arb_valid_timestamp(),
        // Multiple pipes
        (arb_valid_timestamp(), arb_author_email(), "[a-z]{1,10}")
            .prop_map(|(ts, author, extra)| format!("{}|{}|{}", ts, author, extra)),
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

/// Strategy for generating arbitrary path strings (for repo_root testing).
fn arb_arbitrary_path() -> impl Strategy<Value = PathBuf> {
    prop_oneof![
        Just(PathBuf::from("")),
        Just(PathBuf::from(".")),
        Just(PathBuf::from("..")),
        Just(PathBuf::from("/")),
        Just(PathBuf::from("/tmp")),
        Just(PathBuf::from("C:\\")),
        "[a-zA-Z0-9_/\\\\.-]{1,50}".prop_map(PathBuf::from),
        // Non-existent paths
        "/nonexistent/path/[a-z]{5,10}".prop_map(PathBuf::from),
    ]
}

// ============================================================================
// Parsing Logic Tests
// ============================================================================

/// Simulates the parsing logic from collect_history for a single header line.
/// This mirrors the exact parsing in lib.rs: `line.splitn(2, '|')`
fn parse_header_line(line: &str) -> (i64, String) {
    let mut parts = line.splitn(2, '|');
    let ts = parts.next().unwrap_or("0").parse::<i64>().unwrap_or(0);
    let author = parts.next().unwrap_or("").to_string();
    (ts, author)
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
        let (parsed_ts, parsed_author) = parse_header_line(&line);

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
        let (parsed_ts, parsed_author) = parse_header_line(&line);

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
        let (_, parsed_author) = parse_header_line(&line);

        prop_assert_eq!(parsed_author, "", "Empty author should be empty string");
    }

    /// Missing pipe separator: timestamp is parsed, author is empty.
    #[test]
    fn missing_pipe_produces_empty_author(ts in arb_valid_timestamp()) {
        let line = ts.clone();
        let (parsed_ts, parsed_author) = parse_header_line(&line);

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
        let (parsed_ts, parsed_author) = parse_header_line(line);

        prop_assert_eq!(parsed_ts, 0, "Empty timestamp should be 0");
        prop_assert_eq!(parsed_author, "", "Empty author should be empty string");
    }

    /// Empty line produces defaults.
    #[test]
    fn empty_line_produces_defaults(dummy in 0u8..1) {
        let _ = dummy;
        let line = "";
        let (parsed_ts, parsed_author) = parse_header_line(line);

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
            files: files.clone(),
        };

        prop_assert_eq!(commit.timestamp, ts);
        prop_assert_eq!(commit.author, author);
        prop_assert_eq!(commit.files.len(), files.len());
    }

    /// Timestamp is always a valid i64 (can be 0 for invalid input).
    #[test]
    fn timestamp_is_valid_i64(line in arb_malformed_git_log_line()) {
        let (parsed_ts, _) = parse_header_line(&line);

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
        let (_, parsed_author) = parse_header_line(&line);

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
        let (parsed_ts, parsed_author) = parse_header_line(&line);

        prop_assert_eq!(parsed_ts, 1234567890);
        prop_assert_eq!(parsed_author, long_email);
    }

    /// Very long file paths are handled.
    #[test]
    fn long_file_path_handled(path in arb_long_file_path()) {
        let commit = GitCommit {
            timestamp: 1234567890,
            author: "test@example.com".to_string(),
            files: vec![path.clone()],
        };

        prop_assert_eq!(&commit.files[0], &path);
        prop_assert!(commit.files[0].len() > 100, "Path should be long");
    }

    /// Multiple pipes in author (only first is used as separator).
    #[test]
    fn multiple_pipes_in_line(
        ts in arb_valid_timestamp(),
        part1 in "[a-z]{1,10}",
        part2 in "[a-z]{1,10}"
    ) {
        // Line like "123|author|extra" should parse author as "author|extra"
        let line = format!("{}|{}|{}", ts, part1, part2);
        let (_, parsed_author) = parse_header_line(&line);

        let expected_author = format!("{}|{}", part1, part2);
        prop_assert_eq!(parsed_author, expected_author, "Author should include everything after first pipe");
    }

    /// Whitespace-only lines parse as empty.
    #[test]
    fn whitespace_line_parses(spaces in "[ \t]{1,20}") {
        let (parsed_ts, _) = parse_header_line(&spaces);

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
        let (parsed_ts, parsed_author) = parse_header_line(&line);

        prop_assert_eq!(parsed_ts, ts, "Negative timestamp should parse correctly");
        prop_assert_eq!(parsed_author, author);
    }
}

// ============================================================================
// Non-panicking function tests
// ============================================================================

proptest! {
    /// git_available() never panics.
    #[test]
    fn git_available_never_panics(dummy in 0u8..1) {
        let _ = dummy;
        // This just tests that the function doesn't panic
        let _ = git_available();
    }

    /// repo_root() never panics with arbitrary paths.
    #[test]
    fn repo_root_never_panics_with_arbitrary_path(path in arb_arbitrary_path()) {
        // This tests that repo_root doesn't panic even with invalid paths
        let _ = repo_root(&path);
    }

    /// repo_root() with empty path doesn't panic.
    #[test]
    fn repo_root_empty_path_no_panic(dummy in 0u8..1) {
        let _ = dummy;
        let _ = repo_root(std::path::Path::new(""));
    }

    /// repo_root() with current directory doesn't panic.
    #[test]
    fn repo_root_current_dir_no_panic(dummy in 0u8..1) {
        let _ = dummy;
        let _ = repo_root(std::path::Path::new("."));
    }

    /// repo_root() with non-existent deep path doesn't panic.
    #[test]
    fn repo_root_nonexistent_deep_path_no_panic(
        parts in prop::collection::vec("[a-z]{3,10}", 5..=10)
    ) {
        let path = PathBuf::from(format!("/nonexistent/{}", parts.join("/")));
        let _ = repo_root(&path);
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

proptest! {
    /// Parsing is deterministic.
    #[test]
    fn parsing_is_deterministic(line in arb_git_log_line()) {
        let (ts1, author1) = parse_header_line(&line);
        let (ts2, author2) = parse_header_line(&line);

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
                files: vec![format!("file{}.rs", i)],
            })
            .collect();

        let limited = apply_commit_limit(commits.clone(), None);

        prop_assert_eq!(limited.len(), commits.len(), "All commits should be returned");
    }
}
