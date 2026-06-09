## 💡 Summary
Refactored `normalize_slashes` and `normalize_rel_path` in `tokmd-scan` to return `std::borrow::Cow<'_, str>` instead of unconditionally allocating a `String`. This reduces unnecessary allocations in hot paths like filtering, ignore patterns, and path normalization across the core pipeline.

## 🎯 Why
Path normalization is a core operation executed repeatedly during codebase scanning. By returning a `String` unconditionally, we were forcing allocations even when paths were already correctly formatted (no backslashes or traversal segments). Returning `Cow` avoids allocation in the fast path where no modification is needed.

## 🔎 Evidence
- **File paths:** `crates/tokmd-scan/src/path/mod.rs`
- **Observed behavior:** `normalize_slashes` unconditionally called `.into_owned()` on the inner cow, and `normalize_rel_path` unconditionally called `.to_string()`.
- **Receipt:** `cargo test -p tokmd-scan` now runs with zero errors after adopting the `Cow` return types.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Update `normalize_slashes` and `normalize_rel_path` to return `Cow<'_, str>` to prevent allocating unmodified paths.
- **Why it fits**: Fits the core-pipeline shard perfectly and is a classic Bolt allocation-reduction win.
- **Trade-offs**: Requires minor adjustments to callers (`.into_owned()`) where an owned `String` is strictly necessary, but allows cheap matching where ownership isn't needed.

### Option B
- **What it is**: Focus on walking/iteration allocations deep inside git traversal.
- **When to choose it**: If API changes at the module boundary are unacceptable.
- **Trade-offs**: Likely more complex and harder to isolate as a single clear reviewer story.

## ✅ Decision
Option A was chosen as it cleanly removes an unconditional hot-path allocation while preserving correctness and fulfilling the `perf-proof` expectation of a structural win. I also implemented a clean `match` over the `Cow` variants inside `normalize_rel_path` to eliminate any possible double-allocation when falling back from `Cow::Owned`.

## 🧱 Changes made (SRP)
- `crates/tokmd-scan/src/path/mod.rs`: Updated return types for normalizers. Implemented a `match` logic to handle stripped prefixes directly returning a slice.
- `crates/tokmd-scan/src/exclude/mod.rs`: Adapted caller to take ownership only when needed.
- `crates/tokmd-scan/src/ignore_patterns.rs`: Handled `.to_string_lossy()` lifetimes around `Cow` borrowing.
- `crates/tokmd-scan/src/lib.rs`: Handled `.to_string_lossy()` lifetimes around `Cow` borrowing.
- `crates/tokmd-core/src/context_git/mod.rs` and `crates/tokmd-core/src/context/mod.rs`: Updated usages due to Cow returns.
- `crates/tokmd/src/context_pack/select/policy.rs`, `crates/tokmd/src/context_pack/select/pack.rs`, `crates/tokmd/src/context_pack/select.rs`: Adapted path callers.

## 🧪 Verification receipts
```text
cargo build --verbose
bash -c 'cargo test -p tokmd-scan -p tokmd-model -p tokmd-format'
cargo fmt -- --check && cargo clippy -- -D warnings
```

## 🧭 Telemetry
- **Change shape**: Structural allocation reduction
- **Blast radius**: `tokmd-scan` path normalization logic and direct callers.
- **Risk class**: Low, covered by existing tests.
- **Rollback**: Trivial revert of PR.
- **Gates run**: `cargo build --verbose`, `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-bolt-core-001/envelope.json`
- `.jules/runs/run-bolt-core-001/decision.md`
- `.jules/runs/run-bolt-core-001/receipts.jsonl`
- `.jules/runs/run-bolt-core-001/result.json`
- `.jules/runs/run-bolt-core-001/pr_body.md`

## 🔜 Follow-ups
None.
