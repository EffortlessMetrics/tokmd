//! Unit tests for corporate fingerprint detection.

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

// ── Empty / trivial input ────────────────────────────────────────

#[test]
fn empty_slice_returns_empty_domains() {
    let fp = build_corporate_fingerprint(&[]);
    assert!(fp.domains.is_empty());
}

#[test]
fn all_ignored_commits_produce_empty_fingerprint() {
    let commits = vec![
        commit("bot@users.noreply.github.com"),
        commit("ci@localhost"),
        commit("test@example.com"),
        commit("12345+user@users.noreply.github.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

// ── Public-email bucketing ───────────────────────────────────────

#[test]
fn each_public_provider_maps_to_public_email_bucket() {
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
        let commits = vec![commit(&format!("user@{provider}"))];
        let fp = build_corporate_fingerprint(&commits);
        assert_eq!(fp.domains.len(), 1, "expected 1 bucket for {provider}");
        assert_eq!(
            fp.domains[0].domain, "public-email",
            "{provider} should map to public-email"
        );
    }
}

#[test]
fn mixed_public_providers_merge_into_single_bucket() {
    let commits = vec![
        commit("a@gmail.com"),
        commit("b@hotmail.com"),
        commit("c@proton.me"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "public-email");
    assert_eq!(fp.domains[0].commits, 3);
}

// ── Corporate domain handling ────────────────────────────────────

#[test]
fn single_corporate_domain_gets_full_share() {
    let commits = vec![commit("dev@corp.io"), commit("admin@corp.io")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].domain, "corp.io");
    assert_eq!(fp.domains[0].commits, 2);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

#[test]
fn multiple_corporate_domains_ordered_by_commits_desc() {
    let commits = vec![
        commit("a@major.com"),
        commit("b@major.com"),
        commit("c@major.com"),
        commit("d@minor.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains[0].domain, "major.com");
    assert_eq!(fp.domains[0].commits, 3);
    assert_eq!(fp.domains[1].domain, "minor.com");
    assert_eq!(fp.domains[1].commits, 1);
}

#[test]
fn equal_commit_counts_sorted_alphabetically() {
    let commits = vec![commit("x@zulu.org"), commit("y@alpha.org")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains[0].domain, "alpha.org");
    assert_eq!(fp.domains[1].domain, "zulu.org");
}

// ── Percentage accuracy ──────────────────────────────────────────

#[test]
fn percentages_are_correct_for_known_distribution() {
    // 2 acme, 1 beta, 1 public → total 4 counted commits
    let commits = vec![
        commit("a@acme.com"),
        commit("b@acme.com"),
        commit("c@beta.io"),
        commit("d@gmail.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 3);

    let acme = fp.domains.iter().find(|d| d.domain == "acme.com").unwrap();
    let beta = fp.domains.iter().find(|d| d.domain == "beta.io").unwrap();
    let public = fp
        .domains
        .iter()
        .find(|d| d.domain == "public-email")
        .unwrap();

    assert!((acme.pct - 0.5).abs() < f32::EPSILON);
    assert!((beta.pct - 0.25).abs() < f32::EPSILON);
    assert!((public.pct - 0.25).abs() < f32::EPSILON);
}

#[test]
fn percentages_sum_to_one_with_many_domains() {
    let commits: Vec<GitCommit> = (0..100).map(|i| commit(&format!("u@d{i}.com"))).collect();
    let fp = build_corporate_fingerprint(&commits);
    let total_pct: f32 = fp.domains.iter().map(|d| d.pct).sum();
    assert!(
        (total_pct - 1.0).abs() < 0.02,
        "percentage sum was {total_pct}, expected ~1.0"
    );
}

// ── Ignored domains ──────────────────────────────────────────────

#[test]
fn noreply_github_variants_are_all_ignored() {
    let commits = vec![
        commit("user@users.noreply.github.com"),
        commit("123+user@users.noreply.github.com"),
        commit("bot@noreply.github.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn ignored_commits_do_not_inflate_totals() {
    let commits = vec![
        commit("dev@real.com"),
        commit("bot@localhost"),
        commit("ci@example.com"),
        commit("gh@users.noreply.github.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains.len(), 1);
    assert_eq!(fp.domains[0].commits, 1);
    assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
}

// ── Malformed / edge-case authors ────────────────────────────────

#[test]
fn author_without_at_sign_is_skipped() {
    let commits = vec![commit("just-a-name")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn author_with_multiple_at_signs_is_skipped() {
    let commits = vec![commit("a@b@c.com")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn empty_string_author_is_skipped() {
    let commits = vec![commit("")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

#[test]
fn at_sign_only_is_skipped() {
    // "@" splits into ["", ""] — domain normalizes to empty, skipped
    let commits = vec![commit("@")];
    let fp = build_corporate_fingerprint(&commits);
    assert!(fp.domains.is_empty());
}

// ── Domain normalization ─────────────────────────────────────────

#[test]
fn uppercase_domain_is_normalized_to_lowercase() {
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

#[test]
fn uppercase_public_domain_still_bucketed_as_public() {
    let commits = vec![commit("user@YAHOO.COM")];
    let fp = build_corporate_fingerprint(&commits);
    assert_eq!(fp.domains[0].domain, "public-email");
}

// ── Structural invariants ────────────────────────────────────────

#[test]
fn result_is_deterministic_across_repeated_calls() {
    let commits = vec![
        commit("a@x.com"),
        commit("b@y.com"),
        commit("c@x.com"),
        commit("d@gmail.com"),
    ];
    let fp1 = build_corporate_fingerprint(&commits);
    let fp2 = build_corporate_fingerprint(&commits);
    assert_eq!(fp1.domains.len(), fp2.domains.len());
    for (a, b) in fp1.domains.iter().zip(fp2.domains.iter()) {
        assert_eq!(a.domain, b.domain);
        assert_eq!(a.commits, b.commits);
        assert!((a.pct - b.pct).abs() < f32::EPSILON);
    }
}

#[test]
fn domain_list_is_sorted_descending_by_commits_then_by_name() {
    let commits = vec![
        commit("a@big.com"),
        commit("b@big.com"),
        commit("c@big.com"),
        commit("d@mid.com"),
        commit("e@mid.com"),
        commit("f@tiny.com"),
        commit("g@also-tiny.com"),
    ];
    let fp = build_corporate_fingerprint(&commits);
    for pair in fp.domains.windows(2) {
        let ok = pair[0].commits > pair[1].commits
            || (pair[0].commits == pair[1].commits && pair[0].domain <= pair[1].domain);
        assert!(
            ok,
            "sort invariant violated: {:?} before {:?}",
            pair[0].domain, pair[1].domain
        );
    }
}

#[test]
fn corporate_fingerprint_serializes_to_json() {
    let commits = vec![commit("dev@acme.com"), commit("ops@acme.com")];
    let fp = build_corporate_fingerprint(&commits);
    let json = serde_json::to_string(&fp).expect("should serialize");
    assert!(json.contains("acme.com"));
    assert!(json.contains("\"commits\":2"));
}
