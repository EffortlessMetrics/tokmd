## 💡 Summary
This is a learning PR documenting that release and governance surfaces are fully consistent, with no drift detected across the publish plan, version consistency, or CLI documentation. It also logs a minor friction item regarding `cargo run --bin xtask` behavior at the workspace root.

## 🎯 Why
The assignment was to improve release/governance hygiene in one coherent way by addressing potential drift in the publish-plan, version-consistency, release metadata, changelog, or RC-hardening docs. Investigation confirmed that all components are correctly aligned for version `1.9.0` and no manual patching was justified, meaning a learning PR is the correct outcome.

## 🔎 Evidence
- `cargo xtask version-consistency` ran cleanly, confirming that Cargo crate versions, workspace dependencies, and Node package manifests all correctly match `1.9.0`.
- `cargo xtask docs --check` completed without error, verifying `docs/reference-cli.md` is in sync with the CLI.
- `cargo xtask publish --plan` executed successfully and explicitly excluded `tokmd-fuzz` and `xtask` as expected.
- Running `cargo run --bin xtask` throws `error: no bin target named xtask in default-run packages` because Cargo routes it to the `tokmd` package, which acts as the workspace's default-run target.

## 🧭 Options considered
### Option A (recommended)
- Produce a learning PR.
- Record that versioning, release metadata, and documentation are structurally aligned with `1.9.0`.
- Log the `cargo run --bin xtask` UX confusion as a friction item.
- Fits the repo instructions well: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."

### Option B
- Modify the `xtask` Cargo.toml to add an explicit `[[bin]]` section to fix the UX.
- Trade-offs: This introduces unnecessary code changes and violates the anti-drift rules to avoid broad changes unless directly required, given the built-in `.cargo/config.toml` alias `cargo xtask` already resolves the issue properly.

## ✅ Decision
Option A. The governance surfaces are healthy. Producing a learning PR accurately reflects the state of the repo while documenting a minor friction point for future consideration without risking unnecessary changes.

## 🧱 Changes made (SRP)
- None (Learning PR)

## 🧪 Verification receipts
```text
$ cargo xtask version-consistency
Checking version consistency against workspace version 1.9.0
  ✓ Cargo crate versions match 1.9.0.
  ✓ Cargo workspace dependency versions match 1.9.0.
  ✓ Node package manifest versions match 1.9.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo xtask docs --check
Documentation is up to date.

$ cargo run --bin xtask -- version-consistency
error: no bin target named `xtask` in default-run packages
help: a target with a similar name exists: `tok`
help: available bin in `xtask` package:
    xtask
```

## 🧭 Telemetry
- Change shape: learning
- Blast radius: none
- Risk class + why: none (no codebase changes made)
- Rollback: N/A
- Gates run: `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo xtask publish --plan`

## 🗂️ .jules artifacts
- Written: `.jules/runs/run-steward-governance-001/envelope.json`
- Written: `.jules/runs/run-steward-governance-001/decision.md`
- Written: `.jules/runs/run-steward-governance-001/receipts.jsonl`
- Written: `.jules/runs/run-steward-governance-001/result.json`
- Written: `.jules/runs/run-steward-governance-001/pr_body.md`
- Added: `.jules/friction/open/cargo_bin_xtask_confusion.md`
- Added: `.jules/personas/steward/notes/steward_metadata.md`

## 🔜 Follow-ups
- See `.jules/friction/open/cargo_bin_xtask_confusion.md` for details on the workspace default-run binary masking.
