# Decision

## Focus
We are examining feature boundary hygiene across the workspace, specifically looking at how `tokmd-cockpit` relies on `tokmd-git`.

## Context
`tokmd-cockpit` declares a `git` feature in its `Cargo.toml`:
```toml
[features]
default = ["git"]
git = ["dep:tokmd-git"]
```

It is full of `#[cfg(feature = "git")]` conditionals. However, inspecting `crates/tokmd-cockpit/src/lib.rs` and other files reveals heavily nested `#[cfg(feature = "git")]` gates around a huge surface area of Cockpit's logic. Cockpit's purpose is PR metrics computation, which relies inherently on `git`. The attempt to make `git` optional in Cockpit results in dozens of empty stubs or skipped logic when the feature is disabled, reducing its usefulness to near zero without `git`. Since `tokmd-cockpit` depends heavily on `tokmd-git` and is inherently git-driven, there's little value in an optional `git` feature for `tokmd-cockpit`. Making `tokmd-git` a hard dependency of `tokmd-cockpit` will simplify the crate significantly.

## Option A (Recommended)
Remove the `git` feature from `tokmd-cockpit` and make `tokmd-git` a standard, required dependency.
Remove all `#[cfg(feature = "git")]` conditionals from `tokmd-cockpit`.
Update `tokmd` and `tokmd-core` to remove the feature forwarding for `tokmd-cockpit/git`.

- **Trade-offs:**
  - Structure: Simplifies Cockpit code immensely by removing dozens of `#[cfg]` gates. Makes feature boundaries clearer—if you want Cockpit, you get Git.
  - Velocity: Future Cockpit work won't require stubbing out non-git paths.
  - Governance: Aligns the architectural seam. Cockpit is built on top of Git.

## Option B
Keep the `git` feature in `tokmd-cockpit` but clean up the usages.
- **Trade-offs:** Does not simplify the dependency structure, maintains an artificial feature boundary where none actually adds value (Cockpit without Git is mostly useless).

## Decision
Option A. I will remove the `git` feature from `tokmd-cockpit`, making `tokmd-git` a regular dependency, and strip out the `#[cfg(feature = "git")]` gating from `tokmd-cockpit`.
