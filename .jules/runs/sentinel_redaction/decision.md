# Sentinel Redaction Decision

## Target
The redaction correctness and leakage prevention. Specifically, fixing how `excluded` patterns are redacted across different report modes (`lang`, `module`, `diff`). Currently, `scan_args` is called with `None` for the redact mode in `write_lang_report_to` and `write_module_report_to`. This leaks excluded patterns (e.g. paths passed via `--exclude`) when generating JSON receipts for `lang` and `module` reports even when they might contain sensitive information. However, `LangArgs` and `ModuleArgs` do not take a `redact` mode in the CLI. Wait, does `tokmd lang` and `tokmd module` support redaction? No, they don't have a `--redact` flag.

Let's look at another target: `strip_prefix_redacted` and `excluded_redacted` logic.
In `scan_args` in `tokmd-scan-args/src/lib.rs`:
```rust
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
        excluded_redacted,
```

Wait, `excluded_redacted` only happens if `should_redact` is true. But `should_redact` checks `RedactMode::Paths | RedactMode::All`.
But wait, if we look at `RedactMode::Paths`, it redacts paths, but should it redact excluded patterns? The excluded patterns could contain paths. Yes.

Is there a place where redaction is not correctly applied?
In `crates/tokmd-format/src/lib.rs` line 44:
```rust
fn redact_module_roots(roots: &[String], redact: RedactMode) -> Vec<String> {
    if redact == RedactMode::All {
        roots.iter().map(|r| short_hash(r)).collect()
    } else {
        roots.to_vec()
    }
}
```
This is correct according to the `RedactMode` logic (only `All` redacts modules, `Paths` only redacts file paths).

In `crates/tokmd-format/src/lib.rs` line 406:
```rust
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();
```
Is `strip_prefix` redacted?
```rust
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
```
Wait, `args.strip_prefix` is a prefix, not necessarily a path with an extension. Calling `redact_path` on a prefix will preserve its extension if it has one, but it's meant to strip a prefix from paths. `strip_prefix` is a `PathBuf`.
Is there a problem here? If `strip_prefix` is just a directory name, `redact_path` will just hash it to 16 characters.

Let's check if there's any FFI parsing/trust boundaries.
`crates/tokmd_core/ffi`? Let's check `crates/tokmd/src` or FFI.
In `crates/tokmd-core/src/ffi.rs`, `run_json_inner` parses `args_json` to `Value` using `serde_json::from_str`. Memory notes state: "In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."

Looking at `run_json_inner`:
```rust
fn run_json_inner(mode: &str, args_json: &str) -> Result<Value, TokmdError> {
    // Parse common scan settings from the JSON
    let args: Value = serde_json::from_str(args_json)?;
    let inputs = parse_in_memory_inputs(&args)?;
```

If `args_json` is `"0"` or `""`, it will parse (or fail). If `"0"`, `args` is a JSON number. Then `parse_in_memory_inputs(&args)` is called.
```rust
fn parse_in_memory_inputs(args: &Value) -> Result<Option<Vec<InMemoryFile>>, TokmdError> {
    let scan_obj = args.get("scan");
    let root_inputs = args.get("inputs").filter(|value| !value.is_null());
```
`args.get()` on a number returns `None`. `parse_in_memory_inputs` seems safe because `get()` on `Value` just returns `None` if it's not an object.
Wait, the memory note says: "Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."
Is there a panic downstream?

Let's check `parse_scan_settings`:
Aha! `args.get(field)` works on `Value::Number` and `Value::String` (it returns `None` for them). BUT wait, what does `args.get(field)` do if `args` is not an object but an array? It also has `.get()` for index! But `field` is a `&str`, and `Value::get(&str)` only works if it's an object.
If we look at `serde_json::Value::get`:
```rust
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        index.index_into(self)
    }
```
If `Value` is a scalar, `.get("field")` returns `None`. It doesn't panic.

Wait, the prompt says: "In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."

Wait! Is there a place where it expects an object and does something else?
`run_json_inner`
```rust
fn run_json_inner(mode: &str, args_json: &str) -> Result<Value, TokmdError> {
    // Parse common scan settings from the JSON
    let args: Value = serde_json::from_str(args_json)?;

    // Explicitly verify it's an object as per memory note!
    // But currently it does not.
```
If we pass `"0"` to `run_json`, `args` becomes `Value::Number(0)`.
Then `parse_in_memory_inputs(&args)` uses `args.get("scan")` which returns `None`.
Then `parse_scan_settings(&args)` calls `scan_arg_object(&args)` which returns `args` (since `args.get("scan")` is `None`).
Then `parse_scan_settings` calls `parse_string_array(obj, "paths", ...)` which calls `obj.get("paths")` which returns `None`.
Then it returns default `ScanSettings`.
Then `parse_lang_settings(&args)` returns default settings.
Then it executes `lang_workflow(&scan, &settings)`. Wait, it executes the scan workflow with default settings!

