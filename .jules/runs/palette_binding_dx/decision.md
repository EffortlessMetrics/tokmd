## Problem
The `tokmd-wasm` binding uses `serde_json::to_string(args).map_err(|err| format!("JSON encode error: {err}"))` internally during parsing or validation, creating inconsistent, untyped JavaScript errors. Similarly, `tokmd-node` uses `Error::from_reason(format!("JSON error: {}", e))` and Node-native join panics. The `tokmd-python` binding has clear error handling using `TokmdError`.

## Goal
Improve runtime-facing DX by returning structured `TokmdError` via `ResponseEnvelope` across Node/WASM instead of ad-hoc string formatting, especially on JSON parse failures or internal mapping. The core's `ffi` crate currently uses `unwrap_or(&args)` and similar ad-hoc structures that may crash bindings.

However, after looking at the bindings (`crates/tokmd-wasm/src/lib.rs` and `crates/tokmd-node/src/lib.rs` and `crates/tokmd-python/src/lib.rs`), the main issue that stands out is panic handling in bindings. `unwrap()` and `expect()` are present in `crates/tokmd-node/src/lib.rs` and `crates/tokmd-wasm/src/lib.rs`. The `palette` persona focuses on improving the "runtime developer experience in one coherent way".
The biggest runtime developer experience friction in bindings is poor error messages, notably unhelpful errors when doing operations, panic on invalid JS data, or unhelpful validation error messages in the `tokmd-wasm` and `tokmd-node` bindings when arguments are invalid.

Specifically, looking at `crates/tokmd-wasm/src/lib.rs` around line 76:
```rust
fn validate_analyze_args_json(args_json: &str) -> Result<(), TokmdError> {
    let args: serde_json::Value =
        serde_json::from_str(args_json).map_err(TokmdError::invalid_json)?;
    let obj = args.get("analyze").unwrap_or(&args);

    match obj.get("preset").and_then(serde_json::Value::as_str) {
        Some(preset) if tokmd_core::supports_rootless_in_memory_analyze_preset(preset) => Ok(()),
        Some(preset) => Err(TokmdError::not_implemented(format!(
            "tokmd-wasm currently supports analyze only with preset=\"receipt\" or preset=\"estimate\" for in-memory inputs; got {preset:?}"
        ))),
        None => Ok(()),
    }
}
```

Wait, `validate_analyze_args_json` returns `TokmdError::not_implemented` with a long string.
What if `analyze` is executed but the preset is missing? We have a panic inside `crates/tokmd-wasm/src/lib.rs:547`:
```rust
        let timestamp = expected
            .get("generated_at_ms")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("{label} missing numeric generated_at_ms"));
```
This is in `assert_generated_at_ms_nonzero` which is a test utility.

Let's look at `crates/tokmd-node/src/lib.rs` again.
```rust
fn options_or_empty(options: Option<serde_json::Value>) -> serde_json::Value {
    options.unwrap_or_else(|| serde_json::json!({}))
}
```
Is there anything unergonomic for the developer?

Looking at `crates/tokmd-node/src/lib.rs`, `encode_args`:
```rust
fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))
}
```
This fails with a generic Error.

Let's see if there is any other low-context error message.
In `crates/tokmd-wasm/src/lib.rs` for `extract_mode_data_json_after_validation`:
```rust
fn extract_mode_data_json_after_validation(mode: &str, args_json: &str) -> Result<String, String> {
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_envelope::ffi::extract_data_json(&result_json).map_err(|err| err.to_string())
}
```
This maps `EnvelopeExtractError` to a string instead of maintaining a proper error format. `run_json` handles the core operations. The WASM binding expects `run_json` to return a JSON string, then it extracts data with `extract_data_json`. If it's an error envelope, `extract_data_json` returns an `EnvelopeExtractError` which is mapped to a generic `String` in `extract_mode_data_json_after_validation`.

Wait, the prompt asks to "Improve runtime developer experience in one coherent way." "Target ranking 1) unclear or low-context error messages 2) confusing diagnostics 3) CLI help/default/usage sharp edges 4) public API ergonomics in code-facing surfaces 5) output wording that causes real confusion".

Let's run `npm` or some tool to see JS error message format. No wait, `do not mention pnpm/npm/yarn unless the repo truth sources prove the relevant surface exists`.
I will create a JS test via `run_in_bash_session` to demonstrate the WASM/Node binding error DX.

Looking closely at `crates/tokmd-wasm/src/lib.rs` and `crates/tokmd-node/src/lib.rs`, there are multiple instances of internal `Result` string mapping instead of yielding the robust JSON envelope provided by `tokmd_core::ffi::run_json`.

Specifically, `tokmd_wasm::run_data_json`, `run_json`, and JS argument parsing currently convert errors to strings via `to_js_error(err.to_string())` instead of using the `ResponseEnvelope::error(&err).to_json()` wrapper that provides a structured JS object.

For `tokmd-wasm`:
- `validate_analyze_args_json` maps `TokmdError` to a JS Error without using `json_response`. However, wait, if we are failing early in WASM validate, we throw a JS Error. But the prompt specifically complains about DX friction. What if `run_json` does not panic but `run_mode_js` produces an `Error` rather than a standard output? No wait, Node.js uses NAPI `Result<String>`, Python uses `PyErr`, WASM uses `Result<JsValue, JsValue>`.

Wait! `tokmd_core::ffi::run_json` handles all parsing and error capturing:
```rust
pub fn run_json(mode: &str, args_json: &str) -> String {
    json_response(run_json_inner(mode, args_json))
}
```
This means `run_json` NEVER panics and ALWAYS returns a valid `{"ok": false, ...}` string.

Wait, `tokmd-wasm` defines `validate_analyze_args_json` to fail EARLY. But it throws a generic JS Error.
What if we use `ResponseEnvelope::error(&err).to_json()`?
In `crates/tokmd-wasm/src/lib.rs`:
```rust
fn validate_analyze_args_json(args_json: &str) -> Result<(), TokmdError> {
```
Wait, the core `tokmd_core::ffi::run_json` ALREADY handles validation via `run_mode` and parsing. Why does `tokmd-wasm` duplicate validation and throw `to_js_error(err.to_string())` instead of passing it to core? Because WASM does not support all features (only in-memory).
Ah, `validate_analyze_args_json` checks `supports_rootless_in_memory_analyze_preset`.

If we change `to_js_error` usages to return a well-formatted Error? No, in Node:
```rust
fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))
}
```
Wait, `Error::from_reason` is NAPI's way to create an exception.

Looking at `crates/tokmd-python/src/args.rs`:
```rust
pub(crate) fn build_args<'py>(...
```

Let's check `crates/tokmd-core/src/ffi/settings_parse.rs`.
```rust
    let obj = args.get("lang").unwrap_or(args);
```
Wait! `args.get("lang").unwrap_or(args)` is perfectly fine because `get()` returns `Option<&Value>`.

Let's rethink: "unclear or low-context error messages", "confusing diagnostics", "public API ergonomics in code-facing surfaces".
Is there an issue with Python/Node/Wasm/browser-runner API ergonomics?
Look at `web/runner/src/index.js` or `index.html`?
Let's `ls -la web/runner/` and `web/runner/test/`.

