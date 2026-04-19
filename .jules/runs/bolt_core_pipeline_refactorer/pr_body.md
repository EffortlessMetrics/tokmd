## 💡 Summary
Optimized hot-path aggregations in `tokmd-model` by switching from `BTreeMap`/`BTreeSet` to `HashMap`/`HashSet`. Output determinism is preserved by sorting the outputs explicitly. Tested and verified ~1.83x performance win in aggregation.

## 🎯 Why
Aggregating reports per language and module via `BTreeMap`/`BTreeSet` incurs an `O(log N)` penalty for every row addition. In large repos, this overhead becomes noticeable. Switching to `HashMap`/`HashSet` brings insertion time down to `O(1)`.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-model/src/lib.rs`
- Before: `BTreeMap` insertion scaled logarithmically.
- After: `HashMap` insertion is constant time.
- `cargo bench` demonstrates a speedup from 157.24 µs to 85.789 µs (~1.83x) for module and file aggregations.

## 🧭 Options considered
### Option A (recommended)
- what it is: Replace `BTreeMap`/`BTreeSet` with `HashMap`/`HashSet` and sort the result arrays at the end for deterministic output.
- why it fits this repo and shard: Directly fits the `tokmd-model` data transformation layer. Reduces total work dramatically.
- trade-offs: Structure / Velocity / Governance: Requires manually sorting the resulting output values rather than relying on natural BTree iteration. However, the final vectors were already sorted in many places, making this a clear win without significant new logic.

### Option B
- what it is: Optimize `normalize_path` to avoid `String::replace` allocation.
- when to choose it instead: If the path manipulation was overwhelmingly the largest contributor to hot path.
- trade-offs: `replace` is only called once per path (on Windows primarily), whereas tree aggregations do work per file per group, scaling worse.

## ✅ Decision
I went with Option A, replacing BTree components with Hash-based ones and verifying determinism and performance.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`: Updated `collect_in_memory_file_rows`, `collect_file_rows`, `create_lang_report_from_rows`, `create_module_report_from_rows`, and `rows_from_map` to use hash-based data structures.
- `crates/tokmd-types/src/lib.rs`: Implemented `Hash` for `FileKind` enum so it could be used efficiently in hash maps.

## 🧪 Verification receipts
```text
cargo test -p tokmd-model
cargo bench -p tokmd-model --bench model_bench
aggregation_ref/btreemap_ref time: [157.24 µs 157.46 µs 157.70 µs]
aggregation_ref/hashmap_ref time: [85.789 µs 86.192 µs 86.656 µs]
```

## 🧭 Telemetry
- Change shape: Core mapping algorithm change.
- Blast radius: API (internal map behaviors), tests. Deterministic output is protected by existing test assertions.
- Risk class + why: Low-medium. Sorting outputs explicitly avoids output order regressions.
- Rollback: Revert to previous BTreeMap data structures.
- Gates run: `cargo test -p tokmd-model`, `cargo bench -p tokmd-model --bench model_bench`.

## 🗂️ .jules artifacts
- `.jules/runs/bolt_core_pipeline_refactorer/envelope.json`
- `.jules/runs/bolt_core_pipeline_refactorer/decision.md`
- `.jules/runs/bolt_core_pipeline_refactorer/receipts.jsonl`
- `.jules/runs/bolt_core_pipeline_refactorer/result.json`
- `.jules/runs/bolt_core_pipeline_refactorer/pr_body.md`

## 🔜 Follow-ups
None.