Does anything panic if we pass `"0"`? Let's write a quick test and run it.
Okay, it doesn't panic. It just happily returns a successful JSON output. However, passing `"0"` and getting a successful scan of `"."` seems wrong, and the memory note explicitly says:
"In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."

Wait! There might be downstream panics somewhere else! Maybe in `export` or `diff`?
Let's see: `diff_missing_from_returns_error` tests for diff missing `from` which is expected to return error.
If we test `run_json("diff", "\"0\"")`, what happens?
Okay, testing `diff` with `"0"` also returns error:
```json
{"ok":false,"error":{"code":"invalid_settings","message":"from field must be a valid JSON object or path"}}
```
So no panic there either.

Wait, the prompt says:
> In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects.

Oh! I'm "Sentinel 🛡️"! The assigned prompt says:
> Target ranking
> 1) redaction correctness and leakage prevention
> 2) FFI parsing / trust boundaries

Let's fix the FFI parsing / trust boundaries. And the memory explicitly says:
"In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."

I need to apply this fix. I'll modify `crates/tokmd-core/src/ffi.rs` to explicitly verify `args.is_object()`.

Wait, the prompt's assigned focus is:
> Focus: Keep this lane tightly focused on redaction correctness and leakage prevention.
> Scope note: Redaction and trust-bearing output hardening in the core pipeline.

Wait, the scope note strictly says "Redaction and trust-bearing output hardening in the core pipeline."
What was my finding regarding redaction correctness?
In `crates/tokmd-format/src/lib.rs`, `write_lang_report_to` and `write_module_report_to` call `scan_args(&args.paths, global, None)` instead of `Some(args.redact)`. But wait, `LangArgs` does not even have a `redact` field! So there's no `args.redact`! Wait, look at `crates/tokmd-types/src/lib.rs`: `LangArgs` definition.
Ah, `LangArgs` has no `redact` field, and neither does `ModuleArgs`. So they don't support redaction. But wait, what if `global.excluded` contains sensitive paths? They will be leaked in the `scan` part of the receipt, because `global.excluded` is preserved verbatim in the `scan` object. Is it a leak? Yes, but if redaction is not requested, they aren't redacted.
If redaction is requested (e.g. for `export`), then `global.excluded` is redacted.

Let's look at `scan_args` logic again.
```rust
pub fn scan_args(paths: &[PathBuf], global: &ScanOptions, redact: Option<RedactMode>) -> ScanArgs {
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let excluded_redacted = should_redact && !global.excluded.is_empty();
```
Is there a bug here?
Wait! In `crates/tokmd-format/src/lib.rs`, the `ExportReceipt` uses `scan_args(&args.paths, global, Some(args.redact))`.
Wait! What about the FFI trust boundary bug from the memory note?
"In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding."

Since `target ranking 2` is FFI parsing / trust boundaries, and the memory note explicitly mentions it, I should fix it.
But wait, the scope note specifically says:
"Focus: Keep this lane tightly focused on redaction correctness and leakage prevention.
Scope note: Redaction and trust-bearing output hardening in the core pipeline.
If the strongest target you find is outside the shard, record it as friction instead of chasing it."

Is there a bug in redaction correctness and leakage prevention?
Wait, if `excluded_redacted` is `should_redact && !global.excluded.is_empty()`, then if `global.excluded` is empty, `excluded_redacted` is false.
Wait, `strip_prefix_redacted = should_redact && args.strip_prefix.is_some()`.
Let's see `strip_prefix` handling:
```rust
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
```
Is `redact_path` correct for `strip_prefix`?
A prefix might not be a single file path, it might be a directory path like `src/secret`.
`redact_path("src/secret")` will return a 16-char hash, without extension.
But wait! If the user provides a `strip_prefix`, does the `redact_path` hash match the beginning of the hashed file paths?
`redact_path` on a file path hashes the ENTIRE file path.
If I have file path `src/secret/foo.rs`, its redacted path is `hash("src/secret/foo.rs") + ".rs"`.
If the user passes `strip_prefix` `src/secret`, its redacted form is `hash("src/secret")`.
So the redacted prefix and the redacted file paths have NO relationship! They cannot be correlated.
This seems correct from a redaction standpoint: we don't want to leak what the prefix was.
BUT wait, what is the bug?

