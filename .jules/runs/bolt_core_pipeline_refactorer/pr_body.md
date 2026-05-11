## 💡 Summary
Optimized `normalize_path` to avoid unnecessary hot-path allocations. The new logic skips replacing backslashes and building a trailing-slash `String` unless actually necessary, utilizing slices correctly.

## 🎯 Why
`normalize_path` is called repeatedly (for every file parent and blob child) in the model pipeline during scanning. Eliminating temporary strings reduces memory pressure and execution time for large repos.

## 🔎 Evidence
File: `crates/tokmd-model/src/lib.rs`

Performance baseline on 2,000,000 runs using `benches_normalize.rs`:
```text
Orig: 352.146506ms
Opt:  196.845623ms
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Refactor `normalize_path` to avoid unnecessary format allocations.
- why it fits this repo and shard: It targets a major core-pipeline structural component and reduces a known bottleneck area without behavior drift.
- trade-offs: Structure / Velocity / Governance - slightly more complex logical branches to correctly preserve trailing slash matching against prefixes.

### Option B
- what it is: Change `Rows::Key` to avoid allocating Strings.
- when to choose it instead: If the `FileRow` struct could easily accommodate a lifetime.
- trade-offs: Extremely invasive to the type definitions, violating the bounded complexity requirement.

## ✅ Decision
Option A. It's safe, deterministic, and proven by the benchmark to save time inside the scanner's main tight loop.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-model
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
cargo clippy -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.97s
```

## 🧭 Telemetry
- Change shape: Internal implementation detail refactor
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): None, isolated to path normalization logic inside model crate.
- Risk class + why: Low. The full test suite confirms path logic (such as windows paths and `./` prefix handling) continues correctly.
- Rollback: Revert `crates/tokmd-model/src/lib.rs`
- Gates run: `cargo build`, `cargo test`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_core_pipeline_refactorer/envelope.json`
- `.jules/runs/bolt_core_pipeline_refactorer/decision.md`
- `.jules/runs/bolt_core_pipeline_refactorer/receipts.jsonl`
- `.jules/runs/bolt_core_pipeline_refactorer/result.json`
- `.jules/runs/bolt_core_pipeline_refactorer/pr_body.md`

## 🔜 Follow-ups
None.
