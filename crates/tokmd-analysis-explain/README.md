# tokmd-analysis-explain

Single-responsibility microcrate for analysis metric/finding explanation lookup.

## What it does

- Normalizes user-provided metric keys and aliases.
- Resolves a key to a concise explanation string.
- Emits a deterministic catalog of available explanation keys.

## API

- `lookup(key: &str) -> Option<String>`
- `catalog() -> String`
