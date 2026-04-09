# API Specification: tokmd-core Analysis Formatting Facade

**Scope:** Phase 1 ONLY — tokmd-analysis-format tier skip violation
**Target:** `crates/tokmd-core/src/lib.rs` facade extension

---

## Design Principles

1. **Minimal surface** — Re-export only what's needed; no premature abstraction
2. **Feature-gated** — All analysis formatting under `analysis` feature flag
3. **Zero breaking changes** — Additive only; existing APIs untouched
4. **Propagation transparency** — Feature flags flow through cleanly

---

## API Surface

### Option A: Direct Re-export (RECOMMENDED)

Simplest approach: re-export the exact types needed from tokmd-analysis-format.

```rust
// In crates/tokmd-core/src/lib.rs

#[cfg(feature = "analysis")]
pub mod analysis_facade {
    //! Analysis formatting re-exports for Tier 5 products.
    //!
    //! This module provides Tier 4 facade access to Tier 3 analysis formatting,
    //! maintaining tier boundary compliance for tokmd CLI and other products.
    //!
    //! ## Example
    //! ```
    //! use tokmd_core::analysis_facade::{render, RenderedOutput};
    //! use tokmd_types::AnalysisFormat;
    //! use tokmd_analysis_types::AnalysisReceipt;
    //!
    //! fn output_analysis(receipt: &AnalysisReceipt, format: AnalysisFormat) -> anyhow::Result<String> {
    //!     match render(receipt, format)? {
    //!         RenderedOutput::Text(text) => Ok(text),
    //!         RenderedOutput::Binary(_) => Err(anyhow::anyhow!("Binary output not supported here")),
    //!     }
    //! }
    //! ```
    
    /// Render an analysis receipt to the specified format.
    ///
    /// # Arguments
    /// * `receipt` — The analysis receipt to render
    /// * `format` — Target output format (Md, Json, Svg, Midi, Obj, etc.)
    ///
    /// # Returns
    /// `RenderedOutput` enum containing either text or binary data
    ///
    /// # Errors
    /// Returns error if:
    /// - Serialization fails (JSON, XML)
    /// - `fun` feature is disabled but OBJ/MIDI format requested
    pub use tokmd_analysis_format::render;
    
    /// Output container for rendered analysis.
    ///
    /// ## Variants
    /// - `Text(String)` — Textual formats: Markdown, JSON, XML, SVG, Mermaid, Tree, HTML
    /// - `Binary(Vec<u8>)` — Binary formats: MIDI (requires `fun` feature)
    pub use tokmd_analysis_format::RenderedOutput;
}
```

### Option B: Wrapped API (Alternative)

If future API stability is a concern, wrap the underlying calls:

```rust
#[cfg(feature = "analysis")]
pub mod analysis_facade {
    use anyhow::Result;
    use tokmd_analysis_types::AnalysisReceipt;
    use tokmd_analysis_format::RenderedOutput as InnerRenderedOutput;
    use tokmd_types::AnalysisFormat;
    
    /// Output container for rendered analysis.
    #[derive(Debug)]
    pub enum RenderedOutput {
        Text(String),
        Binary(Vec<u8>),
    }
    
    impl From<InnerRenderedOutput> for RenderedOutput {
        fn from(inner: InnerRenderedOutput) -> Self {
            match inner {
                InnerRenderedOutput::Text(s) => RenderedOutput::Text(s),
                InnerRenderedOutput::Binary(b) => RenderedOutput::Binary(b),
            }
        }
    }
    
    /// Render an analysis receipt to the specified format.
    pub fn render_analysis(
        receipt: &AnalysisReceipt,
        format: AnalysisFormat,
    ) -> Result<RenderedOutput> {
        tokmd_analysis_format::render(receipt, format)
            .map(Into::into)
    }
}
```

**Recommendation:** Option A (direct re-export). The tokmd-analysis-format API is stable and simple (one function, one enum). Wrapping adds boilerplate without significant benefit.

---

## Cargo.toml Changes

### tokmd-core/Cargo.toml

```toml
[features]
analysis = [
    "tokmd-analysis-types",
    "dep:tokmd-analysis",
    "dep:tokmd-analysis-format",  # <-- ADD THIS LINE
    "tokmd-analysis/git",
    "tokmd-analysis/walk",
    "tokmd-analysis/content",
    "tokmd-analysis/effort",
    "tokmd-analysis/fun",
    "tokmd-analysis/archetype",
    "tokmd-analysis/halstead",
    "tokmd-analysis/topics",
]
# ADD: fun feature propagation
fun = ["tokmd-analysis-format/fun"]

