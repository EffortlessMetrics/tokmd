# Decision

## Option A (recommended)
Update the doctests in `crates/tokmd/src/config/resolve/export.rs` and `crates/tokmd/src/config/resolve/module.rs`.
Currently `tokmd::cli::Profile` is imported in `resolve_module` and `resolve_export`, but it's not exported by `tokmd::cli`.
The actual profile type in the crate scope is `tokmd_settings::Profile`, or we can just import it from `tokmd_settings::Profile`.

- what it is: Update the doctests in the config resolve modules to ensure they compile and provide valid, executable examples of how configuration resolution works.
- why it fits this repo and shard: It fits the `Librarian` persona by improving factual docs quality and executable examples, reducing uncertainty around API usage. It belongs in the `interfaces` shard.
- trade-offs: Structure / Velocity / Governance: Improves documentation structure and testability with negligible cost to velocity or governance.

## Option B
Update the `docs/reference-cli.md` with explicit doctests.
- what it is: Add a rust executable block to the markdown.
- when to choose it instead: If the priority was testing documentation generation or rendering behavior.
- trade-offs: We can't actually `cargo test --doc` a markdown file easily in the standard cargo pipeline without tools like `rustdoc` set up for standalone markdown files, so it's less guaranteed to be continuously verified.

## Decision
Option A. The `Librarian` persona explicitly states: "Prefer doctests and example tests so docs cannot silently drift." and the `docs-executable` gate profile requires doctests to execute or compile. Fixing the broken/failing or silently ignored doctests in the config module is the best alignment with this mandate.
