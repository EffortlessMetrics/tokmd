# Fail Fast on Configuration Errors

**Context**: Users expect configuration files to be applied if they exist.
**Problem**: Silent failure (ignoring invalid config) leads to confusion and "why is my setting ignored?" debugging.
**Pattern**: When loading configuration files, always return `Result` and propagate parsing errors immediately. Do not use `ok()` or `unwrap_or_default()` on critical user configuration paths.
**Evidence**: `tokmd` previously ignored invalid `tokmd.toml` files, causing user frustration. Fixed in PR-2025-02-19-001.
