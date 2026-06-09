# Decision

## Option A (recommended)
Update the common FFI envelope parser in `tokmd-envelope` to extract the existing `suggestions` field from the error object and append them to the formatted error string.

**Why it fits:**
The `suggestions` field is already populated by `TokmdError` in the backend, but previously it wasn't being shown to consumers of the FFI API (Node, Python, Wasm, browser-runner). Modifying the central `tokmd_envelope::ffi::format_error_message` immediately improves error ergonomics for all binding targets in one fell swoop, perfectly fitting the "Improve runtime developer experience in one coherent way" directive.

**Trade-offs:**
- *Structure:* Centralized fix in a core type. No need to update 4 different language bindings individually.
- *Velocity:* High velocity, straightforward logic change.
- *Governance:* Aligns with existing deterministic tests.

## Option B
Update the individual binding crates (`tokmd-python`, `tokmd-node`, `tokmd-wasm`) to parse the JSON and extract `suggestions` manually when returning native errors.

**Why it fits:**
It allows each language to format the suggestion in a way that matches its specific error idioms (e.g. attaching to a custom property instead of the message string).

**When to choose it instead:**
If the bindings exported complex error objects where `suggestions` could be programmatically accessed.

**Trade-offs:**
Requires changing error handling logic in multiple crates and languages, introducing inconsistency and higher maintenance burden. The current bindings all rely on extracting a single string message.

## Decision
**Option A**. It's the most robust, coherent, and centralized way to ensure all FFI bindings automatically benefit from the rich `suggestions` already provided by the Rust core.
