# Cartographer Decision: Roadmap and docs alignment

## Problem
The `ROADMAP.md` and `CHANGELOG.md` refer to "16 focused crates" or "16 published crates" in the context of the microcrate architecture split (e.g. from version 1.2.0).
The `docs/adr/0003-publish-surface-taxonomy.md` and `docs/publish-surface.md` also refer to 16 published crates.
However, looking at `Cargo.toml` and the `crates/` directory, there are currently 18 crates under `crates/` (plus `fuzz` and `xtask`):
- `tokmd`
- `tokmd-analysis`
- `tokmd-analysis-types`
- `tokmd-cockpit`
- `tokmd-core`
- `tokmd-envelope`
- `tokmd-format`
- `tokmd-gate`
- `tokmd-git`
- `tokmd-io-port`
- `tokmd-model`
- `tokmd-node`
- `tokmd-python`
- `tokmd-scan`
- `tokmd-sensor`
- `tokmd-settings`
- `tokmd-types`
- `tokmd-wasm`

In `docs/publish-surface.md`:
"Current compatibility surface (16 crates published + 4 non-crates.io)"
The 4 non-crates.io are: `tokmd-fuzz`, `tokmd-node`, `tokmd-python`, `xtask`.
The 16 published crates listed under "Current compatibility surface" are:
Supported public crates (11):
- `tokmd`
- `tokmd-analysis-types`
- `tokmd-cockpit`
- `tokmd-core`
- `tokmd-envelope`
- `tokmd-gate`
- `tokmd-io-port`
- `tokmd-sensor`
- `tokmd-settings`
- `tokmd-types`
- `tokmd-wasm`

Published support crates (5):
- `tokmd-analysis`
- `tokmd-format`
- `tokmd-git`
- `tokmd-model`
- `tokmd-scan`

This correctly sums to 16. However, `ROADMAP.md` says:
"Microcrate Architecture: Split into 16 focused crates for modularity and selective compilation"
and lists:
- `tokmd-types`, `tokmd-analysis-types`
- `tokmd-scan`, `tokmd-model`, `tokmd-tokeignore`, `tokmd-redact`
- `tokmd-format`, `tokmd-git`
- `tokmd-analysis`
- `tokmd-config`, `tokmd-core`
- `tokmd`

Notice `tokmd-tokeignore`, `tokmd-redact`, `tokmd-config` are listed. These crates no longer exist (they were merged back according to architecture consolidation plan).
Wait, the `ROADMAP.md` section is under "v1.2.0 - 2026-01-27" history. But wait, `ROADMAP.md` does not have a 1.2.0 section, that's in `CHANGELOG.md`.

Let's check `ROADMAP.md` for "16".
`ROADMAP.md` line 451: "Goal: Extract focused microcrates from monolithic modules for better separation of concerns." - Wait, I need to read `ROADMAP.md` properly.

Let's fetch the exact lines in `ROADMAP.md` and `docs/design.md`.

## Options Considered

### Option A (recommended)
Update the `publish-surface.md`, `ROADMAP.md`, `ADR-0003`, and other documentation artifacts to reflect the actual shipped reality: there are **16** published crates in the workspace (plus 4 non-crates.io packages, totaling 20 workspace members under `crates/` and `xtask` / `fuzz`).
Wait, the `cargo xtask publish --plan` output shows exactly 16 crates in the publish plan!
```
Publish order (16 crates):
   1. tokmd-gate
   2. tokmd-io-port
   3. tokmd-types
   4. tokmd-model
   5. tokmd-settings
   6. tokmd-scan
   7. tokmd-git
   8. tokmd-envelope
   9. tokmd-sensor
  10. tokmd-analysis-types
  11. tokmd-format
  12. tokmd-analysis
  13. tokmd-cockpit
  14. tokmd-core
  15. tokmd-wasm
  16. tokmd

Excluded crates:
  - tokmd-fuzz: NotPublishable
  - tokmd-node: NotPublishable
  - tokmd-python: NotPublishable
  - xtask: NotPublishable
```
So there are indeed exactly 16 crates! My earlier observation was a misunderstanding of what is a published crate vs a non-crates.io package. The "16 focused crates" count is absolutely correct for the publish closure.

Wait, the roadmap section says:
"Microcrate Architecture: Split into 16 focused crates for modularity and selective compilation"
- `tokmd-types`, `tokmd-analysis-types`
- `tokmd-scan`, `tokmd-model`, `tokmd-tokeignore`, `tokmd-redact`
- `tokmd-format`, `tokmd-git`
- `tokmd-analysis`
- `tokmd-config`, `tokmd-core`
- `tokmd`

Wait, `tokmd-tokeignore`, `tokmd-redact`, `tokmd-config` were merged back! So the current 16 crates are DIFFERENT from the 16 crates mentioned in the v1.2.0 changelog/roadmap. The current 16 crates are the ones listed in `xtask publish --plan`.
Is there a problem with the documentation?
Yes! The `ROADMAP.md` says:
"Microcrate Architecture: Focused crates for modularity (16 initial crates; later consolidated where boundaries were implementation-only)"
This is historically accurate.

