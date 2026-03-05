//! Wave-61 depth tests for corporate fingerprint analysis.
//!
//! Covers domain extraction edge cases, public-email bucketing,
//! ignored-domain filtering, percentage precision, sort stability,
//! determinism, serde round-trips, and proptest properties.

use proptest::prelude::*;
use tokmd_analysis_fingerprint::build_corporate_fingerprint;
use tokmd_analysis_types::{CorporateFingerprint, DomainStat};
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

fn commits_from(authors: &[&str]) -> Vec<GitCommit> {
    authors.iter().map(|a| commit(a)).collect()
}

fn fp(authors: &[&str]) -> CorporateFingerprint {
    build_corporate_fingerprint(&commits_from(authors))
}

fn find_domain<'a>(fp: &'a CorporateFingerprint, name: &str) -> Option<&'a DomainStat> {
    fp.domains.iter().find(|d| d.domain == name)
}

// ═══════════════════════════════════════════════════════════════
// 1–5  Empty / no-op inputs
// ═══════════════════════════════════════════════════════════════

#[test]
fn empty_commits_yields_empty_domains() {
    let r = build_corporate_fingerprint(&[]);
    assert!(r.domains.is_empty());
}

#[test]
fn single_empty_author_yields_empty() {
    let r = fp(&[""]);
    assert!(r.domains.is_empty());
}

#[test]
fn no_at_sign_skipped() {
    let r = fp(&["not-an-email"]);
    assert!(r.domains.is_empty());
}

#[test]
fn multiple_at_signs_skipped() {
    let r = fp(&["a@b@c.com"]);
    assert!(r.domains.is_empty());
}

