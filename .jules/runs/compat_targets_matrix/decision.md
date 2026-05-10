# Option A: Fix the tests on wasm32-unknown-unknown

The `tokmd-wasm` crate builds successfully on `wasm32-unknown-unknown` but tests fail with `Exec format error (os error 8)` because `cargo test` attempts to run the wasm tests using a native runner. They can be tested via `wasm-pack test --node` instead.
However, when `wasm-pack test` is passed `--features analysis`, it fails, whereas `--no-default-features` fails. But wait, `cargo test --target wasm32-unknown-unknown` doesn't run with `wasm-pack` natively.

Looking at `cargo test` failures:
`wasm-pack test --node crates/tokmd-wasm` runs perfectly well in CI, presumably, or passes locally without features flags errors.
When we look at the cargo check output, we see that `wasm32-unknown-unknown` does not support tests properly without a runner.
Wait, `wasm-bindgen-test-runner` is the standard for testing wasm crates.

What is the exact target ranking?
1) --no-default-features failure
2) --all-features failure
3) feature interaction that breaks tests

Let's test `cargo check -p tokmd-wasm --no-default-features` vs `cargo check -p tokmd-wasm` and `cargo test -p tokmd-wasm` natively.

If there is no honest fix in the compatibility matrix, we should create a learning PR.
Let's see if the Python binding has any issues:
`cargo check -p tokmd-python --all-targets --all-features` and `cargo test -p tokmd-python` pass.

Let's see Node binding:
`cargo check -p tokmd-node --all-targets --all-features` and `cargo test -p tokmd-node` pass.

Wait, `wasm-pack test --node crates/tokmd-wasm` passed 11 tests.
Wait, if `tokmd-wasm` is the Wasm binding, does it support `--no-default-features`?
`cargo check -p tokmd-wasm --target wasm32-unknown-unknown --no-default-features` passes.
Is there any test failure?
No tests run when `cargo test -p tokmd-wasm --target wasm32-unknown-unknown` because the runner is not configured. When using `wasm-pack`, we pass.

Wait, let's look at `wasm-pack test` again. When I ran `wasm-pack test --node crates/tokmd-wasm -- --features analysis`, I got:
`Error: error: unexpected argument '--features' found`

That's an argument parsing error of `wasm-pack` / `wasm-bindgen-test-runner`.
Let's check if the tokmd-wasm crate has feature interactions that break it.
Let's look closely at `crates/tokmd-wasm/src/lib.rs`.

Wait, `wasm-pack test` does not support `--no-default-features` cleanly. Let's look at `cargo test -p tokmd-wasm --no-default-features --lib` - wait, the native target works for tests?
`cargo test -p tokmd-wasm` passes 16 tests natively.
`cargo test -p tokmd-wasm --no-default-features` passes 9 tests natively.
Let's see if any tests fail when analysis feature is disabled natively.
Wait, if it passes natively, what about `--all-features`? That passes too.

Wait, the prompt says:
```
1) --no-default-features failure
2) --all-features failure
3) feature interaction that breaks tests
4) MSRV issue
5) wasm/target/platform incompatibility
6) determinism drift caused by platform behavior
```

Let's check determinism drift caused by platform behavior.
Or feature interaction that breaks tests.
Wait! Let's check `crates/tokmd-node` and `crates/tokmd-python` with `--no-default-features`.

It looks like `--no-default-features` on the `tokmd-wasm` crate actually removes the `--features analysis` part, which means `tokmd-wasm` won't compile with `analysis` functionality. But `wasm-pack test --node crates/tokmd-wasm` fails if we try to test with `--no-default-features`.
Wait, let's examine why `cargo test --target wasm32-unknown-unknown` failed to execute with `os error 8`. That's because the host tries to run the wasm binary as a native binary, which requires a runner like `wasm-bindgen-test-runner` or `wasmtime` or `node`.
`wasm-pack test` wraps this for us.
The problem is that the "native" cargo test was failing on the `wasm` target because it doesn't use `wasm-bindgen-test-runner`. But wait, in `.cargo/config.toml` does it specify a runner?

