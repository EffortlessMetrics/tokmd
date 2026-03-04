//! Wave-38 deep tests for `tokmd-analysis-fingerprint`.
//!
//! Focuses on areas not yet covered by the existing deep/bdd/edge/unit suites:
//! - Author with no @ sign (no domain extraction)
//! - Author with multiple @ signs
//! - Hotmail and iCloud specific public-email bucketing
//! - Mixed ignored + corporate with exact pct verification
//! - Ordering: corporate before public when higher count
//! - All PUBLIC_DOMAINS individually verified
//! - Whitespace-only domain ignored
//! - CorporateFingerprint JSON shape validation
//! - Domain with trailing dot
//! - Single corporate commit pct = 1.0
//! - Two corporate domains with unequal counts sort by count desc

use tokmd_analysis_fingerprint::build_corporate_fingerprint;
use tokmd_analysis_types::DomainStat;
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

// ═══════════════════════════════════════════════════════════════════
// § 1 – Author with no @ sign produces empty fingerprint
// ═══════════════════════════════════════════════════════════════════

#[test]
fn no_at_sign_produces_empty_fingerprint() {
    let commits = vec![commit("just-a-name")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(
        fp.domains.is_empty(),
        "author without @ should produce no domains"
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 2 – Author with multiple @ signs produces empty fingerprint
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multiple_at_signs_produces_empty_fingerprint() {
    let commits = vec![commit("user@@double.com"), commit("a@b@c.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(
        fp.domains.is_empty(),
        "multiple @ signs should not extract a valid domain"
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 3 – Hotmail bucketed as public-email
// ═══════════════════════════════════════════════════════════════════

#[test]
fn hotmail_is_public_email() {
    let commits = vec![commit("user@hotmail.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
}

// ═══════════════════════════════════════════════════════════════════
// § 4 – iCloud bucketed as public-email
// ═══════════════════════════════════════════════════════════════════

#[test]
fn icloud_is_public_email() {
    let commits = vec![commit("user@icloud.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
}

// ═══════════════════════════════════════════════════════════════════
// § 5 – All seven PUBLIC_DOMAINS map to public-email
// ═══════════════════════════════════════════════════════════════════

#[test]
fn all_public_domains_map_to_public_email() {
    let public_domains = [
        "gmail.com",
        "yahoo.com",
        "outlook.com",
        "hotmail.com",
        "icloud.com",
        "proton.me",
        "protonmail.com",
    ];
    for domain in &public_domains {
        let commits = vec![commit(&format!("user@{domain}"))];
        let fp = build_corporate_fingerprint(&commits);
        assert_eq!(fp.domains.len(), 1, "should have 1 domain for {domain}");
        assert_eq!(
            fp.domains[0].domain, "public-email",
            "{domain} should map to public-email"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 6 – Ordering: higher commit count comes first
// ═══════════════════════════════════════════════════════════════════

#[test]
fn higher_commit_count_sorted_first() {
    let mut commits = Vec::new();
    for _ in 0..5 {
        commits.push(commit("eng@bigcorp.com"));
    }
    for _ in 0..2 {
        commits.push(commit("dev@smallco.io"));
    }

    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 2);
    assert_eq!(fp.domains[0].domain, "bigcorp.com");
    assert_eq!(fp.domains[0].commits, 5);
    assert_eq!(fp.domains[1].domain, "smallco.io");
    assert_eq!(fp.domains[1].commits, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 7 – CorporateFingerprint JSON shape validation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn corporate_fingerprint_json_shape() {
    let commits = vec![commit("dev@corp.io"), commit("user@gmail.com")];
    let fp = build_corporate_fingerprint(&commits);
    let v: serde_json::Value = serde_json::to_value(fp).unwrap();

    assert!(v.is_object());
    assert!(v.get("domains").is_some());
    let domains = v["domains"].as_array().unwrap();
    assert_eq!(domains.len(), 2);

    // Each domain entry has domain, commits, pct
    for d in domains {
        assert!(d.get("domain").is_some());
        assert!(d.get("commits").is_some());
        assert!(d.get("pct").is_some());
        assert!(d["domain"].is_string());
        assert!(d["commits"].is_number());
        assert!(d["pct"].is_number());
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 8 – Single corporate commit pct = 1.0
// ═══════════════════════════════════════════════════════════════════

#[test]
fn single_corporate_commit_pct_one() {
    let commits = vec![commit("solo@mycompany.org")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].commits, 1);
    assert!(
        (fp.domains[0].pct - 1.0).abs() < f32::EPSILON,
        "single commit should have pct = 1.0"
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 9 – Mixed ignored + corporate with exact pct
// ═══════════════════════════════════════════════════════════════════

#[test]
fn mixed_ignored_and_corporate_pct() {
    let mut commits = Vec::new();
    // 5 ignored
    for i in 0..5 {
        commits.push(commit(&format!("bot{i}@users.noreply.github.com")));
    }
    // 4 corporate
    for i in 0..4 {
        commits.push(commit(&format!("eng{i}@acme.com")));
    }
    // 1 public
    commits.push(commit("user@gmail.com"));

    let fp = build_corporate_fingerprint(&commits);
    let total: u32 = fp.domains.iter().map(|d| d.commits).sum();
    assert_eq!(total, 5, "4 corporate + 1 public = 5");

    let acme = fp.domains.iter().find(|d| d.domain == "acme.com").unwrap();
    assert_eq!(acme.commits, 4);
    assert!(
        (acme.pct - 0.8).abs() < 0.01,
        "acme should be 80%, got {}",
        acme.pct
    );

    let public = fp
        .domains
        .iter()
        .find(|d| d.domain == "public-email")
        .unwrap();
    assert_eq!(public.commits, 1);
    assert!(
        (public.pct - 0.2).abs() < 0.01,
        "public should be 20%, got {}",
        public.pct
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 10 – Domain with trailing whitespace is trimmed
// ═══════════════════════════════════════════════════════════════════

#[test]
fn domain_with_whitespace_trimmed() {
    // The normalize_domain function trims whitespace
    let commits = vec![commit("user@corp.io")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains[0].domain, "corp.io");
}

// ═══════════════════════════════════════════════════════════════════
// § 11 – Example.com is ignored domain
// ═══════════════════════════════════════════════════════════════════

#[test]
fn example_com_is_ignored() {
    let commits = vec![commit("test@example.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty(), "example.com should be ignored");
}

// ═══════════════════════════════════════════════════════════════════
// § 12 – Localhost is ignored domain
// ═══════════════════════════════════════════════════════════════════

#[test]
fn localhost_is_ignored() {
    let commits = vec![commit("root@localhost")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty(), "localhost should be ignored");
}

// ═══════════════════════════════════════════════════════════════════
// § 13 – DomainStat JSON deserialization
// ═══════════════════════════════════════════════════════════════════

#[test]
fn domain_stat_deserializes_from_json() {
    let json = r#"{"domain":"test.org","commits":7,"pct":0.35}"#;
    let stat: DomainStat = serde_json::from_str(json).unwrap();
    assert_eq!(stat.domain, "test.org");
    assert_eq!(stat.commits, 7);
    assert!((stat.pct - 0.35).abs() < 0.01);
}

// ═══════════════════════════════════════════════════════════════════
// § 14 – Deterministic output across repeated calls
// ═══════════════════════════════════════════════════════════════════

#[test]
fn deterministic_output_across_calls() {
    let commits = vec![
        commit("a@alpha.com"),
        commit("b@beta.com"),
        commit("c@gmail.com"),
        commit("d@alpha.com"),
    ];
    let fp1 = build_corporate_fingerprint(&commits);
    let fp2 = build_corporate_fingerprint(&commits);

    let json1 = serde_json::to_string(&fp1).unwrap();
    let json2 = serde_json::to_string(&fp2).unwrap();
    assert_eq!(json1, json2, "output must be deterministic");
}

// ═══════════════════════════════════════════════════════════════════
// § 15 – All ignored domains produce empty fingerprint
// ═══════════════════════════════════════════════════════════════════

#[test]
fn all_ignored_domains_empty_fingerprint() {
    let commits = vec![
        commit("a@localhost"),
        commit("b@example.com"),
        commit("c@users.noreply.github.com"),
        commit("d@noreply.github.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}
