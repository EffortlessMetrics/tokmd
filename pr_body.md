# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Replaced extensive use of `push_str(&format!(...))` with `writeln!(...)` and `write!(...)` across the repository to eliminate intermediate string allocations and copying.

## 🎯 Why (perf bottleneck)
The `string.push_str(&format!(...))` pattern is a classic performance anti-pattern. `format!` allocates a new `String` on the heap, formats the content into it, and then `push_str` copies that string into the destination buffer before the temporary string is dropped. This wastes CPU cycles and memory on double-allocations, especially in hot loops generating reports and analysis cards.

## 📊 Proof (before/after)
**Structural Proof:**
Work eliminated: Creating a temporary `String` via `format!` and the subsequent copy operation. By writing directly to the destination `String` using `std::fmt::Write`, we bypass the intermediate allocation entirely. This matters because it reduces GC/allocator pressure and speeds up CLI commands like `tokmd context` and various HTML/TSV formatting exports, which process large amounts of data.

## 🧭 Options considered
### Option A (recommended)
- What it is: Replace `push_str(&format!(...))` with `write!(...)` and `writeln!(...)` from `std::fmt::Write`.
- Why it fits this repo: It is the idiomatic, zero-cost abstraction in Rust for building strings efficiently.
- Trade-offs: Structure / Velocity / Governance: Requires adding `use std::fmt::Write as _;` where `std::io::Write` is also used to avoid trait conflicts, but offers the best performance with no new dependencies.

### Option B
- What it is: Pre-allocate capacity using `String::with_capacity` but keep `push_str(&format!(...))`.
- When to choose it instead: When the format string is too complex to express cleanly in a single `write!` call, or when the string is small and the allocation overhead is negligible.
- Trade-offs: Fails to fix the core issue of the intermediate allocation.

## ✅ Decision
Chose Option A because it directly addresses the root cause of the performance bottleneck (double allocation) using idiomatic Rust without introducing external dependencies.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/commands/context.rs`
- `crates/tokmd-analysis-types/src/lib.rs`
- `crates/tokmd-analysis-html/src/lib.rs`
- `crates/tokmd-export-tree/src/lib.rs`
- `crates/tokmd-fun/src/lib.rs`
- `crates/tokmd-content/src/complexity.rs`
- Numerous integration tests and property tests.

## 🧪 Verification receipts
```json
[
  {"cmd": "cargo build", "status": "PASS", "summary": "Build results"},
  {"cmd": "cargo test", "status": "PASS", "summary": "Test results"},
  {"cmd": "cargo fmt", "status": "PASS", "summary": "Fmt results"},
  {"cmd": "cargo clippy", "status": "PASS", "summary": "Clippy check passed again after fixes"}
]
```

## 🧭 Telemetry
- Change shape: Structural refactor.
- Blast radius (API / IO / format stability / concurrency): Very low. Output determinism and strings remain exactly identical.
- Risk class + why: Low risk. Modifies only string formatting mechanics.
- Rollback: Safe to revert via git without affecting external API contracts.
- Merge-confidence gates: `cargo test`, `cargo build`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules updates
Added a run entry to `.jules/bolt/ledger.json` and created a run envelope `.jules/bolt/envelopes/20260319122613.json` to record the scheduled execution.

## 📝 Notes (freeform)
Remember to use `use std::fmt::Write as _;` when importing to prevent trait collision with `std::io::Write`.

## 🔜 Follow-ups
None.
