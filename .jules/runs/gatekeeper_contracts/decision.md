# Decision

## Option A (recommended)
Update `docs/SCHEMA.md` and `docs/schema.json` to include the `trend` property under `CockpitReceipt`.
- **What it is**: The `TrendComparison` schema component was added to Rust types for Cockpit Receipts in `crates/tokmd-types/src/cockpit.rs` and the JSON format supports a nullable `trend` property. However, `docs/schema.json` defines `trend` but its definition string references `TrendComparison` which requires documentation in `docs/SCHEMA.md`. The documentation in `docs/SCHEMA.md` omits `trend` from the Cockpit Receipt examples and struct tables.
- **Why it fits this repo and shard**: Aligning the schema documentation and the schema files is the responsibility of the `tooling-governance` shard. Drift between Rust structs and `schema.json` or `SCHEMA.md` is a contract surface deviation.
- **Trade-offs**:
  - *Structure*: Enhances the coherence between the Rust implementation, schema definition, and markdown docs.
  - *Velocity*: Minor documentation effort with no changes to the runtime executable.
  - *Governance*: Secures our JSON schema contracts by keeping them transparent to API consumers.

## Option B
Update only the `TrendComparison` structs in Rust to match the `schema.json` directly if the definitions differ.
- **What it is**: Modify `tokmd-types` instead of `docs`.
- **When to choose it instead**: If the current implementation of trend metrics is experimental and shouldn't be publicly documented yet.
- **Trade-offs**: We would regress the feature surface that is already validated via `cargo test`.

## ✅ Decision
Option A. The `TrendComparison` properties are fully verified via properties and integration tests in `tokmd-cockpit`, and `trend` is already documented in `schema.json`. Updating `docs/SCHEMA.md` correctly aligns the documentation contract with the runtime outputs and JSON schemas.