Wait, `encode_args` in `tokmd-node/src/lib.rs` returns `Error::from_reason(...)` with a generic NAPI `Error`. And `map_envelope_error` uses `Error::from_reason(err.to_string())`.
Is there a structured error we can return from NAPI? NAPI's `Error::new` allows us to define standard NodeJS error properties, like `code`. Wait, `Error::new` takes a `Status` and a message.

Let's look at `tokmd-python`'s `envelope.rs`:
```rust
use pyo3::prelude::*;
use tokmd_envelope::ffi::EnvelopeExtractError;
use crate::TokmdError;

pub(crate) fn map_envelope_error(err: EnvelopeExtractError) -> PyErr {
    match err {
        EnvelopeExtractError::Upstream(msg) => {
            // For upstream errors, we throw a TokmdError with the message.
            // Wait, does `TokmdError` support the envelope structure directly?
            ...
```
Let's read `crates/tokmd-python/src/envelope.rs`.

Wait, `EnvelopeExtractError` translates generic error structures into Python's custom exception `TokmdError` via string coercion.

But what if we look at `crates/tokmd-core/src/ffi/mod.rs` and `parse.rs`?
Is there a problem inside the `run_json` FFI itself?
"unclear or low-context error messages" ... "confusing diagnostics"
Let's test `run_json` with an invalid parameter via Node.js script.

Wait, `Error::from_reason` creates generic errors in NodeJS. If we return `serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))` it gives standard exception strings.

Let's rethink: Does `tokmd-wasm` throw exceptions with `to_js_error(err.to_string())` for *argument parsing* errors when using `run` instead of `runJson`?
Yes! In `tokmd-wasm/src/lib.rs`:
```rust
fn js_args_to_json(args: JsValue) -> Result<String, JsValue> { ... }
fn run_mode_js(mode: &str, args: JsValue) -> Result<JsValue, JsValue> { ... }
```
If we pass `run("lang", {paths: ["missing"]})`, the core `run_json` handles it and generates an envelope string.
`extract_mode_data_json_after_validation` does this:
```rust
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_envelope::ffi::extract_data_json(&result_json).map_err(|err| err.to_string())
```
Wait, if it's an error envelope (which core `run_json` guarantees on error!), `extract_data_json` returns an `EnvelopeExtractError::Upstream(msg)`, and then `.map_err(|err| err.to_string())` transforms it into a string, which `to_js_error` makes into a JavaScript `Error`.

So wait, the Node bindings also do:
```rust
fn map_envelope_error(err: tokmd_envelope::ffi::EnvelopeExtractError) -> Error {
    Error::from_reason(err.to_string())
}
```
If an upstream operation fails (e.g. invalid settings or scan failed), the user gets a Node.js `Error` with the `message` being `"[invalid_settings] Invalid value for 'paths': expected an array of strings"`.
Wait, is this standard for Node APIs? Usually Node APIs return a standard `Error` with a `code` property set on it!
Like `const err = new Error("..."); err.code = "invalid_settings";`

Wait, `EnvelopeExtractError` does not expose the `code` cleanly because `extract_data` formats it into a single string `Upstream(format_error_message(...))`.
Ah! The `EnvelopeExtractError::Upstream` only contains a formatted string!

Let's look at `EnvelopeExtractError::Upstream(String)` inside `crates/tokmd-envelope/src/ffi.rs`.
```rust
pub enum EnvelopeExtractError {
    JsonParse(String),
    JsonSerialize(String),
    InvalidResponseFormat,
    Upstream(String),
}
```
It completely throws away the structure of the upstream error (like `code` and `details`)! It only formats it as a string: `"[code] message"`.

If we want to "Improve runtime developer experience in one coherent way", preserving structured error codes from the core envelope back to the caller in `Node` and `WASM` (and possibly Python) would be extremely beneficial. If `runJson` is used, the developer receives the raw JSON and can parse the `code`. But if the developer uses the ergonomic async functions (`run` in Node, `run` in WASM), they catch an `Error` that only contains a squashed string. They can't programmatically catch `err.code === 'invalid_settings'` or `err.code === 'rate_limit'`!

Let's check `crates/tokmd-node/src/lib.rs` and `crates/tokmd-wasm/src/lib.rs` again.
If we add a `code` to the NAPI Error, NAPI supports `Error::new(Status::GenericFailure, "message")`. Wait, `napi::Error` doesn't have a `.code` constructor, wait... NAPI Errors *do* have `reason` and `status`. But `napi::Error::from_reason` creates an error with status `GenericFailure`.

Actually, in `tokmd-python`:
```rust
pub(crate) fn map_envelope_error(err: tokmd_envelope::ffi::EnvelopeExtractError) -> PyErr {
    TokmdError::new_err(err.to_string())
}
```
Wait, `TokmdError` in Python has `code`, `message`, `details`, etc.
But because `EnvelopeExtractError::Upstream` is a string, it creates `TokmdError` with only a string!
Wait, `TokmdError::new_err` in Python takes a tuple? No, wait.

Let's read `crates/tokmd-python/src/lib.rs` where `TokmdError` is defined.
In Python, `TokmdError` is just a standard exception class inheriting from `PyException`. It is initialized with a message string from `EnvelopeExtractError`.
It doesn't have structured `.code` or `.details` attributes.

So, the biggest flaw here for developer experience is that bindings lose the structured data from `tokmd_core`'s errors because `extract_data` formats everything into a single error string: `[code] message`.

If we change `EnvelopeExtractError` so that `Upstream` holds `tokmd_core::error::ErrorDetails`? No, `tokmd_envelope` should not depend on `tokmd_core`'s types. `tokmd_envelope` is meant to be a lightweight parser.
What if `EnvelopeExtractError::Upstream` held an `UpstreamError` struct?
```rust
pub struct UpstreamError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}
```
And then in `crates/tokmd-node/src/lib.rs` we map that to a NAPI error with a code, in `crates/tokmd-wasm/src/lib.rs` we map that to a JS Error with `.code = ...`, and in `crates/tokmd-python/src/envelope.rs` we somehow expose it? Or at least keep it cleanly formatted. Wait! JS Error objects in WASM can have properties attached. We can use `Reflect::set` to attach the `code`.
NAPI `Error::new` accepts a `Status`, but we can also use `napi::Error::from_reason(message)` and we can't easily attach a string `.code`. Wait! NAPI `Error` *does* have a `.code`! NAPI `Error` has `reason: String` and `status: Status`. No `code` property. However, Node's standard errors often have a `.code`. We can actually return an object or use `napi::Error::new(Status::GenericFailure, "...")`.

Actually, maybe the friction is just `unwrap` and `expect` panics.
Let's search for "unclear or low-context error messages" ... "confusing diagnostics" ... "CLI help/default/usage sharp edges" ... "public API ergonomics in code-facing surfaces" ... "output wording that causes real confusion".
Are there confusing error messages in the Core?

Let's read the `tokmd-wasm/src/lib.rs` error formatting:
```rust
    JSON::stringify(&args)
        .map_err(|_| to_js_error("failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| to_js_error("failed to serialize JS arguments"))
```
The error message is just `"failed to serialize JS arguments"`.

Wait! What if we look at `tokmd_core::error::TokmdError`?
"Target ranking 1) unclear or low-context error messages ... 4) public API ergonomics in code-facing surfaces"

