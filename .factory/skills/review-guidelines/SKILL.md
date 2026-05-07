# Droid Review Guidelines for tokmd

## Skill Definition

This skill guides Droid review for the tokmd repository, focusing on determinism, correctness, performance, and architectural consistency.

## Core Review Areas

### 1. Determinism Invariants

**What to check**:
- All sorted collections use `BTreeMap`, not `HashMap`
- Output ordering is descending by code lines, then by name
- JSON snapshots are byte-stable for golden tests
- Path normalization uses forward slashes on all platforms

**Finding if**:
- Any `HashMap` or `FxHashMap` appears in output-critical paths
- Sorting order is non-deterministic or undocumented
- Golden snapshot assertions fail unpredictably

**Repair**:
- Replace with `BTreeMap`
- Document sort order explicitly
- Re-run golden snapshot tests to verify byte stability

---

### 2. Schema Versioning

**What to check**:
- JSON structure changes include schema version bump
- Schema version constant is updated in both `tokmd-types` and documentation
- Receipt metadata correctly reports `schema_version`
- Bump is documented in `docs/SCHEMA.md` and `docs/schema.json`

**Finding if**:
- JSON structure changes without version bump
- Version bump in code but not in docs or vice versa
- No migration or compatibility note for breaking changes
- Old schema version is still advertised in output

**Repair**:
- Identify which receipt family was changed (core, analysis, cockpit, handoff, context, context_bundle)
- Bump the relevant constant in `tokmd-types`
- Update `CLAUDE.md` and `agents/shared/repo.md` to match
- Document change in `docs/SCHEMA.md`
- Add snapshot test for new structure

---

### 3. Dependency Tier Invariant

**What to check**:
- Lower tiers (0, 1, 2) never import from higher tiers (3, 4, 5)
- Feature flags (`git`, `content`, `walk`, `halstead`) are respected at tier boundaries
- New dependencies don't violate tier hierarchy

**Finding if**:
- Tier 1 crate depends on Tier 3+ analysis module
- Tier 0 crate depends on Tier 2 walk or format
- Feature flag is ignored at a boundary (e.g., `git` enabled unconditionally in Tier 0)

**Repair**:
- Extract shared logic to a lower tier
- Use feature flags consistently across all dependents
- Reorder imports to respect tier hierarchy

---

### 4. Output Correctness

**What to check**:
- Receipt structure matches documented schema
- All receipt fields are populated correctly
- Error cases are handled gracefully
- Edge cases (empty repos, single-file repos) are covered by tests

**Finding if**:
- Receipt has null/empty fields that should be populated
- Floating-point values lack precision specification
- Missing validation for file counts, line totals, or derived metrics
- Snapshot test doesn't cover expected output shape

**Repair**:
- Populate missing fields from data sources
- Specify precision (e.g., 2 decimals) for metrics
- Add validation assertions after computation
- Add snapshot test for edge case

---

### 5. Performance and Resource Use

**What to check**:
- Large file scans complete in reasonable time
- Memory use grows linearly with codebase size
- No unnecessary allocations in hot paths
- Git history analysis respects limits (e.g., `--since` bounds)

**Finding if**:
- Command times out on large repos
- Memory grows quadratically or worse
- Temporary allocations are retained after use
- Git log queries are unbounded or very large

**Repair**:
- Add benchmarks with `criterion` or `iai`
- Use iterators instead of vec allocations where possible
- Set reasonable limits on git history range
- Profile with `perf` or `cargo flamegraph` if needed

---

### 6. Feature Flag Boundaries

**What to check**:
- `#[cfg(feature = "git")]` gates are applied consistently
- `#[cfg(feature = "content")]` gates are applied consistently
- `#[cfg(feature = "walk")]` gates are applied consistently
- `halstead` correctly requires `content` and `walk`

**Finding if**:
- Code compiles with features disabled but references missing modules
- Feature gate is applied to a Tier 0 crate but not to Tier 1 importers
- `halstead` is enabled without enabling `content` and `walk`

**Repair**:
- Add `#[cfg(feature = "...")]` gates consistently
- Gate at all tiers that use the feature
- Ensure feature dependency is documented in `Cargo.toml`

---

### 7. Error Handling

**What to check**:
- Error messages are actionable and context-aware
- Exit codes are correct and consistent
- IO errors are propagated with full context
- Git command failures are handled gracefully

**Finding if**:
- Generic "error" messages without context
- Non-zero exit on success or zero exit on failure
- Root cause is hidden by multiple error layers
- Git command failure crashes instead of returning an error

**Repair**:
- Use `anyhow::Context` to add context
- Return appropriate exit code
- Ensure error chain includes root cause
- Wrap git command errors in a context error

---

### 8. Testing Coverage

**What to check**:
- Golden snapshots exist for all output formats
- Property-based tests cover invariants (determinism, ordering, bounds)
- Fuzz targets exist for parser-like code
- Edge cases are tested (empty, single file, deeply nested)

**Finding if**:
- New output format with no snapshot test
- No property test for data structure correctness
- Parser code with no fuzz target
- Edge case is mentioned in PR but not tested

**Repair**:
- Add snapshot test with `insta`
- Add property test with `proptest`
- Add fuzz target in `fuzz/` with seed corpus
- Add regression test for edge case

---

## Evidence Standards

When reporting findings, distinguish between:

### Observed
- Direct code inspection results
- Test output
- Static analysis
- Snapshot test failures

### Reported
- Statements in PR description
- Comments in code
- Commit messages

### Not Verified
- Claims about performance without benchmarks
- Assumptions about future behavior
- External API contracts

---

## Special Considerations for tokmd

1. **Determinism is non-negotiable** — Many use cases depend on byte-stable receipts
2. **Schema versioning is strict** — LLM context generation relies on version compatibility
3. **Tier hierarchy is enforced** — Circular dependencies or violations break integration
4. **Golden snapshots are proofs** — Visual diffs are more reliable than code review alone
5. **Feature flags are critical** — Some downstream tools disable features for size/performance

---

## Non-Goals

- Grammar and typo fixes do not require Droid comment
- Dependency updates are validated by CI; defer to semver-checks
- Formatting and linting are pre-validated; do not flag cosmetics
- Release timing and project management are out of scope
