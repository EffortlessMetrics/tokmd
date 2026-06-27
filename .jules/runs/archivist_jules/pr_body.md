## 💡 Summary
I updated `cargo xtask jules-index` to parse both active friction from `.jules/friction/open/` and historical friction from `.jules/friction/done/`. This generates a more complete `FRICTION_ROLLUP.md` that keeps completed learnings accessible, fulfilling the Archivist mandate to consolidate friction and generated indexes without dropping history. I also fixed strict file policy omissions for `fixtures/**` and `scripts/**`.

## 🎯 Why
The `jules-index` command was previously only rolling up `open` friction items. However, system memory and intent explicitly dictate that `cargo xtask jules-index` should parse both `.jules/friction/open/` and `.jules/friction/done/` to generate `FRICTION_ROLLUP.md`. Excluding historical items drops valuable context from the rollup. Furthermore, `cargo xtask check-file-policy --strict` was failing due to unlisted `fixtures/` and `scripts/` directories, preventing clean validation.

## 🔎 Evidence
- file path: `xtask/src/tasks/jules_index.rs` only parsed `root.join(".jules/friction/open")`.
- `cargo xtask jules-index` output missing `done` entries.
- `cargo xtask check-file-policy --strict` failed on un-allowlisted files.

## 🧭 Options considered
### Option A (recommended)
- Update `cargo xtask jules-index` to collect and extend items from both `open` and `done` directories.
- Fix the policy gaps in `policy/non-rust-allowlist.toml` for `fixtures/` and `scripts/`.
- Fits the repo and shard because it improves Jules scaffolding directly and resolves a policy validation blocker cleanly.
- Trade-offs: Structure is improved, Governance is tightened via proper index generation and file policy alignment.

### Option B
- Only edit the `.jules/friction/done/` markdown files to merge them.
- when to choose it instead: If the goal was simply content cleanup.
- trade-offs: Misses the core structural gap in `jules-index` and fails to fix the strict file policy blocker.

## ✅ Decision
Option A. It's a proper fix for the scaffolding toolchain that fulfills the Archivist mission to consolidate run learnings into shared indexes, strictly complies with system memory regarding `jules-index` behavior, and ensures the repo passes the `core-rust`/strict policy checks cleanly.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/jules_index.rs`
- `policy/non-rust-allowlist.toml`

## 🧪 Verification receipts
```text
cargo xtask jules-index
Jules indexes written under /app/.jules/index/generated

cargo xtask check-file-policy --strict
file-policy OK: 85 entries, 1157 non-Rust files covered, 1309 Rust files skipped
```

## 🧭 Telemetry
- Change shape: Tooling & Scaffold Improvement
- Blast radius: Jules `.jules` artifacts generation and internal strict file policy.
- Risk class: Low - Does not affect production code, only helper commands and indexes.
- Rollback: Revert the PR safely.
- Gates run: `cargo xtask check-file-policy --strict`, `cargo xtask jules-index`.

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
