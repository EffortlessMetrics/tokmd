# Librarian Run 2026-04-06

## Selection
- Lane: Scout Discovery
- Target: Missing CLI doctests for tokmd run, diff, and context

## Options
- **Option A**: Add a doctest for `tokmd run` and `tokmd diff` which is prominently featured in README and tutorials but lacks test coverage in `docs.rs`.
- **Option B**: Add a doctest for `tokmd context` only.

## Decision
Chose **Option A** combined with context tests to maximize value. Implemented `recipe_run_and_diff` and `recipe_context_budget` in `crates/tokmd/tests/docs.rs`.
