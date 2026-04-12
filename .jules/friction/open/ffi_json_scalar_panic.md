# FFI JSON Scalar Panic

The `tokmd-core::ffi` entrypoint (`run_json_inner`) parses FFI JSON arguments using `serde_json::from_str`. It blindly treats the resulting `Value` as an object without explicitly verifying `.is_object()`. While scalar inputs like `"0"` do not immediately panic in `run_json_inner` because `.get()` on a scalar `Value` safely returns `None`, the memory notes indicate this pattern violates a strict security boundary and can cause downstream panics.

However, `tokmd-core` is outside the `core-pipeline` shard's allowed paths (`tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`). Following instructions, this out-of-shard target is recorded as friction rather than forcing a fake fix or violating shard boundaries.
