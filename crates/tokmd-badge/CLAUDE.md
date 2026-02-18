# tokmd-badge

## Purpose

SVG badge rendering helpers. This is a **Tier 2** utility crate.

## Responsibility

- Generate compact two-segment SVG badges from label/value pairs
- XML-escape text nodes to keep SVG valid and safe
- Compute heuristic widths from character counts
- **NOT** for badge command logic (see tokmd CLI `commands/badge.rs`)

## Public API

```rust
/// Build a compact two-segment SVG badge.
pub fn badge_svg(label: &str, value: &str) -> String
```

## Implementation Details

- Width heuristic: `(char_count * 7 + 20).max(60)` per segment
- XML escaping covers `& < > " '` for text nodes
- Height fixed at 24px, font Verdana 11px
- Left segment filled `#555` (label), right segment `#4c9aff` (value)
- Uses character count (not byte length) for correct Unicode width

## Dependencies

None (zero external dependencies).

## Testing

```bash
cargo test -p tokmd-badge
```

Tests cover:
- Label and value presence in output
- Valid SVG structure (`<svg>` ... `</svg>`)
- Dimension calculation correctness
- Text centering positions
- Width scaling with text length
- XML escaping of special characters

## Do NOT

- Add runtime dependencies (this crate must stay zero-dep)
- Add badge logic beyond SVG generation (belongs in CLI)
- Change the SVG structure without updating snapshot tests
