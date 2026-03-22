# Panic burn down in tests
Context: Unwraps in test files can mask issues or make them brittle.
Pattern: Use `unwrap_or(&0)` instead of `unwrap()` for min/max on test arrays.
Prevention: Disallow unwrap even in tests.