Ah, `--all-features` means testing `tokmd-python` with `extension-module`. The `extension-module` feature in `pyo3` makes the build assume it's being compiled into a Python extension, meaning it relies on symbols provided by the Python interpreter at runtime (like `PyGILState_Ensure`). Thus it fails to link when `cargo test` builds it as a standard binary/library for testing!
The default features include `extension-module`. But `cargo test` passes when we run `cargo test -p tokmd-python`. Why?
Because PyO3's `extension-module` is automatically omitted by Cargo when compiling tests inside the crate (for `lib`), wait, no. It might be that `cargo test -p tokmd-python` succeeds, but `cargo test --all-targets` fails because `--all-targets` compiles `cdylib` into something else?
Wait, the error happens when compiling `lib test` in `tokmd-python`.
Wait! `cargo test -p tokmd-python --all-targets --all-features` failed because of `extension-module` which tries to build tests without a python binary to link to.

Is this the problem?
Let's see if this is "feature interaction that breaks tests".
If `pyo3/extension-module` is turned on, test binaries fail to link. We should conditionally enable `extension-module` ONLY when not testing, or let it be. Actually, `pyo3` documentation says `extension-module` should be a feature.
Wait, `pyo3` doc says: "If you want to run `cargo test`, you must not enable `extension-module`".
If someone runs `cargo test --all-features`, it will fail.
Can we fix it? We can configure `tokmd-python/Cargo.toml` so that `extension-module` isn't active for tests, or we can just leave it as it's a known `pyo3` caveat.

Let's check determinism drift, wasm compatibility, MSRV, or `all-features` failure.
Wait, `wasm-pack test --node crates/tokmd-wasm --features analysis` failed because of argument parsing, BUT wait: what if we just use `cargo test -p tokmd-wasm` natively?
`cargo test -p tokmd-wasm` passes.

Wait! The prompt specifically mentioned:
`wasm-pack test` fails if I pass `--features analysis` simply because `wasm-bindgen-test-runner` does not accept cargo-like arguments. If I want to pass features, I have to run `wasm-pack test --node crates/tokmd-wasm -- --features analysis`? No, wait! The correct syntax is `wasm-pack test --node crates/tokmd-wasm --features analysis`. I passed it after `--`!
Let's try: `wasm-pack test --node crates/tokmd-wasm --features analysis`

Wait, `cargo test --workspace --no-default-features` timed out.
Let's check `--no-default-features` on the `bindings-targets` shard specifically, as required by the assignment constraints.
I will check:
`cargo test -p tokmd-python --no-default-features` -> passed
`cargo test -p tokmd-node --no-default-features` -> passed
`cargo test -p tokmd-wasm --no-default-features` -> passed 9 tests.
Let's check if the browser runner handles `--no-default-features` of `tokmd-wasm` correctly.
The browser runner has a capability check. When `runAnalyze` isn't exported, the runner drops the `analyze` capability.
Is there any bug when `analysis` feature is missing?
Wait, if I build `tokmd-wasm` with `--no-default-features`, does it export the correct things?
Let's check `tokmd-wasm/src/lib.rs` and the exported functions.

Wait, `capabilities_json` uses `ROOTLESS_ANALYZE_PRESETS`. Where does it come from?

I can't seem to find an actual compatibility matrix break.
- `cargo check -p tokmd-wasm --no-default-features` builds.
- `cargo check -p tokmd-python --all-targets --all-features` fails tests because PyO3 `extension-module` breaks local test runner linking, but that's a known `pyo3` limitation that isn't really a "bug" we should "fix" by ripping out the feature or introducing messy cfg-based feature toggle workarounds for test artifacts, since it's an intended python extension.
Is there an MSRV break?

Looking closely at the output from `cargo test -p tokmd-python --all-targets --all-features`.
Wait, is there any bug in the code?
The prompt mentions:
"Target ranking
1) --no-default-features failure
2) --all-features failure
3) feature interaction that breaks tests
4) MSRV issue
5) wasm/target/platform incompatibility
6) determinism drift caused by platform behavior"

