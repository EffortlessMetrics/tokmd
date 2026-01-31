# tokmd-content

## Purpose

Content scanning helpers for analysis. This is a **Tier 2** utility crate for file content inspection.

## Responsibility

- File content reading (head, tail, lines)
- Text detection
- File integrity hashing
- Tag counting (TODOs, FIXMEs)
- Entropy calculation
- **NOT** for file listing (see tokmd-walk)

## Public API

### Reading
```rust
pub fn read_head(path: &Path, max_bytes: usize) -> Result<Vec<u8>>
pub fn read_head_tail(path: &Path, max_bytes: usize) -> Result<Vec<u8>>  // Balanced head+tail
pub fn read_lines(path: &Path, max_lines: usize, max_bytes: usize) -> Result<Vec<String>>
pub fn read_text_capped(path: &Path, max_bytes: usize) -> Result<String>
```

### Detection
```rust
/// Checks for null bytes + valid UTF-8
pub fn is_text_like(bytes: &[u8]) -> bool
```

### Hashing
```rust
pub fn hash_bytes(bytes: &[u8]) -> String  // BLAKE3
pub fn hash_file(path: &Path, max_bytes: usize) -> Result<String>
```

### Analysis
```rust
/// Case-insensitive tag counting
pub fn count_tags(text: &str, tags: &[&str]) -> Vec<(String, usize)>

/// Shannon entropy calculation (bits per byte)
pub fn entropy_bits_per_byte(bytes: &[u8]) -> f32
```

## Implementation Details

### Entropy Calculation

Uses Shannon entropy formula: `-Σ(p_i * log2(p_i))`

| Range | Interpretation |
|-------|----------------|
| 0.0 | Empty or uniform |
| < 4.0 | Low (text) |
| 4.0-6.0 | Medium (code) |
| 6.0-7.5 | High (compressed/encrypted) |
| > 7.5 | Suspicious (secrets, random) |

Returns `0.0` for empty input.

## Dependencies

- `blake3` (hashing)
- `anyhow`

## Testing

```bash
cargo test -p tokmd-content
```

Property-based tests with `proptest`.

## Do NOT

- List files (use tokmd-walk)
- Modify files
- Parse language-specific syntax
