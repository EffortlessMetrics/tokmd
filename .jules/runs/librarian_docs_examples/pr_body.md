## 💡 Summary
I fixed a parameter drift in `docs/reference-cli.md` for the `tokmd handoff` command, making it consistent with `docs/handoff.md`, and added missing executable coverage for the handoff CLI variations in `crates/tokmd/tests/docs.rs` to ensure they execute successfully without regressions.

## 🎯 Why
The `docs/reference-cli.md` example for `tokmd handoff` showed a `--budget 64k` value, while `docs/handoff.md` and the actual defaults used `128k`. Additionally, `crates/tokmd/tests/docs.rs` did not actually verify the `tokmd handoff --no-git` nor the `tokmd handoff --budget 128k --strategy spread` variations, violating the `docs-executable` gate profile expectation that examples must execute or compile where possible to prevent silent drift.

## 🔎 Evidence
- `docs/reference-cli.md` parameter drift.
- `docs/handoff.md` examples.
- `cargo test -p tokmd --test docs` and `cargo xtask docs --check` outputs verifying the fixes.

## 🧭 Options considered
### Option A (recommended)
- what it is: Align `docs/reference-cli.md` parameter with `128k` and add `tokmd handoff --budget 128k --strategy spread` and `tokmd handoff --no-git` tests in `crates/tokmd/tests/docs.rs`.
- why it fits this repo and shard: Directly fulfills the `docs-executable` expectation for the `tooling-governance` shard.
- trade-offs: Structure / Velocity / Governance: Improves factual governance without introducing large structural shifts.

### Option B
- what it is: Fix drift without adding verification tests.
- when to choose it instead: If tests were impossible to add or write.
- trade-offs: Leaves documentation vulnerable to future, silent CLI changes.

## ✅ Decision
Option A was chosen to ensure docs remain factually accurate and drift is programmatically prevented.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Changed `64k` budget example to `128k` for `tokmd handoff`.
- `crates/tokmd/tests/docs.rs`: Added `recipe_handoff_budget_spread` and `recipe_handoff_no_git` tests.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd --test docs
running 31 tests
test recipe_check_ignore ... ok
test recipe_check_ignore_verbose ... ok
test recipe_cockpit_format ... ok
test recipe_ci_workflow_snippet ... ok
test recipe_context_budget ... ok
test recipe_context_bundle ... ok
test recipe_context_bundle_compress ... ok
test recipe_context_json ... ok
test recipe_context_list ... ok
test recipe_context_spread ... ok
test recipe_badge_generation ... ok
test recipe_default_map ... ok
test recipe_context_spread_compress ... ok
test recipe_export_full_json ... ok
test recipe_export_map_jsonl ... ok
test recipe_diff ... ok
test recipe_gate_default ... ok
test recipe_gate_json ... ok
test recipe_gate_fail_fast ... ok
test recipe_handoff_budget_spread ... ok
test recipe_generate_baseline ... ok
test recipe_gate_with_baseline ... ok
test recipe_handoff_bundle ... ok
test recipe_init_non_interactive ... ok
test recipe_handoff_no_git ... ok
test recipe_analyze_presets ... ok
test recipe_module_summary_markdown ... ok
test recipe_sensor_json ... ok
test recipe_simple_lang_summary ... ok
test recipe_tools_export_schemas ... ok
test recipe_run_and_diff ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s

$ cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Additions to CLI recipe tests and minor markdown text modification.
- Blast radius: Only affects `docs` and tests in `tokmd`. (API / IO / docs / schema / concurrency / compatibility / dependencies)
- Risk class: Low, only test code and docs text modified.
- Rollback: Revert the commit.
- Gates run: `cargo xtask docs --check`, `cargo test -p tokmd --test docs`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None