Let's check if the redaction correctly redacts `strip_prefix`.
In `crates/tokmd-format/src/lib.rs`:
```rust
pub fn write_export_json_to<W: Write>(
...
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();
...
            args: ExportArgsMeta {
...
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
                strip_prefix_redacted,
```
Wait! What if `args.redact == RedactMode::All`? `should_redact` is true. `args.redact == RedactMode::Paths` is true.
Is there any other leak?
Let's reconsider: "In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."

The memory rule explicitly states this. It says "parse successfully but CAN cause downstream panics if blindly treated as objects." Maybe the panic doesn't happen with `"0"` but happens with something else, or maybe it just violates the invariant.
I will fix the FFI parser to verify `.is_object()`. Is that in the shard? Yes, `tokmd-core` is NOT in the shard! The shard paths are `crates/tokmd-types/**`, `crates/tokmd-scan/**`, `crates/tokmd-model/**`, `crates/tokmd-format/**`. So `tokmd-core` is NOT allowed, UNLESS "a coherent fix requires touching an adjacent contract surface".
But wait, if the strongest target is outside the shard, I should "record it as friction instead of chasing it" and pick a target inside the shard.
The prompt says: "Focus: Keep this lane tightly focused on redaction correctness and leakage prevention."
"Scope note: Redaction and trust-bearing output hardening in the core pipeline."
So the FFI bug is just friction. I should look for a redaction bug in `tokmd-types`, `tokmd-scan`, `tokmd-model`, or `tokmd-format`.

Let's look for redaction/leakage bugs in these crates.
In `crates/tokmd-format/src/lib.rs` (Export JSON):
```rust
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();
```
Is `should_redact` correct? What if `args.redact == RedactMode::None`? Then `should_redact` is false, and `strip_prefix_redacted` is false.
Wait... what if `args.redact == RedactMode::All`? Then `module_roots` is redacted in `export.module_roots`.
```rust
fn redact_module_roots(roots: &[String], redact: RedactMode) -> Vec<String> {
    if redact == RedactMode::All {
        roots.iter().map(|r| short_hash(r)).collect()
    } else {
        roots.to_vec()
    }
}
```
Wait, if `module_roots` is redacted when `redact == RedactMode::All`, shouldn't `module_roots` ALSO be redacted in `ExportArgsMeta`?
```rust
            args: ExportArgsMeta {
                format: args.format,
                module_roots: module_roots.clone(),
                module_depth: export.module_depth,
                children: export.children,
```
Wait! `module_roots` IS redacted because it's using the local variable `module_roots` which was just redacted by `redact_module_roots`!
```rust
    let module_roots = redact_module_roots(&export.module_roots, args.redact);
```
So that is correct.

What else? Let's check `export.rows`. It uses `redact_rows(&export.rows, args.redact)`.
```rust
            data: ExportData {
                module_roots: module_roots.clone(),
                rows: redact_rows(&export.rows, args.redact).map(|r| r.into_owned()).collect(),
            },
```
This looks correct.

What about `ExportReceipt` vs `ExportMeta`?
In `write_export_json_to`, there are two paths: if `data_mode` it renders `ExportReceipt` (which includes `data: ExportData`), if `meta_only` it renders `ExportMeta` (which lacks `data`).
Are there any other fields in `ExportArgsMeta` that need redaction?
`strip_prefix` is redacted. `module_roots` is redacted. `redact` is stored. `format`, `module_depth`, `children`, `min_code`, `max_rows` are primitives.

What about `ScanArgs`?
```rust
pub fn scan_args(paths: &[PathBuf], global: &ScanOptions, redact: Option<RedactMode>) -> ScanArgs {
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
```
Wait! `global.excluded` is redacted. `args.paths` is also redacted!
```rust
    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }
```
Is `args.paths` redacted correctly? `normalize_scan_input(p)` converts it to a string. Then `redact_path(p)` hashes it and adds the extension. BUT WAIT! What if `paths` is a directory path? Like `.`, or `src`?
`redact_path` on `.` or `src` will preserve extension? `.` doesn't have an extension. `src` doesn't have an extension. So it will return a 16-char hash.
Is it a problem? If the path was `.` and it gets hashed to `e3b0c44298fc1c14`, it's redacted. Is that correct? Yes.

