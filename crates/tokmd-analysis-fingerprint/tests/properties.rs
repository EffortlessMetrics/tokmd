//! Property-based tests for corporate fingerprint extraction.

use proptest::prelude::*;
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

/// Strategy producing valid `user@domain.tld` email strings.
fn email_strategy() -> impl Strategy<Value = String> {
    (
        "[a-z]{1,10}",              // local part
        "[a-z]{1,10}\\.[a-z]{2,4}", // domain
    )
        .prop_map(|(user, domain)| format!("{user}@{domain}"))
}

/// Strategy producing a Vec of commits with valid emails.
fn commits_strategy(max_len: usize) -> impl Strategy<Value = Vec<GitCommit>> {
    prop::collection::vec(email_strategy(), 0..max_len)
        .prop_map(|emails| emails.iter().map(|e| commit(e)).collect())
}

proptest! {
    /// The function must never panic regardless of input.
    #[test]
    fn never_panics(author in ".*") {
        let commits = vec![commit(&author)];
        let _ = build_corporate_fingerprint(&commits);
    }

    /// Total commit count across all domain stats must equal the number
    /// of non-ignored, valid-email commits.
    #[test]
    fn commit_count_is_conserved(commits in commits_strategy(50)) {
        let fp = build_corporate_fingerprint(&commits);
        let reported: u32 = fp.domains.iter().map(|d| d.commits).sum();
        // Every generated email has exactly one '@' and a non-empty,
        // non-ignored domain, so all should be counted.
        prop_assert_eq!(reported, commits.len() as u32);
    }

    /// All percentages must be in [0, 1].
    #[test]
    fn percentages_in_range(commits in commits_strategy(50)) {
        let fp = build_corporate_fingerprint(&commits);
        for d in &fp.domains {
            prop_assert!(d.pct >= 0.0, "negative pct: {}", d.pct);
            prop_assert!(d.pct <= 1.0, "pct > 1: {}", d.pct);
        }
    }

    /// Percentages sum to ≈ 1.0 when there is at least one valid commit.
    #[test]
    fn percentages_sum_to_one(commits in commits_strategy(50)) {
        let fp = build_corporate_fingerprint(&commits);
        if !fp.domains.is_empty() {
            let sum: f32 = fp.domains.iter().map(|d| d.pct).sum();
            prop_assert!((sum - 1.0).abs() < 0.01,
                "pct sum was {} (expected ~1.0)", sum);
        }
    }

    /// Domain list is sorted: descending by commits, then ascending by name.
    #[test]
    fn domains_are_sorted(commits in commits_strategy(50)) {
        let fp = build_corporate_fingerprint(&commits);
        for window in fp.domains.windows(2) {
            let (a, b) = (&window[0], &window[1]);
            let ok = a.commits > b.commits
                || (a.commits == b.commits && a.domain <= b.domain);
            prop_assert!(ok,
                "sort violated: {:?} before {:?}", a.domain, b.domain);
        }
    }

    /// Every domain string is non-empty.
    #[test]
    fn no_empty_domain_names(commits in commits_strategy(50)) {
        let fp = build_corporate_fingerprint(&commits);
        for d in &fp.domains {
            prop_assert!(!d.domain.is_empty(), "empty domain name found");
        }
    }

    /// Domains are always lowercase (normalization invariant).
    #[test]
    fn domains_are_lowercase(
        emails in prop::collection::vec(
            ("[a-zA-Z]{1,8}@[a-zA-Z]{1,8}\\.[a-zA-Z]{2,3}", ),
            1..20
        )
    ) {
        let commits: Vec<GitCommit> = emails
            .iter()
            .map(|(e,)| commit(e))
            .collect();
        let fp = build_corporate_fingerprint(&commits);
        for d in &fp.domains {
            prop_assert_eq!(&d.domain, &d.domain.to_lowercase(),
                "domain not lowercase: {}", d.domain);
        }
    }

    /// Empty input always yields empty output.
    #[test]
    fn empty_input_empty_output(_seed in 0u32..100) {
        let fp = build_corporate_fingerprint(&[]);
        prop_assert!(fp.domains.is_empty());
    }

    /// Duplicate authors in successive commits should increment the count.
    #[test]
    fn duplicate_emails_merge(
        email in email_strategy(),
        n in 2usize..20
    ) {
        let commits: Vec<GitCommit> = (0..n).map(|_| commit(&email)).collect();
        let fp = build_corporate_fingerprint(&commits);
        // Should be exactly 1 domain bucket
        prop_assert_eq!(fp.domains.len(), 1);
        prop_assert_eq!(fp.domains[0].commits, n as u32);
        prop_assert!((fp.domains[0].pct - 1.0).abs() < f32::EPSILON);
    }

    /// Fingerprint is deterministic: same input always produces identical output.
    #[test]
    fn deterministic(commits in commits_strategy(50)) {
        let fp1 = build_corporate_fingerprint(&commits);
        let fp2 = build_corporate_fingerprint(&commits);
        prop_assert_eq!(fp1.domains.len(), fp2.domains.len());
        for (a, b) in fp1.domains.iter().zip(fp2.domains.iter()) {
            prop_assert_eq!(&a.domain, &b.domain);
            prop_assert_eq!(a.commits, b.commits);
            prop_assert!((a.pct - b.pct).abs() < f32::EPSILON,
                "pct mismatch for {}: {} vs {}", a.domain, a.pct, b.pct);
        }
    }

    /// Reordering commits must produce the same domain list (order-insensitive aggregation).
    #[test]
    fn order_independent(commits in commits_strategy(30)) {
        let mut reversed = commits.clone();
        reversed.reverse();
        let fp_orig = build_corporate_fingerprint(&commits);
        let fp_rev = build_corporate_fingerprint(&reversed);
        prop_assert_eq!(fp_orig.domains.len(), fp_rev.domains.len());
        for (a, b) in fp_orig.domains.iter().zip(fp_rev.domains.iter()) {
            prop_assert_eq!(&a.domain, &b.domain);
            prop_assert_eq!(a.commits, b.commits);
        }
    }

    /// Number of domain buckets never exceeds the number of commits.
    #[test]
    fn bucket_count_bounded(commits in commits_strategy(50)) {
        let fp = build_corporate_fingerprint(&commits);
        prop_assert!(fp.domains.len() <= commits.len(),
            "more buckets ({}) than commits ({})", fp.domains.len(), commits.len());
    }
}
