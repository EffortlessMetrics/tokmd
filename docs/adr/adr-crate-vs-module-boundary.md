# ADR: Crate vs module boundary

Status: Proposed

## Decision scope
Define when functionality should remain an independent crate versus an owner-module implementation seam.

## Crate-worthy boundary signals
- independent semver contract
- external consumer API
- public capability/workflow boundary
- contract/type boundary
- load-bearing dependency isolation

## Module-worthy boundary signals
- implementation shard
- single-owner helper
- renderer/parser/internal adapter variants
- test support leaf
- no independent support promise
