# Tokmd Publish Surface and Closure Policy

Status: canonical current/target publish policy baseline.

## Why this exists (ADR-level rationale)

The current architecture moved too far from microcrate-as-architecture to microcrate-as-surface-area.

The packaging direction is now explicit:

- keep an intentional published/public surface;
- keep genuine reusable contracts as published support crates;
- absorb everything else into SRP folders inside owning public crates;
- enforce publishability with a closure proof, not package-count aesthetics.

This is the hard rule: publishing correctness is defined by dependency closure, not by a broad `publish = false` pass.

## Publication model

`publish = false` is policy-only and valid only for crates that are truly outside the crates.io closure.

For publishability, every intended public crate must have a full non-dev workspace dependency closure that references only:
- published public crates
- published support crates
- crates intentionally outside the product surface (4 non-crates.io packages)

If a public or support crate depends on anything else, that dependency must be merged into an owner module first.

## Current publish surface (36 crates published + 4 non-crates.io)

This is the current honest crates.io closure, and it matches the encoded target
publish surface.

### Supported public crates (13)

- `tokmd`
- `tokmd-analysis-types`
- `tokmd-cockpit`
- `tokmd-core`
- `tokmd-envelope`
- `tokmd-ffi-envelope`
- `tokmd-gate`
- `tokmd-io-port`
- `tokmd-sensor`
- `tokmd-settings`
- `tokmd-substrate`
- `tokmd-types`
- `tokmd-wasm`

### Published support crates (23)

- `tokmd-analysis`
- `tokmd-analysis-api-surface`
- `tokmd-analysis-complexity`
- `tokmd-analysis-content`
- `tokmd-analysis-effort`
- `tokmd-analysis-entropy`
- `tokmd-analysis-explain`
- `tokmd-analysis-format`
- `tokmd-analysis-git`
- `tokmd-analysis-halstead`
- `tokmd-analysis-html`
- `tokmd-analysis-imports`
- `tokmd-analysis-license`
- `tokmd-analysis-maintainability`
- `tokmd-analysis-near-dup`
- `tokmd-content`
- `tokmd-format`
- `tokmd-fun`
- `tokmd-git`
- `tokmd-model`
- `tokmd-scan`
- `tokmd-test-support`
- `tokmd-walk`

**Count:** 23 published support crates.

## Non-crates.io packages (intentional exceptions) (4)

- `tokmd-fuzz`
- `tokmd-node`
- `tokmd-python`
- `xtask`

**Count:** 4 non-crates.io packages.

## Target publish surface

The target public surface remains the supported public API surface. The target
support surface now matches the current closure. `target_gap` is zero.

### Target public crates (13)

Same as the current supported public crates.

### Target support crates (23)

- `tokmd-analysis`
- `tokmd-analysis-api-surface`
- `tokmd-analysis-complexity`
- `tokmd-analysis-content`
- `tokmd-analysis-effort`
- `tokmd-analysis-entropy`
- `tokmd-analysis-explain`
- `tokmd-analysis-format`
- `tokmd-analysis-git`
- `tokmd-analysis-halstead`
- `tokmd-analysis-html`
- `tokmd-analysis-imports`
- `tokmd-analysis-license`
- `tokmd-analysis-maintainability`
- `tokmd-analysis-near-dup`
- `tokmd-content`
- `tokmd-format`
- `tokmd-fun`
- `tokmd-git`
- `tokmd-model`
- `tokmd-scan`
- `tokmd-test-support`
- `tokmd-walk`

### Target gap: planned support retirements (0)

The checker hard-fails if a current support crate is not classified as either
target support or target gap.

### `tokmd-test-support`

`tokmd-test-support` remains classified as current and target support in this
baseline because it is publishable today and may be part of packaged test
reproducibility. Both publish tooling paths already ignore
`DependencyKind::Development` when computing non-dev publish closure, so
dev-dependencies alone do not force this crate into the support surface.

Changing `tokmd-test-support` back to internal/dev-only should be a focused
follow-up that decides the test reproducibility contract first.

## PR A scope guardrail

PR A is **truth-first**:

- publish-surface documentation
- closure/reporting command
- machine-readable classification
- CI `--json --verify-publish` checks

It makes only the owner-module moves that are needed to remove already-decided
packaging helper crates from the publish closure. Deeper analysis-crate
consolidation remains future work.

## Hard rule

- Do not leave non-published internal crates on the production path as `publish = false` placeholders.
- Absorb non-essential packaging noise crates into SRP module folders under the owning public crate.

## Completed target-gap folder merges

The former analysis Markdown crate now lives under
`crates/tokmd-analysis-format/src/markdown.rs`.
The former analysis assets and fun crates now live under
`crates/tokmd-analysis/src/assets/` and `crates/tokmd-analysis/src/fun/`.
The former analysis archetype, derived, fingerprint, grid, and topics support
crates now live under `crates/tokmd-analysis/src/`.
The former shared analysis utility crate is split between
`crates/tokmd-analysis-types/src/util.rs` for shared contracts/helpers and
`crates/tokmd-analysis/src/util.rs` for the owner facade.

## Publish closure audit

Run:

```bash
cargo xtask publish-surface --json
```

For CI-ready checks, run:

```bash
cargo xtask publish-surface --json --verify-publish
```

The JSON report includes:

- `summary.public_surface`
- `summary.support_surface`
- `summary.non_crates_io_packages`
- `summary.current_public_surface`
- `summary.current_support_surface`
- `summary.current_non_crates_io_surface`
- `summary.target_public_surface`
- `summary.target_support_surface`
- `summary.target_gap`
- `summary.new_unapproved_support_crates`
- per-target `non_dev_workspace_closure`
- per-target `required_public`, `required_support`, `required_internal`, `required_non_crates_io`
- `violations`
- optional `packaging_checks` (`cargo package --list`)

## Command contract for automation

CI must fail when `--json --verify-publish` yields any `violations`.
Violations include:

- non-publishable crates in the current non-dev publish closure
- current support crates not classified as target support or target gap
- stale target support or target-gap entries after a crate is retired
- Cargo packaging validation failures
