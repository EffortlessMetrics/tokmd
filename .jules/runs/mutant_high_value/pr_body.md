## 💡 Summary
Closed missing mutant gaps in the core `normalize_path` utility. Added targeted BDD tests to ensure prefix stripping logic correctly handles partial matches and mixed slashes, preventing silent data corruption.

## 🎯 Why
The `normalize_path` utility in `tokmd-model` uses an optimized fast path vs slow path approach to strip prefixes. A cargo-mutants run revealed gaps where the fast path conditions could be mutated without any tests failing. Without tests covering these edge cases, a refactor could easily drop the trailing slash verification or mishandle mixed slash directions, leading to paths not being matched properly or partial path strings being erroneously stripped (e.g., stripping `project` from `project_extra`).

## 🔎 Evidence
- `crates/tokmd-model/src/lib.rs` (the `normalize_path` function)
- Missing mutants reported by `cargo-mutants -d crates/tokmd-model`:
  - `replace && with || in normalize_path`
  - `delete ! in normalize_path` (twice)

## 🧭 Options considered
### Option A (recommended)
- what it is: Add specific tests in a new file `bdd_normalize_path.rs` to cover these mutants.
- why it fits this repo and shard: It closes a concrete mutant gap in a high-value core surface within `core-pipeline`.
- trade-offs: Structure: Low overhead. Velocity: Fast tests. Governance: Aligns directly with the Mutant persona.

### Option B
- what it is: Wait for the full workspace mutant run to find a different target.
- when to choose it instead: If no mutant gaps are found in the primary crate.
- trade-offs: Substantially slower execution time per prompt.

## ✅ Decision
Chose Option A because it provides a strong, targeted proof-improvement in the core utility without expanding scope or wasting time on unrelated packages.

## 🧱 Changes made (SRP)
- Created `crates/tokmd-model/tests/bdd_normalize_path.rs`
- Added tests `normalize_path_prefix_partial_match` and `normalize_path_prefix_mixed_slashes` to verify the fast path and slow path interactions in prefix stripping.

## 🧪 Verification receipts
```text
cargo mutants -d crates/tokmd-model -F normalize_path
Found 7 mutants to test
ok       Unmutated baseline in 59s build + 30s test
7 mutants tested in 4m: 7 caught

cargo test -p tokmd-model --verbose
test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.17s
```

## 🧭 Telemetry
- Change shape: New test file
- Blast radius: None (tests only)
- Risk class + why: low (test only addition)
- Rollback: Revert the PR
- Gates run: `cargo mutants`, `cargo fmt`, `cargo clippy`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None
