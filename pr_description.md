## 💡 Summary
Replaced owned `String` fields with borrowed `&'a str` inside `tokmd_model::Key` when collecting file rows. By creating a single path cache upfront and borrowing from it, we eliminate unnecessary string cloning for every file and every embedded language child.

## 🎯 Why (perf bottleneck)
The `collect_file_rows` aggregation loops were heavily allocating and cloning `String` objects (such as the normalized file path) for every single entry added to the deterministic `BTreeMap` used for processing. This caused an unnecessary runtime tax during hot aggregation, particularly when traversing many `Language::reports` and nested `children`.

## 📊 Proof (before/after)
A new benchmark (`collect_bench`) for `collect_file_rows` with 10,000 files and 5,000 children showed a steady performance gain from 39ms down to 36ms (~8% improvement in raw throughput on standard collection):

```
collect_file_rows_10000
                        time:   [36.409 ms 36.504 ms 36.600 ms]
                        change: [−8.0582% −7.7052% −7.3548%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## 🧭 Options considered
### Option A (recommended)
- Modify `Key` to use `&'a str` for `path` and `lang` strings instead of owned `String`. Provide an upfront `HashMap<&Path, String>` acting as an allocation arena to share valid string borrows across the subsequent aggregation iteration safely.
- Why it fits this repo: This eliminates repetitive allocations while continuing to fulfill the deterministic ordering requirements (keeping the `BTreeMap` intact for output sorting).
- Trade-offs: Minor code layout shift to prepopulate the path cache, introducing a small constant memory overhead for the hashmap that pays for itself by reducing clones.

### Option B
- Swap out the core `BTreeMap` backing the aggregation step for a `HashMap`.
- When to choose it instead: For generic fast aggregation when output order isn't required.
- Trade-offs: **Rejected**. Explicitly violates tokmd's invariant for deterministic JSON and layout output stability as per `AGENTS.md`.

## ✅ Decision
Option A was chosen. It maintains full adherence to the determinism standards via `BTreeMap` ordering while avoiding redundant string allocations along the critical path of `collect_file_rows`.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`:
  - Adjusted the lifetime signature of `Key` (`Key<'a>`) to borrow its `path` and `lang` contents instead of owning `String`.
  - In `collect_file_rows` and `collect_in_memory_file_rows`, pre-compute the normalized paths before populating the `BTreeMap`, allowing the map keys to correctly borrow those paths safely.

## 🧪 Verification receipts
```
cargo bench: PASS
cargo test: PASS
cargo build: PASS
cargo fmt: PASS
cargo clippy: PASS
```

## 🧭 Telemetry
- Change shape: Internal structure refactor within `tokmd_model`.
- Blast radius: Completely contained within `collect_file_rows` aggregation behavior. No API boundary changes. Output strings explicitly convert to `String` where ownership is handed off.
- Risk class: Low. Output determinism and layout unchanged.
- Rollback: Revert `crates/tokmd-model/src/lib.rs`.
- Merge-confidence gates: `test`, `build`, `fmt`, `clippy`, and `bench` passed successfully.

## 🗂️ .jules updates
- Initialized a run envelope at `.jules/bolt/envelopes/0d6f0122-2ed5-4e14-aa91-6312d56da6d7.json`.
- Appended a structured record to `.jules/bolt/ledger.json`.
- Drafted log summaries inside `.jules/bolt/runs/2026-03-30.md`.
