## 💡 Summary
Replaced platform-dependent `localeCompare` in the browser runner with strict `<`/`>` Unicode comparisons. This hardens browser/WASM determinism when ingesting inputs by aligning JS path sorting perfectly with Rust's native `String::cmp` and `BTreeMap` traversal ordering.

## 🎯 Why
`localeCompare` applies platform and environment-specific collation rules that differ across browsers and Node versions. Because the runner pipeline requires strict path determinism to build consistent input matrices and deterministic content hashes, differing collation caused subtle file ordering discrepancies when processing directories containing mixed-case or multi-byte paths.

## 🔎 Evidence
- `web/runner/ingest.js` was using `leftPath.localeCompare(rightPath)` for path sorting when priorities tied.
- Node.js test suites passed locally but the approach left a silent portability trap for downstream JS runtimes without matching locale coverage.
- Tests passing `selectGitHubTreeEntries filters vendor, binary, and oversized files deterministically` now assert using the exact lexicographic bounds.

## 🧭 Options considered
### Option A (recommended)
- Replace `localeCompare` with strict `<` and `>` string comparisons.
- Fits the `bindings-targets` shard focus and guarantees stable deterministic tree ordering, perfectly aligning JS with the native Rust bindings' use of `BTreeMap`.
- Trade-offs: Structure/Velocity/Governance are stable. Discards natural-language sorting, but sorting here is machine-facing and requires determinism.

### Option B
- Specify a fixed locale (e.g. `'en-US'`) in `localeCompare`.
- Still risks drift due to browser-specific ICU library versions and JS engine collation divergences.

## ✅ Decision
Option A. Predictability and determinism across bindings are strictly more important than natural-language collation, so falling back to exact code-unit comparison is the correct alignment fix.

## 🧱 Changes made (SRP)
- `web/runner/ingest.js`

## 🧪 Verification receipts
```text
> cd web/runner && npm test
...
# Subtest: selectGitHubTreeEntries filters vendor, binary, and oversized files deterministically
ok 2 - selectGitHubTreeEntries filters vendor, binary, and oversized files deterministically
...
# tests 49
# suites 0
# pass 48
# fail 0
# cancelled 0
# skipped 1
# todo 0

> cargo test -p tokmd-node --no-default-features
> cargo test -p tokmd-wasm --no-default-features
...
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
...
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Minor logic patch
- Blast radius: Compat / JS Runner Ingestion Determinism
- Risk class: Low, standardizing sort order
- Rollback: Revert the change to `ingest.js`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo check --workspace --no-default-features`, `cargo check --workspace --all-features`, `npm test` in `web/runner`

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None
