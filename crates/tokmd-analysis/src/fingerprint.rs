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

pub(crate) fn build_corporate_fingerprint(commits: &[tokmd_git::GitCommit]) -> CorporateFingerprint {
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
    domains.sort_by(|a, b| b.commits.cmp(&a.commits).then_with(|| a.domain.cmp(&b.domain)));

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
    PUBLIC_DOMAINS.iter().any(|d| *d == domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buckets_public_domains() {
        let commits = vec![
            tokmd_git::GitCommit {
                timestamp: 0,
                author: "alice@gmail.com".to_string(),
                files: vec![],
            },
            tokmd_git::GitCommit {
                timestamp: 0,
                author: "bob@acme.com".to_string(),
                files: vec![],
            },
            tokmd_git::GitCommit {
                timestamp: 0,
                author: "carol@acme.com".to_string(),
                files: vec![],
            },
        ];
        let report = build_corporate_fingerprint(&commits);
        assert!(report.domains.iter().any(|d| d.domain == "public-email"));
        assert!(report.domains.iter().any(|d| d.domain == "acme.com"));
    }
}
