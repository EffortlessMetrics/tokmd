//! Deep integration tests for corporate fingerprint detection.
//!
//! Targets gaps not covered by existing unit/bdd/edge/property/identity suites:
//! - DomainStat serde round-trip and JSON shape
//! - CorporateFingerprint deserialization from known JSON
//! - Unicode domain handling
//! - Domain with numeric TLD
//! - Very long email addresses
//! - Author string as bare IP address
//! - Interaction: many ignored + many public + many corporate in one batch
//! - Three-way tie in commit counts sorts alphabetically
//! - Commits with hash and files don't affect domain extraction
//! - Public-email bucket pct accuracy at large scale
//! - Empty domain after @ is skipped
//! - Domain deduplication is exact
//! - Fingerprint with single public commit
//! - CorporateFingerprint Clone preserves all fields

use tokmd_analysis_fingerprint::build_corporate_fingerprint;
use tokmd_analysis_types::{CorporateFingerprint, DomainStat};
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

// ===========================================================================
// 1. DomainStat serde round-trip
// ===========================================================================

#[test]
fn domain_stat_serde_round_trip() {
    let stat = DomainStat {
        domain: "acme.com".to_string(),
        commits: 42,
        pct: 0.75,
    };
    let json = serde_json::to_string(&stat).unwrap();
    let deserialized: DomainStat = serde_json::from_str(&json).unwrap();
    assert_eq!(stat.domain, deserialized.domain);
    assert_eq!(stat.commits, deserialized.commits);
    assert!((stat.pct - deserialized.pct).abs() < f32::EPSILON);
}

// ===========================================================================
// 2. DomainStat JSON shape
// ===========================================================================

#[test]
fn domain_stat_json_shape() {
    let stat = DomainStat {
        domain: "example.org".to_string(),
        commits: 10,
        pct: 0.5,
    };
    let v: serde_json::Value = serde_json::to_value(&stat).unwrap();
    assert!(v.is_object());
    assert_eq!(v["domain"], "example.org");
    assert_eq!(v["commits"], 10);
    assert_eq!(v["pct"], 0.5);
}

// ===========================================================================
// 3. CorporateFingerprint deserialization from known JSON
// ===========================================================================

#[test]
fn corporate_fingerprint_deserializes_from_known_json() {
    let json = r#"{"domains":[{"domain":"acme.com","commits":5,"pct":0.625},{"domain":"public-email","commits":3,"pct":0.375}]}"#;
    let fp: CorporateFingerprint = serde_json::from_str(json).unwrap();
    assert_eq!(fp.domains.len(), 2);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 5);
    assert_eq!(fp.domains[1].domain, "public-email");
}

// ===========================================================================
// 4. CorporateFingerprint serde round-trip
// ===========================================================================

#[test]
fn corporate_fingerprint_serde_round_trip() {
    let commits = vec![
        commit("a@corp.io"),
        commit("b@corp.io"),
        commit("c@gmail.com"),
    ];
    let original = build_corporate_fingerprint(&commits);
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CorporateFingerprint = serde_json::from_str(&json).unwrap();

    assert_eq!(original.domains.len(), deserialized.domains.len());
    for (a, b) in original.domains.iter().zip(deserialized.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}

// ===========================================================================
// 5. Unicode domain handling
// ===========================================================================

#[test]
fn unicode_domain_is_counted_as_corporate() {
    let commits = vec![commit("user@ünïcödé.org")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "ünïcödé.org");
}

// ===========================================================================
// 6. Domain with numeric TLD
// ===========================================================================

#[test]
fn numeric_tld_domain_counted() {
    let commits = vec![commit("user@company.123")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "company.123");
}

// ===========================================================================
// 7. Very long email address
// ===========================================================================

#[test]
fn very_long_email_address_handled() {
    let local = "a".repeat(200);
    let domain = format!("{}.{}.com", "b".repeat(100), "c".repeat(50));
    let email = format!("{local}@{domain}");
    let commits = vec![commit(&email)];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, domain.to_lowercase());
}

// ===========================================================================
// 8. Author with bare IP-like domain
// ===========================================================================

#[test]
fn ip_address_domain_is_counted() {
    let commits = vec![commit("admin@192.168.1.1")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "192.168.1.1");
}

// ===========================================================================
// 9. Three-way tie in commit counts sorts alphabetically
// ===========================================================================

