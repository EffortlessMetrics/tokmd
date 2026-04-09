# Migration Path: tokmd-analysis-format Tier Skip Fix

**Scope:** Phase 1 ONLY — tokmd-analysis-format violation  
**Target:** Remove direct Tier 3 dependency from tokmd CLI  
**Approach:** Route through tokmd-core facade

---

## Migration Overview

### Current State (Violation)

```
┌─────────────────┐
│ tokmd (Tier 5)  │  Product: CLI binary
│                 │  analysis_utils.rs → uses tokmd_analysis_format directly
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ tokmd-analysis- │  Tier 3: Orchestration (VIOLATION)
│ format          │  render(), RenderedOutput
└─────────────────┘
```

### Target State (Compliant)

```
┌─────────────────┐
│ tokmd (Tier 5)  │  Product: CLI binary
│                 │  analysis_utils.rs → uses tokmd_core::analysis_facade
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ tokmd-core      │  Tier 4: Facade (COMPLIANT)
│                 │  analysis_facade module re-exports from Tier 3
│                 │  render(), RenderedOutput
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ tokmd-analysis- │  Tier 3: Orchestration (now indirect)
│ format          │  Actual implementation
└─────────────────┘
```

---

## Step-by-Step Migration

### Step 1: Extend tokmd-core Cargo.toml

**File:** `crates/tokmd-core/Cargo.toml`

**Changes:**

```toml
[features]
analysis = [
    "tokmd-analysis-types",
    "dep:tokmd-analysis",
    "dep:tokmd-analysis-format",  # <-- ADD
    "tokmd-analysis/git",
    "tokmd-analysis/walk",
    "tokmd-analysis/content",
    "tokmd-analysis/effort",
    "tokmd-analysis/fun",
    "tokmd-analysis/archetype",
    "tokmd-analysis/halstead",
    "tokmd-analysis/topics",
]
# ADD new feature for fun propagation
fun = ["tokmd-analysis-format/fun"]

[dependencies]
# ... existing deps ...

# ADD optional dependency
tokmd-analysis-format = { workspace = true, optional = true }
```

**Verification:**
```bash
cargo check -p tokmd-core --features analysis
cargo check -p tokmd-core --features "analysis,fun"
```

---

### Step 2: Add analysis_facade Module to tokmd-core

**File:** `crates/tokmd-core/src/lib.rs`

**Changes:**

Add after existing feature-gated modules (after cockpit section, before "Legacy API"):

