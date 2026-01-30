# tokmd-content

Content scanning helpers for tokmd analysis.

## Overview

This is a **Tier 2** utility crate for file content inspection. It provides functions for reading file contents, detecting text files, computing hashes, counting tags, and calculating entropy.

## Installation

```toml
[dependencies]
tokmd-content = "1.2"
```

## Usage

```rust
use tokmd_content::{read_head, hash_file, count_tags, entropy_bits_per_byte, is_text_like};
use std::path::Path;

// Read first 4KB of file
let bytes = read_head(Path::new("src/lib.rs"), 4096)?;

// Check if content is text
if is_text_like(&bytes) {
    // Count TODO/FIXME tags
    let text = String::from_utf8_lossy(&bytes);
    let tags = count_tags(&text, &["TODO", "FIXME", "HACK"]);
}

// Calculate entropy (for secret detection)
let entropy = entropy_bits_per_byte(&bytes);

// Hash file for duplicate detection
let hash = hash_file(Path::new("src/lib.rs"), 1_000_000)?;
```

## Key Functions

### Reading
- `read_head()` - Read first N bytes
- `read_head_tail()` - Read balanced head + tail
- `read_lines()` - Read lines with limits
- `read_text_capped()` - Read as text with byte limit

### Detection
- `is_text_like()` - Check for null bytes and valid UTF-8

### Hashing
- `hash_bytes()` - BLAKE3 hash of bytes
- `hash_file()` - Hash file content (capped)

### Analysis
- `count_tags()` - Case-insensitive tag counting
- `entropy_bits_per_byte()` - Shannon entropy calculation

## Entropy Interpretation

| Range | Interpretation |
|-------|----------------|
| 0.0 | Empty or uniform |
| < 4.0 | Low (plain text) |
| 4.0-6.0 | Medium (source code) |
| 6.0-7.5 | High (compressed/encrypted) |
| > 7.5 | Suspicious (secrets, random) |

## Dependencies

- `blake3` - Fast cryptographic hashing
- `anyhow` - Error handling

## License

MIT OR Apache-2.0
