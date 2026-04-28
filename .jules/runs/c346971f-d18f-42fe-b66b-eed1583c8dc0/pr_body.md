## 💡 Summary
Added missing CLI help markers to `docs/reference-cli.md` and synced the document using `cargo xtask docs --update`. Also fixed `cargo test -p xtask` which was failing because `.jules/runs` was missing from `xtask/src/tasks/gate.rs`.

## 🎯 Why
The `docs/reference-cli.md` file was missing `<!-- HELP: <command> -->` tags for several commands: `diff`, `context`, `check-ignore`, `tools`, `baseline`, and `handoff`. Because the tags were missing, `cargo xtask docs --check` failed and the documentation was outdated, which violates the `docs-executable` gate profile requiring docs and schemas to match CLI behavior. Separately, `gate_runtime_guard_keeps_curated_jules_deps_history` failed in CI because `cargo xtask gate` was missing `.jules/runs` from its runtime paths array.

## 🔎 Evidence
The `cargo xtask docs --check` command failed initially:
```text
Error: Documentation drift detected in /app/docs/reference-cli.md. Run `cargo xtask docs --update` to fix.
```

The test `gate_runtime_guard_keeps_curated_jules_deps_history` failed:
```text
thread 'gate_runtime_guard_keeps_curated_jules_deps_history' panicked at xtask/tests/xtask_deep_w74.rs:358:5:
gate should treat root .jules/runs as runtime state
```

## 🧭 Options considered
### Option A (recommended)
- Programmatically add the missing `<!-- HELP: <cmd> -->` tags and run `cargo xtask docs --update`. Add `.jules/runs` to `xtask/src/tasks/gate.rs`.
- Fits this repo and shard by ensuring documentation syncs mechanically with code and tests pass.
- Trade-offs: Structure is minimal, Velocity is high, Governance is enforced via automated tooling.

### Option B
- Only update the docs, ignoring the test failure.
- This would leave CI broken, which is unacceptable.

## ✅ Decision
Chose Option A. By placing the correct marker pairs and regenerating the file, the workspace tool automatically injects the latest text and passes the `--check` gate. Fixing the test ensures the full `xtask` test suite passes.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`
- `xtask/src/tasks/gate.rs`

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo test -p xtask
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.83s

$ cargo fmt -- --check
$ cargo clippy -- -D warnings
$ cargo check
```

## 🧭 Telemetry
- Change shape: Docs update & minor test fix
- Blast radius: `docs/reference-cli.md` and `xtask/src/tasks/gate.rs`. No API, IO, or schema impact.
- Risk class: Low + why: Only touches markdown text documentation via established xtask mechanisms and fixes a test array.
- Rollback: Revert changes.
- Gates run: `cargo xtask docs --check`, `cargo test -p xtask`, `cargo fmt -- --check`, `cargo clippy`, `cargo check`

## 🗂️ .jules artifacts
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/envelope.json`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/decision.md`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/receipts.jsonl`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/result.json`
- `.jules/runs/c346971f-d18f-42fe-b66b-eed1583c8dc0/pr_body.md`

## 🔜 Follow-ups
None.
