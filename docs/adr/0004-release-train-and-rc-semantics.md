# ADR 0004: Release Train and RC Semantics

- Status: Proposed
- Date: 2026-04-29

## Context
RC behavior needs a durable policy contract beyond workflow YAML.

## Decision scope
Define stable vs RC tag semantics, prerelease metadata behavior, `latest`/`v1`
movement, crates.io and Docker publish behavior, and rollback controls.

## Baseline guardrails
- RC must not move `v1`.
- RC must not become latest.
- RC must not publish stable Docker aliases.
