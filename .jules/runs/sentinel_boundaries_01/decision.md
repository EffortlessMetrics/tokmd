# Options considered

## Option A (recommended)
Fix FFI parsing logic in `crates/tokmd-core/src/ffi/settings_parse.rs` and `crates/tokmd-core/src/ffi/parse.rs`. The code currently uses `args.get("<key>").unwrap_or(args)` which is an anti-pattern. If `<key>` is present but is not a JSON object (e.g. it is a string or array), it will silently bypass validation for non-object values or fail in unexpected ways down the line. We should instead verify that when `<key>` is provided, it is indeed an object. If it is provided but is not an object, we should return a `TokmdError`.

- **Structure**: Strongly aligns with the Sentinel persona by hardening trust boundaries (FFI parsing).
- **Velocity**: A small, focused change that prevents silent failures and potential security/validation bypasses.
- **Governance**: Protects the core FFI surface from malformed inputs.

## Option B
Wait for an overarching FFI rewrite or assume that the calling code (Python/JS bindings) will always provide perfectly formed JSON objects.

- **When to choose it**: If we trust all callers unconditionally.
- **Trade-offs**: Violates the "security-boundary" gate profile which demands strict validation at trust boundaries.

# Decision
Option A. It directly addresses a known vulnerability/anti-pattern in the FFI parsing surface (identified in the memory bank as: "In tokmd-core FFI bindings, nested JSON configuration blocks (e.g., 'scan', 'lang') must be strictly validated as JSON objects. Using `serde_json::Value::get().unwrap_or()` is an anti-pattern that silently bypasses validation for non-object values like strings or arrays. Always enforce explicit type checking (e.g., `is_object()`) at trust boundaries."). We will update `parse.rs` and `settings_parse.rs` to enforce this.
