# Tests unwrap() burndown

Context: Replaced multiple `expect()` and `unwrap()` usages in `crates/tokmd-core/src/ffi.rs` tests.

Pattern:
- Convert `fn my_test() { ... }` into `fn my_test() -> Result<(), Box<dyn std::error::Error>> { ... Ok(()) }`.
- Replace `.unwrap()` and `.expect("...")` with `?`.
- For Option conversions like `.as_str()`, use `.as_str().ok_or("err")?`.

Evidence:
- `crates/tokmd-core/src/ffi.rs`

Prevention guidance:
- Prefer returning `Result` and utilizing `?` in tests over asserting via panicking unwrap/expect to keep test failures clean and adhere to the strict no-panic stance across all repository code, including tests.
