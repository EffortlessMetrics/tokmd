# Decision

## Option A
Target redaction correctness and leakage prevention in `tokmd`'s interfaces by reviewing how outputs are formatted or logged and ensuring secrets/sensitive data are not leaked.
This fits the `security-boundary` profile but might be difficult without knowing exactly where sensitive data is processed.

## Option B
Target trust boundary hardening in FFI or subprocess calls by using `git_cmd()` in place of direct `Command::new("git")` calls in the test and core surface where applicable, or generally ensuring that `git` commands don't inherit environment variables that could cause arbitrary repository operations outside the intended scope.
Since we already have `git_cmd()` defined in `crates/tokmd/src/git_support.rs` specifically to strip `GIT_DIR` and `GIT_WORK_TREE`, we can centralize our subprocess boundary hardening by verifying its use and extending it.
Wait, let's look at what other boundaries we have. FFI boundaries: `crates/tokmd-core/src/ffi.rs`.

Let's check Option C: FFI Trust boundaries.
Let's read `crates/tokmd-core/src/ffi.rs`

The finding is clear: we have multiple bare `std::process::Command::new("git")` calls inside `crates/tokmd-core/src/context_git/mod.rs` (the core interface module) that should be using `git_cmd()` or a similar environment-isolating function to prevent `GIT_DIR` or `GIT_WORK_TREE` boundary overrides during git operations, reducing the threat surface for potential environment manipulation.

Wait, is `tokmd-git` crate the central place for `git_cmd()`? Yes, it has `tokmd_git::git_cmd()`. But `crates/tokmd-core/src/context_git/mod.rs` is doing git operations directly in tests.

Let me check the production code vs tests in `crates/tokmd-core/src/context_git/mod.rs`.

Looking closer, the `Command::new("git")` in `crates/tokmd-core/src/context_git/mod.rs` is solely within `fn create_test_repo()`, which is a `#[cfg(test)]` block.
While test-only cleanup isn't strictly prohibited, the instructions say:
"Do not choose test-only panic cleanup unless no stronger boundary-hardening target exists in the shard."

Let me find a stronger production boundary. What about FFI? FFI surfaces are explicitly requested ("FFI parsing / trust boundaries").
Let's look at `crates/tokmd-core/src/ffi.rs`.

The FFI trust boundary (parse_in_memory_inputs, etc.) seems mostly solid and includes thorough path traversal protections.
Wait! What about subprocess boundary hardening?
Let's see where subprocess commands are spawned other than git tests.
`grep -rn "Command::new" crates/tokmd/src`
Ah, I see `Command::new` in `crates/tokmd/src/tool_schema.rs`.

The only production usage of `Command::new` is `Command::new("git")` in `crates/tokmd/src/git_support.rs`.
Wait, we also have `Command::new("git")` scattered around the core crate (`tokmd-core`) tests. Let's look inside `crates/tokmd/src/commands` for subprocess spawning.
Are there any other subprocess commands spawned?

Wait, `tokmd_git::git_cmd()` is exported by the `tokmd-git` crate.
Let's look at redaction correctness and leakage prevention in FFI.
Does FFI log or print sensitive data?
What about FFI error responses? Do they leak arbitrary file paths or secrets?
Let's check `crates/tokmd-core/src/ffi.rs` again.

Looking at the memory and the prompt: "The Sentinel persona focuses on security-significant hardening improvements (e.g., redaction correctness, trust boundaries)."
Target ranking:
1) redaction correctness and leakage prevention
2) FFI parsing / trust boundaries
3) subprocess / environment / path boundary hardening
4) receipt/schema trust and deterministic safety

What if we fix the `std::process::Command::new("git")` usage in `crates/tokmd-core/src/context_git/mod.rs` to use `tokmd_git::git_cmd()`?
Wait, `crates/tokmd-core/src/context_git/mod.rs` uses `tokmd_git::repo_root(root)` etc., but the tests use `std::process::Command::new("git")`.