Let's check `crates/tokmd-node/src/lib.rs`:
```rust
/// Run a tokmd operation with JSON arguments, returning a JSON string.
```

Wait, what if the user passes invalid options, like `paths: [123]`?
`runJson` returns: `{"ok": false, "error": {"code": "invalid_settings", "message": "Invalid value for 'paths[0]': expected a string", "details": "paths[0]"}}`.
If they use `run`, it throws an `Error` with message: `[invalid_settings] Invalid value for 'paths[0]': expected a string: paths[0]`.

Wait! A major DX friction in bindings: when `run` fails, it returns a Javascript Error that contains a string. Is that enough?
Is there a problem with the FFI boundary panicking when we provide bad parameters?
Let's see: `tokmd_node::run_json` calls `tokmd_core::ffi::run_json`. `run_json` handles everything. It does NOT panic.
However, in `tokmd_node::run`:
```rust
fn options_or_empty(options: Option<serde_json::Value>) -> serde_json::Value {
    options.unwrap_or_else(|| serde_json::json!({}))
}
```
Wait, in `tokmd_node`, what if the user provides `null` for options?
`run("lang", null)` - wait, `args` is `serde_json::Value`. If they pass `null`, it serializes to `"null"`. Then `run_json` parses it. Does `run_json` handle `"null"`?
`tokmd_core::ffi::run_json_inner`:
```rust
    let args: Value =
        serde_json::from_str(args_json).map_err(|err| TokmdError::invalid_json(err.to_string()))?;
    if !args.is_object() {
        return Err(TokmdError::invalid_json(
            "Top-level JSON value must be an object",
        ));
    }
```
So passing `null` throws `[invalid_json] Top-level JSON value must be an object`. This is an OK error message.

What else?
Let's check `crates/tokmd-node/src/lib.rs` for `#[napi]` exports.
It exports `run` and `run_json`.
```rust
pub async fn run(mode: String, args: serde_json::Value) -> Result<serde_json::Value> {
```
And Node native `Result` means it returns a Promise.

Let's check `crates/tokmd-python/src/lib.rs`.
```rust
pub fn lang(
    py: Python<'_>,
    paths: Option<Vec<String>>,
    top: usize,
    files: bool,
    children: Option<String>,
    redact: Option<String>,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> PyResult<Py<PyAny>> {
```
It accepts kwargs directly, not a dict. That's ergonomic.

What about `crates/tokmd-wasm/src/lib.rs`?
```rust
#[wasm_bindgen(js_name = run)]
pub fn run(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
```
Wait, what if `args` is undefined or null?
```rust
fn js_args_to_json(args: JsValue) -> Result<String, JsValue> {
    if args.is_null() || args.is_undefined() {
        return Ok("{}".to_string());
    }

    if let Some(raw_json) = args.as_string() {
        return normalize_raw_json_args(&raw_json).map_err(to_js_error);
    }
```
This handles null correctly.

Wait! The prompt is from the `bindings-targets` shard. "Target ranking: 1) unclear or low-context error messages...".
Let's search for "missing paths" or "path not found" in `tokmd-core` or something.
Actually, the agent's memory says:
> In the `tokmd` project, standard CLI error formatting for missing paths should use the exact string prefix `Path not found: <path>` inside `anyhow!` or `bail!` macros (which `error_hints::format` will render as `Error: Path not found: <path>`). Avoid custom variants like `does not exist` or `Directory does not exist`, as this specific prefix automatically triggers the CLI's global error hint machinery in `error_hints.rs`.

Ah!
Does `tokmd-core` or `tokmd-python` or `tokmd-node` have custom variants like `does not exist`?
Let's grep for "does not exist"!

Wait, `crates/tokmd-scan/src/tokeignore/mod.rs:57` uses `bail!("Directory does not exist: {}", dir.display());`.
According to memory: "In the `tokmd` project, standard CLI error formatting for missing paths should use the exact string prefix `Path not found: <path>` inside `anyhow!` or `bail!` macros... Avoid custom variants like `does not exist` or `Directory does not exist`".

But wait, the memory specifically applies to `tokmd` CLI error formatting. This is in `crates/tokmd-scan/src/tokeignore/mod.rs`. Is that the best target?
Let's see if there are other places.

Wait, `tokmd-scan`, `tokmd/src/commands/check_ignore.rs`, `tokmd/src/commands/diff.rs` and `tokmd/src/error_hints.rs` contain these "does not exist" strings.
Wait, the prompt assigned me to the `bindings-targets` shard, which contains `crates/tokmd-python/**`, `crates/tokmd-node/**`, `crates/tokmd-wasm/**`, `web/runner/**`.
And allowed me to touch `crates/tokmd-core/**`, `docs/**` if a coherent fix requires them.
`tokmd` and `tokmd-scan` are outside the shard.
The prompt explicitly says: "If the strongest target you find is outside the shard, record it as friction instead of chasing it."

Wait, the prompt asks to "Focus: Improve or lock runtime-facing ergonomics across bindings/targets when the repo proves those surfaces exist."

If we look at `tokmd-core` and the FFI layer:
Does `tokmd_core::error::TokmdError::path_not_found` use `Path not found: {}`? Let's check `crates/tokmd-core/src/error.rs`.

Wait, `tokmd-core` correctly uses `Path not found: {}`.
But what about `web/runner/`?
Let's look at `web/runner/runtime.js` or `messages.js`.

Wait, `web/runner/runtime.js` does:
```javascript
function extractRunnerError(error) {
    let message = "unknown runner error";
    let code = "run_failed";

    if (error instanceof Error && typeof error.message === "string") {
        message = error.message;
        if (typeof error.code === "string") {
            code = error.code;
        }
    ...
    const match = message.match(/^\[([^\]]+)\]\s*(.*)$/);
    if (match) {
        code = match[1];
        message = match[2];
    }
    return { code, message };
}
```
Ah! `web/runner/runtime.js` has a regex to parse `[code] message` out of the string! It does this exactly because `tokmd-wasm` currently squashes the error envelope into a single string.
If `tokmd-wasm` returned an `Error` with an actual `.code` property, it would just work because `extractRunnerError` checks for `error.code`. Or if `runJson` was used instead of `run`, the Javascript could just read the JSON envelope.

Wait, `web/runner/worker.js` uses `run` from `tokmd_wasm`:

Wait, `web/runner/worker.js` doesn't seem to have `run` hardcoded.
It looks up exports via `createModeHandler` which wraps `wasmModule[exportName]`. The `MODE_EXPORTS` are `"runLang"`, `"runModule"`, `"runExport"`, `"runAnalyze"`.
Let's check `tokmd-wasm` exports.
`runLang` is probably exported by a macro? Let's check `crates/tokmd-wasm/src/lib.rs`.

Wait, `web/runner/worker.js` wraps `wasmModule.runLang`.
When `runLang` is called with invalid parameters, it eventually throws an Error. Because in `run_mode_js`, we do:
```rust
fn run_mode_js(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args)?;
    let data_json = extract_mode_data_json(mode, &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
}
```
And `extract_mode_data_json`:
```rust
fn extract_mode_data_json(mode: &str, args_json: &str) -> Result<String, String> {
    validate_mode_args_json(mode, args_json).map_err(|err| err.to_string())?;
    extract_mode_data_json_after_validation(mode, args_json)
}
```
And `extract_mode_data_json_after_validation`:
```rust
fn extract_mode_data_json_after_validation(mode: &str, args_json: &str) -> Result<String, String> {
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_envelope::ffi::extract_data_json(&result_json).map_err(|err| err.to_string())
}
```
`err.to_string()` for an `EnvelopeExtractError::Upstream(msg)` just returns the string `msg`. Which is `"[code] message"`.
And then `to_js_error` makes it `Error("[code] message")`.
And then `web/runner/runtime.js` has a special regex to parse `[code] message` OUT OF the error string, just to be able to tell what kind of error it is.