```rust
// =============================================================================
// Analysis formatting facade (requires `analysis` feature)
// =============================================================================

/// Analysis formatting re-exports for Tier 5 products.
///
/// This module provides Tier 4 facade access to Tier 3 analysis formatting,
/// maintaining tier boundary compliance for tokmd CLI and other products.
///
/// ## Example
///
/// ```rust,no_run
/// use tokmd_core::analysis_facade::{render, RenderedOutput};
/// use tokmd_types::AnalysisFormat;
/// use tokmd_analysis_types::AnalysisReceipt;
///
/// fn format_analysis(receipt: &AnalysisReceipt, format: AnalysisFormat) -> anyhow::Result<String> {
///     match render(receipt, format)? {
///         RenderedOutput::Text(text) => Ok(text),
///         RenderedOutput::Binary(_) => Err(anyhow::anyhow!("Binary output not supported")),
///     }
/// }
/// ```
#[cfg(feature = "analysis")]
pub mod analysis_facade {
    /// Render an analysis receipt to the specified format.
    ///
    /// # Type Parameters
    /// * Implicit via `AnalysisReceipt` and `AnalysisFormat`
    ///
    /// # Arguments
    /// * `receipt` — The analysis receipt to render (from `tokmd_analysis_types`)
    /// * `format` — Target output format (from `tokmd_types::AnalysisFormat`)
    ///
    /// # Returns
    /// `RenderedOutput` enum containing either text or binary data
    ///
    /// # Errors
    /// Returns error if:
    /// - JSON/XML serialization fails
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

**Verification:**
```bash
cargo doc -p tokmd-core --features analysis --no-deps
cargo test -p tokmd-core --features analysis
```

---

### Step 3: Update tokmd Cargo.toml

**File:** `crates/tokmd/Cargo.toml`

**Changes:**

```toml
[features]
default = ["git", "walk", "content", "ui", "fun", "topics", "archetype"]
alias-tok = []
git = [
    "tokmd-analysis/git",
    "dep:tokmd-git",
    "dep:tokmd-cockpit",
    "tokmd-cockpit/git",
    "tokmd-context-git/git",
]
walk = ["tokmd-analysis/walk"]
content = ["tokmd-analysis/content"]
# UPDATE: Remove tokmd-analysis-format/fun, add tokmd-core/fun
fun = [
    "tokmd-analysis/fun",
    # REMOVED: "tokmd-analysis-format/fun",
    "tokmd-core/fun",  # ADD
]
topics = ["tokmd-analysis/topics"]
archetype = ["tokmd-analysis/archetype"]
ui = ["dep:dialoguer", "dep:console", "dep:toml", "tokmd-progress/ui"]

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

**Verification:**
```bash
# Check Cargo.toml is valid
cargo check -p tokmd
cargo check -p tokmd --features fun
```

---

### Step 4: Update tokmd analysis_utils.rs

**File:** `crates/tokmd/src/analysis_utils.rs`

**Changes:**

Replace the import section and update usage:

```rust
// BEFORE:
use std::io::IsTerminal;
use std::path::Path;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_format as analysis_format;  // <-- REMOVE
use tokmd_analysis_grid::PresetKind;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;

// AFTER:
use std::io::IsTerminal;
use std::path::Path;

use anyhow::Result;
use tokmd_analysis as analysis;
use tokmd_analysis_grid::PresetKind;
use tokmd_analysis_types as analysis_types;
use tokmd_config as cli;
// ADD: facade import
use tokmd_core::analysis_facade::{render, RenderedOutput};
```

Then update the `write_analysis_output()` function:

```rust
// BEFORE:
pub(crate) fn write_analysis_output(
    receipt: &analysis_types::AnalysisReceipt,
    output_dir: &Path,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = analysis_format::render(receipt, format)?;
    let out_path = output_dir.join(analysis_output_filename(format));
    match rendered {
        analysis_format::RenderedOutput::Text(text) => {
            std::fs::write(&out_path, text)?;
        }
        analysis_format::RenderedOutput::Binary(bytes) => {
            std::fs::write(&out_path, bytes)?;
        }
    }
    Ok(())
}

// AFTER:
pub(crate) fn write_analysis_output(
    receipt: &analysis_types::AnalysisReceipt,
    output_dir: &Path,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = render(receipt, format)?;  // <-- facade call
    let out_path = output_dir.join(analysis_output_filename(format));
    match rendered {
        RenderedOutput::Text(text) => {  // <-- facade type
            std::fs::write(&out_path, text)?;
        }
        RenderedOutput::Binary(bytes) => {  // <-- facade type
            std::fs::write(&out_path, bytes)?;
        }
    }
    Ok(())
}
```

And update `write_analysis_stdout()`:

```rust
// BEFORE:
pub(crate) fn write_analysis_stdout(
    receipt: &analysis_types::AnalysisReceipt,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = analysis_format::render(receipt, format)?;
    match rendered {
        analysis_format::RenderedOutput::Text(text) => {
            print!("{}", text);
        }
        analysis_format::RenderedOutput::Binary(bytes) => {
            if std::io::stdout().is_terminal() {
                anyhow::bail!(
                    "Refusing to write binary output (format: {:?}) to a terminal to prevent rendering garbage characters. Please redirect stdout to a file (e.g., `> output.bin`) or specify an output directory.",
                    format
                );
            }
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&bytes)?;
        }
    }
    Ok(())
}

