# FFI Boundary Checks

When hardening FFI boundaries that accept JSON strings, always ensure that parsed `serde_json::Value` instances are explicitly validated against their expected shapes (e.g., using `.is_object()`, `.is_array()`) before delegating to downstream workflows. While `.get()` safely returns `None` on scalar variants, failing to validate the top-level structure violates strict boundary constraints and can lead to unexpected workflow defaults or panics in less defensive downstream consumers.
