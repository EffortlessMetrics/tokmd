# Decision

## Option A
Add doctests to the `tokmd-settings` crate's key public structures (`TomlConfig`, `ScanConfig`, `UserConfig`).
This fits the `Librarian` persona and `interfaces` shard by providing executable examples of how configuration is parsed and structured, ensuring the configuration schema remains stable and aligned with the actual implementation.

## Option B
Add comprehensive mock testing scripts in the `tests/` directory of `tokmd`.
While useful, this focuses more on integration testing and less on immediately visible, executable documentation on the public APIs themselves, which is the Librarian's primary goal.

## Decision
Proceeding with **Option A**. Adding doctests directly to the configuration structs in `tokmd-settings` provides high-signal proof of parsing behavior that acts as both documentation and tests.
