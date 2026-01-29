# tokmd-git

## Purpose

Streaming git log adapter. This is a **Tier 2** crate for git history collection without loading the entire history into memory.

## Responsibility

- Git history collection
- Commit parsing (timestamp, author, affected files)
- Streaming interface
- **NOT** for analysis computation (see tokmd-analysis)

## Public API

```rust
/// Check if git is available
pub fn git_available() -> bool

/// Find repository root from path
pub fn repo_root(path: &Path) -> Option<PathBuf>

/// Collect commit history
pub fn collect_history(
    repo_root: &Path,
    max_commits: Option<usize>,
    max_commit_files: Option<usize>,
) -> Result<Vec<GitCommit>>

/// Commit data structure
pub struct GitCommit {
    pub timestamp: i64,      // Unix timestamp
    pub author: String,      // Email address
    pub files: Vec<String>,  // Affected file paths
}
```

## Implementation Details

### Git Command
Uses `git log --name-only --pretty=format:%ct|%ae`:
- `%ct` - Unix timestamp
- `%ae` - Author email
- `--name-only` - List affected files

### Streaming
- Parses output line by line
- Doesn't load entire history into memory
- Respects `max_commits` and `max_commit_files` limits

### Error Handling
- Returns error if git command fails
- Returns empty vec if not a git repository

## Dependencies

- `anyhow`

## Testing

```bash
cargo test -p tokmd-git
```

## Do NOT

- Compute git metrics (use tokmd-analysis)
- Modify git history
- Use git2 crate (shell out to git command for simplicity)
