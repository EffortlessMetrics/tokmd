# Spec: Publish Surface Classification

Status: Draft
Date: 2026-04-29

## Purpose
Machine-checkable classification and invariants for publish-surface policy.

## Required classes
- published_crates_io
- production_crates_io
- production_non_crates_io
- dev_tooling_only
- fuzz_test_only
- external_packaging_only
- removed_collapsed_module

## Invariant
`production_non_crates_io` must be empty unless an ADR waiver exists.
