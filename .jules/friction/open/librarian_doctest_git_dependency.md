id: FRIC-20240508-001
persona: Librarian
style: Explorer
shard: docs
status: open

## Problem
Doctests relying on git functionality fail or lack dependencies in some isolated build modes.

## Evidence
- files / paths: `crates/tokmd-git/` docs.
- outputs: Doctest failures.
- related run ids: librarian_api_doctests

## Why it matters
Doctests are the primary API contract for library consumers; broken doctests erode trust.

## Done when
- [ ] Doctest compilation and execution works properly across all relevant test boundaries.
