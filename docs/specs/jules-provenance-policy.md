# Spec: Jules Provenance Policy

Status: Draft

## Scope

Defines when `.jules/**` provenance updates are expected, allowed, or disallowed.

## Required rules

- Intentional provenance updates are not auto-rejection grounds.
- Accidental run-log churn in unrelated patch PRs is disallowed.
- Migration plan required before any hard block on legacy provenance paths.
