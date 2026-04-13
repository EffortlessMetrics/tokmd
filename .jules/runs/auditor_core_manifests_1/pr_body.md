## 💡 Summary
This is a learning PR. The initial attempt to blindly remove the `js` feature from the `uuid` dependency in `tokmd-format` broke the `wasm32-unknown-unknown` CI build. The correct fix (a target-scoped dependency) was already merged in #1112. Since no other obvious unused direct dependencies exist within the `core-pipeline` shard, this PR captures the friction and updates the Auditor persona notes.

## 🎯 Why
Removing the `js` feature natively provides WASM random number generator fallbacks, but introduces unused transitive dependencies when building natively, violating dependency hygiene goals. However, it is required for `wasm32-unknown-unknown` targets. Applying blanket removals of such features in the primary `[dependencies]` table without moving them to a target-specific configuration leads to cross-compilation CI breakages.

## 🔎 Evidence
- File: `crates/tokmd-format/Cargo.toml`
- Error: `error: to use uuid on wasm32-unknown-unknown, specify a source of randomness using one of the js, rng-getrandom, or rng-rand features`

## 🧭 Options considered
### Option A (recommended)
- What it is: Revert the code change and submit a learning PR documenting the friction and updating the Auditor persona notes.
- Why it fits this repo and shard: Complies with the instruction: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
- Trade-offs:
  - Structure: Adds documentation to `.jules/friction/open/`.
  - Velocity: Avoids chasing dead-end unused dependencies.
  - Governance: Documents the finding for future agents under `.jules/personas/auditor/notes/`.

### Option B
- What it is: Search for another unused dependency.
- When to choose it instead: If tools like `cargo machete` or `cargo tree` indicated a clear, obvious unused direct dependency in the specified crates.
- Trade-offs: `cargo machete` found no unused dependencies in the `core-pipeline` crates. The ones available (`tempfile`, `serde_json`) are actively used by the test suites.

## ✅ Decision
Implemented Option A. Reverted the breaking change and recorded the Wasm feature failure as a friction item to prevent future agents from making the same mistake.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/target_scoped_dependencies.md`
- Created `.jules/personas/auditor/notes/target_scoped_dependencies.md`

## 🧪 Verification receipts
```text
$ cargo check -p tokmd-format --target wasm32-unknown-unknown
error: to use uuid on wasm32-unknown-unknown, specify a source of randomness using one of the js, rng-getrandom, or rng-rand features
```

## 🧭 Telemetry
- Change shape: Learning PR, Friction Item.
- Blast radius: Documentation only.
- Risk class + why: Zero risk. Reverted code change.
- Rollback: Remove the `.jules` artifacts.
- Gates run: `cargo check -p tokmd-format --target wasm32-unknown-unknown`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_core_manifests_1/envelope.json`
- `.jules/runs/auditor_core_manifests_1/decision.md`
- `.jules/runs/auditor_core_manifests_1/receipts.jsonl`
- `.jules/runs/auditor_core_manifests_1/result.json`
- `.jules/runs/auditor_core_manifests_1/pr_body.md`
- `.jules/friction/open/target_scoped_dependencies.md`
- `.jules/personas/auditor/notes/target_scoped_dependencies.md`

## 🔜 Follow-ups
- Target-scoped dependency fixes have been merged in #1112.
