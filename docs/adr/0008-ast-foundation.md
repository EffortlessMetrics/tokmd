# ADR-0008: AST foundation and shadow rollout

- Status: proposed
- Date: 2026-05-08

## Context

tokmd currently produces receipt-grade repository facts with deterministic
heuristics. That is intentional: the tool is useful without owning a full parser
stack, and current receipts are stable across native, binding, browser, and CI
surfaces.

The long-term roadmap calls for Tree-sitter AST integration because several
surfaces are inherently approximate without syntax trees:

- function boundary detection
- function-level complexity
- cognitive complexity
- import and API-surface extraction
- language-specific maintainability signals
- later review evidence that needs source-structure precision

AST support touches schema meaning, output stability, dependency footprint,
WASM bundle size, performance, feature flags, and fallback behavior. It should
therefore start as infrastructure and comparison evidence, not as a default
metric rewrite.

## Decision

AST work must roll out in shadow mode before it changes public receipt
semantics.

The first implementation should:

- live inside the owning analysis surface unless an independent public crate is
  justified by ADR-0002;
- start with Rust as the first language;
- use Tree-sitter behind an explicit feature flag;
- parse only a small source-structure surface at first: functions, imports, and
  simple control-flow landmarks;
- emit developer-only comparison artifacts rather than changing default
  analysis receipts;
- preserve heuristic fallback for every language and runtime that lacks AST
  support;
- keep browser/WASM support opt-in until bundle-size and initialization costs
  are measured;
- keep deterministic ordering and stable path normalization at the AST boundary.

The recommended first module shape is:

```text
crates/tokmd-analysis/src/ast/
  mod.rs
  capability.rs
  rust.rs
  shadow.rs
```

The feature flag should describe the capability rather than a historical crate
split:

```toml
[features]
ast = []
```

If Tree-sitter dependencies need isolation after measurement, a crate boundary
can be reconsidered as a capability/dependency-isolation boundary under
ADR-0002. The default starting point remains an owner module.

Shadow artifacts should be developer-facing and excluded from public receipt
contracts:

```text
target/tokmd-ast-shadow/
  heuristic.json
  ast.json
  diff.json
```

## Consequences

- Existing receipt schemas and default outputs remain stable during the AST
  foundation phase.
- AST evidence can be evaluated across real repositories before downstream
  consumers see semantic changes.
- Browser/WASM capability honesty is preserved because AST support is not
  promised until the loaded bundle actually exposes it.
- Proof scopes can target AST parsing and shadow comparison without turning AST
  precision into a global release blocker.
- A future schema bump is required if any public field changes meaning because
  AST replaces a heuristic.

## Alternatives

- Add Tree-sitter as a default analysis dependency and immediately replace
  heuristic complexity/import metrics.
- Create a new public `tokmd-ast` crate before there is external consumption or
  a proven dependency-isolation need.
- Keep AST work out of tree until a full v3 design is ready.

These alternatives were rejected for the first slice because they either risk
receipt churn, expand the public surface too early, or delay useful comparison
evidence.

## Enforcement

- AST code must remain feature-gated until the maintainers accept a production
  rollout.
- Default CLI, library, FFI, Node, Python, and WASM outputs must not change in
  the shadow phase.
- AST-derived receipt fields require schema-version review under ADR-0007.
- Any new AST dependency must be covered by proof policy scopes and publish
  surface review.
- WASM/browser AST exposure requires capability-matrix updates and bundle-size
  evidence.
- Shadow comparison output must be deterministic: sorted paths, normalized path
  separators, stable node identifiers, and no timestamps in comparison payloads.

## Related specs

- `docs/specs/ast-shadow.md`
- `docs/architecture.md`
- `docs/publish-surface.md`
- `docs/adr/0002-crate-vs-module-boundaries.md`
- `docs/adr/0006-deterministic-receipts.md`
- `docs/adr/0007-schema-family-versioning.md`
