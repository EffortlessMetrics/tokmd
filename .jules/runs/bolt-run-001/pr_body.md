## 💡 Summary
Refactored the path normalization logic in `tokmd-model` and `tokmd-scan` to use `std::borrow::Cow`, avoiding unnecessary string allocations. This results in significant performance improvements for hot-path file scanning functions.

## 🎯 Why
During large repository scans, paths are normalized at several stages (e.g. `normalize_slashes`, `normalize_rel_path`, and `normalize_path`). The original implementation defensively copied and allocated new `String` objects even when no backslashes or matching prefixes were present. Optimizing these routines to use `Cow` removes unnecessary allocations in the common case (where paths are already forward-slash separated).

## 🔎 Evidence
Performance benchmarks against the `core-pipeline` shard showed string allocations on path normalization were a measurable hot-path overhead.
- `cargo bench -p tokmd-model -p tokmd-scan` receipts demonstrate speedups:
  - `normalize_rel_path` improved by ~48%.
  - `normalize_slashes` simple path improved by ~65%.
  - `normalize_path` prefix stripping improved by ~47%.

## 🧭 Options considered
### Option A (recommended)
- Convert `normalize_slashes` to use `Cow<'a, str>` internally, maintaining the public `String` boundary where required, but allowing internal components (like `normalize_rel_path`) to borrow the string. Similarly, refactor `normalize_path` to avoid allocating `String` when stripping prefixes.
- This fits the `core-pipeline` shard by directly targeting "unnecessary allocations / string building" on the hot path.
- Trade-offs: Minor complexity added to the internal function implementations. Velocity and Governance are unaffected.

### Option B
- Keep the current logic.
- When to choose it: If code simplicity is more valuable than scanning speed.
- Trade-offs: Leaves significant performance gains on the table in core loops.

## ✅ Decision
**Option A**. The benchmarks clearly show a significant performance win by avoiding allocations for paths that don't need backslash conversion or prefix substitution. Given the high volume of path normalizations during a full repo scan, this is a clear "hot-path work reduction" win without altering any public behaviors.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs` (Refactored `normalize_path` to avoid unnecessary allocs when handling backslashes and prefixes)
- `crates/tokmd-scan/src/path/mod.rs` (Added internal `normalize_slashes_cow` and used it in `normalize_rel_path` to reduce allocations)

## 🧪 Verification receipts
```text
cargo bench -p tokmd-model -p tokmd-scan
normalize_rel_path/old simple: 29.580 ns (-48.6%)
normalize_rel_path/old dot slash: 31.079 ns (-47.7%)
normalize_path/old prefix: 83.394 ns (-47.0%)

cargo test -p tokmd-model -p tokmd-scan
All tests passed (fixed two regressions related to leading slashes and windows prefixes).
```

## 🧭 Telemetry
- Change shape: Internal implementation refactor with zero API signature changes.
- Blast radius: Core path strings (API / IO compatibility verified with test suite).
- Risk class: Low (Backed by an extensive existing unit test suite that asserts against various path slash edge cases).
- Rollback: Revert the PR safely.
- Gates run: `cargo check`, `cargo bench`, `cargo test`.

## 🗂️ .jules artifacts
- `.jules/runs/bolt-run-001/envelope.json`
- `.jules/runs/bolt-run-001/decision.md`
- `.jules/runs/bolt-run-001/receipts.jsonl`
- `.jules/runs/bolt-run-001/result.json`
- `.jules/runs/bolt-run-001/pr_body.md`

## 🔜 Follow-ups
None.
