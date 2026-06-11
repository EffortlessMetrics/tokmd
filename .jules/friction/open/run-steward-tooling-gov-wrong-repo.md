# Wrong Repo Workflow Friction

**Run ID**: `run-steward-tooling-gov`
**Date**: 2026-06-08
**Surface**: `docs/release-readiness.md`, `ci/proof.toml`
**Persona**: Steward 🚢

## Friction Event

A `steward_release` prompt was executed directly against `EffortlessMetrics/tokmd` to align `docs/release-readiness.md` with the `release_metadata` proof scope in `ci/proof.toml`. The resulting PR was closed by the user as "wrong-repo intake".

## Cause

The dual-repo workbench boundary dictates that normal tokmd implementation work starts in `EffortlessMetrics/tokmd-swarm`. PRs are merged into the swarm repo, and then the publication repo (`EffortlessMetrics/tokmd`) imports those changes via explicit merge commits. Jules currently receives prompts directed at `tokmd` but the PR is rejected because it is the publication repo, not the implementation workbench.

## Recommended Fix

The overarching test scaffolding or external prompt router should clone and point Jules at `EffortlessMetrics/tokmd-swarm` when issuing active mutation prompts, rather than asking Jules to mutate `EffortlessMetrics/tokmd` directly and triggering a repo-topology rejection.
