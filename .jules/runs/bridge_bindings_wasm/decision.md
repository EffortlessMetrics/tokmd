# Option A (recommended)
Add `analysis_schema_version` to the JSON payload returned by `run_json("version", "{}")` when the `analysis` feature is enabled. Use this in `worker.js` to initialize the engine capabilities, matching `tokmd-wasm`'s `analysisSchemaVersion`.

* Fits this repo: Yes, satisfies cross-surface drift between bindings and rust core, maintaining symmetry between `version()` / `schema_version()` and the equivalent FFI `run_json("version")`.
* Velocity: Quick, requires updating a few lines.
* Governance: Follows standard feature-gating conventions (`#[cfg(feature = "analysis")]`).

# Option B
Update `worker.js` to unconditionally call `analysisSchemaVersion` directly if the method exists.

* Trade-offs: Bypasses `tokmd-core` version payload alignment entirely, leaving drift where `tokmd-core` lacks the analysis schema version while `tokmd-wasm` provides it.

# Decision
Option A. This explicitly solves the Rust core <-> browser-runner drift by aligning `run_json("version")` to carry the analysis schema version properly.
