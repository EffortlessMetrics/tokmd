# Decision

## Option A (rejected)
Fix documentation drift caused by `/// Examples:` instead of `/// # Examples` in `tokmd/src/cli/parser.rs`.
While this is a standard rustdoc convention that prevents `cargo xtask docs --check` drift, the CLI documentation renders the literal '#' into generated CLI help output, making it user-hostile. Therefore this cannot be shipped.

## Option B (rejected)
Attempt to use `cargo fuzz`. `cargo-fuzz` fails due to ASAN / LLVM toolchain issues natively in this execution environment without a heavy fix payload.

## Decision
Create a learning PR documenting the friction. Both the `cargo-fuzz` missing environment capability and the clap doc headers rendering `#` in the CLI output represent friction that prevents an honest code patch. Therefore, we will fallback to a learning PR, satisfying the run directives.
