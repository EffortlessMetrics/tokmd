# tokmd Specification

This document records tokmd's current product contracts across CLI, library,
binding, and receipt surfaces. Normative keywords `MUST`, `SHOULD`, and `MAY`
use RFC 2119 meaning.

## Scope

tokmd is a deterministic repository inventory and analysis system. It produces
machine-readable receipts and human-readable summaries from the same scan,
model, analysis, and formatting surfaces.

tokmd is not a linter, build system, developer scoring tool, vulnerability
database, or SAST replacement. It reports repository facts, diff-aware evidence,
and degraded capability states so downstream humans or directors can decide what
to do.

## Architecture Contracts

- Public crates represent stable contracts, facades, adapters, or products.
- Implementation details SHOULD live as single-responsibility owner modules
  inside the owning crate unless a public crate boundary is justified by
  ADR-0002.
- Lower tiers MUST NOT depend on higher tiers.
- Contract/type crates MUST remain free of `clap` and product-specific CLI UX.
- `tokmd-core` is the clap-free workflow facade for embedding and FFI.

## CLI Contracts

- `tokmd` and `tokmd lang` produce language summaries.
- `tokmd module`, `export`, `run`, `analyze`, `badge`, `diff`, `cockpit`,
  `gate`, `tools`, `context`, `init`, `check-ignore`, and `completions` are
  product CLI surfaces.
- `tokmd <existing-path>` MUST preserve zero-config language-summary behavior.
- Unknown bare subcommands MUST fail as unrecognized commands instead of being
  silently treated as missing paths.
- CLI paths and diagnostics SHOULD be normalized before user-facing output.
- Optional color or terminal styling MUST respect no-color policy before it is
  enabled by default.

## Receipt Determinism

Receipt and stable renderer outputs are deterministic by contract.

- Emitted keyed data MUST use stable ordering.
- Ranked rows SHOULD sort by descending code lines, then stable name/key
  tie-break unless the surface documents a different order.
- Paths emitted in receipts MUST use forward slashes on all platforms.
- Locale-sensitive ordering MUST NOT be used for generated deterministic
  browser, HTML, or JavaScript artifacts.
- Same input corpus, options, capability set, and tokmd version SHOULD produce
  byte-stable outputs except for explicitly timestamped metadata.

## Children And Embedded Languages

Children-mode behavior is part of the receipt contract.

- `collapse` merges embedded-language children into parent totals.
- `separate` emits embedded-language rows distinctly.
- Language, module, export, run, and analysis workflows SHOULD describe or
  preserve the selected children mode consistently.

## Schema Versioning

Schema families version independently.

| Receipt family | Version identifier |
|----------------|--------------------|
| Core receipts | `SCHEMA_VERSION` |
| Analysis receipts | `ANALYSIS_SCHEMA_VERSION` |
| Cockpit receipts | `COCKPIT_SCHEMA_VERSION` |
| Handoff manifests | `HANDOFF_SCHEMA_VERSION` |
| Context receipts | `CONTEXT_SCHEMA_VERSION` |
| Context bundles | `CONTEXT_BUNDLE_SCHEMA_VERSION` |
| Sensor reports | `SENSOR_REPORT_SCHEMA` |
| Baselines | `BASELINE_VERSION` |
| Tool schemas | `TOOL_SCHEMA_VERSION` |

Breaking structure or semantic changes MUST bump the affected family version and
update the matching docs/schema references. Additive optional fields MAY remain
within the current family version when old consumers can ignore them safely.

## Git Range Semantics

- `A..B` means commits reachable from `B` but not `A`.
- `A...B` means symmetric difference from the merge base.
- Release/tag comparisons and `tokmd cockpit` / `tokmd diff` release-style
  flows SHOULD use two-dot ranges.
- CI branch-divergence workflows MAY use three-dot ranges when comparing a PR to
  its merge base.

## Binding And Runtime Contracts

- FFI JSON entrypoints return envelopes with `ok`, `data`, and `error` rather
  than panicking across the boundary.
- Python bindings return native Python dictionaries and should release the GIL
  during long scans.
- Node bindings return Promises and should move blocking work off the event
  loop.
- WASM/browser surfaces MUST report unsupported capabilities honestly instead
  of implying unavailable checks passed.
- Browser GitHub ingest caches MUST partition authenticated data without storing
  raw tokens.

## Cockpit And Review Evidence

`tokmd cockpit` is the current PR-review evidence surface.

- Cockpit receipts and comment output MUST distinguish passed evidence from
  missing, skipped, or degraded evidence.
- Mutation/cache evidence MUST be invalidated by commit/scope mismatches rather
  than reused silently.
- A separate `tokmd review` command should not duplicate cockpit semantics unless
  it has a distinct orchestrator contract.

## Validation Expectations

Changes SHOULD run the smallest relevant gate first, then broader gates when the
blast radius warrants it.

Common gates include:

- `cargo fmt-check`
- `cargo clippy -- -D warnings`
- crate-specific `cargo test -p <crate>`
- `cargo xtask docs --check`
- `cargo xtask version-consistency`
- schema/publish-surface checks for public contract changes
- web runner tests for browser/worker behavior

## Related Documents

- `docs/architecture.md`
- `docs/design.md`
- `docs/requirements.md`
- `docs/SCHEMA.md`
- `docs/schema.json`
- `docs/testing.md`
