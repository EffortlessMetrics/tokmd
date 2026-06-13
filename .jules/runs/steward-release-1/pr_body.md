## 💡 Summary
Added missing unclassified non-Rust files to the policy allowlist to correct drift and unblock the file-policy gate checks.

## 🎯 Why
The repository contains syntax fixtures (`fixtures/syntax/**`) and build tooling (`scripts/check-no-bare-self-hosted.sh`) that were unclassified in `policy/non-rust-allowlist.toml`, causing `cargo xtask check-file-policy --strict` to fail, thus blocking release/governance CI gates.

## 🔎 Evidence
- File: `policy/non-rust-allowlist.toml`
- Finding: `fixtures/syntax/**` and `scripts/check-no-bare-self-hosted.sh` were flagged as non-Rust files not matching any allowlist glob.
- Receipt:
  ```text
  file-policy findings (4):
    - file fixtures/syntax/python/native_boundary.py does not match any non-Rust allowlist glob
    - file fixtures/syntax/typescript/component.tsx does not match any non-Rust allowlist glob
    - file fixtures/syntax/typescript/native_boundary.ts does not match any non-Rust allowlist glob
    - file scripts/check-no-bare-self-hosted.sh does not match any non-Rust allowlist glob
  ```

## 🧭 Options considered
### Option A (recommended)
- what it is: Add the missing paths to the `policy/non-rust-allowlist.toml` with the correct kind, owner, and surface tags.
- why it fits this repo and shard: This is a direct alignment fix that resolves governance drift, squarely within the steward persona's mandate.
- trade-offs: Structure / Velocity / Governance - Zero risk, corrects governance metadata perfectly.

### Option B
- what it is: Remove the files if they are not necessary.
- when to choose it instead: If the files were accidentally committed and serve no structural purpose.
- trade-offs: More risky, these look like legitimate tree-sitter AST and CI shell fixtures. Option A is significantly safer.

## ✅ Decision
Option A. It aligns the repository's file state with the explicit policy allowlist without risking behavioral regressions.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`: Appended entries for `fixtures/syntax/**` and `scripts/check-no-bare-self-hosted.sh`.

## 🧪 Verification receipts
```text
$ cargo xtask check-file-policy --strict
file-policy OK: 85 entries, 1162 non-Rust files covered, 1309 Rust files skipped

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.13.1
  ✓ Cargo crate versions match 1.13.1.
  ✓ Cargo workspace dependency versions match 1.13.1.
  ✓ Node package manifest versions match 1.13.1.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo xtask publish --plan --verbose
=== Publish Plan ===
Workspace version: 1.13.1
Publish order (16 crates):
   1. tokmd-gate
...
  16. tokmd
Excluded crates:
  - tokmd-fuzz: NotPublishable
  - tokmd-node: NotPublishable
  - tokmd-python: NotPublishable
  - xtask: NotPublishable

$ cargo xtask docs --check
Documentation is up to date.
doc artifacts ok: 2 required doc(s), 54 family file(s), 1 active goal(s), 19 spec-index artifact(s), 0 spec-index lane(s)
```

## 🧭 Telemetry
- Change shape: Additive (configuration)
- Blast radius: build / policy checks
- Risk class + why: Lowest risk. Solely updates an allowlist used by an xtask script. No compiled binaries affected.
- Rollback: Revert the commit.
- Gates run: `cargo xtask check-file-policy --strict`, `cargo xtask version-consistency`, `cargo xtask publish --plan --verbose`, `cargo xtask docs --check`.

## 🗂️ .jules artifacts
- `.jules/runs/steward-release-1/envelope.json`
- `.jules/runs/steward-release-1/decision.md`
- `.jules/runs/steward-release-1/receipts.jsonl`
- `.jules/runs/steward-release-1/result.json`
- `.jules/runs/steward-release-1/pr_body.md`

## 🔜 Follow-ups
None
