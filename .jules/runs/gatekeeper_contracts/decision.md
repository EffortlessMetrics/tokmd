# Decision

## Option A (recommended)
- Update the `docs/reference-cli.md` explicitly with the missing `--profile` flag and verify using `cargo xtask docs --check`.
- Why it fits: Aligns `reference-cli.md` with the Rust-source parser definition without requiring any larger code-level refactors. It explicitly solves factual and output-contracts drift between source and docs schema.
- Trade-offs:
  - Velocity: High. It's a quick fix that leverages existing toolchains (`xtask docs --check`).
  - Governance: High. Reduces mismatch between `tokmd`'s runtime help output and reference markdown.
  - Structure: Low risk.

## Option B
- Modify `xtask docs --check` to automatically parse Rust-level struct fields and enforce mapping for all global arguments dynamically rather than static assertions or text replacements.
- Why it fits: Automates further preventing manual documentation errors in markdown files altogether.
- Trade-offs:
  - Velocity: Slow. Parsing Rust structs using AST analysis or regex requires more complex logic.
  - Governance: Could make build slower or more complex, bringing unwanted logic to `xtask`.

## Decision
Chose Option A because it’s minimal, correct, immediately executable, strictly adheres to the one-prompt-per-story principle, and fully satisfies the `Gatekeeper` profile constraint.
