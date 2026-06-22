# Decision

## Option A (recommended)
Add `after_help` CLI examples to the subcommands specified in the ROADMAP (`analyze`, `diff`, `context`, `gate`, `cockpit`, `handoff`, `run`, and `export`) via clap attributes.

- What it is: Leveraging `clap`'s `after_help` to append concrete usage examples to the `tokmd <cmd> --help` output.
- Why it fits this repo and shard: The `ROADMAP.md` explicitly lists adding CLI help examples for these commands as a key documentation goal. Updating the rust parser struct attributes inherently fulfills this, and ensuring docs stay synchronized fits perfectly within the `tooling-governance` shard.
- Trade-offs: Structure / Velocity / Governance: Provides an immediate improvement to developer experience via terminal help while keeping `clap` structs as the single source of truth for docs generation.

## Option B
Update `reference-cli.md` manually to include examples.

- What it is: Hand-authoring examples directly into the markdown files.
- When to choose it instead: If the CLI tool didn't generate documentation from its source code structure.
- Trade-offs: This violates the repository's practice of generating CLI reference docs from `clap` (as seen by `cargo xtask docs --check`), making it a high risk for drift.

## Decision
Option A. It natively supports the rust-first approach, adheres to `tokmd`'s auto-generated docs approach, explicitly fulfills a ROADMAP goal, and guarantees sync with `cargo xtask docs --update`.