Wait! What about the `diff` tool? Does it redact things?
Let's grep for `redact` in `diff`.
Let's see if `diff` uses redaction.
```rust
pub fn write_diff_report_to<W: Write>(
    mut out: W,
    report: &DiffReport,
    args: &DiffArgs,
) -> Result<()> {
...
```
I'll check `write_diff_report_to`.
Okay, there is no `write_diff` in `tokmd-format`. It might be in `tokmd-diff`.

Wait! Let's go back to the memory explicitly stating:
"In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."

Wait, `tokmd-core` IS in the `core-pipeline` shard? Let's check `envelope.json`:
```json
  "allowed_paths": [
    "crates/tokmd-types/**",
    "crates/tokmd-scan/**",
    "crates/tokmd-model/**",
    "crates/tokmd-format/**",
    "docs/schema.json",
    "docs/SCHEMA.md",
    "crates/tokmd/tests/**"
  ],
```
Ah! `tokmd-core` is NOT in the allowed paths! So I cannot modify `tokmd-core::ffi`.

If the strongest target is outside the shard, I should record it as friction!
> "If the strongest target you find is outside the shard, record it as friction instead of chasing it."

Is there any OTHER target inside the shard?
Target ranking:
1) redaction correctness and leakage prevention
2) FFI parsing / trust boundaries
3) subprocess / environment / path boundary hardening
4) receipt/schema trust and deterministic safety
5) unsafe minimization / justification
6) production panic cleanup on trust-bearing surfaces

Is there any redaction correctness bug in `tokmd-format`, `tokmd-types`, `tokmd-scan`, or `tokmd-model`?
Wait! In `write_lang_report_to` and `write_module_report_to` in `crates/tokmd-format/src/lib.rs`, `scan_args` is called with `None` for redact mode!
```rust
                scan: scan_args(&args.paths, global, None),
```
But `scan_args(&args.paths, global, None)` will NOT redact the excluded paths in the receipt!
Is that a problem?
`tokmd lang` and `tokmd module` do NOT have a `--redact` flag in their CLI args. They always output unredacted receipts. But wait... `tokmd-scan-args` supports redaction, but `lang` and `module` don't use it.
Is it a leakage? Yes, because users can't redact their `lang` and `module` receipts, or they run it and it leaks `excluded`. But since there is no `redact` arg, they can't ask for redaction.

What about `scan_args` function itself?
```rust
pub fn scan_args(paths: &[PathBuf], global: &ScanOptions, redact: Option<RedactMode>) -> ScanArgs {
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
```
If `redact` is `Some(RedactMode::None)`, `should_redact` is false, so it doesn't redact `global.excluded`.
If `redact` is `Some(RedactMode::Paths)` or `Some(RedactMode::All)`, `should_redact` is true, so it REDACTS `global.excluded`.
And sets `excluded_redacted` to `true` (if not empty).
BUT WAIT!
If `global.excluded` is REDACTED, then the receipt contains hashes in `excluded`!
And `excluded_redacted` is true.

What about `strip_prefix_redacted` in `ExportArgsMeta`?
```rust
                strip_prefix_redacted: bool,
```
```rust
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();
```

Wait, is there any panic in `tokmd-format` or `tokmd-model` when parsing receipts?
Let's see if there is any other bug.
What about "receipt/schema trust and deterministic safety"?
Wait, look at `tokmd-model` and `tokmd-types`.

Wait, earlier I ran:
```
grep -rn 'redact' crates/tokmd-types/src crates/tokmd-scan/src crates/tokmd-model/src crates/tokmd-format/src
```
In `crates/tokmd-types/src/lib.rs:324`:
```rust
    /// True if `excluded` patterns were redacted (replaced with hashes).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub excluded_redacted: bool,
```

Is there a `diff` receipt that has a `scan` and does it redact paths?
Wait, `DiffArgs` does not have a `redact` mode? Wait, Diff has redaction?
No, the diff compares two receipts. Receipts ALREADY have their paths redacted or unredacted.
So Diff does not redact paths itself.

Wait, the prompt says:
> Target ranking
> 1) redaction correctness and leakage prevention
> ...
> If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix.
> ...
> If the strongest target you find is outside the shard, record it as friction instead of chasing it.

Since I found that `tokmd-core::ffi` has an FFI trust boundary bug (it doesn't use `.is_object()`), but it's outside my allowed paths, I should record it as friction!
Wait, but is there ANY target IN the shard? Let's verify.
Let's check if there is a panic in `tokmd-scan`.
Is there ANY target inside the shard?
The prompt specifically says:
"Target ranking
1) redaction correctness and leakage prevention
...
Focus: Keep this lane tightly focused on redaction correctness and leakage prevention.
Scope note: Redaction and trust-bearing output hardening in the core pipeline."

