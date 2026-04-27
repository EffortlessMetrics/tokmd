# Friction Item: Risk Preset Panic

**Surface**: `tokmd analyze --preset risk`
**Persona**: Librarian

## Observation
When running `cargo run -- analyze --preset risk --format md` within the repository root, the command immediately panics due to a character boundary error.

## Error Trace
```
thread 'main' (15888) panicked at crates/tokmd-analysis/src/halstead/mod.rs:388:47:
end byte index 4 is not a char boundary; it is inside '日' (bytes 2..5) of `("日本`
stack backtrace:
   ...
   6: tokmd_analysis::halstead::tokenize_for_halstead
             at ./crates/tokmd-analysis/src/halstead/mod.rs:388:47
   7: tokmd_analysis::halstead::build_halstead_report
             at ./crates/tokmd-analysis/src/halstead/mod.rs:501:22
```

## Impact
This prevents running the documented `risk` preset analysis, breaking multiple recipes and workflows detailed in the `docs/` directory.
