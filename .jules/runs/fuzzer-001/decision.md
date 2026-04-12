# Investigation

The mission is to improve fuzzability or input hardening around parser/input surfaces.
In `crates/tokmd-core/src/ffi.rs`, `run_json_inner` was parsing JSON args and trusting it to be an object:

```rust
    let args: Value = serde_json::from_str(args_json)?;
    let inputs = parse_in_memory_inputs(&args)?;
```

If a scalar like `"123"` was passed, `serde_json::from_str` correctly parses it into a `Value::Number` (or `Value::String`, etc.), but subsequent code expects an object when querying fields (e.g. `args.get("inputs")`), which could lead to unexpected `None` values falling back to defaults silently, or in the case of `parse_in_memory_inputs`:

```rust
    let root_inputs = args.get("inputs").filter(|value| !value.is_null());
```

If `args` is not an object, `.get("inputs")` returns `None`. It was falling back to scanning the current directory, which is a logic flaw because it ignores the input. In the worst case, scalar inputs could cause downstream panics. The memory explicitly mentions:

> In `tokmd_core::ffi`, FFI boundaries that parse JSON strings using `serde_json::from_str` must explicitly verify the resulting `Value` is an object (e.g., using `.is_object()`) before proceeding. Scalar JSON inputs (like `"0"`) parse successfully but can cause downstream panics if blindly treated as objects.

# Options considered

## Option A (recommended)
Add an explicit check to verify that `args` is a JSON object after parsing.
- What it is: Add `if !args.is_object() { return Err(TokmdError::invalid_field("args", "a JSON object")); }` in `run_json_inner`.
- Why it fits: Hardens the input surface against scalar inputs correctly, as specified by the memory constraint. Also adds a deterministic test for it.
- Trade-offs: Structure is improved (safer input boundary), Velocity is not affected (small change), Governance is improved (better security/fuzzing properties).

## Option B
Change `serde_json::from_str` to parse directly into a strictly defined struct.
- What it is: Define a serde struct that matches the expected input and use it instead of `Value`.
- Why it fits: Stronger typing.
- Trade-offs: Requires a large refactoring of the FFI layer which might introduce bugs and break backward compatibility.

# Decision
Option A. It's safe, adheres perfectly to the memory and the instructions, and fixes the vulnerability minimally while establishing deterministic tests.
