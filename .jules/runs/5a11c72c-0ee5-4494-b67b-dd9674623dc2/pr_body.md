# Context
The documentation in `docs/tutorial.md` outlines how to estimate effort using `tokmd analyze --preset estimate` with the flags `--effort-base-ref main --effort-head-ref HEAD` and also documents deterministic Monte Carlo outputs using `--monte-carlo --mc-seed`. However, these specific variants were not covered by the execution tests in `crates/tokmd/tests/docs.rs`, meaning the docs could drift silently without breaking the test suite.

# Change
* Added an assertion for `tokmd analyze --preset estimate --effort-base-ref HEAD --effort-head-ref HEAD --format md` in `crates/tokmd/tests/docs.rs`. (Used `HEAD` to avoid relying on `main` existing locally/in CI per guidelines).
* Added an assertion for `tokmd analyze --preset estimate --monte-carlo --mc-seed 42 --format json`.

# Proof
1. `cargo test -p tokmd --test docs` runs successfully.
2. The commands exactly mirror those found in `docs/tutorial.md`.
