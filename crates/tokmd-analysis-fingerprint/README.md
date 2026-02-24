# tokmd-analysis-fingerprint

This microcrate computes corporate fingerprint enrichment from commit metadata.
It belongs to the `tokmd-analysis` family as a small, stable adapter crate so the
fingerprint logic can evolve without forcing broader analysis changes.

## Overview

`tokmd-analysis-fingerprint` exposes a single API:

- `build_corporate_fingerprint(commits: &[tokmd_git::GitCommit]) -> CorporateFingerprint`

## Usage

```rust
use tokmd_analysis_fingerprint::build_corporate_fingerprint;
use tokmd_git::GitCommit;

let commits = vec![GitCommit {
    timestamp: 0,
    author: "alice@acme.com".to_string(),
    hash: None,
    subject: String::new(),
    files: vec!["src/main.rs".to_string()],
}];

let fingerprint = build_corporate_fingerprint(&commits);
assert!(!fingerprint.domains.is_empty());
```
