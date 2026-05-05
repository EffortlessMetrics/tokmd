## Options Considered

### Option A: Fix GitHub Actions workflows calling `EffortlessMetrics/tokmd@v1` to explicitly use release candidate or v1 tag without conflicting paths.
- **What it is**: The file `docs/github-action.md` indicates that GitHub actions uses should point to `@v1` or `@v1.10.0-rc.1`. However, a grep in `.github/workflows/` did not show any usages of `EffortlessMetrics/tokmd@...` at all. This means our codebase doesn't test the action locally using the explicit `v1` tag, or the actual action might not be called in these internal CI workflows. Let's look closer at `test-action.yml`.
- **Trade-offs**: If we don't have this, it's not actually an issue unless we find drift elsewhere.

Let me explore `docs/github-action.md` vs `.github/workflows/test-action.yml`.

I checked `docs/github-action.md` and `.github/workflows/test-action.yml`. The test action uses `uses: ./` which is testing the local action. The docs say to use `uses: EffortlessMetrics/tokmd@v1.10.0-rc.1`. Wait, looking at `publish-surface.md`, I see the `v1` tag is mentioned.

Let's check `cargo xtask version-consistency`. It checks "Node package manifest versions match 1.10.0". So everything matches.

What is the best release-hygiene target?
1. publish-plan/version-consistency drift (We just ran `cargo xtask version-consistency` and it passed.)
2. release metadata or changelog mismatch
3. RC-hardening docs/checks

Looking at `docs/github-action.md`, wait, it says `uses: EffortlessMetrics/tokmd@v1.10.0-rc.1` in the examples!
Line 52: `- uses: EffortlessMetrics/tokmd@v1.10.0-rc.1`
Line 64: `| 1.10.0-rc.1 | v1.10.0-rc.1 |`

But the latest release is actually `v1.10.0`! The `CHANGELOG.md` says "Stable release following `v1.10.0-rc.1` validation" and `ROADMAP.md` says `v1.10.0` is complete. The action documentation has drift! It's showing the `rc` version instead of the stable version in the example!
Also, what about `.github/workflows/`? No, the action docs are user-facing.

Let's do a more thorough grep in `docs/` and `README.md` for `1.10.0-rc.1` and `rc.1`.

Wait, `docs/github-action.md` lines 49-64 are specifically showing how to use an RC!
"For release-candidate smoke tests, pin both the Action ref and the downloaded binary:
- uses: EffortlessMetrics/tokmd@v1.10.0-rc.1"
This is explicitly showing the rc as an example of how to test an rc! It is correct that it mentions rc.1 there.

So what's the real problem? Let's check `xtask` issue?
The prompt mentions:
"Target ranking
1) publish-plan/version-consistency drift
2) release metadata or changelog mismatch
3) RC-hardening docs/checks
4) low-risk release-surface fixes in workflows/docs/metadata"

Let's check 2) release metadata or changelog mismatch.
What about `Cargo.toml` descriptions, `README.md`, `LICENSE`, `CHANGELOG.md`?

If everything is consistent, where is the drift or mismatch?
Wait, what does `cargo xtask version-consistency` test?
It tests "Cargo crate versions match X", "Cargo workspace dependency versions match X", "Node package manifest versions match X", "No case-insensitive tracked-path collisions detected."
But what about Python? It didn't test Python versions!
Let's check `crates/tokmd-python/pyproject.toml` and other Python configuration for version.

If Python uses `dynamic = ["version"]` meaning it pulls from `Cargo.toml`, then it will match.

Let's check 3) RC-hardening docs/checks.
What about `docs/github-action.md` having `v1.10.0-rc.1`?
Wait, if `1.10.0` is already released (CHANGELOG says "Stable release following v1.10.0-rc.1 validation" and ROADMAP says "v1.10.0 Complete"), the GitHub action documentation in `docs/github-action.md` STILL uses `v1.10.0-rc.1` in the examples!
Line 52: `- uses: EffortlessMetrics/tokmd@v1.10.0-rc.1`
Line 54: `version: '1.10.0-rc.1'`
Line 64: `| 1.10.0-rc.1 | v1.10.0-rc.1 |`

Is it drift? The action doc says:
"For release-candidate smoke tests, pin both the Action ref and the downloaded binary:
- uses: EffortlessMetrics/tokmd@v1.10.0-rc.1"
This makes sense as a persistent example of HOW to do an RC test. It doesn't necessarily mean it is outdated; it just uses `v1.10.0-rc.1` as the example.

Wait! What about `.github/workflows/` files using an old version?
None of the workflows use `tokmd@`. Wait, let me check `docs/README.md` or `README.md`?

Ah! Look at `docs/sensor-report-v1.md`, `docs/SCHEMA.md`, `docs/NOW.md`, and `docs/design.md`. They all mention `1.9.0` instead of `1.10.0`!
`docs/NOW.md` says: "Updated after the `1.9.0` release. ... `1.9.0` is out"
Wait, `v1.10.0` is the actual current version.

Let's check `docs/NOW.md`:
"Updated after the 1.9.0 release.
1.9.0 is out, the release pipeline proved green end-to-end, and main is back to the normal development lane.
Docs and operator surfaces should match reality: keep roadmap, release instructions, architecture docs, and repo-native commands aligned with what actually shipped in 1.9.0."

