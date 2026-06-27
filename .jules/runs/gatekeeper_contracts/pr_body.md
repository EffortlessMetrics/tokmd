## 💡 Summary
Explicitly cover `fixtures/syntax/**` and `scripts/**` directories in the non-Rust file policy. This prevents non-Rust assets in these paths from triggering `file-policy` failures on CI and brings these testing/tooling files into the tracked asset inventory.

## 🎯 Why
Running `cargo xtask check-file-policy --strict` fails with unmatched findings for `fixtures/syntax/python/native_boundary.py`, `fixtures/syntax/typescript/component.tsx`, `fixtures/syntax/typescript/native_boundary.ts`, and `scripts/check-no-bare-self-hosted.sh`. Untracked files cause the deterministic CI gate to fail and violate the gatekeeper requirement for a strict, fully-tracked repository surface.

## 🔎 Evidence
- **File path(s):** `policy/non-rust-allowlist.toml`
- **Finding:** Missing globs cause `--strict` failures on valid test/script files.
- **Receipt:**
  ```text
  file-policy findings (4):
    - file fixtures/syntax/python/native_boundary.py does not match any non-Rust allowlist glob
    - file fixtures/syntax/typescript/component.tsx does not match any non-Rust allowlist glob
    - file fixtures/syntax/typescript/native_boundary.ts does not match any non-Rust allowlist glob
    - file scripts/check-no-bare-self-hosted.sh does not match any non-Rust allowlist glob
  ```

## 🧭 Options considered
### Option A (recommended)
- Explicitly add `fixtures/syntax/**` and `scripts/**` globs to the file policy.
- Fits the repo and shard by protecting deterministic build outputs and tracking test fixtures properly.
- **Structure:** Tightens deterministic gate assertions around repo assets.
- **Velocity:** Low risk. Fast single-file change.
- **Governance:** Brings existing tooling scripts and testing fixtures into the unified file-policy inventory.

### Option B
- Ignore the policy enforcement tool findings and let it continue failing silently or just on `--strict` mode.
- **When to choose it instead:** When the policy tool is unused or slated for removal.
- **Trade-offs:** We leak non-Rust files into the repo without tracking, violating the Gatekeeper persona's objective.

## ✅ Decision
Option A. The policy enforcement exists, and `--strict` is meant to be a valid deterministic CI target. By explicitly adding the `fixtures/syntax/**` and `scripts/**` directories, we fix `cargo xtask check-file-policy --strict` and re-secure the repository's file boundary.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`: Added explicitly owned `fixtures/syntax/**` and `scripts/**` globs.

## 🧪 Verification receipts
```text
$ python3 patch_policy.py && cargo xtask check-file-policy --strict
file-policy OK: 85 entries, 1157 non-Rust files covered, 1309 Rust files skipped
```

## 🧭 Telemetry
- **Change shape:** Policy configuration fix
- **Blast radius:** IO / Schema / Gates
- **Risk class:** Low. Policy definition only.
- **Rollback:** `git revert`
- **Gates run:** `cargo xtask check-file-policy --strict`, `cargo clippy -- -D warnings`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
