# ADR: Release Train and RC Semantics

- Status: Proposed
- Date: 2026-04-29

## Context

Release automation needs durable rules for RC safety and stable tag behavior.

## Decision (target)

Define stable vs RC semantics for tags, prerelease assets, package publishing, and channel aliases.

## Required RC safeguards

- RC must not move `v1`.
- RC must not become latest.
- RC must not publish stable Docker aliases.
- RC crates.io publication must be explicit/opt-in.
