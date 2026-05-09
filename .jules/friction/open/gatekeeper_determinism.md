---
id: gatekeeper_determinism
persona: Gatekeeper
style: Prover
shard: core-pipeline
status: open
---

# Gatekeeper determinism learning PR: No honest patch justified

Explored the core pipeline's proof surfaces (snapshots, determinism tests, schemas).

**Findings:**
1. `cli_snapshot_golden.rs` and the snapshots in `crates/tokmd/tests/snapshots` normalize out version numbers (some use `0.0.0`, some use `<VERSION>`), timestamps, file hashes, and root paths properly.
2. Property and BDD determinism checks pass across crates without flaky states.
3. Schemas correctly use `schema_version: 2` consistently.
4. Version consistency checked by `xtask version-consistency` confirms there are no crate version mismatches or semantic drifts across outputs.
5. All workspace tests `cargo test --workspace` run and pass.

Given the instructions "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix," I chose to create a learning PR.
