# Decision

## Problem
In `tokmd-core/src/ffi/settings_parse.rs`, JSON configuration blocks for modes like `lang`, `module`, `export`, etc. were retrieved using `args.get("field").unwrap_or(args)`. This fallback logic allowed a non-object value (like a string or array) passed in the `field` key to bypass validation and cause a panic when trying to parse the actual parameters, or quietly fallback to parsing the root object when it shouldn't.

## Options Considered

### Option A: Use strict object validation
Write a `get_config_block` helper that explicitly checks if the block exists, and if it does, ensures it is an object (`v.is_object()`). If not an object, it returns a validation error instead of panicking or falling back to the root `args`.

- Fits this repo and shard because the `interfaces` shard has a strong emphasis on trust boundaries, FFI correctness, and avoiding panics.
- Trade-offs: Increases code slightly but dramatically improves robustness.

### Option B: Panic safely
Instead of returning a validation error, unwrap an error or let `serde_json` fail earlier.

- Trade-offs: The FFI should never panic. A soft validation error is the correct approach.

## Decision
Option A was chosen. I implemented a `get_config_block` helper in `parse.rs` and replaced all the `args.get(field).unwrap_or(args)` instances in `settings_parse.rs` with `get_config_block(args, field)?`. This fixes the non-object validation bypass at the FFI trust boundary.