[dependencies]
# ... existing deps ...

# Optional analysis support (already present)
tokmd-analysis = { workspace = true, optional = true }
tokmd-analysis-types = { workspace = true, optional = true }
# ADD: tokmd-analysis-format as optional
tokmd-analysis-format = { workspace = true, optional = true }

# Optional cockpit support (already present)
tokmd-cockpit = { workspace = true, optional = true }
tokmd-git = { workspace = true, optional = true }
```

### tokmd/Cargo.toml

```toml
[features]
default = ["git", "walk", "content", "ui", "fun", "topics", "archetype"]
# ...
fun = [
    "tokmd-analysis/fun",
    # REMOVE: "tokmd-analysis-format/fun",
    "tokmd-core/fun",  # ADD: propagate through facade
]
# ...

[dependencies]
anyhow.workspace = true
clap = { version = "4.6.0", features = ["derive"] }
tokmd-analysis = { workspace = true }
tokmd-analysis-explain.workspace = true
# REMOVE: tokmd-analysis-format = { workspace = true }
tokmd-analysis-types.workspace = true
tokmd-analysis-grid.workspace = true
# ... rest unchanged ...
```

---

## Usage in tokmd CLI

### Before (Current)

```rust
// crates/tokmd/src/analysis_utils.rs
use std::io::IsTerminal;
use std::path::Path;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_format as analysis_format;  // <-- TIER 3 DEPENDENCY
use tokmd_analysis_grid::PresetKind;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;

pub(crate) fn write_analysis_output(
    receipt: &analysis_types::AnalysisReceipt,
    output_dir: &Path,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = analysis_format::render(receipt, format)?;  // <-- DIRECT TIER 3 CALL
    let out_path = output_dir.join(analysis_output_filename(format));
    match rendered {
        analysis_format::RenderedOutput::Text(text) => {  // <-- DIRECT TIER 3 TYPE
            std::fs::write(&out_path, text)?;
        }
        analysis_format::RenderedOutput::Binary(bytes) => {  // <-- DIRECT TIER 3 TYPE
            std::fs::write(&out_path, bytes)?;
        }
    }
    Ok(())
}

pub(crate) fn write_analysis_stdout(
    receipt: &analysis_types::AnalysisReceipt,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = analysis_format::render(receipt, format)?;  // <-- DIRECT TIER 3 CALL
    match rendered {
        analysis_format::RenderedOutput::Text(text) => {  // <-- DIRECT TIER 3 TYPE
            print!("{}", text);
        }
        analysis_format::RenderedOutput::Binary(bytes) => {  // <-- DIRECT TIER 3 TYPE
            if std::io::stdout().is_terminal() {
                anyhow::bail!("Refusing to write binary output to terminal");
            }
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&bytes)?;
        }
    }
    Ok(())
}
```

### After (Phase 1)

```rust
// crates/tokmd/src/analysis_utils.rs
use std::io::IsTerminal;
use std::path::Path;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_core::analysis_facade::{render, RenderedOutput};  // <-- TIER 4 FACADE
use tokmd_analysis_grid::PresetKind;  // <-- STILL TIER 3 (Phase 2)
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;

pub(crate) fn write_analysis_output(
    receipt: &analysis_types::AnalysisReceipt,
    output_dir: &Path,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = render(receipt, format)?;  // <-- FACADE CALL
    let out_path = output_dir.join(analysis_output_filename(format));
    match rendered {
        RenderedOutput::Text(text) => {  // <-- FACADE TYPE
            std::fs::write(&out_path, text)?;
        }
        RenderedOutput::Binary(bytes) => {  // <-- FACADE TYPE
            std::fs::write(&out_path, bytes)?;
        }
    }
    Ok(())
}

