## 💡 Summary
This is a learning PR. I investigated the `core-pipeline` shard (`tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`) for unused dependencies or redundant features to remove. I found that the core pipeline crates are currently in a good state regarding direct dependency hygiene.

## 🎯 Why
The assignment was to land a high-signal dependency hygiene improvement in the `core-pipeline` shard, specifically targeting unused dependencies or redundant declarations. If the strongest target found is outside the shard, the instructions mandate recording it as a friction item instead of chasing it.

## 🔎 Evidence
- `cargo machete --with-metadata` confirmed no unused dependencies in `crates/tokmd-types`, `crates/tokmd-scan`, `crates/tokmd-model`, and `crates/tokmd-format`.
- `cargo machete` identified unused dependencies in `tokmd-fuzz` and `tokmd-node`, which fall outside the assigned primary shard.

## 🧭 Options considered
### Option A
Remove unused `clap` feature and optional dependency from `tokmd-types`.
- what it is: Moving CLI-specific enum derivations out of `tokmd-types`.
- why it fits this repo and shard: The `clap` feature is only used by `tokmd` itself, but `tokmd-types` is a core pipeline crate. It separates CLI vs Core Types.
- trade-offs: Structure: Better separation of concerns. Velocity: Higher friction, requires moving code and potentially breaking contracts. Governance: Higher risk.

### Option B (recommended)
Create a learning PR documenting the hygiene gap in `tokmd-fuzz` and `tokmd-node` as friction.
- what it is: Recording the findings and deferring the out-of-shard cleanup.
- when to choose it instead: When no safe, unused direct dependency can be removed in the primary shard, and out-of-shard targets are explicitly disallowed by the prompt.
- trade-offs: Structure: No structural change. Velocity: Fast. Governance: Low risk.

## ✅ Decision
I chose Option B. I've thoroughly investigated the target crates for unused dependencies using `cargo machete` and `cargo tree`. `cargo machete` reported "didn't find any unused dependencies" for all four core crates. While unused dependencies were found in `tokmd-fuzz` and `tokmd-node`, the instructions explicitly state to "record it as friction instead of chasing it" if the strongest target is outside the shard.

## 🧱 Changes made (SRP)
- `.jules/friction/open/auditor_core_manifests_friction.md`

## 🧪 Verification receipts
```text
cargo machete --with-metadata crates/tokmd-types
cargo-machete didn't find any unused dependencies in crates/tokmd-types. Good job!

cargo machete --with-metadata crates/tokmd-model
cargo-machete didn't find any unused dependencies in crates/tokmd-model. Good job!

cargo machete --with-metadata crates/tokmd-format
cargo-machete didn't find any unused dependencies in crates/tokmd-format. Good job!

cargo machete --with-metadata crates/tokmd-scan
cargo-machete didn't find any unused dependencies in crates/tokmd-scan. Good job!
```

## 🧭 Telemetry
- Change shape: Learning PR / Documentation
- Blast radius: None (API / IO / docs / schema / concurrency / compatibility / dependencies)
- Risk class: Low
- Rollback: N/A
- Gates run: `cargo machete`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_core_manifests/envelope.json`
- `.jules/runs/auditor_core_manifests/decision.md`
- `.jules/runs/auditor_core_manifests/receipts.jsonl`
- `.jules/runs/auditor_core_manifests/result.json`
- `.jules/runs/auditor_core_manifests/pr_body.md`
- `.jules/friction/open/auditor_core_manifests_friction.md`

## 🔜 Follow-ups
- Clean up unused dependencies in `tokmd-fuzz` (`anyhow`, `blake3`, `tempfile`) and `tokmd-node` (`napi-build`) (captured in `.jules/friction/open/auditor_core_manifests_friction.md`).
