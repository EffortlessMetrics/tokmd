# ADR 0002: Crate vs Module Boundary

- Status: Proposed
- Date: 2026-04-29

## Context
Prevent microcrate sprawl while preserving intentional public boundaries.

## Decision draft
Use crates for independent semver/API/capability/contract/workflow boundaries.
Use modules for implementation shards, owner-local helpers, and internal
adapters not meant to carry independent support promises.

## Follow-up
Document evaluative checklist and required migration path for crate collapse.
