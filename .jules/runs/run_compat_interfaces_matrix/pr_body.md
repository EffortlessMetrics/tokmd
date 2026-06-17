## 💡 Summary
This is a learning PR documenting a successful investigation of the compatibility matrix on the `interfaces` shard. Feature sets (`--no-default-features`, `--all-features`) on `tokmd` and `tokmd-core` were verified and pass correctly. A friction item was created regarding a misaligned path (`tokmd-config`) provided in the shard scope.

## 🎯 Why
The mission was to fix a compatibility issue across features, targets, platforms, or toolchains. Since thorough testing showed the matrix to be robust and fully functional without drift, no hallucinated fix was created. We also captured the friction that `crates/tokmd-config` does not exist as a separate crate but is integrated into `crates/tokmd/src/config/`.

## 🔎 Evidence
- Path investigated: `crates/tokmd-config/` (does not exist)
- Verified path: `crates/tokmd/src/config/`
- Verified tests: `crates/tokmd/tests/config_resolution.rs`
- Receipt: `cargo test -p tokmd --all-features` and `cargo test -p tokmd --no-default-features` passed cleanly without any failing tests.

## 🧭 Options considered
### Option A (recommended)
- Record a **learning PR** capturing the clean state of the matrices and the friction item regarding pathing rules for the `interfaces` shard.
- Why it fits: Avoids creating unnecessary changes or hallucinated bugs while capturing the valuable friction point that will help fix the system prompts.
- Trade-offs: Velocity increases since we avoid forcing an unnecessary code change, while Governance is improved through clear telemetry.

### Option B
- Attempt to tighten or restyle tests in `crates/tokmd/tests/config_resolution.rs`.
- When to choose: When tests are flaky across feature flags.
- Trade-offs: Fails the requirement of "Do not touch: performance or dependency cleanup not required by the compatibility story."

## ✅ Decision
Chose Option A to create a learning PR. The code is already passing the `compat-matrix` gates smoothly.

## 🧱 Changes made (SRP)
No code or tests were mutated. The focus was recording telemetry for system improvement.

## 🧪 Verification receipts
```text
$ bash -c "cargo test -p tokmd --no-default-features --test config_resolution"

running 13 tests
test test_resolve_export_cli_overrides_profile ... ok
test test_resolve_export_profile_overrides_default_format ... ok
test test_resolve_export_with_config ... ok
test test_resolve_lang_cli_overrides_profile ... ok
test test_resolve_lang_no_args_no_profile ... ok
test test_resolve_lang_partial_overrides ... ok
test test_resolve_lang_with_config_precedence ... ok
test test_resolve_lang_profile_overrides_default ... ok
test test_resolve_module_cli_overrides_profile_scalars ... ok
test test_resolve_module_no_args_no_profile ... ok
test test_resolve_module_profile_overrides_default ... ok
test test_resolve_module_with_config ... ok
test test_resolve_export_no_args_no_profile ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: 0 (No production code changed)
- Risk class: Low (Documentation and internal tooling records only)
- Rollback: Revert the PR
- Gates run: `cargo check --no-default-features`, `cargo check --all-features`, `cargo test --no-default-features`, `cargo test --all-features`

## 🗂️ .jules artifacts
- `.jules/runs/run_compat_interfaces_matrix/envelope.json`
- `.jules/runs/run_compat_interfaces_matrix/decision.md`
- `.jules/runs/run_compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/run_compat_interfaces_matrix/result.json`
- `.jules/runs/run_compat_interfaces_matrix/pr_body.md`
- `.jules/friction/open/FRIC-20250101-001.md`
- `.jules/personas/compat/notes/interfaces_matrix_healthy.md`

## 🔜 Follow-ups
- Update shard definitions in `.jules/policy/shards.json` to properly map `config` work to `crates/tokmd/src/config/**` instead of `crates/tokmd-config/**`.
