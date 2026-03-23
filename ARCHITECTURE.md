# Architecture Overview

`tokmd` uses a tiered microcrate workspace. That is deliberate.

The project ships a CLI, library facade, Python/Node bindings, and a browser
WASM surface over the same deterministic receipt model. Small crates keep those
surfaces composable, reduce blast radius, and make it easier to isolate optional
host-backed capabilities from pure data and analysis logic.

The exact crate count changes over time. The important constraint is not "how
many crates", but whether each crate has a clear boundary and fits the tier
rules below.

## Tier Model

The canonical detailed inventory lives in [docs/architecture.md](docs/architecture.md).
At a high level:

- Tier 0: pure contracts and shared DTOs such as `tokmd-types`,
  `tokmd-analysis-types`, `tokmd-settings`, `tokmd-envelope`, and
  `tokmd-io-port`
- Tier 1: core processing such as scanning, modeling, math, path/module-key
  derivation, redaction, and settings-to-scan translation
- Tier 2: adapters such as formatting, file walking, content scanning, git, and
  badge/progress helpers
- Tier 3: orchestration crates such as `tokmd-analysis`, its focused enrichers,
  `tokmd-cockpit`, and `tokmd-gate`
- Tier 4: facades such as `tokmd-config`, `tokmd-core`, `tokmd-ffi-envelope`,
  and `tokmd-tool-schema`
- Tier 5: products such as the CLI and language/browser bindings

## Why This Shape Exists

- Deterministic receipts are the product boundary, so low-level contracts stay
  small and reusable.
- Optional capabilities like git, content scanning, filesystem walking, and UI
  helpers are easier to keep honest when they sit behind explicit crate and
  feature boundaries.
- The same core workflows need to serve CLI, FFI, and browser/WASM callers
  without pulling clap or host-only assumptions into lower tiers.
- Smaller crates are easier to test, fuzz, mutation-check, and evolve without
  dragging unrelated surfaces along.

This structure helps both human contributors and automated tooling. It is not a
claim that every tiny distinction deserves a new crate forever.

## Architecture Rules

- Contracts stay clap-free and as pure as possible.
- Lower tiers do not depend on higher tiers.
- Optional host-backed behavior is feature-gated and capability-honest.
- Ordered inputs, normalized paths, and stable sorting take priority over local
  convenience because determinism is part of the user contract.
- Browser/WASM paths only expose modes that can stay rootless and honest about
  missing host or git capabilities.

## When To Add A Crate

Add a crate when the boundary is real:

- the code has a reusable contract that other tiers or products can consume
- the dependency set is meaningfully different or optional
- the feature can be tested and versioned with a focused surface
- the split improves dependency direction or keeps a lower tier pure

## When To Keep Code In An Existing Crate

Do not create a new crate just because code is long.

Keep code in an existing crate when:

- it is only a variation of an existing workflow or preset
- it always changes in lockstep with its parent crate
- it is formatter or glue code that does not create a new public boundary
- the split would add naming and workspace overhead without clarifying ownership

## Where To Go Deeper

- [docs/architecture.md](docs/architecture.md): detailed crate inventory and
  dependency rules
- [docs/design.md](docs/design.md): design principles and system context
- [docs/implementation-plan.md](docs/implementation-plan.md): forward-looking
  work and sequencing
- [CONTRIBUTING.md](CONTRIBUTING.md): local workflow, testing, and contribution
  guidance
