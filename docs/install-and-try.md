# Install And Try tokmd

Use this path when you want a fast first run before learning the rest of the
repo evidence system.

This guide gives you one useful command first, then shows where to go when the
job becomes PR review, agent handoff, CI evidence, or browser evaluation.

## 1. Install

Install from crates.io:

```bash
cargo install tokmd --locked
tokmd --version
```

Other install paths:

- Download release binaries from
  <https://github.com/EffortlessMetrics/tokmd/releases>.
- Use Nix:

  ```bash
  nix run github:EffortlessMetrics/tokmd -- --version
  ```

- Use the GitHub Action when you want CI artifacts. See
  [GitHub Action quickstart](action-quickstart.md).

## 2. Inspect A Repo

Run:

```bash
tokmd --format md --top 8
```

Open first: terminal output.

This shows the language mix, total size, and largest surfaces. It does not run
tests, review a PR, or prove release readiness.

Next useful command:

```bash
tokmd analyze --preset risk --format md
```

Use the risk preset when you want derived risk, effort, complexity, and
git-backed signals before choosing what to inspect next.

## 3. Review A PR

Run:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review
```

Open first:

1. `.tokmd/review/review-map.md`
2. `.tokmd/review/comment.md`
3. `.tokmd/review/evidence.json`

In a tokmd contributor checkout, verify the packet:

```bash
cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

The packet tells you what to review first, what evidence exists, what evidence
is missing or degraded, and which commands reproduce the evidence. It is not a
merge verdict.

## 4. Prepare A Coding-Agent Handoff

Run:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --out-dir .handoff
```

Open first:

1. `.handoff/work-order.md`
2. `.handoff/code.txt`
3. `.handoff/manifest.json`

Give the agent `work-order.md` first and `code.txt` as the bounded source
bundle. The handoff does not prove that tests passed. When review or proof
artifacts exist, link them with the flags shown in [Handoff bundles](handoff.md)
instead of pasting logs into the prompt.

## 5. Try Browser Mode

Use the browser runner when you want no-install inspection over browser-safe
inputs. It can summarize supported GitHub-loaded or local-file inputs and
download a browser-safe receipt.

Browser mode is not native mode. It does not produce cockpit packets, gates,
context bundles, handoff directories, git-history enrichers, or AST-backed
claims.

Next native command for real PR review:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review
```

See [Browser runner](browser.md) for supported modes and native-only
boundaries, or [Browser to native](browser-to-native.md) when a browser trial
needs to become PR review, handoff, or CI evidence.

## 6. Use CI Evidence

Use the GitHub Action when you want the first CI adoption path:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.12.0'
    paths: .
    artifact: 'true'
    comment: 'false'
```

For PR review packets, use `mode: cockpit` and `review-packet: 'true'`. The
Action can upload artifacts and optionally comment on a PR. It does not promote
advisory proof, enable Codecov upload by default, or make a merge verdict.

See [GitHub Action quickstart](action-quickstart.md) for copy-ready workflows
or [GitHub Action reference](github-action.md) for all inputs and outputs.

## 7. Check Release-Facing Evidence

Run:

```bash
cargo xtask publish-surface --json --verify-publish
cargo xtask version-consistency
```

Open first: publish-surface output or saved JSON, then version-consistency
output.

This checks package-surface and version-readiness evidence. It does not publish
crates, create tags, create GitHub releases, move release aliases, or approve
release mutation.

See [Publishing evidence](publishing-evidence.md) for the release-facing
reading order or [Release readiness](release-readiness.md) for the shorter
pre-release evidence quickstart.

## Where To Go Next

- [User paths](user-paths.md) maps each job to the command, primary artifact,
  first file to open, meaning, non-meaning, and next action.
- [Copy-Ready Workflows](workflows.md) gives complete command sequences.
- [Artifact glossary](artifacts.md) explains receipt and packet names.
- [Review packet contract](review-packet.md) explains cockpit review packets.
- [Handoff bundles](handoff.md) explains coding-agent work orders.
