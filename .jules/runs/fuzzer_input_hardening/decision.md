# Input Hardening Decision

## What was inspected
- Inspected the fuzzer targets in `fuzz/fuzz_targets/` (`fuzz_toml_config`, `fuzz_policy_toml`, `fuzz_json_types`, `fuzz_scan_args`, etc).
- Inspected the seed corpora in `fuzz/corpus/`. Noticed that `fuzz_toml_config` and `fuzz_policy_toml` lacked seed corpora.
- Checked `fuzz/dict/` for dictionaries, which appear to be well-populated for JSON and TOML.
- Checked xtask dictionary definitions (fuzz_dictionaries_do_not_define_empty_tokens test passed).
- We have the constraint that the fuzzer `cargo fuzz` isn't installed/working properly in the sandbox (`error: no such command: fuzz`), so we cannot directly execute `cargo fuzz` or add new targets to a working fuzz test flow.

## Option A (recommended)
- Add seed corpus files to `fuzz_toml_config` and `fuzz_policy_toml` targets to lock in real edge cases and improve future fuzzing efficiency, addressing the "corpus improvements that lock in real edge cases" target from the Fuzzer persona.
- Why it fits: The persona mission explicitly values "corpus improvements that lock in real edge cases". The targets `fuzz_toml_config` and `fuzz_policy_toml` parse significant chunks of external input but were missing seed corpora, which makes initial fuzzing slower to find deep paths.
- Trade-offs:
    - Structure: Improves the fuzzing infrastructure.
    - Velocity: Future fuzzing runs will start from a higher coverage baseline.
    - Governance: Minimal risk since it only adds test/fuzz data.

## Option B
- Extract a deterministic regression from a fuzzable surface into `tokmd-settings` or `tokmd-gate` tests.
- When to choose it: If we found a specific bug through fuzzing that needs a deterministic regression test.
- Trade-offs: Without the ability to run the fuzzer to find a novel crash, we'd just be guessing at edge cases for regressions. The existing tests are already quite comprehensive for these crates.

## Decision
Option A. Adding the missing seed corpora for `fuzz_toml_config` and `fuzz_policy_toml` directly improves the fuzzability of these input surfaces, which is a high-ranking target for the Fuzzer persona. Since we can't run `cargo fuzz` directly, providing these seeds is a solid deterministic improvement to the fuzzing harness setup.
