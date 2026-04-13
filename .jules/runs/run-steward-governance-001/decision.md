Option A: Fix `cargo xtask` by giving the xtask crate an explicitly named binary matching "xtask".
If `xtask` does not have an explicit `[[bin]]` section, Cargo uses the crate name, which is "xtask". But because `tokmd` crate specifies `default-run = "tokmd"`, and `xtask` is not in `default-members` but `tokmd` is, `cargo xtask` which alias to `run -p xtask --` does work! Wait. My earlier `cargo xtask version-consistency` command succeeded!

Wait! Let me re-read the first tool call's result.
```
cargo run --bin xtask -- version-consistency || cargo xtask version-consistency || echo "no version-consistency check found"
```
The result was:
```
error: no bin target named `xtask` in default-run packages

help: a target with a similar name exists: `tok`
help: available bin in `xtask` package:
    xtask
Checking version consistency against workspace version 1.9.0
...
Version consistency checks passed.
```

So `cargo run --bin xtask` failed, but `cargo xtask version-consistency` SUCCEEDED!
So there is no bug with `cargo xtask`.

Let's rethink: What is the issue?
"RC-hardening docs/checks"
Wait, memory says:
`The tokmd-fuzz crate intentionally stays at version 0.0.0 and does not participate in the workspace version. It requires explicit exclusion in release-aligned metadata checks, such as version-consistency tooling.`

Let's check `xtask/src/tasks/version_consistency.rs`:
```rust
        for pkg in &metadata.packages {
            if !workspace_member_ids.contains(&pkg.id) {
                continue;
            }
            // Skip non-publishable crates
            if pkg.publish.as_ref().is_some_and(|p| p.is_empty()) {
                continue;
            }
            if pkg.name == "xtask" || pkg.name == "tokmd-fuzz" || pkg.name == "fuzz" {
                continue;
            }

            let pkg_version = pkg.version.to_string();
            if pkg_version != workspace_version {
                inconsistent.push(format!("{} ({})", pkg.name, pkg_version));
            }
        }
```
`tokmd-fuzz` is ALREADY excluded in `version-consistency.rs`! Let's verify by looking at `version-consistency.rs` again.
Option A: There is no valid patch to make in the allowed paths that justifies a change. We explored target 1 (version consistency/publish plan), target 2 (changelog), target 3 (RC-hardening docs/checks) and target 4. All checks are passing, and version consistency explicitly ignores `tokmd-fuzz`. The only issue encountered was `cargo xtask` which is an alias in `.cargo/config.toml` that acts weirdly when a specific error arises because `tokmd` crate is the `default-run`. I will produce a learning PR explaining this, or propose to fix `xtask` Cargo.toml by explicitely creating `[[bin]] name="xtask"` to avoid user confusion, or create a `learning` PR without any change.
Option B: Update the changelog to specify `Unreleased` instead of leaving it empty? Actually, the changelog HAS an `Unreleased` section.
Wait, target 1: "publish-plan/version-consistency drift". I did not find any.
I will proceed with Option B: create a learning PR documenting that release and governance surfaces are fully consistent, and log the friction item regarding the confusing `cargo run --bin xtask` error due to `tokmd`'s `default-run` configuration causing Cargo to search for the `xtask` binary in the wrong package when the explicit `--package` argument is omitted.