// AFTER:
pub(crate) fn write_analysis_stdout(
    receipt: &analysis_types::AnalysisReceipt,
    format: tokmd_types::AnalysisFormat,
) -> Result<()> {
    let rendered = render(receipt, format)?;  // <-- facade call
    match rendered {
        RenderedOutput::Text(text) => {  // <-- facade type
            print!("{}", text);
        }
        RenderedOutput::Binary(bytes) => {  // <-- facade type
            if std::io::stdout().is_terminal() {
                anyhow::bail!(
                    "Refusing to write binary output (format: {:?}) to a terminal to prevent rendering garbage characters. Please redirect stdout to a file (e.g., `> output.bin`) or specify an output directory.",
                    format
                );
            }
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&bytes)?;
        }
    }
    Ok(())
}
```

**Verification:**
```bash
cargo check -p tokmd
cargo check -p tokmd --features fun
cargo clippy -p tokmd
```

---

### Step 5: Pre-commit Protocol

Run the standard pre-commit checks:

```bash
# Format
cargo fmt --all

# Check
cargo check --all-features

# Clippy
cargo clippy --all-features -- -D warnings

# Tests
cargo test -p tokmd-core --features analysis
cargo test -p tokmd

# Build verification
cargo build -p tokmd
cargo build -p tokmd --features fun
```

---

### Step 6: Feature Flag CI Matrix

Required CI test matrix (all must pass):

| Command | Purpose |
|---------|---------|
| `cargo build -p tokmd` | Default features |
| `cargo build -p tokmd --features fun` | Fun format support (OBJ/MIDI) |
| `cargo build -p tokmd --features analysis` | Analysis workflows |
| `cargo build -p tokmd --no-default-features` | Minimal build |
| `cargo test -p tokmd` | Default test suite |
| `cargo test -p tokmd --features fun` | Fun format tests |

---

## Backward Compatibility

### API Compatibility

| Consumer | Impact | Mitigation |
|----------|--------|------------|
| CLI users | None | Behavior identical |
| Library (tokmd-core) | None | Additive only |
| FFI (Python/Node) | None | Uses JSON API |
| WASM (tokmd-wasm) | Positive | Can now use facade |

### Feature Flag Compatibility

**Before:**
```toml
# tokmd enables tokmd-analysis-format/fun directly
fun = ["tokmd-analysis/fun", "tokmd-analysis-format/fun"]
```

**After:**
```toml
# tokmd enables tokmd-core/fun which propagates to tokmd-analysis-format/fun
fun = ["tokmd-analysis/fun", "tokmd-core/fun"]
```

Feature behavior is identical; just the propagation path changes.

---

## Rollback Plan

If issues are discovered post-merge:

1. **Revert commits:**
   ```bash
   git revert HEAD  # or range of commits
   ```

2. **Restore direct dependency (emergency patch):**
   ```toml
   # Add back to tokmd/Cargo.toml
   tokmd-analysis-format = { workspace = true }
   
   # Restore imports in analysis_utils.rs
   use tokmd_analysis_format as analysis_format;
   ```

3. **Verification of rollback:**
   ```bash
   cargo check -p tokmd
   cargo test -p tokmd
   ```

---

## Success Criteria

| Criterion | Verification Command |
|-----------|---------------------|
| Cargo.toml clean | `grep -c "tokmd-analysis-format" crates/tokmd/Cargo.toml` → 0 |
| Compilation | `cargo build -p tokmd` succeeds |
| Tests pass | `cargo test -p tokmd` passes |
| Feature parity | `cargo test -p tokmd --features fun` passes |
| Architecture compliance | tokmd only depends on Tier 4 + Tier 0 (except deferred violations) |

---

## Remaining Work (Phases 2-3)

After Phase 1 merges, these Tier 3 dependencies remain:

| Dependency | Usage Location | Phase |
|------------|----------------|-------|
| `tokmd-analysis-grid` | `analysis_utils.rs:7` | Phase 2 |
| `tokmd-analysis` | `analysis_utils.rs:5` | Phase 3 |
| `tokmd-analysis-explain` | `commands/analyze.rs:2` | Phase 3 |

**Phase 2 scope:** Re-export `PresetKind` from tokmd-core or move mapping logic.

**Phase 3 scope:** Extend tokmd-core facade with analysis execution helpers for remaining dependencies.

---

## References

- ADR-001: Architecture decision and rationale
- api-spec-analysis-format.md: Detailed API specification
- Issue #996: Original tier skip violation report
- `docs/architecture.md`: Tier structure and dependency rules