Wait, what is the design/roadmap/requirements drift then?

Let's look at `docs/design.md`.

Ah, what about `CHANGELOG.md` or `ROADMAP.md`? The prompt specifically stated:
`Target ranking
1) roadmap/design/requirements drift from shipped reality
2) missing explanation of a real architectural/design choice
3) stale implementation-plan sections that mislead contributors
4) ADR/design-reference gaps that block clear future work`

And `ROADMAP.md` says:
"Microcrate Architecture: Focused crates for modularity (16 initial crates; later consolidated where boundaries were implementation-only)"

`CHANGELOG.md` line 1046: "Microcrate Architecture: Split into 16 focused crates for modularity and selective compilation"

Wait, where is the drift between shipped reality and roadmap/design/requirements docs?
Is there a difference in how many crates exist? No, the shipped reality literally has 16 published crates, as proven by `cargo xtask publish --plan --verbose`.

Let's look at `docs/architecture.md`.

Ah, wait, `docs/design.md` lines 165-175:
```
| `receipt` | derived + dup + git + complexity + API surface |
| `estimate` | `receipt` + effort estimation and optional base/head delta |
| `health` | derived + content (TODOs) + complexity + Halstead |
| `risk` | `health` + git (hotspots, coupling, freshness) |
| `supply` | derived + walk (assets) + content (deps) |
| `architecture` | derived + content (imports) |
| `topics` | semantic topic clouds (TF-IDF) |
| `security` | license radar + entropy profiling |
| `identity` | archetype detection + corporate fingerprint |
| `git` | predictive churn + advanced git metrics |
| `deep` | all enrichers (except fun) |
```

Wait, `docs/design.md` specifies the presets for `tokmd-analysis`. Let's check `crates/tokmd/src/cli/parser.rs` or `tokmd analyze --help` to see what the presets actually are, because maybe `design.md` is drifting from actual implementation. Let me run `tokmd analyze --help` or read `crates/tokmd/src/cli/parser/analyze.rs`.

Ah! Look at `AnalysisPreset`:
```rust
pub enum AnalysisPreset {
    Receipt,
    Estimate,
    BunUb,
    Health,
    Risk,
    Supply,
    Architecture,
    Topics,
    Security,
    Identity,
    Git,
    Deep,
    Fun,
}
```
Wait! `BunUb` is not listed in `docs/design.md`!
```markdown
| `receipt` | derived + dup + git + complexity + API surface |
| `estimate` | `receipt` + effort estimation and optional base/head delta |
| `health` | derived + content (TODOs) + complexity + Halstead |
| `risk` | `health` + git (hotspots, coupling, freshness) |
| `supply` | derived + walk (assets) + content (deps) |
| `architecture` | derived + content (imports) |
| `topics` | semantic topic clouds (TF-IDF) |
| `security` | license radar + entropy profiling |
| `identity` | archetype detection + corporate fingerprint |
| `git` | predictive churn + advanced git metrics |
| `deep` | all enrichers (except fun) |
```
Wait, `fun` is missing from `docs/design.md` too in the table? Wait, let me check `docs/design.md` again.
Line 175 of `docs/design.md` is:
```markdown
| `deep` | all enrichers (except fun) |
| `fun` | eco-label, novelty outputs |
```
So `fun` IS there, but `BunUb` is MISSING from `docs/design.md`. Let me grep `BunUb` in the docs.

Ah, so `bun-ub` is definitely an active and documented preset (it even has its own `docs/analyze/bun-ub.md` file), but it is completely missing from the architecture and design overviews.
Specifically, `docs/design.md` lines 165-175:
```markdown
| `receipt` | derived + dup + git + complexity + API surface |
| `estimate` | `receipt` + effort estimation and optional base/head delta |
| `health` | derived + content (TODOs) + complexity + Halstead |
| `risk` | `health` + git (hotspots, coupling, freshness) |
| `supply` | derived + walk (assets) + content (deps) |
| `architecture` | derived + content (imports) |
| `topics` | semantic topic clouds (TF-IDF) |
| `security` | license radar + entropy profiling |
| `identity` | archetype detection + corporate fingerprint |
| `git` | predictive churn + advanced git metrics |
| `deep` | all enrichers (except fun) |
```
It is missing `bun-ub`.

Let's check if there are other presets missing.
`crates/tokmd/src/cli/parser/analysis.rs` has:
```rust
pub enum AnalysisPreset {
    Receipt,
    Estimate,
    BunUb,
    Health,
    Risk,
    Supply,
    Architecture,
    Topics,
    Security,
    Identity,
    Git,
    Deep,
    Fun,
}
```

So `BunUb` is the only missing one in `docs/design.md`!
Also, what about `ROADMAP.md`? Let's check `ROADMAP.md` for `bun-ub`... wait, `grep "bun-ub" ROADMAP.md` returned nothing! So the `ROADMAP.md` is also missing the `bun-ub` preset! It must have been added in `v1.12.0` (as per `CHANGELOG.md`/releases) but the Roadmap and Design docs were never updated.

