# tokmd-analysis-license

License discovery for tokmd analysis receipts.

## Problem

You need license signals from metadata and text without coupling that logic to
the main analysis orchestrator.

## What it gives you

- `build_license_report`

## Integration notes

- Checks package metadata first, then falls back to license-file and text
  scanning.
- Produces a sorted list of findings plus an effective SPDX guess.
- Uses deterministic ranking so ties remain stable.

## Go deeper

### Reference

- [Source](src/lib.rs)
- [tokmd-walk](../tokmd-walk/README.md)
- [tokmd-content](../tokmd-content/README.md)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Philosophy](../../docs/explanation.md)
