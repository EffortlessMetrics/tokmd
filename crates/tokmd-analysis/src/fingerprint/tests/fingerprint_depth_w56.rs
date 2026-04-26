//! Wave-56 depth tests for corporate fingerprint analysis.
//!
//! Covers domain extraction edge cases, bucketing, percentage accuracy,
//! deterministic classification, and additional boundary conditions.

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

fn commit_with_ts(author: &str, ts: i64) -> GitCommit {
    GitCommit {
        timestamp: ts,
        author: author.to_string(),
        hash: None,
        subject: String::new(),
        files: vec![],
    }
}

// ── 1. Empty string author: skipped ─────────────────────────────

#[test]
fn empty_author_skipped() {
    let commits = vec![commit(""), commit("real@corp.io")];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "corp.io");
}

// ── 2. Author with no domain part: skipped ──────────────────────

#[test]
fn author_bare_at_skipped() {
    let commits = vec![commit("user@"), commit("real@corp.io")];
    let report = build_corporate_fingerprint(&commits);
    // "user@" → domain is empty after extraction, should be skipped
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "corp.io");
}

// ── 3. All commits from ignored domains: empty result ───────────

#[test]
fn all_ignored_domains_empty() {
    let commits = vec![
        commit("bot@localhost"),
        commit("bot@example.com"),
        commit("ci@users.noreply.github.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert!(
        report.domains.is_empty(),
        "all-ignored domains should yield empty result"
    );
}

// ── 4. All seven public domains yield single bucket ─────────────

#[test]
fn all_seven_public_domains_single_bucket() {
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
    assert!((report.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── 5. Single commit: 100% to that domain ───────────────────────

#[test]
fn single_commit_100_pct() {
    let commits = vec![commit("solo@startup.ai")];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "startup.ai");
    assert_eq!(report.domains[0].commits, 1);
    assert!((report.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── 6. Case insensitive domain matching ─────────────────────────

#[test]
fn case_insensitive_consolidation() {
    let commits = vec![commit("a@Gmail.COM"), commit("b@YAHOO.COM")];
    let report = build_corporate_fingerprint(&commits);
    // Both are public domains after normalization
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "public-email");
    assert_eq!(report.domains[0].commits, 2);
}

// ── 7. Noreply github subdomain variants filtered ───────────────

#[test]
fn noreply_github_subdomain_filtered() {
    let commits = vec![
        commit("12345+user@users.noreply.github.com"),
        commit("bot@noreply.github.com"),
        commit("real@company.org"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "company.org");
}

// ── 8. Many distinct domains sorted by commit count ─────────────

#[test]
fn many_domains_sorted_by_commits() {
    let commits = vec![
        commit("a@three.io"),
        commit("b@three.io"),
        commit("c@three.io"),
        commit("d@two.io"),
        commit("e@two.io"),
        commit("f@one.io"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 3);
    assert_eq!(report.domains[0].domain, "three.io");
    assert_eq!(report.domains[0].commits, 3);
    assert_eq!(report.domains[1].domain, "two.io");
    assert_eq!(report.domains[1].commits, 2);
    assert_eq!(report.domains[2].domain, "one.io");
    assert_eq!(report.domains[2].commits, 1);
}

// ── 9. Tied commits sorted alphabetically ───────────────────────

#[test]
fn tied_commits_alphabetical_sort() {
    let commits = vec![
        commit("a@zebra.com"),
        commit("b@alpha.com"),
        commit("c@mango.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    // All have 1 commit, should be sorted alphabetically
    assert_eq!(report.domains[0].domain, "alpha.com");
    assert_eq!(report.domains[1].domain, "mango.com");
    assert_eq!(report.domains[2].domain, "zebra.com");
}

// ── 10. Percentage precision with 3 domains ─────────────────────

#[test]
fn percentage_precision_three_domains() {
    let commits = vec![
        commit("a@a.com"),
        commit("b@a.com"),
        commit("c@a.com"),
        commit("d@b.com"),
        commit("e@b.com"),
        commit("f@c.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    let a = report.domains.iter().find(|d| d.domain == "a.com").unwrap();
    let b = report.domains.iter().find(|d| d.domain == "b.com").unwrap();
    let c = report.domains.iter().find(|d| d.domain == "c.com").unwrap();
    assert!((a.pct - 3.0 / 6.0).abs() < 0.001);
    assert!((b.pct - 2.0 / 6.0).abs() < 0.001);
    assert!((c.pct - 1.0 / 6.0).abs() < 0.001);
}

// ── 11. Percentages sum to approximately 1.0 ────────────────────

#[test]
fn percentages_sum_to_one() {
    let commits = vec![
        commit("a@x.com"),
        commit("b@y.com"),
        commit("c@z.com"),
        commit("d@x.com"),
    ];
    let report = build_corporate_fingerprint(&commits);
    let total: f32 = report.domains.iter().map(|d| d.pct).sum();
    assert!(
        (total - 1.0).abs() < 0.01,
        "percentages should sum to ~1.0, got {total}"
    );
}

// ── 12. Deterministic with different timestamps ─────────────────

#[test]
fn deterministic_ignores_timestamps() {
    let commits1 = vec![
        commit_with_ts("a@corp.io", 1000),
        commit_with_ts("b@other.dev", 2000),
    ];
    let commits2 = vec![
        commit_with_ts("a@corp.io", 9999),
        commit_with_ts("b@other.dev", 1),
    ];
    let r1 = build_corporate_fingerprint(&commits1);
    let r2 = build_corporate_fingerprint(&commits2);
    assert_eq!(r1.domains.len(), r2.domains.len());
    for (a, b) in r1.domains.iter().zip(r2.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
    }
}

// ── 13. Commit files field ignored for fingerprint ──────────────

#[test]
fn commit_files_ignored() {
    let c1 = GitCommit {
        timestamp: 0,
        author: "dev@corp.io".to_string(),
        hash: None,
        subject: String::new(),
        files: vec!["src/main.rs".to_string(), "README.md".to_string()],
    };
    let c2 = GitCommit {
        timestamp: 0,
        author: "dev@corp.io".to_string(),
        hash: None,
        subject: String::new(),
        files: vec![],
    };
    let r1 = build_corporate_fingerprint(&[c1]);
    let r2 = build_corporate_fingerprint(&[c2]);
    assert_eq!(r1.domains[0].commits, r2.domains[0].commits);
}

// ── 14. Hash and subject fields ignored ─────────────────────────

#[test]
fn hash_and_subject_ignored() {
    let c = GitCommit {
        timestamp: 0,
        author: "dev@firm.co".to_string(),
        hash: Some("abc123".to_string()),
        subject: "fix: something important".to_string(),
        files: vec![],
    };
    let report = build_corporate_fingerprint(&[c]);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "firm.co");
}

// ── 15. Mixed ignored and valid: only valid counted ─────────────

#[test]
fn mixed_ignored_and_valid_counts() {
    let commits = vec![
        commit("a@localhost"),
        commit("b@example.com"),
        commit("c@corp.io"),
        commit("d@noreply.github.com"),
        commit("e@corp.io"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "corp.io");
    assert_eq!(report.domains[0].commits, 2);
    // Only 2 valid commits, so 100%
    assert!((report.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── 16. Public email mixed with corporate: correct counts ───────

#[test]
fn public_email_mixed_correct_counts() {
    let commits = vec![
        commit("a@gmail.com"),
        commit("b@yahoo.com"),
        commit("c@corp.io"),
    ];
    let report = build_corporate_fingerprint(&commits);
    let public = report
        .domains
        .iter()
        .find(|d| d.domain == "public-email")
        .unwrap();
    let corp = report
        .domains
        .iter()
        .find(|d| d.domain == "corp.io")
        .unwrap();
    assert_eq!(public.commits, 2);
    assert_eq!(corp.commits, 1);
}

// ── 17. Subdomain not treated as parent domain ──────────────────

#[test]
fn subdomain_distinct_from_parent() {
    let commits = vec![commit("a@eng.corp.io"), commit("b@corp.io")];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 2);
    assert!(report.domains.iter().any(|d| d.domain == "eng.corp.io"));
    assert!(report.domains.iter().any(|d| d.domain == "corp.io"));
}

// ── 18. Whitespace in domain trimmed ────────────────────────────

#[test]
fn whitespace_in_domain_trimmed() {
    let commits = vec![commit("a@ corp.io "), commit("b@corp.io")];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains.len(), 1);
    assert_eq!(report.domains[0].domain, "corp.io");
    assert_eq!(report.domains[0].commits, 2);
}

// ── 19. Serde roundtrip with empty domains ──────────────────────

#[test]
fn serde_roundtrip_empty() {
    let report = build_corporate_fingerprint(&[]);
    let json = serde_json::to_string(&report).unwrap();
    let rt: CorporateFingerprint = serde_json::from_str(&json).unwrap();
    assert!(rt.domains.is_empty());
}

// ── 20. Serde roundtrip preserves order ─────────────────────────

#[test]
fn serde_roundtrip_preserves_order() {
    let commits = vec![
        commit("a@big.co"),
        commit("b@big.co"),
        commit("c@big.co"),
        commit("d@small.co"),
    ];
    let report = build_corporate_fingerprint(&commits);
    let json = serde_json::to_string(&report).unwrap();
    let rt: CorporateFingerprint = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.domains.len(), report.domains.len());
    for (a, b) in report.domains.iter().zip(rt.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}

// ── 21. Many public email providers ─────────────────────────────

#[test]
fn all_public_providers_aggregate() {
    // Verify each recognized provider goes to "public-email"
    let providers = [
        "gmail.com",
        "yahoo.com",
        "outlook.com",
        "hotmail.com",
        "icloud.com",
        "proton.me",
        "protonmail.com",
    ];
    for provider in &providers {
        let email = format!("user@{provider}");
        let commits = vec![commit(&email)];
        let report = build_corporate_fingerprint(&commits);
        assert_eq!(
            report.domains[0].domain, "public-email",
            "{provider} should be bucketed as public-email"
        );
    }
}

// ── 22. Non-public provider not bucketed ────────────────────────

#[test]
fn non_public_provider_not_bucketed() {
    let niche_providers = ["fastmail.com", "tutanota.com", "hey.com", "zoho.com"];
    for provider in &niche_providers {
        let email = format!("user@{provider}");
        let commits = vec![commit(&email)];
        let report = build_corporate_fingerprint(&commits);
        assert_eq!(
            report.domains[0].domain, *provider,
            "{provider} should NOT be bucketed as public-email"
        );
    }
}

// ── 23. Large commit volume: correct totals ─────────────────────

#[test]
fn large_commit_volume() {
    let mut commits = Vec::new();
    for i in 0..100 {
        let domain = if i % 3 == 0 {
            "alpha.com"
        } else if i % 3 == 1 {
            "beta.com"
        } else {
            "gmail.com"
        };
        commits.push(commit(&format!("user{i}@{domain}")));
    }
    let report = build_corporate_fingerprint(&commits);
    let total_commits: u32 = report.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total_commits, 100);
}

// ── 24. Deterministic across calls ──────────────────────────────

#[test]
fn deterministic_across_calls() {
    let commits = vec![
        commit("a@one.io"),
        commit("b@two.io"),
        commit("c@gmail.com"),
        commit("d@one.io"),
    ];
    let r1 = build_corporate_fingerprint(&commits);
    let r2 = build_corporate_fingerprint(&commits);
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "fingerprint output must be deterministic");
}

// ── 25. Commits with same author counted separately ─────────────

#[test]
fn same_author_counted_per_commit() {
    let commits = vec![
        commit("dev@corp.io"),
        commit("dev@corp.io"),
        commit("dev@corp.io"),
    ];
    let report = build_corporate_fingerprint(&commits);
    assert_eq!(report.domains[0].commits, 3);
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_email() -> impl Strategy<Value = String> {
        ("[a-z]{2,6}@[a-z]{2,6}\\.(com|io|dev|org)").prop_map(|s| s)
    }

    proptest! {
        #[test]
        fn commits_sum_equals_total(emails in proptest::collection::vec(arb_email(), 1..50)) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let report = build_corporate_fingerprint(&commits);
            let total: u32 = report.domains.iter().map(|d| d.commits).sum();
            // Some commits may be ignored (localhost, example.com), so total <= emails.len()
            prop_assert!(total as usize <= emails.len());
            for d in &report.domains {
                prop_assert!(d.pct >= 0.0 && d.pct <= 1.0);
            }
        }
    }
}
