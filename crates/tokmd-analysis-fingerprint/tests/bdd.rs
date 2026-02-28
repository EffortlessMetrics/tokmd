//! BDD-style scenario tests for corporate fingerprint extraction.

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

// ── Empty input ──────────────────────────────────────────────────

#[test]
fn empty_commits_produce_empty_fingerprint() {
    // Given: no commits
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&[]);
    // Then: no domains are reported
    assert!(fp.domains.is_empty());
}

// ── Single author ────────────────────────────────────────────────

#[test]
fn single_corporate_author() {
    // Given: one commit from a corporate email
    let commits = vec![commit("alice@acme.com")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: exactly one domain with 100% share
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 1);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

#[test]
fn single_public_email_author() {
    // Given: one commit from a public email
    let commits = vec![commit("alice@gmail.com")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: bucketed as "public-email"
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 1);
}

// ── Public domain bucketing ──────────────────────────────────────

#[test]
fn all_public_providers_collapse_into_single_bucket() {
    // Given: commits from every recognized public email provider
    let commits = vec![
        commit("a@gmail.com"),
        commit("b@yahoo.com"),
        commit("c@outlook.com"),
        commit("d@hotmail.com"),
        commit("e@icloud.com"),
        commit("f@proton.me"),
        commit("g@protonmail.com"),
    ];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: all collapse into one "public-email" bucket
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 7);
}

// ── Multiple domains ─────────────────────────────────────────────

#[test]
fn multiple_corporate_domains_sorted_by_commit_count() {
    // Given: commits from two corporate domains, one appearing more often
    let commits = vec![
        commit("a@bigcorp.io"),
        commit("b@bigcorp.io"),
        commit("c@bigcorp.io"),
        commit("d@startup.dev"),
    ];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: highest commit count comes first
    assert_eq!(fp.domains.len(), 2);
    assert_eq!(fp.domains[0].domain, "bigcorp.io");
    assert_eq!(fp.domains[0].commits, 3);
    assert_eq!(fp.domains[1].domain, "startup.dev");
    assert_eq!(fp.domains[1].commits, 1);
}

#[test]
fn tie_breaking_by_domain_name() {
    // Given: two domains with equal commit counts
    let commits = vec![commit("a@zebra.com"), commit("b@alpha.com")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: alphabetical ordering breaks the tie
    assert_eq!(fp.domains[0].domain, "alpha.com");
    assert_eq!(fp.domains[1].domain, "zebra.com");
}

#[test]
fn mixed_corporate_and_public_domains() {
    // Given: a mix of corporate and public email authors
    let commits = vec![
        commit("a@acme.com"),
        commit("b@acme.com"),
        commit("c@gmail.com"),
        commit("d@yahoo.com"),
    ];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: corporate and public-email buckets both appear
    assert_eq!(fp.domains.len(), 2);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 2);
    assert_eq!(fp.domains[1].domain, "public-email");
    assert_eq!(fp.domains[1].commits, 2);
}

// ── Percentage calculation ───────────────────────────────────────

#[test]
fn percentages_sum_to_approximately_one() {
    // Given: commits across several domains
    let commits = vec![
        commit("a@foo.com"),
        commit("b@foo.com"),
        commit("c@bar.com"),
        commit("d@gmail.com"),
    ];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: percentages sum ≈ 1.0
    let total_pct: f32 = fp.domains.iter().map(|d| d.pct).sum();
    assert!((total_pct - 1.0).abs() < 0.01);
}

#[test]
fn percentage_values_are_correct() {
    // Given: 3 commits from acme, 1 from startup
    let commits = vec![
        commit("a@acme.com"),
        commit("b@acme.com"),
        commit("c@acme.com"),
        commit("d@startup.dev"),
    ];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: 75% and 25%
    assert!((fp.domains[0].pct - 0.75).abs() < f32::EPSILON);
    assert!((fp.domains[1].pct - 0.25).abs() < f32::EPSILON);
}

// ── Ignored domains ──────────────────────────────────────────────

#[test]
fn localhost_is_ignored() {
    // Given: a commit from localhost
    let commits = vec![commit("root@localhost")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: no domains reported
    assert!(fp.domains.is_empty());
}

#[test]
fn example_com_is_ignored() {
    // Given: a commit from example.com
    let commits = vec![commit("test@example.com")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: no domains reported
    assert!(fp.domains.is_empty());
}

#[test]
fn noreply_github_is_ignored() {
    // Given: a commit from GitHub's noreply address
    let commits = vec![commit("user@users.noreply.github.com")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: no domains reported
    assert!(fp.domains.is_empty());
}

#[test]
fn ignored_domains_do_not_affect_percentages() {
    // Given: one real commit and one from an ignored domain
    let commits = vec![commit("a@acme.com"), commit("bot@users.noreply.github.com")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: only acme.com at 100%
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── Malformed input ──────────────────────────────────────────────

#[test]
fn no_at_sign_is_skipped() {
    // Given: an author string with no '@'
    let commits = vec![commit("noatsign")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: no domains
    assert!(fp.domains.is_empty());
}

#[test]
fn multiple_at_signs_are_skipped() {
    // Given: an author string with multiple '@'
    let commits = vec![commit("bad@@address.com")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: no domains (requires exactly 2 parts)
    assert!(fp.domains.is_empty());
}

#[test]
fn empty_author_string_is_skipped() {
    // Given: an empty author string
    let commits = vec![commit("")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: no domains
    assert!(fp.domains.is_empty());
}

// ── Domain normalization ─────────────────────────────────────────

#[test]
fn domain_case_is_normalized() {
    // Given: commits with mixed-case domains
    let commits = vec![
        commit("a@ACME.COM"),
        commit("b@Acme.Com"),
        commit("c@acme.com"),
    ];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: all merge into one lowercase domain
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 3);
}

#[test]
fn public_domain_case_insensitive() {
    // Given: public email with uppercase domain
    let commits = vec![commit("user@GMAIL.COM")];
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: still bucketed as public-email
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
}

// ── Large input ──────────────────────────────────────────────────

#[test]
fn handles_large_commit_set() {
    // Given: 1000 commits from 10 domains
    let commits: Vec<GitCommit> = (0..1000)
        .map(|i| commit(&format!("dev{}@company{}.com", i, i % 10)))
        .collect();
    // When: we build a fingerprint
    let fp = build_corporate_fingerprint(&commits);
    // Then: exactly 10 domains, all counts sum to 1000
    assert_eq!(fp.domains.len(), 10);
    let total: u32 = fp.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 1000);
}
