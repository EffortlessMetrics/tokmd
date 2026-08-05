# Friction: Stale CycloneDX refactor

- **What:** The `.to_string()` allocation removal from the CycloneDX exporter's `CycloneDxProperty` name field was deferred because it was stale work from before the 1.15.0 release and not a release blocker.
- **Why:** The refactoring itself is valid (removing unnecessary string allocations for static keys), but the timing and prioritization meant it was closed out of the active queue.
- **Resolution:** The code changes were abandoned to follow the reviewer's instructions, and this learning was recorded. The branch and PR history can be salvaged in the future if this specific optimization is desired again against current main.
