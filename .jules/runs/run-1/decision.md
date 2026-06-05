## Option A (recommended)
Add targeted unit tests in `crates/tokmd-format/src/redact/mod.rs` to close the `cargo mutants` gaps around path redaction logic. Specifically:
- Add a test for trailing dot normalization in `clean_path` to catch `-` vs `+` mutations.
- Add tests for hidden file extensions (e.g. `.env`, `.rs`, `.tar.gz`) to catch logical operator mutations (`==` vs `!=`) in `redact_path`.

This fits the `mutation` gate profile by strengthening behavioral assertions around meaningful code paths (path redaction boundary).
Trade-offs: Increases test code slightly, but proves correctness of edge cases without changing production code.

## Option B
Refactor `clean_path` and `redact_path` to use a simpler path manipulation crate like `std::path::Path`.
While this might reduce the need for custom string manipulation, it risks cross-platform determinism (e.g., Windows vs Linux path separators) and introduces complexity to a sensitive tier-0 function. It also doesn't directly address the missing mutation coverage without rewriting the logic.
Trade-offs: Higher risk of determinism regressions.

## Decision
Choosing Option A because it provides direct, evidence-backed coverage of the exact gaps identified by `cargo mutants` without changing the proven deterministic production code.
