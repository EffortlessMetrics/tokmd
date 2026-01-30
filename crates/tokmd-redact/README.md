# tokmd-redact

Redaction utilities for privacy-safe tokmd output.

## Overview

This is a **Tier 0.5** utility crate providing BLAKE3-based hashing functions for redacting sensitive information in receipts while preserving useful structure.

## Installation

```toml
[dependencies]
tokmd-redact = "1.2"
```

## Usage

```rust
use tokmd_redact::{short_hash, redact_path};

// Hash any string to 16 characters
let hash = short_hash("my-secret-path");
assert_eq!(hash.len(), 16);

// Redact path while preserving extension
let redacted = redact_path("src/secrets/config.json");
assert!(redacted.ends_with(".json"));
// Result: "a1b2c3d4e5f6g7h8.json"
```

## Functions

### `short_hash(s: &str) -> String`
Returns a 16-character BLAKE3 hex hash of the input string.

### `redact_path(path: &str) -> String`
Returns a hashed path with the file extension preserved.

## Cross-Platform Consistency

Both functions normalize path separators (`\` to `/`) before hashing, ensuring identical hashes across Windows and Unix:

```rust
assert_eq!(short_hash("src\\lib"), short_hash("src/lib"));
assert_eq!(redact_path("src\\main.rs"), redact_path("src/main.rs"));
```

## Use Cases

- Share receipts without exposing internal paths
- Privacy-safe LLM context generation
- Anonymize repository structure in reports

## Redaction Modes (in tokmd-format)

| Mode | Behavior |
|------|----------|
| `None` | Paths shown as-is |
| `Paths` | Hash file paths, preserve extension |
| `All` | Hash paths and excluded patterns |

## License

MIT OR Apache-2.0
