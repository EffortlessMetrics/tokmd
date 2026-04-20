## 💡 Summary
Migrated the legacy `.jules/docs/` and `.jules/quality/` ledgers into the unified `.jules/runs/` packet format. This reduces duplication in parsing and makes the repository layout strictly follow `RUN_PACKET.md` as the single source of truth.

## 🎯 Why
Jules historically had separate tracking methods (append-only ledgers vs per-run packets) which created parsing overhead for index builders and violated the core tenet in `RUN_PACKET.md`: "Each run writes a self-contained packet under: `.jules/runs/<run-id>/`".

## 🔎 Evidence
- Found `docs/ledger.json` and `quality/ledger.json` tracking separate formats.
- Found `RUN_PACKET.md` explicitly deprecating shared append-only ledgers as primary truth.
- Command receipt: Generated `RUNS_ROLLUP.md` correctly processes the migrated packets.

## 🧭 Options considered
### Option A (recommended)
- Migrate legacy ledger entries into standard `.jules/runs/` directories and remove the old directories entirely.
- Fits the `workspace-wide` shard and the `Archivist` persona by consolidating shared scaffolding.
- Trade-offs: Structure (Simpler layout), Velocity (Easier tooling), Governance (Preserves history).

### Option B
- Modify `build_index.py` to recursively parse and adapt old ledger formats forever.
- When to choose: If strict historical layout preservation is more important than repository simplicity.
- Trade-offs: Adds brittle legacy-parsing logic to active tooling.

## ✅ Decision
Chose **Option A**. The value of a single source of truth for all historical Jules runs outweighs the minimal work to migrate the JSON envelopes over.

## 🧱 Changes made (SRP)
- Migrated `36cec87d-2836-42ed-9ae1-33dbf2702319` into `runs/`
- Migrated `09c9d819-02cd-4f63-b662-921c812f93dd` into `runs/`
- Migrated `d657338a-caa9-4ccf-93a1-4733ada7154c` into `runs/`
- `git rm -r .jules/docs/ .jules/quality/`
- Re-ran `.jules/bin/build_index.py` to regenerate the index

## 🧪 Verification receipts
```text
{"cmd": "git rm -r .jules/docs/ .jules/quality/", "status": "success"}
{"cmd": "python3 .jules/bin/build_index.py", "status": "success"}
{"cmd": "cargo xtask docs --check", "status": "success"}
{"cmd": "cargo fmt -- --check", "status": "success"}
{"cmd": "cargo clippy -- -D warnings", "status": "success"}
{"cmd": "cargo xtask publish --plan --verbose", "status": "success"}
{"cmd": "cargo xtask version-consistency", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Migration
- Blast radius: Jules tooling/metadata only
- Risk class: Low, only `.jules/` directory modified
- Rollback: `git revert`
- Gates run: Docs, Fmt, Clippy, xtask version-consistency, xtask publish

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/` packet written

## 🔜 Follow-ups
None
