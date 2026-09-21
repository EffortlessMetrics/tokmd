# Option A (recommended)
Add explicit regression tests in `crates/tokmd/tests/cli_parser_fuzz_regression.rs` to ensure numerical flags utilizing custom `value_parser` configurations (like `--max-commits` and `--max-commit-files`) gracefully reject invalid UTF-8 byte inputs with an `InvalidUtf8` error, rather than panicking.
* **Fit for repo and shard:** This perfectly fits the `interfaces` shard and the `Fuzzer` persona. The memory notes specifically highlight that numerical flags with custom `value_parser` must safely handle raw bytes reaching through `OsString` to avoid panics. Adding deterministic regression tests extracts value from fuzzy surfaces, matching the target ranking.
* **Structure / Velocity / Governance:** High structure alignment (fuzz regression locking), positive velocity (prevents panics explicitly), strong governance (matches memory constraints on CLI parser invariants).

# Option B
Introduce fuzzing harnesses using `cargo fuzz` directly on the CLI parser's argument strings.
* **When to choose instead:** If `cargo fuzz` tooling was consistently available and reliable across all sandbox environments, and we needed to explore the entire state space of `clap` argument parsing.
* **Trade-offs:** Running real fuzzers in constrained environments can be flaky or unavailable. The deterministic regression test achieves the specific hardening required without depending on external tools, making it much more robust for immediate proof.

# Decision
Proceeding with **Option A**. It's deterministic, directly addresses the memory rule about `value_parser` and `InvalidUtf8`, and provides a solid proof-improvement patch.