Let's look at `CHANGELOG.md` for "bun-ub".

Wait, `CHANGELOG.md` does not have "bun-ub" in the grep output earlier! Oh wait, I didn't grep `CHANGELOG.md` for `bun-ub`. Let's do that.
Also let's check `docs/implementation-plan.md` for "bun-ub".

Okay, so `bun-ub` preset was added in v1.12.0. However:
1. `docs/design.md` Analysis preset table does not list `bun-ub`.
2. `ROADMAP.md` does not list `bun-ub` under any release! Wait, let me check `ROADMAP.md` for `v1.12.0`.

Ah!
`ROADMAP.md` is out of date. It still lists `v1.12.x` as a future horizon:
```
### v1.12.x — Selection-First Product and Evidence Work
```
But `Cargo.toml` is currently at `1.13.1` !
And `docs/implementation-plan.md` has Phase 5d ending at `v1.11.0`, but we actually shipped `v1.12.0` and `v1.13.0` which added the `bun-ub` preset and other things!
Let me verify if `ROADMAP.md` has a `v1.13` section.

Wow, `ROADMAP.md` and `docs/implementation-plan.md` are completely missing the 1.12.0 and 1.13.0 releases!
They both still treat `1.12.x` or `v2.0` as future milestones, while `CHANGELOG.md` lists `v1.12.0` and `v1.13.0` which added the `bun-ub` preset.

Let me grep `CHANGELOG.md` to see what actually shipped in 1.12.0 and 1.13.0.

Okay, so the roadmap/design docs drifted from reality because they do not document:
1. `bun-ub` preset (added in 1.12.0)
2. `syntax` feature and evidence packets (added in 1.13.0)
3. The roadmap needs an update to mark `v1.12.x` and `v1.13.x` as completed phases/milestones, moving them to completed.

Let's check `docs/design.md`.
It lists `analyze` presets. We should add `bun-ub` preset:
`| bun-ub | scoped bun-ub review evidence |`
Wait, let's look at `docs/requirements.md` to see if there is drift there.
`docs/requirements.md` says:
```
| analyze | Derived metrics with preset system |
| cockpit | PR metrics with evidence gates |
...
| evidence-packet | PR review evidence packet generation | <-- Missing in requirements.md CLI list!
| syntax | Syntax-aware analysis receipts | <-- Missing in requirements.md CLI list!
```
Let me check `tokmd --help` or `Cargo.toml` to see what commands are present.

Okay, so `syntax` and `evidence-packet` are missing from `docs/requirements.md`.
Also, `docs/design.md` Analysis presets are missing `bun-ub`.

Let's check `ROADMAP.md` again.

So the target is exactly this:
The shipped reality has drifted from the roadmap/design docs. Specifically:
1. `ROADMAP.md` still lists `v1.12.x` under "Future Horizons" (line 608), but `1.12.0` and `1.13.0` have already shipped. I should move these into the completed sections in `ROADMAP.md` and `docs/implementation-plan.md` using the data from `CHANGELOG.md` and release ledgers.
2. `docs/design.md` is missing the `bun-ub` preset in its analysis presets list.
3. `docs/requirements.md` is missing `syntax` and `evidence-packet` commands in its `CLI (Stable Surfaces)` interfaces list.

I will formulate a patch to fix all of these.
Option A: Update `ROADMAP.md`, `docs/implementation-plan.md`, `docs/design.md`, and `docs/requirements.md` to reflect the 1.12 and 1.13 realities.

Let's prepare the plan!

I will now create the PR plan.

Option A (recommended)
Update `ROADMAP.md` and `docs/implementation-plan.md` to move 1.12.0 and 1.13.0 to completed features.
Update `docs/design.md` to add `bun-ub` to the preset table.
Update `docs/requirements.md` to add `evidence-packet` and `syntax` to the CLI table.

Option B
Wait and write a learning PR instead.

I will proceed with Option A because there is factual drift in the planning and requirements documentation.

Ah, I must formulate the plan using direct, imperative language. I need to be completely literal with commands.

Let's do some more gathering of the exact text to replace.
I'll create patch files or use exact sed commands for ROADMAP.md, implementation-plan.md, design.md, and requirements.md.

Also, I have already verified that `syntax` and `evidence-packet` exist via running `tokmd --help`. So it's not a hallucination, I saw it in my `tokmd --help` trace! But I'll use concrete literal commands.

Let me construct the diffs.

I have prepared the specific `patch_*.diff` files for the updates. The plan will be to use the `replace_with_git_merge_diff` API with these exact files, or just `sed` / `patch` commands in bash to do it.

Actually, using the `replace_with_git_merge_diff` tool would mean I need to construct the JSON payload. So I can just do this in a bash script inside the plan:
"Run a bash script to read the patch files and apply them using patch tool, or better just use `replace_with_git_merge_diff` in the execution directly."
Since the prompt specifically says to make tool calls during execution, I will just plan to run the bash patch.

But the instructions say "replace this step with concrete commands (e.g. `sed` or `cat << 'EOF' > patch`)..."
Okay, let's write out the full plan.