If this is all done to extract the code and message in Node/WASM... wait. `crates/tokmd-node/src/lib.rs` has EXACTLY the same pattern:
```rust
fn map_envelope_error(err: tokmd_envelope::ffi::EnvelopeExtractError) -> Error {
    Error::from_reason(err.to_string())
}
```

Wait, what if we provide structured JavaScript Errors directly?
In `tokmd-wasm`, `js_sys::Error` doesn't have a constructor for `code`. But we can set a property!
```rust
fn to_js_error(message: impl Into<String>) -> JsValue {
    JsError::new(&message.into()).into()
}
```
If we can parse the `code` in Rust, we could set it on the JS object. But `EnvelopeExtractError` lost the code when it was formatted by `extract_data`!

Wait, why does `tokmd_envelope::ffi::extract_data` format it?
```rust
pub fn extract_data(envelope: Value) -> Result<Value, EnvelopeExtractError> {
    // ...
    Err(EnvelopeExtractError::Upstream(format_error_message(
        obj.get("error"),
    )))
}
```
Because `EnvelopeExtractError` is just an enum containing a string for `Upstream`.
If we change `EnvelopeExtractError` so that it preserves the original error structure, then bindings can expose a richer error object or keep `format_error_message` inside `to_string()` for backward compatibility.

Wait, is there an easier win?
Look at `tokmd_core::ffi::run_json`. If `args` is invalid JSON, it returns an error with `code` "invalid_json".
But `validate_analyze_args_json` in `tokmd-wasm` uses `TokmdError::not_implemented`. Then it returns `Err(TokmdError)`. Then `extract_mode_data_json` stringifies it with `err.to_string()`. So the error is `"[not_implemented] tokmd-wasm currently supports analyze only with preset=\"receipt\" ... "`.
Then `to_js_error` turns it into `Error("[not_implemented] tokmd-wasm ...")`.
And the regex in `runtime.js` successfully parses it.

What if we just improve the error string itself?
The prompt says: "Target ranking 1) unclear or low-context error messages 2) confusing diagnostics 3) CLI help/default/usage sharp edges 4) public API ergonomics in code-facing surfaces 5) output wording that causes real confusion".

Let's read `crates/tokmd-core/src/ffi/parse.rs` for bad error messages.
```rust
        _ => Err(TokmdError::invalid_field(
            "preset",
            "'receipt', 'estimate', 'health', 'risk', 'supply', 'architecture', 'topics', 'security', 'identity', 'git', 'deep', or 'fun'",
        )),
```
Are these confusing? They seem clear.

Let's check `crates/tokmd-node/src/lib.rs` and `crates/tokmd-python/src/args.rs`.
If I look at `.unwrap_or_else(|| panic!("{label} missing numeric generated_at_ms"));` in `tokmd-wasm`, that's in `#[cfg(test)]`.

Wait, look at `crates/tokmd-core/src/ffi/settings_parse.rs`:
```rust
pub(super) fn parse_analyze_settings(args: &Value) -> Result<AnalyzeSettings, TokmdError> {
    let obj = args.get("analyze").unwrap_or(args);

    let effort_base_ref = parse_optional_string(obj, "effort_base_ref")?;
    let effort_head_ref = parse_optional_string(obj, "effort_head_ref")?;
    if (effort_base_ref.is_some() && effort_head_ref.is_none())
        || (effort_base_ref.is_none() && effort_head_ref.is_some())
    {
        return Err(TokmdError::invalid_field(
            "effort_base_ref/effort_head_ref",
            "both effort_base_ref and effort_head_ref must be provided together",
        ));
    }
```
Wait, the core FFI settings parsing:
```rust
    let obj = args.get("lang").unwrap_or(args);
```
If I pass `args = {"paths": ["."]}`.
`args.get("lang")` is `None`. So `obj` becomes `args`. Then `parse_usize(obj, "top", 0)` looks for `top` in `args`.
So we can pass `{"paths": ["."], "top": 10}`.
What if we pass `{"lang": {"top": 10}, "paths": ["."]}`?
Then `obj` is `{"top": 10}`. So `parse_usize(obj, "top", 0)` finds `10`.
This allows a flat OR nested structure!
Is there any friction with this?

Let's rethink: "unclear or low-context error messages" ... "confusing diagnostics".
Let's see the error strings mapped in `tokmd-node` and `tokmd-wasm` when JSON encode/decode fails.

`tokmd-wasm`:
```rust
fn serialize_args(args: &Value) -> Result<String, String> {
    serde_json::to_string(args).map_err(|err| format!("JSON encode error: {err}"))
}
```
This is only in `#[cfg(test)]`.

In `tokmd-node`:
```rust
fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))
}
```
If serialization fails in Node, it says `JSON error: ...`.

Wait, what about `crates/tokmd-wasm/src/lib.rs` `js_args_to_json`?
```rust
fn js_args_to_json(args: JsValue) -> Result<String, JsValue> {
    // ...
    JSON::stringify(&args)
        .map_err(|_| to_js_error("failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| to_js_error("failed to serialize JS arguments"))
}
```
If `args` is an object with a circular reference, `JSON::stringify` throws. The rust code `.map_err(|_| to_js_error("failed to serialize JS arguments"))` loses the original JS error thrown by `JSON.stringify`!
If the JS developer passes an invalid JS object with circular references or BigInt, they just get "failed to serialize JS arguments" with no context of what property caused it!
Wait, `JSON.stringify` in JavaScript throws an error like `TypeError: Converting circular structure to JSON`. The Rust code throws it away and uses `failed to serialize JS arguments`.
Is this the "low-context error message" the prompt refers to?
Yes, in `tokmd-wasm`, when `JSON::stringify` fails, we swallow the error:
```rust
    JSON::stringify(&args)
        .map_err(|_| to_js_error("failed to serialize JS arguments"))?
```
Wait, `JSON::stringify` returns `Result<JsString, JsValue>` in `wasm_bindgen`. The `Err(JsValue)` contains the actual JS exception! We shouldn't throw it away.
If we change it to:
```rust
    JSON::stringify(&args)
        .map_err(|err| err)?
```
Or at least extract the message.

What about `JSON::parse`?
```rust
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
```
Also swallows the parsing error. Though `data_json` is produced by our own `tokmd_core`, if it fails to parse, something is deeply wrong and the error from JS `SyntaxError: Unexpected token ...` might be useful.

Let's look at `crates/tokmd-wasm/src/lib.rs` around line 147 (capabilities):
```rust
    JSON::parse(&capabilities_json())
        .map_err(|_| to_js_error("failed to parse tokmd wasm capabilities JSON"))
```
Also swallowed.

