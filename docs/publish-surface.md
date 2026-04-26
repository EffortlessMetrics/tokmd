# Tokmd Publish Surface and Closure Policy

Status: canonical publish-surface classification baseline.

## Why this exists (ADR-level rationale)

The current architecture moved too far from microcrate-as-architecture to microcrate-as-surface-area.

The packaging direction is explicit:

- publish product, contract, workflow, and capability boundaries;
- keep internal SRP seams as module families under owner crates;
- treat conditional public crates as pending boundary decisions, not default promises;
- keep dev/tool/binding packages off the crates.io dependency closure;
- enforce publishability with a closure proof, not package-count aesthetics.

This is the hard rule: publishing correctness is defined by dependency closure, not by a broad `publish = false` pass.

A published crate is a support promise. A module folder is an architecture seam.

## Publication model

`publish = false` is policy-only and valid only for crates that are truly outside the crates.io closure.

For publishability, every intended public crate must have a full non-dev workspace dependency closure that references only:
- classified published crates
- conditional public crates while their boundary memo is still pending
- crates intentionally outside the product surface only when they are not in the non-dev closure

If a published crate depends on anything else through a normal or build dependency, that dependency must be classified for publication or merged into an owner module first. Dev-dependencies are not part of the publish closure.

## Forward policy classes

These are the policy classes the checker now reports. The older `public_surface`
and `support_surface` JSON fields remain compatibility fields for existing
automation.

### Public product crates (3)

- `tokmd`
- `tokmd-core`
- `tokmd-wasm`

### Public contract crates (5)

- `tokmd-analysis-types`
- `tokmd-envelope`
- `tokmd-io-port`
- `tokmd-settings`
- `tokmd-types`

### Public workflow crates (3)

- `tokmd-cockpit`
- `tokmd-gate`
- `tokmd-sensor`

### Public capability crates (5)

- `tokmd-analysis`
- `tokmd-format`
- `tokmd-git`
- `tokmd-model`
- `tokmd-scan`

### Conditional public crates (5)

These packages need focused boundary memos before the repo decides whether they
remain public or collapse into owner modules.

- `tokmd-content`
- `tokmd-ffi-envelope`
- `tokmd-fun`
- `tokmd-substrate`
- `tokmd-walk`

### Internal module families still packaged today (4)

These are current crates.io package boundaries that should be treated as
transitional implementation seams, not the desired final registry surface.

- `tokmd-analysis-explain`
- `tokmd-analysis-format`
- `tokmd-analysis-git`
- `tokmd-analysis-html`

### Dev-only package under policy review (1)

- `tokmd-test-support`

It remains publishable in the compatibility support surface until a focused
test reproducibility decision changes that policy.

## Current compatibility surface (26 crates published + 4 non-crates.io)

This is the current honest crates.io closure. It matches the encoded
compatibility target, but it is not the final product/contract/capability model.

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

### Published support crates (13, compatibility classification)

- `tokmd-analysis`
- `tokmd-analysis-explain`
- `tokmd-analysis-format`
- `tokmd-analysis-git`
- `tokmd-analysis-html`
- `tokmd-content`
- `tokmd-format`
- `tokmd-fun`
- `tokmd-git`
- `tokmd-model`
- `tokmd-scan`
- `tokmd-test-support`
- `tokmd-walk`

**Count:** 13 published support crates.

Support is now a compatibility classification for existing automation. It is
not the final desired category.

## Non-crates.io packages (intentional exceptions) (4)

- `tokmd-fuzz`
- `tokmd-node`
- `tokmd-python`
- `xtask`

**Count:** 4 non-crates.io packages.

## Compatibility target surface

The compatibility target public surface remains the supported public API
surface. The compatibility support surface now matches the current closure.
`target_gap` is zero.

### Target public crates (13)

Same as the current supported public crates.

### Target support crates (13)

- `tokmd-analysis`
- `tokmd-analysis-explain`
- `tokmd-analysis-format`
- `tokmd-analysis-git`
- `tokmd-analysis-html`
- `tokmd-content`
- `tokmd-format`
- `tokmd-fun`
- `tokmd-git`
- `tokmd-model`
- `tokmd-scan`
- `tokmd-test-support`
- `tokmd-walk`

### Target gap: planned compatibility support retirements (0)

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

## Scope guardrail

Publish-surface policy work is **truth-first**:

- publish-surface documentation
- closure/reporting command
- machine-readable classification
- CI `--json --verify-publish` checks

Crate-collapse work should stay in focused follow-ups. Deeper analysis-crate
consolidation, renderer migration, and gray-zone decisions remain future work.

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
The former analysis complexity, entropy, halstead, license, and
maintainability crates now live under `crates/tokmd-analysis/src/`.
The former analysis API surface, effort, and near-duplicate crates now live
under `crates/tokmd-analysis/src/`.
The former analysis content and imports crates now live under
`crates/tokmd-analysis/src/content/` and `crates/tokmd-analysis/src/imports/`.

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
- `summary.public_product_crates`
- `summary.public_contract_crates`
- `summary.public_workflow_crates`
- `summary.public_capability_crates`
- `summary.conditional_public_crates`
- `summary.internal_module_families`
- `summary.dev_only_packages`
- `summary.new_unclassified_packages`
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
- workspace packages not classified in the forward policy model
- stale forward policy entries after a package is removed
- Cargo packaging validation failures
