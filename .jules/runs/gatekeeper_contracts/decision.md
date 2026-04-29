# Option A: Replace manual parameter tables with `<!-- HELP: <cmd> -->` markers

The `docs/reference-cli.md` contains manual documentation of CLI parameters in tables (for `tokmd module`, `tokmd export`, `tokmd run`, etc.). This violates the principle mentioned in memory: "In `tokmd` documentation, `docs/reference-cli.md` relies on `<!-- HELP: <cmd> -->` and `<!-- /HELP: <cmd> -->` markers to automatically inject CLI help output via `cargo xtask docs --update`. Avoid writing manual parameter tables for CLI commands, as they will drift from the actual clap parser."

We need to add the missing markers and remove the manual tables for commands that don't have them yet.

# Option B: Add more missing rules to `.tokeignore` template

Not directly addressing the primary issue of drift and manual doc generation that goes against the established xtask pattern in the repo.

# Decision: Option A

The memory clearly states: "In `tokmd` documentation, `docs/reference-cli.md` relies on `<!-- HELP: <cmd> -->` and `<!-- /HELP: <cmd> -->` markers to automatically inject CLI help output via `cargo xtask docs --update`. Avoid writing manual parameter tables for CLI commands, as they will drift from the actual clap parser."

Currently `docs/reference-cli.md` has manual tables for `tokmd module`, `tokmd export`, `tokmd run`, `tokmd analyze`, `tokmd baseline`, `tokmd badge`, `tokmd diff`, `tokmd init`, `tokmd context`, `tokmd handoff`, `tokmd check-ignore`, `tokmd tools`, `tokmd completions`.

The `xtask/src/tasks/docs.rs` already supports these markers:
```rust
    let markers = [
        ("lang", "lang"), // Explicitly use lang subcommand help
        ("module", "module"),
        ("export", "export"),
        ("run", "run"),
        ("analyze", "analyze"),
        ("baseline", "baseline"),
        ("badge", "badge"),
        ("diff", "diff"),
        ("init", "init"),
        ("context", "context"),
        ("handoff", "handoff"),
        ("check-ignore", "check-ignore"),
        ("tools", "tools"),
        ("cockpit", "cockpit"),
        ("sensor", "sensor"),
        ("gate", "gate"),
        ("completions", "completions"),
    ];
```
But the markdown file is missing the markers for most of them.
