# Decision

## Option A
Implement custom Levenshtein distance calculation to improve input UX for typos. (REJECTED: Code review noted this as an unrelated UX fix and not part of the fuzzer input hardening assignment).

## Option B
Attempt to run `cargo fuzz` locally to find corpus edge cases or parser panics.
We installed `cargo-fuzz` and `rustup default nightly`, and attempted to build `fuzz_scan_args` and `fuzz_toml_config`. However, both target builds failed with identical ASAN linker errors (`undefined symbol: __sancov_gen_...`). Without a functional fuzzing environment or preexisting corpus data, we cannot reliably extract deterministic regressions or run the fuzz targets.

## Option C
Given the failure to operate `cargo fuzz` and the absence of a clear fuzzable input surface bug without forcing a fake feature, we will terminate the run with a learning PR documenting the friction item that `cargo fuzz` fails to link ASAN properly in this environment.

## Decision
Option C. This conforms to the instruction: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
