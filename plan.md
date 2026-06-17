1. **Explore `tokmd-wasm`, `tokmd-python`, and `tokmd-node` bindings to identify how they parse raw JSON arguments.**
   - Determine if they enforce the rule: "raw JSON arguments must always be strictly validated as top-level JSON objects to maintain parity with `tokmd-core`".
2. **Update `crates/tokmd-wasm/src/lib.rs`**
   - Modify `normalize_raw_json_args` to ensure that the parsed JSON is an object (`value.is_object()`). If not, return an error.
   - Add a test `normalize_raw_json_args_rejects_scalar_json_strings` to verify this behavior.
3. **Update `crates/tokmd-python/src/runtime.rs`**
   - Modify `run_json` to ensure the parsed JSON is an object. If not, return a `ValueError`.
   - Update tests in `crates/tokmd-python/src/tests.rs` to include a test for scalar/array JSON input to `run_json`.
4. **Run fall-back validation expectations for `compat-matrix` profile**
   - Run `cargo test/check --no-default-features` on affected crates
   - Run `cargo test/check --all-features` on affected crates
   - Run `wasm-pack test --node` for `crates/tokmd-wasm`
   - Run `cargo fmt -- --check` and `cargo clippy -- -D warnings`
   - Ensure all `.jules/runs/bridge_bindings_wasm` artifacts are written and checked.
5. **Pre Commit Instructions**
   - Run pre-commit instructions to ensure proper testing, verification, review, and reflection are done.
6. **Submit PR**
   - Once all checks pass, submit the unified PR fixing drift across FFI bindings.
