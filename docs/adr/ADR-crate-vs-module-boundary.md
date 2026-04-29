# ADR: Crate vs Module Boundary

- Status: Proposed
- Date: 2026-04-29

## Context

The repo needs durable criteria for when to keep a Cargo crate versus collapsing into owner modules.

## Decision

Create a crate only when it carries an independent public/support boundary (contract, workflow, capability, dependency isolation, or semver promise). Keep implementation shards as module folders.

## Criteria

### Keep as crate
- Independent semver/API support promise.
- External consumer contract.
- Load-bearing capability/workflow boundary.

### Collapse to module
- Internal helper shard.
- Single-owner implementation detail.
- Not useful as a standalone published unit.
