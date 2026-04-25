## 💡 Summary
Added `#![cfg(feature = "analysis")]` to integration test files that execute CLI commands requiring the `analysis` feature. This fixes widespread test failures when running the test suite without default features enabled.

## 🎯 Why
When the `analysis` feature is omitted (e.g. via `cargo test --no-default-features`), the `tokmd` CLI router returns an error `analysis feature is not enabled` for subcommands like `analyze`, `badge`, `baseline`, `gate`, and `run`. However, the integration tests themselves were not conditionally compiled, causing them to execute and fail when they received this error instead of the expected subcommand output. This breaks feature compatibility testing for downstream consumers or CI matrices that verify the project with minimal features.

## 🔎 Evidence
- **Finding:** Running `cargo test -p tokmd --no-default-features` panicked due to failing assertions in tests invoking `analyze` and `badge` commands.
- **Receipt:**
  ```text
  thread 'badge_lines_svg_stdout' (58069) panicked at .../library/core/src/ops/function.rs:250:5:
  Unexpected failure.
  code=1
  stderr=```"Error: analysis feature is not enabled\n"```
  command=`cd "/tmp/tokmd-fixtures-..." && "/app/target/debug/tokmd" "badge" "--metric" "lines"`
  code=1
  stdout=""
  stderr="Error: analysis feature is not enabled\n"
  ```

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Add `#![cfg(feature = "analysis")]` at the top of all integration test files that explicitly invoke analysis-gated subcommands.
- **Why it fits this repo and shard:** Isolates feature-specific integration tests using standard Cargo conditionally compiled macros. Ensures the CLI interfaces and core tests pass cleanly with or without the feature.
- **Trade-offs:**
  - *Structure:* Tests are cleanly excluded from the compilation graph when the feature is disabled.
  - *Velocity:* Quick to implement across many test files using standard `sed`/`bash` automation.
  - *Governance:* Aligns perfectly with idiomatic Rust feature gating practices.

### Option B
- **What it is:** Modify the test implementations to match on the `analysis` feature status at runtime, asserting for the error string `"analysis feature is not enabled"` when the feature is off.
- **When to choose it instead:** If we strictly wanted to test the error path router logic within every single integration test (which is redundant).
- **Trade-offs:** Highly verbose, pollutes the test assertions with conditional logic, and duplicates routing checks that are better tested centrally.

## ✅ Decision
I chose **Option A**. It's the most idiomatic way to handle tests that rely on optional crate features, preventing test compilation when the target feature is disabled and avoiding false-positive failures in CI feature matrix checks.

## 🧱 Changes made (SRP)
- Added `#![cfg(feature = "analysis")]` to the top of 55 integration test files in `crates/tokmd/tests/` that invoke `analyze`, `badge`, `baseline`, `gate`, or `run` subcommands.
  - Examples: `analyze_integration.rs`, `badge_integration.rs`, `baseline_integration.rs`, `cli_pipeline_e2e_w54.rs`, etc.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd --no-default-features
...
test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
...
test diff_via_receipt_json_resolves_to_sibling_lang_json ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
...
test cli_top_zero_shows_all ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## 🧭 Telemetry
- **Change shape:** Feature flag inclusions for test modules.
- **Blast radius:** Test compilation paths for `tokmd` (no production API or IO changes).
- **Risk class:** Low (test-only changes).
- **Rollback:** `git revert <commit>`.
- **Gates run:** `cargo check` and `cargo test` on affected crates, both with `--no-default-features` and `--all-features`.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_1/envelope.json`
- `.jules/runs/compat_interfaces_1/decision.md`
- `.jules/runs/compat_interfaces_1/receipts.jsonl`
- `.jules/runs/compat_interfaces_1/result.json`
- `.jules/runs/compat_interfaces_1/pr_body.md`

## 🔜 Follow-ups
None.
