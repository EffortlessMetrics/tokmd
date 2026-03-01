//! # tokmd-analysis-fingerprint
//!
//! Corporate fingerprint enrichment adapter for analysis receipts.

use std::collections::BTreeMap;

use tokmd_analysis_types::{CorporateFingerprint, DomainStat};

const PUBLIC_DOMAINS: [&str; 7] = [
    "gmail.com",
    "yahoo.com",
    "outlook.com",
    "hotmail.com",
    "icloud.com",
    "proton.me",
    "protonmail.com",
];

/// Build a corporate fingerprint from commit author email domains.
pub fn build_corporate_fingerprint(commits: &[tokmd_git::GitCommit]) -> CorporateFingerprint {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut total = 0u32;

    for commit in commits {
        if let Some(domain) = extract_domain(&commit.author) {
            let domain = normalize_domain(&domain);
            if domain.is_empty() || is_ignored_domain(&domain) {
                continue;
            }
            let bucket = if is_public_domain(&domain) {
                "public-email".to_string()
            } else {
                domain
            };
            *counts.entry(bucket).or_insert(0) += 1;
            total += 1;
        }
    }

    let mut domains: Vec<DomainStat> = counts
        .into_iter()
        .map(|(domain, commits)| DomainStat {
            domain,
            commits,
            pct: if total == 0 {
                0.0
            } else {
                (commits as f32) / (total as f32)
            },
        })
        .collect();
    domains.sort_by(|a, b| {
        b.commits
            .cmp(&a.commits)
            .then_with(|| a.domain.cmp(&b.domain))
    });

    CorporateFingerprint { domains }
}

fn extract_domain(email: &str) -> Option<String> {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return None;
    }
    Some(parts[1].to_string())
}

fn normalize_domain(domain: &str) -> String {
    domain.trim().to_lowercase()
}

fn is_ignored_domain(domain: &str) -> bool {
    domain == "localhost"
        || domain == "example.com"
        || domain.contains("noreply.github.com")
        || domain.contains("users.noreply.github.com")
}

fn is_public_domain(domain: &str) -> bool {
    PUBLIC_DOMAINS.contains(&domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_domain_valid() {
        assert_eq!(
            extract_domain("alice@example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(extract_domain("bob@ACME.COM"), Some("ACME.COM".to_string()));
    }

    #[test]
    fn extract_domain_invalid() {
        assert_eq!(extract_domain("no-at-sign"), None);
        assert_eq!(extract_domain("too@many@ats"), None);
        assert_eq!(extract_domain(""), None);
    }

    #[test]
    fn ignored_domains_filtered() {
        assert!(is_ignored_domain("localhost"));
        assert!(is_ignored_domain("example.com"));
        assert!(is_ignored_domain("12345+user@users.noreply.github.com"));
        assert!(!is_ignored_domain("acme.com"));
    }

    #[test]
    fn empty_commits_produces_empty_fingerprint() {
        let report = build_corporate_fingerprint(&[]);
        assert!(report.domains.is_empty());
    }

    #[test]
    fn normalize_domain_trims_and_lowercases() {
        assert_eq!(normalize_domain("  ACME.COM  "), "acme.com");
        assert_eq!(normalize_domain("GitHub.Com"), "github.com");
    }

    #[test]
    fn buckets_public_domains() {
        let commits = vec![
            tokmd_git::GitCommit {
                timestamp: 0,
                author: "alice@gmail.com".to_string(),
                hash: None,
                subject: String::new(),
                files: vec![],
            },
            tokmd_git::GitCommit {
                timestamp: 0,
                author: "bob@acme.com".to_string(),
                hash: None,
                subject: String::new(),
                files: vec![],
            },
            tokmd_git::GitCommit {
                timestamp: 0,
                author: "carol@acme.com".to_string(),
                hash: None,
                subject: String::new(),
                files: vec![],
            },
        ];

        let report = build_corporate_fingerprint(&commits);

        assert!(report.domains.iter().any(|d| d.domain == "public-email"));
        assert!(report.domains.iter().any(|d| d.domain == "acme.com"));
    }
}
