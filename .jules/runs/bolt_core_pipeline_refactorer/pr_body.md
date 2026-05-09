## 💡 Summary
Optimized hot-path path normalization across `tokmd-scan` and `tokmd-model` to reduce unnecessary string cloning and parsing overhead.

## 🎯 Why
During path traversal and metric accumulation, path normalization is a very hot execution path. Unconditional string allocation `.into_owned()` and path parsing `Path::new()` add hidden overhead when checking patterns or standardizing relative paths. The refactor avoids this overhead by using string slice offsets and short-circuit byte scans.

## 🔎 Evidence
Benchmarks show an ~20% improvement in `normalize_path`, ~10% improvement in `normalize_rel_path`, and ~55% improvement in `is_absolute_pattern` checks:
- `crates/tokmd-scan/src/lib.rs` (`is_absolute_pattern`)
- `crates/tokmd-scan/src/path/mod.rs` (`normalize_rel_path`)
- `crates/tokmd-model/src/lib.rs` (`normalize_path`)

## 🧭 Options considered
### Option A (recommended)
- **What it is:** The path normalizers `tokmd_model::normalize_path` and `tokmd_scan::path::normalize_rel_path` currently perform unnecessary allocations when stripping `./` segments by unconditionally calling `.to_string()` or `.into_owned()` on path slices that haven't structurally mutated. Option A fixes this by returning the slice converted to a string only when the length actually changes, avoiding allocation for already-normalized paths. It also includes optimizing `is_absolute_pattern` to avoid parsing into a `Path` if string-based checks succeed.
- **Why it fits:** The `core-pipeline` shard's `tokmd-scan` and `tokmd-model` heavily process files and paths during tree traversal. These normalization functions sit in the hot-path. Benchmarks show a significant performance win by reducing allocations.
- **Trade-offs:** Structure is slightly more verbose to handle lifetimes, but velocity is preserved, and governance determinism is strictly maintained.

### Option B
- **What it is:** Swap `BTreeMap` for `HashMap` in `tokmd_model` for aggregation stages.
- **When to choose it instead:** When insertion/lookup performance is critically bottlenecking and deterministic iteration order is not required.
- **Trade-offs:** Breaks determinism and violates the shard's anti-drift constraint to preserve output determinism unless explicitly justified.

## ✅ Decision
I have chosen **Option A**. The benchmark clearly proves that avoiding unconditional string cloning and string ownership parsing in normalization fast paths yields a solid performance win (~10-50% improvements) without altering API semantics or structural outputs.

## 🧱 Changes made (SRP)
- `crates/tokmd-scan/src/lib.rs`: Moved `Path::new(pattern).is_absolute()` to the end of the short-circuiting OR expression in `is_absolute_pattern`.
- `crates/tokmd-scan/src/path/mod.rs`: Optimized string allocation in `normalize_rel_path`.
- `crates/tokmd-model/src/lib.rs`: Overhauled the prefix-stripping fallback path in `normalize_path` to utilize prefix slice offsetting without allocating intermediate strings or pushing slashes.

## 🧪 Verification receipts
```text
{"command": "rustc benches_normalize_path.rs -O && ./benches_normalize_path", "output": "Original: 335.478232ms (len: 35000000)\nOptimized: 267.37771ms (len: 35000000)"}
{"command": "rustc benches2.rs -O && ./benches2", "output": "Original normalize_rel_path: 165.514888ms (len: 53000000)\nOptimized normalize_rel_path: 148.717563ms (len: 53000000)"}
{"command": "rustc benches3.rs -O && ./benches3", "output": "Original is_absolute_pattern: 7.712177ms (true: 2000000)\nOptimized is_absolute_pattern: 3.442905ms (true: 2000000)"}
```

## 🧭 Telemetry
- Change shape: Internal structural refactoring for path allocations.
- Blast radius: `tokmd-scan` and `tokmd-model` internals; fully covered by deterministic test suites.
- Risk class: Low - logic only reduces internal string clones and avoids full object parsing via early-returns. Behavior semantics are identical.
- Rollback: Revert the PR.
- Gates run: `cargo test -p tokmd-model -p tokmd-scan`, `perf-proof` custom benches.

## 🗂️ .jules artifacts
- `.jules/runs/bolt_core_pipeline_refactorer/envelope.json`
- `.jules/runs/bolt_core_pipeline_refactorer/decision.md`
- `.jules/runs/bolt_core_pipeline_refactorer/receipts.jsonl`
- `.jules/runs/bolt_core_pipeline_refactorer/result.json`
- `.jules/runs/bolt_core_pipeline_refactorer/pr_body.md`

## 🔜 Follow-ups
None.
