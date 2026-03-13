## Description

Fixes the `--no-default-features` build for `tokmd-cockpit` by properly gating git-specific imports, functions, structs, and tests behind `#[cfg(feature = "git")]`. This resolves dead code and unused import warnings that were causing compilation failures under the project's strict `-D warnings` setup.

### Strategy / Constraints
* **SRP**: Only fixed `--no-default-features` compilation issues in `tokmd-cockpit`.
* **Verification**: Ran `cargo test`, `cargo clippy`, and `cargo check` for both `default`, `no-default-features` and `all-features`.
* **State tracking**: State changes accurately recorded in `.jules/compat`.

### Receipts

```json
[
  {
    "cmd": "cargo check --no-default-features -p tokmd-cockpit",
    "exit_code": 0
  },
  {
    "cmd": "cargo check --all-features -p tokmd-cockpit",
    "exit_code": 0
  },
  {
    "cmd": "cargo test -p tokmd-cockpit",
    "exit_code": 0
  },
  {
    "cmd": "cargo test --no-default-features -p tokmd-cockpit",
    "exit_code": 0
  }
]
```
