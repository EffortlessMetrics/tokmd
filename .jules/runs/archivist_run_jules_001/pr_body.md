## 💡 Summary
Consolidated three duplicate `cargo fuzz` environment blocker issues into a single clean friction item and archived an outdated mutant schema friction item. This cleans up the friction backlog and improves clarity for future fuzzer runs.

## 🎯 Why
The friction log contained three separate files reporting the same basic issue: `cargo fuzz` fails out of the box due to missing `nightly` and ASAN configuration constraints. We also had a `cargo_mutants_schema_drift.md` file reporting an issue that had already been fixed in `.cargo/mutants.toml`. This creates unnecessary noise.

## 🔎 Evidence
- Found duplicate files: `.jules/friction/open/FRIC-20260413-001.md`, `.jules/friction/open/FRIC-20260428-001.md`, and `.jules/friction/open/cargo_fuzz_asan_linker_failure.md`.
- Verified `.cargo/mutants.toml` correctly uses `additional_cargo_args = ["--all-features"]`.

## 🧭 Options considered
### Option A (recommended)
- Consolidate the fuzz tooling friction items into one file, and move the stale mutants file to `done/`.
- This directly satisfies the Archivist persona's #1 target ranking: "consolidate recurring friction themes".
- Trade-offs: Structure (improves clarity) / Velocity (minor clean up effort) / Governance (keeps knowledge base tidy).

### Option B
- Summarize per-run packets.
- Redundant for this run as the backlog clean-up is higher value right now.

## ✅ Decision
Option A. I consolidated the duplicate fuzz issues and archived the fixed mutant issue, aligning perfectly with the Archivist's mandate.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/fuzz_toolchain_blocker.md` to unify the fuzz blockers.
- Deleted `.jules/friction/open/FRIC-20260413-001.md`.
- Deleted `.jules/friction/open/FRIC-20260428-001.md`.
- Deleted `.jules/friction/open/cargo_fuzz_asan_linker_failure.md`.
- Moved `.jules/friction/open/cargo_mutants_schema_drift.md` to `.jules/friction/done/`.

## 🧪 Verification receipts
```text
{"cmd": "cargo fmt -- --check", "status": "success", "summary": "Code formatting is clean"}
{"cmd": "cargo clippy -- -D warnings", "status": "success", "summary": "No clippy warnings"}
{"cmd": "cargo xtask docs --check", "status": "success", "summary": "Documentation is up to date."}
```

## 🧭 Telemetry
- Change shape: Metadata/Friction clean-up
- Blast radius: Jules documentation
- Risk class: Low - strictly `.jules` scaffolding changes
- Rollback: `git restore .jules/friction`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- Written: `.jules/runs/archivist_run_jules_001/envelope.json`
- Written: `.jules/runs/archivist_run_jules_001/decision.md`
- Written: `.jules/runs/archivist_run_jules_001/receipts.jsonl`
- Written: `.jules/runs/archivist_run_jules_001/result.json`
- Written: `.jules/runs/archivist_run_jules_001/pr_body.md`
- Added friction item: `FRIC-fuzz-toolchain-blocker`
- Resolved friction item: `cargo_mutants_schema_drift`

## 🔜 Follow-ups
None for this specific clean-up effort.
