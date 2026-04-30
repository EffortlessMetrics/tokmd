# v1.10.0 Stable Release Train Checklist

This checklist captures the minimum-release discipline for promoting `v1.10.0-rc.1` to stable `v1.10.0`.

## Scope fence

Merge only:
- release blockers
- release-correctness docs/policy fixes
- already-clean proof PRs that do not widen product scope

Defer to `v1.11`:
- browser cache/progress/retry/auth work
- CLI ergonomics
- performance and model-behavior expansions

## Phase 0 — rebaseline

- `git fetch origin`
- `git switch --detach origin/main`
- `cargo xtask docs --check`
- `cargo xtask version-consistency`
- `cargo xtask publish-surface --json`
- `git diff --check`
- `cargo test -p tokmd --no-default-features`
- `cargo test -p tokmd --all-features`

## Phase 1 — blocker `#1457`

- If no-default-features already passes on main, close `#1457` as superseded.
- Otherwise restack and keep only:
  - `crates/tokmd/tests/cli_snapshot_golden.rs`
  - `crates/tokmd/tests/determinism.rs`
- Remove `.jules/runs/**`, `Cargo.lock` churn, and unrelated artifacts.

## Phase 2 — ADR keeper and publishability wording

Use `#1449` as keeper (if cleanly restacked), with tight scope:
- `docs/adr/0000-adr-process.md`
- `docs/adr/0001-production-package-publishability.md`
- `docs/adr/0002-crate-vs-module-boundaries.md`
- `docs/adr/0003-publish-surface-taxonomy.md`
- `docs/adr/0004-binding-surfaces.md`
- `docs/adr/0005-release-train-and-rc-semantics.md`
- `CHANGELOG.md` wording fix only if still needed

Policy wording that must remain true:
- no production Rust package may be `publish = false`
- `publish = false` is only allowed for dev/tooling/fuzz packages outside production closures

## Phase 3 — close duplicate PR families

After `#1457` and `#1449` resolution:
- close `#1451` if superseded by `#1457`
- close `#1442 #1443 #1444 #1445 #1446 #1447 #1448` as superseded by `#1449`

Park (do not close):
- `#1456 #1214 #1156 #1144`

## Phase 4 — stable prep PR

Create `release/v1.10.0-stable` and run:
- `cargo xtask bump 1.10.0`

Expected touched files:
- `Cargo.toml`
- `Cargo.lock`
- `crates/tokmd-node/package.json`
- `crates/tokmd-node/npm/package.json`
- `CHANGELOG.md`
- `ROADMAP.md`
- `docs/implementation-plan.md`
- `README.md` only if stable-version examples must change

## Phase 5 — pre-merge validation

Run sequentially (gate then deny):
- `cargo fmt-check`
- `cargo xtask docs --check`
- `cargo xtask version-consistency`
- `cargo xtask publish-surface --json`
- `cargo xtask publish-surface --json --verify-publish`
- `cargo xtask gate --check`
- `cargo deny --all-features check`
- `npm test --prefix web/runner`
- `cargo test -p tokmd --no-default-features`
- `cargo test -p tokmd --all-features`
- `git diff --check`

## Phase 6 — tag and verify stable

- `git tag -a v1.10.0 -m "v1.10.0"`
- `git push origin v1.10.0`
- monitor release workflow and verify:
  - release is not prerelease and is latest
  - `v1` tag moves to `v1.10.0`
  - crates.io + Docker publish complete
  - assets + checksums exist
