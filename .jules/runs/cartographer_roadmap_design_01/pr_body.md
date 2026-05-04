## 💡 Summary
Updated `docs/design.md` to reflect that in-memory and WASM execution paths are now actively supported and no longer future concepts.

## 🎯 Why
The architectural design document described `MemFs` and WASM execution as "future in-memory substrates" and "future in-memory and WASM execution", while `ROADMAP.md` correctly lists WASM productization and in-memory paths as shipped in v1.9.0 and v1.10.0. This mismatch caused factual drift between the design reference and shipped reality.

## 🔎 Evidence
- `docs/design.md`: Contained phrases "future in-memory substrates" and "future in-memory and WASM execution".
- `ROADMAP.md`: Shows `v1.9.0` (Browser/WASM Productization) and `v1.10.0` (WASM truth) as "✅ Complete".
- `cargo test -p tokmd-io-port` confirms the `MemFs` and `HostFs` boundary is active.

## 🧭 Options considered
### Option A (recommended)
- Update `docs/design.md` to remove "future" qualifiers from in-memory and WASM documentation.
- Fits the repo and shard by keeping the architectural truth aligned with shipped code without unnecessary rewrites.
- Trade-offs: Structure/Governance (correctness > drift), Velocity (unblocks clarity).

### Option B
- Keep the docs as is or only add a small comment indicating completion.
- When to choose: When changes are actively in flux and not yet stable.
- Trade-offs: Maintains factual drift which misleads contributors.

## ✅ Decision
Option A. Updated `docs/design.md` to remove the "future" qualifiers from the `MemFs` and `WASM` descriptions, as they are now fully shipped features in the `tokmd-io-port` design.

## 🧱 Changes made (SRP)
- `docs/design.md`

## 🧪 Verification receipts
```text
{"command": "sed -i 's/Host-abstracted file access for future in-memory and WASM execution:/Host-abstracted file access for in-memory and WASM execution:/g' docs/design.md", "outcome": "success"}
{"command": "sed -i 's/→ MemFs (tests \\/ future in-memory substrates)/→ MemFs (tests \\/ in-memory substrates)/g' docs/design.md", "outcome": "success"}
{"command": "git diff docs/design.md", "outcome": "success"}
{"command": "cargo test -p tokmd-io-port", "outcome": "success"}
{"command": "cargo xtask docs --check", "outcome": "success"}
{"command": "cargo xtask publish --plan --verbose", "outcome": "success"}
{"command": "cargo fmt -- --check", "outcome": "success"}
{"command": "cargo clippy -- -D warnings", "outcome": "success"}
{"command": "cargo test -p tokmd --no-default-features", "outcome": "success"}
{"command": "cargo test -p tokmd --no-default-features --lib", "outcome": "success"}
```

## 🧭 Telemetry
- Change shape: Docs update.
- Blast radius: Docs only.
- Risk class: None (no code changes).
- Rollback: `git checkout docs/design.md`.
- Gates run: `cargo xtask docs --check`, `cargo xtask publish --plan --verbose`, `cargo test -p tokmd-io-port`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test -p tokmd --no-default-features`.

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design_01/envelope.json`
- `.jules/runs/cartographer_roadmap_design_01/decision.md`
- `.jules/runs/cartographer_roadmap_design_01/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design_01/result.json`
- `.jules/runs/cartographer_roadmap_design_01/pr_body.md`

## 🔜 Follow-ups
None.
