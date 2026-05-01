# Friction Item

id: FRIC-20260429-004
persona: librarian
style: builder
shard: docs
status: open

## Problem
The `cockpit_workflow` public API doctest fails with `not inside a git repository` when executed normally, requiring `no_run`.

## Evidence
- path: `crates/tokmd-core/src/lib.rs`

## Why it matters
Violates `docs-executable` gate profile expectation and risks silent drift.

## Done when
- [ ] `MockGit` trait is provided, or
- [ ] Test helper creates a temporary repository for doctests
