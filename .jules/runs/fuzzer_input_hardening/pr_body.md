## 💡 Summary
This is a learning PR. Attempted to execute fuzzer targets on input/parser surfaces (`fuzz_scan_args`, `fuzz_toml_config`), but encountered an unrecoverable ASAN linker error in the execution environment that prevents `cargo-fuzz` from building the targets.

## 🎯 Why
The fuzzer persona requires running `cargo fuzz` or extracting deterministic regressions from corpora. Without functional ASAN linking in the nightly toolchain within this execution container, the fuzz scripts cannot be built, blocking the intended verification loop. We are recording this friction item to unblock future fuzzer runs.

## 🔎 Evidence
- `fuzz/Cargo.toml`
- Attempting to run `cargo fuzz run fuzz_toml_config --features config -- -runs=1000` failed to compile.
- Resulting error: `rust-lld: error: undefined symbol: __sancov_gen_.279`

## 🧭 Options considered
### Option A (recommended)
- Attempt to execute `cargo fuzz` to extract regressions or harden inputs, but terminate with a learning PR if environment friction prevents compilation.
- Fits the repo because fuzzing requires working toolchains, and documenting the failure aligns with Jules protocol for avoiding "fake fixes."
- Trade-offs: Velocity is paused for this prompt, but governance and tooling awareness is improved.

### Option B
- Add a fake deterministic test or unrelated UX input improvement to land a patch.
- Trade-offs: Strongly discouraged by the persona constraints ("Do not claim a win you did not prove," "Do not force a fake fix").

## ✅ Decision
Option A. We explored the fuzz targets but could not compile them due to ASAN linker limitations. Rather than forcing an unrelated fix, we are cleanly exiting with a learning PR and a documented friction item.

## 🧱 Changes made (SRP)
- Documented ASAN linker friction item.

## 🧪 Verification receipts
```text
$ cargo fuzz run fuzz_scan_args --features scan_args -- -runs=10
error: linking with `cc` failed: exit status: 1
  |
  = note: rust-lld: error: undefined symbol: __sancov_gen_.279
```

## 🧭 Telemetry
- Change shape: Documentation
- Blast radius: None (Learning PR)
- Risk class: None
- Rollback: Revert the PR
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`
- `.jules/friction/open/cargo_fuzz_asan_linker_failure.md`

## 🔜 Follow-ups
- Resolve the ASAN linker errors (`__sancov_gen_*`) so `cargo-fuzz` can compile fuzz targets in the sandbox.
