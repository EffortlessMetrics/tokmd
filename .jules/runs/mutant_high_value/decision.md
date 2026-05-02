## Options considered

### Option A: Improve testing for `env_interpreter_token`

- **What it is**: The `env_interpreter_token` function in `crates/tokmd-model/src/lib.rs` has several missed mutants (e.g., `delete match arm`, `replace match guard`). This indicates that the core scan model has gaps in how it determines environment interpreter tokens (like `env -S node`).
- **Why it fits**: Core modeling logic should be strictly tested. The `env_interpreter_token` function logic is high value because if it fails or behaves incorrectly, language detection based on shebangs is broken.
- **Trade-offs**:
  - *Structure*: Straightforward addition to unit tests.
  - *Velocity*: Fast to implement.
  - *Governance*: Increases mutation coverage.

### Option B: Improve testing for `get_file_metrics`

- **What it is**: The `get_file_metrics` and `metrics_from_byte_len` functions also lack robust coverage for the return tuples (e.g., replacing return with `(0, 0)`).
- **Why it fits**: Metrics calculation is central to `tokmd`'s purpose.
- **Trade-offs**:
  - *Structure*: Trivial.
  - *Velocity*: Very fast.
  - *Governance*: Might not be as complex to test.

## Decision

**Option A**. The `env_interpreter_token` function contains a lot of logic about `env` arguments (`--split-string`, `--chdir`, etc.) that are clearly un-exercised by tests. This creates a risk of regression in how `tokmd` detects languages via shebang lines containing `env`. It's a very meaningful behavioral path to secure.
