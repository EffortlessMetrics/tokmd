# Determinism Unwraps in tests

There are raw `unwrap()` calls on line ~243-246, 263-266, 283-286 of `determinism_hardening.rs` that should be `expect()` calls to clearly state what's failing if the CLI determinism outputs are malformed.

Tags: test, quality, determinism