pub(crate) fn write_analysis_stdout(
    receipt: &analysis_types::AnalysisReceipt,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = render(receipt, format)?;  // <-- FACADE CALL
    match rendered {
        RenderedOutput::Text(text) => {  // <-- FACADE TYPE
            print!("{}", text);
        }
        RenderedOutput::Binary(bytes) => {  // <-- FACADE TYPE
            if std::io::stdout().is_terminal() {
                anyhow::bail!("Refusing to write binary output to terminal");
            }
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&bytes)?;
        }
    }
    Ok(())
}
```

---

## Supported Analysis Formats

The facade supports all formats defined in `tokmd_types::AnalysisFormat`:

| Format | Output Type | `fun` Required |
|--------|-------------|----------------|
| `Md` | Text | No |
| `Json` | Text | No |
| `Jsonld` | Text | No |
| `Xml` | Text | No |
| `Svg` | Text | No |
| `Mermaid` | Text | No |
| `Tree` | Text | No |
| `Html` | Text | No |
| `Obj` | Text | Yes |
| `Midi` | Binary | Yes |

---

## Testing Requirements

### Unit Tests (in tokmd-core)

```rust
#[cfg(test)]
#[cfg(feature = "analysis")]
mod analysis_facade_tests {
    use super::*;
    use tokmd_analysis_types::{AnalysisReceipt, /* ... */};
    use tokmd_types::AnalysisFormat;
    
    fn minimal_receipt() -> AnalysisReceipt {
        // ... minimal valid receipt for testing
    }
    
    #[test]
    fn render_returns_text_for_md() {
        let receipt = minimal_receipt();
        let result = analysis_facade::render(&receipt, AnalysisFormat::Md).unwrap();
        assert!(matches!(result, analysis_facade::RenderedOutput::Text(_)));
    }
    
    #[test]
    fn render_returns_text_for_json() {
        let receipt = minimal_receipt();
        let result = analysis_facade::render(&receipt, AnalysisFormat::Json).unwrap();
        assert!(matches!(result, analysis_facade::RenderedOutput::Text(_)));
    }
    
    #[cfg(feature = "fun")]
    #[test]
    fn render_returns_binary_for_midi() {
        let receipt = minimal_receipt();
        let result = analysis_facade::render(&receipt, AnalysisFormat::Midi).unwrap();
        assert!(matches!(result, analysis_facade::RenderedOutput::Binary(_)));
    }
    
    #[cfg(not(feature = "fun"))]
    #[test]
    fn render_obj_errors_without_fun() {
        let receipt = minimal_receipt();
        let result = analysis_facade::render(&receipt, AnalysisFormat::Obj);
        assert!(result.is_err());
    }
}
```

### Integration Tests (in tokmd)

```rust
// Verify all analysis output formats work through facade
#[test]
fn analysis_all_formats_through_facade() {
    // Test Md, Json, Jsonld, Xml, Svg, Mermaid, Tree, Html
    // Requires: tempdir, sample receipt, each format
}

#[cfg(feature = "fun")]
#[test]
fn analysis_fun_formats_through_facade() {
    // Test Obj, Midi formats
}
```

---

## Backward Compatibility

### For tokmd CLI Users

**No breaking changes.** The CLI behavior is identical:
- All analysis output formats work the same way
- File output and stdout output behave identically
- Feature flags (`fun`) work the same way

### For Library Consumers (tokmd-core)

**Additive change only.** New module `analysis_facade` is added. Existing APIs unchanged.

### For FFI Consumers (Python/Node)

**No impact.** FFI uses JSON API, not the Rust `render()` function.

---

## Open Questions

| Question | Resolution |
|----------|------------|
| Wrap or re-export? | **Re-export** — simpler, API is stable |
| Should `analysis_facade` be a separate module or flat in lib.rs? | **Separate module** — cleaner docs, clear feature gating |
| How to handle the remaining Tier 3 deps? | **Phases 2-3** — tokmd-analysis-grid, tokmd-analysis, tokmd-analysis-explain |

---

## References

- `crates/tokmd-analysis-format/src/lib.rs` — Source of re-exported items
- `crates/tokmd-analysis-types/src/lib.rs` — `AnalysisReceipt` definition
- `crates/tokmd-types/src/lib.rs` — `AnalysisFormat` enum definition
- ADR-001: Decision context and rationale
