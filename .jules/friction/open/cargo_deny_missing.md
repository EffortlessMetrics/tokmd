# Friction Item: cargo-deny missing in sandbox

## Description
The fallback gate profile for `governance-release` asks to run `cargo deny --all-features check when manifests change`. However, `cargo-deny` is not installed in the current environment (`error: no such command: deny`).

## Recommendation
Either install `cargo-deny` in the base environment image or adjust the gate profile instructions for the Steward persona to skip it if it's intentionally excluded.
