1. **Understand and Explore**: Use grep to locate missing `is_object()` boundary checks in `tokmd-core`.
2. **Draft Decision**: Write `decision.md` covering Option A (harden all parser surfaces) and Option B (only harden scan). Option A will be selected.
3. **Write Regression Tests**: Introduce `regression_settings_fuzz.rs` exposing the invalid unwrap fallbacks.
4. **Harden Parsing Module**: Add `is_object()` validation to `tokmd-core/src/ffi/settings_parse.rs` and `parse.rs` to enforce strong JSON object boundary types.
5. **Run Gate Profile Checks**: Ensure CI tests (`CI=true cargo test -p tokmd-core --verbose`), `cargo build --verbose`, `cargo clippy`, and `cargo fmt` complete successfully.
6. **Compile PR artifacts**: Generate `pr_body.md` matching standard expectations and document the exact procedure in `receipts.jsonl`. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
7. **Submit Changes**: Issue final PR completion payload.
