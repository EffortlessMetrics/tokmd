# Spec: Determinism and Timestamp Policy

Status: Draft

## Scope

Defines deterministic output expectations across receipts and runtime surfaces.

## Required rules

- Identify byte-stable outputs and allowed variability fields.
- Prohibit silent `generated_at_ms = 0`.
- Define snapshot normalization and version-stamping policy.