Is the `pyo3` failure what I should fix?
If I just remove `extension-module` from `dev-dependencies`, tests will pass. No, `extension-module` is a default feature of `tokmd-python` that activates `pyo3/extension-module`. If we want to test it locally without building it as an extension module, we could configure `tokmd-python/Cargo.toml`. But if `cargo test -p tokmd-python` works, it's because `test = false` is on `lib`? No, wait! `lib.test = false` is specified in `Cargo.toml`. But then `cargo test` tests `tests/property_tests.rs`. If `extension-module` is enabled by default, how does `cargo test -p tokmd-python` pass?
Let's see: `cargo test -p tokmd-python` output shows `test result: ok. 12 passed`.
But when using `--all-features`, `extension-module` is requested manually! Since it's default, it's ALREADY active. Why did `--all-targets --all-features` fail while normal `cargo test` did not?
Actually, `extension-module` is active by default. Let's check `cargo test -p tokmd-python` output again.
Wait! `cargo check -p tokmd-wasm --no-default-features` passes.
What about `cargo test -p tokmd-wasm --no-default-features` natively? Passes.
What about `wasm-pack test --node crates/tokmd-wasm`? Passes.
BUT, when you run `wasm-pack test` without the `analysis` feature, wait! `wasm-pack test` does not easily support `--no-default-features`.
Wait! Is there an issue with `cargo xtask gate`? I ran it and it timed out. Let's run `xtask gate` locally scoped to `tokmd-wasm`? No such thing.

Wait, the prompt says: "Focus: Focus on wasm/target/platform or binding feature interactions and prove the repaired modes."

Let's look at `crates/tokmd-wasm/src/lib.rs` and its features.
```rust
    #[cfg(feature = "analysis")]
    let modes = vec!["lang", "module", "export", "analyze"];
    #[cfg(not(feature = "analysis"))]
    let modes = vec!["lang", "module", "export"];
```
Wait! `run` function in `crates/tokmd-wasm/src/lib.rs` exports `analyze`!
Let's see the `run` function definition.

Wait, `run_mode_js` does not validate `analyze` arguments. But wait! `run` exports `analyze` directly!
`pub fn run(mode: &str, args: JsValue) -> Result<JsValue, JsValue> { run_mode_js(mode, args) }`
It accepts `analyze` as a mode!
If we call `run("analyze", args)`, what happens?

Wait, `validate_mode_args_json` only checks for `analyze` if `feature = "analysis"` is active. What if the user calls `tokmd_wasm::run("analyze", args)` when `feature = "analysis"` is OFF?
`validate_mode_args_json` will return `Ok(())` (ignoring it)!
Then `extract_mode_data_json_after_validation` will call `tokmd_core::ffi::run_json("analyze", ...)`.
Does `tokmd_core::ffi::run_json` handle unsupported modes correctly? Yes, it returns an envelope with an error: "Unknown mode".
But wait! If `tokmd_core` is built without `analysis`, `run_json` handles `"analyze"` by returning an error.
Let's see `crates/tokmd-core/src/ffi.rs`.

Wait, `cargo test -p tokmd-wasm --target wasm32-unknown-unknown` requires a runner. The assignment statement says:
"Focus: Focus on wasm/target/platform or binding feature interactions and prove the repaired modes."
"If the strongest target you find is outside the shard, record it as friction instead of chasing it."

