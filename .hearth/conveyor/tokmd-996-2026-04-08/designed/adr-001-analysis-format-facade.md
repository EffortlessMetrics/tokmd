# ADR-001: Extend tokmd-core Facade with Analysis Formatting

## Status

**Proposed** → Phase 1 of 3

| Field | Value |
|-------|-------|
| ADR ID | 001 |
| Run ID | tokmd-996-2026-04-04 |
| Issue | #996 |
| Scope | Phase 1 ONLY: tokmd-analysis-format tier skip |
| Target Branch | `fix/996-tier-skip-analysis-format` |
| Gate | designed |

---

## Context

### The Violation

Per architecture.md Dependency Rule #2: "Lower tiers MUST NOT depend on higher tiers."

**Current state:**
```
Tier 5 (Product) tokmd CLI
         ↓
Tier 3 (Orchestration) tokmd-analysis-format  ← VIOLATION
```

**Violation sites:**
1. `crates/tokmd/Cargo.toml` line ~57: `tokmd-analysis-format = { workspace = true }`
2. `crates/tokmd/src/analysis_utils.rs` lines 6-7: `use tokmd_analysis_format as analysis_format;`
3. `crates/tokmd/src/analysis_utils.rs` lines 89-105: Direct usage of `analysis_format::render()` and `RenderedOutput`

### Why This Exists

The tokmd-core facade (Tier 4) provides `analyze_workflow()` for executing analysis but does NOT provide formatting capabilities for analysis receipts. The CLI must render `AnalysisReceipt` (Tier 0 contract) to various output formats (Markdown, JSON, SVG, MIDI, OBJ, etc.), forcing the direct Tier 3 dependency.

---

## Decision

**Extend tokmd-core facade to expose analysis formatting capabilities.**

This follows the existing pattern where tokmd-core provides complete workflows:
- `lang_workflow()` → returns formatted `LangReceipt`
- `module_workflow()` → returns formatted `ModuleReceipt`
- `analyze_workflow()` → returns raw `AnalysisReceipt` (gap: no formatting)

The fix adds a formatting capability to tokmd-core that wraps tokmd-analysis-format.

### Alternative Rejected: Create tokmd-analysis-core

A separate Tier 4 crate `tokmd-analysis-core` would provide clean separation but introduces:
- New crate overhead (Cargo.toml, lib.rs boilerplate, CI)
- Dependency graph complexity
- Overkill for a single-function facade extension

### Alternative Rejected: Move to tokmd-format (Tier 2)

Moving analysis formatting to tokmd-format would place Tier 3 analysis types in Tier 2, creating an upward dependency (Tier 2 → Tier 3), which is a more severe violation.

---

## Consequences

### Positive

1. **Architectural compliance** — tokmd (Tier 5) will only depend on tokmd-core (Tier 4) and contracts (Tier 0)
2. **Product layer purity** — CLI contains no direct orchestration dependencies
3. **Future maintenance** — Format changes only require facade updates, not CLI updates
4. **WASM compatibility** — tokmd-wasm (also Tier 5) can now use analysis formatting through the facade

### Neutral

1. **tokmd-core growth** — Slightly larger facade surface, justified by consistent workflow pattern
2. **Feature flag complexity** — Additional `fun` feature propagation through facade

### Negative

1. **Remaining violations** — Three other Tier 3 dependencies remain (tokmd-analysis, tokmd-analysis-explain, tokmd-analysis-grid), requiring Phases 2-3
2. **Leaky abstraction retained** — Binary output terminal check stays in CLI (acceptable: this is UI policy, not formatting logic)

---

## Implementation

### Phase 1: tokmd-analysis-format ONLY

**Hard boundary:** This ADR and implementation cover ONLY tokmd-analysis-format. Other Tier 3 violations are deferred to Phases 2-3.

**Files Modified:**

| File | Change Type | Description |
|------|-------------|-------------|
| `crates/tokmd-core/Cargo.toml` | Modify | Add `tokmd-analysis-format` to `analysis` feature; add `fun` feature propagation |
| `crates/tokmd-core/src/lib.rs` | Modify | Add `analysis_facade` module with re-exports |
| `crates/tokmd/Cargo.toml` | Modify | Remove `tokmd-analysis-format` dependency; update `fun` feature |
| `crates/tokmd/src/analysis_utils.rs` | Modify | Replace direct imports with tokmd_core facade usage |

### Feature Flag Matrix

| Feature | tokmd-core | tokmd |
|---------|-----------|-------|
| `analysis` | Enables analysis workflow + formatting | Enables analysis commands |
| `fun` | Propagates to `tokmd-analysis-format/fun` | Propagates to `tokmd-core/fun` |

**CI Matrix Required:**
- `cargo build -p tokmd` (default features)
- `cargo build -p tokmd --features fun`
- `cargo build -p tokmd --features analysis`
- `cargo build -p tokmd --features "fun,analysis"`

---

## API Specification

See `api-spec-analysis-format.md` for exact facade surface.

### Summary

```rust
// In tokmd_core::analysis_facade
pub use tokmd_analysis_format::{render, RenderedOutput};
```

Or wrapped for API stability:

```rust
pub fn render_analysis_receipt(
    receipt: &AnalysisReceipt,
    format: AnalysisFormat,
) -> anyhow::Result<RenderedOutput> {
    tokmd_analysis_format::render(receipt, format)
}
```

---

## Migration Path

See `migration-path.md` for detailed migration and backward compatibility specification.

### Before (Current)

```rust
// crates/tokmd/src/analysis_utils.rs
use tokmd_analysis_format as analysis_format;

let rendered = analysis_format::render(receipt, format)?;
match rendered {
    analysis_format::RenderedOutput::Text(text) => { ... }
    analysis_format::RenderedOutput::Binary(bytes) => { ... }
}
```

### After (Phase 1)

```rust
// crates/tokmd/src/analysis_utils.rs
use tokmd_core::analysis_facade::{render, RenderedOutput};

let rendered = render(receipt, format)?;
match rendered {
    RenderedOutput::Text(text) => { ... }
    RenderedOutput::Binary(bytes) => { ... }
}
```

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Feature flag misalignment | Medium | Medium | Explicit CI matrix for all combinations |
| Breaking FFI consumers | Low | Medium | FFI uses JSON API, not Rust internals |
| OBJ/MIDI breakage | Low | High | Test `fun` feature in CI; feature propagation verified |
| Phase 2-3 never happens | Medium | Low | Document remaining violations; tracking issue |

---

## Rollback Plan

If issues arise:
1. Revert Cargo.toml changes
2. Restore `use tokmd_analysis_format` in analysis_utils.rs
3. Remove facade re-exports from tokmd-core

All changes are additive/removal; no data migrations or API breakages for external consumers.

---

## References

- Issue: https://github.com/EffortlessMetrics/tokmd/issues/996
- Architecture: `docs/architecture.md` — Tier structure and dependency rules
- Phase 2-3: Deferred to future PRs (tokmd-analysis-grid, tokmd-analysis, tokmd-analysis-explain)

---

## Decision Log

| Date | Party | Decision |
|------|-------|----------|
| 2026-04-08 | plan-reviewer | Recommend expanded scope to all 4 violations, phased delivery |
| 2026-04-08 | maintainer-vision | GO with phased implementation, 3 separate PRs, feature flag CI matrix |
| 2026-04-08 | adr-spec-agent | Author ADR-001 with API spec and migration path |

---

*This ADR follows the tokmd architectural decision process.*