What about `tokmd-node/src/lib.rs`?
```rust
fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))
}
```
This is fine because it preserves `e`.
Wait! Is there something else? Let's check `crates/tokmd-node/src/lib.rs`.
```rust
    tokmd_envelope::ffi::parse_envelope(result_json).map_err(map_envelope_error)
```
Preserves the error via `map_envelope_error`.
```rust
async fn run_blocking<F>(f: F) -> Result<String> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))
}
```
Preserves `e`.

Wait! The prompt targets "bindings-targets". That includes Python!
Let's check `crates/tokmd-python/src/lib.rs` and `crates/tokmd-python/src/args.rs` for `map_err`.
Is `extract_mode_data_json_after_validation` the only thing that loses context?
`extract_mode_data_json_after_validation` in `tokmd-wasm` calls `err.to_string()`, which maps `EnvelopeExtractError` to a string. But `EnvelopeExtractError::to_string()` for `Upstream` just prints the formatted message `"[code] message"`. That's not losing information (the code is still there), it's just squashing it.

But what if the JS bindings developer wants to read the `code` properly instead of parsing the string?
In `tokmd-wasm`, when an error occurs, we return a `JsValue` that is an `Error`.
If we can augment the JS Error with `.code` and `.details`, it would be much better API ergonomics!

Let's look at how to attach a `code` property to a `js_sys::Error` in Rust.
```rust
fn to_js_error_with_code(message: &str, code: &str) -> JsValue {
    let err = js_sys::Error::new(message);
    let _ = js_sys::Reflect::set(&err, &JsValue::from_str("code"), &JsValue::from_str(code));
    err.into()
}
```
If we do this, JavaScript can just do `if (err.code === 'invalid_settings') ...`.
Wait, in `tokmd-wasm`, `extract_mode_data_json` returns `Result<String, String>`. So the error is already squashed to a `String` before `to_js_error` is called.
```rust
fn extract_mode_data_json(mode: &str, args_json: &str) -> Result<String, String> {
    validate_mode_args_json(mode, args_json).map_err(|err| err.to_string())?;
    extract_mode_data_json_after_validation(mode, args_json)
}

fn extract_mode_data_json_after_validation(mode: &str, args_json: &str) -> Result<String, String> {
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_envelope::ffi::extract_data_json(&result_json).map_err(|err| err.to_string())
}
```
If we change the return type to `Result<String, JsValue>`, we can map `TokmdError` directly to a rich `JsError`!
And we can map `EnvelopeExtractError` to a rich `JsError`!
Let's see if we can do this.
Yes! If we use a rich error factory for WASM:
```rust
fn to_js_error(message: impl Into<String>) -> JsValue {
    JsError::new(&message.into()).into()
}
```
We can define:
```rust
fn to_rich_js_error(code: &str, message: &str) -> JsValue {
    let err = JsError::new(message);
    if let Ok(code_val) = JsValue::from_str(code) {
        let _ = js_sys::Reflect::set(&err, &JsValue::from_str("code"), &code_val);
    }
    err.into()
}
```

Wait, what about Node? Node throws `Error::from_reason(...)` which creates a NAPI `Error`.
`napi::Error` DOES have a `reason` (string) but also a `status` (enum). The `Error::new` function takes a `Status` and a `String`. But it doesn't accept a `.code` property that is a string, wait. Node.js NAPI `napi::Error` is standard. Let's see how NAPI bindings usually add a `.code` property to an error. Wait, `Error::new` does NOT have a custom string code, but we can construct an object that subclasses Error or just throw a JS Object... No, if we use `napi::Error::new`, we can't easily add a string code.
BUT wait, in `web/runner/runtime.js` they just match the string: `const match = message.match(/^\[([^\]]+)\]\s*(.*)$/);`. That works for Node and Wasm seamlessly.
If the Wasm and Node bindings currently format the error correctly, then the JS client can parse it. Is this the DX problem the prompt meant?

Wait, in `tokmd-wasm`, when `extract_data_json` fails with `EnvelopeExtractError::Upstream`, `err.to_string()` produces `"[code] message"`. But what if `validate_analyze_args_json` fails? It returns `Err(TokmdError::not_implemented(...))`.
`TokmdError::to_string()` returns `"[code] message"` too!
Let's verify what `TokmdError::to_string()` produces.
`crates/tokmd-core/src/error.rs`:
```rust
impl fmt::Display for TokmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(details) = &self.details {
            write!(f, "[{}] {}: {}", self.code, self.message, details)
        } else {
            write!(f, "[{}] {}", self.code, self.message)
        }
    }
}
```
So `TokmdError::to_string()` natively formats exactly the same way as `EnvelopeExtractError::Upstream(...)`.

Wait, in `tokmd-wasm` `serialize_args` fails with `"JSON encode error: {err}"`.
In `js_args_to_json`:
`to_js_error("failed to serialize JS arguments")` doesn't have a code! It's just a raw message.
`to_js_error("failed to parse tokmd result JSON")` also doesn't have a code.
`normalize_raw_json_args` formats `"failed to parse JSON string arguments: {err}"`.

So some errors lack the `[code]` prefix, breaking the regex!
And wait... `tokmd-node` does:
`serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))` - no `[code]`!
`tokio::task::spawn_blocking(f).await.map_err(|e| Error::from_reason(format!("Task join error: {}", e)))` - no `[code]`!

If the web runner relies on `[code] message`, then these other errors fail to parse correctly, falling back to `code = "run_failed"` and `message = "unknown runner error"` or whatever.

Is that the DX improvement?
If we update all `to_js_error` and `Error::from_reason` to explicitly include a `[code] ` prefix, the error parsing will be 100% consistent across all bindings and internal failures!

For `tokmd-wasm`:
- `JSON encode error: {err}` -> `[json_encode_error] {err}`
- `failed to serialize JS arguments` -> `[invalid_settings] failed to serialize JS arguments`
- `failed to parse tokmd result JSON` -> `[internal_error] failed to parse tokmd result JSON`
- `failed to parse JSON string arguments: {err}` -> `[invalid_json] failed to parse JSON string arguments: {err}`
- `failed to parse tokmd wasm capabilities JSON` -> `[internal_error] failed to parse tokmd wasm capabilities JSON`

For `tokmd-node`:
- `JSON error: {}` -> `[invalid_json] JSON error: {}`
- `Task join error: {}` -> `[internal_error] Task join error: {}`
Wait, what if we use `TokmdError` everywhere instead?
In `tokmd-wasm`, `js_args_to_json` could return a `TokmdError`, which we then format with `err.to_string()`.
```rust
fn js_args_to_json(args: JsValue) -> Result<String, TokmdError> {
    if args.is_null() || args.is_undefined() {
        return Ok("{}".to_string());
    }

    if let Some(raw_json) = args.as_string() {
        return normalize_raw_json_args(&raw_json);
    }

    JSON::stringify(&args)
        .map_err(|_| TokmdError::invalid_settings("failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| TokmdError::invalid_settings("failed to serialize JS arguments"))
}

fn normalize_raw_json_args(raw_json: &str) -> Result<String, TokmdError> {
    serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|err| TokmdError::invalid_json(err.to_string()))?;
    Ok(raw_json.to_string())
}
```

Then in `run_mode_js`:
```rust
fn run_mode_js(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args).map_err(|e| to_js_error(e.to_string()))?;
    let data_json = extract_mode_data_json(mode, &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("[internal_error] failed to parse tokmd result JSON"))
}
```

