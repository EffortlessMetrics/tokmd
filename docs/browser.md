# Browser Runner

Use the browser runner when you want a no-install, capability-honest way to
inspect a repository or local file set and download a receipt. It is a front
door for safe inspection, not a browser clone of the native CLI.

## Best Fit

Browser mode is useful when the job is:

- tell me what this repo is;
- inspect a public GitHub repo without installing Rust;
- inspect selected local files or a local directory;
- download a JSON artifact for later native, CI, or agent use;
- demonstrate `tokmd` on a machine that should not run native tools.

Use native `tokmd` instead when the job requires git history, filesystem
baselines, PR review packets, policy gates, source context bundles, handoff
directories, or host-backed sensors.

## Supported Browser-Safe Modes

The current browser-safe command set is intentionally narrow:

| Mode | Browser status | Output |
| --- | --- | --- |
| `lang` | supported | language receipt |
| `module` | supported | module receipt |
| `export` | supported | file inventory |
| `analyze` | partial | browser-safe `receipt` and `estimate` presets |

The machine-readable contract is
[`docs/capabilities/wasm.json`](capabilities/wasm.json). The browser UI should
disable unavailable modes from the loaded bundle instead of pretending native
capabilities exist.

## Inputs

The browser runner works from ordered in-memory inputs:

- public GitHub repositories loaded through the GitHub tree and contents APIs;
- optional session-only GitHub token auth for higher rate limits;
- local file or directory selection from the browser.

It does not use native filesystem paths as trust roots. Local selections are
converted to normalized `{ path, text }` rows in memory.

## Artifacts

Use browser output as a downloadable receipt, then carry it to the next system
that needs it:

```text
browser run
  -> download JSON artifact
  -> inspect locally, attach to review, or hand to a native workflow
```

Browser artifacts are useful inputs for inspection and handoff. They are not a
replacement for native review packets, proof plans, baselines, or gates.

When the browser result shows the repo needs real PR review, proof routing, or
agent context, use [Browser to native](browser-to-native.md) for the shortest
bridge from browser receipts to native `cockpit`, `handoff`, and CI evidence
workflows.

## Native-Only Boundaries

These stay native-first:

- `cockpit` review packets;
- `gate` policy evaluation;
- `sensor` envelopes;
- `baseline` persistence;
- `context` bundles;
- `handoff` directories;
- git-history enrichers such as churn, hotspots, and freshness;
- filesystem sensors and validated-root scans.

If a workflow needs one of these, use the native CLI and treat the browser
artifact as supporting evidence only.

## Runner Development

The browser runner lives in `web/runner`.

```bash
npm --prefix web/runner run build:wasm
npm --prefix web/runner test
```

The release artifact `tokmd-wasm-<tag>.tar.gz` is extracted into
`web/runner/vendor/tokmd-wasm/` for repeatable deployments. See
[`web/runner/README.md`](../web/runner/README.md) for cache semantics, local
file ingest, progress events, authenticated fetch behavior, and integration
notes.