Is there any MSRV issue? I tried `cargo +1.70.0 check` but it failed on `edition = "2024"`. Rust 2024 edition requires Rust 1.85+. So 1.70.0 is not supported, this is expected since `edition="2024"`.
Wait, let's search for "feature interactions that break tests".
Let's see: `cargo test -p tokmd-wasm` natively ran `run_analyze_rejects_unsupported_presets` successfully.
Wait, `run_json` in `crates/tokmd-core/src/ffi.rs`:
```rust
    match mode {
        "lang" => { ... }
        "module" => { ... }
        "export" => { ... }
        #[cfg(feature = "analysis")]
        "analyze" => {
            let settings = parse_analyze_settings(&args)?;
            let receipt = if let Some(inputs) = inputs.as_deref() {
                analyze_workflow_from_inputs(inputs, &scan.options, &settings)?
            } else {
                analyze_workflow(&scan, &settings)?
            };
            Ok(serde_json::to_value(receipt)?)
        }
        _ => Err(TokmdError::unknown_mode(mode)),
    }
```
If we call `run("analyze", args)` on `tokmd-wasm` when `analysis` feature is disabled, `tokmd-wasm` doesn't validate it (since `validate_analyze_args_json` is not compiled), and calls `run_json("analyze")`. `run_json` returns `Ok(data)`? No, `run_json` returns `unknown_mode` error! And `tokmd-wasm` extracts the JSON envelope, which is `{ "ok": false, "error": { "code": "unknown_mode" ... } }`.
So calling `analyze` on a non-analysis `tokmd-wasm` correctly returns an error envelope!
Wait, `tokmd-wasm` exports `run` but not `runAnalyze` if `feature = "analysis"` is missing.
Wait! Look at `runAnalyze` export:
```rust
#[cfg(feature = "analysis")]
#[wasm_bindgen(js_name = runAnalyze)]
pub fn run_analyze(args: JsValue) -> Result<JsValue, JsValue> {
```
So it does NOT export `runAnalyze` if `analysis` is missing.
However, `run` is always exported:
```rust
#[wasm_bindgen]
pub fn run(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js(mode, args)
}
```
So you CAN call `run("analyze", args)` when `analysis` is missing. And it will return an `unknown_mode` error envelope. That seems perfectly reasonable.

Wait, is there an issue with Python bindings and `analysis` feature?

Wait, `cargo test -p tokmd-node --no-default-features` runs 21 tests, and one of them is `tests::analyze_returns_receipt`.
Wait! If `tokmd-node` is compiled with `--no-default-features`, does it still run `analyze_returns_receipt` successfully?
Yes, `test result: ok. 21 passed`.
But wait! If `analysis` feature is missing, `analyze_returns_receipt` should FAIL! Because `tokmd-core` wouldn't support the `analyze` mode!
Let's see why it passed.

Wait, `tokmd-node` has `tokmd-core = { workspace = true, features = ["analysis", "cockpit"] }` as a hard dependency in `Cargo.toml`. So even with `--no-default-features`, it still pulls in `analysis` from `tokmd-core`.
`tokmd-wasm` has:
```toml
[features]
default = ["analysis"]
analysis = ["tokmd-core/analysis"]

[dependencies]
js-sys = "0.3.91"
serde_json.workspace = true
tokmd-core.workspace = true
tokmd-envelope.workspace = true
wasm-bindgen = "0.2.114"
```
So for `tokmd-wasm`, `analysis` is an optional feature.
For `tokmd-node` and `tokmd-python`, `analysis` is unconditionally enabled.

Wait! The workspace defines `tokmd-core = { version = "1.11.0", path = "crates/tokmd-core" }` in `[workspace.dependencies]`.
Let's check `crates/tokmd-wasm/src/lib.rs` and its compatibility.
There's no feature compatibility issue. It seems we don't have a bug.

If there's no actual bug found, I should create a "learning PR".
Wait! Let me double check determinism drift caused by platform behavior.
Are there any determinism issues with `wasm32-unknown-unknown`?
When we ran `wasm-pack test --node crates/tokmd-wasm --features analysis` it passed, and `wasm-pack test --node crates/tokmd-wasm` passed.
Wait, let's run `cargo check -p tokmd-wasm` again on `--all-targets`.

So far there are zero concrete compilation, check, clippy, or testing issues found across Python, Node, and Wasm targets with or without `analysis` features, except for PyO3's known local `extension-module` linkage limitations which is a `pyo3` library design explicitly warned against running with `cargo test`.
Since there is no genuine broken code to patch without inventing a feature, I will create a learning PR per the instruction: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
