## 💡 Summary
Updated `.jules/bin/build_index.py` to aggregate metadata from historical ledgers (`.jules/docs/ledger.json` and `.jules/quality/ledger.json`) in addition to live runs. This consolidates all known run data into the single `RUNS_ROLLUP.md` file without rewriting historical truth.

## 🎯 Why
The previous version of `build_index.py` only pulled data from live runs in the `.jules/runs/` directory. Historical runs logged in `.jules/docs/` and `.jules/quality/` were invisible to the generated index. As per the Jules rules, historical packets/ledgers are primary truth and cannot be deleted or migrated. Thus, the index builder must natively support pulling from those older ledger formats to give reviewers and LLMs a complete rollup.

## 🔎 Evidence
- file paths: `.jules/bin/build_index.py`, `.jules/index/generated/RUNS_ROLLUP.md`
- observed behavior: Before the patch, the generated index contained 0 historical runs. After the patch, it successfully loads all 3 runs from the historical quality and docs ledgers.

## 🧭 Options considered

### Option A (recommended)
- what it is: Update `build_index.py` to natively aggregate data from both live `.jules/runs/` and historical ledgers.
- why it fits this repo and shard: It adheres to the Archivist mission to summarize per-run packets into rollups, while strictly following the rule that historical ledgers must not be altered or deleted.
- trade-offs:
  - Structure: Centralizes all run metadata into one index.
  - Velocity: Quick, one-time script update.
  - Governance: Safe; preserves historical facts perfectly.

### Option B
- what it is: Migrate the old ledgers into the new `.jules/runs/` directory format and delete the old ledgers.
- when to choose it instead: If the project decided to break the "never rewrite historical packets" rule for consistency.
- trade-offs: Violates a core anti-drift memory rule and modifies primary truth files.

## ✅ Decision
Chosen Option A to keep historical truth files strictly untouched while fixing the fragmentation in the generated index.

## 🧱 Changes made (SRP)
- `.jules/bin/build_index.py`: Added parsing loops for `.jules/docs/ledger.json` and `.jules/quality/ledger.json`.
- `.jules/index/generated/RUNS_ROLLUP.md`: Re-generated with the combined run counts and source column.

## 🧪 Verification receipts
```text
{"cmd": "ls -la .jules/", "status": "success", "phase": "investigation"}
{"cmd": "cat .jules/README.md", "status": "success", "phase": "investigation"}
{"cmd": "cat .jules/bin/build_index.py", "status": "success", "phase": "investigation"}
{"cmd": "cat .jules/docs/ledger.json", "status": "success", "phase": "investigation"}
{"cmd": "cat .jules/quality/ledger.json", "status": "success", "phase": "investigation"}
{"cmd": "python3 .jules/bin/build_index.py", "status": "success", "phase": "execution"}
{"cmd": "cargo xtask docs --check", "status": "success", "phase": "verification"}
{"cmd": "cargo xtask version-consistency", "status": "success", "phase": "verification"}
```

## 🧭 Telemetry
- Change shape: Script logic extension
- Blast radius: Only affects the generated Jules run index
- Risk class: Low
- Rollback: `git restore .jules/bin/build_index.py && rm .jules/index/generated/RUNS_ROLLUP.md`
- Gates run: `python3 .jules/bin/build_index.py`, `cargo xtask docs --check`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