In `crates/tokmd-format/src/lib.rs`, the `should_redact` logic handles `export` redaction.
```rust
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();
```
Is there a problem here?
Wait! What if `should_redact` is FALSE? Then `strip_prefix_redacted` is false.
What happens to `args.strip_prefix`? It is stored as is in `ExportArgsMeta`:
```rust
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
```
Is `redact_path` safe to run on `strip_prefix`?
If `strip_prefix` is `""`, `redact_path` will hash it to `e3b0c44298fc1c14`.

Wait, what about `scan_args(&args.paths, global, Some(args.redact))`?
```rust
pub fn scan_args(paths: &[PathBuf], global: &ScanOptions, redact: Option<RedactMode>) -> ScanArgs {
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let excluded_redacted = should_redact && !global.excluded.is_empty();

    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
        excluded_redacted,
```
Wait, if `should_redact` is true, then `args.paths` is mapped via `redact_path` at the end:
```rust
    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }
```
Is this correctly applying redaction to `args.paths`? Yes.

Wait! What about `LangReceipt` and `ModuleReceipt` generated by `write_lang_report_to` and `write_module_report_to`?
```rust
        TableFormat::Json => {
            let receipt = LangReceipt {
                schema_version: tokmd_types::SCHEMA_VERSION,
                generated_at_ms: now_ms(),
                tool: ToolInfo::current(),
                mode: "lang".to_string(),
                status: ScanStatus::Complete,
                warnings: vec![],
                scan: scan_args(&args.paths, global, None),
```
They call `scan_args(&args.paths, global, None)` which does NOT redact the paths or exclusions, even if the user didn't request redaction (there is no `--redact` flag for `lang` or `module`). Should they?
No, the user requested unredacted format, so there shouldn't be redaction.

Wait! What if there IS a bug in `redact_path` or `short_hash`? They are in `tokmd-redact`, which is not in the shard.

Let's look at `RedactMode::Paths`.
If `RedactMode::Paths` is used, `should_redact` is TRUE.
Then `excluded_redacted` is true. `strip_prefix_redacted` is true.
What about `module_roots`? In `RedactMode::Paths`, `module_roots` are NOT redacted.
```rust
fn redact_module_roots(roots: &[String], redact: RedactMode) -> Vec<String> {
    if redact == RedactMode::All {
        roots.iter().map(|r| short_hash(r)).collect()
    } else {
        roots.to_vec()
    }
}
```
Wait... if `redact == RedactMode::Paths`, `module_roots` are NOT redacted, but `excluded` IS redacted?
`global.excluded` contains patterns which might be directories! Why is `global.excluded` redacted under `RedactMode::Paths`?
```rust
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let excluded_redacted = should_redact && !global.excluded.is_empty();
    // ...
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
```
If `RedactMode::Paths` is intended to redact FILE PATHS, but not MODULE PATHS (directories), then shouldn't `excluded` (which often contains directory patterns like `node_modules`) NOT be redacted under `RedactMode::Paths`?
No, `excluded` usually contains both file and directory names. So redacting it under `Paths` might be intentional.
But wait! What about `strip_prefix`?
```rust
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();
```
`strip_prefix` is a directory path! If `redact == RedactMode::Paths`, `strip_prefix` IS redacted! But `module_roots` are NOT redacted! This is a mismatch!
If `RedactMode::Paths` redacts file paths but leaves module/directory names intact, why does it redact `strip_prefix` (which is a directory)?

But wait! `RedactMode` documentation:
In `crates/tokmd-types/src/lib.rs`:
```rust
pub enum RedactMode {
    /// Do not redact.
    None,
    /// Redact file paths but leave module names verbatim.
    Paths,
    /// Redact both file paths and module names.
    All,
}
```
Ah! "Redact file paths but leave module names verbatim."
If `RedactMode::Paths` leaves module names verbatim, what should it do with `strip_prefix`?
`strip_prefix` is a directory prefix. Should it be redacted under `Paths` mode?
Wait! If `strip_prefix` is a prefix applied to FILE PATHS, it might be the prefix `src/secrets`. If we don't redact it, we leak `src/secrets`! But wait, `module_roots` also leaks `src/secrets` if the module root is `src/secrets`.
But `module_roots` is exactly what the user passed as `module` or what was extracted. If `Paths` leaves module names verbatim, it means it leaves `src/secrets` verbatim.
But what about `excluded` patterns? They are currently redacted under `Paths` mode.
And `paths` (the scan input paths) are ALSO redacted under `Paths` mode!
```rust
pub fn scan_args(paths: &[PathBuf], global: &ScanOptions, redact: Option<RedactMode>) -> ScanArgs {
    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
```
Wait! `paths` in `scan_args` are the DIRECTORIES the user passed to scan (e.g. `.` or `src`). If we redact them, we are redacting directory names! But `RedactMode::Paths` says "leave module names verbatim".
Why redact `paths` and `excluded` in `Paths` mode?
Actually, `paths` and `excluded` might be file paths.

