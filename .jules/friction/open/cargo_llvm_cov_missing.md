# Friction Item: cargo-llvm-cov is missing in CI context during proof execution

**Persona**: Fuzzer / Specsmith

When the repository invokes `cargo xtask proof --profile affected` under the `CI=true` context (such as during the `scoped-coverage-executor` workflow), it attempts to run `cargo llvm-cov -p tokmd --all-features --lcov` to capture coverage artifacts.

However, in the CI environment (or at least the `scoped-coverage-executor` environment observed), `cargo-llvm-cov` is not installed natively, resulting in a fatal executor failure: `error: no such command: llvm-cov`.

This breaks the `affected` proof policy. The CI environment should either pre-install `cargo-llvm-cov` using `cargo binstall cargo-llvm-cov` or `rustup component add llvm-tools-preview`, or the executor must gracefully degrade when coverage tooling is unavailable.
