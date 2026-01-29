# tokmd-walk

## Purpose

File listing and asset discovery utilities. This is a **Tier 2** utility crate for filesystem traversal with gitignore support.

## Responsibility

- Filesystem traversal respecting gitignore
- License candidate detection
- File size queries
- **NOT** for content scanning (see tokmd-content)

## Public API

```rust
/// List files in directory, respecting gitignore
pub fn list_files(root: &Path, max_files: Option<usize>) -> Result<Vec<PathBuf>>

/// Find license-related files
pub fn license_candidates(files: &[PathBuf]) -> LicenseCandidates

/// Get file size
pub fn file_size(root: &Path, relative: &Path) -> Result<u64>
```

### Internal Helper
```rust
/// Try git ls-files first, fall back to ignore crate
fn git_ls_files(root: &Path) -> Result<Option<Vec<PathBuf>>>
```

## Behavior

### File Listing Priority
1. Try `git ls-files` for accurate repo file listing
2. Fall back to `ignore` crate with gitignore respecting
3. Sort files alphabetically

### License Detection
Identifies common license files:
- `LICENSE*`, `COPYING*`, `NOTICE*`
- Metadata: `Cargo.toml`, `package.json`, `pyproject.toml`

```rust
pub struct LicenseCandidates {
    pub license_files: Vec<PathBuf>,
    pub metadata_files: Vec<PathBuf>,
}
```

## Dependencies

- `ignore` (0.4.22) - gitignore-aware walking
- `anyhow`

## Testing

```bash
cargo test -p tokmd-walk
```

Uses `tempfile` for directory structure tests.

## Do NOT

- Read file contents (use tokmd-content)
- Modify files
- Add git history logic (use tokmd-git)
