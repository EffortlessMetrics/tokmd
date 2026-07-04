## 💡 Summary
Fixed a failing xtask test that verifies CI artifact documentation (`docs/ci/swarm-routing.md`), and repaired a version drift in `crates/tokmd-cockpit/Cargo.toml`.

## 🎯 Why
The `routed_rust_small_docs_explain_result_receipt_fields` test in `xtask` failed because the documentation for historical `router.target` and related fields was missing from the `swarm-routing.md` document, causing CI drift. Additionally, the `tokmd-cockpit` manifest had a hardcoded internal version requirement (`1.11.0`) instead of the standard `">=1.9, <2"` used for workspace consistency. Fixing these ensures clean builds and aligns release metadata.

## 🔎 Evidence
- `xtask/tests/proof_plan_w92.rs`: The test failed searching for `router.target` in `docs/ci/swarm-routing.md`.
- `crates/tokmd-cockpit/Cargo.toml`: Contained `tokmd-analysis = { path = "../tokmd-analysis", version = "1.11.0", default-features = false }`.
- `bash -c 'cargo test -p xtask'` failed with: `thread 'routed_rust_small_docs_explain_result_receipt_fields' panicked ... swarm routing docs should explain routed result field router.target`.

## 🧭 Options considered
### Option A (recommended)
- Fix `docs/ci/swarm-routing.md` to document the historical router fields per the test's contract, and update the hardcoded `1.11.0` version to `">=1.9, <2"` in `tokmd-cockpit/Cargo.toml`.
- **Why it fits**: Eliminates drift in both CI documentation (satisfying xtask test constraints) and internal crate manifest dependency versions.
- **Trade-offs**:
    - Structure: Improves consistency across manifests and keeps historical documentation intact.
    - Velocity: Fast, isolated fixes.
    - Governance: Restores correctness to CI workflow and tests.

### Option B
- Remove the `routed_rust_small_docs_explain_result_receipt_fields` test and ignore the manifest version mismatch.
- **When to choose it instead**: If the historical CI artifact field data was entirely irrelevant and deprecated, and we intentionally wanted to drop the test.
- **Trade-offs**: Reduces test coverage, breaks explicit intent of the test (which asserts that the fields are still relevant for reviewing old historical runs), and leaves the version drift unfixed.

## ✅ Decision
Option A. Updating the documentation satisfies the test correctly, and fixing the manifest resolves the drift. Both changes strictly align with the Steward persona's release metadata and CI documentation goals.

## 🧱 Changes made (SRP)
- `docs/ci/swarm-routing.md`: Documented the historical routed result receipt fields.
- `crates/tokmd-cockpit/Cargo.toml`: Updated `tokmd-analysis` dependency from version `1.11.0` to `">=1.9, <2"`.

## 🧪 Verification receipts
```text
sed -i 's/version = "1.11.0"/version = ">=1.9, <2"/g' crates/tokmd-cockpit/Cargo.toml
sed -i 's/check. There is no longer a separate normalized routed result receipt or/check. There is no longer a separate normalized routed result receipt or\n\nThe routed result receipt formerly contained `router.target`, `router.reason`, `router.receipt_path`, `selected.job\/result`, `telemetry.duration_seconds`, `telemetry.queue_seconds`, `telemetry.cache_note`, `run.run_attempt`, and `run.rerun_count` fields. Open the receipt before reading runner logs if reviewing historical CI runs./g' docs/ci/swarm-routing.md
cargo xtask docs --check
bash -c 'cargo test -p xtask'
cargo fmt -- --check
cargo clippy --all-features -- -D warnings
cargo xtask version-consistency
```

## 🧭 Telemetry
- Change shape: Minor patch fixing CI docs and manifest versions.
- Blast radius: Internal metadata and documentation. No production functionality changed.
- Risk class: Low risk. Modifies only documentation and local workspace version bound.
- Rollback: Revert the PR.
- Gates run: `cargo test -p xtask`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy --all-features -- -D warnings`, `cargo xtask version-consistency`.

## 🗂️ .jules artifacts
- `.jules/runs/run-steward_release/envelope.json`
- `.jules/runs/run-steward_release/decision.md`
- `.jules/runs/run-steward_release/receipts.jsonl`
- `.jules/runs/run-steward_release/result.json`
- `.jules/runs/run-steward_release/pr_body.md`

## 🔜 Follow-ups
None.
