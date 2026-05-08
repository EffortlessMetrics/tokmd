# tokmd file policy

tokmd is Rust-first but legitimately contains non-Rust surfaces:

- GitHub Actions YAML
- Nix flake / lock
- Documentation
- JSON schemas
- Test fixtures
- Browser runner assets and Node package files
- Release packaging (Homebrew, AUR, Scoop, winget, Docker)
- Vendored / patched dependencies

These should be **explicit, owned, and covered** rather than tolerated by silence.
Prefer Rust-native tooling (`cargo xtask ...`) for generated or automatable repo
workflows; keep non-Rust files only when the consuming platform requires that
format or runtime.

## Allowlist shape

`policy/non-rust-allowlist.toml` lists `[[allow]]` entries:

| Field | Required | Meaning |
|-------|----------|---------|
| `glob` | yes | What the entry covers (e.g. `.github/workflows/*.yml`). |
| `kind` | yes | Structural category (e.g. `ci_declarative`, `documentation`, `test_fixture`). |
| `owner` | yes | Team/role responsible. "tokmd" is not an owner. |
| `surface` | yes | Where the file shows up — `build`, `release`, `ci`, `docs`, `agents`, etc. |
| `classification` | yes | `production` / `config` / `documentation` / `vendor` / `test_fixture`. |
| `reason` | yes | Why a non-Rust file lives in a Rust-first repo. |
| `covered_by` | optional | The proof obligation that exercises it. |

## Checker

```bash
cargo xtask check-file-policy
```

The checker walks the repo (excluding `.git/` and `target/`), normalises
file paths, and reports any non-Rust file that does not match an
`[[allow]]` glob. Rust source files (`*.rs`) are not the subject of this
policy and are skipped.

The advisory phase reports drift; `--strict` turns drift into a non-zero
exit.

## Adding a new non-Rust file

1. Add a `[[allow]]` entry with the right `kind`, `owner`,
   `classification`, and `reason`.
2. Re-run `cargo xtask check-file-policy` and confirm the file matches.
3. Commit the policy edit alongside the file change.

## Rust files

Rust files are governed by the workspace lints, the proof policy
(`ci/proof.toml`), and the lint/no-panic ledgers. They are deliberately
out of scope here.