And in `tokmd-node`:
```rust
fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("[invalid_settings] JSON error: {}", e)))
}
```
Wait, if it's `encode_args`, it means the object they passed to `run` failed to serialize to JSON. That's `invalid_settings`. Or `invalid_json`.

Is it possible we can do better by implementing a standard JS/Node Error conversion for `TokmdError`?
No, because Node uses `napi::Error` which enforces `from_reason(String)`. Wasm uses `JsError::new(String)`. The string format `[code] message` is exactly the right protocol for the Web Runner!

Wait, `[code] message` is the format extracted by `web/runner/runtime.js`.
```javascript
function extractRunnerError(error) {
    let message = "unknown runner error";
    let code = "run_failed";
    ...
    const match = message.match(/^\[([^\]]+)\]\s*(.*)$/);
```

Let's do a search-and-replace to enforce `[code] message` format for all binding panics and errors across `tokmd-wasm` and `tokmd-node`.

Wait, `[code] message` is exactly what `tokmd_core::error::TokmdError::to_string()` produces.
And `tokmd_envelope::ffi::EnvelopeExtractError::Upstream` produces it too.
But what about `EnvelopeExtractError::JsonParse(err)`?
```rust
    Self::JsonParse(err) => write!(f, "JSON parse error: {err}"),
    Self::JsonSerialize(err) => write!(f, "JSON serialize error: {err}"),
    Self::InvalidResponseFormat => write!(f, "Invalid response format"),
```
Wait, these `EnvelopeExtractError` variants do NOT have a `[code]` prefix!
If `map_envelope_error` is used for `parse_envelope(result_json)`, it does `Error::from_reason(err.to_string())`.
So `JSON parse error` becomes `Error("JSON parse error: ...")`. Which `runtime.js` falls back to `run_failed`.
We should add `[code]` prefixes to `EnvelopeExtractError` variants!
```rust
impl std::fmt::Display for EnvelopeExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonParse(err) => write!(f, "[invalid_json] JSON parse error: {err}"),
            Self::JsonSerialize(err) => write!(f, "[internal_error] JSON serialize error: {err}"),
            Self::InvalidResponseFormat => write!(f, "[internal_error] Invalid response format"),
            Self::Upstream(msg) => write!(f, "{msg}"),
        }
    }
}
```

Wait, `tokmd_envelope` is an independent crate, but it's part of the repo.
Let's check if we're allowed to touch it. "Adjacent paths allowed only if a coherent fix requires them: crates/tokmd-core/**, docs/**".
Wait, `crates/tokmd-envelope` is NOT in the allowed list! "Primary paths: crates/tokmd-python/**, crates/tokmd-node/**, crates/tokmd-wasm/**, web/runner/**".

So we CANNOT modify `tokmd-envelope/src/ffi.rs`.
If we can't touch `tokmd-envelope`, we can wrap it in `tokmd-wasm` and `tokmd-node`!

```rust
// tokmd-node/src/lib.rs
fn map_envelope_error(err: tokmd_envelope::ffi::EnvelopeExtractError) -> Error {
    use tokmd_envelope::ffi::EnvelopeExtractError::*;
    let msg = match err {
        JsonParse(e) => format!("[invalid_json] JSON parse error: {}", e),
        JsonSerialize(e) => format!("[internal_error] JSON serialize error: {}", e),
        InvalidResponseFormat => "[internal_error] Invalid response format".to_string(),
        Upstream(msg) => msg,
    };
    Error::from_reason(msg)
}
```
And similarly for `encode_args`:
```rust
fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("[invalid_json] JSON error: {}", e)))
}
```
And `Task join error`:
```rust
        .map_err(|e| Error::from_reason(format!("[internal_error] Task join error: {}", e)))
```

For `tokmd-wasm`:
```rust
fn map_envelope_error(err: tokmd_envelope::ffi::EnvelopeExtractError) -> JsValue {
    use tokmd_envelope::ffi::EnvelopeExtractError::*;
    let msg = match err {
        JsonParse(e) => format!("[invalid_json] JSON parse error: {}", e),
        JsonSerialize(e) => format!("[internal_error] JSON serialize error: {}", e),
        InvalidResponseFormat => "[internal_error] Invalid response format".to_string(),
        Upstream(msg) => msg,
    };
    to_js_error(msg)
}

fn extract_mode_data_json_after_validation(mode: &str, args_json: &str) -> Result<String, JsValue> {
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_envelope::ffi::extract_data_json(&result_json).map_err(map_envelope_error)
}
```
And for `js_args_to_json`:
```rust
fn js_args_to_json(args: JsValue) -> Result<String, JsValue> {
    if args.is_null() || args.is_undefined() {
        return Ok("{}".to_string());
    }

    if let Some(raw_json) = args.as_string() {
        return normalize_raw_json_args(&raw_json);
    }

    JSON::stringify(&args)
        .map_err(|_| to_js_error("[invalid_json] failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| to_js_error("[invalid_json] failed to serialize JS arguments"))
}

fn normalize_raw_json_args(raw_json: &str) -> Result<String, JsValue> {
    serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|err| to_js_error(format!("[invalid_json] failed to parse JSON string arguments: {err}")))?;
    Ok(raw_json.to_string())
}
```
This confirms Wasm requires formatting!
If we change `extract_mode_data_json` to return `Result<String, JsValue>`, we can use our new mapper directly instead of wrapping with `to_js_error` later. But wait, `extract_mode_data_json` is used in `tests`!
```rust
#[cfg(test)]
fn run_mode_value(mode: &str, args: &Value) -> Result<Value, String> {
    let args_json = serialize_args(args)?;
    let data_json = extract_mode_data_json(mode, &args_json)?;
    ...
```
So we should keep the internal helpers returning `Result<String, String>` and apply the formatting exactly when converting `EnvelopeExtractError` to `String`.

Wait, `EnvelopeExtractError` enum is imported from `tokmd_envelope`. In `tokmd-wasm`, we can just map it!
```rust
fn map_envelope_error_string(err: tokmd_envelope::ffi::EnvelopeExtractError) -> String {
    use tokmd_envelope::ffi::EnvelopeExtractError::*;
    match err {
        JsonParse(e) => format!("[invalid_json] JSON parse error: {e}"),
        JsonSerialize(e) => format!("[internal_error] JSON serialize error: {e}"),
        InvalidResponseFormat => "[internal_error] Invalid response format".to_string(),
        Upstream(msg) => msg,
    }
}
```
And replace `.map_err(|err| err.to_string())` with `.map_err(map_envelope_error_string)`.
For Node, `map_envelope_error` is already there. Just update it!

For `to_js_error` usages:
```rust
fn normalize_raw_json_args(raw_json: &str) -> Result<String, String> {
    serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|err| format!("[invalid_json] failed to parse JSON string arguments: {err}"))?;
    Ok(raw_json.to_string())
}
```
And in `js_args_to_json`:
```rust
fn js_args_to_json(args: JsValue) -> Result<String, JsValue> {
    if args.is_null() || args.is_undefined() {
        return Ok("{}".to_string());
    }

    if let Some(raw_json) = args.as_string() {
        return normalize_raw_json_args(&raw_json).map_err(to_js_error);
    }

    JSON::stringify(&args)
        .map_err(|_| to_js_error("[invalid_json] failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| to_js_error("[invalid_json] failed to serialize JS arguments"))
}
```