Let's look at `scan_args`:
Is `scan_args` redacting correctly?
If there is no code drift or actionable target inside the shard, I should create a learning PR!
> If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix.
> Focus: Keep this lane tightly focused on redaction correctness and leakage prevention.
> If the strongest target you find is outside the shard, record it as friction instead of chasing it.

Wait, I should check if there IS a bug in `tokmd-types` serialization of `excluded_redacted` or `strip_prefix_redacted`.
In `crates/tokmd-types/src/lib.rs`:
```rust
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub excluded_redacted: bool,
```
If it's false, it won't be serialized. That's fine.

What about `scan_args` logic in `crates/tokmd-scan-args`? Wait, `tokmd-scan-args` is NOT in the allowed paths!
```json
  "allowed_paths": [
    "crates/tokmd-types/**",
    "crates/tokmd-scan/**",
    "crates/tokmd-model/**",
    "crates/tokmd-format/**",
    "docs/schema.json",
    "docs/SCHEMA.md",
    "crates/tokmd/tests/**"
  ]
```
So I cannot modify `tokmd-scan-args` anyway!
But I CAN modify `tokmd-format`.
In `crates/tokmd-format/src/lib.rs`, `write_export_json_to` logic:
```rust
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();

        let receipt = ExportReceipt {
            // ...
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
                // ...
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
                strip_prefix_redacted,
```
Wait, if `RedactMode::Paths` is used, should `strip_prefix` be redacted?
`strip_prefix` is a prefix applied to FILE paths. It is technically a directory path.
If `RedactMode::Paths` is "Redact file paths but leave module names verbatim", maybe `strip_prefix` shouldn't be redacted?
But wait, if we don't redact `strip_prefix`, then we leak the path prefix (which could contain sensitive directory names). But module names ALSO leak directory names!
If I shouldn't touch `tokmd-scan-args` and I don't see a clear redaction bug in `tokmd-format` that violates tests... wait!

What if `export.rows` are NOT being correctly redacted in CSV or JSON?
Let's check `export.rows` redaction.
```rust
fn redact_rows(rows: &[FileRow], mode: RedactMode) -> impl Iterator<Item = Cow<'_, FileRow>> {
    rows.iter().map(move |r| {
        if mode == RedactMode::None {
            Cow::Borrowed(r)
        } else {
            let mut owned = r.clone();
            if mode == RedactMode::Paths || mode == RedactMode::All {
                owned.path = redact_path(&owned.path);
            }
            if mode == RedactMode::All {
                owned.module = short_hash(&owned.module);
            }
            Cow::Owned(owned)
        }
    })
}
```
This looks 100% correct.

What about `CycloneDX` export?
```rust
fn write_export_cyclonedx_impl<W: Write>(
    mut out: W,
    export: &ExportReport,
    redact: RedactMode,
    serial_number: Option<&str>,
    timestamp: Option<&str>,
) -> Result<()> {
    // Apply redaction to rows before generating components
    let components: Vec<CycloneDxComponent> = redact_rows(&export.rows, redact)
        .map(|r| CycloneDxComponent::from_file_row(r.as_ref(), redact))
        .collect();
```
And inside `from_file_row`:
```rust
    fn from_file_row(row: &FileRow, redact: RedactMode) -> Self {
        let name = if redact == RedactMode::All {
            short_hash(&row.path)
        } else {
            row.path.clone() // path is already redacted by `redact_rows`!
        };
```
Wait! `row.path` is ALREADY REDACTED by `redact_rows(&export.rows, redact)`!
So `row.path` is ALREADY a hash (e.g. `hash.rs`).
Then `from_file_row` does:
```rust
        let name = if redact == RedactMode::All {
            short_hash(&row.path)
        } else {
            row.path.clone()
        };
```
Wait! If `redact == RedactMode::All`, it calls `short_hash(&row.path)` on the ALREADY REDACTED path!
Let's test this!
Okay, `write_export_cyclonedx_impl` looks totally fine.
```rust
    let components: Vec<CycloneDxComponent> = redact_rows(&export.rows, redact)
        .map(|row| {
...
            CycloneDxComponent {
                ty: "file",
                name: row.path.clone(),
```
It maps over `redact_rows` and copies the ALREADY REDACTED path directly to `name: row.path.clone()`. There is no double hashing!

