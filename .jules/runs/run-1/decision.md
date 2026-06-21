# Decision

## Option A (Recommended)
Add `if !args.is_object() { return args; }` or similar protection to `scan_arg_object` in `crates/tokmd-core/src/ffi/parse.rs`, or simply ensure that the returned value is still useful, but `unwrap_or` on `get` can be problematic if `args` is not an object. The root issue is `unwrap_or(args)`. `args.get("scan")` returns `None` if `args` is not an object, which then returns `args`. If `args` is an array or string, it gets passed along. However, `parse.rs` assumes the `args` passed to it are objects because of `run_json_inner` in `mod.rs`: `if !args.is_object() { return Err(...) }`.

Wait, in `run_json_inner`, `args` is validated to be an object:
```rust
    let args: Value =
        serde_json::from_str(args_json).map_err(|err| TokmdError::invalid_json(err.to_string()))?;
    if !args.is_object() {
        return Err(TokmdError::invalid_json(
            "Top-level JSON value must be an object",
        ));
    }
```
If `args` is an object, then `args.get("scan")` returns `Some(scan_value)`. If `scan` is not an object, `scan_arg_object` will return `scan_value`. Then `parse_string_array` will be called on `scan_value`. Wait, `parse_scan_settings` uses `scan_arg_object`:
```rust
pub(super) fn scan_arg_object(args: &Value) -> &Value {
    args.get("scan").unwrap_or(args)
}
```
If `args` has `"scan": "string"`, then `scan_arg_object` returns `"string"`. Then `obj.get("paths")` on `"string"` returns `None`, so `parse_string_array` uses the default! Wait, `get` on a string value returns `None`. It silently ignores the invalid `"scan"` object and uses defaults for all fields!

Let's test this:

The tests confirm that passing `"scan": "not an object"` or `"lang": "not an object"` silently falls back to empty defaults, because `Value::get(field)` on a string returns `None`, the same as if the fields were missing! This is a gap in the trust boundary, as we silently ignore a malformed settings block instead of validating the FFI input strictly.

Let's look at `scan_arg_object` and `parse_*_settings`:

```rust
pub(super) fn scan_arg_object(args: &Value) -> &Value {
    let scan_val = args.get("scan").unwrap_or(args);
    // If it's explicitly "scan": "something_else", we just return it and it acts like an empty object since it's not an object.
}
```

Wait! It's even worse. If `"scan"` is provided but it's not an object, `args.get("scan").unwrap_or(args)` returns the `"scan"` value (a string, array, etc). Then `scan_val.get("paths")` returns `None`. So it silently ignores `"scan"` being a string/array, AND it ignores the rest of `args` because we are now calling `get` on a string/array instead of `args`!

Actually, it says: "Missing keys use sensible defaults. Invalid values return errors (no silent fallback to defaults)". But `{"scan": "not an object"}` is an invalid value that does NOT return an error, it silently falls back to defaults!

Let's check `parse_lang_settings`:
```rust
pub(super) fn parse_lang_settings(args: &Value) -> Result<LangSettings, TokmdError> {
    let obj = args.get("lang").unwrap_or(args);
```
Same thing here.

And `parse_module_settings`, `parse_export_settings`, `parse_analyze_settings`, `parse_cockpit_settings`, `parse_diff_settings`.

Wait, in `parse_diff_settings`:
```rust
pub(super) fn parse_diff_settings(args: &Value) -> Result<DiffSettings, TokmdError> {
    let obj = args.get("diff").unwrap_or(args);

    let from = parse_required_string(obj, "from")?;
    let to = parse_required_string(obj, "to")?;
```
If `"diff": "string"`, then `obj` is a string, `parse_required_string` will return `None`, but since it's required it will return an error because it expects a string. However, if there are optional properties, they would fall back to default.

For `scan`, `paths` is optional (defaults to `vec![".".to_string()]`).
For `lang`, `top` is optional (defaults to `0`).

Wait, how should we fix this?
Option A: Create a helper function in `parse.rs` like `get_arg_object<'a>(args: &'a Value, field: &str) -> Result<&'a Value, TokmdError>` which returns `args` if `field` is missing, but returns `Err` if `field` is present but not an object.
And use it in `settings_parse.rs`.

Let's check `parse.rs`:
```rust
pub(super) fn scan_arg_object(args: &Value) -> &Value {
    args.get("scan").unwrap_or(args)
}
```

We can change it to return `Result<&Value, TokmdError>` and validate it!
Wait, if it returns `Result`, we need to change callers.

```rust
pub(super) fn get_settings_object<'a>(args: &'a Value, field: &str) -> Result<&'a Value, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(args),
        Some(Value::Object(_)) => Ok(args.get(field).unwrap()),
        Some(_) => Err(TokmdError::invalid_field(field, "an object")),
    }
}
```

Wait, `scan_arg_object(args: &Value) -> &Value` does not return a `Result`. Let's change `scan_arg_object` to return `Result<&Value, TokmdError>` or just replace it with `get_settings_object(args, "scan")?`.

Let's do this for `scan`, `lang`, `module`, `export`, `analyze`, `cockpit`, `diff`.

Let's try to update `parse.rs` and `settings_parse.rs`.
This fixes the boundary gap where `{"scan": []}` or `{"lang": "rust"}` would silently act like an empty object `{}` and drop all actual arguments inside `args` because `get()` on non-objects returns `None`.

I will:
1. Update `crates/tokmd-core/src/ffi/parse.rs` to replace `scan_arg_object` with `get_settings_object(args: &Value, field: &str) -> Result<&Value, TokmdError>`.
2. Add a test in `parse.rs` for `get_settings_object`.
3. Update `crates/tokmd-core/src/ffi/settings_parse.rs` to use `get_settings_object(args, "scan")?`, `get_settings_object(args, "lang")?`, etc.
4. Update `tests/ffi_bug_test.rs` to ensure the correct `invalid_settings` error is returned.
5. Review other files where `scan_arg_object` is used.

Are there other usages of `scan_arg_object`?

Perfect. Option A solves the gap correctly by hardening the trust boundary for FFI inputs.

## Option B
Keep the current behavior and document that `scan: string` drops all configuration. This is unacceptable for a security/trust boundary. FFI inputs are inherently untrusted and must be strictly validated.

Decision: Proceed with Option A.
