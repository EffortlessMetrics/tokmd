//! W75 security & identity tests for corporate fingerprint detection.
//!
//! Focuses on:
//! - Corporate domain detection and bucketing
//! - Public email provider merging
//! - Ignored domain filtering (noreply, localhost, example.com)
//! - Commit pattern analysis (varied authors)
//! - File pattern metadata (hash, subject, files fields)
//! - Percentage accuracy and structural invariants

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

fn rich_commit(author: &str, hash: &str, subject: &str, files: &[&str]) -> GitCommit {
    GitCommit {
        timestamp: 1_700_000_000,
        author: author.to_string(),
        hash: Some(hash.to_string()),
        subject: subject.to_string(),
        files: files.iter().map(|s| s.to_string()).collect(),
    }
}

// ===========================================================================
// 1. Single corporate domain produces 100% share
// ===========================================================================

#[test]
fn single_corporate_domain_100_pct() {
    let commits = vec![
        commit("alice@acme.com"),
        commit("bob@acme.com"),
        commit("carol@acme.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);

    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 3);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ===========================================================================
// 2. Multiple corporate domains sorted by commit count
// ===========================================================================

#[test]
fn multiple_corporate_domains_sorted_by_commits() {
    let commits = vec![
        commit("a@big.co"),
        commit("b@big.co"),
        commit("c@big.co"),
        commit("d@big.co"),
        commit("e@small.co"),
    ];
    let fp = build_corporate_fingerprint(&commits);

    assert_eq!(fp.domains[0].domain, "big.co");
    assert_eq!(fp.domains[0].commits, 4);
    assert_eq!(fp.domains[1].domain, "small.co");
    assert_eq!(fp.domains[1].commits, 1);
}

// ===========================================================================
// 3. Public email providers merged into single bucket
// ===========================================================================

#[test]
fn public_providers_merged_into_single_bucket() {
    let commits = vec![
        commit("a@gmail.com"),
        commit("b@yahoo.com"),
        commit("c@outlook.com"),
        commit("d@hotmail.com"),
        commit("e@icloud.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);

    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 5);
}

// ===========================================================================
// 4. Noreply GitHub addresses ignored
// ===========================================================================

#[test]
fn noreply_github_addresses_ignored() {
    let commits = vec![
        commit("user@users.noreply.github.com"),
        commit("12345+user@users.noreply.github.com"),
        commit("dev@real.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);

    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "real.com");
    assert_eq!(fp.domains[0].commits, 1);
}

// ===========================================================================
// 5. localhost and example.com ignored
// ===========================================================================

#[test]
fn localhost_and_example_ignored() {
    let commits = vec![
        commit("bot@localhost"),
        commit("test@example.com"),
        commit("dev@corp.io"),
    ];
    let fp = build_corporate_fingerprint(&commits);

    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "corp.io");
}

// ===========================================================================
// 6. Rich commit metadata does not affect fingerprint
// ===========================================================================

#[test]
fn rich_commit_fields_do_not_affect_result() {
    let simple = vec![commit("dev@work.com"), commit("ops@work.com")];
    let rich = vec![
        rich_commit(
            "dev@work.com",
            "abc123",
            "feat: new feature",
            &["src/lib.rs"],
        ),
        rich_commit(
            "ops@work.com",
            "def456",
            "fix: bug fix",
            &["src/main.rs", "tests/t.rs"],
        ),
    ];

    let fp_simple = build_corporate_fingerprint(&simple);
    let fp_rich = build_corporate_fingerprint(&rich);

    assert_eq!(fp_simple.domains.len(), fp_rich.domains.len());
    for (a, b) in fp_simple.domains.iter().zip(fp_rich.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
    }
}

// ===========================================================================
// 7. Mixed corporate and public domains with correct percentages
// ===========================================================================

#[test]
fn mixed_corporate_and_public_percentages_correct() {
    let commits = vec![
        commit("a@corp.io"),
        commit("b@corp.io"),
        commit("c@corp.io"),
        commit("d@gmail.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);

    let corp = fp.domains.iter().find(|d| d.domain == "corp.io").unwrap();
    let public = fp
        .domains
        .iter()
        .find(|d| d.domain == "public-email")
        .unwrap();

    assert_eq!(corp.commits, 3);
    assert!((corp.pct - 0.75).abs() < f32::EPSILON);
    assert_eq!(public.commits, 1);
    assert!((public.pct - 0.25).abs() < f32::EPSILON);
}

// ===========================================================================
// 8. Domain normalization: uppercase treated as lowercase
// ===========================================================================

#[test]
fn domain_case_normalized() {
    let commits = vec![
        commit("a@CORP.COM"),
        commit("b@Corp.Com"),
        commit("c@corp.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);

    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "corp.com");
    assert_eq!(fp.domains[0].commits, 3);
}

// ===========================================================================
// 9. Author without @ sign skipped
// ===========================================================================

#[test]
fn author_without_at_skipped() {
    let commits = vec![commit("no-email-here"), commit("dev@real.com")];
    let fp = build_corporate_fingerprint(&commits);

    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "real.com");
}

// ===========================================================================
// 10. Empty commits yields empty fingerprint
// ===========================================================================

#[test]
fn empty_commits_empty_fingerprint() {
    let fp = build_corporate_fingerprint(&[]);
    assert!(fp.domains.is_empty());
}
