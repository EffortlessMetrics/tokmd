## Description

Fix drift in the `tokmd-core` README API example. The documentation was still incorrectly referencing the older `scan_workflow` with `GlobalArgs`, `LangArgs`, and `TableFormat`, which have been replaced.

This update modifies the example to properly use `lang_workflow` alongside the correct structures: `ScanSettings` and `LangSettings`. The updated code snippet has been validated using cargo's doctest capabilities to ensure future drift is caught by the test runner.

## Receipts
```
running 6 tests
test crates/tokmd-core/src/../README.md - readme_doctests (line 50) ... ignored
test crates/tokmd-core/src/../README.md - readme_doctests (line 69) ... ignored
test crates/tokmd-core/src/../README.md - readme_doctests (line 20) - compile ... ok
test crates/tokmd-core/src/ffi.rs - ffi::run_json (line 51) ... ok
test crates/tokmd-core/src/lib.rs - (line 24) ... ok
test crates/tokmd-core/src/lib.rs - (line 42) ... ok

test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
```
