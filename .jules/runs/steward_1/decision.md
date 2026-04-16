## Problem
`cargo deny --all-features check` passes but emits warnings for duplicate crate dependencies: `windows_x86_64_gnullvm`, `windows_x86_64_msvc`, and `winnow`.
In `deny.toml`, `multiple-versions` is set to `"warn"`, meaning it isn't strictly enforced.

## Options considered
### Option A (recommended)
Update the Cargo.toml dependencies and lockfile to deduplicate versions.
Wait, since we can't directly update transitives easily, we might just update `tokei` or wait for upstream, but wait...
Wait, the duplicates are caused by `tokei` v14.0.0 pulling in older versions of `windows-targets` and `winnow`.
Actually, we can fix the duplicate warnings by updating `tokei` to the latest version, if possible, but the current latest on crates.io might not have it fixed.
Wait, we could set `multiple-versions = "allow"` in `deny.toml` since it's just a warning and not breaking the build, but wait, the prompt says "Improve release/governance hygiene in one coherent way." and "Target ranking 1) publish-plan/version-consistency drift 2) release metadata or changelog mismatch 3) RC-hardening docs/checks 4) low-risk release-surface fixes in workflows/docs/metadata".

Let's check target ranking 1: `cargo xtask version-consistency` passed.
Target ranking 2: release metadata or changelog mismatch.
Wait, let's look at `.github/workflows/release.yml`.
The docker image is built from `.` using `linux/amd64,linux/arm64`.
Let's see if there are any other issues.
What about `cargo xtask publish --plan --verbose`? It showed `cargo xtask docs --check` failed initially because `cargo deny` wasn't installed, but `cargo deny` is required for `cargo deny --all-features check`.
Wait, the fallback gate expectations:
- `cargo xtask publish --plan --verbose`
- `cargo xtask version-consistency`
- `cargo xtask docs --check`
- `cargo deny --all-features check` when manifests change

Wait, when I ran `cargo xtask docs --check` previously it passed. Wait, `cargo deny` is not part of the `xtask`! Wait, `cargo xtask docs --check` failed initially because of `deny`? No, wait:
```
     Running `target/debug/xtask version-consistency`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/xtask docs --check`
error: no such command: `deny`
```
Ah! `xtask docs --check` failed because it probably internally calls `cargo deny`?
No, the commands in the bash session were:
```bash
cargo xtask publish --plan --verbose
cargo xtask version-consistency
cargo xtask docs --check
cargo deny --all-features check
```
And the output:
```
...
     Running `target/debug/xtask version-consistency`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/xtask docs --check`
error: no such command: `deny`
```
Ah! The bash session concatenated the outputs! `cargo xtask docs --check` ran and succeeded, and then `cargo deny --all-features check` failed because `deny` was not an installed cargo command.

So `cargo deny` was just missing in my sandbox. I installed it and it gave warnings about duplicate versions, but exited with `ok`.

Let's look closely at `deny.toml`:
```toml
[[licenses.clarify]]
name = "ring"
expression = "MIT AND ISC AND OpenSSL"
license-files = [{ path = "LICENSE", hash = 0xbd0eed23 }]
```
Does `ring` still exist in our dependency tree?
```bash
cargo tree -p ring
```

Wait, `cargo tree -p ring` failed, meaning `ring` is no longer a dependency!
Let's check if there are other unused licenses in `deny.toml`.
Wait, the prompt memory says:
> When `cargo deny --all-features check` emits a `license-not-encountered` warning, it means an allowed license in the configuration file (e.g., `deny.toml`) is no longer used by any dependency in the workspace lockfile. Remove the unused license from the allowed list to resolve the warning.

Did `cargo deny` emit a `license-not-encountered` warning?
Wait, I piped the output of `cargo deny` and it didn't show the full output. Let's run it again and grep for `license-not-encountered` or just look at the full output.

Ah! I see `Unicode-DFS-2016` is a `license-not-encountered` warning. Let's see if there are any others, like `ring`.

The `cargo deny` check emits a `license-not-encountered` warning for `Unicode-DFS-2016` because it's in the allowed list but no longer used by any dependency in the workspace lockfile.
Additionally, we have a clarify rule for `ring` in `deny.toml`, but `ring` is no longer a dependency in the project (as `cargo tree -p ring` failed). So this clarify block is stale metadata.

Targeting this as a "publish-plan/version-consistency drift" or "release metadata mismatch" matches our target ranking 2 and 4. It fixes the `cargo deny` warnings which are governance checks.

Option A: Remove the `Unicode-DFS-2016` license allowance and the `ring` license clarification from `deny.toml`. This matches the memory instruction: "When `cargo deny --all-features check` emits a `license-not-encountered` warning, it means an allowed license in the configuration file (e.g., `deny.toml`) is no longer used by any dependency in the workspace lockfile. Remove the unused license from the allowed list to resolve the warning."

Option B: Do nothing and leave the warnings.

### Option A (recommended)
- what it is: Remove unused `Unicode-DFS-2016` from `licenses.allow` and remove the stale `licenses.clarify` block for `ring` in `deny.toml`.
- why it fits this repo and shard: It resolves `cargo deny` warnings about unencountered licenses, which are part of our governance gates. The `tooling-governance` shard explicitly covers `deny.toml` since it's governance configuration (part of workspace tooling).
- trade-offs:
  - Structure: Cleans up configuration.
  - Velocity: Fast and low-risk.
  - Governance: Ensures our governance checks run perfectly clean without warnings, avoiding "warning fatigue" for real issues.

### Option B
- what it is: Ignore the warnings.
- when to choose it instead: If the dependencies were temporarily removed but expected to be re-added shortly.
- trade-offs: We keep noisy warnings in our release gates.

## Decision
I will choose Option A because it aligns with our memory instructions, cleans up the governance configuration, and ensures `cargo deny` passes without noisy warnings.
