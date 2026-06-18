# Option A: Fix policy for test fixtures and scripts

- what it is: Update `policy/non-rust-allowlist.toml` to explicitly allow the `fixtures/syntax/**` testing files, `scripts/check-no-bare-self-hosted.sh` verification script, and ignore `xtask/target/**` artifacts.
- why it fits this repo and shard: It locks in the contract for non-Rust file paths, directly addressing a policy gate finding (`cargo xtask check-file-policy`), fixing the specific un-allowlisted paths and protecting the repo's determinism contract. Ignoring `xtask/target` avoids surfacing test artifacts from tests we just ran.
- trade-offs:
  - Structure: Improves repo structural governance by explicitly covering test data and scripts.
  - Velocity: Eliminates the CI/friction warnings during local dev when running `cargo xtask check-file-policy`.
  - Governance: Ensures all script and fixture additions meet the explicit file policy and pass validation, reinforcing the gatekeeper persona.

# Option B: Remove the unallowlisted files entirely

- what it is: Delete `fixtures/syntax/**` and `scripts/check-no-bare-self-hosted.sh` as they shouldn't exist if they aren't approved.
- when to choose it instead: If the files are obsolete or unmaintained cruft.
- trade-offs: We lose testing fixtures and a security/governance script, which seems clearly unintended.

# Decision
We will go with **Option A**, fixing the policy allowlist to govern these existing testing fixtures and infrastructure scripts correctly, and ignoring xtask target directory properly.
