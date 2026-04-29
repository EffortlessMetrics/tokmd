# Spec: Publish Surface Classification

Status: Draft

## Purpose

Machine-checkable classification for workspace package publication boundaries.

## Classes

- `published_crates_io`
- `production_crates_io`
- `production_non_crates_io`
- `dev_tooling_only`
- `fuzz_test_only`
- `external_packaging_only`
- `removed_collapsed_module`

## Invariants

- `production_non_crates_io` must be empty unless an ADR waiver exists.
- Publish correctness is verified by non-dev dependency closure.
