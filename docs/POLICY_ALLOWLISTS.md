# tokmd policy allowlists

tokmd uses TOML allowlists to make exceptions reviewable rather than
invisible. Each allowlist follows the same schema shape: an entry has an
**identity**, an **owner**, a **classification**, a **reason**, and an
**expiry** or `review_after` date.

| Allowlist | Subject | Owner |
|-----------|---------|-------|
| `policy/clippy-debt.toml` | Temporary Clippy debt for the strict baseline. | per-entry |
| `policy/clippy-exceptions.toml` (PR 07) | Source-suppression receipts for `#[expect(clippy::...)]`. | per-entry |
| `policy/no-panic-allowlist.toml` | Semantic no-panic-family receipts. | per-entry |
| `policy/non-rust-allowlist.toml` (PR 05) | Non-Rust file surfaces (YAML, Nix, JSON, etc). | per-entry |
| `policy/ci-lane-whitelist.toml` (PR 02) | CI lane purpose / cost / trigger receipts. | per-lane |
| `policy/ci-whitelist-exceptions.toml` (PR 02) | Carve-outs for default-PR expensive lanes during transition. | per-exception |
| `policy/ripr-suppressions.toml` (PR 11) | ripr advisory suppressions using RIPR's native manifest schema. | per-entry |

## Common rules

- Identity should be **semantic** (selector + container + fingerprint) not
  positional. Line/column metadata is `last_seen` advisory only — not the
  primary key.
- Every entry MUST have an owner. "tokmd" is not an owner.
- Every entry MUST have a `reason` that explains *why the exception is
  allowed*, not what the lint says.
- Every entry MUST have either an `expires` date or a `review_after` date.
- Schema version pinning lives in the file header (`schema_version = ...`).

`policy/ripr-suppressions.toml` follows the native `ripr` suppression manifest
shape so the installed `ripr` binary can validate and consume it directly.

## Adding an entry

1. Run the propose command for the relevant ledger (e.g. `cargo xtask
   no-panic propose`).
2. Edit the proposed entry — fill in `owner`, `classification`,
   `explanation`, `expires`.
3. Re-run the corresponding `check-*` command and confirm clean.
4. Commit the policy edit alongside the source change so the receipt is
   reviewable.

## Expiry

Expired entries are surfaced by the `check-*` commands. A repo with
expired entries will eventually fail the policy gate; do not paper over an
expiry by extending it without a real review.
