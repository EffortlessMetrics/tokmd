# tokmd-analysis-format

## Purpose

Formatting and rendering for tokmd analysis receipts. This is a **Tier 3** crate that renders analysis results in multiple formats.

## Responsibility

- Render analysis results to various formats
- Support optional fun outputs
- **NOT** for analysis computation (see tokmd-analysis)

## Public API

```rust
pub fn render(receipt: &AnalysisReceipt, format: AnalysisFormat) -> Result<RenderedOutput>

pub enum RenderedOutput {
    Text(String),
    Binary(Vec<u8>),
}
```

## Supported Formats

| Format | Output | Description |
|--------|--------|-------------|
| `Md` | Text | Markdown with tables and sections |
| `Json` | Text | Pretty-printed JSON |
| `Jsonld` | Text | JSON-LD with semantic markup |
| `Xml` | Text | XML serialization |
| `Svg` | Text | SVG visualization |
| `Mermaid` | Text | Mermaid diagram syntax |
| `Obj` | Text | 3D object file (code city) |
| `Midi` | Binary | MIDI audio rendering |
| `Tree` | Text | Directory tree visualization |
| `Html` | Text | HTML rendering |

## Feature Flags

```toml
[features]
fun = ["tokmd-fun"]  # Enable OBJ, MIDI outputs
```

**Note:** OBJ and MIDI formats return an error when requested without the `fun` feature enabled. The error message instructs users to enable the feature.

## Markdown Format Details

Sections rendered (when data present):
- Archetype with evidence
- Topics with TF scores
- Entropy suspects table
- License findings table
- Churn trends table
- Corporate domains table
- Git metrics (hotspots, bus factor, freshness)
- And more...

Tables limited to top 10 items by default.

## Dependencies

- `serde_json`, `time`
- `tokmd-analysis-types`, `tokmd-config`
- Optional: `tokmd-fun` (for MIDI/OBJ)

## Testing

```bash
cargo test -p tokmd-analysis-format
cargo test -p tokmd-analysis-format --all-features
```

Snapshot tests with `insta`.

## Do NOT

- Compute analysis metrics (use tokmd-analysis)
- Add CLI parsing logic
- Modify schema without updating version
