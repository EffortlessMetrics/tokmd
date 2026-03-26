# Descriptive unwraps in tests

## Context
When maintaining tests, test failures due to `.unwrap()` calls provide little context to exactly what invariant or assertion failed, often just showing a `called Result::unwrap() on an Err value` trace.

## Pattern
Avoid raw `.unwrap()` in test suites, including test helper setups. Instead, use `.expect("...")` with descriptive text detailing what operation was taking place and what result was expected, so developers can quickly identify the source of the issue.

## Prevention Guidance
- If a test expects to safely unwrap a JSON string, avoid `.unwrap()`. Prefer `serde_json::from_str(&data).expect("parse json");`.
- When using bindings where item extraction from dictionaries is common (like PyO3), provide explicit extraction paths: `dict.get_item("key").expect("get key")`.

## Links
- [tokmd-node test suites](crates/tokmd-node/src/lib.rs)
- [tokmd-python test suites](crates/tokmd-python/src/lib.rs)
