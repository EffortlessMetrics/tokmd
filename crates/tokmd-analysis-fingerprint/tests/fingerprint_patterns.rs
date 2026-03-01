//! Tests for corporate fingerprint detection patterns.
//!
//! Covers: edge cases (empty repos, no corporate markers, mixed signals),
//! determinism, percentage invariants, and domain classification.

use tokmd_analysis_fingerprint::build_corporate_fingerprint;
use tokmd_git::GitCommit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn commit(author: &str) -> GitCommit {
    GitCommit {
        timestamp: 0,
        author: author.to_string(),
        hash: None,
        subject: String::new(),
        files: vec![],
    }
}

// ===========================================================================
// Edge case: empty repo (no commits)
// ===========================================================================

#[test]
fn empty_repo_produces_empty_fingerprint() {
    let fp = build_corporate_fingerprint(&[]);
    assert!(fp.domains.is_empty());
}

// ===========================================================================
// Edge case: repo with only ignored domains
// ===========================================================================

#[test]
fn only_noreply_github_produces_empty() {
    let commits = vec![
        commit("bot1@users.noreply.github.com"),
        commit("bot2@users.noreply.github.com"),
        commit("12345+user@users.noreply.github.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn only_localhost_produces_empty() {
    let commits = vec![commit("root@localhost"), commit("admin@localhost")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn only_example_com_produces_empty() {
    let commits = vec![commit("test@example.com"), commit("dev@example.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn mix_of_all_ignored_domains_produces_empty() {
    let commits = vec![
        commit("a@localhost"),
        commit("b@example.com"),
        commit("c@users.noreply.github.com"),
        commit("d@noreply.github.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

// ===========================================================================
// Edge case: repo with no corporate markers (only public email)
// ===========================================================================

#[test]
fn only_public_email_shows_single_bucket() {
    let commits = vec![
        commit("a@gmail.com"),
        commit("b@yahoo.com"),
        commit("c@outlook.com"),
        commit("d@hotmail.com"),
        commit("e@icloud.com"),
        commit("f@proton.me"),
        commit("g@protonmail.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 7);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ===========================================================================
// Edge case: malformed author strings
// ===========================================================================

#[test]
fn no_at_sign_skipped() {
    let commits = vec![commit("justausername")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn empty_string_skipped() {
    let commits = vec![commit("")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn triple_at_sign_skipped() {
    let commits = vec![commit("a@b@c@d.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn malformed_among_valid_commits() {
    let commits = vec![
        commit("good@acme.com"),
        commit("bad-no-at"),
        commit("also@good@but-invalid"),
        commit("fine@acme.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 2);
}

// ===========================================================================
// Mixed signals: corporate + public + ignored
// ===========================================================================

#[test]
fn mixed_corporate_public_ignored() {
    let commits = vec![
        commit("dev@corp.io"),
        commit("dev2@corp.io"),
        commit("dev3@corp.io"),
        commit("oss@gmail.com"),
        commit("oss2@yahoo.com"),
        commit("bot@users.noreply.github.com"),
        commit("test@example.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 2);
    assert_eq!(fp.domains[0].domain, "corp.io");
    assert_eq!(fp.domains[0].commits, 3);
    assert_eq!(fp.domains[1].domain, "public-email");
    assert_eq!(fp.domains[1].commits, 2);
    // Ignored commits should not affect totals
    let total: u32 = fp.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 5);
}

// ===========================================================================
// Percentage invariants
// ===========================================================================

#[test]
fn percentages_sum_to_one() {
    let commits = vec![
        commit("a@alpha.com"),
        commit("b@alpha.com"),
        commit("c@beta.com"),
        commit("d@gamma.org"),
        commit("e@gmail.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    let total_pct: f32 = fp.domains.iter().map(|d| d.pct).sum();
    assert!(
        (total_pct - 1.0).abs() < 0.01,
        "percentages should sum to ~1.0, got {total_pct}"
    );
}

#[test]
fn each_percentage_is_between_zero_and_one() {
    let commits = vec![
        commit("a@one.com"),
        commit("b@two.com"),
        commit("c@three.com"),
        commit("d@gmail.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    for d in &fp.domains {
        assert!(
            d.pct >= 0.0 && d.pct <= 1.0,
            "pct for {} should be in [0,1], got {}",
            d.domain,
            d.pct
        );
    }
}

#[test]
fn single_domain_gets_100_percent() {
    let commits = vec![
        commit("a@solo.com"),
        commit("b@solo.com"),
        commit("c@solo.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ===========================================================================
// Sorting: descending by commit count, then alphabetical
// ===========================================================================

#[test]
fn domains_sorted_by_count_desc_then_name_asc() {
    let commits = vec![
        commit("a@zebra.io"),
        commit("b@zebra.io"),
        commit("c@zebra.io"), // 3
        commit("d@alpha.com"),
        commit("e@alpha.com"), // 2
        commit("f@beta.org"),  // 1
        commit("g@delta.net"), // 1
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains[0].domain, "zebra.io");
    assert_eq!(fp.domains[0].commits, 3);
    assert_eq!(fp.domains[1].domain, "alpha.com");
    assert_eq!(fp.domains[1].commits, 2);
    // Tie at 1 commit: alphabetical
    assert_eq!(fp.domains[2].domain, "beta.org");
    assert_eq!(fp.domains[3].domain, "delta.net");
}

// ===========================================================================
// Domain normalization: case-insensitive
// ===========================================================================

#[test]
fn mixed_case_domains_merge() {
    let commits = vec![
        commit("a@ACME.COM"),
        commit("b@Acme.Com"),
        commit("c@acme.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 3);
}

#[test]
fn public_domain_case_insensitive() {
    let commits = vec![commit("a@GMAIL.COM"), commit("b@Yahoo.Com")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
}

// ===========================================================================
// Determinism: same input always same output
// ===========================================================================

#[test]
fn determinism_across_100_runs() {
    let commits = vec![
        commit("a@acme.com"),
        commit("b@gmail.com"),
        commit("c@startup.io"),
        commit("d@acme.com"),
        commit("e@yahoo.com"),
        commit("bot@users.noreply.github.com"),
        commit("bad-email"),
    ];
    let reference = build_corporate_fingerprint(&commits);
    for _ in 0..100 {
        let result = build_corporate_fingerprint(&commits);
        assert_eq!(reference.domains.len(), result.domains.len());
        for (a, b) in reference.domains.iter().zip(result.domains.iter()) {
            assert_eq!(a.domain, b.domain);
            assert_eq!(a.commits, b.commits);
            assert!((a.pct - b.pct).abs() < f32::EPSILON);
        }
    }
}

// ===========================================================================
// Subdomain handling: full domain preserved
// ===========================================================================

#[test]
fn subdomains_are_distinct_from_parent() {
    let commits = vec![
        commit("a@eng.bigcorp.com"),
        commit("b@eng.bigcorp.com"),
        commit("c@bigcorp.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 2);
    let names: Vec<&str> = fp.domains.iter().map(|d| d.domain.as_str()).collect();
    assert!(names.contains(&"eng.bigcorp.com"));
    assert!(names.contains(&"bigcorp.com"));
}

// ===========================================================================
// Large input stress test
// ===========================================================================

#[test]
fn large_input_correct_aggregation() {
    let commits: Vec<GitCommit> = (0..500)
        .map(|i| commit(&format!("dev{}@company{}.com", i, i % 5)))
        .collect();
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 5);
    let total: u32 = fp.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 500);
    // Each company should have 100 commits
    for d in &fp.domains {
        assert_eq!(d.commits, 100);
    }
}

// ===========================================================================
// Single commit edge case
// ===========================================================================

#[test]
fn single_valid_commit() {
    let fp = build_corporate_fingerprint(&[commit("solo@tiny.org")]);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "tiny.org");
    assert_eq!(fp.domains[0].commits, 1);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

#[test]
fn single_ignored_commit() {
    let fp = build_corporate_fingerprint(&[commit("bot@users.noreply.github.com")]);
    assert!(fp.domains.is_empty());
}

#[test]
fn single_malformed_commit() {
    let fp = build_corporate_fingerprint(&[commit("no-at-sign")]);
    assert!(fp.domains.is_empty());
}
