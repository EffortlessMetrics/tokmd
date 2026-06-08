# Option A (recommended)
Add a new continuous fuzzing target `fuzz_cli_parser` that directly feeds arbitrary strings into `tokmd::cli::Cli` via `clap`.
- **Why it fits:** The prompt explicitly asks to "Improve fuzzability or input hardening around parser/input surfaces." We already have a property-based test (`cli_parser_properties.rs`), but adding a true fuzz target unlocks libfuzzer coverage over the CLI parser, hardening it against panics on malformed input (like long strings, invalid unicode, or strange argument combinations).
- **Trade-offs:**
  - *Structure:* Minimal addition, sits naturally alongside other fuzz targets.
  - *Velocity:* High, straight-forward to implement using existing `tokmd` dependencies.
  - *Governance:* Aligns well with the fuzz-gate profile.

# Option B
Expand the existing `cli_parser_properties.rs` proptests to cover more edge cases, like empty arguments or massive vectors of string parts.
- **Why it fits:** Still targets input hardening of the parser surface.
- **Trade-offs:**
  - *Structure:* Lower signal compared to actual continuous fuzzing with `cargo-fuzz`.
  - *Velocity:* Takes more thought to design good proptest generators.
  - *Governance:* Does not fully leverage the libfuzzer engine we have set up in `fuzz_targets/`.

# Decision
Option A. It adds a direct fuzz target that works within the libfuzzer ecosystem established in `fuzz/`, providing better long-term security/robustness than a purely random proptest could.
