# Option A (recommended)
Fix `tokmd-wasm`'s `wasm32-unknown-unknown` failing CI step `npm --prefix web/runner test`. There was actually a skipped test `"worker boots the real tokmd-wasm bundle when it has been built"` because `HAS_REAL_WASM_BUNDLE` was `false`.

Actually looking closely at the initial state and tests, what is the exact compatibility bug?
"Target ranking
1) --no-default-features failure
2) --all-features failure
...
Gate notes: Prefer reproducing the failing mode first and then the repaired mode."

Wait, `cargo build -p tokmd-wasm --target wasm32-unknown-unknown` failed earlier when `rustup target add wasm32-unknown-unknown` wasn't run. That's just the environment missing the target.
Is there a code failure?

Let's run `cargo check -p tokmd-wasm --no-default-features`
Let's run `cargo check -p tokmd-node --no-default-features`
Let's run `cargo check -p tokmd-python --no-default-features`

All passed.

Let's check if `cargo build` on the WASM bindings fails without default features. No, we did that.
What about building `tokmd-node`?
What about building `tokmd-python`?

Wait, in `crates/tokmd-wasm/src/lib.rs`, there is:
```rust
    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_analyze_estimate_reports_analysis_schema_and_matches_core_payload() {
```

Let's check if there is any `cfg(feature = "analysis")` logic inside `tokmd-wasm` that is incorrectly gated?
