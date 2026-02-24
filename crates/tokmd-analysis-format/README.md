# tokmd-analysis-format

Formatting and rendering for tokmd analysis receipts.

## Overview

This is a **Tier 3** crate that renders analysis results in multiple formats. It transforms `AnalysisReceipt` structures into human-readable or machine-processable outputs.

The HTML output path is delegated to `tokmd-analysis-html` to keep rendering concerns single-purpose.

## Installation

```toml
[dependencies]
tokmd-analysis-format = "1.3"

# Enable fun outputs (OBJ, MIDI)
[dependencies.tokmd-analysis-format]
version = "1.3"
features = ["fun"]
```

## Usage

```rust
use tokmd_analysis_format::{render, RenderedOutput};
use tokmd_types::AnalysisFormat;

let output = render(&receipt, AnalysisFormat::Md)?;

match output {
    RenderedOutput::Text(s) => println!("{}", s),
    RenderedOutput::Binary(b) => std::fs::write("output.midi", b)?,
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
| `Obj` | Text | 3D code city (OBJ format) |
| `Midi` | Binary | Audio representation |
| `Tree` | Text | Directory tree visualization |
| `Html` | Text | Self-contained HTML report |

## Markdown Output

Sections rendered when data is present:
- Archetype with evidence
- Topics with TF scores
- Entropy suspects table
- License findings table
- Churn trends table
- Corporate domains table
- Git metrics (hotspots, bus factor, freshness)
- COCOMO estimates
- Distribution statistics
- Top offenders

Tables limited to top 10 items by default.

## Feature Flags

```toml
[features]
fun = ["tokmd-fun"]  # Enable OBJ, MIDI outputs
```

## Output Types

```rust
pub enum RenderedOutput {
    Text(String),    // Most formats
    Binary(Vec<u8>), // MIDI
}
```

## License

MIT OR Apache-2.0