#[test]
fn three_way_tie_sorted_alphabetically() {
    let commits = vec![
        commit("x@zebra.com"),
        commit("y@mango.com"),
        commit("z@apple.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 3);
    assert_eq!(fp.domains[0].domain, "apple.com");
    assert_eq!(fp.domains[1].domain, "mango.com");
    assert_eq!(fp.domains[2].domain, "zebra.com");
}

// ===========================================================================
// 10. Many ignored + many public + many corporate
// ===========================================================================

#[test]
fn large_mixed_batch_correct_counts() {
    let mut commits: Vec<GitCommit> = Vec::new();

    // 20 ignored
    for i in 0..10 {
        commits.push(commit(&format!("bot{i}@users.noreply.github.com")));
    }
    for i in 0..5 {
        commits.push(commit(&format!("ci{i}@localhost")));
    }
    for i in 0..5 {
        commits.push(commit(&format!("test{i}@example.com")));
    }

    // 15 public
    for i in 0..5 {
        commits.push(commit(&format!("user{i}@gmail.com")));
    }
    for i in 0..5 {
        commits.push(commit(&format!("dev{i}@yahoo.com")));
    }
    for i in 0..5 {
        commits.push(commit(&format!("ops{i}@outlook.com")));
    }

    // 10 corporate across 2 domains
    for i in 0..7 {
        commits.push(commit(&format!("eng{i}@bigcorp.com")));
    }
    for i in 0..3 {
        commits.push(commit(&format!("qa{i}@smallco.io")));
    }

    let fp = build_corporate_fingerprint(&commits);
    let total: u32 = fp.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 25, "15 public + 10 corporate = 25 valid");

    let public = fp
        .domains
        .iter()
        .find(|d| d.domain == "public-email")
        .unwrap();
    assert_eq!(public.commits, 15);

    let bigcorp = fp
        .domains
        .iter()
        .find(|d| d.domain == "bigcorp.com")
        .unwrap();
    assert_eq!(bigcorp.commits, 7);

    let smallco = fp
        .domains
        .iter()
        .find(|d| d.domain == "smallco.io")
        .unwrap();
    assert_eq!(smallco.commits, 3);
}

// ===========================================================================
// 11. Empty domain after @ is skipped
// ===========================================================================

#[test]
fn trailing_at_sign_empty_domain_skipped() {
    let commits = vec![commit("user@")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

// ===========================================================================
// 12. Domain deduplication is exact (case-insensitive)
// ===========================================================================

#[test]
fn mixed_case_domains_deduplicate() {
    let commits = vec![
        commit("a@Acme.COM"),
        commit("b@acme.com"),
        commit("c@ACME.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "acme.com");
    assert_eq!(fp.domains[0].commits, 3);
}

// ===========================================================================
// 13. Single public commit
// ===========================================================================

#[test]
fn single_public_commit_produces_public_email_bucket() {
    let commits = vec![commit("lone@icloud.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 1);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ===========================================================================
// 14. CorporateFingerprint Clone preserves all fields
// ===========================================================================

#[test]
fn corporate_fingerprint_clone_preserves_fields() {
    let commits = vec![commit("a@corp.io"), commit("b@gmail.com")];
    let original = build_corporate_fingerprint(&commits);
    let cloned = original.clone();

    assert_eq!(original.domains.len(), cloned.domains.len());
    for (a, b) in original.domains.iter().zip(cloned.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}

// ===========================================================================
// 15. CorporateFingerprint Debug contains domain info
// ===========================================================================

#[test]
fn corporate_fingerprint_debug_contains_domain_info() {
    let commits = vec![commit("dev@acme.com")];
    let fp = build_corporate_fingerprint(&commits);
    let dbg = format!("{:?}", fp);
    assert!(dbg.contains("acme.com"), "Debug should contain domain");
}

// ===========================================================================
// 16. DomainStat Clone preserves fields
// ===========================================================================

#[test]
fn domain_stat_clone_preserves_fields() {
    let stat = DomainStat {
        domain: "test.com".to_string(),
        commits: 5,
        pct: 0.5,
    };
    let cloned = stat.clone();
    assert_eq!(stat.domain, cloned.domain);
    assert_eq!(stat.commits, cloned.commits);
    assert!((stat.pct - cloned.pct).abs() < f32::EPSILON);
}

// ===========================================================================
// 17. Fingerprint with only proton.me and protonmail.com
// ===========================================================================

#[test]
fn proton_variants_both_map_to_public_email() {
    let commits = vec![commit("alice@proton.me"), commit("bob@protonmail.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 2);
}

// ===========================================================================
// 18. Domain with hyphen is valid corporate
// ===========================================================================

#[test]
fn hyphenated_domain_is_counted() {
    let commits = vec![commit("user@my-company.co.uk")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "my-company.co.uk");
}

// ===========================================================================
// 19. Commits with populated hash/subject/files still extract domain
// ===========================================================================

#[test]
fn full_commit_fields_do_not_affect_extraction() {
    let commits = vec![GitCommit {
        timestamp: 1_700_000_000,
        author: "dev@corp.io".to_string(),
        hash: Some("abc123def456".to_string()),
        subject: "feat: add something".to_string(),
        files: vec!["src/main.rs".to_string(), "Cargo.toml".to_string()],
    }];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "corp.io");
}

// ===========================================================================
// 20. Public-email pct accuracy at large scale
// ===========================================================================

#[test]
fn public_email_pct_accurate_at_scale() {
    let mut commits: Vec<GitCommit> = Vec::new();
    // 300 public
    for i in 0..300 {
        commits.push(commit(&format!("user{i}@gmail.com")));
    }
    // 200 corporate
    for i in 0..200 {
        commits.push(commit(&format!("dev{i}@bigcorp.com")));
    }

    let fp = build_corporate_fingerprint(&commits);
    let public = fp
        .domains
        .iter()
        .find(|d| d.domain == "public-email")
        .unwrap();
    assert_eq!(public.commits, 300);
    assert!(
        (public.pct - 0.6).abs() < 0.01,
        "expected ~0.6, got {}",
        public.pct
    );

    let corp = fp
        .domains
        .iter()
        .find(|d| d.domain == "bigcorp.com")
        .unwrap();
    assert_eq!(corp.commits, 200);
    assert!(
        (corp.pct - 0.4).abs() < 0.01,
        "expected ~0.4, got {}",
        corp.pct
    );
}

// ===========================================================================
// 21. CorporateFingerprint with empty domains serializes correctly
// ===========================================================================

#[test]
fn empty_fingerprint_serializes_correctly() {
    let fp = build_corporate_fingerprint(&[]);
    let json = serde_json::to_string(&fp).unwrap();
    assert_eq!(json, r#"{"domains":[]}"#);
}

// ===========================================================================
// 22. Ordering stability with many equal-count domains
// ===========================================================================

#[test]
fn many_equal_count_domains_sorted_alphabetically() {
    let commits: Vec<GitCommit> = ('a'..='z')
        .map(|c| commit(&format!("user@{c}.com")))
        .collect();
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 26);
    for window in fp.domains.windows(2) {
        assert!(
            window[0].domain < window[1].domain,
            "expected alphabetical: {} before {}",
            window[0].domain,
            window[1].domain
        );
    }
}

// ===========================================================================
// 23. Noreply.github.com with different prefixes
// ===========================================================================

#[test]
fn various_noreply_github_prefixes_all_ignored() {
    let commits = vec![
        commit("user@users.noreply.github.com"),
        commit("12345+user@users.noreply.github.com"),
        commit("bot@noreply.github.com"),
        commit("actions@noreply.github.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

// ===========================================================================
// 24. JSON round-trip preserves empty domains array
// ===========================================================================

#[test]
fn json_round_trip_preserves_empty_domains() {
    let fp = CorporateFingerprint { domains: vec![] };
    let json = serde_json::to_string(&fp).unwrap();
    let deserialized: CorporateFingerprint = serde_json::from_str(&json).unwrap();
    assert!(deserialized.domains.is_empty());
}

// ===========================================================================
// 25. Subdomain emails are kept distinct
// ===========================================================================

#[test]
fn subdomains_are_kept_distinct_not_merged() {
    let commits = vec![
        commit("a@eng.corp.com"),
        commit("b@sales.corp.com"),
        commit("c@eng.corp.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 2);

    let eng = fp
        .domains
        .iter()
        .find(|d| d.domain == "eng.corp.com")
        .unwrap();
    assert_eq!(eng.commits, 2);

    let sales = fp
        .domains
        .iter()
        .find(|d| d.domain == "sales.corp.com")
        .unwrap();
    assert_eq!(sales.commits, 1);
}

// ===========================================================================
// 26. Percentage precision with 3 domains
// ===========================================================================

#[test]
fn percentage_precision_three_domains() {
    // 3 domains with 1 commit each → each 33.33%
    let commits = vec![
        commit("a@alpha.org"),
        commit("b@beta.org"),
        commit("c@gamma.org"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 3);
    for d in &fp.domains {
        let expected = 1.0 / 3.0;
        assert!(
            (d.pct - expected).abs() < 0.01,
            "expected ~{expected}, got {} for {}",
            d.pct,
            d.domain
        );
    }
}