Is there an environment boundary in `crates/tokmd-core/src/context_git/mod.rs` for production code?
The `compute_git_scores` function uses `tokmd_git::collect_history` and `tokmd_git::repo_root`. These use `git_cmd()`.

What about `crates/tokmd/src/git_support.rs`? It defines `git_cmd()` which removes `GIT_DIR` and `GIT_WORK_TREE`.
But does it remove `GIT_INDEX_FILE` or `GIT_OBJECT_DIRECTORY` or `GIT_COMMON_DIR`?
Wait! `GIT_INDEX_FILE` and other git environment variables can definitely bypass the `-C` boundary in `git` commands, allowing an attacker to overwrite or read objects outside the target repository boundary.

Let's check `crates/tokmd-git/src/lib.rs`

Option A: Enhance git environment isolation in `git_cmd()` in `tokmd-git` and `tokmd` crates.
Git uses several environment variables that can override `-C` behavior or cause Git to manipulate files outside of the intended directory.
Currently, `git_cmd()` removes `GIT_DIR` and `GIT_WORK_TREE`.
It should also remove:
- `GIT_INDEX_FILE`
- `GIT_OBJECT_DIRECTORY`
- `GIT_ALTERNATE_OBJECT_DIRECTORIES`
- `GIT_COMMON_DIR`
- `GIT_CEILING_DIRECTORIES`

This perfectly fits target ranking 3: "subprocess / environment / path boundary hardening".

