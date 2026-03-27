# tokmd-redact

Privacy-safe redaction helpers for tokmd output.

## Problem
Receipts need to preserve structure without leaking path details or other sensitive strings.

## What it gives you
- `short_hash`
- `redact_path`

## API / usage notes
- Use `short_hash` when you need a stable identifier.
- Use `redact_path` when you want to hide path contents but keep file-type hints.
- Path separators are normalized before hashing so Windows and Unix produce the same result.
- `src/lib.rs` is the canonical source for the redaction rules.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Troubleshooting](../../docs/troubleshooting.md)
- Reference: [src/lib.rs](src/lib.rs)
- Explanation: [Design](../../docs/design.md)
