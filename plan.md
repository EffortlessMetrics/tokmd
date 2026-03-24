1. **Explore the codebase and `.jules/` directory.**
   - Verified the existence of `.jules/policy/scheduled_tasks.json`, `.jules/runbooks/PR_GLASS_COCKPIT.md`, `.jules/security/ledger.json`, etc.
   - Identified the target of the run: burn down 31 `.unwrap()` calls in `crates/tokmd/src/context_pack.rs` tests.

2. **Refactor `.unwrap()` calls.**
   - Changed test method signatures in `test_parse_budget*` to return `anyhow::Result<()>`.
   - Replaced `.unwrap()` calls with `?` in parsing tests.
   - Replaced `x.unwrap()` with `x.expect("...")` on `find()` iterators and structured option properties.
   - Completed execution by applying Python scripting safely over AST boundary brackets.

3. **Verify the changes using strict quality gates.**
   - Ran `cargo test -p tokmd --lib context_pack`, `cargo clippy`, and `cargo fmt`.

4. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Call the `pre_commit_instructions` tool to run the pre-commit checklist.

5. **Commit the changes with a Glass Cockpit PR template.**
   - Create a commit using the `submit` tool with a descriptive PR body matching the `.jules` outline.
