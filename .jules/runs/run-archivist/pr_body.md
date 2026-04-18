## 💡 Summary
Updated `.jules/bin/build_index.py` to aggregate historical runs from `.jules/docs/ledger.json` and append them to the generated `RUNS_ROLLUP.md` index. This ensures historical truth is preserved untouched in version control, per memory rules, while still surfacing in unified telemetry and tracking indexes.

## 🎯 Why
Tooling like `build_index.py` previously only scanned the active `.jules/runs/` directory. By memory constraint, historical runs stored in `.jules/docs/ledger.json` cannot be deleted or rewritten. To bridge this gap, `build_index.py` needs to synthesize both sources so that generated rollups contain the complete picture.

## 🔎 Evidence
- file path: `.jules/bin/build_index.py`
- finding: The script looped over `.jules/runs/` but ignored `.jules/docs/ledger.json`.
- receipt: Output of `.jules/index/generated/RUNS_ROLLUP.md` after patching correctly shows historical runs (e.g., Run `36cec87d-2836-42ed-9ae1-33dbf2702319`).

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `build_index.py` to read `ledger.json` and append those historical runs to the rollup.
- why it fits this repo and shard: Safely aggregates historical records without mutating tracked files, directly addressing the prompt.
- trade-offs: Structure (completeness), Velocity (low effort), Governance (preserves truth sources).

### Option B
- what it is: Migrate the historical runs into the active `.jules/runs/` format.
- when to choose it instead: If strict format unification is preferred over history preservation.
- trade-offs: Violates constraints against rewriting tracked historical packets.

## ✅ Decision
Chose **Option A** to cleanly aggregate historical runs without violating constraints against rewriting history.

## 🧱 Changes made (SRP)
- `.jules/bin/build_index.py`: Added logic to parse `.jules/docs/ledger.json` if it exists, formatting entries to match the rollup markdown table expectations.

## 🧪 Verification receipts
```text
{"cmd": "python3 .jules/bin/build_index.py && cat .jules/index/generated/RUNS_ROLLUP.md", "status": "success"}
{"cmd": "cargo clippy -- -D warnings", "status": "success"}
{"cmd": "cargo fmt -- --check", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Tooling enhancement.
- Blast radius: Jules tooling/indexing only.
- Risk class + why: Low; does not affect product code.
- Rollback: `git restore .jules/bin/build_index.py`
- Gates run: `python3 .jules/bin/build_index.py`, `cargo clippy`, `cargo fmt`

## 🗂️ .jules artifacts
- `.jules/runs/run-archivist/envelope.json`
- `.jules/runs/run-archivist/decision.md`
- `.jules/runs/run-archivist/receipts.jsonl`
- `.jules/runs/run-archivist/result.json`
- `.jules/runs/run-archivist/pr_body.md`

## 🔜 Follow-ups
None.
