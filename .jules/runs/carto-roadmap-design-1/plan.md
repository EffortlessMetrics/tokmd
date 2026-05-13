1. Update `ROADMAP.md`
   - Modify the `v1.12.0` and `v3.0` sections in `ROADMAP.md` to reflect that `v1.12.0` is the active goal and `v3.0` shadow mode is active, resolving the factual drift.
   - Use the `replace_with_git_merge_diff` tool with exact blocks to update `ROADMAP.md`:
```
<<<<<<< SEARCH
### v3.0 — Tree-sitter Integration (Long-term)

_Goal: Accurate parsing for precise metrics. This is a significant undertaking requiring substantial R&D investment and is intentionally deferred well beyond the v2.x roadmap._
=======
### v3.0 — Tree-sitter Integration (Long-term)

_Goal: Accurate parsing for precise metrics. This is a significant undertaking requiring substantial R&D investment and is intentionally deferred well beyond the v2.x roadmap for full default integration, but foundation shadow work has begun._
>>>>>>> REPLACE
```
   - Use the `read_file` tool on `ROADMAP.md` to verify the changes.
2. Run validation tests
   - Run `cargo xtask docs --check`, `cargo fmt -- --check`, and `cargo clippy -- -D warnings` to verify no regressions.
3. Write artifacts
   - Use exact shell commands like `cat << 'EOF' > ...` to write `.jules/runs/carto-roadmap-design-1/pr_body.md` and `.jules/runs/carto-roadmap-design-1/result.json`.
   - Use the `read_file` tool to verify the created files.
4. Complete pre-commit steps
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. Submit
   - Call the `submit` tool to finalize the prompt-to-PR pipeline.
