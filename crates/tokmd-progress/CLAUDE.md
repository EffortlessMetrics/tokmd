# tokmd-progress

## Purpose

Progress spinner and progress-bar abstractions. This is a **Tier 2** utility crate.

## Responsibility

- Provide `Progress` spinner for indeterminate operations
- Provide `ProgressBarWithEta` for determinate operations with ETA
- No-op stubs when the `ui` feature is disabled
- TTY detection and `NO_COLOR` / `TOKMD_NO_PROGRESS` environment variable respect
- **NOT** for business logic or CLI parsing

## Public API

```rust
/// Indeterminate spinner (writes to stderr).
pub struct Progress;

impl Progress {
    pub fn new(enabled: bool) -> Self;
    pub fn set_message(&self, msg: impl Into<String>);
    pub fn finish_and_clear(&self);
}

/// Determinate progress bar with ETA.
pub struct ProgressBarWithEta;

impl ProgressBarWithEta {
    pub fn new(enabled: bool, total: u64, message: &str) -> Self;
    pub fn inc(&self);
    pub fn inc_by(&self, delta: u64);
    pub fn set_position(&self, pos: u64);
    pub fn set_message(&self, msg: &str);
    pub fn set_length(&self, len: u64);
    pub fn finish_with_message(&self, msg: &str);
    pub fn finish_and_clear(&self);
}
```

## Implementation Details

### Feature Gate

- `ui` feature enables `indicatif` dependency and real progress output
- Without `ui`, all methods are no-ops (zero overhead)
- Both structs implement `Drop` to clear the spinner/bar on panic or early return

### TTY / Environment Detection

The spinner is shown only when all conditions are met:
1. `enabled` parameter is `true`
2. `stderr` is a TTY (`IsTerminal`)
3. `NO_COLOR` env var is **not** set
4. `TOKMD_NO_PROGRESS` env var is **not** set

## Dependencies

- `indicatif` (optional, behind `ui` feature)

## Testing

```bash
cargo test -p tokmd-progress
cargo test -p tokmd-progress --features ui
```

Tests verify that all methods are callable without panicking when disabled.

## Do NOT

- Add business logic (belongs in command handlers)
- Make `indicatif` a required dependency (must stay behind `ui` feature)
- Write to stdout (all progress output goes to stderr)
