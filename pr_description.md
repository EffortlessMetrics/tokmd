---
# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Optimized `normalize_path` in `tokmd-model` to reduce allocations in the hot path. The changes utilize `Cow` more effectively by checking for valid UTF-8 and existing path cleanliness before allocating new strings.

## 🎯 Why (perf bottleneck)
`normalize_path` is called for every file encountered during a scan. The previous implementation eagerly allocated strings (`to_string_lossy`) even when paths were already valid UTF-8 and clean.

## 📊 Proof (before/after)
**Benchmark:** `cargo run --release --example bench_normalize`

- **Before:** ~65ns per call
- **After:** ~61ns per call
- **Improvement:** ~6% reduction in latency per call.

## 🧭 Options considered
### Option A (recommended)
- Use `path.to_str()` to get `&str` directly when possible.
- Optimize prefix stripping logic to check `&str` first.
- Reuse `Cow::Owned` capacity if allocation is forced.
- **Why:** Safest optimization without changing API behavior.

### Option B
- Change API to return `Cow<str>` instead of `String`.
- **Trade-offs:** Would be a breaking change affecting many downstream crates.

## ✅ Decision
Chosen Option A to preserve API compatibility while gaining performance.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`: Rewrite `normalize_path` to avoid unnecessary allocations.

## 🧪 Verification receipts
```
cargo run --release --example bench_normalize --package tokmd-model
Time taken for 5000000 iterations: 307.899219ms
Average time per call: 61ns
```

## 🧭 Telemetry
- Change shape: Internal implementation detail of a pure function.
- Blast radius: Low (unit tested for regressions).
- Risk class: Low.

## 🗂️ .jules updates
- Created `.jules/bolt/ledger.json` entry.
- Created `.jules/bolt/envelopes/20260227-132931.json`.
- Created `.jules/friction/open/FRIC-20260227-001.md` (process friction).

## 📝 Notes (freeform)
Correctness verified via `cargo test -p tokmd-model` (property tests included).

## 🔜 Follow-ups
None.
---