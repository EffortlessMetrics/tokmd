## 💡 Summary
Removed three superseded friction items and regenerated the `.jules` indexes to reflect the cleaner state.

## 🎯 Why
The friction index and directory were polluted with items (`FRIC-20260428-001.md`, `FRIC-20260413-001.md`, and `cargo_fuzz_asan_linker_failure.md`) that had already been superseded by `fuzz_toolchain_blocker`. Cleaning these up reduces duplication and noise for future runs.

## 🔎 Evidence
- `.jules/friction/done/FRIC-20260428-001.md`
- `.jules/friction/done/FRIC-20260413-001.md`
- `.jules/friction/done/cargo_fuzz_asan_linker_failure.md`
- All files noted they were superseded by `fuzz_toolchain_blocker`.

## 🧭 Options considered
### Option A (recommended)
- Delete the superseded friction items and run `cargo xtask jules-index`.
- It fits the Archivist persona by consolidating out-of-date learnings.
- Structure: High (removes duplication). Velocity: High (quick). Governance: High (ensures a single source of truth).

### Option B
- Add a section to fuzzer README directly referencing the finding in `fuzz_toolchain_blocker`.
- Choose this if the persona itself needs immediate context on why fuzzing fails in sandbox environments.
- Trade-offs: Low value for Archivist, as `fuzz_toolchain_blocker` already exists and documents the setup requirement. Duplicate info may go out of date.

## ✅ Decision
Option A was chosen because it directly addresses the duplicated, superseded friction items, leaving the workspace cleaner.

## 🧱 Changes made (SRP)
- `rm .jules/friction/done/FRIC-20260428-001.md .jules/friction/done/FRIC-20260413-001.md .jules/friction/done/cargo_fuzz_asan_linker_failure.md`
- `cargo xtask jules-index` modified `.jules/index/generated/FRICTION_ROLLUP.md` and `.jules/index/generated/RUNS_ROLLUP.md`

## 🧪 Verification receipts
```text
$ rm .jules/friction/done/FRIC-20260428-001.md .jules/friction/done/FRIC-20260413-001.md .jules/friction/done/cargo_fuzz_asan_linker_failure.md
$ cargo xtask jules-index
Jules indexes written under /app/.jules/index/generated
$ cargo fmt -- --check
$ cargo clippy -- -D warnings
$ cargo test -p xtask
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.07s
```

## 🧭 Telemetry
- Change shape: Deletion and generation.
- Blast radius: Internal `.jules` documentation and metadata.
- Risk class: Low, only touches `.jules` state, not product code.
- Rollback: Revert the PR and regenerate the index.
- Gates run: `cargo xtask jules-index`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test -p xtask`.

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules_001/envelope.json`
- `.jules/runs/archivist_jules_001/decision.md`
- `.jules/runs/archivist_jules_001/receipts.jsonl`
- `.jules/runs/archivist_jules_001/result.json`
- `.jules/runs/archivist_jules_001/pr_body.md`

## 🔜 Follow-ups
None