And in `run_mode_js` and `run_analyze_js`:
```rust
    JSON::parse(&data_json).map_err(|_| to_js_error("[internal_error] failed to parse tokmd result JSON"))
```
Wait, if it's an internal error, `web/runner/runtime.js` will now parse it properly.

Let's do this! This fulfills the goal "improve runtime developer experience in one coherent way" by fixing exactly the parsing errors format required by the JS frontend across Node and Wasm bindings, which was breaking runtime metrics/diagnostics reporting.

Wait, are there tests failing because of this change?
In `tokmd-node`:
```rust
    #[test]
    fn encode_args_maps_serde_error() {
        let err = encode_args(&BadSerialize).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("JSON error"));
    }

    #[test]
    fn parse_envelope_maps_json_error() {
        let err = parse_envelope("{").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("JSON parse error"));
    }

    #[test]
    fn run_blocking_maps_join_error() {
        let err = block_on(run_blocking(|| panic!("boom"))).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Task join error"));
    }
```
These tests assert `message.contains("...")`, which means prepending `[invalid_json]` will NOT break them!

Wait, `tokmd-wasm` has tests too:
```rust
    #[test]
    fn run_analyze_with_invalid_args_json_fails() {
        let err = run_analyze_js(JsValue::from_str("{")).unwrap_err();
        let message = err.dyn_into::<JsError>().unwrap().message().as_string().unwrap();
        assert!(message.contains("failed to parse JSON string arguments"));
    }
```
These also use `.contains("...")`, so prepending `[invalid_json]` won't break them either.

Is there any test asserting EXACT match? Let's check.

Option A (recommended)
Enhance Node and Wasm bindings to explicitly enforce the `[code] message` FFI error-format contract natively for internal exceptions.
By prefixing JSON parse/serialize errors with `[invalid_json]` and internal join/extract errors with `[internal_error]`, we ensure JS callers (like `web/runner`) can programmatically distinguish and handle these failures instead of falling back to untyped `run_failed` messages.

Option B
Wait for Wasm/Node engines to adopt the JSON envelope for ALL errors including parse failures (which implies altering the `run` methods to return envelopes instead of exceptions). This breaks semantic JavaScript expectations (where bad arguments throw exceptions, not return `{ok: false}`).

I will implement Option A. It's safe, requires no core schema changes, directly targets the assigned persona ("runtime developer experience", "unclear or low-context error messages"), and remains tightly within the `bindings-targets` shard.
The clippy output indicates it successfully built without warnings.
The tests for `tokmd-node` passed correctly.

I'll execute the `wasm-pack` tests locally by installing `wasm-pack`. Wait! `wasm-pack` is not strictly necessary for local Wasm verification if we can just test the Rust unit tests. The prompt mentions "feature-boundary checks and tests for Wasm are run using `wasm-pack test --node`". The memory says I can run `wasm-pack test --node`. But `wasm-pack: command not found`.
Actually, `CI=true cargo test --verbose` inside `tokmd-wasm` ran successfully (as shown above):
```
running 16 tests
...
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Let's double-check the plan. I will complete the pre-commit steps.
I have executed all builds and tests for the affected crates. They pass.
The frontend uses standard JS unit tests using Web Runner which relies on the error parsing in `web/runner/runtime.js`.
The Wasm bundle needs to be built so we can run `npm test` inside `web/runner` to ensure `runtime.js` regex matches the modified output. But wait, `wasm-pack test --node --features analysis` doesn't work locally without `wasm-pack`.
However, there are `node` unit tests using `mocha`? Let's check `web/runner/package.json`.
All tests pass. Wasm check passes. Node test passes. Browser runner unit tests pass.

I'm ready to commit.

The GitHub CI Check Suite failed with:
`PR plan estimated 139 LEM (>125 hard ceiling) — apply ci-budget-override or full-ci to bypass, or split the PR.`

This is the `ci-plan` advisory limit. We've changed the error handling in `tokmd-wasm` and `tokmd-node`. The `cargo build` for both targets likely triggered a lot of rebuilds.
To bypass this, we just need to use `cargo xtask jules-index` to append the appropriate label (or bypass) to the PR, or use `gh pr edit` if we had GitHub CLI access, or we can just leave it as it is because this CI failure is advisory and we can bypass it by editing the PR title / body. But the memory says "In the Jules prompt-to-PR pipeline, the assistant's final response must act as the authoritative PR submission packet." and the prompt says "Your final response must contain the exact PR title followed by the full markdown PR body".
Wait, the CI fail is just a budget override limit because Wasm + Node rebuilds cost 139 LEM (Limit Estimation Metric) while the ceiling is 125. The error says `apply ci-budget-override or full-ci to bypass`. This is done using GitHub Labels!

If the CI is failing purely because of `ci-budget-override` (139 LEM > 125), I can't add labels directly via tools unless there is a `gh` CLI. Wait, Jules doesn't apply labels using `gh`. The instructions for the check run say: "apply ci-budget-override or full-ci to bypass". I will just add the word `ci-budget-override` into the PR body to see if the GitHub Action parses it, or just ignore it since it's an administrative CI limit on LEM.
Wait, "Your goal now is to analyze the provided check run details... and make a fix."
Can I reduce the blast radius?
In `tokmd-wasm/src/lib.rs` and `tokmd-node/src/lib.rs` I only modified a few lines. Why did the LEM estimate jump to 139?
"24 lane(s) selected, estimated 139 LEM (override-required)"
Because touching `tokmd-node` and `tokmd-wasm` triggers tests in both, plus Node.js runner tests, etc.
To fix the CI, maybe I should just add `ci-budget-override` to the PR title or body?
Or maybe I can change the PR title or envelope? Wait, Jules doesn't have a label setter.
If I just re-submit with the exact same code, the user expects me to "fix" the CI failure.

Wait, the check failure is `PR Plan (advisory)`. It's a GitHub action running `cargo xtask ci-plan ... --enforce`.
Can I bypass it by adding `ci-budget-override` to the `.github/workflows`? No, that's not allowed.
The PR is already pushed and CI ran. So the PR exists! But I am not doing `gh pr edit`. I am supposed to submit again.
Actually, wait, if I add `ci-budget-override` to the PR body, does the PR title / body parser map it to labels? No.
Let's see how `ci-plan` determines labels.
In `tokmd/xtask/src/ci_plan.rs` or something?
Ah! `xtask/src/tasks/ci_plan.rs:435` checks `labels_set.contains("ci-budget-override")`.
Where does `labels_set` come from?
It comes from `--labels-json "$LABELS_JSON"`.
The GitHub action `.github/workflows/ci.yml` passes the PR labels to `ci-plan`.
Since I cannot add a PR label myself through code, I have to rely on the user to add it. But I must output a complete response. Wait, there's a file called `policy/ci-risk-packs.toml`. Let's see what's in there.

Wait, `ci-plan.json` output above says `24 lane(s) selected, estimated 139 LEM`.
The `xtask/ci_plan.rs` adds up LEM cost. Since I changed `crates/tokmd-node` and `crates/tokmd-wasm`, I hit both `risk_pack.wasm` and `risk_pack.node` (or whatever the node one is). Wait, `tokmd-node` is NOT listed in the `ci-risk-packs.toml` file!
If it's not listed, what does `ci-plan` do? It includes fallback lanes!
And Wasm adds `wasm_compile_test` which might cost a lot.
Ah, `tokmd-node` is not in `ci-risk-packs.toml`.
Let me check `ci-risk-packs.toml`.
Wait, `tokmd-node` and `tokmd-python` are missing. When a file is missing from any risk pack, `ci-plan.rs` might assign it to a "catch_all" or default fallback (which probably includes `build_test_linux`, `build_test_windows`, `msrv_check`, etc.).
If it falls back, it blows up the LEM budget.

I can just add `tokmd-node` and `tokmd-python` to `ci-risk-packs.toml`. But this violates the Prompt: "Do not edit shared ledgers, shared daily logs, or shared scaffolding unless this prompt explicitly allows it."
"This is a prompt-to-PR pipeline. You must finish with a PR-worthy diff: 1) PR-ready patch..."
"If behavior changes, update docs/tests/contracts together when relevant."

The only way I can "apply ci-budget-override" is to include it in the PR title, PR body, or ask the user. Wait! The prompt says "In the Jules prompt-to-PR pipeline, the assistant's final response must act as the authoritative PR submission packet... Your final response must contain the exact PR title followed by the full markdown PR body."
I can just add `ci-budget-override` to the PR body! But wait, `ci_plan.rs` says:
```rust
        labels_set.contains("full-ci") || labels_set.contains("ci-budget-override");
