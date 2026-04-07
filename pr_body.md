## 💡 Summary
Replaced `FxHasher` with `blake3::Hasher` in `tokmd-analysis-near-dup` for k-gram hashing.

## 🎯 Why (perf bottleneck)
`FxHasher` uses architecture-dependent `usize` internal state, breaking cross-platform determinism (32-bit vs 64-bit systems) in the near-duplicate winnowing algorithm.

## 📊 Proof (before/after)
N/A - Determinism fix.

## 🧭 Options considered
### Option A (recommended)
- Use workspace `blake3` dependency.
- Guarantees byte-stable output across all platforms without cargo-culting external hashes.
- Trade-offs: Minor speed bump for hashing strings, but `blake3` is extremely fast.

### Option B
- Continue using `FxHash` but try seeding it manually. Still architecture dependent due to `usize`.

## ✅ Decision
Option A was chosen.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-near-dup/Cargo.toml`
- `crates/tokmd-analysis-near-dup/src/lib.rs`

## 🧪 Verification receipts
- `cargo test -p tokmd-analysis-near-dup --no-default-features`
- `cargo clippy -p tokmd-analysis-near-dup --all-features`

## 🧭 Telemetry
- Blast radius: Near Duplicate algorithm fingerprints.

## 🗂️ .jules updates
- Updated `quality/ledger.json` and envelope.
