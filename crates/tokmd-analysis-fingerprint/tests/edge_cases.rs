//! Edge-case BDD tests for corporate fingerprint detection.

use tokmd_analysis_fingerprint::build_corporate_fingerprint;
use tokmd_git::GitCommit;

fn commit(author: &str) -> GitCommit {
    GitCommit {
        timestamp: 0,
        author: author.to_string(),
        hash: None,
        subject: String::new(),
        files: vec![],
    }
}

fn commit_with_ts(author: &str, timestamp: i64) -> GitCommit {
    GitCommit {
        timestamp,
        author: author.to_string(),
        hash: None,
        subject: String::new(),
        files: vec![],
    }
}

// ── Scenario: timestamp variation does not affect fingerprint ────────

#[test]
fn given_same_authors_with_different_timestamps_when_fingerprinted_then_same_result() {
    let commits_a = vec![
        commit_with_ts("dev@acme.com", 1000),
        commit_with_ts("ops@acme.com", 2000),
        commit_with_ts("user@gmail.com", 3000),
    ];
    let commits_b = vec![
        commit_with_ts("dev@acme.com", 9999),
        commit_with_ts("ops@acme.com", 8888),
        commit_with_ts("user@gmail.com", 7777),
    ];

    let fp_a = build_corporate_fingerprint(&commits_a);
    let fp_b = build_corporate_fingerprint(&commits_b);

    assert_eq!(fp_a.domains.len(), fp_b.domains.len());
    for (a, b) in fp_a.domains.iter().zip(fp_b.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}

// ── Scenario: domain with trailing whitespace ───────────────────────

#[test]
fn given_domain_with_whitespace_when_fingerprinted_then_trimmed() {
    let commits = vec![commit("user@  acme.com  ")];
    let fp = build_corporate_fingerprint(&commits);
    // After trimming and lowercasing, should be "acme.com"
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
}

// ── Scenario: at-sign at start/end of string ────────────────────────

#[test]
fn given_at_sign_at_start_when_fingerprinted_then_skipped() {
    let commits = vec![commit("@domain.com")];
    let fp = build_corporate_fingerprint(&commits);
    // splits into ["", "domain.com"] — two parts, but local part is empty
    // domain is "domain.com" which is valid
    // This should still count (the function only checks part count == 2)
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "domain.com");
}

#[test]
fn given_at_sign_at_end_when_fingerprinted_then_empty_domain_ignored() {
    let commits = vec![commit("user@")];
    let fp = build_corporate_fingerprint(&commits);
    // domain would be "" which is_ignored because it's empty after normalize
    assert!(fp.domains.is_empty());
}

// ── Scenario: subdomain handling ────────────────────────────────────

#[test]
fn given_subdomain_emails_when_fingerprinted_then_full_domain_kept() {
    let commits = vec![commit("dev@eng.bigcorp.com"), commit("ops@ops.bigcorp.com")];
    let fp = build_corporate_fingerprint(&commits);
    // Subdomains are kept as-is (no domain normalization beyond case)
    assert_eq!(fp.domains.len(), 2);
    let domains: Vec<&str> = fp.domains.iter().map(|d| d.domain.as_str()).collect();
    assert!(domains.contains(&"eng.bigcorp.com"));
    assert!(domains.contains(&"ops.bigcorp.com"));
}

// ── Scenario: many unique domains ───────────────────────────────────

#[test]
fn given_100_unique_domains_when_fingerprinted_then_all_counted() {
    let commits: Vec<GitCommit> = (0..100)
        .map(|i| commit(&format!("user@company{i}.com")))
        .collect();
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 100);
    let total: u32 = fp.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 100);
    // Each has 1% share
    for d in &fp.domains {
        assert!((d.pct - 0.01).abs() < f32::EPSILON);
    }
}

// ── Scenario: mixed valid, ignored, and malformed authors ───────────

#[test]
fn given_mixed_valid_ignored_and_malformed_when_fingerprinted_then_only_valid_counted() {
    let commits = vec![
        commit("dev@real.com"),                     // valid
        commit("bot@localhost"),                    // ignored
        commit("ci@example.com"),                   // ignored
        commit("noreply@users.noreply.github.com"), // ignored
        commit("malformed"),                        // no @
        commit("a@b@c.com"),                        // multiple @
        commit(""),                                 // empty
        commit("ops@real.com"),                     // valid
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "real.com");
    assert_eq!(fp.domains[0].commits, 2);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── Scenario: serialization preserves all fields ────────────────────

#[test]
fn given_fingerprint_when_serialized_then_contains_expected_fields() {
    let commits = vec![commit("a@foo.com"), commit("b@foo.com"), commit("c@bar.io")];
    let fp = build_corporate_fingerprint(&commits);
    let json = serde_json::to_value(fp).expect("should serialize");

    let domains = json["domains"].as_array().unwrap();
    assert_eq!(domains.len(), 2);

    let first = &domains[0];
    assert!(first.get("domain").is_some());
    assert!(first.get("commits").is_some());
    assert!(first.get("pct").is_some());
}

// ── Scenario: pct is exactly 1.0 for single domain ─────────────────

#[test]
fn given_single_domain_when_fingerprinted_then_pct_is_exactly_one() {
    let commits: Vec<GitCommit> = (0..50).map(|_| commit("dev@only.com")).collect();
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].commits, 50);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── Scenario: all public email providers with corporate ──────────────

#[test]
fn given_all_public_providers_and_corporate_when_fingerprinted_then_two_buckets() {
    let mut commits = vec![
        commit("a@gmail.com"),
        commit("b@yahoo.com"),
        commit("c@outlook.com"),
        commit("d@hotmail.com"),
        commit("e@icloud.com"),
        commit("f@proton.me"),
        commit("g@protonmail.com"),
    ];
    // Add 3 corporate commits
    commits.push(commit("x@corp.dev"));
    commits.push(commit("y@corp.dev"));
    commits.push(commit("z@corp.dev"));

    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 2);
    // public-email: 7 commits, corp.dev: 3 commits → public-email sorted first
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 7);
    assert_eq!(fp.domains[1].domain, "corp.dev");
    assert_eq!(fp.domains[1].commits, 3);
}
