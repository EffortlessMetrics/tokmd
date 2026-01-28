# Repo facts (tokmd)

This file exists so scheduled prompts can stay short.

Keep it factual. Keep it current.

## Language and toolchain
- Rust
- Build tool: Cargo

## Merge-confidence gates (default)
- `cargo build --verbose`
- `CI=true cargo test --verbose`
- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`

## Docs gates (when relevant)
- `cargo test --doc --verbose`
- `cargo test --examples --verbose` (if examples exist)

## Feature matrix (when relevant)
- `cargo build --no-default-features --verbose`
- `cargo build --all-features --verbose`

## Output stability expectations
- Prefer deterministic output.
- If output format changes, update docs and tests together.

## Code quality direction
- Unwrap/expect/panic burn-down is an explicit goal.
- Prefer structured errors with context.
