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
