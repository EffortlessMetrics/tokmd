# Install tokmd

For a first-run path after installation, use
[Install and try tokmd](install-and-try.md).

## Cargo

```bash
cargo install tokmd --locked
tokmd --version
```

## GitHub Releases

Download a platform binary from the latest GitHub release:

https://github.com/EffortlessMetrics/tokmd/releases

Stable release assets include Linux, macOS, and Windows binaries plus checksums.

### Platform support

| Tier | Platforms | Notes |
| --- | --- | --- |
| Supported | Linux, macOS, Windows | Release binaries, default CI, and documented config paths |
| Best-effort | Other Rust targets (for example WASM builds) | May compile with degraded user-directory discovery; not a release promise |

tokmd vendors a minimal `home` crate patch so uncommon targets can compile; see
[dependency maintenance spec](specs/dependency-maintenance.md#vendored-home-patch).

## Nix

```bash
nix run github:EffortlessMetrics/tokmd -- --version
```

## GitHub Action

Use the root composite Action when you want CI receipts, PR summaries, artifacts, or gates.

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    paths: .
```

See [GitHub Action reference](github-action.md) for modes, inputs, outputs, checkout guidance, release assets, comments, and failure behavior.

For machine-readable progress telemetry in custom Actions, set `TOKMD_PROGRESS_EVENTS=1` and use `--no-progress`. See [Progress events spec](specs/progress-events.md).