```
And it reads from `$LABELS_JSON` which is passed from the GitHub workflow. It doesn't parse the PR body.

What if I split the PR? "or split the PR".
I modified `tokmd-node` and `tokmd-wasm`. What if I only modify ONE of them, e.g. `tokmd-node`, to stay under budget?
Wait, the prompt says "Improve runtime developer experience in one coherent way." "Target ranking 1) unclear or low-context error messages ... Scope note: Runtime DX across Python/Node/Wasm/browser-runner surfaces."
A single coherent PR for this makes sense. The CI failure is advisory. I will submit and state the user needs to label the PR.

Actually! What if I add `ci-budget-override` as a branch name or something? No, it says "apply `ci-budget-override` or `full-ci` to bypass, or split the PR." This is an instruction to the human user who reviews the PR in GitHub. Since this CI failure is advisory, my code is perfectly correct and there are no actual test failures.

Let's just resubmit the exact same patch and add a note to the PR body that the CI budget was exceeded because it's a cross-target change, so the user should add the label.
Or, does the system simulate the PR label? No, the system checks if the check suite failed. "Your goal now is to analyze the provided check run details... identify the root cause of the failure, and make a fix."
Wait! If the system *blocks* my success because of this advisory failure, I must bypass it.
Can I modify `.github/workflows/ci.yml`? It says `policy/**, xtask/**, .github/workflows/**` is in `risk_pack.policy` which costs LEM too.
Can I bypass it by removing Wasm changes? If I only do Node changes, I might drop the LEM under 125.
Let's see the LEM of Wasm vs Node.
Actually, if I just revert `tokmd-node` and only submit `tokmd-wasm`? No, the prompt requires "across Python/Node/Wasm/browser-runner surfaces".
Wait. If `tokmd-node` is missing from `ci-risk-packs.toml`, it falls into the fallback which runs `build_test_windows`, `msrv`, etc.
If I add `tokmd-node` and `tokmd-python` to `risk_pack.bindings` in `ci-risk-packs.toml`, it will reduce the LEM because they won't trigger the fallback!

Let's verify `tokmd-node` is missing from `ci-risk-packs.toml`.
Yes, `tokmd-node` is NOT in the `ci-risk-packs.toml`. Wasm is in `risk_pack.wasm` which adds `wasm_compile_test`.

Let's add `tokmd-node` and `tokmd-python` to `ci-risk-packs.toml`.
```toml
[risk_pack.bindings]
description = "Language bindings and target-specific FFI."
paths = [
  "crates/tokmd-node/**",
  "crates/tokmd-python/**",
]
lanes = ["build_test_linux"]
deep_lanes = ["build_test_windows"]
```
If I do this, it will reduce the LEM! But wait, is modifying `ci-risk-packs.toml` allowed?
"Do not edit shared ledgers, shared daily logs, or shared scaffolding unless this prompt explicitly allows it."
Is `ci-risk-packs.toml` considered "shared scaffolding"? It is under `policy/`, and the README says `policy/ — shared policy, gate profiles, shard maps, schemas`. "Agents must not write shared runbooks/policy/templates unless the prompt explicitly says the run is a Jules-scaffolding run."
So I CANNOT edit `policy/ci-risk-packs.toml`.

If I cannot edit `ci-risk-packs.toml` and cannot edit `xtask/ci_plan.rs`, and the CI fails, how do I "make a fix"?
Wait! Is it possible to just bypass the ci-plan by faking the output? No, I don't control the runner.

What if I change the PR title or body so that the GitHub Action automatically adds the label?
Some GitHub actions use `gh pr edit --add-label "ci-budget-override"` if a certain keyword is in the PR body.
Let's look at `.github/workflows/ci.yml`.
The action uses `github.event.pull_request.labels`.
If there's no way to automatically apply labels via the PR body, then the system instruction `PR plan estimated 139 LEM (>125 hard ceiling) — apply ci-budget-override or full-ci to bypass, or split the PR.` means I must literally split the PR!
"or split the PR"
If I split the PR, I can just do `tokmd-wasm` OR `tokmd-node`. The budget is currently 139 LEM. `tokmd-wasm` adds `wasm_compile_test`. `tokmd-node` triggers fallback which might trigger a lot.

Let's do `tokmd-node` ONLY, and stash `tokmd-wasm` changes.
Actually, wait! `tokmd-node` and `tokmd-wasm` changes were BOTH done in this PR.
Let's see the LEM of ONLY `tokmd-wasm`.

Wait, `ci-plan.json` output above says `estimated 139 LEM (override-required)`. The limit is 125 LEM.
If I pass `--labels-json '["ci-budget-override"]'`, it bypasses the error. But the GitHub CI check runs without it (since the workflow doesn't allow me to add the label automatically).
Is there any way I can lower the LEM under 125?
If I only commit `crates/tokmd-node/src/lib.rs` and the `.jules` artifacts, what happens?

Wait, `estimated 114 LEM (high-cost)` is under `125`.
So if I ONLY include `tokmd-node` changes, the budget check PASSES (it gives a `::warning::` but exits with 0).
The PR that failed the check suite was doing BOTH `tokmd-wasm` and `tokmd-node`. The error instructed me to `apply ci-budget-override ... or split the PR`.
"or split the PR" means I must only submit one part at a time if the user cannot add the label for me!
Since the user cannot add the label during the Jules evaluation flow, I must follow the instruction to "split the PR".
I will submit ONLY the `tokmd-node` changes in this PR.

I already committed `tokmd-wasm` in a stash / separate commit and then `git reset --hard HEAD~1` it. Wait, I ran `git commit --amend --no-edit`. So the `WIP` commit has everything except `tokmd-wasm`.
Let me fix `pr_body.md` to reflect that this PR only covers `tokmd-node` (and I will state the `tokmd-wasm` fix will be a follow-up) to fit within the `ci-budget-override` limit.
