## Target: test: lock deterministic ordering for import edges 🧪 Gatekeeper

**Persona:** Gatekeeper
**Lane:** Lane B — scout discovery (determinism / ordering fix (stable output))

### Options Considered
- **Option A:** Add secondary tie breakers to `.sort_by` loops where missing (e.g. `crates/tokmd-analysis-content/src/content.rs` on `ImportEdge`).
- **Option B:** Refactor `HashSet` usages in tests to `BTreeSet` for stability.

### Decision
I chose **Option A** because I found a legitimate determinism bug in `build_import_report`: when aggregating `ImportEdge` objects, edges were only sorted by `count` descending and `from` ascending. If a file imported modules from multiple different crates an identical number of times (e.g. `count = 1`), their order in the resulting export was completely non-deterministic due to missing the `.to` field in the tiebreaker chain.

### Changes
- Updated `crates/tokmd-analysis-content/src/content.rs` to add `.then_with(|| a.to.cmp(&b.to))` to the `edge_rows.sort_by` routine.
- Added a robust test case `import_edges_are_deterministically_sorted_by_destination` to verify this specific edge case behaviour.

### Receipts
```text
✓ cargo fmt -- --check
✓ cargo clippy -- -D warnings
✓ cargo build
✓ CI=true cargo test -p tokmd-analysis-content
```
