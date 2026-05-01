# Decision

## Option A (Extract duplicate test sorting functions to the core API)
Extract `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows` into public standalone functions in `tokmd-model`.
Currently, the `determinism_w66.rs` test suite defines its own versions of these sorting functions, which means it tests its own sorting logic rather than the *actual* inline sorting closures used in `build_model` and `create_export_data_from_rows`.
By centralizing these functions, we ensure that a mutation in the actual library sorting logic is properly caught by the test suite, effectively closing a massive mutation gap.

## Option B (Add e2e determinism tests via CLI)
We could leave the inline sorting closures as they are and instead add a full end-to-end test that calls the `build_model` function and compares the full output with differently ordered inputs.
This works but is a heavier and slower test. It also doesn't fix the fact that `determinism_w66.rs` has dummy sorting functions that could give a false sense of security.

## Decision
Option A. It's cleaner, removes duplicated test logic that causes false mutation test coverage, and fulfills the Mutant persona's goal of improving tests around a high-value production surface (deterministic sorting).
