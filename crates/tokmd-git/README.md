# tokmd-git

Streaming git log adapter for tokmd analysis.

## Overview

This is a **Tier 2** crate for git history collection. It provides a streaming interface to collect commit information without loading entire history into memory.

## Installation

```toml
[dependencies]
tokmd-git = "1.2"
```

## Usage

```rust
use tokmd_git::{git_available, repo_root, collect_history};
use std::path::Path;

// Check git availability
if git_available() {
    // Find repository root
    if let Some(root) = repo_root(Path::new(".")) {
        // Collect history with limits
        let commits = collect_history(&root, Some(500), Some(50))?;

        for commit in commits {
            println!("{}: {} files by {}",
                commit.timestamp,
                commit.files.len(),
                commit.author
            );
        }
    }
}
```

## Key Functions

### Detection
```rust
pub fn git_available() -> bool
pub fn repo_root(path: &Path) -> Option<PathBuf>
```

### History Collection
```rust
pub fn collect_history(
    repo_root: &Path,
    max_commits: Option<usize>,
    max_commit_files: Option<usize>,
) -> Result<Vec<GitCommit>>

pub struct GitCommit {
    pub timestamp: i64,      // Unix timestamp
    pub author: String,      // Email address
    pub files: Vec<String>,  // Affected file paths
}
```

## Implementation Details

- Uses `git log --name-only --pretty=format:%ct|%ae`
- Parses output line by line (streaming)
- Respects `max_commits` and `max_commit_files` limits
- Returns error if git command fails
- Returns empty vec if not a git repository

## Why Shell Out?

This crate uses the git CLI rather than libgit2 for simplicity and to avoid native dependency complexity. The streaming approach keeps memory usage low for large repositories.

## License

MIT OR Apache-2.0
