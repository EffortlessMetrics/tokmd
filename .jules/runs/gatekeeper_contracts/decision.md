# Option A: Fix bindings-parity --check returning an error when run without args

While `cargo xtask bindings-parity --check` does not return an error, running `cargo xtask bindings-parity` returns:
`Error: bindings-parity requires --check (update mode is not implemented)`

Since update mode is not implemented, maybe we should just make it default to `--check` mode, or implement update mode.

# Option B: Fix file-policy findings by adding them to non-rust-allowlist.toml

The file-policy tool finds 18 non-rust files that are not allowed by the allowlist:

```
file-policy findings (18):
  - file fixtures/bindings-parity/golden/invalid_json.json does not match any non-Rust allowlist glob
  - file fixtures/bindings-parity/golden/unknown_mode.json does not match any non-Rust allowlist glob
...
```

This violates the determinism and policy enforcement of `cargo xtask check-file-policy --strict`. We can fix this by appending missing paths to `policy/non-rust-allowlist.toml`.

# Decision: Option B

Option B aligns perfectly with the Gatekeeper persona's mission to "Protect contract-bearing surfaces and lock in deterministic behavior", specifically targeting "policy/gate semantic drift" by fixing the file-policy gate. Option A is an ergonomics fix, not a gatekeeper/contracts issue.
