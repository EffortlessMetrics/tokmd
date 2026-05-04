## 🧭 Options considered

### Option A (recommended)
- Add `tokmd-config` back into the workspace members, fix the test suite compilation failures caused by recent struct renames, and exclude non-workspace crates like `vendor/home-0.5.12` and test data paths. Use `[package.metadata.cargo-machete]` to ignore dynamically loaded features to clean up linter warnings.
- Why it fits: This resolves immediate structural hygiene issues spotted by tools like `cargo machete`, repairs broken tests in an orphaned crate, and solidifies boundaries without dropping test coverage.
- Trade-offs: Requires updating `tokmd-config` test suites which drifted slightly out of date.

### Option B
- Completely remove `tokmd-config` if it's unused.
- Why it fits: Simplifies the workspace.
- Trade-offs: `tokmd-config` seems to contain a lot of deep/comprehensive test suites that test configuration logic. Throwing away the crate would lose valuable property-based and BDD scenario tests.

## ✅ Decision
Chose Option A. `tokmd-config` was left out of workspace members, meaning its extensive test suite was not running in CI or local `cargo test --workspace`. Adding it back and fixing the compilation errors preserves test coverage. We also excluded `crates/tokmd/tests/data` and `vendor/home-0.5.12` to fix `cargo metadata` errors.
