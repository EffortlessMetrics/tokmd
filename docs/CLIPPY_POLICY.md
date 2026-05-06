# Clippy policy

`tokmd` uses the Effortless Metrics Rust lint policy as a governed engineering
surface, not as an informal collection of local preferences. The policy is
intended to make the workspace panic-free, suppression-governed, and safe around
text, paths, file handles, async work, and numeric edge cases.

## Source of truth

The root `Cargo.toml` contains the active Rust workspace lint block under
`[workspace.lints.rust]`. The strict Clippy profile is tracked as staged policy
debt in `policy/clippy-lints.toml` so follow-up PRs can promote it without
surprise breakage. Every workspace package
must inherit that block with:

```toml
[lints]
workspace = true
```

`policy/clippy-lints.toml` is the machine-readable ledger that mirrors the
active block, records the staged strict Clippy profile, and tracks planned
Rust/Clippy flips before the MSRV bump. The
ledger records each lint's level, status, class, and reason so policy changes can
be reviewed as explicit governance changes.

## Panic-free workspace posture

The policy bans panic-family and unchecked-collapse shapes in production and
tests:

- `panic!`, `todo!`, `unimplemented!`, `unreachable!`, and `dbg!`
- unchecked `unwrap`/`expect` result or option collapse
- unchecked indexing and string slicing
- panic-prone time subtraction

There are no test carveouts. Tests should return `Result` and use fallible setup
or assertion helpers rather than panic-driven fixture setup.

```rust
#[test]
fn parses_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = std::fs::read_to_string("tests/fixtures/input.rs")?;
    let parsed = parse(&fixture)?;

    ensure_eq(parsed.items.len(), 3, "fixture should expose three items")?;

    Ok(())
}
```

## Suppression style

Do not use broad or silent `#[allow(...)]` suppressions for policy lints. If a
local exception is unavoidable, use a narrow `#[expect(..., reason = "...")]`
attribute and explain why the exception is safe and temporary. Longer-lived or
repo-wide exceptions belong in `policy/clippy-debt.toml` with an owner, path,
lint, reason, and expiry.

## Policy files

- `policy/clippy-lints.toml` — active lint ledger plus planned Rust 1.94 and
  Rust 1.95 flips.
- `policy/clippy-debt.toml` — temporary lint-policy exceptions. Expired debt
  fails the policy check.
- `policy/no-panic-allowlist.toml` — semantic panic-family exception schema for
  identity by path, family, and selector. `last_seen` locations are advisory.
- `policy/non-rust-allowlist.toml` — structured file-policy exceptions for
  non-Rust implementation, fixture, config, schema, and asset surfaces.
- `clippy.toml` — repo-local Clippy configuration only. It must not contain test
  carveouts such as `allow-unwrap-in-tests = true`.

## Check command

Run the policy check with:

```bash
cargo xtask check-lint-policy
```

The check verifies that the workspace MSRV matches the policy ledger, every
workspace package inherits workspace lints, active lints match `Cargo.toml`,
planned Rust 1.94/1.95 lints are not accidentally activated early, test carveouts
are absent from `clippy.toml`, and debt entries are complete and unexpired.