Let's rethink: Is there a bug in `tokmd-types` or `tokmd-format` where `excluded_redacted` or `strip_prefix_redacted` might not be updated correctly when redaction is applied, OR when `RedactMode::None` is passed?
Let's see `strip_prefix_redacted`.
In `crates/tokmd-types/src/lib.rs`:
```rust
    /// True if `strip_prefix` was redacted (replaced with a hash).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub strip_prefix_redacted: bool,
```
In `crates/tokmd-format/src/lib.rs`:
```rust
        let should_redact = args.redact == RedactMode::Paths || args.redact == RedactMode::All;
        let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();
```
Is `args.strip_prefix` redacted correctly when `strip_prefix` is `Some("src")`?
```rust
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
```
Yes, `redact_path("src")` gives a 16 char string. But wait! `strip_prefix` might be `src/`, `redact_path("src/")` is also handled.
What about `redact_path` bug? "Redaction correctness and leakage prevention."
Wait! If `strip_prefix` is `src/`, and `args.redact == RedactMode::Paths`.
If we redact `strip_prefix`, does it redact it exactly as `args.paths`?
`args.paths` might be `["src"]`.
What about `args.paths` in `lang` or `module`?
If `scan_args(&args.paths, global, None)` is called for `lang` and `module`, it does NOT redact. BUT if the user wants `RedactMode` in `lang` or `module`, they CAN'T. That's a feature missing, not a bug.

Wait! The prompt is: "Optimize for useful, aligned, evidence-backed work per prompt. Land one security-significant hardening improvement. Target ranking 1) redaction correctness and leakage prevention"

Let's check `crates/tokmd-format/tests/format_tests.rs`.
```bash
grep -rn 'strip_prefix_redacted' crates/tokmd-format/tests/
```
In `test_jsonl_strip_prefix_redacted_with_paths_mode`, it expects `strip_prefix_redacted` to be true.
Wait, what if `args.strip_prefix` is None?
`let strip_prefix_redacted = should_redact && args.strip_prefix.is_some();`
This is correct.

Wait... Look at the Sentinel Persona in the prompt:
"Target ranking
1) redaction correctness and leakage prevention
2) FFI parsing / trust boundaries
3) subprocess / environment / path boundary hardening
4) receipt/schema trust and deterministic safety"

Wait, is there any FFI boundary in `tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`?
No, FFI parsing is in `tokmd-core`. But `tokmd-core` is NOT IN THE SHARD.
Is there any subprocess/environment boundary?
Let's check `tokmd-scan` for subprocesses. `tokmd-scan` doesn't use subprocesses.
Let's check `tokmd-types`. It just has types.
Let's check `tokmd-model` for trust boundaries.
Let's check `tokmd-format` for receipt parsing.
Wait... does `tokmd-types` or `tokmd-format` parse receipts?
Yes, `ExportReceipt`, `LangReceipt`, `ModuleReceipt` are parsed when diffing!
Where is diff? It's in `tokmd`. Wait, `crates/tokmd/tests/**` is allowed only if a coherent fix requires them.

Wait! The memory specifically mentioned:
"In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects."
BUT `tokmd_core` is NOT allowed in the shard!
Is it? Let's check the prompt!
`Primary paths: crates/tokmd-types/**, crates/tokmd-scan/**, crates/tokmd-model/**, crates/tokmd-format/**`
`Adjacent paths allowed only if a coherent fix requires them: docs/schema.json, docs/SCHEMA.md, crates/tokmd/tests/**`
If the prompt explicitly says: "If the strongest target you find is outside the shard, record it as friction instead of chasing it."
Then the FFI bug in `tokmd-core::ffi` is outside the shard!
Wait... "record it as friction instead of chasing it."
"If you do not find an honest patch, create a **learning PR** containing:
- the full per-run packet
- one or more friction items under `.jules/friction/open/`
- optional persona note(s) under `.jules/personas/sentinel/notes/`"

