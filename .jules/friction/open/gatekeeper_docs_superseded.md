# Friction Item: Docs sync superseded

## Issue
Attempted to align `docs/reference-cli.md` with `cargo xtask docs --update` by replacing manual tables with `<!-- HELP: <cmd> -->` markers. The user commented that the PR was superseded by #1211 and closed the PR.

## Impact
Wasted compute cycles on a task that was already completed externally.

## Recommendation
Before executing sweeping codebase maintenance (especially docs/linting), consider checking recent closed PRs or doing a quick `git log` or `git fetch origin main` to ensure the work hasn't already been done recently on the mainline branch, if such information is accessible.
