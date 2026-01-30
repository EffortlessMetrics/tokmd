# tokmd-walk

File listing and asset discovery utilities for tokmd.

## Overview

This is a **Tier 2** utility crate for filesystem traversal with gitignore support. It provides efficient file listing and license candidate detection.

## Installation

```toml
[dependencies]
tokmd-walk = "1.2"
```

## Usage

```rust
use tokmd_walk::{list_files, license_candidates, file_size};
use std::path::Path;

// List all files respecting gitignore
let files = list_files(Path::new("."), Some(1000))?;

// Find license-related files
let candidates = license_candidates(&files);
println!("License files: {:?}", candidates.license_files);
println!("Metadata files: {:?}", candidates.metadata_files);

// Get file size
let size = file_size(Path::new("."), Path::new("src/lib.rs"))?;
```

## Key Functions

### File Listing
```rust
pub fn list_files(root: &Path, max_files: Option<usize>) -> Result<Vec<PathBuf>>
```
Lists files respecting gitignore. Tries `git ls-files` first, falls back to the `ignore` crate.

### License Detection
```rust
pub fn license_candidates(files: &[PathBuf]) -> LicenseCandidates
```
Identifies common license files:
- License files: `LICENSE*`, `COPYING*`, `NOTICE*`
- Metadata: `Cargo.toml`, `package.json`, `pyproject.toml`

### File Size
```rust
pub fn file_size(root: &Path, relative: &Path) -> Result<u64>
```

## Behavior

1. Try `git ls-files` for accurate repo file listing
2. Fall back to `ignore` crate with gitignore support
3. Sort files alphabetically for deterministic output

## Dependencies

- `ignore` - Gitignore-aware file walking
- `anyhow` - Error handling

## License

MIT OR Apache-2.0
