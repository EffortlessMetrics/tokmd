//! Extended identity & security tests for corporate fingerprint detection.
//!
//! Covers gaps not exercised by existing unit/bdd/edge/property suites:
//! - Input ordering independence (permutation stability)
//! - Commits with populated hash, subject, and files fields
//! - Single-character and unusual domain formats
//! - Interaction between corporate and public domains at scale

use crate::fingerprint::build_corporate_fingerprint;
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

fn full_commit(author: &str, hash: &str, subject: &str, files: &[&str]) -> GitCommit {
    GitCommit {
        timestamp: 1_700_000_000,
        author: author.to_string(),
        hash: Some(hash.to_string()),
        subject: subject.to_string(),
        files: files.iter().map(|f| f.to_string()).collect(),
    }
}

// ===========================================================================
// 1. Input ordering independence — permutations yield identical output
// ===========================================================================

#[test]
fn ordering_of_commits_does_not_affect_fingerprint() {
    let commits_a = vec![
        commit("a@alpha.org"),
        commit("b@beta.org"),
        commit("c@alpha.org"),
        commit("d@gmail.com"),
    ];
    let commits_b = vec![
        commit("d@gmail.com"),
        commit("c@alpha.org"),
        commit("a@alpha.org"),
        commit("b@beta.org"),
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

// ===========================================================================
// 2. Hash, subject, and files do not affect fingerprint
// ===========================================================================

#[test]
fn hash_subject_files_do_not_affect_fingerprint() {
    let minimal = vec![commit("dev@corp.io"), commit("ops@corp.io")];

    let full = vec![
        full_commit("dev@corp.io", "abc123", "feat: add login", &["src/auth.rs"]),
        full_commit(
            "ops@corp.io",
            "def456",
            "fix: deploy script",
            &["scripts/deploy.sh", "k8s/pod.yaml"],
        ),
    ];

    let fp_min = build_corporate_fingerprint(&minimal);
    let fp_full = build_corporate_fingerprint(&full);

    assert_eq!(fp_min.domains.len(), fp_full.domains.len());
    for (a, b) in fp_min.domains.iter().zip(fp_full.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}

// ===========================================================================
// 3. Single-character domain parts
// ===========================================================================

#[test]
fn single_char_domain_is_counted() {
    let commits = vec![commit("user@x.y")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "x.y");
    assert_eq!(fp.domains[0].commits, 1);
}

#[test]
fn domain_without_tld_dot_is_still_counted() {
    // A domain like "intranet" without a dot is technically valid for git
    let commits = vec![commit("user@intranet")];
    let fp = build_corporate_fingerprint(&commits);
    // "intranet" is not in ignored list, not public, so should be counted
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "intranet");
}

// ===========================================================================
// 4. Domain with leading dot
// ===========================================================================

#[test]
fn domain_with_leading_dot_is_counted() {
    let commits = vec![commit("user@.weird.com")];
    let fp = build_corporate_fingerprint(&commits);
    // After lowercase + trim, ".weird.com" — not ignored, not public
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, ".weird.com");
}

// ===========================================================================
// 5. Large scale: many commits from same domain
// ===========================================================================

#[test]
fn thousand_commits_same_domain_pct_is_one() {
    let commits: Vec<GitCommit> = (0..1000)
        .map(|i| commit(&format!("dev{i}@megacorp.com")))
        .collect();
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "megacorp.com");
    assert_eq!(fp.domains[0].commits, 1000);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ===========================================================================
// 6. Mixed ignored and valid — ignored commits don't distort percentages
// ===========================================================================

#[test]
fn many_ignored_with_few_valid_percentages_correct() {
    let mut commits: Vec<GitCommit> = (0..50)
        .map(|i| commit(&format!("bot{i}@users.noreply.github.com")))
        .collect();
    commits.push(commit("dev@real.com"));
    commits.push(commit("ops@real.com"));
    commits.push(commit("user@other.com"));

    let fp = build_corporate_fingerprint(&commits);
    let total: u32 = fp.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 3, "only 3 non-ignored commits");

    let real = fp.domains.iter().find(|d| d.domain == "real.com").unwrap();
    assert_eq!(real.commits, 2);
    assert!((real.pct - 2.0 / 3.0).abs() < 0.01);
}

// ===========================================================================
// 7. All seven public providers with equal counts
// ===========================================================================

#[test]
fn all_public_providers_equal_counts_single_bucket() {
    let providers = [
        "gmail.com",
        "yahoo.com",
        "outlook.com",
        "hotmail.com",
        "icloud.com",
        "proton.me",
        "protonmail.com",
    ];
    let commits: Vec<GitCommit> = providers
        .iter()
        .flat_map(|p| (0..3).map(move |i| commit(&format!("user{i}@{p}"))))
        .collect();

    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 21); // 7 providers × 3 each
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ===========================================================================
// 8. Determinism across multiple calls with same data
// ===========================================================================

#[test]
fn ten_consecutive_calls_produce_identical_results() {
    let commits = vec![
        commit("a@foo.com"),
        commit("b@bar.io"),
        commit("c@foo.com"),
        commit("d@gmail.com"),
        commit("e@bar.io"),
    ];

    let baseline = build_corporate_fingerprint(&commits);
    for _ in 0..10 {
        let fp = build_corporate_fingerprint(&commits);
        assert_eq!(fp.domains.len(), baseline.domains.len());
        for (a, b) in fp.domains.iter().zip(baseline.domains.iter()) {
            assert_eq!(a.domain, b.domain);
            assert_eq!(a.commits, b.commits);
            assert!((a.pct - b.pct).abs() < f32::EPSILON);
        }
    }
}

// ===========================================================================
// 9. Verify JSON serialization stability
// ===========================================================================

#[test]
fn json_serialization_is_deterministic() {
    let commits = vec![
        commit("a@corp.com"),
        commit("b@corp.com"),
        commit("c@gmail.com"),
    ];

    let fp1 = build_corporate_fingerprint(&commits);
    let fp2 = build_corporate_fingerprint(&commits);
    let json1 = serde_json::to_string(&fp1).unwrap();
    let json2 = serde_json::to_string(&fp2).unwrap();
    assert_eq!(json1, json2, "JSON serialization must be deterministic");
}

// ===========================================================================
// 10. Domain that looks like a noreply variant but isn't
// ===========================================================================

#[test]
fn similar_to_noreply_but_different_is_counted() {
    let commits = vec![commit("user@noreply.gitlab.com")];
    let fp = build_corporate_fingerprint(&commits);
    // "noreply.gitlab.com" does NOT contain "noreply.github.com"
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "noreply.gitlab.com");
}

// ===========================================================================
// 11. Only-whitespace domain after trimming is ignored
// ===========================================================================

#[test]
fn whitespace_only_domain_is_ignored() {
    let commits = vec![commit("user@   ")];
    let fp = build_corporate_fingerprint(&commits);
    // After trim + lowercase, domain is empty → ignored
    assert!(fp.domains.is_empty());
}

// ===========================================================================
// 12. Empty input yields empty fingerprint
// ===========================================================================

#[test]
fn empty_commits_yields_empty_fingerprint() {
    let fp = build_corporate_fingerprint(&[]);
    assert!(fp.domains.is_empty());
}
