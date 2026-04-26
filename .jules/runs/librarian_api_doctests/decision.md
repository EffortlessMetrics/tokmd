## Target Assessment

The primary targets identified for improvement are the doctests within `crates/tokmd-core/src/lib.rs` and `crates/tokmd/src/config.rs`.

1.  `cockpit_workflow` in `crates/tokmd-core/src/lib.rs` (lines 537-550): The doctest is marked with `no_run`. This violates the gate profile expectation (`docs-executable`) where doctests and examples should execute or compile where possible to prevent silent drift.
2.  `analysis_facade` in `crates/tokmd-core/src/lib.rs` (lines 612-624): Similar to `cockpit_workflow`, the doctest is marked with `no_run`.
3.  `resolve_export` and `resolve_export_with_config` in `crates/tokmd/src/config.rs`: The doctests initialize `CliExportArgs` but omit many fields recently added to the struct, causing warnings or errors if checked strictly, or simply lacking coverage of the full CLI arguments structure. However, in `config.rs`, these examples currently compile because they omit fields and Rust allows partial initialization with `..Default::default()` *if* the struct implements `Default`, BUT as per memory:
    "In the `tokmd-config` crate, CLI argument struct types like `CliModuleArgs` and `CliExportArgs` do not derive `Default`. When initializing them in tests, you must explicitly provide all fields (usually as `None` or `false`) instead of using `..Default::default()`."
    Looking at the current source code, the doctest just specifies fields individually up to `strip_prefix`. Let's check `CliExportArgs` definition. It might have new fields like `split` or `merge`.

Let's inspect the `CliExportArgs` struct to see what fields are missing in the doctest.

Option A (recommended)
Update the `no_run` doctests in `tokmd-core` to be fully executable, removing the `no_run` attribute by using `#[cfg(feature = "cockpit")]` or similar conditional compilation around the doctest if needed, or just mocking the required arguments correctly. Also update the `tokmd/src/config.rs` doctests to ensure all fields of `CliExportArgs`, `CliLangArgs`, `CliModuleArgs` are correctly and fully specified.

Option B
Write a learning PR indicating that `no_run` is acceptable for features that require heavy Git context.

Decision
We will proceed with Option A. `no_run` should be avoided for public APIs if we can make them compile/run. We can change them to compile-only or fully executable tests by setting up minimal valid state. However, making them fully executable might be tricky if they require an active Git repository. In that case, we can keep them compile-only (by using `no_run` or by wrapping in a main function that isn't called, or ignoring the execution). Wait, removing `no_run` and replacing it with something that just verifies compilation is better than nothing, but let's see if we can make it run.
