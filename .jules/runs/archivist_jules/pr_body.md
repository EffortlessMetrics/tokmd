## 💡 Summary
Learning PR: The intended patch to `.jules/bin/build_index.py` to generate `FRICTION_ROLLUP.md` was superseded by PR #1606. I have backed out the redundant patch and captured the workflow edge case as a friction item.

## 🎯 Why
During the review cycle, it was noted that PR #1606 already merged active-friction indexing into `.jules/bin/build_index.py` on main. Proceeding with the local patch would create unnecessary conflicts and duplicate work.

## 🔎 Evidence
- file path: `.jules/bin/build_index.py`
- observed behavior: The intended fix was superseded by another merged PR.
- receipt: PR Comment: "Superseded by #1606, which merged generated active-friction indexing through .jules/bin/build_index.py on current main."

## 🧭 Options considered
### Option A (recommended)
- what it is: Gracefully abort the redundant fix and create a learning PR.
- why it fits this repo and shard: Follows the explicit rule to not force fake fixes or redundant work when a patch is superseded.
- trade-offs: Structure / Velocity / Governance. Prioritizes velocity and governance over landing duplicate lines of code.

### Option B
- what it is: Force the patch anyway.
- when to choose it instead: Never.
- trade-offs: Creates merge conflicts and wastes reviewer time.

## ✅ Decision
Option A. Captured the learning and friction item.

## 🧱 Changes made (SRP)
- `.jules/friction/open/FRIC-20260502-001.md`

## 🧪 Verification receipts
```text
{"command": "python3 patch_build_index.py", "output": "success"}
{"command": "python3 .jules/bin/build_index.py", "output": "success"}
{"command": "cat .jules/index/generated/FRICTION_ROLLUP.md", "output": "success"}
{"command": "cargo xtask docs --check", "output": "success"}
{"command": "cargo fmt -- --check", "output": "success"}
{"command": "cargo clippy -- -D warnings", "output": "success"}
{"command": "cargo check", "output": "success"}
```

## 🧭 Telemetry
- Change shape: Learning PR and Friction Item
- Blast radius: Isolated to internal `.jules/` artifacts.
- Risk class + why: None. No product code changes.
- Rollback: N/A
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy`, `cargo check`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`
- `.jules/friction/open/FRIC-20260502-001.md`

## 🔜 Follow-ups
None.
