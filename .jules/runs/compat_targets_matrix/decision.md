# Decision

## Option A (recommended)
- Replace `localeCompare` in `web/runner/ingest.js` with direct Unicode code-unit comparisons (`<` and `>`).
- This fixes determinism drift across environments/platforms since `localeCompare` behaves differently depending on the runtime language settings, whereas strict `<`/`>` evaluates string values lexicographically matching Rust's internal `BTreeMap` and `String::cmp` behavior.
- Trade-offs: Strict Unicode comparison is less culturally aware (e.g., ignoring natural language collation rules) but we value absolute determinism over cultural sorting since the primary consumer of these paths is the machine-based runner pipeline targeting hashes and caching.

## Option B
- Pass an explicit locale configuration such as `en-US` to `localeCompare`.
- While passing `en-US` and `{ sensitivity: 'base' }` fixes variations across system locales, differing JS engine implementations might still apply nuanced rules leading to subtle edge-case determinism breaks.

## Decision
Option A. `<`/`>` is completely deterministic and guarantees identical order to the Rust WASM/Node bindings' map traversals.
