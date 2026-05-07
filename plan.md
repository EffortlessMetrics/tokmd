1. **Remove unwraps from `xtask/src/tasks/publish.rs`**:
   - In `xtask/src/tasks/publish.rs`, there's a `.unwrap()` around line 254: `let pkg = workspace_packages.iter().find(|p| p.name == *name).unwrap();`. I will change it to return an error properly using `context` or `ok_or_else`.
   - In `xtask/src/tasks/publish.rs` tests (around line 1180), change `.unwrap()` to `expect("...")`.

2. **Remove unwraps from `xtask/src/tasks/bump.rs`**:
   - Change `.unwrap()` in tests in `xtask/src/tasks/bump.rs` to `.expect("...")`.

3. **Verify tests and format**:
   - Run `cargo test -p xtask`
   - Run `cargo fmt`
   - Run `cargo clippy -p xtask`

4. **Complete Pre-commit Steps**: Ensure proper testing, verification, review, and reflection are done using `pre_commit_instructions`.

5. **Commit and Submit**: Update envelope and ledger, then submit PR.
