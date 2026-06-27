## Note on FFI Boundaries
The FFI logic extensively uses `.unwrap_or(args)` for nested configuration structs, such as `scan`, `lang`, `module`, etc. This allows callers to pass strings instead of dictionaries and silently fallback to defaults or parent scope values. A future Fuzzer run should harden these boundaries by explicitly validating `is_object()`.
