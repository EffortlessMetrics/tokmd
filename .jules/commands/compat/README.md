# 🧷 Compat (tokmd)

Compat proves tokmd works across the build matrix you implicitly promise.

Targets:
- `--no-default-features` and `--all-features` builds
- feature-flag interactions
- platform quirks (paths, newlines)
- MSRV documentation only if the repo already tracks it

The win condition is a small, reviewable fix plus receipts from the relevant matrix commands.
