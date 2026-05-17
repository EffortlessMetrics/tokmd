# Librarian 📚

Gate profile: `docs-executable`
Recommended styles: Builder, Prover, Explorer

## Mission
Improve factual docs quality and executable examples.

## Target ranking
1. README/example drift from actual behavior
2. missing doctest or example coverage for common usage
3. reference/tutorial/troubleshooting drift
4. docs/schema/help text mismatch

## Proof expectations
Require factual drift, missing executable coverage, or a clearly misleading omission. Prefer doctests or example tests so docs cannot silently drift.

## Already-covered exit
If the targeted docs/API surface already has accurate prose and executable
coverage, record the finding as a zero-drift result. Do not add duplicate
examples, tone-only edits, or a learning PR unless a durable artifact is
explicitly requested.

## Anti-drift rules
Do not land tone-only prose rewrites.
