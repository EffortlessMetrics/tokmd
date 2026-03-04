//! Wave-49 deep tests for corporate fingerprint analysis.
//!
//! Covers domain bucketing, percentage calculation, sorting,
//! ignored domains, serde roundtrips, and property-based tests.

use tokmd_analysis_fingerprint::build_corporate_fingerprint;
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

// ── 1. Empty commits returns empty domains ──────────────────────

#[test]
fn empty_commits_empty_domains() {
    let report = build_corporate_fingerprint(&[]);
    assert!(report.domains.is_empty());
}

// ── 2. Single corporate domain ──────────────────────────────────

#[test]
fn single_corporate_domain() {
    let commits = vec![commit("alice@acme.com"), commit("bob@acme.com")];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "acme.com");
    assert_eq!(report.domains[0].commits, 2);
    assert!((report.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── 3. Public domains bucketed as "public-email" ────────────────

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
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "public-email");
    assert_eq!(report.domains[0].commits, 7);
}

// ── 4. Mixed public and corporate ───────────────────────────────

#[test]
fn mixed_public_and_corporate() {
    let commits = vec![
        commit("a@gmail.com"),
        commit("b@acme.com"),
        commit("c@acme.com"),
        commit("d@acme.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 2);
    // acme.com has 3 commits → sorted first
    assert_eq!(report.domains[0].domain, "acme.com");
    assert_eq!(report.domains[0].commits, 3);
    assert_eq!(report.domains[1].domain, "public-email");
    assert_eq!(report.domains[1].commits, 1);
}

// ── 5. Ignored domains filtered ─────────────────────────────────

#[test]
fn ignored_domains_filtered() {
    let commits = vec![
        commit("bot@localhost"),
        commit("bot@example.com"),
        commit("bot@users.noreply.github.com"),
        commit("bot@noreply.github.com"),
        commit("real@acme.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "acme.com");
    assert_eq!(report.domains[0].commits, 1);
}

// ── 6. Percentage calculation ───────────────────────────────────

#[test]
fn percentage_calculation() {
    let commits = vec![
        commit("a@corp.io"),
        commit("b@corp.io"),
        commit("c@startup.dev"),
    ];
    let report = build_corporate_fingerprint(&commits);
    let corp = report
        .domains
        .iter()
        .find(|d| d.domain == "corp.io")
        .unwrap();
    let startup = report
        .domains
        .iter()
        .find(|d| d.domain == "startup.dev")
        .unwrap();
    // 2/3 ≈ 0.6667
    assert!((corp.pct - 2.0 / 3.0).abs() < 0.001);
    // 1/3 ≈ 0.3333
    assert!((startup.pct - 1.0 / 3.0).abs() < 0.001);
}

// ── 7. Sorting: commits desc, domain asc ────────────────────────

#[test]
fn sorting_commits_desc_domain_asc() {
    let commits = vec![
        commit("a@beta.com"),
        commit("b@alpha.com"),
        commit("c@beta.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    // beta.com: 2 commits, alpha.com: 1 commit
    assert_eq!(report.domains[0].domain, "beta.com");
    assert_eq!(report.domains[1].domain, "alpha.com");

    // With tied commits → alphabetical
    let commits2 = vec![commit("a@beta.com"), commit("b@alpha.com")];
    let report2 = build_corporate_fingerprint(&commits2);
    // Both have 1 commit → sorted alphabetically
    assert_eq!(report2.domains[0].domain, "alpha.com");
    assert_eq!(report2.domains[1].domain, "beta.com");
}

// ── 8. No @ sign → commit skipped ──────────────────────────────

#[test]
fn no_at_sign_skipped() {
    let commits = vec![
        commit("no-email-here"),
        commit("also-no-email"),
        commit("real@corp.io"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "corp.io");
}

// ── 9. Domain normalization (case + whitespace) ─────────────────

#[test]
fn domain_normalization() {
    let commits = vec![
        commit("a@ACME.COM"),
        commit("b@Acme.Com"),
        commit("c@acme.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "acme.com");
    assert_eq!(report.domains[0].commits, 3);
}

// ── 10. Serde roundtrip preserves all fields ────────────────────

#[test]
fn serde_roundtrip_preserves_all_fields() {
    let commits = vec![
        commit("a@acme.com"),
        commit("b@gmail.com"),
        commit("c@startup.dev"),
    ];
    let report = build_corporate_fingerprint(&commits);

    let json = serde_json::to_string(&report).unwrap();
    let deser: CorporateFingerprint = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.domains.len(), report.domains.len());
    for (orig, rt) in report.domains.iter().zip(deser.domains.iter()) {
        assert_eq!(orig.domain, rt.domain);
        assert_eq!(orig.commits, rt.commits);
        assert!((orig.pct - rt.pct).abs() < f32::EPSILON);
    }
}

// ── 11. Multiple @ signs → skipped ──────────────────────────────

#[test]
fn multiple_at_signs_skipped() {
    let commits = vec![commit("user@host@extra"), commit("real@corp.io")];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "corp.io");
}

// ── 12. Deterministic across calls ──────────────────────────────

#[test]
fn deterministic_across_calls() {
    let commits = vec![
        commit("a@corp.io"),
        commit("b@startup.dev"),
        commit("c@gmail.com"),
    ];
    let r1 = build_corporate_fingerprint(&commits);
    let r2 = build_corporate_fingerprint(&commits);
    assert_eq!(r1.domains.len(), r2.domains.len());
    for (a, b) in r1.domains.iter().zip(r2.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
    }
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_commit() -> impl Strategy<Value = GitCommit> {
        "[a-z]{3,8}@[a-z]{3,8}\\.(com|io|dev)".prop_map(|author| GitCommit {
            timestamp: 0,
            author,
            hash: None,
            subject: String::new(),
            files: vec![],
        })
    }

    proptest! {
        #[test]
        fn pct_sum_lte_one(commits in proptest::collection::vec(arb_commit(), 1..30)) {
            let report = build_corporate_fingerprint(&commits);
            let total_pct: f32 = report.domains.iter().map(|d| d.pct).sum();
            // Sum should be approximately 1.0 (accounting for float rounding)
            prop_assert!(total_pct <= 1.01, "pct sum should be ~1.0, got {total_pct}");
            prop_assert!(total_pct >= 0.99, "pct sum should be ~1.0, got {total_pct}");
            for d in &report.domains {
                prop_assert!(d.pct >= 0.0 && d.pct <= 1.0, "pct should be in [0,1]: {}", d.pct);
                prop_assert!(d.commits >= 1, "each domain must have >= 1 commit");
            }
        }
    }
}
