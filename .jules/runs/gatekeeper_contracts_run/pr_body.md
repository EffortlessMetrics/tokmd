## 💡 Summary
Added `.jules/runs` to `TRACKED_AGENT_RUNTIME_PATHS` in `xtask/src/tasks/gate.rs` to fix `xtask_deep_w74::gate_runtime_guard_keeps_curated_jules_deps_history` test.

## 🎯 Why
The `cargo xtask gate` command checks for tracked agent runtime state and will fail if any files under `.jules/runs/` are staged in the git index. However, `TRACKED_AGENT_RUNTIME_PATHS` was missing `.jules/runs`, causing the test suite to fail. Adding the missing path protects the gate from tracked agent runtime state as originally designed.

## 🔎 Evidence
- `xtask/src/tasks/gate.rs`
- Running `cargo test -p xtask` previously failed on `gate_runtime_guard_keeps_curated_jules_deps_history`.

## 🧭 Options considered
### Option A (recommended)
- Add `.jules/runs` to `TRACKED_AGENT_RUNTIME_PATHS` in `xtask/src/tasks/gate.rs`.
- Why it fits this repo and shard: This satisfies the `gate_runtime_guard_keeps_curated_jules_deps_history` test and correctly protects the gate from tracked agent runtime state as the test expects.
- Trade-offs: Structure / Velocity / Governance: Aligns the implementation with the contract.

### Option B
- Remove the test.
- When to choose it instead: If the path should not be tracked by the runtime guard.
- Trade-offs: Decreases the quality and assurance of the code.

## ✅ Decision
Option A: adding `.jules/runs` to `TRACKED_AGENT_RUNTIME_PATHS` in `xtask/src/tasks/gate.rs` satisfies the missing guard rail and restores the test suite.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/gate.rs`: Added `.jules/runs` to `TRACKED_AGENT_RUNTIME_PATHS`.

## 🧪 Verification receipts
```text
running 36 tests
test all_task_modules_declared_in_mod_rs ... ok
test boundaries_check_forbidden_list_covers_higher_tiers ... ok
test boundaries_check_scans_sorted_manifests ... ok
test all_commands_dispatched_in_main ... ok
test boundaries_no_analysis_crate_depends_on_core ... ok
test bump_workspace_dep_update_targets_tokmd_prefix ... ok
test bump_schema_non_numeric_version_rejected ... ok
test bump_schema_invalid_format_rejected ... ok
test docs_task_markers_list_covers_all_subcommands ... ok
test docs_task_uses_cargo_run_for_help ... ok
test gate_check_flag_exists_in_cli ... ok
test gate_excludes_tokmd_python ... ok
test gate_excludes_xtask_from_compile_only_step ... ok
test gate_fmt_step_has_check_mode_variant ... ok
test gate_reports_pass_fail_count ... ok
test gate_runtime_guard_keeps_curated_jules_deps_history ... ok
```

## 🧭 Telemetry
- Change shape: Add element to array.
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): None.
- Risk class + why: Low, corrects test suite failure.
- Rollback: `git checkout xtask/src/tasks/gate.rs`
- Gates run: `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts_run/envelope.json`
- `.jules/runs/gatekeeper_contracts_run/decision.md`
- `.jules/runs/gatekeeper_contracts_run/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts_run/result.json`
- `.jules/runs/gatekeeper_contracts_run/pr_body.md`

## 🔜 Follow-ups
None.