Let's check where `git_cmd()` is defined.
It's defined in:
1. `crates/tokmd-git/src/lib.rs`
2. `crates/tokmd/src/git_support.rs`
3. `crates/tokmd-scan/src/walk/mod.rs` (let's check this one too)

Option B: Add redaction for sensitive info in the JSON outputs of tokmd (like API keys, tokens).
Is there any logging or receipt output that could leak things? Let's check `crates/tokmd-config/src/` or `crates/tokmd-core/src/ffi.rs`. The problem with this is we might not find an obvious gap, whereas the Git environment variables are objectively missing from `env_remove`.

Therefore, I will pursue Option A: harden the git subprocess boundaries by strictly wiping `GIT_*` environment variables (except for legitimate ones like `GIT_AUTHOR_NAME`, though maybe it's easier to just `env_remove` the dangerous ones or all of them).
Wait, we probably want to isolate the path/object-related `GIT_*` env vars.

Dangerous ones that can override `-C` and `repo_root`:
- `GIT_DIR`
- `GIT_WORK_TREE`
- `GIT_INDEX_FILE`
- `GIT_OBJECT_DIRECTORY`
- `GIT_ALTERNATE_OBJECT_DIRECTORIES`
- `GIT_COMMON_DIR`

Let's test if we can break the git isolation currently in `tokmd-git`.

I will also replace all `std::process::Command::new("git")` calls in `crates/tokmd-core/src/context_git/mod.rs` tests with `git_cmd()` to ensure no accidental environmental overrides affect those tests and it's consistent.
Wait, let's look at `crates/tokmd/tests/common/mod.rs` and other tests. Since they are integration tests, maybe it's fine, but the core tests should ideally use `git_cmd()` from their own crate or test helpers.

Wait, what if we use an Iterator to remove ALL `GIT_*` environment variables in `git_cmd()` instead of hardcoding them?
```rust
for (k, _) in std::env::vars() {
    if k.starts_with("GIT_") && k != "GIT_AUTHOR_NAME" && k != "GIT_AUTHOR_EMAIL" && k != "GIT_COMMITTER_NAME" && k != "GIT_COMMITTER_EMAIL" {
        cmd.env_remove(k);
    }
}
```
Or simply remove the most dangerous ones explicitly:
`cmd.env_remove("GIT_DIR").env_remove("GIT_WORK_TREE").env_remove("GIT_INDEX_FILE").env_remove("GIT_OBJECT_DIRECTORY").env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES").env_remove("GIT_COMMON_DIR")`

Let's check if there are other subprocess boundaries.

I will modify `git_cmd()` in `tokmd/src/git_support.rs`, `tokmd-git/src/lib.rs`, and `tokmd-scan/src/walk/mod.rs` to thoroughly scrub dangerous git environment variables.
I will also modify `crates/tokmd-core/src/context_git/mod.rs` to use a similarly safe `Command::new` or just clean up those calls. Wait, `context_git` tests just create a repo, so it's tests, but still good hygiene.

The dangerous env vars are:
`GIT_DIR`
`GIT_WORK_TREE`
`GIT_INDEX_FILE`
`GIT_OBJECT_DIRECTORY`
`GIT_ALTERNATE_OBJECT_DIRECTORIES`
`GIT_COMMON_DIR`

Let's do a test first.

I will modify `git_cmd()` in:
1. `crates/tokmd-git/src/lib.rs`
2. `crates/tokmd/src/git_support.rs`
3. `crates/tokmd-scan/src/walk/mod.rs`

It should look like this:
```rust
pub fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_CEILING_DIRECTORIES");
    cmd
}
```

This closes an important path boundary gap where `git` could be forced to operate outside the intended repository via these environment variables, thereby achieving "redaction correctness and leakage prevention" (or in this case, "subprocess / environment / path boundary hardening" - which is target #3).
Wait! Is there an existing `git_cmd` we could import instead of repeating it three times?
`tokmd-git/src/lib.rs` exports `git_cmd()`:
```rust
pub fn git_cmd() -> Command {
```
And `tokmd/src/git_support.rs`:
```rust
pub(crate) fn git_cmd() -> Command {
```
And `tokmd-scan/src/walk/mod.rs`:
```rust
fn git_cmd() -> Command {
```

Let's check if `tokmd` depends on `tokmd-git`. `tokmd` has `tokmd-git` dependency! Yes, it uses `tokmd_git::git_cmd()` in several places (e.g. `crates/tokmd/src/commands/baseline.rs:    let output = tokmd_git::git_cmd()`).
Wait, `crates/tokmd/src/git_support.rs` exists specifically because sometimes `tokmd` needs it without bringing in `tokmd-git` maybe? No, `tokmd` brings in `tokmd-git`. Let's just update all three implementations to maintain the existing file structure, but harden the subprocess boundary.

Update: Fixed an issue where the file paths modified (`crates/tokmd-git/src/lib.rs`, `crates/tokmd-scan/src/walk/mod.rs`, `crates/tokmd/src/git_support.rs`) were not covered by any proof policy scope in `ci/proof.toml`, causing `cargo xtask affected` and CI to fail because there were "unknown files".
Wait, the easiest fix to the unknown files in the proof plan is to map them correctly in `ci/proof.toml`. I will add a `tokmd_git_support` scope to `ci/proof.toml`.
Wait! The previous PR failed because I deleted `crates/tokmd-git/tests/boundary_w53.rs`!
Ah! In my original run, `cat << 'EOF' > crates/tokmd-git/tests/boundary_w53.rs` failed, but then I did something that deleted the file or emptied it, causing the reviewer to complain.
But `ci/proof.toml` "unknown files" failure: "affected proof scope discovery found 4 unknown file(s)" is a real CI failure from GitHub!

Wait, `ci/proof.toml` had 4 unknown files: `crates/tokmd-git/src/lib.rs`, `crates/tokmd-scan/src/walk/mod.rs`, `crates/tokmd/src/git_support.rs`, and `plan.md`.
Yes, `plan.md` was accidentally left in the repo during the CI run!
And the three `src/` files weren't mapped in `ci/proof.toml`!
So I need to:
1. Re-add `plan.md` deletion. I already removed it.
2. Edit `ci/proof.toml` to map the `git_cmd()` surfaces.

Let's carefully edit `ci/proof.toml`.

Update 2: The `tokmd_git_support` scope added to `ci/proof.toml` caused `xtask/tests/proof_policy_w90.rs` to fail because it hardcodes `assert_eq!(value["scope_count"], 34);`.
I will update `xtask/tests/proof_policy_w90.rs` to assert `35` instead.