This implies that if I can't find a bug IN the shard, I should create a learning PR and record the FFI bug as friction!
Wait, but is there a redaction bug in the shard?
Let's verify.
Let's check if there is a redaction bug in `tokmd-scan` or `tokmd-model`.
In `tokmd-scan`, `normalize_logical_path` strips out `.`, and rejects `..` and absolute paths. This is a security boundary (path boundary hardening!).
Let's look at `normalize_logical_path`:
```rust
fn normalize_logical_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() {
        anyhow::bail!("In-memory path must not be empty");
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            Component::CurDir => {}
            Component::ParentDir => {
                anyhow::bail!(
                    "In-memory path must not contain parent traversal: {}",
                    path.display()
                );
            }
            Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("In-memory path must be relative: {}", path.display());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        anyhow::bail!("In-memory path must resolve to a file: {}", path.display());
    }

    Ok(normalized)
}
```
Is this safe? Yes. It rejects `..`, absolute paths, and empty paths.

What about receipt/schema trust and deterministic safety?
In `tokmd-model`, there's a lot of things. But I don't see any glaring bugs.

Let's check `crates/tokmd-types/src/lib.rs` for `excluded_redacted` bug.
```rust
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub excluded_redacted: bool,
```
Wait, `tokmd-scan-args` sets `excluded_redacted` to `true` if `should_redact && !global.excluded.is_empty()`. That's correct.

Is there any OTHER place where `excluded_redacted` is missed?
Wait! In `tokmd-format`'s `export` report:
```rust
            scan: scan_args(&args.paths, global, Some(args.redact)),
            args: ExportArgsMeta {
```
`scan_args` uses `Some(args.redact)`, which means it REDACTS `global.excluded`. AND it REDACTS `args.paths`.
BUT wait, what about `args.module_roots`?
`args.module_roots` is REDACTED using `redact_module_roots`.
What about `args.strip_prefix`? It is REDACTED using `redact_path`.

Are there any OTHER outputs that should be redacted?
What about `warnings`?
```rust
            warnings: vec![],
```
If there were warnings with file paths, they wouldn't be redacted! But they are empty.

What about `ExportData`?
```rust
            data: ExportData {
                module_roots: module_roots.clone(),
                rows: redact_rows(&export.rows, args.redact).map(|r| r.into_owned()).collect(),
            },
```
It is redacted.

Wait! What about the FFI parser!
Wait, look closely at the FFI parser code in `tokmd-core/src/ffi.rs`.
If I modify `tokmd-core/src/ffi.rs`, it is NOT in the shard! The prompt strictly says:
> "Stay within your assigned shard unless a coherent fix requires touching an adjacent contract surface."
> "If the strongest target you find is outside the shard, record it as friction instead of chasing it."

This strongly implies I MUST NOT fix `tokmd-core/src/ffi.rs` because the prompt literally tells me to record it as friction if the strongest target is outside the shard.
Wait, is there ANY other bug in the shard?

Let's look at `tokmd-format`! Is there any FFI JSON parsing?
No, `tokmd-format` has NO FFI parsing.
Let's look at `crates/tokmd-scan`. Does it parse JSON? No.
Let's look at `crates/tokmd-types`. Does it parse JSON? Yes, `serde_json::from_str` is used in tests.

Let's consider if this is a trick to make me create a Learning PR!
The prompt: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
"If the strongest target you find is outside the shard, record it as friction instead of chasing it."

I have spent 20 minutes verifying there are no glaring redaction bugs in `core-pipeline`.
The only bug is the one explicitly mentioned in the memory: FFI parser in `tokmd-core` does not check `.is_object()`.
Since `tokmd-core` is NOT in the allowed paths, I should create a learning PR.

Let me double check the allowed paths again:
```json
  "allowed_paths": [
    "crates/tokmd-types/**",
    "crates/tokmd-scan/**",
    "crates/tokmd-model/**",
    "crates/tokmd-format/**",
    "docs/schema.json",
    "docs/SCHEMA.md",
    "crates/tokmd/tests/**"
  ]
```
Wait, `tokmd-core` is DEFINITELY not there.

Let me write the friction item!
Option A: Modify `tokmd-core/src/ffi.rs` to enforce `.is_object()`.
- Pros: Fixes the bug mentioned in memory.
- Cons: Violates the strict path constraints (`tokmd-core` is out-of-shard).

Option B: Record the finding as friction and generate a learning PR.
- Pros: Strictly follows the prompt's instructions: "If the strongest target you find is outside the shard, record it as friction instead of chasing it" and "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
- Cons: No code patch is made.

Decision: Choose Option B. The FFI boundary bug is explicitly located in `tokmd-core`, which is out-of-shard. Forcing a change outside the allowed paths violates the assignment. Recording a learning PR matches the exact escape hatch provided for this scenario.
