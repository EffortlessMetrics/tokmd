//! Deep W68 tests for corporate fingerprint computation.
//!
//! Covers: domain extraction, public-email bucketing, ignored domains,
//! normalization, deterministic output, sorting, percentage accuracy,
//! edge cases (empty, single commit, no @), and serde round-trips.

use crate::fingerprint::build_corporate_fingerprint;
use tokmd_analysis_types::CorporateFingerprint;
use tokmd_git::GitCommit;

// ── Helpers ─────────────────────────────────────────────────────

fn commit(author: &str) -> GitCommit {
    GitCommit {
        timestamp: 0,
        author: author.to_string(),
        hash: None,
        subject: String::new(),
        files: vec![],
    }
}

// ── 1. Empty commits produce empty fingerprint ──────────────────

#[test]
fn empty_commits_empty_fingerprint() {
    let fp = build_corporate_fingerprint(&[]);
    assert!(fp.domains.is_empty());
}

// ── 2. Single corporate domain ──────────────────────────────────

#[test]
fn single_corporate_domain() {
    let fp = build_corporate_fingerprint(&[commit("dev@acme.com")]);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 1);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── 3. Public domains bucketed together ─────────────────────────

#[test]
fn public_domains_bucketed() {
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
}

// ── 4. Ignored: noreply.github.com ──────────────────────────────

#[test]
fn noreply_github_ignored() {
    let fp = build_corporate_fingerprint(&[commit("bot@users.noreply.github.com")]);
    assert!(fp.domains.is_empty());
}

// ── 5. Ignored: localhost ───────────────────────────────────────

#[test]
fn localhost_ignored() {
    let fp = build_corporate_fingerprint(&[commit("dev@localhost")]);
    assert!(fp.domains.is_empty());
}

// ── 6. Ignored: example.com ─────────────────────────────────────

#[test]
fn example_com_ignored() {
    let fp = build_corporate_fingerprint(&[commit("test@example.com")]);
    assert!(fp.domains.is_empty());
}

// ── 7. No @ sign skipped ────────────────────────────────────────

#[test]
fn no_at_sign_skipped() {
    let fp = build_corporate_fingerprint(&[commit("noemail")]);
    assert!(fp.domains.is_empty());
}

// ── 8. Multiple @ signs skipped ─────────────────────────────────

#[test]
fn multiple_at_signs_skipped() {
    let fp = build_corporate_fingerprint(&[commit("bad@@domain.com")]);
    assert!(fp.domains.is_empty());
}

// ── 9. Domain normalization lowercase ───────────────────────────

#[test]
fn domain_normalized_lowercase() {
    let fp = build_corporate_fingerprint(&[commit("a@ACME.COM"), commit("b@Acme.Com")]);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 2);
}

// ── 10. Sorting by commit count descending ──────────────────────

#[test]
fn sorted_by_commits_descending() {
    let commits = vec![
        commit("a@small.co"),
        commit("b@big.co"),
        commit("c@big.co"),
        commit("d@big.co"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains[0].domain, "big.co");
    assert_eq!(fp.domains[1].domain, "small.co");
}

// ── 11. Tie-breaking by domain name alphabetically ──────────────

#[test]
fn tie_breaking_alphabetical() {
    let commits = vec![commit("a@beta.com"), commit("b@alpha.com")];
    let fp = build_corporate_fingerprint(&commits);
    // Both have 1 commit, sorted alphabetically
    assert_eq!(fp.domains[0].domain, "alpha.com");
    assert_eq!(fp.domains[1].domain, "beta.com");
}

// ── 12. Percentage accuracy ─────────────────────────────────────

#[test]
fn percentage_accuracy() {
    let commits = vec![
        commit("a@corp.io"),
        commit("b@corp.io"),
        commit("c@other.io"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    let corp = fp.domains.iter().find(|d| d.domain == "corp.io").unwrap();
    let other = fp.domains.iter().find(|d| d.domain == "other.io").unwrap();
    assert!((corp.pct - 2.0 / 3.0).abs() < 0.001);
    assert!((other.pct - 1.0 / 3.0).abs() < 0.001);
}

// ── 13. Mixed corporate and public ──────────────────────────────

#[test]
fn mixed_corporate_and_public() {
    let commits = vec![
        commit("a@corp.com"),
        commit("b@corp.com"),
        commit("c@gmail.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 2);
    assert!(fp.domains.iter().any(|d| d.domain == "corp.com"));
    assert!(fp.domains.iter().any(|d| d.domain == "public-email"));
}

// ── 14. Deterministic output ────────────────────────────────────

#[test]
fn deterministic_output() {
    let commits = vec![
        commit("a@x.com"),
        commit("b@y.com"),
        commit("c@z.com"),
        commit("d@gmail.com"),
    ];
    let fp1 = build_corporate_fingerprint(&commits);
    let fp2 = build_corporate_fingerprint(&commits);
    assert_eq!(fp1.domains.len(), fp2.domains.len());
    for (a, b) in fp1.domains.iter().zip(fp2.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
    }
}

// ── 15. Serde round-trip for CorporateFingerprint ───────────────

#[test]
fn serde_round_trip() {
    let commits = vec![commit("a@corp.com"), commit("b@gmail.com")];
    let fp = build_corporate_fingerprint(&commits);
    let json = serde_json::to_string(&fp).unwrap();
    let deserialized: CorporateFingerprint = serde_json::from_str(&json).unwrap();
    assert_eq!(fp.domains.len(), deserialized.domains.len());
    for (a, b) in fp.domains.iter().zip(deserialized.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}
