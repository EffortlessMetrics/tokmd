# Debugging tokmd Development

Use this guide when a contributor or maintainer change fails locally, in CI, or
in review evidence. It is a runbook for finding the failing boundary quickly,
not a replacement for the testing strategy in [testing.md](testing.md) or the
user-facing CLI help in [troubleshooting.md](troubleshooting.md).

## Start Small

First reproduce the narrowest failing command from the repository root:

```bash
cargo test -p <crate> <test_name> --verbose
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan.json
```

If a command only fails in CI, open the failing job and copy the exact command,
feature flags, working directory, and environment variables before widening the
search. Do not treat a skipped route job as a failed required check; the routed
aggregate is the status that branch protection consumes.

## Test Failures

For a failing Rust test:

1. Run the single failing test with `--verbose`.
2. Re-run with backtraces enabled.
3. Inspect the fixture or receipt that the assertion compares.
4. Fix the smallest behavior or normalization issue, then re-run the same test.

Backtrace examples:

```powershell
$env:RUST_BACKTRACE='1'; cargo test -p tokmd <test_name> --verbose
```

```bash
RUST_BACKTRACE=1 cargo test -p tokmd <test_name> --verbose
```

On Windows, use `cargo fmt-check` and `cargo fmt-fix` instead of plain
`cargo fmt --all`; the workspace can exceed Cargo's formatter argument limit.

If `target/debug` is crowding the system drive, set `CARGO_TARGET_DIR` to a
larger local disk for the proof command, then delete that alternate target
directory after the run if it is no longer needed.

## Snapshot Failures

Snapshot diffs are proof artifacts, not formatting noise to accept blindly.

When an `insta` snapshot fails:

1. Run the specific snapshot test first.
2. Review semantic changes in the `.snap.new` file.
3. Check path separators, timestamps, versions, temp paths, and line endings.
4. Fix normalization if the diff is accidental.
5. Accept only intended snapshots with `cargo insta review`.

Prefer structured JSON snapshots for structured receipts. They keep field
ordering and nested diffs reviewable.

## Property, Fuzz, And Mutation Failures

For property tests, keep the minimal failing input. Proptest writes regression
seeds under the crate's `tests/` area; commit a seed only when it protects a real
regression and belongs with the change.

Useful local commands:

```bash
cargo test -p tokmd-scan --test properties
PROPTEST_CASES=1024 cargo test -p tokmd-scan --test properties
```

For fuzz failures, preserve the crashing input and the target name. If the
nightly or sanitizer toolchain is unavailable, document that blocker and add
deterministic regression coverage for the same boundary when practical.

Mutation survivors usually mean the assertion is too weak, the input does not
exercise the branch, or the code path is dead. Prefer a targeted test that kills
the survivor over broad snapshot churn.

## Receipt Debugging

Most tokmd behavior is easier to debug by reading receipts than by reading raw
terminal logs. Write artifacts to `target/` and inspect the machine-readable
form first:

```bash
cargo run -p tokmd -- run --path . --out target/tokmd-debug
cargo run -p tokmd -- analyze --path . --format json
cargo run -p tokmd -- cockpit --base origin/main --head HEAD --out target/cockpit-debug
```

When changing a receipt shape, update the owning types, schema docs, and tests
together. Schema versions are family-specific; do not increment an unrelated
receipt family to make a test pass.

Use `RUST_LOG` only for temporary diagnosis. Durable PR evidence should point to
commands, receipts, checks, or artifacts rather than copied debug logs.

## CI Mismatches

For a CI-only failure:

1. Identify whether the failing check is required, advisory, or a route
   implementation job.
2. Compare the CI command with the local command you ran.
3. Check whether affected-proof routing included the changed files.
4. Re-run the narrow command locally with the same feature flags.
5. Record the hosted run ID in the PR if it is part of the proof.

The normal swarm required check is:

```text
Tokmd Rust Small Result
```

The runner-specific jobs may skip depending on route and event trust. Do not
paper over missing evidence with workflow-level path ignores; classify files in
the proof policy or CI lane whitelist so the repository explains why the proof
is sufficient.

## Performance Debugging

Start from a fresh timing receipt before optimizing:

```bash
cargo xtask perf-smoke
cargo run -p tokmd -- run --path . --out target/perf-debug
```

Compare like with like: same checkout, same feature set, same target directory
policy, and the same input corpus. A single local run is a lead, not proof of a
stable performance improvement.

## Further Reading

| Need | Read |
| --- | --- |
| Testing strategy and frameworks | [testing.md](testing.md) |
| Contributor workflow | [contributor-guide.md](contributor-guide.md) |
| User-facing CLI troubleshooting | [troubleshooting.md](troubleshooting.md) |
| Proof artifacts and review packets | [review-packet.md](review-packet.md) |
| Swarm CI routing and graph rules | [ci/swarm-routing.md](ci/swarm-routing.md) |
| Receipt schemas | [SCHEMA.md](SCHEMA.md) |