#[test]
fn trailing_at_empty_domain_skipped() {
    let r = fp(&["user@"]);
    assert!(r.domains.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 6–10  Ignored domain filtering
// ═══════════════════════════════════════════════════════════════

#[test]
fn localhost_ignored() {
    let r = fp(&["bot@localhost"]);
    assert!(r.domains.is_empty());
}

#[test]
fn example_com_ignored() {
    let r = fp(&["test@example.com"]);
    assert!(r.domains.is_empty());
}

#[test]
fn noreply_github_ignored() {
    let r = fp(&["bot@noreply.github.com"]);
    assert!(r.domains.is_empty());
}

#[test]
fn users_noreply_github_ignored() {
    let r = fp(&["12345+user@users.noreply.github.com"]);
    assert!(r.domains.is_empty());
}

#[test]
fn all_ignored_domains_together() {
    let r = fp(&[
        "a@localhost",
        "b@example.com",
        "c@users.noreply.github.com",
        "d@noreply.github.com",
    ]);
    assert!(r.domains.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// 11–17  Public email bucketing (all 7 providers)
// ═══════════════════════════════════════════════════════════════

#[test]
fn gmail_bucketed_as_public() {
    let r = fp(&["a@gmail.com"]);
    assert_eq!(r.domains.len(), 1);
    assert_eq!(r.domains[0].domain, "public-email");
}

#[test]
fn yahoo_bucketed_as_public() {
    let r = fp(&["a@yahoo.com"]);
    assert_eq!(r.domains[0].domain, "public-email");
}

#[test]
fn outlook_bucketed_as_public() {
    let r = fp(&["a@outlook.com"]);
    assert_eq!(r.domains[0].domain, "public-email");
}

#[test]
fn hotmail_bucketed_as_public() {
    let r = fp(&["a@hotmail.com"]);
    assert_eq!(r.domains[0].domain, "public-email");
}

#[test]
fn icloud_bucketed_as_public() {
    let r = fp(&["a@icloud.com"]);
    assert_eq!(r.domains[0].domain, "public-email");
}

#[test]
fn proton_me_bucketed_as_public() {
    let r = fp(&["a@proton.me"]);
    assert_eq!(r.domains[0].domain, "public-email");
}

#[test]
fn protonmail_bucketed_as_public() {
    let r = fp(&["a@protonmail.com"]);
    assert_eq!(r.domains[0].domain, "public-email");
}

// ═══════════════════════════════════════════════════════════════
// 18–22  Corporate domain handling
// ═══════════════════════════════════════════════════════════════

#[test]
fn single_corporate_domain() {
    let r = fp(&["dev@acme.com"]);
    assert_eq!(r.domains.len(), 1);
    assert_eq!(r.domains[0].domain, "acme.com");
    assert_eq!(r.domains[0].commits, 1);
    assert!((r.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

#[test]
fn niche_provider_not_bucketed_as_public() {
    for provider in &["fastmail.com", "tutanota.com", "hey.com", "zoho.com"] {
        let r = fp(&[&format!("u@{provider}")]);
        assert_eq!(
            r.domains[0].domain, *provider,
            "{provider} should NOT be public-email"
        );
    }
}

#[test]
fn hyphenated_domain() {
    let r = fp(&["a@my-company.co.uk"]);
    assert_eq!(r.domains[0].domain, "my-company.co.uk");
}

#[test]
fn subdomain_distinct_from_parent() {
    let r = fp(&["a@eng.corp.io", "b@corp.io"]);
    assert_eq!(r.domains.len(), 2);
    assert!(find_domain(&r, "eng.corp.io").is_some());
    assert!(find_domain(&r, "corp.io").is_some());
}

#[test]
fn numeric_tld() {
    let r = fp(&["u@company.123"]);
    assert_eq!(r.domains[0].domain, "company.123");
}

// ═══════════════════════════════════════════════════════════════
// 23–27  Case normalisation & dedup
// ═══════════════════════════════════════════════════════════════

#[test]
fn mixed_case_consolidated() {
    let r = fp(&["a@Acme.COM", "b@acme.com", "c@ACME.Com"]);
    assert_eq!(r.domains.len(), 1);
    assert_eq!(r.domains[0].domain, "acme.com");
    assert_eq!(r.domains[0].commits, 3);
}

#[test]
fn uppercase_public_domain_bucketed() {
    let r = fp(&["a@GMAIL.COM", "b@Yahoo.COM"]);
    assert_eq!(r.domains.len(), 1);
    assert_eq!(r.domains[0].domain, "public-email");
    assert_eq!(r.domains[0].commits, 2);
}

#[test]
fn whitespace_in_domain_trimmed() {
    let r = fp(&["a@ corp.io ", "b@corp.io"]);
    assert_eq!(r.domains.len(), 1);
    assert_eq!(r.domains[0].domain, "corp.io");
    assert_eq!(r.domains[0].commits, 2);
}

#[test]
fn unicode_domain_counted() {
    let r = fp(&["u@ünïcödé.org"]);
    assert_eq!(r.domains.len(), 1);
    assert_eq!(r.domains[0].domain, "ünïcödé.org");
}

#[test]
fn ip_address_domain() {
    let r = fp(&["admin@192.168.1.1"]);
    assert_eq!(r.domains[0].domain, "192.168.1.1");
}

// ═══════════════════════════════════════════════════════════════
// 28–32  Sorting
// ═══════════════════════════════════════════════════════════════

#[test]
fn sorted_by_commits_descending() {
    let r = fp(&[
        "a@three.io",
        "b@three.io",
        "c@three.io",
        "d@two.io",
        "e@two.io",
        "f@one.io",
    ]);
    assert_eq!(r.domains[0].domain, "three.io");
    assert_eq!(r.domains[1].domain, "two.io");
    assert_eq!(r.domains[2].domain, "one.io");
}

#[test]
fn tied_commits_sorted_alphabetically() {
    let r = fp(&["a@zebra.com", "b@alpha.com", "c@mango.com"]);
    assert_eq!(r.domains[0].domain, "alpha.com");
    assert_eq!(r.domains[1].domain, "mango.com");
    assert_eq!(r.domains[2].domain, "zebra.com");
}

#[test]
fn twenty_six_single_letter_domains_sorted() {
    let authors: Vec<String> = ('a'..='z').map(|c| format!("u@{c}.com")).collect();
    let commits: Vec<GitCommit> = authors.iter().map(|a| commit(a)).collect();
    let r = build_corporate_fingerprint(&commits);
    assert_eq!(r.domains.len(), 26);
    for w in r.domains.windows(2) {
        assert!(
            w[0].domain < w[1].domain,
            "alphabetical: {} before {}",
            w[0].domain,
            w[1].domain
        );
    }
}

#[test]
fn public_email_sorts_with_corporate_by_count() {
    // 3 corporate, 1 public → corporate first
    let r = fp(&["a@corp.io", "b@corp.io", "c@corp.io", "d@gmail.com"]);
    assert_eq!(r.domains[0].domain, "corp.io");
    assert_eq!(r.domains[0].commits, 3);
    assert_eq!(r.domains[1].domain, "public-email");
    assert_eq!(r.domains[1].commits, 1);
}

#[test]
fn sort_stability_across_runs() {
    let authors = &["a@x.com", "b@y.com", "c@z.com", "d@x.com", "e@y.com"];
    let r1 = fp(authors);
    let r2 = fp(authors);
    for (a, b) in r1.domains.iter().zip(r2.domains.iter()) {
        assert_eq!(a.domain, b.domain);
    }
}

// ═══════════════════════════════════════════════════════════════
// 33–37  Percentage precision
// ═══════════════════════════════════════════════════════════════

#[test]
fn single_domain_100_pct() {
    let r = fp(&["a@corp.io"]);
    assert!((r.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

#[test]
fn two_domains_equal_split() {
    let r = fp(&["a@x.com", "b@y.com"]);
    for d in &r.domains {
        assert!((d.pct - 0.5).abs() < 0.01);
    }
}

#[test]
fn three_domains_equal_split() {
    let r = fp(&["a@a.com", "b@b.com", "c@c.com"]);
    for d in &r.domains {
        assert!(
            (d.pct - 1.0 / 3.0).abs() < 0.01,
            "expected ~0.333, got {}",
            d.pct
        );
    }
}

#[test]
fn percentages_sum_to_one() {
    let r = fp(&["a@x.com", "b@y.com", "c@z.com", "d@x.com", "e@gmail.com"]);
    let total: f32 = r.domains.iter().map(|d| d.pct).sum();
    assert!(
        (total - 1.0).abs() < 0.01,
        "percentages should sum to ~1.0, got {total}"
    );
}

#[test]
fn large_scale_pct_accuracy() {
    let mut authors = Vec::new();
    for i in 0..300 {
        authors.push(format!("u{i}@gmail.com"));
    }
    for i in 0..200 {
        authors.push(format!("d{i}@bigcorp.com"));
    }
    let commits: Vec<GitCommit> = authors.iter().map(|a| commit(a)).collect();
    let r = build_corporate_fingerprint(&commits);
    let public = find_domain(&r, "public-email").unwrap();
    assert_eq!(public.commits, 300);
    assert!((public.pct - 0.6).abs() < 0.01);
    let corp = find_domain(&r, "bigcorp.com").unwrap();
    assert_eq!(corp.commits, 200);
    assert!((corp.pct - 0.4).abs() < 0.01);
}

// ═══════════════════════════════════════════════════════════════
// 38–40  Determinism
// ═══════════════════════════════════════════════════════════════

#[test]
fn deterministic_json_across_runs() {
    let authors = &["a@one.io", "b@two.io", "c@gmail.com", "d@one.io"];
    let r1 = fp(authors);
    let r2 = fp(authors);
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2, "JSON must be deterministic");
}

#[test]
fn timestamps_do_not_affect_result() {
    let c1 = vec![
        GitCommit {
            timestamp: 1000,
            author: "a@corp.io".to_string(),
            hash: None,
            subject: String::new(),
            files: vec![],
        },
        GitCommit {
            timestamp: 2000,
            author: "b@other.dev".to_string(),
            hash: None,
            subject: String::new(),
            files: vec![],
        },
    ];
    let c2 = vec![
        GitCommit {
            timestamp: 9999,
            author: "a@corp.io".to_string(),
            hash: None,
            subject: String::new(),
            files: vec![],
        },
        GitCommit {
            timestamp: 1,
            author: "b@other.dev".to_string(),
            hash: None,
            subject: String::new(),
            files: vec![],
        },
    ];
    let r1 = build_corporate_fingerprint(&c1);
    let r2 = build_corporate_fingerprint(&c2);
    for (a, b) in r1.domains.iter().zip(r2.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
    }
}

#[test]
fn hash_subject_files_do_not_affect_result() {
    let c1 = vec![GitCommit {
        timestamp: 0,
        author: "dev@firm.co".to_string(),
        hash: Some("abc123".to_string()),
        subject: "feat: something".to_string(),
        files: vec!["src/main.rs".to_string()],
    }];
    let c2 = vec![GitCommit {
        timestamp: 0,
        author: "dev@firm.co".to_string(),
        hash: None,
        subject: String::new(),
        files: vec![],
    }];
    let r1 = build_corporate_fingerprint(&c1);
    let r2 = build_corporate_fingerprint(&c2);
    assert_eq!(r1.domains[0].domain, r2.domains[0].domain);
    assert_eq!(r1.domains[0].commits, r2.domains[0].commits);
}

// ═══════════════════════════════════════════════════════════════
// 41–43  Serde round-trips
// ═══════════════════════════════════════════════════════════════

#[test]
fn serde_round_trip_with_data() {
    let r = fp(&["a@corp.io", "b@corp.io", "c@gmail.com"]);
    let json = serde_json::to_string(&r).unwrap();
    let rt: CorporateFingerprint = serde_json::from_str(&json).unwrap();
    assert_eq!(r.domains.len(), rt.domains.len());
    for (a, b) in r.domains.iter().zip(rt.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}

#[test]
fn serde_round_trip_empty() {
    let r = build_corporate_fingerprint(&[]);
    let json = serde_json::to_string(&r).unwrap();
    assert_eq!(json, r#"{"domains":[]}"#);
    let rt: CorporateFingerprint = serde_json::from_str(&json).unwrap();
    assert!(rt.domains.is_empty());
}

#[test]
fn domain_stat_json_shape() {
    let stat = DomainStat {
        domain: "test.org".to_string(),
        commits: 7,
        pct: 0.25,
    };
    let v: serde_json::Value = serde_json::to_value(stat).unwrap();
    assert!(v.is_object());
    assert_eq!(v["domain"], "test.org");
    assert_eq!(v["commits"], 7);
}

// ═══════════════════════════════════════════════════════════════
// 44  Large-scale mixed batch
// ═══════════════════════════════════════════════════════════════

#[test]
fn large_mixed_batch() {
    let mut authors = Vec::new();
    // 20 ignored
    for i in 0..10 {
        authors.push(format!("bot{i}@users.noreply.github.com"));
    }
    for i in 0..5 {
        authors.push(format!("ci{i}@localhost"));
    }
    for i in 0..5 {
        authors.push(format!("test{i}@example.com"));
    }
    // 15 public
    for i in 0..5 {
        authors.push(format!("u{i}@gmail.com"));
    }
    for i in 0..5 {
        authors.push(format!("d{i}@yahoo.com"));
    }
    for i in 0..5 {
        authors.push(format!("o{i}@outlook.com"));
    }
    // 10 corporate
    for i in 0..7 {
        authors.push(format!("e{i}@bigcorp.com"));
    }
    for i in 0..3 {
        authors.push(format!("q{i}@smallco.io"));
    }

    let commits: Vec<GitCommit> = authors.iter().map(|a| commit(a)).collect();
    let r = build_corporate_fingerprint(&commits);
    let total: u32 = r.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 25);

    let public = find_domain(&r, "public-email").unwrap();
    assert_eq!(public.commits, 15);
    let big = find_domain(&r, "bigcorp.com").unwrap();
    assert_eq!(big.commits, 7);
    let small = find_domain(&r, "smallco.io").unwrap();
    assert_eq!(small.commits, 3);
}

// ═══════════════════════════════════════════════════════════════
// 45  Same author counted per-commit
// ═══════════════════════════════════════════════════════════════

#[test]
fn same_author_counted_per_commit() {
    let r = fp(&["dev@corp.io", "dev@corp.io", "dev@corp.io"]);
    assert_eq!(r.domains[0].commits, 3);
}

// ═══════════════════════════════════════════════════════════════
// Proptest properties
// ═══════════════════════════════════════════════════════════════

mod w61_properties {
    use super::*;

    fn arb_email() -> impl Strategy<Value = String> {
        ("[a-z]{2,6}@[a-z]{2,6}\\.(com|io|dev|org)").prop_map(|s| s)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(80))]

        #[test]
        fn total_commits_conserved(emails in proptest::collection::vec(arb_email(), 1..50)) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let r = build_corporate_fingerprint(&commits);
            let total: u32 = r.domains.iter().map(|d| d.commits).sum();
            prop_assert_eq!(total, emails.len() as u32);
        }

        #[test]
        fn pct_in_range(emails in proptest::collection::vec(arb_email(), 1..50)) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let r = build_corporate_fingerprint(&commits);
            for d in &r.domains {
                prop_assert!(d.pct >= 0.0);
                prop_assert!(d.pct <= 1.0);
            }
        }

        #[test]
        fn pct_sum_approx_one(emails in proptest::collection::vec(arb_email(), 1..50)) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let r = build_corporate_fingerprint(&commits);
            if !r.domains.is_empty() {
                let sum: f32 = r.domains.iter().map(|d| d.pct).sum();
                prop_assert!((sum - 1.0).abs() < 0.01, "pct sum was {sum}");
            }
        }

        #[test]
        fn domains_sorted_correctly(emails in proptest::collection::vec(arb_email(), 1..50)) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let r = build_corporate_fingerprint(&commits);
            for w in r.domains.windows(2) {
                let ok = w[0].commits > w[1].commits
                    || (w[0].commits == w[1].commits && w[0].domain <= w[1].domain);
                prop_assert!(ok, "sort violated: {} then {}", w[0].domain, w[1].domain);
            }
        }

        #[test]
        fn no_empty_domain_names(emails in proptest::collection::vec(arb_email(), 1..50)) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let r = build_corporate_fingerprint(&commits);
            for d in &r.domains {
                prop_assert!(!d.domain.is_empty());
            }
        }

        #[test]
        fn never_panics(author in ".*") {
            let _ = build_corporate_fingerprint(&[commit(&author)]);
        }

        #[test]
        fn domains_lowercase(emails in proptest::collection::vec(
            "[a-zA-Z]{1,6}@[a-zA-Z]{1,6}\\.[a-zA-Z]{2,3}",
            1..20
        )) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let r = build_corporate_fingerprint(&commits);
            for d in &r.domains {
                prop_assert_eq!(&d.domain, &d.domain.to_lowercase());
            }
        }

        #[test]
        fn deterministic(emails in proptest::collection::vec(arb_email(), 1..30)) {
            let commits: Vec<GitCommit> = emails.iter().map(|e| commit(e)).collect();
            let r1 = build_corporate_fingerprint(&commits);
            let r2 = build_corporate_fingerprint(&commits);
            let j1 = serde_json::to_string(&r1).unwrap();
            let j2 = serde_json::to_string(&r2).unwrap();
            prop_assert_eq!(j1, j2);
        }
    }
}
