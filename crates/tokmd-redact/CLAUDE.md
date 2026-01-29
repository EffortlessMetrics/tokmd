# tokmd-redact

## Purpose

Redaction utilities for privacy-safe output. This is a **Tier 0.5** utility crate for path and string hashing.

## Responsibility

- Path redaction (hash while preserving extension)
- String hashing for sensitive data
- **NOT** for general-purpose file hashing (see tokmd-content)
- **NOT** for integrity hashing

## Public API

```rust
/// Returns 16-character BLAKE3 hex hash
pub fn short_hash(s: &str) -> String

/// Returns hashed path with preserved extension
/// Example: "src/main.rs" → "a1b2c3d4e5f6g7h8.rs"
pub fn redact_path(path: &str) -> String
```

## Implementation Details

- Uses BLAKE3 for cryptographic hashing
- Truncates to 16 characters for brevity
- Preserves **final** file extension for readability
- Double extensions: `file.test.ts` → `<hash>.ts`

## Use Cases

- Sharing receipts without exposing internal paths
- Privacy-safe LLM context generation
- Anonymizing repository structure in reports

## Redaction Modes (used in tokmd-format)

- `RedactMode::None` - Paths shown as-is
- `RedactMode::Paths` - Hash file paths, preserve extension
- `RedactMode::All` - Hash paths and excluded patterns

## Dependencies

- `blake3` (fast cryptographic hashing)

## Testing

```bash
cargo test -p tokmd-redact
```

Tests cover:
- Determinism (same input → same output)
- Length validation (always 16 chars)
- Extension preservation
- Double extension handling

## Do NOT

- Use for file integrity checking (use tokmd-content)
- Use for content hashing
- Modify hash length without updating dependent crates
