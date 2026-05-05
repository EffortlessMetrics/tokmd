## 💡 Summary
Removed unnecessary string heap allocations in the `compare_integrity_rows` hot-path. By writing `bytes` and `lines` into fixed-size stack buffers instead of using `format!("{}:{}")`, we speed up integrity sorting significantly without changing the determinism of the output hash. Buffer boundaries correctly accommodate `usize::MAX` values with size 64.

## 🎯 Why
The `compare_integrity_rows` function is passed to `sort_unstable_by` and evaluated hundreds or thousands of times for large workspaces. When paths are equal, it falls back to comparing `bytes:lines` representations. The previous implementation used `format!("{}:{}", a.bytes, a.lines)` inside this hot loop, causing 2 allocations per comparison and slowing down derived metric calculation.

## 🔎 Evidence
- `crates/tokmd-analysis/src/derived/mod.rs`
- Custom `criterion` benchmark on sorting 1000 identical-path rows showed a drop from 1.89ms to 645us per run (~65% reduction in time).
```text
compare_integrity_rows_sort/old
                        time:   [1.8856 ms 1.8895 ms 1.8941 ms]
compare_integrity_rows_sort/new
                        time:   [643.22 µs 645.02 µs 646.87 µs]
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Eliminate the `format!` allocations by formatting the numbers backwards into fixed-size `[0u8; 64]` byte buffers on the stack, then comparing the slices.
- why it fits this repo and shard: Analysis metrics should execute as fast as possible without sacrificing deterministic outputs. This meets the `perf-proof` constraint by retaining exact byte-for-byte matching with the old string behavior.
- trade-offs: Increases code size slightly with a manual number formatting function, but provides high velocity and structure preservation.

### Option B
- what it is: Avoid allocations by directly returning `Ordering::Greater` or `Less` using mathematics (e.g., comparing string length mathematically via logarithm).
- when to choose it instead: When memory is extremely constrained and byte array size is problematic.
- trade-offs: Emulating lexicographical string sorting over numeric values mathematically without formatting is extremely prone to determinism drift.

## ✅ Decision
Option A was chosen. I confirmed the logic with property-based comparisons and criterion benches. It perfectly replicates the previous lexicographical string sorting but is entirely zero-allocation.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/mod.rs`: Rewrote `compare_integrity_rows` to use stack byte buffers. Added helper `write_num_rev` to format integers into the buffer.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-analysis --all-features
...
test prop_integrity_entry_count_equals_file_count ... ok
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.42s
```
```text
$ cargo clippy -p tokmd-analysis -- -D warnings
    Checking tokmd-analysis v1.10.0 (/app/crates/tokmd-analysis)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.30s
```

## 🧭 Telemetry
- Change shape: Optimization
- Blast radius: `tokmd-analysis` / Determinism
- Risk class: Low - Verified that the new comparison output matches string comparison exactly.
- Rollback: Revert to `format!("{}:{}")`
- Gates run: `cargo clippy`, `cargo test`, `cargo fmt`, `cargo bench` (local)

## 🗂️ .jules artifacts
- `.jules/runs/bolt_run_01/envelope.json`
- `.jules/runs/bolt_run_01/decision.md`
- `.jules/runs/bolt_run_01/receipts.jsonl`
- `.jules/runs/bolt_run_01/result.json`
- `.jules/runs/bolt_run_01/pr_body.md`

## 🔜 Follow-ups
None.
