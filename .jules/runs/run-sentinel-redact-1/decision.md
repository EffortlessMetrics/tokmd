# Sentinel Redaction Option Analysis

## Option A (recommended): Ensure `safe_path_extension` matches case-insensitively and converts output to lowercase
- **What it is**: The current `safe_path_extension` implementation does `let lower = ext.to_ascii_lowercase();` to check against the binary searched sorted array, but it returns the *original case* `ext` instead of the lowercased extension. If a user has `file.JSON`, it currently outputs a redacted path ending in `.JSON`, leaking the case information from the original file path. The fix involves converting the extracted extension to lowercase when adding it to the redacted path string.
- **Why it fits this repo and shard**: Path redaction falls under the `tokmd-format` crate and is explicitly the responsibility of `core-pipeline`. Preventing leakage (even of case information) is a core hardening requirement under the `security-boundary` gate profile.
- **Trade-offs**:
  - **Structure**: Improves the redaction interface correctness by ensuring output suffixes are deterministic regardless of input case.
  - **Velocity**: Minor change, easy to verify with existing tests.
  - **Governance**: Fixes an unaddressed minor trust-boundary leak.

## Option B: Do nothing and record a learning PR
- **What it is**: If the case leak is deemed negligible or within acceptable threat bounds.
- **When to choose it instead**: If fixing this breaks some integration that relies on original-case extensions being preserved through redaction, or if a stronger target exists.
- **Trade-offs**: Leaves a minor info-leak in place.

## ✅ Decision
We choose **Option A** as it aligns cleanly with the prompt's request for "redaction correctness and leakage prevention" and "trust-boundary hardening". The original intent was likely to output the lowercase extension, as checking it was done lowercased but it failed to map the output to lowercase. We will implement the case-insensitivity fix and write tests to ensure the case is stripped from redacted extensions.
