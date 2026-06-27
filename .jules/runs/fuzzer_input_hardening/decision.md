# Option A (recommended)
Add strict `is_object()` validation to all settings parser surfaces (`scan`, `lang`, `module`, `export`, `analyze`, `cockpit`, `diff`) in `tokmd-core/src/ffi/settings_parse.rs` and `parse.rs` when accessing nested configuration fields. Include a regression test in `tokmd-core/tests/regression_settings_fuzz.rs` to lock this behavior in.

- **Structure**: Strongly types the trust boundary against unexpected JSON inputs (e.g., passing a string or array where an object is expected for a settings block).
- **Velocity**: Straightforward addition of `is_object()` checks and a targeted integration test.
- **Governance**: Protects the core library's FFI interface from panics or silent misinterpretation of malformed configurations, directly aligning with the `fuzzer` persona's mandate for input hardening.

# Option B
Only fix the `scan` settings parser, as it's the most foundational block, and leave the mode-specific ones alone.

- **When to choose**: If the mode-specific settings blocks are somehow validated externally before reaching the core, or if the risk is deemed too low for individual modes.
- **Trade-offs**: Leaves a gap in the FFI surface where other modes (`lang`, `export`, etc.) could still misbehave or silently ignore malformed non-object inputs by falling back to defaults via `unwrap_or(args)` instead of returning a strict error.

## Decision
Option A. The anti-pattern of using `args.get(field).unwrap_or(args)` allows non-object inputs (like strings or arrays) to silently bypass validation or cause unexpected behavior when nested fields are subsequently queried. Enforcing `is_object()` explicitly hardens the trust boundary as requested by the fuzzer persona guidelines, preventing deterministic bugs or fuzzing failures.
