These files are checked-in documentation assets referenced by the root `README.md`.

- `readme-badge-lines.svg`
- `readme-badge-hotspot.svg`

They are intentionally committed so the rendered repository docs show stable example output on GitHub and docs mirrors.

They are generated SVG badges, not screenshots.

Policy:

- Refresh them only when the README examples should visibly change.
- Treat them as generated examples, not canonical test fixtures.
- Do not gitignore this directory while the README embeds these files.