Wait, `NOW.md` definitely needs to be bumped to `1.10.0`.
And `docs/SCHEMA.md` and `docs/sensor-report-v1.md` have hardcoded `"version": "1.9.0"`. It's better to update them to `1.10.0` since `1.10.0` shipped. Wait, let's see if `cargo xtask docs --check` checks those. No, `cargo xtask docs --check` only checks the help markers in `reference-cli.md`.
Wait, is `schema.json` version also outdated?

So the schemas define "version" but do not hardcode a specific version.
However, `docs/NOW.md` explicitly has `1.9.0` out of date.
`docs/SCHEMA.md`, `docs/design.md`, `docs/sensor-report-v1.md`, and `docs/implementation-plan.md` have examples hardcoded to `1.9.0`. Wait, `docs/implementation-plan.md` has `v1.9.0` but that's for "Phase 5: ... v1.9.0" which is correct history. But `NOW.md` is clearly stating "1.9.0 is out" and it should be "1.10.0".

Wait! What about the `README.md` itself? Does it have `tokmd@v1.9.0` or `1.10.0`?
Let's check `README.md`.

So the mismatch is the examples in `docs/` and the status in `docs/NOW.md`.

Let's check `cargo xtask docs --check`. It doesn't check versions, only CLI help drift.
Wait, `cargo xtask version-consistency` tests:
  - Cargo crate versions match 1.10.0.
  - Cargo workspace dependency versions match 1.10.0.
  - Node package manifest versions match 1.10.0.
  - No case-insensitive tracked-path collisions detected.

Wait! In `action.yml`, what is the default version?

So the target is "publish-plan/version-consistency drift" or "release metadata or changelog mismatch".
The `docs/NOW.md` explicitly says "Updated after the 1.9.0 release. 1.9.0 is out ... keep roadmap ... aligned with what actually shipped in 1.9.0." But the current release is `1.10.0`. This is docs drift.
Are there any other surfaces that are out of sync?

Let's check `docs/SCHEMA.md`, it has `"version": "1.9.0"` in its examples.
Let's check `docs/sensor-report-v1.md`, it has `"version": "1.9.0"`.
`docs/design.md` has `"tool_version": "1.9.0"`.

Is `Steward` supposed to fix docs? The prompt says "Release and governance surfaces such as docs drift, publish plan, version consistency, and metadata alignment." and "Target ranking 1) publish-plan/version-consistency drift 2) release metadata or changelog mismatch 3) RC-hardening docs/checks 4) low-risk release-surface fixes in workflows/docs/metadata".

So Option A is to fix `docs/NOW.md` and the version examples in `SCHEMA.md`, `sensor-report-v1.md`, and `design.md` to reflect `1.10.0`.
What else might be out of date?

Let's do a search for `1.9.0` to see if there are other places.

Wait, `docs/implementation-plan.md` and `docs/architecture.md` are discussing historical `1.9.0` phases, which is fine to keep as `1.9.0` since they're specifically referencing the goals/non-goals of that historical phase.
But `docs/NOW.md` should be updated to `1.10.0` as it's the current "One-screen operational truth".
`docs/SCHEMA.md`, `docs/sensor-report-v1.md`, and `docs/design.md` can have their examples bumped to `1.10.0` to match the current release.

Is there any other version consistency problem? Let's check `xtask/src/tasks/version_consistency.rs`.

Wait, `tests` in `version_consistency.rs` have hardcoded `1.9.0` which is perfectly fine since they are unit tests verifying mismatches.
However, there is `version_consistency.rs` checking `Cargo.toml` and node manifests, but it does NOT check python versions!
Python versions are set via `dynamic = ["version"]` which uses `cargo` version (Maturin does this). So they are implicitly synced.
What about the Github Action documentation drift? The version examples in schema docs?
I will write Option A as updating `NOW.md` and the schemas/docs to `1.10.0` to eliminate documentation drift for the current stable release.

Option A: Fix docs/NOW.md and schema examples to align with 1.10.0 release reality.
- What it is: Bumping `docs/NOW.md` to indicate 1.10.0 is the current stable release and updating JSON schema examples from 1.9.0 to 1.10.0.
- Why it fits: "docs drift" and "metadata alignment" are specifically ranked #1 and #4 for Steward, and "docs and operator surfaces should match reality" is literally in `NOW.md`.

Option B: Do nothing, because `cargo xtask version-consistency` passes.
- What it is: Create a learning PR without touching code.
- Why it fits: If docs examples don't represent a broken contract.

I'll choose Option A and fix `NOW.md`, `docs/SCHEMA.md`, `docs/sensor-report-v1.md`, and `docs/design.md`.
Wait, let's also update the example `action.yml` uses in `README.md`? The prompt says "Do not broaden scope once a coherent target is chosen". I'll just fix the `1.9.0` -> `1.10.0` docs drift for the stable version. Wait! `README.md` already has `1.10.0` correctly!

Wait, is there anything else missing in `publish-plan`? Let's check `cargo xtask publish --plan`. It output "Workspace version: 1.10.0". Is there any drift there?
Let's check `CHANGELOG.md` again.
Okay, I will choose Option A: fix the docs drift for the 1.10.0 release.
Specifically modifying `docs/NOW.md`, `docs/SCHEMA.md`, `docs/sensor-report-v1.md`, and `docs/design.md`.

Wait, the prompt says:
"Target ranking
1) publish-plan/version-consistency drift
2) release metadata or changelog mismatch
3) RC-hardening docs/checks
4) low-risk release-surface fixes in workflows/docs/metadata"

This falls under #4 (low-risk release-surface fixes in workflows/docs/metadata) or #2 (release metadata mismatch) depending on how you view `NOW.md`.

Let's do the updates.
