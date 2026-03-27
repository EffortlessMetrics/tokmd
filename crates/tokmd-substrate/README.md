# tokmd-substrate

Shared repo-substrate types for sensor computation.

## Problem
Sensors should work from one shared snapshot instead of re-running scan and diff logic.

## What it gives you
- `RepoSubstrate`, `SubstrateFile`, and `LangSummary`
- `DiffRange` for carrying changed-file context into sensors

## API / usage notes
- `tokmd-sensor::substrate_builder::build_substrate` produces this data.
- Files carry normalized paths, language, module, metrics, and diff membership.
- `src/lib.rs` is the authoritative field reference.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [tokmd-sensor](../tokmd-sensor/README.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
