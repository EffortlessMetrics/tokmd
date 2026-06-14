## Inspection
Reviewed `cli_snapshot_golden.rs` and `tests/snapshots` directory. Found that `diff` command snapshots were entirely missing. Determinism for other commands (`lang`, `module`, `export`, `analyze`) is protected by golden snapshot tests, but the `diff` pipeline's JSON and Markdown output formats had no such guard against regression.

## Options Considered

### Option A (recommended)
Add `snapshot_diff_json` and `snapshot_diff_markdown` tests in `cli_snapshot_golden.rs` to close this snapshot gap. This generates deterministic output from `diff` by comparing two `lang` format receipts (with and without `--exclude tests`), normalizes the output, and saves them as `.snap` files using `insta`.

### Option B
Refactor the `run_diff` module directly to unit test diff creation and serialization without CLI integration tests.

## Decision
Option A. This fits the 'Gatekeeper' persona perfectly by tightening snapshot coverage around output contracts and catching regressions in deterministic outputs across the entire `diff` CLI surface area.
