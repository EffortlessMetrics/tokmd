# DX Improvement Decision

## Option A: Expose Error Suggestions in Python/Node Bindings (Recommended)
**What it is:** Update `TokmdError` -> `ErrorDetails` serialization in `tokmd-core` to include the `suggestions` field if present, and update `format_error_message` in `tokmd-envelope` to display them. This will make actionable error hints (like those for missing git) visible in Python, Node, and Web targets.
**Why it fits:** The task asks to improve runtime DX across target surfaces. `TokmdError` has a `suggestions` field that isn't included in the JSON envelope, meaning bindings like Node/Python just show `[git_not_available] git is not available on PATH` without the help text.
**Trade-offs:**
- Structure: Touches core error envelope and envelope parsing logic.
- Velocity: Simple serialization and formatting additions.
- Governance: Safe since the `suggestions` field already exists in Rust CLI, just needs exposing.

## Option B: Add custom typed exceptions per error code
**What it is:** Create specific exception classes for Node/Python (e.g., `TokmdPathNotFoundError`) based on the error code.
**When to choose it instead:** If bindings users specifically requested fine-grained try/catch error handling.
**Trade-offs:**
- Massive API surface increase.
- Brittle mappings from string error codes to language-specific exceptions.

## Decision
**Option A**. Exposing `suggestions` directly improves the lowest-context error messages (like `git_not_available` or `path_not_found`) in bindings without changing the API contract.
